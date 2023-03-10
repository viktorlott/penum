use syn::{punctuated::Punctuated, Fields, Token};

mod clause;
mod pattern;
mod subject;

pub use clause::*;
pub use pattern::*;
pub use subject::*;

// TODO: Replace `Punctuated` with custom sequence type
pub type PunctuatedParameters = Punctuated<Parameter, Token![,]>;
pub type MatchPair<'a> = (&'a Group, &'a Fields);

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

impl<'a> PatternMatcher for MatchPair<'a> {
    fn get_matches(&self) -> MatchPair {
        (self.0, self.1)
    }
}

fn pattern_match<'a>(fields: &'a Fields) -> impl FnMut(&'a PatternFrag) -> Option<MatchPair<'a>> {
    // This is kind of expensive.. clean up when possible
    move |shape: &PatternFrag| match (&shape.group, fields) {
        pair @ ((&Group::Named { .. }, &Fields::Named(..))
        | (&Group::Unnamed { .. }, &Fields::Unnamed(..))) => {
            if pair.has_variadic_last() {
                pair.has_minimum_matches().then_some(pair)
            } else {
                pair.has_same_len().then_some(pair)
            }
        }
        pair @ (Group::Unit, Fields::Unit) => Some(pair),
        _ => None,
    }
}
