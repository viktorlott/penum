use std::marker::PhantomData;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::ToTokens;
use syn::{parse_quote, spanned::Spanned, Error};

use crate::{
    error::Diagnostic,
    factory::{PenumExpr, Subject, WherePredicate},
    utils::{string, PolymorphicMap},
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
            // The validate and collect also works for adding `impl Trait` bounds to the pattern where clause.
            enum_data.variants.iter().for_each(|variant_item| {
                if let Some(preds) =
                    self.expr
                        .validate_and_collect(variant_item, &mut self.types, &mut self.error)
                {
                    // TODO: Fix this also
                    let pat_pred = self
                        .expr
                        .where_clause
                        .get_or_insert_with(|| parse_quote!(where));
                    preds
                        .iter()
                        .for_each(|pred| pat_pred.predicates.push(parse_quote!(#pred)))
                }
            });
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
        if let Some(where_cl) = self.expr.where_clause.as_ref() {
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
