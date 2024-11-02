use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    ops::Deref,
};

use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, spanned::Spanned, Type};

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

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Type};

    use crate::polym::UniqueHashId;

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
