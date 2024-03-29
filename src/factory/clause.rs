use proc_macro2::Ident;
use quote::format_ident;
use syn::{
    punctuated::Punctuated, token, BoundLifetimes, Lifetime, Token, TraitBoundModifier, Type,
};

mod parse;
mod to_tokens;

#[derive(Debug)]
pub struct WhereClause {
    pub where_token: Token![where],
    pub predicates: Punctuated<WherePredicate, Token![,]>,
}

#[derive(Debug)]
pub enum WherePredicate {
    Type(PredicateType),
    Lifetime(PredicateLifetime), // NOT SUPPORTED
}

#[derive(Debug)]
pub struct PredicateType {
    pub lifetimes: Option<BoundLifetimes>,
    pub bounded_ty: Type,
    pub colon_token: Token![:],
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

#[derive(Debug)]
pub struct PredicateLifetime {
    pub lifetime: Lifetime,
    pub colon_token: Token![:],
    pub bounds: Punctuated<Lifetime, Token![+]>,
}

#[derive(Debug)]
pub enum TypeParamBound {
    Trait(TraitBound),
    #[allow(dead_code)]
    Lifetime(Lifetime),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TraitBound {
    pub paren_token: Option<token::Paren>,
    pub dispatch: Option<Token![^]>,
    pub modifier: TraitBoundModifier,
    pub lifetimes: Option<BoundLifetimes>,
    pub ty: Type,
}

impl TypeParamBound {
    /// FIXME: Only get methods with receivers. `fn method()` vs `fn method(&self)`.
    pub fn get_dispatchable_trait_bound(&self) -> Option<&TraitBound> {
        match self {
            TypeParamBound::Trait(tb) => tb.dispatch.map(|_| tb),
            _ => None,
        }
    }
}

impl TraitBound {
    pub fn get_ident(&self) -> Ident {
        if let Type::Path(p) = &self.ty {
            p.path
                .segments
                .last()
                .expect("dispatchable trait to have a name")
                .ident
                .clone()
        } else {
            format_ident!("{}", "omg")
        }
    }
}
