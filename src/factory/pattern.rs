use quote::ToTokens;
use syn::{
    parse_quote,
    punctuated::{IntoIter, Iter, Punctuated},
    spanned::Spanned,
    token, ExprRange, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Token, Variant,
};

use crate::{
    error::Diagnostic,
    utils::{string, PolymorphicMap},
};

use super::{pattern_match, PunctuatedParameters, WhereClause};

mod parse;
mod to_tokens;

/// A Penum expression consists of one or more patterns, and an optional WhereClause.
pub struct PenumExpr {
    pub pattern: Vec<PatternFrag>,
    pub where_clause: Option<WhereClause>,
}

pub struct PatternFrag {
    pub ident: Option<Ident>,
    pub group: Group,
}

// TODO: Replace `Punctuated` with custom sequence type
pub enum Group {
    Named {
        parameters: PunctuatedParameters,
        delimiter: token::Brace,
    },
    Unnamed {
        parameters: PunctuatedParameters,
        delimiter: token::Paren,
    },
    Unit,
}

pub enum Parameter {
    Regular(Field),
    ///
    /// ```rust
    ///     (T, U, ..)
    ///     (T, U, ..10)    // NOT SUPPORTED
    ///     (T, U, ...)     // NOT SUPPORTED
    ///     (T, U, ..Copy)  // NOT SUPPORTED
    ///     (T, U, Copy..2) // NOT SUPPORTED
    /// ```
    Variadic(Token![..]),
    Range(ExprRange),
}

// TODO: Refacto this when you can
impl PenumExpr {
    const PLACEHOLDER_SYMBOL: &str = "_";

    pub fn pattern_matching_on<'a>(
        &'a self,
        variant_item: &'a Variant,
    ) -> Option<(&'a Group, &'a Fields)> {
        self.pattern
            .iter()
            .find_map(pattern_match(&variant_item.fields))
    }

    pub fn print_pattern(&self) -> String {
        self.pattern
            .iter()
            .map(|s| s.to_token_stream().to_string())
            .reduce(|acc, s| {
                if acc.is_empty() {
                    s
                } else {
                    format!("{acc} | {s}")
                }
            })
            .unwrap()
    }

    pub fn validate_and_collect(
        &self,
        variant: &Variant,
        types: &mut PolymorphicMap,
        error: &mut Diagnostic,
    ) {
        // A pattern can contain multiple shapes, e.g. `(_) | (_, _) | { name: _, age: usize }`
        // So if the variant_item matches a shape, we associate the pattern with the variant.
        let Some((group, ifields)) = self.pattern_matching_on(variant) else {
            return error.extend(
                variant.fields.span(),
                format!(
                    "`{}` doesn't match pattern `{}`",
                    variant.to_token_stream(),
                    self.print_pattern()
                ),
            );
        };

        // TODO: No support for empty unit iter, yet...
        if group.is_unit() {
            return;
        }

        let mut variadic_or_range = false;
        for (p, item) in group.into_iter().zip(ifields.into_iter()) {
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
            let is_generic = pty.eq(Self::PLACEHOLDER_SYMBOL) || pty.to_uppercase().eq(&pty);

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

impl Parameter {
    pub fn is_field(&self) -> bool {
        matches!(self, Parameter::Regular(_))
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self, Parameter::Variadic(_))
    }

    pub fn is_range(&self) -> bool {
        matches!(self, Parameter::Range(_))
    }

    fn get_field(&self) -> Option<&Field> {
        match self {
            Parameter::Regular(field) => Some(field),
            _ => None,
        }
    }
}

impl Group {
    pub fn len(&self) -> usize {
        match self {
            Group::Named { parameters, .. } => parameters.len(),
            Group::Unnamed { parameters, .. } => parameters.len(),
            Group::Unit => 0,
        }
    }

    pub fn iter(&self) -> Iter<'_, Parameter> {
        thread_local! {static EMPTY_SLICE_ITER: Punctuated<Parameter, ()> = Punctuated::new();}

        match self {
            // UNSAFE: Don't do this sh*t. The thing is that we are transmuting an empty iter that is created from a static Punctuated struct.
            //         The lifetime is invariant in Iter<'_> which mean that we are not allowed to return another lifetime, even if it outlives 'a.
            //         It should be "okay" given its static and empty, but I'm not 100% sure if this actually can cause UB.
            Group::Unit => EMPTY_SLICE_ITER.with(|f| unsafe { std::mem::transmute(f.iter()) }),
            // Group::Unit => panic!("Empty Iter is unsupported right now."),
            Group::Named { parameters, .. } => parameters.iter(),
            Group::Unnamed { parameters, .. } => parameters.iter(),
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Group::Unit)
    }

    pub fn has_variadic(&self) -> bool {
        match self {
            // TODO: Should probably just check last field, or should it have support for (T, .., U)?
            Group::Named { parameters, .. } => parameters.iter().any(|fk| fk.is_variadic()),
            Group::Unnamed { parameters, .. } => parameters.iter().any(|fk| fk.is_variadic()),
            Group::Unit => false,
        }
    }

    pub fn has_last_variadic(&self) -> bool {
        match self {
            Group::Named { parameters, .. } => {
                matches!(parameters.iter().last().take(), Some(val) if val.is_variadic())
            }
            Group::Unnamed { parameters, .. } => {
                matches!(parameters.iter().last().take(), Some(val) if val.is_variadic())
            }
            Group::Unit => false,
        }
    }

    pub fn count_with(&self, mut f: impl FnMut(&Parameter) -> bool) -> usize {
        match self {
            Group::Named { parameters, .. } => {
                parameters
                    .iter()
                    .fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc })
            }
            Group::Unnamed { parameters, .. } => {
                parameters
                    .iter()
                    .fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc })
            }
            Group::Unit => 0,
        }
    }
}

impl From<&Fields> for Group {
    fn from(value: &Fields) -> Self {
        match value {
            Fields::Named(FieldsNamed { named, brace_token }) => Group::Named {
                parameters: parse_quote!(#named),
                delimiter: *brace_token,
            },
            Fields::Unnamed(FieldsUnnamed {
                unnamed,
                paren_token,
            }) => Group::Unnamed {
                parameters: parse_quote!(#unnamed),
                delimiter: *paren_token,
            },
            Fields::Unit => Group::Unit,
        }
    }
}

impl IntoIterator for Group {
    type Item = Parameter;
    type IntoIter = IntoIter<Parameter>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Group::Unit => Punctuated::<Parameter, ()>::new().into_iter(),
            Group::Named { parameters, .. } => parameters.into_iter(),
            Group::Unnamed { parameters, .. } => parameters.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Group {
    type Item = &'a Parameter;
    type IntoIter = Iter<'a, Parameter>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
