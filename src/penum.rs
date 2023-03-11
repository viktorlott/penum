use std::marker::PhantomData;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Type;
use syn::{parse_quote, spanned::Spanned, Error};

use crate::factory::ComparablePatterns;

use crate::{
    error::Diagnostic,
    factory::{PenumExpr, Subject, WherePredicate},
    utils::{ident_impl, no_match_found, string, PolymorphicMap},
};

pub struct Disassembled;
pub struct Assembled;

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

    pub fn assemble(mut self) -> Penum<Assembled> {
        let enum_data = &self.subject.data;

        if !enum_data.variants.is_empty() {
            // Prepare our patterns by converting them into `Comparables`.
            let comparable_patterns = ComparablePatterns::from(&self.expr);

            // Might change this later, but the point is that as we check for equality, we also do impl assertions
            // by extending the `subjects` where clause. This is something that we might want to change in the future
            // and instead use `spanned_quote` or some other bound assertion.
            let mut predicates: Punctuated<WherePredicate, Comma> = Default::default();

            // Expecting failure, hence pre-calling
            let pattern_fmt = self.expr.pattern_to_string();

            // For each variant => check if it matches a specified pattern
            for comp_item in self.subject.get_comparable_fields() {
                // 1. Check if we match in `shape`
                let Some(matched_pair) = comparable_patterns.compare(&comp_item) else {
                    self.error.extend(comp_item.value.span(), no_match_found(comp_item.value, &pattern_fmt));
                    continue
                };

                // No support for empty unit iter, yet...
                // NOTE: Make sure to handle composite::unit iterator before removing this
                if matched_pair.as_composite().is_unit() {
                    continue;
                }

                // 2. Check if we match in `structure`. (We are naively always expecting to never have infixed variadics)
                for (pat_param, item_field) in matched_pair.zip() {
                    // If we cannot desctructure a pattern field, then it must be variadic.
                    // This might change later
                    let Some(pat_field) = pat_param.get_field() else {
                        break;
                    };

                    let item_ty = string(&item_field.ty);

                    // Check for impl expressions, `(impl Trait, T)`.
                    if let Type::ImplTrait(imptr) = &pat_field.ty {
                        // We use a `dummy` identifier to store our bound under.
                        let tty = ident_impl(imptr);
                        let bounds = &imptr.bounds;

                        predicates.push(parse_quote!(#tty: #bounds));

                        // First we check if pty (T) exists in polymorphicmap.
                        // If it exists, insert new concrete type.
                        self.types.polymap_insert(tty.to_string(), item_ty)
                    } else {
                        // Check if we are generic or concrete type.
                        let pat_ty = string(&pat_field.ty);
                        let is_generic = pat_ty.eq("_") || pat_ty.to_uppercase().eq(&pat_ty);

                        // If pattern type is concrete, make sure it matches item type
                        if !is_generic && pat_ty != item_ty {
                            self.error.extend(
                                item_field.ty.span(),
                                format!("Found {item_ty} but expected {pat_ty}."),
                            );
                            continue;
                        } else {
                            // First we check if pty (T) exists in polymorphicmap.
                            // If it exists, insert new concrete type.
                            self.types.polymap_insert(pat_ty, item_ty);
                        }
                    }
                }
            }

            // The validate and collect also works for adding `impl Trait` bounds to the pattern where clause.
            let pat_pred = self.expr.clause.get_or_insert_with(|| parse_quote!(where));

            predicates
                .iter()
                .for_each(|pred| pat_pred.predicates.push(parse_quote!(#pred)))
        } else {
            self.error.extend(
                enum_data.variants.span(),
                "Expected to find at least one variant.",
            );
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
