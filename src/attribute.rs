#![allow(irrefutable_let_patterns)]
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use proc_macro::TokenStream;
use proc_macro2::{Span, Ident};
use quote::{ToTokens};
use syn::{
    parse::{Parse,ParseStream},
    spanned::Spanned,
    DeriveInput, Error, Fields, token, Token, Variant, WhereClause,
};

pub type PatternItem = (Option<Ident>, Fields);
pub type PatternTypePairs = BTreeMap<String, BTreeSet<String>>;

pub struct State {
    pub shape: VariantPattern,
    pub input: DeriveInput,
    pub error: ErrorStash
}

pub struct VariantPattern {
    #[allow(dead_code)]
    /// name(T) where T : Hello
    pub pattern: Vec<PatternItem>,
    pub where_clause: Option<WhereClause>,
}

pub struct ErrorStash(Option<Error>);


impl State {
    pub fn new(shape: VariantPattern, input: DeriveInput, error: ErrorStash) -> Self {
        Self { shape, input, error }
    }
    pub fn matcher(
        &mut self,
        variant_item: &Variant,
        ptype_pairs: &mut PatternTypePairs,
    ) {
        self.shape.matcher(variant_item, ptype_pairs, &mut self.error)
    }
}

impl VariantPattern {
    fn pattern<'a>(&'a self, variant_item: &'a Variant) -> Option<(&'a Fields, &'a Fields)> {
        let pattern = &self.pattern.get(0).unwrap().1;
        if let value = match (pattern, &variant_item.fields) {
            value @ (
                (Fields::Named(_), Fields::Named(_)) | 
                (Fields::Unnamed(_), Fields::Unnamed(_))
            ) if value.0.len() == value.1.len() => Some((value.0, value.1)),
            _ => None,
        } {
            value
        } else {
             None
        }
    }
    pub fn matcher(
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
                    self.pattern.get(0).unwrap().1.to_token_stream()
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

impl Parse for State {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            shape: input.parse()?,
            input: input.parse()?,
            error: ErrorStash(None)
        })
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

    pub fn into_or(self, data: impl FnOnce() -> DeriveInput) -> TokenStream {
        if let Some(error) = self.0 {
            error.to_compile_error().into()
        } else {
            data().to_token_stream().into()
        }
    }
}

fn parse_fields(input: ParseStream) -> syn::Result<PatternItem> {
    if input.peek(Token![$]) { 
        let _: Token![$] = input.parse()?;
    }
    Ok((input.parse()?, if input.peek(token::Brace) {
        Fields::Named(input.parse()?)
    } else if input.peek(token::Paren) {
        Fields::Unnamed(input.parse()?)
    } else {
        Fields::Unit
    }))
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
