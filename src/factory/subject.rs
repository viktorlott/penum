use std::collections::BTreeMap;

use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{
    punctuated::Punctuated,
    token::{self, Comma},
    Attribute, DataEnum, Expr, ExprMacro, Fields, Generics, Macro, Token, TraitBound, Variant,
    Visibility,
};

use crate::{
    penum::Stringify,
    utils::{ABSTRACT_MACRO_EXPR_SYMBOL, DEFAULT_VARIANT_SYMBOL},
};

use super::Comparable;

mod parse;
mod to_tokens;

pub type Variants = Punctuated<Variant, Comma>;

#[derive(Clone, Debug)]
pub struct Subject {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataEnum,
}

#[derive(Clone, Debug)]
pub struct AbstractExpr {
    pub bound: TraitBound,
    pub arrow: token::FatArrow,
    pub expr: Expr,
}

#[derive(Clone, Debug)]
pub struct DiscriminantImpl {
    composite: Punctuated<AbstractExpr, Token![,]>,
}

impl Subject {
    /// Should maybe remove this..
    pub fn get_variants(&self) -> &Variants {
        &self.data.variants
    }

    /// This will basically break each variant into two parts, VariantIdent and a Comparable. A
    /// Comparable will eventually pair up with another Comparable to create a ComparablePair.
    ///
    /// This intermediate construct is used to extract fields that will be used multiple times during
    /// compairs.
    pub fn comparable_fields_iter(&self) -> impl Iterator<Item = (&Ident, Comparable<Fields>)> {
        self.get_variants()
            .iter()
            .map(|variant| (&variant.ident, Comparable::from(&variant.fields)))
    }

    /// I just wanted to add this quickly and try it out, so I need to refactor this once I'm done testing.
    pub fn variants_to_arms(
        &self,
        wapper: impl Fn(&Expr) -> proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        self.get_variants()
            .iter()
            .filter_map(|variant| {
                variant.discriminant.as_ref()?;
                let name = &variant.ident;

                if name.get_string().contains(DEFAULT_VARIANT_SYMBOL) {
                    return None;
                }

                let (_, expr) = variant.discriminant.as_ref().unwrap();

                let expr_toks = match expr {
                    syn::Expr::Lit(_) => wapper(expr),
                    _ => expr.to_token_stream(),
                };

                match &variant.fields {
                    Fields::Named(named) => {
                        let fields = named.named.iter().enumerate().map(|(_, f)| {
                            let name = f.ident.as_ref();
                            quote::quote!(#name)
                        });

                        let tokens: proc_macro2::TokenStream =
                            itertools::intersperse(fields, quote::quote!(,)).collect();

                        quote::quote!(
                            Self::#name { #tokens } => { #expr_toks },
                        )
                    }
                    Fields::Unnamed(tup) => {
                        let fields = tup
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format_ident!("f{i}").to_token_stream());

                        let tokens: proc_macro2::TokenStream =
                            itertools::intersperse(fields, quote::quote!(,)).collect();

                        quote::quote!(
                            Self::#name ( #tokens ) => { #expr_toks },
                        )
                    }
                    Fields::Unit => {
                        quote::quote!(
                                Self::#name => { #expr_toks },
                        )
                    }
                }
                .into()
            })
            .collect()
    }

    /// The idea behind this method is that it will construct a Map that contains `TraitBound -> Self::$V $fields => Expr`
    ///
    /// Note that I'm thinking that if an implement! TraitBound exists, then we expect that there should be
    /// a default handle for that TraitBound if not all the variants have the TraitBound implement!.
    /// ```rust
    /// enum Enum {
    ///     Variant0      = implement! {
    ///         ToString => "Something".to_string(),
    /// //      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// //      |
    /// //      AbstractExpr
    ///     },
    /// //  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// //  |
    /// //  DiscriminantImpl
    ///
    ///     Variant1(i32) = implement! {
    ///         ToString => format!("{f0}"),
    ///     },
    ///
    ///     Variant2,
    ///
    ///     default       = implement! {
    ///         ToString => "Fallback string",
    ///     }
    /// //  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// //  |
    /// //  This part will act as a `catch all that doesn't implement`.
    /// }
    /// ```
    pub fn variants_to_arms_multi(&self) -> BTreeMap<String, proc_macro2::TokenStream> {
        let mut arms_map = BTreeMap::new();

        for variant in self.get_variants() {
            let name = &variant.ident;

            let Some((_, Expr::Macro(ExprMacro {mac: Macro { path, tokens: mac_tokens, ..}, ..} ))) = variant.discriminant.as_ref() else {continue};

            if !path.get_string().contains(ABSTRACT_MACRO_EXPR_SYMBOL) {
                continue;
            }

            let implementation: DiscriminantImpl = syn::parse_quote!({ #mac_tokens });
            let impls_iter = implementation.composite.iter();

            if name.get_string().contains(DEFAULT_VARIANT_SYMBOL) {
                impls_iter.for_each(|AbstractExpr { expr, bound, .. }| {
                    arms_map.insert(
                        bound.to_token_stream().to_string(),
                        quote::quote!(
                            _ => { #expr },
                        ),
                    );
                });

                continue;
            }

            let partial_arm = match &variant.fields {
                Fields::Named(named) => {
                    let fields = named.named.iter().enumerate().map(|(_, f)| {
                        let name = f.ident.as_ref();
                        quote::quote!(#name)
                    });

                    let tokens: proc_macro2::TokenStream =
                        itertools::intersperse(fields, quote::quote!(,)).collect();

                    quote::quote!(Self::#name { #tokens })
                }
                Fields::Unnamed(tup) => {
                    let fields = tup
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format_ident!("f{i}").to_token_stream());

                    let tokens: proc_macro2::TokenStream =
                        itertools::intersperse(fields, quote::quote!(,)).collect();

                    quote::quote!(Self::#name ( #tokens ))
                }
                Fields::Unit => {
                    quote::quote!(Self::#name)
                }
            };

            impls_iter.for_each(|AbstractExpr { expr, bound, .. }| {
                arms_map.insert(
                    bound.to_token_stream().to_string(),
                    quote::quote!(
                        #partial_arm => { #expr },
                    ),
                );
            });
        }

        arms_map
    }

    pub fn get_censored_subject_and_default_arm(
        mut self,
        default_else: Option<proc_macro2::TokenStream>,
    ) -> (Subject, proc_macro2::TokenStream) {
        let mut has_default = None;
        self.data.variants = self
            .data
            .variants
            .into_iter()
            .filter_map(|mut variant| {
                if variant.discriminant.is_some() && variant.ident == DEFAULT_VARIANT_SYMBOL {
                    let (_, expr) = variant.discriminant.as_ref().unwrap();
                    // This is a bad idea.. But I'm to lazy to change this implementation.
                    // Note that we are assuming that when `default_else` is None, we are in a
                    // `to_string` context, and because we are handling `default` we want to wrap this
                    // in a `format!()`. So given a `default = "hello from enum"`, we want it to end up
                    // being `default = format!("hello from enum")` implicitly.
                    has_default = Some(if default_else.is_none() {
                        quote::quote!(format!(#expr))
                    } else {
                        quote::quote!(#expr)
                    });
                    return None;
                }

                variant.discriminant = None;
                Some(variant)
            })
            .collect();

        (
            self,
            has_default
                .or(default_else)
                .or_else(|| Some(quote::quote!("".to_string())))
                .unwrap(),
        )
    }
}
