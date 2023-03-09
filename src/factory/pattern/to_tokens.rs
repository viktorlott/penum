use proc_macro2::TokenStream;
use quote::ToTokens;

use super::{Group, Parameter, PatternFrag};

impl ToTokens for PatternFrag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.group.to_tokens(tokens);
    }
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Parameter::Regular(f) => f.to_tokens(tokens),
            Parameter::Variadic(v) => v.to_tokens(tokens),
            Parameter::Range(r) => r.to_tokens(tokens),
        }
    }
}

impl ToTokens for Group {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Group::Named {
                parameters,
                delimiter,
            } => delimiter.surround(tokens, |tokens| parameters.to_tokens(tokens)),
            Group::Unnamed {
                parameters,
                delimiter,
            } => delimiter.surround(tokens, |tokens| parameters.to_tokens(tokens)),
            Group::Unit => (),
        }
    }
}
