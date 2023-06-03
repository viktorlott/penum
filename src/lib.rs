#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use syn::ItemTrait;

use factory::PenumExpr;
use factory::Subject;
use penum::Penum;
use utils::variants_to_arms;

use crate::dispatch::T_SHM;
use crate::penum::Stringify;

mod dispatch;
mod error;
mod factory;
mod penum;
mod utils;

/// Use this to make an enum conform to a pattern with or without trait
/// bounds.
///
/// # Examples
/// It's also possible to make an enum conform to multiple shapes by
/// seperating a `shape` with `|` symbol, for example:
/// ```rust
/// #[penum( (T) | (T, T) | { num: T } where T: Copy )]
/// enum Foo {
///     Bar(i32),
///     Ber(u32, i32),
///     Bur { num: f32 }
/// }
/// ```
///
/// Also, If an enum should break a `pattern`, like if a variant doesn't
/// implement the correct `Trait`, an error would occur:
/// ```rust
/// #[penum( (T) | (T, T) | { num: T } where T: Copy )]
/// enum Foo {
///     Bar(String),
///         ^^^^^^
///     // ERROR: `String` doesn't implement `Copy`
///     Ber(u32, i32),
///     Bur { num: f32 }
/// }
/// ```
/// ..or if a variant doesn't match the specified `shape`:
/// ```rust
/// #[penum( (T) | (T, T) | { num: T } where T: Copy )]
/// enum Foo {
///     Bar(u32),
///     Ber(u32, i32, i32),
///         ^^^^^^^^^^^^^
///     // Found: `Ber(u32, i32, i32)`
///     // Expected: `(T) | (T, T) | { num: T }`
///     Bur { num: f32 }
/// }
/// ```
/// Sometime we don't care about specifying a `where clause` and just
/// want our enum to follow a specific `shape`. This is done by
/// specifing `_`:
/// ```rust
/// #[penum( (_) | (_, _) | { num: _ } )]
/// enum Foo {
///     Bar(u32),
///     Ber(u32, i32, i32),
///     Bur { num: f32 }
/// }
/// ```
/// If your not into generics, use `impl` expressions instead:
/// ```rust
/// #[penum( (impl Copy, ..) | { num: f32 }]
/// enum Foo {
///     Bar(u32),
///     Ber(u32, i32, i32),
///     Bur { num: f32 }
/// }
/// ```
#[proc_macro_attribute]
pub fn penum(attr: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: Make it bi-directional, meaning it's also possible to register enums and then do
    // the implementations when we tag a trait. (That is actually better).
    if attr.is_empty() {
        let output = input.clone();
        let item_trait = parse_macro_input!(input as ItemTrait);

        // If we cannot find the trait the user wants to dispatch, we need to store it.
        T_SHM.insert(item_trait.ident.get_string(), item_trait.get_string());

        output
    } else {
        let pattern = parse_macro_input!(attr as PenumExpr);
        let input = parse_macro_input!(input as Subject);

        let penum = Penum::from(pattern, input).assemble();

        // Loop through enum definition and match each variant with each
        // shape pattern. for each variant => pattern.find(variant)
        penum.unwrap_or_error()
    }
}

/// Use this to express how `ToString` should be implemented through variants descriminant.
///
/// ```rust
/// #[penum::to_string]
/// enum EnumVariants {
///     Variant0 = "Return on match",
///     Variant1(i32) = "Return {f0} on match",
///     Variant2(i32, u32) = stringify!(f0, f1).to_string(),
///     Variant3 { name: String } = format!("My string {name}"),
///     Variant4 { age: u32 } = age.to_string(),
/// }
/// let enum_variants = Enum::Variant0;
/// println!("{}", enum_variants.to_string());
/// ```
#[proc_macro_attribute]
pub fn to_string(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut subject = parse_macro_input!(input as Subject);

    let matching_arms: proc_macro2::TokenStream =
        variants_to_arms(subject.get_variants().iter(), |expr| {
            quote::quote!(format!(#expr))
        });

    let mut has_default = quote::quote!("".to_string()).to_token_stream();
    subject.data.variants = subject
        .data
        .variants
        .into_iter()
        .filter_map(|mut variant| {
            if variant.discriminant.is_some() && variant.ident == "__Default__" {
                let (_, expr) = variant.discriminant.as_ref().unwrap();
                has_default = quote::quote!(#expr);
                return None;
            }
            variant.discriminant = None;
            Some(variant)
        })
        .collect();

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

/// Use this to express how `Display` should be implemented through variants descriminant.
///
/// ```rust
/// #[penum::fmt]
/// enum EnumVariants {
///     Variant0 = "Return on match",
///     Variant1(i32) = "Return {f0} on match",
///     Variant2(i32, u32) = stringify!(f0, f1).to_string().fmt(f),
///     Variant3 { name: String } = format!("My string {name}").fmt(f),
///     Variant4 { age: u32 } = write!(f, age.to_string()),
/// }
/// let enum_variants = Enum::Variant0;
/// println!("{}", enum_variants);
/// ```
#[proc_macro_attribute]
pub fn fmt(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut subject = parse_macro_input!(input as Subject);

    let matching_arms: proc_macro2::TokenStream =
        variants_to_arms(subject.get_variants().iter(), |expr| {
            quote::quote!(write!(f, #expr))
        });

    let mut has_default = quote::quote!(write!(f, "{}", "".to_string())).to_token_stream();
    subject.data.variants = subject
        .data
        .variants
        .into_iter()
        .filter_map(|mut variant| {
            if variant.discriminant.is_some() && variant.ident == "__Default__" {
                let (_, expr) = variant.discriminant.as_ref().unwrap();
                has_default = quote::quote!(#expr);
                return None;
            }
            variant.discriminant = None;
            Some(variant)
        })
        .collect();

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
