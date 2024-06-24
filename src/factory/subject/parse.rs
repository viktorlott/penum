use proc_macro2::Ident;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, DataEnum, Generics, Token, Variant, Visibility, WhereClause,
};

use super::{AbstractExpr, DiscriminantImpl, Subject};

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

impl Parse for AbstractExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            bound: input.parse()?,
            arrow: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl Parse for DiscriminantImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);
        Ok(Self {
            composite: content.parse_terminated(AbstractExpr::parse)?,
        })
    }
}

pub fn parse_enum(
    input: ParseStream,
) -> syn::Result<(
    Option<WhereClause>,
    token::Brace,
    Punctuated<Variant, Token![,]>,
)> {
    let where_clause = input.parse()?;

    let content;
    let brace = braced!(content in input);
    let variants = content.parse_terminated(Variant::parse)?;

    Ok((where_clause, brace, variants))
}
