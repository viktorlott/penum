use syn::{Fields, Token, punctuated::Punctuated};

mod pattern;
mod penum;
mod subject;

pub use pattern::*;
pub use penum::*;
pub use subject::*;

use Scope::*;

pub type PunctuatedFieldKinds = Punctuated<FieldKind, Token![,]>;

// TODO: Change this sh*t, or add some
pub trait PatternMatcher {
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