use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Field, Ident, LitInt, Token,
};

use crate::utils::parse_pattern;

use super::{Composite, ParameterKind, PatFrag, PenumExpr};

impl Parse for PenumExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pattern: input.call(parse_pattern)?,
            clause: {
                if input.peek(Token![where]) {
                    Some(input.parse()?)
                } else {
                    None
                }
            },
        })
    }
}

impl Parse for PatFrag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
        }

        Ok(PatFrag {
            ident: input.parse()?,
            group: input.parse()?,
        })
    }
}

impl Parse for Composite {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(if input.peek(token::Brace) {
            let token = braced!(content in input);
            Composite::Named {
                parameters: content.parse_terminated(ParameterKind::parse)?,
                delimiter: token,
            }
        } else if input.peek(token::Paren) {
            let token = parenthesized!(content in input);
            Composite::Unnamed {
                parameters: content.parse_terminated(ParameterKind::parse)?,
                delimiter: token,
            }
        } else {
            Composite::Unit
        })
    }
}

impl Parse for ParameterKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![..]) && input.peek2(LitInt) {
            ParameterKind::Range(input.parse()?)
        } else if input.peek(Token![..]) {
            ParameterKind::Variadic(input.parse()?)
        } else if input.peek(Ident) && input.peek2(Token![:]) {
            ParameterKind::Regular(input.call(Field::parse_named)?)
        } else {
            ParameterKind::Regular(input.call(Field::parse_unnamed)?)
        })
    }
}
