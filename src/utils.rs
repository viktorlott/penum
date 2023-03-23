#![allow(unused)]
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    io::Sink,
    ops::Deref,
};

use ::rustfmt::{format_input, Input};
use proc_macro2::{Ident, Span};
use quote::{format_ident, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self},
    visit::{visit_binding, visit_lifetime, visit_path, visit_type_reference, Visit},
    Token, TraitBound, Type, TypeImplTrait, Variant, WhereClause,
};

use crate::{factory::PatFrag, penum::Stringify};

#[derive(Default, Debug)]
pub struct PolymorphicMap<K: Hash, V: Hash>(BTreeMap<K, BTreeSet<V>>);

/// Fix these later
impl<K: Hash, V: Hash> PolymorphicMap<UniqueHashId<K>, UniqueHashId<V>>
where
    UniqueHashId<K>: Ord,
    UniqueHashId<V>: Ord,
{
    pub fn polymap_insert(&mut self, pty: UniqueHashId<K>, ity: UniqueHashId<V>) {
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

#[derive(Hash, Debug, Clone)]
pub struct UniqueHashId<T: Hash>(pub T);

impl<T: Hash> UniqueHashId<T> {
    pub fn get_unique_ident(&self) -> Ident
    where
        T: Spanned + ToTokens,
    {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        format_ident!("__Unique_Id_{}", hasher.finish(), span = self.0.span())
    }

    pub fn get_unique_string(&self) -> String {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        format!("__Unique_Id_{}", hasher.finish())
    }
}

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
        Self(parse_quote!(!))
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

pub fn parse_pattern(input: ParseStream) -> syn::Result<Vec<PatFrag>> {
    let mut shape = vec![input.call(parse_pattern_fragment)?];

    while input.peek(token::Or) {
        let _: token::Or = input.parse()?;
        shape.push(input.call(parse_pattern_fragment)?);
    }

    Ok(shape)
}

pub fn parse_pattern_fragment(input: ParseStream) -> syn::Result<PatFrag> {
    if input.peek(Token![$]) {
        let _: Token![$] = input.parse()?;
    }
    Ok(PatFrag {
        ident: input.parse()?,
        group: input.parse()?,
    })
}

pub fn parse_enum(
    input: ParseStream,
) -> syn::Result<(
    Option<WhereClause>,
    token::Brace,
    Punctuated<Variant, Token![,]>,
)> {
    let where_clause = input.parse()?;

    let content;
    let brace = braced!(content in input);
    let variants = content.parse_terminated(Variant::parse)?;

    Ok((where_clause, brace, variants))
}

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

pub fn into_unique_ident(value: &str, tag: &Ident, span: Span) -> Ident {
    format_ident!("__IMPL_{}_{}_", tag, value, span = span)
}

pub fn get_unique_assertion_statement(
    ident: &Ident,
    ty: &Type,
    pred_idx: usize,
    idx: usize,
) -> Ident {
    format_ident!(
        "__Assert_{}_{}_{}_{}",
        ident,
        UniqueHashId(ty).get_unique_string(),
        pred_idx,
        idx,
        span = ty.span()
    )
}

pub fn format_code(orig: String) -> String {
    format_input(Input::Text(orig), &<_>::default(), None::<&mut Sink>)
        .map(|(_, v, _)| v.into_iter().next())
        .ok()
        .flatten()
        .map(|(_, m)| m.to_string())
        .expect("source_code input should be formatted")
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
        assert_eq!("__Unique_Id_8289286104171367827", ty_string1);
        assert_eq!("__Unique_Id_2029180714094036370", ty_string2);
    }
}
