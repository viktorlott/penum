use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

use proc_macro2::{Ident, Span};
use quote::{format_ident, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self},
    visit::{visit_binding, visit_path, Visit},
    Token, TraitBound, Type, TypeImplTrait, Variant, WhereClause,
};

use crate::factory::PatFrag;

type GenericIdent = String;
#[derive(Clone, Debug)]
pub struct TypeId(pub Ident, pub Option<Type>);
#[derive(Default, Debug)]
pub struct PolymorphicMap(BTreeMap<GenericIdent, BTreeSet<TypeId>>);

pub struct UniqueIdentifier(Vec<String>);

/// Fix these later
impl PolymorphicMap {
    /// First we check if pty (T) exists in polymorphicmap.
    /// If it exists, insert new concrete type.
    pub fn polymap_insert(&mut self, pty: String, ity: TypeId) {
        if let Some(set) = self.0.get_mut(&pty) {
            set.insert(ity);
        } else {
            self.0.insert(pty.clone(), vec![ity].into_iter().collect());
        }
    }
}

impl<'ast> Visit<'ast> for UniqueIdentifier {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(item) = node.segments.last() {
            self.0.push(item.ident.to_string());
        }
        visit_path(self, node)
    }

    fn visit_binding(&mut self, node: &'ast syn::Binding) {
        self.0.push(node.ident.to_string());
        visit_binding(self, node);
    }
}

impl Deref for PolymorphicMap {
    type Target = BTreeMap<String, BTreeSet<TypeId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

pub fn string<T: ToTokens>(x: &T) -> String {
    x.to_token_stream().to_string()
}

#[allow(dead_code)]
pub fn ident_impl(imptr: &TypeImplTrait) -> Ident {
    format_ident!(
        "__IMPL_{}",
        string(&imptr.bounds)
            .replace(' ', "_")
            .replace(['?', '\''], "")
    )
}

pub fn no_match_found(item: &impl ToTokens, pat: &str) -> String {
    format!(
        "`{}` doesn't match pattern `{}`",
        item.to_token_stream(),
        pat
    )
}

/// We use a `dummy` identifier to store our
/// bounds under.
///
/// impl Trait<Ty1, Target = Ty3>: "_IMPL_Trait_Ty1_Target_Ty3"
/// impl Trait<Ty2>: "_IMPL_Trait_Ty2"
pub fn get_unique_trait_bound_id(value: &TraitBound, tag: &Ident, index: usize) -> Ident {
    let mut unique = UniqueIdentifier(vec![]);
    unique.visit_trait_bound(value);
    format_ident!(
        "__IMPL_{}_{}_{}",
        tag,
        unique.0.join("_"),
        index,
        span = Span::call_site()
    )
}

pub fn get_unique_type_string(value: &Type) -> String {
    let mut unique = UniqueIdentifier(vec![]);
    unique.visit_type(value);
    unique.0.join("_")
}

impl TypeId {
    pub fn get_type(&self) -> Option<&Type> {
        self.1.as_ref()
    }
}

impl From<&Type> for TypeId {
    fn from(value: &Type) -> Self {
        let mut unique = UniqueIdentifier(vec![]);
        unique.visit_type(value);
        Self(format_ident!("{}", unique.0.join("_")), Some(value.clone()))
    }
}

impl PartialEq for TypeId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for TypeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for TypeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Eq for TypeId {}

// let ty_span = pred.span();
// let assert_sync = quote_spanned!{ty_span=>
//     struct _AssertSync where #pred: Sync;
// };
// println!("{}", assert_sync);
