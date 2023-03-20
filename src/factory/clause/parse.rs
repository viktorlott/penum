use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, BoundLifetimes, Lifetime, ParenthesizedGenericArguments, Path, PathArguments, Token,
    TraitBoundModifier,
};

use super::*;

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
            // let lifetime: Lifetime = input.parse()?;
            // return Err(Error::new(
            //     lifetime.span(),
            //     "Lifetimes are not supported in Penums where clause".to_string(),
            // ));
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
            let token: Token![^] = input.parse()?;
            // let name_trait: Ident = input.fork().parse()?;

            // TODO: We should also check if the trait that comes after exists in our "allow" list.
            //       The allow list should for now contain core traits.
            //       We do all this just to support `^` (dispatch) symbol.

            Some(token)
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
