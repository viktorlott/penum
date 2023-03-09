use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, BoundLifetimes, Lifetime, Token, Type, TraitBoundModifier, ParenthesizedGenericArguments, PathArguments, Path, parenthesized,
};
use proc_macro2::TokenStream;

pub struct WhereClause {
    pub where_token: Token![where],
    pub predicates: Punctuated<WherePredicate, Token![,]>,
}

pub enum WherePredicate {
    /// e.g `for<'c> Foo<'c>: Trait<'c>`.
    Type(PredicateType),
    /// e.g 'a: 'b + 'c`.
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

impl Parse for WhereClause {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(WhereClause {
            where_token: input.parse()?,
            predicates: {
                let mut predicates = Punctuated::new();
                loop {
                    if input.is_empty()
                        || input.peek(token::Brace)
                        || input.peek(Token![,])
                        || input.peek(Token![;])
                        || input.peek(Token![:]) && !input.peek(Token![::])
                        || input.peek(Token![=])
                    {
                        break;
                    }
                    let value = input.parse()?;
                    predicates.push_value(value);
                    if !input.peek(Token![,]) {
                        break;
                    }
                    let punct = input.parse()?;
                    predicates.push_punct(punct);
                }
                predicates
            },
        })
    }
}

impl Parse for WherePredicate {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Lifetime) && input.peek2(Token![:]) {
            Ok(WherePredicate::Lifetime(PredicateLifetime {
                lifetime: input.parse()?,
                colon_token: input.parse()?,
                bounds: {
                    let mut bounds = Punctuated::new();
                    loop {
                        if input.is_empty()
                            || input.peek(token::Brace)
                            || input.peek(Token![,])
                            || input.peek(Token![;])
                            || input.peek(Token![:])
                            || input.peek(Token![=])
                        {
                            break;
                        }
                        let value = input.parse()?;
                        bounds.push_value(value);
                        if !input.peek(Token![+]) {
                            break;
                        }
                        let punct = input.parse()?;
                        bounds.push_punct(punct);
                    }
                    bounds
                },
            }))
        } else {
            Ok(WherePredicate::Type(PredicateType {
                lifetimes: input.parse()?,
                bounded_ty: input.parse()?,
                colon_token: input.parse()?,
                bounds: {
                    let mut bounds = Punctuated::new();
                    loop {
                        if input.is_empty()
                            || input.peek(token::Brace)
                            || input.peek(Token![,])
                            || input.peek(Token![;])
                            || input.peek(Token![:]) && !input.peek(Token![::])
                            || input.peek(Token![=])
                        {
                            break;
                        }
                        let value = input.parse()?;
                        bounds.push_value(value);
                        if !input.peek(Token![+]) {
                            break;
                        }
                        let punct = input.parse()?;
                        bounds.push_punct(punct);
                    }
                    bounds
                },
            }))
        }
    }
}

impl Parse for TypeParamBound {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Lifetime) {
            return input.parse().map(TypeParamBound::Lifetime);
        }

        if input.peek(token::Paren) {
            let content;
            let paren_token = parenthesized!(content in input);
            let mut bound: TraitBound = content.parse()?;
            bound.paren_token = Some(paren_token);
            return Ok(TypeParamBound::Trait(bound));
        }

        input.parse().map(TypeParamBound::Trait)
    }
}

impl Parse for TraitBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let dispatch = if input.peek(Token![^]) {

            // TODO: We should also check if the trait that comes after exists in over "allow" list.
            //       The allow list should for now contain core traits.
            // 
            Some(input.parse()?) // We do all this just to support `^` (dispatch) symbol.
        } else {
            None
        };
        let modifier: TraitBoundModifier = input.parse()?;
        let lifetimes: Option<BoundLifetimes> = input.parse()?;

        let mut path: Path = input.parse()?;
        if path.segments.last().unwrap().arguments.is_empty()
            && (input.peek(token::Paren) || input.peek(Token![::]) && input.peek3(token::Paren))
        {
            input.parse::<Option<Token![::]>>()?;
            let args: ParenthesizedGenericArguments = input.parse()?;
            let parenthesized = PathArguments::Parenthesized(args);
            path.segments.last_mut().unwrap().arguments = parenthesized;
        }

        Ok(TraitBound {
            paren_token: None,
            dispatch,
            modifier,
            lifetimes,
            path,
        })
    }
}

impl ToTokens for WhereClause {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.predicates.is_empty() {
            self.where_token.to_tokens(tokens);
            self.predicates.to_tokens(tokens);
        }
    }
}

impl ToTokens for WherePredicate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            WherePredicate::Type(t) => t.to_tokens(tokens),
            WherePredicate::Lifetime(l) => l.to_tokens(tokens),
        }
    }
}

impl ToTokens for TypeParamBound {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypeParamBound::Trait(t) => t.to_tokens(tokens),
            TypeParamBound::Lifetime(l) => l.to_tokens(tokens),
        }
    }
}

impl ToTokens for PredicateType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lifetimes.to_tokens(tokens);
        self.bounded_ty.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.bounds.to_tokens(tokens);
    }
}

impl ToTokens for PredicateLifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lifetime.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.bounds.to_tokens(tokens);
    }
}

impl ToTokens for TraitBound {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let to_tokens = |tokens: &mut TokenStream| {
            self.modifier.to_tokens(tokens);
            self.lifetimes.to_tokens(tokens);
            self.path.to_tokens(tokens);
        };
        match &self.paren_token {
            Some(paren) => paren.surround(tokens, to_tokens),
            None => to_tokens(tokens),
        }
    }
}

