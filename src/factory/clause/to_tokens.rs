use proc_macro2::TokenStream;
use quote::ToTokens;

use super::*;

impl ToTokens for WhereClause {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.predicates.is_empty() {
            self.where_token.to_tokens(tokens);
            self.predicates.to_tokens(tokens);
        }
    }
}

impl ToTokens for WherePredicate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            WherePredicate::Type(t) => t.to_tokens(tokens),
            WherePredicate::Lifetime(l) => l.to_tokens(tokens),
        }
    }
}

impl ToTokens for TypeParamBound {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypeParamBound::Trait(t) => t.to_tokens(tokens),
            TypeParamBound::Lifetime(l) => l.to_tokens(tokens),
        }
    }
}

impl ToTokens for PredicateType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lifetimes.to_tokens(tokens);
        self.bounded_ty.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.bounds.to_tokens(tokens);
    }
}

impl ToTokens for PredicateLifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lifetime.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.bounds.to_tokens(tokens);
    }
}

impl ToTokens for TraitBound {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let to_tokens = |tokens: &mut TokenStream| {
            self.modifier.to_tokens(tokens);
            self.lifetimes.to_tokens(tokens);
            self.path.to_tokens(tokens);
        };
        match &self.paren_token {
            Some(paren) => paren.surround(tokens, to_tokens),
            None => to_tokens(tokens),
        }
    }
}
