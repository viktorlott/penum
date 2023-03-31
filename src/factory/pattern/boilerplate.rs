use syn::{
    parse_quote,
    punctuated::{IntoIter, Iter, Punctuated},
    Fields, FieldsNamed, FieldsUnnamed,
};

use crate::factory::{PatComposite, PatFieldKind};

impl From<&Fields> for PatComposite {
    fn from(value: &Fields) -> Self {
        match value {
            Fields::Named(FieldsNamed { named, brace_token }) => PatComposite::Named {
                parameters: parse_quote!(#named),
                delimiter: *brace_token,
            },
            Fields::Unnamed(FieldsUnnamed {
                unnamed,
                paren_token,
            }) => PatComposite::Unnamed {
                parameters: parse_quote!(#unnamed),
                delimiter: *paren_token,
            },
            Fields::Unit => PatComposite::Unit,
        }
    }
}

impl IntoIterator for PatComposite {
    type Item = PatFieldKind;
    type IntoIter = IntoIter<PatFieldKind>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PatComposite::Named { parameters, .. } => parameters.into_iter(),
            PatComposite::Unnamed { parameters, .. } => parameters.into_iter(),
            _ => Punctuated::<PatFieldKind, ()>::new().into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a PatComposite {
    type Item = &'a PatFieldKind;
    type IntoIter = Iter<'a, PatFieldKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
