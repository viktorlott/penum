#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use syn::parse_macro_input;

use factory::{PenumExpr, Subject};
use penum::Penum;

mod dispatch;
mod error;
mod factory;
mod penum;
mod utils;

/// Use this to make an enum conform to a pattern with or without trait bounds.
///
/// # Examples
/// It's also possible to make an enum conform to multiple shapes by seperating a `shape` with `|` symbol, for example:
/// ```rust
/// #[penum( (T) | (T, T) | { num: T } where T: Copy )]
/// enum Foo {
///     Bar(i32),
///     Ber(u32, i32),
///     Bur { num: f32 }
/// }
/// ```
///
/// Also, If an enum should break a `pattern`, like if a variant doesn't implement the correct `Trait`,
/// an error would occur:
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
/// Sometime we don't care about specifying a `where clause` and just want our enum to follow a specific `shape`.
/// This is done by specifing `_`:
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
    let pattern = parse_macro_input!(attr as PenumExpr);
    let input = parse_macro_input!(input as Subject);

    // Loop through enum definition and match each variant with each shape pattern.
    // for each variant => pattern.find(variant)
    Penum::from(pattern, input).assemble().unwrap_or_error()
}

// struct F {
//     ident: Ident,
//     eq_token: Token![=],
//     s: LitStr,
// }

// struct M {
//     token_enum: Token![enum],
//     name: Ident,
//     brace: token::Brace,
//     fields: Punctuated<F, token::Comma>,

// }

// impl Parse for F {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         Ok(Self { ident: input.parse()?, eq_token: input.parse()?, s: input.parse()? })
//     }
// }

// impl ToTokens for F {
//     fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
//         self.ident.to_tokens(tokens);
//     }
// }

// impl Parse for M {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let token_enum = input.parse()?;
//         let name = input.parse()?;
//         let content;
//         let brace = braced!(content in input);

//         Ok(Self { token_enum, name, brace, fields: content.parse_terminated(F::parse)? })
//     }
// }

// impl ToTokens for M {
//     fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
//         self.token_enum.to_tokens(tokens);
//         self.name.to_tokens(tokens);
//         self.brace.surround(tokens, |tokens| self.fields.to_tokens(tokens))
//     }
// }

// #[proc_macro_attribute]
// pub fn tester(attr: TokenStream, input: TokenStream) -> TokenStream {
//     let pattern = parse_macro_input!(input as M);

//     quote::quote!(#pattern).to_token_stream().into()
// }
