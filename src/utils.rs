use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self},
    Token, Variant, WhereClause,
};

use crate::factory::PatternFrag;

pub type PolymorphicMap = BTreeMap<String, BTreeSet<String>>;

pub fn parse_pattern(input: ParseStream) -> syn::Result<Vec<PatternFrag>> {
    let mut shape = vec![input.call(parse_pattern_fragment)?];

    while input.peek(token::Or) {
        let _: token::Or = input.parse()?;
        shape.push(input.call(parse_pattern_fragment)?);
    }

    Ok(shape)
}

pub fn parse_pattern_fragment(input: ParseStream) -> syn::Result<PatternFrag> {
    if input.peek(Token![$]) {
        let _: Token![$] = input.parse()?;
    }
    Ok(PatternFrag {
        ident: input.parse()?,
        group: input.parse()?,
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

// let ty_span = pred.span();
// let assert_sync = quote_spanned!{ty_span=>
//     struct _AssertSync where #pred: Sync;
// };
// println!("{}", assert_sync);
