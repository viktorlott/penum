#![allow(irrefutable_let_patterns)]
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::format_ident;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    token::{self},
    DeriveInput, Error,
    Fields::{self, Named, Unnamed},
    Token, Variant, WhereClause, WherePredicate,
};
pub type PatternItem = (Option<Ident>, Fields);
pub type PatternTypePairs = BTreeMap<String, BTreeSet<String>>;

pub struct State {
    pub shape: VariantPattern,
    pub input: DeriveInput,
    pub error: ErrorStash,
    pub types: PatternTypePairs,
}

pub struct VariantPattern {
    #[allow(dead_code)]
    /// name(T) where T : Hello
    pub pattern: Vec<PatternItem>,
    pub where_clause: Option<WhereClause>,
}

pub struct ErrorStash(Option<Error>);

impl State {
    pub fn new(shape: VariantPattern, input: DeriveInput) -> Self {
        Self {
            shape,
            input,
            error: ErrorStash::new(),
            types: PatternTypePairs::new(),
        }
    }
    pub fn matcher(&mut self, variant_item: &Variant) {
        self.shape
            .matcher(variant_item, &mut self.types, &mut self.error)
    }
    pub fn collect_tokens(mut self) -> TokenStream {
        let bound_tokens = self.link_bounds();

        // TODO: Fix this shit
        if let Some(ref error) = self.error.0 {
            error.to_compile_error().into()
        } else {
            self.extend_where_clause(&bound_tokens);
            self.input.to_token_stream().into()
        }
    }

    fn link_bounds(&mut self) -> Vec<TokenStream2> {
        let mut bound_tokens = Vec::new();
        if let Some(where_cl) = self.shape.where_clause.as_ref() {
            for predicate in where_cl.predicates.iter() {
                match predicate {
                    WherePredicate::Type(pred) => {
                        if let Some(pty_set) = self.types.get(&string(&pred.bounded_ty)) {
                            pty_set
                                .iter()
                                .map(|ident| (format_ident!("{}", ident), &pred.bounds))
                                .for_each(|(ident, bound)| bound_tokens.push(parse_quote!(#ident: #bound)))
                        }
                    }
                    _ => self
                        .error
                        .extend(Span::call_site(), "Unsupported `where clause`"),
                }
            }
        }
        bound_tokens
    }

    fn extend_where_clause(&mut self, bounds: &[TokenStream2]) {
        bounds.iter().for_each(|bound| {
            self.input
                .generics
                .where_clause
                .get_or_insert_with(|| parse_quote!(where))
                .predicates
                .push(parse_quote!(#bound))
        })
    }
}

impl VariantPattern {
    fn pattern<'a>(&'a self, variant_item: &'a Variant) -> Option<(&'a Fields, &'a Fields)> {
        self.pattern
            .iter()
            .find_map(|(_, pattern)| match (pattern, &variant_item.fields) {
                value @ ((Named(_), Named(_)) | (Unnamed(_), Unnamed(_)))
                    if value.0.len() == value.1.len() => Some(value),
                _ => None,
            })
    }
    fn matcher(
        &self,
        variant_item: &Variant,
        ptype_pairs: &mut PatternTypePairs,
        errors: &mut ErrorStash,
    ) {
        let Some((pfields, ifields)) = self.pattern(variant_item) else {
            return errors.extend(
                variant_item.fields.span(),
                format!(
                    "`{}` doesn't match pattern `{}`",
                    variant_item.to_token_stream(),
                    // Fix this shit
                    self.pattern
                        .iter()
                        .map(|(_, f)| f.to_token_stream().to_string())
                        .reduce(|acc, s| if acc.is_empty() {s} else {format!("{acc} | {s}")}).unwrap()
                ),
            );
        };

        for (pat, item) in pfields.into_iter().zip(ifields.into_iter()) {
            let (pty, ity) = (string(&pat.ty), string(&item.ty));
            let is_generic = pty.eq("_") || pty.to_uppercase().eq(&pty);

            if !is_generic && pty != ity {
                return errors.extend(item.ty.span(), format!("Found {ity} but expected {pty}."));
            }

            if let Some(set) = ptype_pairs.get_mut(&pty) {
                set.insert(ity);
            } else {
                ptype_pairs.insert(pty, vec![ity].into_iter().collect());
            }
        }
    }
}

impl Parse for VariantPattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pattern: input.call(parse_pattern)?,
            where_clause: input.parse()?,
        })
    }
}

impl ErrorStash {
    pub fn new() -> Self {
        Self(None)
    }
    pub fn extend(&mut self, span: Span, error: impl Display) {
        if let Some(err) = self.0.as_mut() {
            err.combine(Error::new(span, error));
        } else {
            self.0 = Some(Error::new(span, error));
        }
    }
}

fn parse_fields(input: ParseStream) -> syn::Result<PatternItem> {
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

fn string<T: ToTokens>(x: &T) -> String {
    x.to_token_stream().to_string()
}
