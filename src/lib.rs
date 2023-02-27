use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, Ident};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse,ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Error, Fields, token, PredicateType, Token, Variant, WhereClause, WherePredicate,
};

type PatternItem = (Option<Ident>, Fields);

/// name(T) where T : Hello
struct VariantPattern {
    #[allow(dead_code)]
    pattern: Vec<PatternItem>,
    where_clause: Option<WhereClause>,
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

fn parse_pattern(input: ParseStream) -> syn::Result<Vec<PatternItem>> {
    let mut pattern = vec![input.call(parse_fields)?];
    while input.peek(token::Or) {
        let _: token::Or = input.parse()?;
        pattern.push(input.call(parse_fields)?);
    }
  
    Ok(pattern)
}

impl Parse for VariantPattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pattern: input.call(parse_pattern)?,
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

fn string<T: ToTokens>(x: &T) -> String {
    x.to_token_stream().to_string()
}

fn construct_bounds_tokens(
    pw_clause: Option<&WhereClause>,
    ppairs: &PatternTypePairs,
    errors: &mut ErrorStash,
) -> TokenStream2 {
    let mut bound_tokens = TokenStream2::new();
    if let Some(where_cl) = pw_clause {
        for predicate in where_cl.predicates.iter() {
            if let syn::WherePredicate::Type(PredicateType { bounded_ty, bounds, .. }) = predicate {
                if let Some(pty_set) = ppairs.get(&bounded_ty.to_token_stream().to_string()) {
                    pty_set.iter().for_each(|ty| {
                        let ty = format_ident!("{}", ty);
                        let ty_predicate = quote!(#ty: #bounds);
                        bound_tokens = quote!(#bound_tokens #ty_predicate,)
                    });
                }
            } else {
                errors.extend(Span::call_site(), "Unsupported `where clause`")
            }
           
        }
    }
    bound_tokens
}
// e.g. `T -> [i32, f32]`, `U -> [String, usize, CustomStruct]
type PatternTypePairs = BTreeMap<String, BTreeSet<String>>;
fn matcher(
    fields_pattern: &Fields,
    variant_item: &Variant,
    ptype_pairs: &mut PatternTypePairs,
    errors: &mut ErrorStash,
) {
    let Some((pfields, ifields)) = (match (&fields_pattern, &variant_item.fields) {
        value @ (
            (Fields::Named(_), Fields::Named(_)) | 
            (Fields::Unnamed(_), Fields::Unnamed(_))
        ) if value.0.len() == value.1.len() => Some(value),
        _ => None,
    }) else {
        return errors.extend(
            variant_item.fields.span(),
            format!(
                "`{}` doesn't match pattern `{}`",
                variant_item.to_token_stream(),
                fields_pattern.to_token_stream()
            ),
        )
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

    let mut source = derived_input.clone();

    let variant_pattern = parse_macro_input!(attr as VariantPattern);
    let mut ptype_pairs: PatternTypePairs = PatternTypePairs::new();
    let mut errors: ErrorStash = ErrorStash(None);

    let pattern = variant_pattern.pattern.get(0).unwrap();

    for variant in enum_definition.variants.iter() {
        matcher(
            &pattern.1,
            variant,
            &mut ptype_pairs,
            &mut errors,
        )
    }

    let ty_predicate = construct_bounds_tokens(
        variant_pattern.where_clause.as_ref(),
        &ptype_pairs,
        &mut errors,
    );

    // TODO: Fix this shit
    errors.into_or(|| {
        if let Some(ref mut swc) = source.generics.where_clause {
            // TODO: Change this to optional later
            let where_clause: Punctuated<WherePredicate, Token![,]> = parse_quote!(#ty_predicate);
            where_clause
                .iter()
                .for_each(|nwc| swc.predicates.push(nwc.clone()))
        } else {
            source.generics.where_clause = Some(parse_quote!(where #ty_predicate))
        }
        source
    })
}
