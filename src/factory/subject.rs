use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    AttrStyle, Attribute, DataEnum, Generics, Token, Visibility,
};

use crate::utils::parse_enum;

pub struct Subject {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataEnum,
}

impl Parse for Subject {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![enum]) {
            let enum_token = input.parse::<Token![enum]>()?;
            let ident = input.parse::<Ident>()?;
            let generics = input.parse::<Generics>()?;
            let (where_clause, brace, variants) = parse_enum(input)?;

            let generics = Generics {
                where_clause,
                ..generics
            };

            let data = DataEnum {
                enum_token,
                brace_token: brace,
                variants,
            };

            Ok(Subject {
                attrs,
                vis,
                ident,
                generics,
                data,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

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
