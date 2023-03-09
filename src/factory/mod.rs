use syn::{punctuated::Punctuated, Fields, Token};

mod pattern;
mod penum;
mod subject;
mod clause;

pub use pattern::*;
pub use penum::*;
pub use subject::*;
pub use clause::*;

pub type PunctuatedParameters = Punctuated<Parameter, Token![,]>;

// TODO: Change this sh*t, or add some
pub trait PatternMatcher {
    fn get_matches(&self) -> (&Group, &Fields);

    fn has_same_len(&self) -> bool {
        matches!(self.get_matches(), (p, i) if p.len() == i.len())
    }

    fn has_variadic(&self) -> bool {
        matches!(self.get_matches(), (p, _) if p.has_variadic())
    }

    fn has_variadic_last(&self) -> bool {
        matches!(self.get_matches(), (p, _) if p.has_last_variadic())
    }

    fn has_minimum_matches(&self) -> bool {
        matches!(self.get_matches(), (p, i) if p.count_with(|fk| fk.is_field()) <= i.len())
    }
}

impl<'a> PatternMatcher for (&'a Group, &'a Fields) {
    fn get_matches(&self) -> (&'a Group, &'a Fields) {
        (self.0, self.1)
    }
}

fn pattern_match<'a>(
    fields: &'a Fields,
) -> impl FnMut(&'a PatternFrag) -> Option<(&'a Group, &'a Fields)> {
    move |shape: &PatternFrag| match (&shape.group, fields) {
        tail @ ((&Group::Named{..}, &Fields::Named(..)) | (&Group::Unnamed{..}, &Fields::Unnamed(..)))
        // This is kind of expensive.. But what do I care?
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
        tail @ (Group::Unit, Fields::Unit) => Some(tail),
        _ => None,
    }
}
