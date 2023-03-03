#![allow(irrefutable_let_patterns)]
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Fields::{self, Named, Unnamed},
    Variant, WhereClause,
};

use crate::{
    error::Diagnostic,
    utils::{parse_pattern, string, Shape, TypeMap},
};

/// A pattern can contain multiple shapes, but only one where clause.
/// e.g. `(_) | (_, _) | { name: _, age: usize }` 
pub struct Pattern {
    pub shapes: Vec<Shape>,
    pub where_clause: Option<WhereClause>,
}

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            shapes: input.call(parse_pattern)?,
            where_clause: input.parse()?,
        })
    }
}

impl Pattern {
    pub fn match_pattern_with<'a>(
        &'a self,
        variant_item: &'a Variant,
    ) -> Option<(&'a Fields, &'a Fields)> {
        self.shapes
            .iter()
            .find_map(|(_, pattern)| match (pattern, &variant_item.fields) {
                value @ ((Named(_), Named(_)) | (Unnamed(_), Unnamed(_)))
                    if value.0.len() == value.1.len() =>
                {
                    Some(value)
                }
                _ => None,
            })
    }

    pub fn validate_and_collect(
        &self,
        variant: &Variant,
        types: &mut TypeMap,
        error: &mut Diagnostic,
    ) {
        // A pattern can contain multiple shapes, e.g. `(_) | (_, _) | { name: _, age: usize }`
        // So if the variant_item matches a shape, we associate the pattern with the variant.
        let Some((pfields, ifields)) = self.match_pattern_with(variant) else {
            return error.extend(
                variant.fields.span(),
                format!(
                    "`{}` doesn't match pattern `{}`",
                    variant.to_token_stream(),
                    // Fix this shit
                    self.shapes
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
                error.extend(item.ty.span(), format!("Found {ity} but expected {pty}."));
                continue;
            }

            if let Some(set) = types.get_mut(&pty) {
                set.insert(ity);
            } else {
                types.insert(pty, vec![ity].into_iter().collect());
            }
        }
    }
}
