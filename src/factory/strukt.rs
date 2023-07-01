use proc_macro2::{Ident, Literal};
use syn::{
    punctuated::Punctuated,
    token::{self},
    Attribute, Expr, ExprMacro, Generics, Macro, Token, Type, Visibility,
};

mod parse;
mod to_tokens;

#[derive(Clone, Debug)]
pub struct Strukt {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: DataStruct,
}

#[derive(Clone, Debug)]
pub struct DataStruct {
    pub struct_token: Token![struct],
    pub fields: FieldsKind,
    pub semi_token: Option<Token![;]>,
}

#[derive(Clone, Debug)]
pub enum FieldsKind {
    /// Named fields of a struct or struct variant such as `Point { x: f64,
    /// y: f64 }`.
    Named(FieldsNamed),

    /// Unnamed fields of a tuple struct or tuple variant such as `Some(T)`.
    Unnamed(FieldsUnnamed),
    /// Unit struct or unit variant such as `None`.
    Unit,
}

#[derive(Clone, Debug)]
pub struct FieldsNamed {
    pub brace_token: token::Brace,
    pub named: Punctuated<FieldDisc, Token![,]>,
}

#[derive(Clone, Debug)]
pub struct FieldsUnnamed {
    pub paren_token: token::Paren,
    pub unnamed: Punctuated<FieldDisc, Token![,]>,
}

#[derive(Clone, Debug)]
pub struct FieldDisc {
    /// Attributes tagged on the field.
    pub attrs: Vec<Attribute>,

    /// Visibility of the field.
    pub vis: Visibility,

    /// Name of the field, if any.
    ///
    /// Fields of tuple structs have no names.
    pub ident: Option<Ident>,

    pub colon_token: Option<Token![:]>,

    /// Type of the field.
    pub ty: Type,

    pub discriminant: Option<(Token![=], Expr)>,
}

pub struct ExprCall(pub Type, pub Expr);
