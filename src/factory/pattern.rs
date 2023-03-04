use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::{IntoIter, Iter, Punctuated},
    spanned::Spanned,
    token, ExprRange, Field,
    Fields::{self},
    FieldsNamed, FieldsUnnamed, Ident, LitInt, Token, Variant, WhereClause,
};

use crate::{
    error::Diagnostic,
    utils::{parse_shapes, string, TypeMap},
};

use Scope::*;

pub type PunctuatedFieldKinds = Punctuated<FieldKind, Token![,]>;

/// A pattern can contain multiple shapes, but only one where clause.
/// e.g. `(_) | (_, _) | { name: _, age: usize }`
pub struct Pattern {
    pub shapes: Vec<Shape>,
    pub where_clause: Option<WhereClause>,
}

pub struct Shape {
    pub ident: Option<Ident>,
    pub scope: Scope,
}

pub enum Scope {
    // TODO: Replace `Punctuated` with custom sequence type
    Named(PunctuatedFieldKinds, token::Brace),
    Unnamed(PunctuatedFieldKinds, token::Paren),
    Unit,
}

// TODO: Variadic should be with 3 dots?
/// 
/// ```rust
///     (T, U)
///     (T, U, ..)
///     (T, U, ..10)   // NOT SUPPORTED YET
///     (T, U, ...)    // NOT SUPPORTED YET
///     (T, U, ..Copy) // NOT SUPPORTED YET
/// ```
pub enum FieldKind {
    Field(Field),
    Variadic(Token![..]),
    Range(ExprRange),
}

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            shapes: input.call(parse_shapes)?,
            where_clause: input.parse()?,
        })
    }
}

impl Parse for Shape {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
        }

        Ok(Shape {
            ident: input.parse()?,
            scope: input.parse()?,
        })
    }
}

impl Parse for Scope {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(if input.peek(token::Brace) {
            let token = braced!(content in input);
            Scope::Named(content.parse_terminated(FieldKind::parse)?, token)
        } else if input.peek(token::Paren) {
            let token = parenthesized!(content in input);
            Scope::Unnamed(content.parse_terminated(FieldKind::parse)?, token)
        } else {
            Scope::Unit
        })
    }
}

impl Parse for FieldKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![..]) && input.peek2(LitInt) {
            FieldKind::Range(input.parse()?)
        } else if input.peek(Token![..]) {
            FieldKind::Variadic(input.parse()?)
        } else if input.peek(Ident) && input.peek2(Token![:]) {
            FieldKind::Field(input.call(Field::parse_named)?)
        } else {
            FieldKind::Field(input.call(Field::parse_unnamed)?)
        })
    }
}

impl ToTokens for Shape {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.scope.to_tokens(tokens);
    }
}

impl ToTokens for FieldKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldKind::Field(f) => f.to_tokens(tokens),
            FieldKind::Variadic(v) => v.to_tokens(tokens),
            FieldKind::Range(r) => r.to_tokens(tokens),
        }
    }
}

impl ToTokens for Scope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Scope::Named(content, b) => b.surround(tokens, |tokens| content.to_tokens(tokens)),
            Scope::Unnamed(content, p) => p.surround(tokens, |tokens| content.to_tokens(tokens)),
            Scope::Unit => (),
        }
    }
}

impl From<&Fields> for Scope {
    fn from(value: &Fields) -> Self {
        match value {
            Fields::Named(FieldsNamed { named, brace_token }) => {
                Self::Named(parse_quote!(#named), *brace_token)
            }
            Fields::Unnamed(FieldsUnnamed {
                unnamed,
                paren_token,
            }) => Self::Unnamed(parse_quote!(#unnamed), *paren_token),
            Fields::Unit => Self::Unit,
        }
    }
}

impl FieldKind {
    pub fn is_field(&self) -> bool {
        matches!(self, FieldKind::Field(_))
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self, FieldKind::Variadic(_))
    }

    pub fn is_range(&self) -> bool {
        matches!(self, FieldKind::Range(_))
    }
}

impl Scope {
    pub fn len(&self) -> usize {
        match self {
            Named(n, _) => n.len(),
            Unnamed(u, _) => u.len(),
            Unit => 0,
        }
    }

    pub fn iter(&self) -> Iter<FieldKind> {
        match self {
            // TODO: Empty Iter is unsupported right now.
            Scope::Unit => panic!("Empty Iter is unsupported right now."),
            Scope::Named(n, _) => n.iter(),
            Scope::Unnamed(u, _) => u.iter(),
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Scope::Unit)
    }

    pub fn has_variadic(&self) -> bool {
        match self {
            // TODO: Should probably just check last field, or should it have support for (T, .., U)?
            Named(n, _) => n.iter().any(|fk| fk.is_variadic()),
            Unnamed(u, _) => u.iter().any(|fk| fk.is_variadic()),
            Unit => false,
        }
    }

    pub fn last_is_variadic(&self) -> bool {
        match self {
            Named(n, _) => matches!(n.iter().last().take(), Some(val) if val.is_variadic()),
            Unnamed(u, _) => matches!(u.iter().last().take(), Some(val) if val.is_variadic()),
            Unit => false,
        }
    }

    pub fn count_by_include(&self, mut f: impl FnMut(&FieldKind) -> bool) -> usize {
        match self {
            Named(n, _) => n.iter().fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc }),
            Unnamed(u, _) => u.iter().fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc }),
            Unit => 0,
        }
    }
}


