#![allow(irrefutable_let_patterns)]
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token, DeriveInput, Error, Fields, PredicateType, Token, Variant, WhereClause, WherePredicate,
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
        let mut bound_tokens = TokenStream2::new();
        if let Some(where_cl) = self.shape.where_clause.as_ref() {
            for predicate in where_cl.predicates.iter() {
                if let syn::WherePredicate::Type(PredicateType {
                    bounded_ty, bounds, ..
                }) = predicate
                {
                    if let Some(pty_set) = self.types.get(&bounded_ty.to_token_stream().to_string())
                    {
                        for ty in pty_set.iter() {
                            let ty = format_ident!("{}", ty);
                            let ty_predicate = quote!(#ty: #bounds);
                            bound_tokens = quote!(#bound_tokens #ty_predicate,)
                        }
                    }
                } else {
                    self.error
                        .extend(Span::call_site(), "Unsupported `where clause`")
                }
            }
        }
        // TODO: Fix this shit
        if let Some(ref error) = self.error.0 {
            error.to_compile_error().into()
        } else if let Some(ref mut swc) = self.input.generics.where_clause {
            // TODO: Change this to optional later
            let where_clause: Punctuated<WherePredicate, Token![,]> = parse_quote!(#bound_tokens);
            for nwc in where_clause.iter() {
                swc.predicates.push(nwc.clone())
            }
            self.input.to_token_stream().into()
        } else {
            self.input.generics.where_clause = Some(parse_quote!(where #bound_tokens));
            self.input.to_token_stream().into()
        }
    }
}

impl VariantPattern {
    fn pattern<'a>(&'a self, variant_item: &'a Variant) -> Option<(&'a Fields, &'a Fields)> {
        let pattern = &self.pattern.get(0).unwrap().1;
        if let value = match (pattern, &variant_item.fields) {
            value @ ((Fields::Named(_), Fields::Named(_))
            | (Fields::Unnamed(_), Fields::Unnamed(_)))
                if value.0.len() == value.1.len() =>
            {
                Some((value.0, value.1))
            }
            _ => None,
        } {
            value
        } else {
            None
        }
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
