#![allow(dead_code)]
use std::iter::repeat;
use std::iter::zip;

use syn::Field;
use syn::Fields;

mod clause;
mod pattern;
mod subject;

pub use clause::*;
pub use pattern::*;
pub use subject::*;

// ComPairAble would be a stupid name
pub struct ComparablePair<'disc>(
    /// Matched penum pattern
    &'disc Comparable<'disc, PatComposite>,
    /// Matched variant item
    &'disc Comparable<'disc, Fields>,
);

/// We use this to represent either a `Pattern` or an `Item` that can be compared with eachother.
///
/// If we want to compare other things in the future, we extend this struct.
/// I have marked it as non_exhaustive even though that won't do anything--because we're not
/// exposing it to other crates.
#[non_exhaustive]
pub struct Comparable<'disc, T> {
    /// To identify the discriminant of the composite type
    pub inner: &'disc T,

    /// Some(usize) implies it has variadic at position `usize`.
    variadic: Option<usize>,

    /// The number of arguments in the group.
    arity: usize,
}

/// This is just an intermediate struct to hide some logic behind.
pub struct ComparablePats<'disc>(Vec<Comparable<'disc, PatComposite>>);

/// We use this to identify what kind of pair we have matched.
///
/// NOTE: Could probably have used discriminants instead..
enum MatchKind {
    /// Infer Compound and fields
    ///
    /// This is used when we want to register all fields in a group.
    Inferred,

    /// Mathed either a `Named` or an `Unnamed` pair.
    ///
    /// Compound matches implies that we have inner structure to continue comparing
    Compound,

    /// Matched a unit pair
    ///
    /// Empty matches implies that we satisfy the pattern shape,
    /// and that we don't need to compare inner structure
    Empty,

    /// Nothing match
    None,
}

impl<'disc> ComparablePair<'disc> {
    /// Used to get access to composite methods.
    ///
    /// e.g. `is_unit()`
    pub fn as_composite(&self) -> &PatComposite {
        self.0.inner
    }

    /// Given that we only allow variadic at the end lets us always be able to zip these together.
    ///
    pub fn zip(&self) -> impl Iterator<Item = (&PatFieldKind, &Field)> {
        if self.contains_residual() {
            // Might be better to emit this as a compile error instead.
            debug_assert!(self.has_variadic_last());
        }

        // FIXME: We could probably use a different strategy than this one.
        if let PatComposite::Inferred = self.0.inner {
            zip(repeat(&PatFieldKind::Infer), self.1.inner)
                .collect::<Vec<(&PatFieldKind, &Field)>>()
                .into_iter()
        } else {
            zip(self.0.inner, self.1.inner)
                .collect::<Vec<(&PatFieldKind, &Field)>>()
                .into_iter()
        }
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
        match (self.0.inner, self.1.inner) {
            (&PatComposite::Named { .. }, &Fields::Named(..)) => MatchKind::Compound,
            (&PatComposite::Unnamed { .. }, &Fields::Unnamed(..)) => MatchKind::Compound,

            (PatComposite::Unit, Fields::Unit) => MatchKind::Empty,

            (PatComposite::Inferred, _) => MatchKind::Inferred,
            _ => MatchKind::None,
        }
    }
}

impl<'disc> ComparablePats<'disc> {
    /// Each compare creates a new Iter where we then compare incoming field with each pattern
    pub fn compare(&'disc self, comp_item: &'disc Comparable<Fields>) -> Option<ComparablePair> {
        self.iter().find_map(into_comparable_pair(comp_item))
    }
}

/// This is a very expensive way of finding a match. We should convert both into ComparableItems before looping over them.
pub fn into_comparable_pair<'a>(
    fields: &'a Comparable<Fields>,
) -> impl FnMut(&'a Comparable<PatComposite>) -> Option<ComparablePair<'a>> {
    move |shape: &Comparable<PatComposite>| {
        let cmp_pair = ComparablePair::from((shape, fields));

        match cmp_pair.match_kind() {
            MatchKind::Inferred => Some(cmp_pair),
            MatchKind::Compound => {
                if cmp_pair.has_variadic_last() {
                    cmp_pair
                        .check_minimum_arity_satisfaction()
                        .then_some(cmp_pair)
                } else {
                    cmp_pair.check_arity_equality().then_some(cmp_pair)
                }
            }
            MatchKind::Empty => Some(cmp_pair),
            _ => None,
        }
    }
}

mod boilerplate {
    use std::ops::Deref;

    use super::*;

    impl<'disc> Deref for ComparablePats<'disc> {
        type Target = Vec<Comparable<'disc, PatComposite>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'disc> From<ComparablePair<'disc>> for (&'disc PatComposite, &'disc Fields) {
        fn from(val: ComparablePair<'disc>) -> Self {
            (val.0.inner, val.1.inner)
        }
    }

    impl<'disc> From<&'disc PenumExpr> for ComparablePats<'disc> {
        fn from(value: &'disc PenumExpr) -> Self {
            Self(
                value
                    .pattern
                    .iter()
                    .map(|pattern| Comparable::from(&pattern.group))
                    .collect(),
            )
        }
    }

    impl<'a> From<(&'a Comparable<'a, PatComposite>, &'a Comparable<'a, Fields>)>
        for ComparablePair<'a>
    {
        fn from(value: (&'a Comparable<PatComposite>, &'a Comparable<Fields>)) -> Self {
            Self(value.0, value.1)
        }
    }

    impl<'disc> From<&'disc PatComposite> for Comparable<'disc, PatComposite> {
        fn from(value: &'disc PatComposite) -> Self {
            Self {
                inner: value,
                variadic: value.get_variadic_position(),
                arity: value.len(),
            }
        }
    }

    impl<'disc> Comparable<'disc, PatComposite> {
        pub fn new(value: &'disc PatComposite) -> Self {
            Self {
                inner: value,
                variadic: value.get_variadic_position(),
                arity: value.len(),
            }
        }
    }

    impl<'disc> From<&'disc Fields> for Comparable<'disc, Fields> {
        fn from(value: &'disc Fields) -> Self {
            Self {
                inner: value,
                variadic: None,
                arity: value.len(),
            }
        }
    }
}
