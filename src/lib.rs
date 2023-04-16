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

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use syn::parse_quote;

    use crate::{
        factory::{PenumExpr, Subject},
        penum::Penum,
    };

    fn penum_assertion(attr: TokenStream, input: TokenStream, expect: TokenStream) {
        let pattern: PenumExpr = parse_quote!( #attr );
        let input: Subject = parse_quote!( #input );

        let penum = Penum::from(pattern, input)
            .assemble()
            .get_tokenstream()
            .to_string();

        assert_eq!(penum, expect.to_string());
    }

    #[test]
    #[rustfmt::skip]
    fn test_expression() {
        let attr = quote::quote!(
            (T) where T: Trait
        );
        
        let input = quote::quote!(
            enum Enum {
                V1(i32),
                V2(usize),
                V3(String)
            }
        );

        let expect = quote::quote!(
            enum Enum
            where
                usize: Trait,
                String: Trait,
                i32: Trait
            {
                V1(i32),
                V2(usize),
                V3(String)
            }
        );

        penum_assertion(attr, input, expect)
    }

    #[test]
    #[rustfmt::skip]
    fn test_std_dispatch() {
        let attr = quote::quote!(
            (T) where T: ^AsRef<str>
        );

        let input = quote::quote!(
            enum Enum {
                V1(String),
            }
        );

        let expect = quote::quote!(
            enum Enum where String: AsRef<str> {
                V1(String),
            }

            impl AsRef<str> for Enum {
                fn as_ref(&self) -> &str {
                    match self {
                        Enum::V1(val) => val.as_ref(),
                        _ => ""
                    }
                }
            }
        );

        penum_assertion(attr, input, expect)
    }

    // TODO: Decide how variadics should be interpreted when we have concrete type bounds.
    // Make sure to update `tests/test-concrete-bound.rs` if this later gets supported.
    //
    // #[test]
    // fn test_variadic_with_concrete_type_bound() {
    //     let attr = quote::quote!(
    //         (..) where String: AsRef<str>
    //     );

    //     let input = quote::quote!(
    //         enum Foo {
    //             Bar(f32, i32),
    //             Ber(String, Vec<String>),
    //             Bur(),
    //         }
    //     );

    //     let expect = "enum Foo where String : AsRef < str > { Bar (f32 , i32) , Ber (String , Vec < String >) , Bur () , }".to_string();

    //     penum_assertion(attr, input, expect)
    // }
}