impl IntoIterator for Scope {
    type Item = FieldKind;
    type IntoIter = IntoIter<FieldKind>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Scope::Unit => Punctuated::<FieldKind, ()>::new().into_iter(),
            Scope::Named(n, _) => n.into_iter(),
            Scope::Unnamed(u, _) => u.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Scope {
    type Item = &'a FieldKind;
    type IntoIter = Iter<'a, FieldKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FieldKind {
    fn get_field(&self) -> Option<&Field> {
        match self {
            FieldKind::Field(field) => Some(field),
            _ => None,
        }
    }
}

impl Pattern {
    pub fn pattern_matching_on<'a>(
        &'a self,
        variant_item: &'a Variant,
    ) -> Option<(&'a Scope, &'a Fields)> {
        self.shapes
            .iter()
            .find_map(pattern_match(&variant_item.fields))
    }

    pub fn validate_and_collect(
        &self,
        variant: &Variant,
        types: &mut TypeMap,
        error: &mut Diagnostic,
    ) {
        // A pattern can contain multiple shapes, e.g. `(_) | (_, _) | { name: _, age: usize }`
        // So if the variant_item matches a shape, we associate the pattern with the variant.
        let Some((scope, ifields)) = self.pattern_matching_on(variant) else {
            return error.extend(
                variant.fields.span(),
                format!(
                    "`{}` doesn't match pattern `{}`",
                    variant.to_token_stream(),
                    // Fix this sh*t
                    self.shapes
                        .iter()
                        .map(|s| s.to_token_stream().to_string())
                        .reduce(|acc, s| if acc.is_empty() {s} else {format!("{acc} | {s}")}).unwrap()
                ),
            );
        };

        // TODO: No support for empty unit iter, yet...
        if scope.is_unit() {
            return;
        }

        let mut variadic_or_range = false;
        for (p, item) in scope.into_iter().zip(ifields.into_iter()) {
            // TODO: Right now, if a variadic is found, we skip validating the rest of the fields
            //       Might want to change this in the future.
            if variadic_or_range {
                continue;
            }

            let Some(pat) = p.get_field() else {
                variadic_or_range = true;
                continue;
            };

            let (pty, ity) = (string(&pat.ty), string(&item.ty));
            let is_generic = pty.eq("_") || pty.to_uppercase().eq(&pty);

            if !is_generic && pty != ity {
                error.extend(item.ty.span(), format!("Found {ity} but expected {pty}."));
                continue;
            }

            if let Some(set) = types.get_mut(&pty) {
                set.insert(ity);
            } else {
                types.insert(pty, vec![ity].into_iter().collect());
            }
        }
    }
}

// TODO: Change this sh*t, or add some
trait PatternMatcher {
    fn get_matches(&self) -> (&Scope, &Fields);

    fn has_same_len(&self) -> bool {
        matches!(self.get_matches(), (p, i) if p.len() == i.len())
    }

    fn has_variadic(&self) -> bool {
        matches!(self.get_matches(), (p, _) if p.has_variadic())
    }
    
    fn has_variadic_last(&self) -> bool {
        matches!(self.get_matches(), (p, _) if p.last_is_variadic())
    }

    fn has_minimum_matches(&self) -> bool {
        matches!(self.get_matches(), (p, i) if p.count_by_include(|fk| fk.is_field()) <= i.len())
    }
}

impl<'a> PatternMatcher for (&'a Scope, &'a Fields) {
    fn get_matches(&self) -> (&'a Scope, &'a Fields) {
        (self.0, self.1)
    }
}

fn pattern_match<'a>(
    fields: &'a Fields,
) -> impl FnMut(&'a Shape) -> Option<(&'a Scope, &'a Fields)> {
    move |shape: &Shape| match (&shape.scope, fields) {
        tail @ ((&Named(..), &Fields::Named(..)) | (&Unnamed(..), &Fields::Unnamed(..)))
        // TODO: Add support for variadic and range patterns
        // This is kind of expensive..
            => if tail.has_variadic_last() {
                if tail.has_minimum_matches() {
                    Some(tail)
                } else {
                    None
                }
            } else if tail.has_same_len() {
                Some(tail)
            } else {
                None
            }
        tail @ (Unit, Fields::Unit) => Some(tail),
        _ => None,
    }
}
