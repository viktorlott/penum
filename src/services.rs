use proc_macro::TokenStream;
use quote::format_ident;
use quote::ToTokens;
use syn::parse_macro_input;
use syn::ItemTrait;
use syn::Type;

use crate::dispatch::T_SHM;
use crate::factory::PenumExpr;
use crate::factory::Subject;
use crate::penum::Penum;
use crate::penum::Stringify;

pub fn penum_expand(attr: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: Make it bi-directional, meaning it's also possible to register enums and then do
    // the implementations when we tag a trait. (That is actually better).
    if attr.is_empty() {
        let output = input.clone();
        let item_trait = parse_macro_input!(input as ItemTrait);

        // If we cannot find the trait the user wants to dispatch, we need to store it.
        T_SHM.insert(item_trait.ident.get_string(), item_trait.get_string());

        output
    } else {
        let expr = parse_macro_input!(attr as PenumExpr);
        let subject = parse_macro_input!(input as Subject);

        let penum = Penum::new(expr, subject);

        // Loop through enum definition and match each variant with each
        // shape pattern. for each variant => pattern.find(variant)
        penum.assemble().unwrap_or_error()
    }
}

pub fn to_string_expand(input: TokenStream) -> TokenStream {
    let subject = parse_macro_input!(input as Subject);
    let matching_arms = subject.variants_to_arms(|expr| quote::quote!(format!(#expr)));
    let (subject, has_default) = subject.get_censored_subject_and_default_arm(None);
    let enum_name = &subject.ident;

    quote::quote!(
        #subject

        impl std::string::ToString for #enum_name {
            fn to_string(&self) -> String {
                match self {
                    #matching_arms
                    _ => #has_default
                }
            }
        }
    )
    .to_token_stream()
    .into()
}

pub fn fmt_expand(input: TokenStream) -> TokenStream {
    let subject = parse_macro_input!(input as Subject);
    let matching_arms = subject.variants_to_arms(|expr| quote::quote!(write!(f, #expr)));
    let (subject, has_default) = subject
        .get_censored_subject_and_default_arm(Some(quote::quote!(write!(f, "{}", "".to_string()))));
    let enum_name = &subject.ident;

    quote::quote!(
        #subject

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #matching_arms
                    _ => #has_default
                }
            }
        }
    )
    .to_token_stream()
    .into()
}

pub fn into_expand(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(attr as Type);
    let subject = parse_macro_input!(input as Subject);
    let matching_arms = subject.variants_to_arms(|expr| quote::quote!(#expr));
    let (subject, has_default) =
        subject.get_censored_subject_and_default_arm(Some(quote::quote!(Default::default())));
    let enum_name = &subject.ident;

    quote::quote!(
        #subject

        impl Into<#ty> for #enum_name {
            fn into(self) -> #ty {
                match self {
                    #matching_arms
                    _ => #has_default
                }
            }
        }
    )
    .to_token_stream()
    .into()
}

pub fn deref_expand(
    attr: TokenStream,
    input: TokenStream,
    extend: Option<fn(&Subject) -> proc_macro2::TokenStream>,
) -> TokenStream {
    let ty = parse_macro_input!(attr as Type);
    let subject = parse_macro_input!(input as Subject);
    let matching_arms = subject.variants_to_arms(|expr| quote::quote!(#expr));
    let (subject, has_default) =
        subject.get_censored_subject_and_default_arm(Some(quote::quote!(Default::default())));
    let enum_name = &subject.ident;
    let extensions = extend.map(|extend| extend(&subject));

    quote::quote!(
        #subject

        impl std::ops::Deref for #enum_name {
            type Target = #ty;
            fn deref(&self) -> &Self::Target {
                match self {
                    #matching_arms
                    _ => #has_default
                }
            }
        }

        #extensions
    )
    .to_token_stream()
    .into()
}

pub fn static_str(input: TokenStream) -> TokenStream {
    deref_expand(
        quote::quote!(str).into(),
        input,
        Some(|subject| {
            let enum_name = &subject.ident;

            quote::quote!(
                impl AsRef<str> for #enum_name {
                    fn as_ref(&self) -> &str { &**self }
                }

                impl #enum_name {
                    fn as_str(&self) -> &str  { &**self }
                    fn static_str(&self) -> &str { &**self }
                }
            )
        }),
    )
}

/// UNDER DEVELOPMENT
pub fn lazy_string(input: TokenStream) -> TokenStream {
    let subject = parse_macro_input!(input as Subject);
    let _matching_arms = subject.variants_to_arms(|expr| quote::quote!(#expr));
    let (subject, _has_default) =
        subject.get_censored_subject_and_default_arm(Some(quote::quote!(Default::default())));
    let enum_name = &subject.ident;
    let store_name = format_ident!("STORE_{enum_name}");

    // UNSAFE: This is not safe at all. We are able to drop in-use Strings atm.
    // FIXME: MAKE SAFE.
    quote::quote!(
        #subject

        static mut #store_name: std::sync::OnceLock<std::collections::hash_map::HashMap<u64, String>> = std::sync::OnceLock::new();

        impl std::ops::Deref for OP {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                let _ = unsafe { #store_name.get_or_init(|| std::collections::hash_map::HashMap::new()) };
                let store = unsafe { #store_name.get_mut().unwrap() };

                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::mem::discriminant(self).hash(&mut hasher);

                match self {
                    OP::A(f0) => {
                        f0.hash(&mut hasher);
                        let id = hasher.finish();

                        if !store.contains_key(&id) {
                            let actual_disc_expr = format!("A variant {f0}");
                            store.insert(id, actual_disc_expr);
                        }
                        // Could do unwrap_unchecked when its garanteed to be safe
                        store.get(&id).unwrap()
                    }
                    OP::B(f0) => {
                        f0.hash(&mut hasher);
                        let id = hasher.finish();

                        if !store.contains_key(&id) {
                            let actual_disc_expr = format!("B variant {f0}");
                            store.insert(id, actual_disc_expr);
                        }

                        store.get(&id).unwrap()
                    }
                }
            }
        }

        impl Drop for OP {
            fn drop(&mut self) {
                // We could check if STORE exists, and if it does not, we could just return here.
                // UNSAFE: FIXME
                let _ = unsafe { #store_name.get_or_init(|| std::collections::hash_map::HashMap::new()) };
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::mem::discriminant(self).hash(&mut hasher);

                let id = match self {
                    OP::A(f0) => {
                        f0.hash(&mut hasher);
                        hasher.finish()
                    }
                    OP::B(f0) => {
                        f0.hash(&mut hasher);
                        hasher.finish()
                    }
                };

                let store = unsafe { #store_name.get_mut().unwrap() };
                let _ = store.remove(&id);
            }
        }

    )
    .to_token_stream()
    .into()
}
