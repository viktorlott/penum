use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{AttrStyle, Attribute};

use super::Subject;

impl ToTokens for Subject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.attrs
            .iter()
            .filter(is_outer)
            .for_each(|attr| attr.to_tokens(tokens));

        self.vis.to_tokens(tokens);
        self.data.enum_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);

        self.generics.where_clause.to_tokens(tokens);
        self.data.brace_token.surround(tokens, |tokens| {
            self.data.variants.to_tokens(tokens);
        });
    }
}

#[inline(always)]
fn is_outer(attr: &&Attribute) -> bool {
    match attr.style {
        AttrStyle::Outer => true,
        AttrStyle::Inner(_) => false,
    }
}
