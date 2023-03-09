use syn::{
    punctuated::Punctuated, token, BoundLifetimes, Lifetime, Path, Token, TraitBoundModifier, Type,
};

mod parse;
mod to_tokens;

pub struct WhereClause {
    pub where_token: Token![where],
    pub predicates: Punctuated<WherePredicate, Token![,]>,
}

pub enum WherePredicate {
    Type(PredicateType),
    Lifetime(PredicateLifetime), // NOTE: Might not even need this... it's not even supported
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

pub struct TraitBound {
    pub paren_token: Option<token::Paren>,
    pub dispatch: Option<Token![^]>,
    pub modifier: TraitBoundModifier,
    /// The `for<'a>` in `for<'a> Foo<&'a T>`
    pub lifetimes: Option<BoundLifetimes>,
    /// The `Foo<&'a T>` in `for<'a> Foo<&'a T>`
    pub path: Path,
}
