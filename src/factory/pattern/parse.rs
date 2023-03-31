use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Field, Ident, LitInt, LitStr, Token,
};

use super::{Composite, ParameterKind, PatFrag, PenumExpr};

impl Parse for PenumExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(token::Eq) {
            let _: Ident = input.parse()?;
            let _: token::Eq = input.parse()?;

            if input.peek(token::Gt) {
                let _: token::Gt = input.parse()?;
            }
        }

        if input.peek(LitStr) {
            let pat: LitStr = input.parse()?;
            let penum: PenumExpr = pat.parse_with(PenumExpr::parse)?;
            return Ok(penum);
        }

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

    if input.peek(Token![_]) {
        let _: Token![_] = input.parse()?;
        Ok(PatFrag {
            ident: None,
            group: Composite::Inferred,
        })
    } else {
        Ok(PatFrag {
            ident: input.parse()?,
            group: input.parse()?,
        })
    }
}
