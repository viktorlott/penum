use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{AttrStyle, Attribute};

use super::*;

impl ToTokens for Strukt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.attrs
            .iter()
            .filter(is_inner)
            .for_each(|attr| attr.to_tokens(tokens));

        self.vis.to_tokens(tokens);

        self.data.struct_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.generics.where_clause.to_tokens(tokens);

        match &self.data.fields {
            FieldsKind::Named(fields) => {
                self.generics.where_clause.to_tokens(tokens);
                fields.to_tokens(tokens);
            }
            FieldsKind::Unnamed(fields) => {
                fields.to_tokens(tokens);
                self.generics.where_clause.to_tokens(tokens);
                TokensOrDefault(&self.data.semi_token).to_tokens(tokens);
            }
            FieldsKind::Unit => {
                self.generics.where_clause.to_tokens(tokens);
                TokensOrDefault(&self.data.semi_token).to_tokens(tokens);
            }
        };
    }
}

impl ToTokens for FieldDisc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.attrs
            .iter()
            .filter(is_outer)
            .for_each(|attr| attr.to_tokens(tokens));

        self.vis.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);

        self.ty.to_tokens(tokens);

        // Skip discriminant
        // if let Some((eq_token, disc)) = &self.discriminant {
        //     eq_token.to_tokens(tokens);
        //     disc.to_tokens(tokens);
        // }
    }
}

impl ToTokens for FieldsNamed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            self.named.to_tokens(tokens);
        });
    }
}

impl ToTokens for FieldsUnnamed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren_token.surround(tokens, |tokens| {
            self.unnamed.to_tokens(tokens);
        });
    }
}

pub struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

impl<'a, T> ToTokens for TokensOrDefault<'a, T>
where
    T: ToTokens + Default,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            Some(t) => t.to_tokens(tokens),
            None => T::default().to_tokens(tokens),
        }
    }
}

pub fn is_outer(attr: &&Attribute) -> bool {
    match attr.style {
        AttrStyle::Outer => true,
        AttrStyle::Inner(_) => false,
    }
}

pub fn is_inner(attr: &&Attribute) -> bool {
    match attr.style {
        AttrStyle::Outer => false,
        AttrStyle::Inner(_) => true,
    }
}
