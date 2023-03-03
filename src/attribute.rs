#![allow(irrefutable_let_patterns)]
use std::marker::PhantomData;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::format_ident;
use quote::ToTokens;
use syn::{
    parse_quote,
    spanned::Spanned,
    WherePredicate,
};

use crate::{
    error::ErrorStash,
    subject::Subject,
    utils::{string, PatternTypePairs},
    shape::Shape
};

pub struct Initialized;
pub struct Matched;

pub struct EnumShape<State = Initialized> {
    pub shape: Shape,
    pub input: Subject,
    pub error: ErrorStash,
    pub types: PatternTypePairs,
    _marker: PhantomData<State>,
}

impl EnumShape<Initialized> {
    pub fn new(shape: Shape, input: Subject) -> Self {
        Self {
            shape,
            input,
            error: Default::default(),
            types: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn matcher(mut self) -> EnumShape<Matched> {
        let enum_data = &self.input.data;
        if self.input.data.variants.is_empty() {
            self.error.extend(
                enum_data.variants.span(),
                "Expected to find at least one variant.",
            );
        }

        for variant_item in enum_data.variants.iter() {
            self.shape
                .validate_and_collect(variant_item, &mut self.types, &mut self.error);
        }

        // SAFETY: Transmuting Self into Self with a different ZST is safe.
        unsafe { std::mem::transmute(self) }
    }
}

impl EnumShape<Matched> {
    pub fn unwrap_or_error(mut self) -> TokenStream {
        let bound_tokens = link_bounds(&mut self);

        self.error
            .map(|err| err.to_compile_error())
            .unwrap_or_else(|| {
                extend_where_clause(&mut self, &bound_tokens);
                self.input.to_token_stream()
            })
            .into()
    }
}

fn link_bounds(enum_shape: &mut EnumShape<Matched>) -> Vec<TokenStream2> {
    let mut bound_tokens = Vec::new();
    if let Some(where_cl) = enum_shape.shape.where_clause.as_ref() {
        for predicate in where_cl.predicates.iter() {
            match predicate {
                WherePredicate::Type(pred) => {
                    if let Some(pty_set) = enum_shape.types.get(&string(&pred.bounded_ty)) {
                        pty_set
                            .iter()
                            .map(|ident| (format_ident!("{}", ident), &pred.bounds))
                            .for_each(|(ident, bound)| {
                                bound_tokens.push(parse_quote!(#ident: #bound))
                            })
                    }
                }
                _ => enum_shape
                    .error
                    .extend(Span::call_site(), "Unsupported `where clause`"),
            }
        }
    }
    bound_tokens
}

fn extend_where_clause(enum_shape: &mut EnumShape<Matched>, bounds: &[TokenStream2]) {
    bounds.iter().for_each(|bound| {
        enum_shape
            .input
            .generics
            .where_clause
            .get_or_insert_with(|| parse_quote!(where))
            .predicates
            .push(parse_quote!(#bound))
    })
}
