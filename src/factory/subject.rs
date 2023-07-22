use proc_macro2::Ident;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, DataEnum, Fields, Generics, Variant,
    Visibility,
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
}
