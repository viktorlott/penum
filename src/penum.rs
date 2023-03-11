use std::marker::PhantomData;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Type;
use syn::{parse_quote, spanned::Spanned, Error};

use crate::factory::{insert_polymap, pattern_match};

use crate::{
    error::Diagnostic,
    factory::{PenumExpr, Subject, WherePredicate},
    utils::{no_match_found, string, PolymorphicMap},
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

        if enum_data.variants.is_empty() {
            self.error.extend(
                enum_data.variants.span(),
                "Expected to find at least one variant.",
            );
        } else { 
            let comparable_patterns = self.expr.get_comparable_patterns();
            let comparable_fields_iter = self.subject.get_comparable_fields();

            let mut predicates: Punctuated<WherePredicate, Comma> = Default::default();

            // Expecting failure, hence pre-calling
            let pattern_fmt = &self.expr.pattern_to_string();

            for comp_item in comparable_fields_iter {
                let Some((group, ifields)) = comparable_patterns.iter().find_map(pattern_match(&comp_item)).map(Into::into) else {
                    self.error.extend(comp_item.value.span(), no_match_found(comp_item.value, pattern_fmt));
                    continue
                };

                // TODO: Before removing this, make sure to check `Group.iter()` function!
                // No support for empty unit iter, yet...
                if group.is_unit() {
                    continue;
                }

                for (pat, item) in group.into_iter().zip(ifields.into_iter()) {
                    // If we cannot desctructure a pattern field, then it must be variadic.
                    let Some(pfield) = pat.get_field() else {
                        break;
                    };

                    let ity = string(&item.ty);

                    // Check if we have a impl statement, `(impl Trait, T)`.
                    if let Type::ImplTrait(imptr) = &pfield.ty {
                        // TODO: Fix placeholder ident
                        let tty = format_ident!(
                            "__IMPL_{}",
                            string(&imptr.bounds)
                                .replace(' ', "_")
                                .replace(['?', '\''], "")
                        );
                        let bounds = &imptr.bounds;
                        predicates.push(parse_quote!(#tty: #bounds));

                        let pty = tty.to_string();
                        // First we check if pty (T) exists in polymorphicmap.
                        // If it exists, insert new concrete type.
                        insert_polymap(&mut self.types, pty, ity);
                    } else {
                        // Check if we are generic or concrete type.
                        let pty = string(&pfield.ty);
                        let is_generic = pty.eq("_") || pty.to_uppercase().eq(&pty);

                        // If pattern type is concrete, make sure it matches item type
                        if !is_generic && pty != ity {
                            self.error
                                .extend(item.ty.span(), format!("Found {ity} but expected {pty}."));
                            continue;
                        } else {
                            // First we check if pty (T) exists in polymorphicmap.
                            // If it exists, insert new concrete type.
                            insert_polymap(&mut self.types, pty, ity);
                        }
                    }
                }
            }

            // The validate and collect also works for adding `impl Trait` bounds to the pattern where clause.
            let pat_pred = self.expr.clause.get_or_insert_with(|| parse_quote!(where));
            predicates
                .iter()
                .for_each(|pred| pat_pred.predicates.push(parse_quote!(#pred)))
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
