use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, DataEnum, Generics, Token, Visibility,
};

use crate::utils::parse_enum;

use super::Subject;

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
