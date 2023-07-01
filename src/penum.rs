use std::marker::PhantomData;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::format_ident;
use quote::ToTokens;

use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Ident;
use syn::ItemImpl;
use syn::TraitBound;
use syn::TypeImplTrait;

use syn::parse_quote;
use syn::spanned::Spanned;
use syn::Error;
use syn::Type;

use crate::factory::PenumExpr;
use crate::factory::Subject;
use crate::factory::WherePredicate;

use crate::dispatch::VariantSig;
use crate::error::Diagnostic;

use crate::utils::create_impl_string;
use crate::utils::create_unique_ident;
use crate::utils::no_match_found;
use crate::utils::PolymorphicMap;
use crate::utils::UniqueHashId;

pub struct Disassembled;
pub struct Assembled;

type PolyMap = PolymorphicMap<UniqueHashId<Type>, UniqueHashId<Type>>;

/// Top level container type for Penum.
///
/// It contains everything we need to construct our dispatcher and
/// pattern validator.
pub struct Penum<State = Disassembled> {
    expr: PenumExpr,
    subject: Subject,
    error: Diagnostic,
    types: PolyMap,
    impls: Vec<ItemImpl>,
    _marker: PhantomData<State>,
}

impl Penum<Disassembled> {
    pub fn from(expr: PenumExpr, subject: Subject) -> Self {
        Self {
            expr,
            subject,
            error: Default::default(),
            types: Default::default(),
            impls: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn assemble(mut self) -> Penum<Assembled> {
        // NOTE: I might be using [field / parameter / argument] interchangeably.
        // - Field usually refers to a named variants
        // - Argument usually refers to unnamed variants
        // - Parameter usually refers to penum patterns (unnamed/named).
        let variants = &self.subject.get_variants();
        let enum_ident = &self.subject.ident;
        let error = &mut self.error;

        if !variants.is_empty() {
            // Expecting failure like `variant doesn't match shape`,
            // hence pre-calling.
            let pattern_fmt = self.expr.pattern_to_string();

            // The point is that as we check for equality, we also do
            // impl assertions by extending the `subjects` where clause.
            // This is something that we might want to change in the
            // future and instead use `spanned_quote` or some other
            // bound assertion.
            let mut predicates = Punctuated::<WherePredicate, Comma>::default();

            // Prepare our patterns by converting them into
            // `Comparables`. This is just a wrapper type that contains
            // commonly used props.
            let comparable_pats = self.expr.get_comparable_patterns();

            // We pre-check our clause because we might be needing this
            // during the dispatch step. Should add
            // `has_dispatchable_member` maybe? let has_clause =
            // self.expr.has_clause(); Turn into iterator instead?
            let mut maybe_blueprint_map = self.expr.get_blueprints_map(error);

            // For each variant:
            // 1. Validate its shape by comparing discriminant and
            //    unit/tuple/struct arity. (OUTER)
            //    - Failure: add a "no_match_found" error and continue
            //      to next variant.
            // 2. Validate each parameter    ...continue... (INNER)
            for (variant_ident, comp_item) in self.subject.comparable_fields_iter() {
                // FIXME: This only affects concrete types.. but
                //  `.compare(..)` should return a list of matches
                //  instead of just the first match it finds.
                //
                //  # Uni-matcher -> Multi-matcher
                //  Currently, we can end up returning a pattern that matches in shape, but not
                //  in structure, even though another pattern could satisfy our variant. In a case
                //  like the one below, we have a "catch all" variadic.
                //
                //  e.g. (i32, ..) | (..) => V1(String, i32), V2(String, String)
                //                              ^^^^^^           ^^^^^^
                //                              |                |
                //                              `Found 'String' but expected 'i32'`
                //
                //  Because the first pattern fragment contains a concrete type, it should be possible
                //  mark the error as temporary and then check for other pattern matches. Note, the first
                //  error should always be the default one.
                //
                //  Given our pattern above, `(..)` should be a fallback pattern.
                //
                //  Should we allow concrete types with trait bound at argument position?
                //  e.g.
                //    (i32: Trait,  ..) | (..)
                //    (i32: ^Trait, ..) | (..)
                //
                //  For future reference! This should help with dispach inference.
                //
                //  # "catch-all" syntax
                //  Given the example above, if we were to play with it a little, we could end up with
                //  something like this:
                //  `(i32, ..) | _` that translate to `(i32, ..) | (..) | {..}`
                //
                //  Maybe it's something that would be worth having considering something like this:
                //  `_ where String: ^AsRef<str>`

                // 1. Check if we match in `shape`
                let Some(matched_pair) = comparable_pats.compare(&comp_item) else {
                    let (span, message) = eor!(
                        comp_item.inner.is_empty(),
                            (variant_ident.span(), no_match_found(variant_ident, &pattern_fmt)),
                            (comp_item.inner.span(), no_match_found(comp_item.inner, &pattern_fmt))
                        );
                    self.error.extend(span, message);
                    continue
                };

                // No support for empty unit iter, yet...
                // NOTE: Make sure to handle composite::unit iterator before removing this
                if matched_pair.as_composite().is_unit() {
                    continue;
                }

                let arity = comp_item.inner.len();

                // 2. Check if we match in `structure`. (We are naively
                // always expecting to never have infixed variadics)
                for (field_index, (param_pattern, field_item)) in matched_pair.zip().enumerate() {
                    let item_ty_unique = field_item.ty.get_unique_id();

                    if param_pattern.is_infer() {
                        if let Some(blueprints) = maybe_blueprint_map.as_mut() {
                            let variant_sig = VariantSig::new(
                                enum_ident,
                                variant_ident,
                                field_item,
                                field_index,
                                arity,
                            );

                            blueprints.find_and_attach(
                                &item_ty_unique,
                                &variant_sig,
                                Some(&item_ty_unique),
                            );
                        }
                        self.types
                            .polymap_insert(item_ty_unique.clone(), item_ty_unique);

                        continue;
                    }

                    // If we cannot desctructure a pattern field, then it must be variadic.
                    //
                    // NOTE: This causes certain bugs (see tests/test-concrete-bound.rs)
                    let Some(pat_field) = param_pattern.get_field() else {
                        break;
                    };

                    // FIXME: Remove this, or refactor it. Remember that there's
                    // tests that needs to be removed/changed.
                    if let Some(ty_impl_trait) = pat_field.ty.get_type_impl_trait() {
                        let bounds = &ty_impl_trait.bounds;

                        let Some(impl_string) = create_impl_string(bounds, &mut self.error) else {
                            // FIXME: Add debug logs.
                            //
                            // No point of continuing if we have errors or
                            // unique_impl_id is empty
                            continue;
                        };

                        let unique_impl_id =
                            create_unique_ident(&impl_string, variant_ident, ty_impl_trait.span());

                        predicates.push(parse_quote!(#unique_impl_id: #bounds));

                        // First we check if pty (T) exists in polymorphicmap.
                        // If it exists, insert new concrete type.
                        self.types
                            .polymap_insert(unique_impl_id.clone().into(), item_ty_unique);
                    } else {
                        let pat_ty_unique = pat_field.ty.get_unique_id();

                        let variant_sig = VariantSig::new(
                            enum_ident,
                            variant_ident,
                            field_item,
                            field_index,
                            arity,
                        );

                        // Check if it's a generic or concrete type
                        // - We only accept `_|[A-Z][A-Z0-9]*` as generics.
                        //
                        // NOTE: `is_generic` is redundant given that we have already created the
                        // pat_ty_string.
                        if pat_field.ty.is_generic() {
                            // If the variant field is equal to the pattern field, then the variant
                            // field must be generic, therefore we should introduce a <gen> expr for
                            // the enum IF there doesn't exist one.
                            if item_ty_unique.eq(&pat_ty_unique) {
                                // Continuing means that we wont add T bounds to polymap
                                if let Some(blueprints) = maybe_blueprint_map.as_mut() {
                                    blueprints.find_and_attach(
                                        &pat_ty_unique,
                                        &variant_sig,
                                        Some(&item_ty_unique),
                                    );
                                };

                                self.types
                                    .polymap_insert(pat_ty_unique, item_ty_unique.clone());
                            } else {
                                if let Some(blueprints) = maybe_blueprint_map.as_mut() {
                                    for ty_unique in [&pat_ty_unique, &item_ty_unique] {
                                        blueprints.find_and_attach(
                                            ty_unique,
                                            &variant_sig,
                                            Some(&item_ty_unique),
                                        );
                                    }
                                };

                                for ty_unique in [pat_ty_unique, item_ty_unique.clone()] {
                                    self.types.polymap_insert(ty_unique, item_ty_unique.clone());
                                }
                            }

                            // FIXME: This will only work for nullary type constructors.
                        } else if pat_field.ty.is_placeholder() {
                            // Make sure we map the concrete type instead of the pat_ty
                            if let Some(blueprints) = maybe_blueprint_map.as_mut() {
                                blueprints.find_and_attach(
                                    &item_ty_unique,
                                    &variant_sig,
                                    Some(&item_ty_unique),
                                );
                            }
                            self.types
                                .polymap_insert(item_ty_unique.clone(), item_ty_unique);

                            // is concrete type equal to concrete type
                        } else if item_ty_unique.eq(&pat_ty_unique) {
                            // 3. Dispachable list
                            if let Some(blueprints) = maybe_blueprint_map.as_mut() {
                                blueprints.find_and_attach(
                                    &item_ty_unique,
                                    &variant_sig,
                                    Some(&item_ty_unique),
                                );
                            }

                            self.types.polymap_insert(
                                pat_ty_unique, // PATTERN
                                item_ty_unique,
                            );
                        } else {
                            // TODO: Refactor into TypeId instead.
                            let item_ty_string = field_item.ty.get_string();
                            // NOTE: This string only contains the Ident, so any generic parameters will
                            // be discarded.
                            let pat_ty_string = pat_field.ty.get_string();

                            self.error.extend_spanned(
                                &field_item.ty,
                                format!("Found `{item_ty_string}` but expected `{pat_ty_string}`."),
                            );
                        }
                    }
                }
            }

            // Assemble all our impl statements
            if let Some(blueprints) = maybe_blueprint_map {
                let (impl_generics, ty_generics, where_clause) =
                    &self.subject.generics.split_for_impl();

                blueprints.for_each_blueprint(|blueprint| {
                    let trait_path = blueprint.get_sanatized_impl_path();
                    let assoc_methods = blueprint.get_associated_methods();

                    let assoc_types = blueprint.get_mapped_bindings().map(|bind| {
                        bind.iter()
                            .map(|b| b.to_token_stream())
                            .collect::<TokenStream2>()
                    });

                    let implementation: ItemImpl = parse_quote!(
                        impl #impl_generics #trait_path for #enum_ident #ty_generics #where_clause {
                            #assoc_types

                            #(#assoc_methods)*
                        }
                    );

                    self.impls.push(implementation);
                });
            }

            let penum_expr_clause = self.expr.clause.get_or_insert_with(|| parse_quote!(where));

            // Might be a little unnecessary to loop through our
            // predicates again.. But we can refactor later.
            predicates
                .iter()
                .for_each(|pred| penum_expr_clause.predicates.push(parse_quote!(#pred)));
        } else {
            self.error.extend(
                self.subject.ident.span(),
                "Expected to find at least one variant.",
            );
        }

        // SAFETY: We are transmuting self into self with a different
        //         ZST marker that is just there to let us decide what
        //         methods should be available during different stages.
        //         So it's safe for us to transmute.
        unsafe { std::mem::transmute(self) }
    }
}

impl Penum<Assembled> {
    // NOTE: This is only used for unit tests
    #[allow(dead_code)]
    pub fn get_tokenstream(mut self) -> TokenStream2 {
        self.attach_assertions();
        if self.error.has_error() {
            self.error.map(Error::to_compile_error).unwrap()
        } else {
            let enum_item = self.subject;
            let impl_items = self.impls;

            let output = quote::quote!(#enum_item #(#impl_items)*);

            output
        }
    }

    pub fn unwrap_or_error(mut self) -> TokenStream {
        self.attach_assertions();

        self.error
            .map(Error::to_compile_error)
            .unwrap_or_else(|| {
                let enum_item = self.subject;
                let impl_items = self.impls;
                let output = quote::quote!(#enum_item #(#impl_items)*);

                output
            })
            .into()
    }

    pub(self) fn attach_assertions(&mut self) {
        if let Some(where_cl) = self.expr.clause.as_ref() {
            for (_, predicate) in where_cl.predicates.iter().enumerate() {
                match predicate {
                    WherePredicate::Type(pred) => {
                        let id = pred.bounded_ty.get_unique_id();

                        if let Some(pty_set) = self.types.get(&id) {
                            for (_, ty_id) in pty_set.iter().enumerate() {
                                let ty = &**ty_id;

                                // Could remove this.
                                let spanned_bounds = pred
                                    .bounds
                                    .to_token_stream()
                                    .into_iter()
                                    .map(|mut token| {
                                        // NOTE: This is the only way we can
                                        // impose a new span for a `bound`..
                                        // FIXES: tests/ui/placeholder_with_bound.rs
                                        // FIXES: tests/ui/trait-bound-not-satisfied.rs
                                        token.set_span(ty.span());
                                        token
                                    })
                                    .collect::<TokenStream2>();

                                self.subject
                                    .generics
                                    .make_where_clause()
                                    .predicates
                                    .push(parse_quote! {#ty: #spanned_bounds})
                            }
                        }
                    }
                    WherePredicate::Lifetime(pred) => self
                        .error
                        .extend(pred.span(), "lifetime predicates are unsupported"),
                }
            }
        }
    }
}

pub trait Stringify: ToTokens {
    fn get_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}

pub trait TraitBoundUtils {
    fn get_unique_trait_bound_id(&self) -> String;
}

pub trait TypeUtils {
    fn is_generic(&self) -> bool;
    fn is_placeholder(&self) -> bool;
    fn some_generic(&self) -> Option<String>;
    fn get_generic_ident(&self) -> Ident;
    fn get_unique_id(&self) -> UniqueHashId<Type>;
    fn get_type_impl_trait(&self) -> Option<&TypeImplTrait>;
}

impl<T> Stringify for T where T: ToTokens {}

impl TypeUtils for Type {
    fn get_type_impl_trait(&self) -> Option<&TypeImplTrait> {
        if let Type::ImplTrait(ref ty_impl_trait) = self {
            Some(ty_impl_trait)
        } else {
            None
        }
    }

    fn is_generic(&self) -> bool {
        let pat_ty_string = self.to_token_stream().to_string();
        !self.is_placeholder() && pat_ty_string.to_uppercase().eq(&pat_ty_string)
    }

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

    /// Only use this when you are sure it's a generic type.
    fn get_generic_ident(&self) -> Ident {
        format_ident!("{}", self.get_string(), span = self.span())
    }

    fn get_unique_id(&self) -> UniqueHashId<Type> {
        UniqueHashId::new(self)
    }
}

impl TraitBoundUtils for TraitBound {
    fn get_unique_trait_bound_id(&self) -> String {
        UniqueHashId(self).get_unique_string()
    }
}

macro_rules! eor {
    ($x:expr, $left:expr, $right:expr) => {
        if $x {
            $left
        } else {
            $right
        }
    };
}

pub(self) use eor;

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use syn::{parse_quote, ItemTrait};

    use crate::{
        dispatch::T_SHM,
        factory::{PenumExpr, Subject},
        penum::{Penum, Stringify},
    };

    fn penum_assertion(attr: TokenStream, input: TokenStream, expect: TokenStream) {
        let pattern: PenumExpr = parse_quote!( #attr );
        let input: Subject = parse_quote!( #input );

        let penum = Penum::from(pattern, input)
            .assemble()
            .get_tokenstream()
            .to_string();

        assert_eq!(penum, expect.to_string());
    }

    fn register_trait(input: TokenStream) {
        let item_trait: ItemTrait = parse_quote!(#input);
        // If we cannot find the trait the user wants to dispatch, we need to store it.
        T_SHM.insert(item_trait.ident.get_string(), item_trait.get_string());
    }

    #[test]
    #[rustfmt::skip]
    fn simple_expression() {
        let attr = quote::quote!(
            (T) where T: Trait
        );
        
        let input = quote::quote!(
            enum Enum {
                V1(i32),
                V2(usize),
                V3(String)
            }
        );

        let expect = quote::quote!(
            enum Enum
            where
                usize: Trait,
                String: Trait,
                i32: Trait
            {
                V1(i32),
                V2(usize),
                V3(String)
            }
        );

        penum_assertion(attr, input, expect);
    }

    #[test]
    #[rustfmt::skip]
    fn dispatch_std_trait() {
        let attr = quote::quote!(
            (T) where T: ^AsRef<str>
        );

        let input = quote::quote!(
            enum Enum {
                V1(String),
            }
        );

        let expect = quote::quote!(
            enum Enum where String: AsRef<str> {
                V1(String),
            }

            impl AsRef<str> for Enum {
                fn as_ref(&self) -> &str {
                    match self {
                        Enum::V1(val) => val.as_ref(),
                        _ => ""
                    }
                }
            }
        );

        penum_assertion(attr, input, expect);
    }

    #[test]
    #[rustfmt::skip]
    fn dispatch_custom_trait() {
        let blueprint = quote::quote!(
            trait Abc {
                type Input;
                fn get(&self) -> &Self::Input;
            }
        );

        let attr = quote::quote!(
            (T) where T: ^Abc<Input = str>
        );

        let input = quote::quote!(
            enum Enum {
                V1(String),
                V2(String)
            }
        );

        let expect = quote::quote!(
            enum Enum where String: Abc<Input = str> {
                V1(String),
                V2(String)
            }

            impl Abc<Input = str> for Enum {
                type Input = str;
                fn get(&self) -> &Self::Input {
                    match self {
                        Enum::V1(val) => val.get(),
                        Enum::V2(val) => val.get(),
                        _ => panic!("Missing arm")
                    }
                }
            }
        );

        register_trait(blueprint);
        penum_assertion(attr, input, expect);
    }

    #[test]
    #[rustfmt::skip]
    fn dispatch_custom_trait_with_impl_expression() {
        let blueprint = quote::quote!(
            trait Abc {
                type Input;
                fn get(&self) -> &Self::Input;
            }
        );

        let attr = quote::quote!(
            impl Abc<Input = str> for String
        );

        let input = quote::quote!(
            enum Enum {
                V1(String, i32),
                V2(i32, String)
            }
        );

        let expect = quote::quote!(
            enum Enum where String: Abc<Input = str> {
                V1(String, i32),
                V2(i32, String)
            }

            impl Abc<Input = str> for Enum {
                type Input = str;
                fn get(&self) -> &Self::Input {
                    match self {
                        Enum::V1(val, ..) => val.get(),
                        Enum::V2(_, val) => val.get(),
                        _ => panic!("Missing arm")
                    }
                }
            }
        );

        register_trait(blueprint);
        penum_assertion(attr, input, expect);
    }

    // TODO: Decide how variadics should be interpreted when we have concrete type bounds.
    // Make sure to update `tests/test-concrete-bound.rs` if this later gets supported.
}
