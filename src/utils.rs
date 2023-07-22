#![allow(unused)]
use std::{
    cell::UnsafeCell,
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Once,
};

use proc_macro2::{Ident, Span};
use quote::{format_ident, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Add},
    Expr, Fields, Token, TraitBound, Type, TypeParamBound, Variant, WhereClause,
};

use crate::{
    error::Diagnostic,
    factory::{PatComposite, PatFrag, Subject},
    penum::{Stringify, TraitBoundUtils},
};

pub const DEFAULT_VARIANT_NAME: &str = "default";

pub struct Static<T, F = fn() -> T>(UnsafeCell<Option<T>>, Once, F);

#[derive(Default, Debug)]
pub struct PolymorphicMap<K: Hash, V: Hash>(BTreeMap<K, BTreeSet<V>>);

#[derive(Hash, Debug, Clone, Copy)]
pub struct UniqueHashId<T: Hash>(pub T);

/// Fix these later
impl<K: Hash + Clone, V: Hash + Clone> PolymorphicMap<UniqueHashId<K>, UniqueHashId<V>>
where
    UniqueHashId<K>: Ord,
    UniqueHashId<V>: Ord,
{
    pub fn polymap_insert(&mut self, pty: UniqueHashId<K>, ity: UniqueHashId<V>) {
        // First we check if pty (T) exists in
        // polymorphicmap. If it exists, insert new
        // concrete type.
        if let Some(set) = self.0.get_mut(&pty) {
            set.insert(ity);
        } else {
            self.0.insert(pty, vec![ity].into_iter().collect());
        }
    }
}

impl<K: Hash, V: Hash> Deref for PolymorphicMap<UniqueHashId<K>, UniqueHashId<V>> {
    type Target = BTreeMap<UniqueHashId<K>, BTreeSet<UniqueHashId<V>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Hash + Clone> UniqueHashId<T> {
    pub fn new(value: &T) -> Self {
        Self(value.clone())
    }

    pub fn get_unique_ident(&self) -> Ident
    where
        T: Spanned + ToTokens,
    {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        format_ident!("_{}", hasher.finish(), span = self.0.span())
    }

    pub fn get_unique_string(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        format!("_{}", hasher.finish())
    }
}

impl<T> Static<T> {
    pub const fn new(func: fn() -> T) -> Self {
        Self(UnsafeCell::new(None), Once::new(), func)
    }
    pub fn get(&self) -> &'static T {
        // SAFETY: Read [dispatch/ret.rs:23]
        self.1
            .call_once(|| unsafe { *self.0.get() = Some(self.2()) });
        unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
    }
}

unsafe impl<T> Sync for Static<T> {}

impl From<Ident> for UniqueHashId<Type> {
    fn from(value: Ident) -> Self {
        Self(parse_quote!(#value))
    }
}

impl<T: ToTokens + Hash + Spanned + Clone> From<&T> for UniqueHashId<T> {
    fn from(value: &T) -> Self {
        Self(value.clone())
    }
}

impl<T: Hash> Deref for UniqueHashId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for UniqueHashId<Type> {
    fn default() -> Self {
        Self(parse_quote!(_))
    }
}

impl PartialEq for UniqueHashId<Type> {
    fn eq(&self, other: &Self) -> bool {
        self.get_unique_ident() == other.get_unique_ident()
    }
}

impl PartialOrd for UniqueHashId<Type> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_unique_ident()
            .partial_cmp(&other.get_unique_ident())
    }
}

impl Ord for UniqueHashId<Type> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_unique_ident().cmp(&other.get_unique_ident())
    }
}

impl Eq for UniqueHashId<Type> {}

pub fn no_match_found(item: &impl ToTokens, pat: &str) -> String {
    format!(
        "`{}` doesn't match pattern `{}`",
        item.to_token_stream(),
        pat
    )
}

pub fn maybe_bounds_not_permitted(trait_bound: &TraitBound) -> String {
    format!(
        "`?{}` bounds are only permitted at the point where a type parameter is declared",
        trait_bound.path.get_string()
    )
}

pub fn lifetime_not_permitted() -> &'static str {
    "Lifetime annotation not permitted"
}

pub fn create_unique_ident(value: &str, tag: &Ident, span: Span) -> Ident {
    format_ident!("_{}_{}", tag, value, span = span)
}

/// I just wanted to add this quickly and try it out, so I need to refactor this once I'm done testing.
pub fn variants_to_arms<'a>(
    variants: impl Iterator<Item = &'a Variant>,
    wapper: impl Fn(&'a Expr) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    variants
        .filter_map(|variant| {
            variant.discriminant.as_ref()?;
            let name = &variant.ident;

            if name.get_string().contains(DEFAULT_VARIANT_NAME) {
                return None;
            }

            let (_, expr) = variant.discriminant.as_ref().unwrap();

            let expr_toks = match expr {
                syn::Expr::Lit(_) => wapper(expr),
                _ => expr.to_token_stream(),
            };

            let arm = match &variant.fields {
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
            };
            Some(arm)
        })
        .collect()
}

pub fn create_impl_string<'a>(
    bounds: &'a Punctuated<TypeParamBound, Add>,
    error: &'a mut Diagnostic,
) -> Option<String> {
    let mut impl_string = String::new();

    for bound in bounds.iter() {
        match bound {
            syn::TypeParamBound::Trait(trait_bound) => {
                if let syn::TraitBoundModifier::None = trait_bound.modifier {
                    impl_string.push_str(&trait_bound.get_unique_trait_bound_id())
                } else {
                    error.extend(bound.span(), maybe_bounds_not_permitted(trait_bound));
                }
            }
            syn::TypeParamBound::Lifetime(_) => {
                error.extend_spanned(bound, lifetime_not_permitted());
            }
        }
    }

    if error.has_error() || impl_string.is_empty() {
        None
    } else {
        Some(impl_string)
    }
}

pub fn censor_discriminants_get_default(
    mut subject: Subject,
    default_else: Option<proc_macro2::TokenStream>,
) -> (Subject, proc_macro2::TokenStream) {
    let mut has_default = None;
    subject.data.variants = subject
        .data
        .variants
        .into_iter()
        .filter_map(|mut variant| {
            if variant.discriminant.is_some() && variant.ident == DEFAULT_VARIANT_NAME {
                println!("got here");
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
        subject,
        has_default
            .or(default_else)
            .or_else(|| Some(quote::quote!("".to_string())))
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Type};

    use crate::utils::UniqueHashId;

    #[test]
    fn hash_type() {
        let ty1: Type = parse_quote!(&'a mut Typer<T, i32, Target = A<i32>>);
        let ty2: Type = parse_quote!(&'a mut Typer<T, usize, Target = A<i32>>);

        let ty_string1 = UniqueHashId(&ty1).get_unique_string();
        let ty_string2 = UniqueHashId(&ty2).get_unique_string();

        // If both are OK, then both must be different, making them
        // unique.
        assert_eq!("_8289286104171367827", ty_string1);
        assert_eq!("_2029180714094036370", ty_string2);
    }
}
