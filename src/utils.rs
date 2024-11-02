#![allow(unused)]
use std::{
    cell::UnsafeCell,
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Once,
};

use proc_macro2::{Ident, Span};
use quote::{format_ident, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Add},
    Expr, Fields, Token, TraitBound, Type, TypeImplTrait, TypeParamBound, Variant, WhereClause,
};

use crate::{
    error::Diagnostic,
    factory::{PatComposite, PatFrag, Subject},
    polym::UniqueHashId,
};

pub const DEFAULT_VARIANT_SYMBOL: &str = "default";
pub const ABSTRACT_MACRO_EXPR_SYMBOL: &str = "implement";

pub fn no_match_found(item: &impl ToTokens, pat: &str) -> String {
    format!(
        "`{}` doesn't match pattern `{}`",
        item.to_token_stream(),
        pat
    )
}

pub fn maybe_bounds_not_permitted(trait_bound: &TraitBound) -> String {
    format!(
        "`?{}` bounds are only permitted at the point where a type parameter is declared",
        trait_bound.path.get_string()
    )
}

pub fn lifetime_not_permitted() -> &'static str {
    "Lifetime annotation not permitted"
}

pub fn create_unique_ident(value: &str, tag: &Ident, span: Span) -> Ident {
    format_ident!("_{}_{}", tag, value, span = span)
}

// NOTE: I will eventually clean this mess up
pub trait Stringify: ToTokens {
    fn get_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}

impl<T> Stringify for T where T: ToTokens {}

pub trait TypeUtils {
    fn is_generic(&self) -> bool;
    fn is_placeholder(&self) -> bool;
    #[allow(dead_code)]
    fn some_generic(&self) -> Option<String>;
    #[allow(dead_code)]
    fn get_generic_ident(&self) -> Ident;
    fn get_unique_id(&self) -> UniqueHashId<Type>;
    fn get_type_impl_trait(&self) -> Option<&TypeImplTrait>;
}

impl TypeUtils for Type {
    fn get_type_impl_trait(&self) -> Option<&TypeImplTrait> {
        if let Type::ImplTrait(ref ty_impl_trait) = self {
            Some(ty_impl_trait)
        } else {
            None
        }
    }

    fn is_generic(&self) -> bool {
        let pat_ty_string = self.to_token_stream().to_string();
        !self.is_placeholder() && pat_ty_string.to_uppercase().eq(&pat_ty_string)
    }

    fn is_placeholder(&self) -> bool {
        matches!(self, Type::Infer(_))
    }

    fn some_generic(&self) -> Option<String> {
        self.is_placeholder()
            .then(|| {
                let pat_ty = self.get_string();
                pat_ty.to_uppercase().eq(&pat_ty).then_some(pat_ty)
            })
            .flatten()
    }

    /// Only use this when you are sure it's a generic type.
    fn get_generic_ident(&self) -> Ident {
        format_ident!("{}", self.get_string(), span = self.span())
    }

    fn get_unique_id(&self) -> UniqueHashId<Type> {
        UniqueHashId::new(self)
    }
}

pub trait TraitBoundUtils {
    fn get_unique_trait_bound_id(&self) -> String;
}

impl TraitBoundUtils for TraitBound {
    /// We use this when we want to create an "impl" string. It's
    fn get_unique_trait_bound_id(&self) -> String {
        UniqueHashId(self).get_unique_string()
    }
}
