use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Error;

#[derive(Default)]
pub struct Diagnostic(Option<Error>);

impl Diagnostic {
    pub fn extend(&mut self, span: Span, error: impl Display) {
        if let Some(err) = self.0.as_mut() {
            err.combine(Error::new(span, error));
        } else {
            self.0 = Some(Error::new(span, error));
        }
    }

    pub fn extend_spanned(&mut self, token: impl ToTokens, error: impl Display) {
        if let Some(err) = self.0.as_mut() {
            err.combine(Error::new_spanned(token, error));
        } else {
            self.0 = Some(Error::new_spanned(token, error));
        }
    }

    pub fn map<F>(&self, f: F) -> Option<TokenStream>
    where
        F: FnOnce(&Error) -> TokenStream,
    {
        self.0.as_ref().map(f)
    }

    pub fn has_error(&self) -> bool {
        self.0.is_some()
    }
}
