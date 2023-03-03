#![allow(irrefutable_let_patterns)]
use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::{Ident};

use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    token::{self},
    Fields,
    Token, WhereClause, punctuated::Punctuated, Variant, braced
};

pub type PatternItem = (Option<Ident>, Fields);
pub type MatchedPatterns = BTreeMap<String, BTreeSet<String>>;

pub fn parse_fields(input: ParseStream) -> syn::Result<PatternItem> {
    if input.peek(Token![$]) {
        let _: Token![$] = input.parse()?;
    }
    Ok((
        input.parse()?,
        if input.peek(token::Brace) {
            Fields::Named(input.parse()?)
        } else if input.peek(token::Paren) {
            Fields::Unnamed(input.parse()?)
        } else {
            Fields::Unit
        },
    ))
}

pub fn parse_pattern(input: ParseStream) -> syn::Result<Vec<PatternItem>> {
    let mut pattern = vec![input.call(parse_fields)?];

    while input.peek(token::Or) {
        let _: token::Or = input.parse()?;
        pattern.push(input.call(parse_fields)?);
    }

    Ok(pattern)
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

pub fn string<T: ToTokens>(x: &T) -> String {
    x.to_token_stream().to_string()
}
