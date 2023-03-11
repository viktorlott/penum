use syn::{
    parse_quote,
    punctuated::{IntoIter, Iter, Punctuated},
    token::{self},
    ExprRange, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Token,
};

use quote::ToTokens;

use super::{ComparableItem, PunctuatedParameters, WhereClause};

mod parse;
mod to_tokens;

/// #### A Penum expression consists of one or more patterns, and an optional WhereClause.
///
/// ```text
/// (T) | { name: T }   where T: Clone
/// ^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^
/// <Pattern>           <clause>
/// ```
pub struct PenumExpr {
    /// Used for matching against incoming variants
    pub pattern: Vec<PatFrag>,

    /// Contains an optional where clause with one or more where predicates.
    pub clause: Option<WhereClause>,
}

/// #### Pattern fragments are used as constituents for the Penum expression composite type.
///
/// A group can only contain one group type.
/// ```text
///  Variant    () | (T, T) | { name: T }
///  ^^^^^^^    ^^   ^^^^^^   ^^^^^^^^^^^
///  <Ident>    <Composite>
/// ```
pub struct PatFrag {
    /// An optional identifier that is currently only used to mark nullary variants.
    pub ident: Option<Ident>,

    /// A group is a composite of zero or more ParameterKinds surrounded by a delimiter
    pub group: Composite,
}

/// #### A composite can come in 3 flavors:
///
/// ```text
/// { ParameterKind,* } | (ParameterKind,*) | ()
/// ^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^   ^^
/// <Named>               <Unnamed>           <Unit>
/// ```
pub enum Composite {
    /// Represents a `struct`-like pattern
    Named {
        parameters: PunctuatedParameters,
        delimiter: token::Brace,
    },

    /// Represents a `tuple`-like pattern
    Unnamed {
        parameters: PunctuatedParameters,
        delimiter: token::Paren,
    },

    /// Represents a `Unit`-like pattern
    Unit,
}

/// #### A parameter comes in different flavors:
///
/// ```text
/// Ident: Type   |   Type     |  ..
/// ^^^^^^^^^^^       ^^^^        ^^
/// <Field>           <Field>     <Variadic>
/// ```
///
/// Given that the `Regular(Field)` can also either be `named` or `unnamed`, it's possible to use a
/// `ParameterKind::Regular->Named` field inside a `GroupKind::Unnamed-Parameters` composite type.
pub enum ParameterKind {
    /// We use this to represent a `normal` field, that is, a field that is either `named` or `unnamed`.
    ///
    /// This is done by having the `ident` and `colon_token` fields be optional.
    Regular(Field),

    /// We use this to represent that we don't care amount the left over arguments.
    ///
    /// The use for variadic fields are currently only supported in the last argument position.
    Variadic(Token![..]),

    /// Use `Variadic(Token![..])` instead.
    ///
    /// Supported `>` Not supported
    /// ```text
    /// (T, ..) > (T, ..10) (T, ...) (T, ..Copy) (T, Copy..2)
    ///     ^^        ^^^^      ^^^      ^^^^^^      ^^^^^^^
    ///
    /// Variadic(Token![..]) > Range(ExprRange)
    /// ```
    Range(ExprRange),
}

impl PenumExpr {
    pub fn pattern_to_string(&self) -> String {
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

    pub fn get_comparable_patterns(&self) -> Vec<ComparableItem<Composite>> {
        self.pattern
            .iter()
            .map(|pattern| ComparableItem::from(&pattern.group))
            .collect()
    }
}

impl ParameterKind {
    /// This is useful when we just want to check if we should care about
    /// checking the inner structure of ParameterKind.
    pub fn is_field(&self) -> bool {
        matches!(self, ParameterKind::Regular(_))
    }

    /// Used in ComparablePair method calls to check if a parameter is variadic
    pub fn is_variadic(&self) -> bool {
        matches!(self, ParameterKind::Variadic(_))
    }

    /// We currently don't use this one
    pub fn is_range(&self) -> bool {
        matches!(self, ParameterKind::Range(_))
    }

