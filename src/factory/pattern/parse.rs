use proc_macro2::{Span, TokenStream};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Field, Ident, LitInt, LitStr, Token, Type,
};

use crate::factory::{TraitBound, WhereClause};

use super::{PatComposite, PatFieldKind, PatFrag, PenumExpr};

struct ImplExpr {
    impl_token: token::Impl,
    trait_bound: TraitBound,
    for_token: token::For,
    tys: Vec<Type>,
}

impl Parse for ImplExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            impl_token: input.parse()?,
            trait_bound: {
                let mut tb: TraitBound = input.parse()?;

                // Always dispatch for impl expressions
                if tb.dispatch.is_none() {
                    tb.dispatch = Some(token::Caret(Span::call_site()));
                }

                tb
            },
            for_token: input.parse()?,
            tys: {
                if input.peek(token::Brace) {
                    let content;
                    let _ = braced!(content in input);
                    let mut tys = vec![];

                    loop {
                        if content.is_empty() {
                            break;
                        }

                        let ty: Type = content.parse()?;
                        tys.push(ty);

                        if !content.peek(token::Comma) {
                            break;
                        } else {
                            let _: token::Comma = content.parse()?;
                        }
                    }
                    tys
                } else {
                    vec![input.parse()?]
                }
            },
        })
    }
}

impl ImplExpr {
    fn into_clause(self) -> WhereClause {
        let Self {
            trait_bound, tys, ..
        } = self;

        let mut bounds = TokenStream::new();

        for (index, ty) in tys.iter().enumerate() {
            bounds.extend(quote::quote!(#ty: #trait_bound));

            if index != tys.len() - 1 {
                bounds.extend(quote::quote!(,));
            }
        }

        syn::parse_quote!(where #bounds)
    }
}

impl Parse for PenumExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let pat: LitStr = input.parse()?;
            let penum: PenumExpr = pat.parse_with(PenumExpr::parse)?;
            return Ok(penum);
        }

        if input.peek(token::Where) || input.peek(token::For) || input.peek(token::Impl) {
            if ImplExpr::parse(&input.fork()).is_ok() {
                let impl_expr: ImplExpr = input.parse()?;

                return Ok(Self {
                    pattern: vec![PatFrag {
                        ident: None,
                        group: PatComposite::Inferred,
                    }],
                    clause: Some(impl_expr.into_clause()),
                });
            }

            return Ok(Self {
                pattern: vec![PatFrag {
                    ident: None,
                    group: PatComposite::Inferred,
                }],
                clause: Some(input.parse()?),
            });
        }

        if input.peek(Ident) && input.peek2(token::Eq) {
            let _: Ident = input.parse()?;
            let _: token::Eq = input.parse()?;

            if input.peek(token::Gt) {
                let _: token::Gt = input.parse()?;
            }
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

impl Parse for PatComposite {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(if input.peek(token::Brace) {
            let token = braced!(content in input);
            PatComposite::Named {
                parameters: content.parse_terminated(PatFieldKind::parse)?,
                delimiter: token,
            }
        } else if input.peek(token::Paren) {
            let token = parenthesized!(content in input);
            PatComposite::Unnamed {
                parameters: content.parse_terminated(PatFieldKind::parse)?,
                delimiter: token,
            }
        } else {
            PatComposite::Unit
        })
    }
}

impl Parse for PatFieldKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![..]) && input.peek2(LitInt) {
            PatFieldKind::Range(input.parse()?)
        } else if input.peek(Token![..]) {
            PatFieldKind::Variadic(input.parse()?)
        } else if input.peek(Ident) && input.peek2(Token![:]) {
            PatFieldKind::Field(input.call(Field::parse_named)?)
        } else {
            PatFieldKind::Field(input.call(Field::parse_unnamed)?)
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
            group: PatComposite::Inferred,
        })
    } else {
        Ok(PatFrag {
            ident: input.parse()?,
            group: input.parse()?,
        })
    }
}
