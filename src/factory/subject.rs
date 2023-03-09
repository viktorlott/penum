use proc_macro2::Ident;
use syn::{Attribute, DataEnum, Generics, Visibility};

mod parse;
mod to_tokens;

pub struct Subject {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataEnum,
}
