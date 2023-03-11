#![allow(dead_code)]
use syn::{punctuated::Punctuated, Fields, Token};

mod clause;
mod pattern;
mod subject;

pub use clause::*;
pub use pattern::*;
pub use subject::*;

// TODO: Replace `Punctuated` with custom sequence type
pub type PunctuatedParameters = Punctuated<ParameterKind, Token![,]>;

pub struct MatchPair<'a>(&'a Composite, &'a Fields);
pub struct ComparablePair<'disc>(
    /// Matched penum pattern
    ComparableItem<'disc, Composite>,
    /// Matched variant item
    ComparableItem<'disc, Fields>,
);

/// We use this to represent a Item that can be compared with another Item.
///
pub struct ComparableItem<'disc, T> {
    /// To identify the discriminant of the composite type
    discriminant: &'disc T,

    /// Some(usize) implies it has variadic at position `usize`.
    variadic: Option<usize>,

    /// The number of arguments in the group.
    arity: usize,
}

/// We use this to identify what kind of pair we have matched.
///
/// NOTE: Could probably have used discriminants instead..
enum MatchResult {
    /// Mathed either a `Named` or an `Unnamed` pair
    Compound,

    /// Matched a unit pair
    Nullary,

    // Nothing match
    None,
}

impl<'disc> ComparablePair<'disc> {
    /// Used to ensure that a matched pair have the same arity.
    ///
    /// If they do not we deduce that the item doesn't match our pattern.
    fn check_arity_equality(&self) -> bool {
        matches!(self, ComparablePair(p, i) if p.arity == i.arity)
    }

    /// Use to check if our pattern contains a variadic field.
    ///
    /// NOTE: It can be anywhere in the pattern.
    fn contains_residual(&self) -> bool {
        matches!(self, ComparablePair(p, _) if p.variadic.is_some())
    }

    /// Use to check if we have a variadic field at the last argument position.
    /// It will default to false if its not found.
    ///
    /// NOTE: This will return false even if our pattern has a variadic field.
    /// e.g `(.., T) => false` | `(T, ..) => true`
    fn has_variadic_last(&self) -> bool {
        matches!(self, ComparablePair(p, _) if p.variadic.map(|pos| pos == p.arity - 1).unwrap_or_default())
    }

    /// Use this only when you know that our pattern contains a variadic field.
    ///  
    /// Check if the item satisfies the minimum parameter length required.
    fn check_minimum_arity_satisfaction(&self) -> bool {
        // TODO: Change this if we every choose to accept variadic at positions other than last. e.g (T, .., T) | (.., T)
        matches!(self, ComparablePair(p, i) if p.variadic.map(|_| p.arity - 1).unwrap_or_else(|| p.arity) <= i.arity )
    }
}

impl<'a> MatchPair<'a> {
    fn pattern_match(&self) -> MatchResult {
        match self {
            MatchPair(&Composite::Named { .. }, &Fields::Named(..))
            | MatchPair(&Composite::Unnamed { .. }, &Fields::Unnamed(..)) => MatchResult::Compound,
            MatchPair(Composite::Unit, Fields::Unit) => MatchResult::Nullary,
            _ => MatchResult::None,
        }
    }
}

fn pattern_match<'a>(fields: &'a Fields) -> impl FnMut(&'a PatFrag) -> Option<ComparablePair<'a>> {
    // let cmp_item_fields = ComparableItem::from(fields);

    move |shape: &PatFrag| {
        let match_pair = MatchPair::from((&shape.group, fields));

        match match_pair.pattern_match() {
            MatchResult::Compound => {
                let cmp_pair = ComparablePair::from(match_pair);

                if cmp_pair.has_variadic_last() {
                    cmp_pair
                        .check_minimum_arity_satisfaction()
                        .then_some(cmp_pair)
                } else {
                    cmp_pair.check_arity_equality().then_some(cmp_pair)
                }
            }
            MatchResult::Nullary => Some(match_pair.into()),
            _ => None,
        }
    }
}

mod boilerplate {
    use super::*;
    impl<'disc> From<ComparablePair<'disc>> for (&'disc Composite, &'disc Fields) {
        fn from(val: ComparablePair<'disc>) -> Self {
            (val.0.discriminant, val.1.discriminant)
        }
    }

    impl<'a> From<(&'a Composite, &'a Fields)> for MatchPair<'a> {
        fn from(value: (&'a Composite, &'a Fields)) -> Self {
            Self(value.0, value.1)
        }
    }

    impl<'disc> From<&'disc Composite> for ComparableItem<'disc, Composite> {
        fn from(value: &'disc Composite) -> Self {
            Self {
                discriminant: value,
                variadic: value.get_variadic_position(),
                arity: value.len(),
            }
        }
    }

    impl<'disc> From<&'disc Fields> for ComparableItem<'disc, Fields> {
        fn from(value: &'disc Fields) -> Self {
            Self {
                discriminant: value,
                variadic: None,
                arity: value.len(),
            }
        }
    }

    impl<'disc> From<MatchPair<'disc>> for ComparablePair<'disc> {
        fn from(value: MatchPair<'disc>) -> Self {
            Self(value.0.into(), value.1.into())
        }
    }
}
