use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self},
    Token, TypeImplTrait, Variant, WhereClause,
};

use crate::factory::PatFrag;

#[derive(Default)]
pub struct PolymorphicMap(BTreeMap<String, BTreeSet<String>>);

/// Fix these later
impl PolymorphicMap {
    /// First we check if pty (T) exists in polymorphicmap.
    /// If it exists, insert new concrete type.
    pub fn polymap_insert(&mut self, pty: String, ity: String) {
        if let Some(set) = self.0.get_mut(pty.as_str()) {
            set.insert(ity);
        } else {
            self.0.insert(pty, vec![ity].into_iter().collect());
        }
    }
}

impl Deref for PolymorphicMap {
    type Target = BTreeMap<String, BTreeSet<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_pattern(input: ParseStream) -> syn::Result<Vec<PatFrag>> {
    let mut shape = vec![input.call(parse_pattern_fragment)?];

    while input.peek(token::Or) {
        let _: token::Or = input.parse()?;
        shape.push(input.call(parse_pattern_fragment)?);
    }

    Ok(shape)
}

pub fn parse_pattern_fragment(input: ParseStream) -> syn::Result<PatFrag> {
    if input.peek(Token![$]) {
        let _: Token![$] = input.parse()?;
    }
    Ok(PatFrag {
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

#[allow(dead_code)]
pub fn ident_impl(imptr: &TypeImplTrait) -> Ident {
    format_ident!(
        "__IMPL_{}",
        string(&imptr.bounds)
            .replace(' ', "_")
            .replace(['?', '\''], "")
    )
}

pub fn no_match_found(item: &impl ToTokens, pat: &str) -> String {
    format!(
        "`{}` doesn't match pattern `{}`",
        item.to_token_stream(),
        pat
    )
}

// let ty_span = pred.span();
// let assert_sync = quote_spanned!{ty_span=>
//     struct _AssertSync where #pred: Sync;
// };
// println!("{}", assert_sync);
