use std::borrow::Borrow;

use proc_macro2::Ident;
use syn::{
    punctuated::Punctuated, token, BoundLifetimes, Lifetime, Path, Token, TraitBoundModifier, Type
};

mod parse;
mod to_tokens;

pub struct WhereClause {
    pub where_token: Token![where],
    pub predicates: Punctuated<WherePredicate, Token![,]>,
}

pub enum WherePredicate {
    Type(PredicateType),
    Lifetime(PredicateLifetime), // NOT SUPPORTED
}

pub struct PredicateType {
    pub lifetimes: Option<BoundLifetimes>,
    pub bounded_ty: Type,
    pub colon_token: Token![:],
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

pub struct PredicateLifetime {
    pub lifetime: Lifetime,
    pub colon_token: Token![:],
    pub bounds: Punctuated<Lifetime, Token![+]>,
}

pub enum TypeParamBound {
    Trait(TraitBound),
    #[allow(dead_code)]
    Lifetime(Lifetime),
}

#[derive(Clone)]
pub struct TraitBound {
    pub paren_token: Option<token::Paren>,
    pub dispatch: Option<Token![^]>,
    pub modifier: TraitBoundModifier,
    pub lifetimes: Option<BoundLifetimes>,
    pub path: Path,
}



impl TypeParamBound {
    pub fn get_dispatchable_trait_bound(&self) -> Option<&TraitBound> {
        match self {
            TypeParamBound::Trait(tb) => tb.dispatch.map(|_| tb),
            _ => None,
        }
    }
}

impl TraitBound {
    pub fn get_ident(&self) -> &Ident {
        self
            .path
            .segments
            .last()
            .expect("dispatchable trait to have a name")
            .ident.borrow()
    }
}