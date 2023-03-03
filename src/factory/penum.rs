use std::marker::PhantomData;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::format_ident;
use quote::ToTokens;
use syn::{parse_quote, spanned::Spanned, WherePredicate, Error};

use crate::{
    error::Diagnostic,
    utils::{string, TypeMap},
    factory::{pattern::Pattern, subject::Subject}
};

pub struct Disassembled;
pub struct Assembled;

pub struct Penum<State = Disassembled> {
    pub pattern: Pattern,
    pub subject: Subject,
    pub error: Diagnostic,
    pub types: TypeMap,
    _marker: PhantomData<State>,
}

impl Penum<Disassembled> {
    pub fn from(pattern: Pattern, subject: Subject) -> Self {
        Self {
            pattern,
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
            enum_data.variants
                .iter()
                .for_each(|variant_item| self.pattern
                    .validate_and_collect(variant_item, &mut self.types, &mut self.error)
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
        if let Some(where_cl) = self.pattern.where_clause.as_ref() {
            for predicate in where_cl.predicates.iter() {
                match predicate {
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
                    _ => self
                        .error
                        .extend(Span::call_site(), "Unsupported `where clause`"),
                }
            }
        }
        bound_tokens
    }
}

