use std::borrow::Borrow;

use std::marker::PhantomData;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::ToTokens;

use syn::punctuated::Punctuated;
use syn::token::Comma;

use syn::Type;
use syn::{parse_quote, spanned::Spanned, Error};

use crate::{
    dispatch::VariantSignature,
    error::Diagnostic,
    factory::ComparablePats,
    factory::{PenumExpr, Subject, WherePredicate},
    utils::{no_match_found, string, PolymorphicMap},
};

pub struct Disassembled;
pub struct Assembled;

/// Top level container type for Penum.
///
/// It contains everything we need
pub struct Penum<State = Disassembled> {
    pub expr: PenumExpr,
    pub subject: Subject,
    pub error: Diagnostic,
    pub types: PolymorphicMap,
    _marker: PhantomData<State>,
}

impl Penum<Disassembled> {
    pub fn from(expr: PenumExpr, subject: Subject) -> Self {
        Self {
            expr,
            subject,
            error: Default::default(),
            types: Default::default(),
            _marker: Default::default(),
        }
    }

    /// I am using [field / parameter / argument] interchangeably
    pub fn assemble(mut self) -> Penum<Assembled> {
        let variants = &self.subject.get_variants();
        let enum_ident = &self.subject.ident;

        if !variants.is_empty() {
            // The point is that as we check for equality, we also do impl assertions
            // by extending the `subjects` where clause. This is something that we might
            // want to change in the future and instead use `spanned_quote` or some other
            // bound assertion.
            let mut predicates: Punctuated<WherePredicate, Comma> = Default::default();

            // Prepare our patterns by converting them into `Comparables`. This is just a wrapper type
            // that contains commonly used props.
            let comparable_pats: ComparablePats = self.expr.borrow().into();

            // We pre-check for clause because we might be needing this during the dispatch step.
            // Should add `has_dispatchable_member` maybe? let has_clause = self.expr.has_clause();
            // Turn into iterator instead?
            let mut blueprints = self.expr.get_blueprints();

            // Expecting failure like `variant doesn't match shape`, hence pre-calling.
            let pattern_fmt = self.expr.pattern_to_string();

            // For each variant:
            // 1. Validate its shape by comparing discriminant and unit/tuple/struct arity. (OUTER)
            //    - Failure: add a "no_match_found" error and continue to next variant.
            // 2. Validate each parameter    ...continue...                                 (INNER)
            for (variant_ident, comp_item) in self.subject.get_comparable_fields() {
                // FIXME: This only affects concrete types.. but `.compare(..)` should return a list of
                //        matches instead of just the first match it finds.
                //
                //        # Uni-matcher -> Multi-matcher
                //        Currently, we can end up returning a pattern that matches in shape, but not
                //        in structure, even though another pattern could satisfy our variant. In a case
                //        like the one below, we have a "catch all" variadic.
                //
                //        e.g. (i32, ..) | (..) => V1(String, i32), V2(String, String)
                //                                    ^^^^^^           ^^^^^^
                //                                    |                |
                //                                    `Found 'String' but expected 'i32'`
                //
                //        Because the first pattern fragment contains a concrete type, it should be possible
                //        mark the error as temporary and then check for other pattern matches. Note, the first
                //        error should always be the default one.
                //
                //        Given our pattern above, `(..)` should be a fallback pattern.
                //
                //        Should we allow concrete types with trait bound at argument position?
                //        e.g.
                //          (i32: Trait,  ..) | (..)
                //          (i32: ^Trait, ..) | (..)
                //
                //        For future reference! This should help with dispach inference.
                //
                //        # "catch-all" syntax
                //        Given the example above, if we were to play with it a little, we could end up with
                //        something like this:
                //        `(i32, ..) | _` that translate to `(i32, ..) | (..) | {..}`
                //
                //        Maybe it's something that would be worth having considering something like this:
                //        `_ where String: ^AsRef<str>`

                // 1. Check if we match in `shape`
                let Some(matched_pair) = comparable_pats.compare(&comp_item) else {
                    self.error.extend(comp_item.value.span(), no_match_found(comp_item.value, &pattern_fmt));
                    continue
                };

                // No support for empty unit iter, yet...
                // NOTE: Make sure to handle composite::unit iterator before removing this
                if matched_pair.as_composite().is_unit() {
                    continue;
                }

                let max_fields_len = comp_item.value.len();

                // 2. Check if we match in `structure`.
                // (We are naively always expecting to never have infixed variadics)
                for (_index_param, (pat_parameter, item_field)) in matched_pair.zip().enumerate() {
                    // If we cannot desctructure a pattern field, then it must be variadic.
                    // This might change later
                    let Some(pat_field) = pat_parameter.get_field() else {
                        break;
                    };

                    let item_ty = item_field.ty.get_string();

                    if let Type::ImplTrait(ref ty_impl_trait) = pat_field.ty {
                        // FIXME: SUPPORT DISPATCHING FROM `impl Trait` expressions. e.g (impl ^Trait) | (_, _)
                        //
                        // Should we infer dispatching also?
                        let id = match ty_impl_trait.bounds.first().unwrap() {
                            syn::TypeParamBound::Trait(t) => {
                                t.path.segments.last().unwrap().ident.get_string()
                            }
                            syn::TypeParamBound::Lifetime(_) => continue,
                        };

                        // We use a `dummy` identifier to store our bound under.
                        // let tty = ident_impl(ty_impl_trait);
                        let tty = format_ident!("_IMPL_{id}");
                        let bounds = &ty_impl_trait.bounds;

                        predicates.push(parse_quote!(#tty: #bounds));

                        // First we check if pty (T) exists in polymorphicmap.
                        // If it exists, insert new concrete type.
                        self.types.polymap_insert(tty.to_string(), item_ty);

                        continue;
                    }

                    let pat_ty = pat_field.ty.get_string();
                    // Check if it's a generic or concrete type
                    // - We only accept `_|[A-Z][A-Z0-9]*` as generics.
                    let is_generic =
                        pat_field.ty.is_placeholder() || pat_ty.to_uppercase().eq(&pat_ty);

                    if is_generic {
                        // First we check if pty (T) exists in polymorphicmap.
                        // If it exists, insert new concrete type.
                        self.types.polymap_insert(pat_ty.clone(), item_ty);

                        // 3. Dispachable list
                        let Some(blueprints) = blueprints.as_mut().and_then(|bp| bp.get_mut(&pat_ty)) else {
                            continue
                        };

                        let variant_sig = VariantSignature::new(
                            enum_ident,
                            variant_ident,
                            item_field,
                            max_fields_len,
                        );

                        // FIXME: We are only expecting one dispatch per generic now, so CHANGE THIS WHEN POSSIBLE:
                        //        - where T: ^Trait, T: ^Mate -> only ^Trait will be found. :( Fixed?
                        //        - where T: ^Trait + ^Mate   -> should be just turn this into a poly map instead?
                        //
                        // where T: ^Trait + ^Mate, T: ^Fate, T: ^Mate turns into T => [^Trait, ^Mate, ^Fate]

                        // I had to use a vec instead because of partial ordering not being implemented for TraitBound
                        for blueprint in blueprints.iter_mut() {
                            blueprint.attatch(&variant_sig)
                        }
                    } else if item_ty.ne(&pat_ty) {
                        self.error.extend(
                            item_field.ty.span(),
                            format!("Found {item_ty} but expected {pat_ty}."),
                        );
                    }
                }
            }

            // [Generic]
            //     [Trait]
            //         [Method]
            //             [Arm -> dispatch]
            if let Some(blueprints) = blueprints {
                for (ident, blueprints) in blueprints.iter() {
                    println!("{}", ident);

                    blueprints.iter().for_each(|blueprint| {
                        println!("|-{}", blueprint.bound.path.to_token_stream());
                        blueprint.methods.iter().for_each(|arm| {
                            println!("  |- {}", arm.0);
                            arm.1
                                .iter()
                                .for_each(|ar| println!("     |- {}", ar.to_token_stream()));
                        })
                    })
                }
            }

            // FIXME: Instead of extending the enums where clause with predicate assertion, use spanned_quote
            // Extend our expr where clause with `impl Trait` bounds if found. (predicates)
            let penum_expr_clause = self.expr.clause.get_or_insert_with(|| parse_quote!(where));

            predicates
                .iter()
                .for_each(|pred| penum_expr_clause.predicates.push(parse_quote!(#pred)));

            // Might be a little unnecessary to loop through our predicates again.. But we can refactor later.
            // penum_expr_clause.predicates.iter().filter_map(f)
        } else {
            self.error
                .extend(variants.span(), "Expected to find at least one variant.");
        }

        // SAFETY: Transmuting Self into Self with a different ZST is safe.
        unsafe { std::mem::transmute(self) }
    }
}

impl Penum<Assembled> {
    pub fn unwrap_or_error(mut self) -> TokenStream {
        let bound_tokens = self.link_bounds();

        self.error
            .map(Error::to_compile_error)
            .unwrap_or_else(|| self.extend_enum_with(&bound_tokens).to_token_stream())
            .into()
    }

    fn extend_enum_with(&mut self, bounds: &[TokenStream2]) -> &Subject {
        bounds.iter().for_each(|bound| {
            self.subject
                .generics
                .where_clause
                .get_or_insert_with(|| parse_quote!(where))
                .predicates
                .push(parse_quote!(#bound))
        });

        &self.subject
    }

    fn link_bounds(self: &mut Penum<Assembled>) -> Vec<TokenStream2> {
        let mut bound_tokens = Vec::new();

        if let Some(where_cl) = self.expr.clause.as_ref() {
            where_cl
                .predicates
                .iter()
                .for_each(|predicate| match predicate {
                    WherePredicate::Type(pred) => {
                        if let Some(pty_set) = self.types.get(&string(&pred.bounded_ty)) {
                            pty_set
                                .iter()
                                .map(|ident| (format_ident!("{}", ident), &pred.bounds))
                                .for_each(|(ident, bound)| {
                                    bound_tokens.push(parse_quote!(#ident: #bound))
                                })
                        }
                    }
                    WherePredicate::Lifetime(pred) => self
                        .error
                        .extend(pred.span(), "lifetime predicates are unsupported"),
                })
        }
        bound_tokens
    }
}

pub trait Stringify: ToTokens {
    fn get_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}

pub trait TypeUtils {
    fn is_placeholder(&self) -> bool;
    fn some_generic(&self) -> Option<String>;
}

impl<T> Stringify for T where T: ToTokens {}

impl TypeUtils for Type {
    fn is_placeholder(&self) -> bool {
        matches!(self, Type::Infer(_))
    }

    fn some_generic(&self) -> Option<String> {
        self.is_placeholder()
            .then(|| {
                let pat_ty = self.get_string();
                pat_ty.to_uppercase().eq(&pat_ty).then_some(pat_ty)
            })
            .flatten()
    }
}
