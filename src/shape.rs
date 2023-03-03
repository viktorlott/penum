#![allow(irrefutable_let_patterns)]
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},

    spanned::Spanned,
    Fields::{self, Named, Unnamed},
    Variant, WhereClause 
};

use crate::{error::ErrorStash, utils::{PatternItem, MatchedPatterns, parse_pattern, string}};

pub struct Shape {
    /// name(T) where T : Hello
    pub pattern: Vec<PatternItem>,
    pub where_clause: Option<WhereClause>,
}

impl Parse for Shape {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pattern: input.call(parse_pattern)?,
            where_clause: input.parse()?,
        })
    }
}

impl Shape {
    pub fn pattern<'a>(&'a self, variant_item: &'a Variant) -> Option<(&'a Fields, &'a Fields)> {
        self.pattern
            .iter()
            .find_map(|(_, pattern)| match (pattern, &variant_item.fields) {
                value @ ((Named(_), Named(_)) | (Unnamed(_), Unnamed(_)))
                    if value.0.len() == value.1.len() => Some(value),
                _ => None,
            })
    }

    pub fn validate_and_collect(
        &self,
        variant_item: &Variant,
        ptype_pairs: &mut MatchedPatterns,
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
