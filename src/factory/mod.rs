#![allow(dead_code)]
use std::iter::Zip;

use syn::{punctuated::Iter, punctuated::Punctuated, Field, Fields, Token};

mod clause;
mod pattern;
mod subject;

pub use clause::*;
pub use pattern::*;
pub use subject::*;

// TODO: Replace `Punctuated` with custom sequence type
pub type PunctuatedParameters = Punctuated<ParameterKind, Token![,]>;

pub struct ComparablePair<'disc>(
    /// Matched penum pattern
    &'disc ComparableItem<'disc, Composite>,
    /// Matched variant item
    &'disc ComparableItem<'disc, Fields>,
);

/// We use this to represent a Item that can be compared with another Item.
///
pub struct ComparableItem<'disc, T> {
    /// To identify the discriminant of the composite type
    pub value: &'disc T,

    /// Some(usize) implies it has variadic at position `usize`.
    variadic: Option<usize>,

    /// The number of arguments in the group.
    arity: usize,
}

/// We use this to identify what kind of pair we have matched.
///
/// NOTE: Could probably have used discriminants instead..
enum MatchKind {
    /// Mathed either a `Named` or an `Unnamed` pair.
    ///
    /// Compound matches implies that we have inner structure to continue comparing
    Compound,

    /// Matched a unit pair
    ///
    /// Nullary matches implies that we satisfy the pattern shape,
    /// and that we don't need to compare inner structure
    Nullary,

    /// Nothing match
    None,
}

impl<'disc> ComparablePair<'disc> {
    /// Used to get access to composite methods.
    ///
    /// e.g. `is_unit()`
    pub fn as_composite(&self) -> &Composite {
        self.0.value
    }

    /// Given that we only allow variadic at the end lets us always be able to zip these together.
    ///
    pub fn zip(&self) -> Zip<Iter<ParameterKind>, Iter<Field>> {
        if self.contains_residual() {
            // Might be better to emit this as a compile error instead.
            debug_assert!(self.has_variadic_last());
        }

        self.0.value.into_iter().zip(self.1.value.into_iter())
    }

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
        // NOTE: Change this if we every choose to accept variadic at positions other than last. e.g (T, .., T) | (.., T)
        matches!(self, ComparablePair(p, i) if p.variadic.map(|_| p.arity - 1).unwrap_or_else(|| p.arity) <= i.arity )
    }

    fn match_kind(&self) -> MatchKind {
        match (self.0.value, self.1.value) {
            (&Composite::Named { .. }, &Fields::Named(..))
            | (&Composite::Unnamed { .. }, &Fields::Unnamed(..)) => MatchKind::Compound,
            (Composite::Unit, Fields::Unit) => MatchKind::Nullary,
            _ => MatchKind::None,
        }
    }
}

/// This is a very expensive way of finding a match. We should convert both into ComparableItems before looping over them.
pub fn pattern_match<'a>(
    fields: &'a ComparableItem<Fields>,
) -> impl FnMut(&'a ComparableItem<Composite>) -> Option<ComparablePair<'a>> {
    // let cmp_item_fields = ComparableItem::from(fields);
    // TODO: Rewrite this when it's possible so that we use comparable items instead.
    move |shape: &ComparableItem<Composite>| {
        let cmp_pair = ComparablePair::from((shape, fields));

        match cmp_pair.match_kind() {
            MatchKind::Compound => {
                if cmp_pair.has_variadic_last() {
                    cmp_pair
                        .check_minimum_arity_satisfaction()
                        .then_some(cmp_pair)
                } else {
                    cmp_pair.check_arity_equality().then_some(cmp_pair)
                }
            }
            MatchKind::Nullary => Some(cmp_pair),
            _ => None,
        }
    }
}

mod boilerplate {
    use super::*;
    impl<'disc> From<ComparablePair<'disc>> for (&'disc Composite, &'disc Fields) {
        fn from(val: ComparablePair<'disc>) -> Self {
            (val.0.value, val.1.value)
        }
    }

    impl<'a>
        From<(
            &'a ComparableItem<'a, Composite>,
            &'a ComparableItem<'a, Fields>,
        )> for ComparablePair<'a>
    {
        fn from(value: (&'a ComparableItem<Composite>, &'a ComparableItem<Fields>)) -> Self {
            Self(value.0, value.1)
        }
    }

    impl<'disc> From<&'disc Composite> for ComparableItem<'disc, Composite> {
        fn from(value: &'disc Composite) -> Self {
            Self {
                value,
                variadic: value.get_variadic_position(),
                arity: value.len(),
            }
        }
    }

    impl<'disc> From<&'disc Fields> for ComparableItem<'disc, Fields> {
        fn from(value: &'disc Fields) -> Self {
            Self {
                value,
                variadic: None,
                arity: value.len(),
            }
        }
    }
}
