use proc_macro2::TokenStream;
use quote::ToTokens;

use super::{Composite, ParameterKind, PatFrag};

impl ToTokens for PatFrag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.group.to_tokens(tokens);
    }
}

impl ToTokens for ParameterKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ParameterKind::Regular(f) => f.to_tokens(tokens),
            ParameterKind::Variadic(v) => v.to_tokens(tokens),
            ParameterKind::Range(r) => r.to_tokens(tokens),
            ParameterKind::Nothing => (),
        }
    }
}

impl ToTokens for Composite {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Composite::Named {
                parameters,
                delimiter,
            } => delimiter.surround(tokens, |tokens| parameters.to_tokens(tokens)),
            Composite::Unnamed {
                parameters,
                delimiter,
            } => delimiter.surround(tokens, |tokens| parameters.to_tokens(tokens)),
            Composite::Unit => (),
        }
    }
}
