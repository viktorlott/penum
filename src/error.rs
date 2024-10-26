use std::{cell::RefCell, fmt::Display};

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Error;

#[derive(Default)]
pub struct Diagnostic(RefCell<Option<Error>>);

impl Diagnostic {
    pub fn extend(&self, span: Span, error: impl Display) {
        let mut opt_error = self.0.borrow_mut();
        if let Some(err) = opt_error.as_mut() {
            err.combine(Error::new(span, error));
        } else {
            *opt_error = Some(Error::new(span, error));
        }
    }

    pub fn extend_spanned(&self, token: impl ToTokens, error: impl Display) {
        let mut opt_error = self.0.borrow_mut();
        if let Some(err) = opt_error.as_mut() {
            err.combine(Error::new_spanned(token, error));
        } else {
            *opt_error = Some(Error::new_spanned(token, error));
        }
    }

    pub fn map<F>(&self, f: F) -> Option<TokenStream>
    where
        F: FnOnce(&Error) -> TokenStream,
    {
        self.0.borrow().as_ref().map(f)
    }

    pub fn has_error(&self) -> bool {
        self.0.borrow().is_some()
    }
}
