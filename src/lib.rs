use std::{
    collections::{BTreeMap, BTreeSet},
};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, Ident};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse,ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Error, Fields, PredicateType, Token, Variant, WhereClause, WherePredicate,
};

mod attribute;

use attribute::{VariantPattern, ErrorStash, State, PatternTypePairs};

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
                    for ty in pty_set.iter() {
                        let ty = format_ident!("{}", ty);
                        let ty_predicate = quote!(#ty: #bounds);
                        bound_tokens = quote!(#bound_tokens #ty_predicate,)
                    }
                }
            } else {
                errors.extend(Span::call_site(), "Unsupported `where clause`")
            }
        }
    }
    bound_tokens
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

    let mut state = State::new(parse_macro_input!(attr as VariantPattern), derived_input.clone(), ErrorStash::new());
    let mut ptype_pairs: PatternTypePairs = PatternTypePairs::new();

    // Loop through enum definition and match each variant with each shape pattern.
    // for each variant => pattern.find(variant)

    for variant in enum_definition.variants.iter() {
        state.matcher(variant, &mut ptype_pairs);
    }

    let ty_predicate = construct_bounds_tokens(
        state.shape.where_clause.as_ref(),
        &ptype_pairs,
        &mut state.error,
    );

    // TODO: Fix this shit
    state.error.into_or(|| {
        if let Some(ref mut swc) = state.input.generics.where_clause {
            // TODO: Change this to optional later
            let where_clause: Punctuated<WherePredicate, Token![,]> = parse_quote!(#ty_predicate);
            for nwc in where_clause.iter() {
                swc.predicates.push(nwc.clone())
            }
        } else {
            state.input.generics.where_clause = Some(parse_quote!(where #ty_predicate))
        }
        state.input
    })
}
