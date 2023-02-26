use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use proc_macro::{TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{self, Parse},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Error, Fields, FieldsUnnamed, PredicateType,
    Token, Type, Variant, WhereClause, WherePredicate,
};

/// name(T) where T : Hello
struct VariantPattern {
    variant: Variant,
    where_clause: Option<WhereClause>,
}


impl Parse for VariantPattern {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            variant: input.parse()?,
            where_clause: input.parse()?,
        })
    }
}

struct ErrorStash(Option<Error>);

impl ErrorStash {
    fn extend(&mut self, span: Span, error: impl Display) {
        if let Some(err) = self.0.as_mut() {
            err.combine(Error::new(span, error));
        } else {
            self.0 = Some(Error::new(span, error));
        }
    }

    fn into_or(self, data: impl FnOnce() -> DeriveInput) -> TokenStream {
        if let Some(error) = self.0 {
            error.to_compile_error().into()
        } else {
            data().to_token_stream().into()
        }
    }
}

fn compare_ty(pty: &Type, ity: &Type, pairs: &mut PatternTypePairs) -> Option<(String, String)> {
    let pty = pty.to_token_stream().to_string();
    let ity = ity.to_token_stream().to_string();

    if !(pty.eq("_") || pty.to_uppercase().eq(&pty)) && pty != ity {
        return Some((pty, ity));
    }

    if let Some(set) = pairs.get_mut(&pty) {
        set.insert(ity);
    } else {
        let mut bset = BTreeSet::new();
        bset.insert(ity);
        pairs.insert(pty, bset);
    }

    None
}

fn validate_and_collect(
    pat_fields: &FieldsUnnamed,
    item_fields: &FieldsUnnamed,
    errors: &mut ErrorStash,
) -> PatternTypePairs {
    let mut pairs: PatternTypePairs = BTreeMap::default();
    pat_fields
        .unnamed
        .iter()
        .zip(item_fields.unnamed.iter())
        .for_each(|(pat, item)| {
            if let Some((fty, ty)) = compare_ty(&pat.ty, &item.ty, &mut pairs) {
                errors.extend(item.ty.span(), format!("Found {fty} but expected {ty}."))
            }
        });
    pairs
}

type PatternTypePairs = BTreeMap<String, BTreeSet<String>>;
fn matcher(
    variant_pattern: &VariantPattern,
    variant: &Variant,
    errors: &mut ErrorStash,
    source: &mut DeriveInput,
    wclause: &mut TokenStream2,
) {
    match (&variant_pattern.variant.fields, &variant.fields) {
        (Fields::Named(_pat_fields), Fields::Named(_item_fields)) => {}
        (Fields::Unnamed(pat_fields), Fields::Unnamed(item_fields)) => {
            if pat_fields.unnamed.len() == item_fields.unnamed.len() {
                // e.g. `T -> [i32, f32]`, `U -> [String, usize, CustomStruct]
                let ptype_pairs: PatternTypePairs =
                    validate_and_collect(pat_fields, item_fields, errors);

                if let Some(where_cl) = variant_pattern.where_clause.as_ref() {
                    where_cl.predicates.iter().for_each(|predicate| {
                        match predicate {
                            syn::WherePredicate::Type(PredicateType {
                                bounded_ty, bounds, ..
                            }) => {
                                if let Some(pty_set) =
                                    ptype_pairs.get(&bounded_ty.to_token_stream().to_string())
                                {
                                    pty_set.iter().for_each(|ty| {
                                        let ty = format_ident!("{}", ty);
                                        let ty_predicate = quote!(#ty: #bounds);
                                        *wclause = quote!(#wclause #ty_predicate,)
                                    });
                                }
                            }
                            _ => errors.extend(source.span(), "Unsupported where clause"),
                        }
                    });
                }
            } else {
                errors.extend(
                    variant.fields.span(),
                    format!(
                        "`{}` doesn't match pattern `{} {}`",
                        variant.to_token_stream(),
                        variant_pattern.variant.to_token_stream(),
                        variant_pattern.where_clause.to_token_stream()
                    ),
                )
            }
        }
        _ => errors.extend(variant.span(), "Variant doesn't match pattern"),
    };
}

#[proc_macro_attribute]
pub fn shape(attr: TokenStream, input: TokenStream) -> TokenStream {
    let derived_input = parse_macro_input!(input as DeriveInput);

    let Data::Enum(enum_definition) = &derived_input.data else {
        return Error::new(derived_input.ident.span(), "Expected an enum.").to_compile_error().into();
    };

    if enum_definition.variants.is_empty() {
        return Error::new(
            enum_definition.variants.span(),
            "Expected to find at least one variant.",
        )
        .to_compile_error()
        .into();
    }

    let variant_pattern = parse_macro_input!(attr as VariantPattern);
    let mut errors: ErrorStash = ErrorStash(None);
    let mut source = derived_input.clone();
    let mut wclause: TokenStream2 = TokenStream2::new();

    enum_definition.variants.iter().for_each(|variant| {
        matcher(
            &variant_pattern,
            variant,
            &mut errors,
            &mut source,
            &mut wclause,
        )
    });

    // TODO: Change this to optional later
    let where_clause: Punctuated<WherePredicate, Token![,]> = parse_quote!(#wclause);

    println!("{}", source.to_token_stream());
    // TODO: Fix this shit
    errors.into_or(|| {
        if let Some(ref mut swc) = source.generics.where_clause {
            where_clause.iter().for_each(|nwc| swc.predicates.push(nwc.clone()))
        } else {
            source.generics.where_clause = Some(parse_quote!(where #where_clause))
        }
        source
    })
}
