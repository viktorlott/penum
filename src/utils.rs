use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    token::{self},
    Token, WhereClause, punctuated::Punctuated, Variant, braced,
};

use crate::factory::Shape;

pub type TypeMap = BTreeMap<String, BTreeSet<String>>;

pub fn parse_shapes(input: ParseStream) -> syn::Result<Vec<Shape>> {
    let mut shape = vec![input.call(parse_shape)?];

    while input.peek(token::Or) {
        let _: token::Or = input.parse()?;
        shape.push(input.call(parse_shape)?);
    }

    Ok(shape)
}

pub fn parse_shape(input: ParseStream) -> syn::Result<Shape> {
    if input.peek(Token![$]) {
        let _: Token![$] = input.parse()?;
    }
    Ok(Shape {
        ident: input.parse()?,
        scope: input.parse()?
    })
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
