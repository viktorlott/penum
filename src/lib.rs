#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::ItemTrait;

use factory::PenumExpr;
use factory::Subject;

use penum::Penum;

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
    if attr.is_empty() {
        let output = input.clone();
        let item_trait = parse_macro_input!(input as ItemTrait);

        // If we cannot find the trait the user wants to dispatch, we need to store it.
        T_SHM.insert(item_trait.ident.get_string(), item_trait.get_string());

        return output;
    }

    let input = parse_macro_input!(input as Subject);
    let pattern = parse_macro_input!(attr as PenumExpr);

    let penum = Penum::from(pattern, input).assemble();

    // Loop through enum definition and match each variant with each
    // shape pattern. for each variant => pattern.find(variant)
    penum.unwrap_or_error()
}
