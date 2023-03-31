use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::ToTokens;

use super::{PatComposite, PatFieldKind, PatFrag};

impl ToTokens for PatFrag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.group.to_tokens(tokens);
    }
}

impl ToTokens for PatFieldKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PatFieldKind::Field(f) => f.to_tokens(tokens),
            PatFieldKind::Variadic(v) => v.to_tokens(tokens),
            PatFieldKind::Range(r) => r.to_tokens(tokens),
            PatFieldKind::Infer => tokens.extend(TokenStream::from_str("_")),
            PatFieldKind::Nothing => (),
        }
    }
}

impl ToTokens for PatComposite {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PatComposite::Named {
                parameters,
                delimiter,
            } => delimiter.surround(tokens, |tokens| parameters.to_tokens(tokens)),
            PatComposite::Unnamed {
                parameters,
                delimiter,
            } => delimiter.surround(tokens, |tokens| parameters.to_tokens(tokens)),
            _ => (),
        }
    }
}
