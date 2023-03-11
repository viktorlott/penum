use syn::{
    parse_quote,
    punctuated::{IntoIter, Iter, Punctuated},
    Fields, FieldsNamed, FieldsUnnamed,
};

use crate::factory::{Composite, ParameterKind};

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
