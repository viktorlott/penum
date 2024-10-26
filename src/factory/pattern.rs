use syn::{
    punctuated::{Iter, Punctuated},
    spanned::Spanned,
    token, ExprRange, Field, Ident, Token,
};

use quote::ToTokens;

use crate::{
    dispatch::{Blueprint, BlueprintsMap},
    error::Diagnostic,
    utils::UniqueHashId,
};

use super::{ComparablePats, PredicateType, WhereClause, WherePredicate};

mod boilerplate;
mod parse;
mod to_tokens;

// TODO: Replace `Punctuated` with custom sequence type
pub type PunctuatedParameters = Punctuated<PatFieldKind, Token![,]>;

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

    /// A group is a composite of zero or more PatComposite surrounded
    /// by a delimiter
    pub group: PatComposite,
}

/// A composite can come in 3 flavors:
///
/// ```text
/// { PatComposite,* } | (PatComposite,*) | ()
/// ^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^   ^^
/// <Named>               <Unnamed>           <Unit>
/// ```
#[derive(Debug)]
pub enum PatComposite {
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

    /// Represents a `Inferred` pattern
    Inferred,
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
/// `unnamed`, it's possible to use a `PatParamKind::Regular->Named`
/// field inside a `GroupKind::Unnamed-Parameters` composite type.
#[derive(Debug)]
pub enum PatFieldKind {
    /// Used to indicate that this field will be inferred
    Infer,

    /// We use this to represent a `normal` field, that is, a field that
    /// is either `named` or `unnamed`.
    ///
    /// This is done by having the `ident` and `colon_token` fields be
    /// optional.
    Field(Field),

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

    pub fn get_comparable_patterns(&self) -> ComparablePats {
        self.into()
    }

    pub fn has_predicates(&self) -> bool {
        matches!(&self.clause, Some(wc) if !wc.predicates.is_empty())
    }

    pub fn has_clause(&self) -> bool {
        self.clause.is_some()
    }

    /// This should probably be refactored...
    ///
    /// NOTE: This totally works when we are using Generics with patterns. But if we use
    /// concrete types with trait bounds it breaks.
    ///
    /// We somehow need to know that if a penum expression contains two or more concrete types with
    /// the same dispatched trait bound we should interprete them as the same. If we do not do this
    /// we create x implementations for the same trait.
    ///
    /// I feel like the blueprints map key should be based on trait bound instead of type.
    ///
    /// SOLUTION: We could keep this as it is, and instead fold our blueprints map so that types with the
    /// same trait bounds are combined.
    pub fn get_blueprints_map(&self, error: &Diagnostic) -> Option<BlueprintsMap> {
        let Some(clause) = self.clause.as_ref() else {
            return None;
        };

        let mut polymap = BlueprintsMap::default();

        for pred in clause.predicates.iter() {
            if let WherePredicate::Type(pred_ty) = pred {
                let mut blueprints = Vec::<Blueprint>::default();

                for param_bound in pred_ty.bounds.iter() {
                    // Only get trait bound with `^` caret. e.g Type: ^Trait
                    if let Some(trait_bound) = param_bound.get_dispatchable_trait_bound() {
                        // This will try to first check if the trait exists in our
                        // std trait store, and if it's not found, we'll check our
                        // SHM map.
                        match Blueprint::try_from(trait_bound) {
                            Ok(blueprint) => blueprints.push(blueprint),
                            Err(err) => error.extend(trait_bound.span(), err),
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

impl PatFieldKind {
    /// This is useful when we just want to check if we should care
    /// about checking the inner structure of PatParamKind.
    pub fn is_field(&self) -> bool {
        matches!(self, PatFieldKind::Field(_))
    }

    /// Used in ComparablePair method calls to check if a parameter is
    /// variadic
    pub fn is_variadic(&self) -> bool {
        matches!(self, PatFieldKind::Variadic(_))
    }

    /// We currently don't use this one
    pub fn is_range(&self) -> bool {
        matches!(self, PatFieldKind::Range(_))
    }

    /// Used to quickly check if PatFieldKind is `Infer`
    pub fn is_infer(&self) -> bool {
        matches!(self, PatFieldKind::Infer)
    }

    /// This is basically the same as `is_field` but instead of
    /// returning a boolean, we return an Option<&Field> if self is a
    /// field.
    pub fn get_field(&self) -> Option<&Field> {
        match self {
            PatFieldKind::Field(field) => Some(field),
            _ => None,
        }
    }
}

impl PatComposite {
    pub fn len(&self) -> usize {
        match self {
            PatComposite::Named { parameters, .. } => parameters.len(),
            PatComposite::Unnamed { parameters, .. } => parameters.len(),
            _ => 0,
        }
    }

    pub fn iter(&self) -> Iter<'_, PatFieldKind> {
        thread_local! {static EMPTY_SLICE_ITER: Punctuated<PatFieldKind, ()> = Punctuated::new();}

        match self {
            PatComposite::Named { parameters, .. } => parameters.iter(),
            PatComposite::Unnamed { parameters, .. } => parameters.iter(),
            _ => EMPTY_SLICE_ITER.with(|f| {
                // "SAFETY": This is not recommended. The thing is that we
                //           are transmuting an empty iter that is created
                //           from a static Punctuated struct. The lifetime
                //           is invariant in Iter<'_> which means that we
                //           are not allowed to return another lifetime,
                //           even if it outlives 'a. But, in this case it
                //           should be "okay" given that it's static and empty.
                //           Though, I'm not 100% sure if this actually can
                //           cause UB because of the Punctuated Adding to this, it's not currently
                //           possible to get here right now because of the
                //           if statement in
                //           `penum::assemble->is_unit()->continue`. So this
                //           could also be marked as `unreachable`.
                unsafe { std::mem::transmute(f.iter()) }
            }),
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, PatComposite::Unit)
    }

    pub fn has_variadic(&self) -> bool {
        match self {
            PatComposite::Named { parameters, .. } => parameters.iter().any(|fk| fk.is_variadic()),
            PatComposite::Unnamed { parameters, .. } => {
                parameters.iter().any(|fk| fk.is_variadic())
            }
            _ => false,
        }
    }

    pub fn get_variadic_position(&self) -> Option<usize> {
        match self {
            PatComposite::Named { parameters, .. } => parameters
                .iter()
                .enumerate()
                .find_map(|(pos, fk)| fk.is_variadic().then_some(pos)),
            PatComposite::Unnamed { parameters, .. } => parameters
                .iter()
                .enumerate()
                .find_map(|(pos, fk)| fk.is_variadic().then_some(pos)),
            _ => None,
        }
    }

    pub fn has_last_variadic(&self) -> bool {
        match self {
            PatComposite::Named { parameters, .. } => {
                matches!(parameters.iter().last().take(), Some(val) if val.is_variadic())
            }
            PatComposite::Unnamed { parameters, .. } => {
                matches!(parameters.iter().last().take(), Some(val) if val.is_variadic())
            }
            _ => false,
        }
    }

    pub fn count_with(&self, mut f: impl FnMut(&PatFieldKind) -> bool) -> usize {
        match self {
            PatComposite::Named { parameters, .. } => {
                parameters
                    .iter()
                    .fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc })
            }
            PatComposite::Unnamed { parameters, .. } => {
                parameters
                    .iter()
                    .fold(0, |acc, fk| if f(fk) { acc + 1 } else { acc })
            }
            _ => 0,
        }
    }
}
