use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Field, Ident, LitInt, Token,
};

use crate::utils::parse_pattern;

use super::{Group, Parameter, PatternFrag, PenumExpr};

impl Parse for PenumExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pattern: input.call(parse_pattern)?,
            where_clause: {
                if input.peek(Token![where]) {
                    Some(input.parse()?)
                } else {
                    None
                }
            },
        })
    }
}

impl Parse for PatternFrag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
        }

        Ok(PatternFrag {
            ident: input.parse()?,
            group: input.parse()?,
        })
    }
}

impl Parse for Group {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(if input.peek(token::Brace) {
            let token = braced!(content in input);
            Group::Named {
                parameters: content.parse_terminated(Parameter::parse)?,
                delimiter: token,
            }
        } else if input.peek(token::Paren) {
            let token = parenthesized!(content in input);
            Group::Unnamed {
                parameters: content.parse_terminated(Parameter::parse)?,
                delimiter: token,
            }
        } else {
            Group::Unit
        })
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![..]) && input.peek2(LitInt) {
            Parameter::Range(input.parse()?)
        } else if input.peek(Token![..]) {
            Parameter::Variadic(input.parse()?)
        } else if input.peek(Ident) && input.peek2(Token![:]) {
            Parameter::Regular(input.call(Field::parse_named)?)
        } else {
            Parameter::Regular(input.call(Field::parse_unnamed)?)
        })
    }
}
