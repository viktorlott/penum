use proc_macro2::Ident;
use syn::{Attribute, DataEnum, Fields, Generics, Visibility};

use super::ComparableItem;

mod parse;
mod to_tokens;

pub struct Subject {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataEnum,
}

impl Subject {
    pub fn get_comparable_fields(&self) -> impl Iterator<Item = ComparableItem<Fields>> {
        self.data
            .variants
            .iter()
            .map(|variant| ComparableItem::from(&variant.fields))
    }
}