    /// This is basically the same as `is_field` but instead of returning a boolean,
    /// we return an Option<&Field> if self is a field.
    pub fn get_field(&self) -> Option<&Field> {
        match self {
            ParameterKind::Regular(field) => Some(field),
            _ => None,
        }
    }
}

impl Composite {
    pub fn len(&self) -> usize {
        match self {
            Composite::Named { parameters, .. } => parameters.len(),
            Composite::Unnamed { parameters, .. } => parameters.len(),
            Composite::Unit => 0,
        }
    }

    pub fn iter(&self) -> Iter<'_, ParameterKind> {
        thread_local! {static EMPTY_SLICE_ITER: Punctuated<ParameterKind, ()> = Punctuated::new();}

        match self {
            // UNSAFE: Don't do this sh*t. The thing is that we are transmuting an empty iter that
            //         is created from a static Punctuated struct. The lifetime is invariant in Iter<'_>
            //         which mean that we are not allowed to return another lifetime, even if it outlives 'a.
            //         It should be "okay" given its static and empty, but I'm not 100% sure if this actually
            //         can cause UB.
            Composite::Unit => EMPTY_SLICE_ITER.with(|f| unsafe { std::mem::transmute(f.iter()) }),
            // Group::Unit => panic!("Empty Iter is unsupported right now."),
            Composite::Named { parameters, .. } => parameters.iter(),
            Composite::Unnamed { parameters, .. } => parameters.iter(),
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Composite::Unit)
    }

    pub fn has_variadic(&self) -> bool {
        match self {
            Composite::Named { parameters, .. } => parameters.iter().any(|fk| fk.is_variadic()),
            Composite::Unnamed { parameters, .. } => parameters.iter().any(|fk| fk.is_variadic()),
            Composite::Unit => false,
        }
    }

    pub fn get_variadic_position(&self) -> Option<usize> {
        match self {
            Composite::Named { parameters, .. } => parameters
                .iter()
                .enumerate()
                .find_map(|(pos, fk)| fk.is_variadic().then_some(pos)),
            Composite::Unnamed { parameters, .. } => parameters
                .iter()
                .enumerate()
                .find_map(|(pos, fk)| fk.is_variadic().then_some(pos)),
            Composite::Unit => None,
        }
    }

    pub fn has_last_variadic(&self) -> bool {
        match self {
            Composite::Named { parameters, .. } => {
                matches!(parameters.iter().last().take(), Some(val) if val.is_variadic())
            }
            Composite::Unnamed { parameters, .. } => {
                matches!(parameters.iter().last().take(), Some(val) if val.is_variadic())
            }
            Composite::Unit => false,
        }
    }

    pub fn count_with(&self, mut f: impl FnMut(&ParameterKind) -> bool) -> usize {
        match self {
            Composite::Named { parameters, .. } => {
                parameters
                    .iter()
                    .fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc })
            }
            Composite::Unnamed { parameters, .. } => {
                parameters
                    .iter()
                    .fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc })
            }
            Composite::Unit => 0,
        }
    }
}
mod boilerplate {
    use super::*;

    impl From<&Fields> for Composite {
        fn from(value: &Fields) -> Self {
            match value {
                Fields::Named(FieldsNamed { named, brace_token }) => Composite::Named {
                    parameters: parse_quote!(#named),
                    delimiter: *brace_token,
                },
                Fields::Unnamed(FieldsUnnamed {
                    unnamed,
                    paren_token,
                }) => Composite::Unnamed {
                    parameters: parse_quote!(#unnamed),
                    delimiter: *paren_token,
                },
                Fields::Unit => Composite::Unit,
            }
        }
    }

    impl IntoIterator for Composite {
        type Item = ParameterKind;
        type IntoIter = IntoIter<ParameterKind>;

        fn into_iter(self) -> Self::IntoIter {
            match self {
                Composite::Unit => Punctuated::<ParameterKind, ()>::new().into_iter(),
                Composite::Named { parameters, .. } => parameters.into_iter(),
                Composite::Unnamed { parameters, .. } => parameters.into_iter(),
            }
        }
    }

    impl<'a> IntoIterator for &'a Composite {
        type Item = &'a ParameterKind;
        type IntoIter = Iter<'a, ParameterKind>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }
}
