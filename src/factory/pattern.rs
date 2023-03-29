use syn::{
    punctuated::{Iter, Punctuated},
    spanned::Spanned,
    token, ExprRange, Field, Ident, Token,
};

use quote::ToTokens;

use crate::{
    dispatch::{Blueprint, Blueprints},
    error::Diagnostic,
    utils::UniqueHashId,
};

use super::{Comparable, PredicateType, WhereClause, WherePredicate};

mod boilerplate;
mod parse;
mod to_tokens;

// TODO: Replace `Punctuated` with custom sequence type
pub type PunctuatedParameters = Punctuated<ParameterKind, Token![,]>;

/// A Penum expression consists of one or more patterns, and an optional WhereClause.
///
/// ```text
/// (T) | { name: T }   where T: Clone
/// ^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^
/// <Pattern>           <clause>
/// ```
#[derive(Default, Debug)]
pub struct PenumExpr {
    /// Used for matching against incoming variants
    pub pattern: Vec<PatFrag>,

    /// Contains an optional where clause with one or more where
    /// predicates.
    pub clause: Option<WhereClause>,
}

/// Pattern fragments are used as constituents for the Penum expression composite type.
///
/// A group can only contain one group type.
/// ```text
///  Variant    () | (T, T) | { name: T }
///  ^^^^^^^    ^^   ^^^^^^   ^^^^^^^^^^^
///  <Ident>    <Composite>
/// ```
#[derive(Debug)]
pub struct PatFrag {
    /// An optional identifier that is currently only used to mark
    /// nullary variants.
    pub ident: Option<Ident>,

    /// A group is a composite of zero or more ParameterKinds surrounded
    /// by a delimiter
    pub group: Composite,
}

/// A composite can come in 3 flavors:
///
/// ```text
/// { ParameterKind,* } | (ParameterKind,*) | ()
/// ^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^   ^^
/// <Named>               <Unnamed>           <Unit>
/// ```
#[derive(Debug)]
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

/// A parameter comes in different flavors:
///
/// ```text
/// Ident: Type   |   Type     |  ..
/// ^^^^^^^^^^^       ^^^^        ^^
/// <Field>           <Field>     <Variadic>
/// ```
///
/// Given that the `Regular(Field)` can also either be `named` or
/// `unnamed`, it's possible to use a `ParameterKind::Regular->Named`
/// field inside a `GroupKind::Unnamed-Parameters` composite type.
#[derive(Debug)]
pub enum ParameterKind {
    /// We use this to represent a `normal` field, that is, a field that
    /// is either `named` or `unnamed`.
    ///
    /// This is done by having the `ident` and `colon_token` fields be
    /// optional.
    Regular(Field),

    /// We use this to represent that we don't care amount the left over
    /// arguments.
    ///
    /// The use for variadic fields are currently only supported in the
    /// last argument position.
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

    /// Suppose to be used for derived Default
    Nothing,
}

impl PenumExpr {
    pub fn pattern_to_string(&self) -> String {
        self.pattern
            .iter()
            .map(|s| s.to_token_stream().to_string())
            .reduce(|acc, s| {
                acc.is_empty()
                    .then(|| s.clone())
                    .unwrap_or_else(|| format!("{acc} | {s}"))
            })
            .unwrap()
    }

    pub fn get_comparable_patterns(&self) -> Vec<Comparable<Composite>> {
        self.pattern
            .iter()
            .map(|pattern| Comparable::from(&pattern.group))
            .collect()
    }

    pub fn has_predicates(&self) -> bool {
        matches!(&self.clause, Some(wc) if !wc.predicates.is_empty())
    }

    pub fn has_clause(&self) -> bool {
        self.clause.is_some()
    }

    /// This should probably be refactored...
    pub fn get_blueprints(&self, error: &mut Diagnostic) -> Option<Blueprints> {
        let Some(clause) = self.clause.as_ref() else {
            return None
        };

        let mut polymap: Blueprints = Default::default();

        for pred in clause.predicates.iter() {
            if let WherePredicate::Type(pred_ty) = pred {
                let mut blueprints: Vec<Blueprint> = Default::default();
                for tr in pred_ty.bounds.iter() {
                    if let Some(dtr) = tr.get_dispatchable_trait_bound() {
                        match Blueprint::try_from(dtr) {
                            Ok(blueprint) => blueprints.push(blueprint),
                            Err(err) => error.extend(dtr.span(), err),
                        }
                    }
                }
                if blueprints.is_empty() {
                    return None;
                }

                let ty = UniqueHashId(pred_ty.bounded_ty.clone());

                if let Some(entry) = polymap.get_mut(&ty) {
                    entry.append(&mut blueprints);
                } else {
                    polymap.insert(ty, blueprints);
                }
            }
        }
        (!polymap.is_empty()).then_some(polymap)
    }

    pub fn find_predicate(
        &self,
        f: impl Fn(&PredicateType) -> Option<&PredicateType>,
    ) -> Option<&PredicateType> {
        if self.has_predicates() {
            // SAFETY: We can only have predicates if we have a where
            // clause.
            unsafe { self.clause.as_ref().unwrap_unchecked() }
                .predicates
                .iter()
                .find_map(|pred| match pred {
                    WherePredicate::Type(pred_ty) => f(pred_ty),
                    _ => None,
                })
        } else {
            None
        }
    }
}

impl ParameterKind {
    /// This is useful when we just want to check if we should care
    /// about checking the inner structure of ParameterKind.
    pub fn is_field(&self) -> bool {
        matches!(self, ParameterKind::Regular(_))
    }

    /// Used in ComparablePair method calls to check if a parameter is
    /// variadic
    pub fn is_variadic(&self) -> bool {
        matches!(self, ParameterKind::Variadic(_))
    }

    /// We currently don't use this one
    pub fn is_range(&self) -> bool {
        matches!(self, ParameterKind::Range(_))
    }

    /// This is basically the same as `is_field` but instead of
    /// returning a boolean, we return an Option<&Field> if self is a
    /// field.
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
            // "SAFETY": This is not recommended. The thing is that we
            //           are transmuting an empty iter that is created
            //           from a static Punctuated struct. The lifetime
            //           is invariant in Iter<'_> which mean that we are
            //           not allowed to return another lifetime, even if
            //           it outlives 'a. It should be "okay" given its
            //           static and empty, but I'm not 100% sure if this
            //           actually can cause UB. Adding to this, it's not
            //           currently possible to get here right now
            //           because of the if statement in
            //           `penum::assemble->is_unit()->continue`. So this
            //           could also be marked as `unreachable`.
            Composite::Unit => EMPTY_SLICE_ITER.with(|f| unsafe { std::mem::transmute(f.iter()) }),
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
