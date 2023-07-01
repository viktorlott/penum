use proc_macro2::Ident;
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    token::{self},
    Attribute, Expr, Generics, Token, Visibility, WhereClause,
};

use super::*;

impl Parse for Strukt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![struct]) {
            let struct_token = input.parse::<Token![struct]>()?;
            let ident = input.parse::<Ident>()?;
            let generics = input.parse::<Generics>()?;

            let (where_clause, fields, semi) = data_struct(input)?;

            let generics = Generics {
                where_clause,
                ..generics
            };

            let data = DataStruct {
                struct_token,
                fields,
                semi_token: semi,
            };

            Ok(Self {
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

impl Parse for FieldsNamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(FieldsNamed {
            brace_token: braced!(content in input),
            named: content.parse_terminated(FieldDisc::parse)?,
        })
    }
}

impl Parse for FieldsUnnamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(FieldsUnnamed {
            paren_token: parenthesized!(content in input),
            unnamed: content.parse_terminated(FieldDisc::parse)?,
        })
    }
}

impl Parse for FieldDisc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let colon_token = input.parse()?;

        let is_ident = input.peek(Ident::peek_any);
        let is_bang = input.peek2(Token![!]);

        let (ty, discriminant) = if is_ident && is_bang {
            let _: Ident = input.parse()?;
            let _: Token![!] = input.parse()?;
            let content;
            if input.peek(token::Paren) {
                let _ = parenthesized!(content in input);
            } else {
                let _ = braced!(content in input);
            }
            let ty: Type = content.parse()?;
            let eq_token: Token![=] = content.parse()?;
            let expr: Expr = content.parse()?;
            (ty, Some((eq_token, expr)))
        } else {
            (input.parse()?, None)
        };

        Ok(Self {
            attrs,
            vis,
            ident,
            colon_token,
            ty,
            discriminant,
        })
    }
}

impl Parse for ExprCall {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;

        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
        } else if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
        }

        Ok(Self(ty, input.parse()?))
    }
}

pub fn data_struct(
    input: ParseStream,
) -> syn::Result<(Option<WhereClause>, FieldsKind, Option<Token![;]>)> {
    let mut lookahead = input.lookahead1();
    let mut where_clause = None;
    if lookahead.peek(Token![where]) {
        where_clause = Some(input.parse()?);
        lookahead = input.lookahead1();
    }

    if where_clause.is_none() && lookahead.peek(token::Paren) {
        let fields = input.parse()?;

        lookahead = input.lookahead1();
        if lookahead.peek(Token![where]) {
            where_clause = Some(input.parse()?);
            lookahead = input.lookahead1();
        }

        if lookahead.peek(Token![;]) {
            let semi = input.parse()?;
            Ok((where_clause, FieldsKind::Unnamed(fields), Some(semi)))
        } else {
            Err(lookahead.error())
        }
    } else if lookahead.peek(token::Brace) {
        let fields = input.parse()?;
        Ok((where_clause, FieldsKind::Named(fields), None))
    } else if lookahead.peek(Token![;]) {
        let semi = input.parse()?;
        Ok((where_clause, FieldsKind::Unit, Some(semi)))
    } else {
        Err(lookahead.error())
    }
}
