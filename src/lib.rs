#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;

mod dispatch;
mod error;
mod factory;
mod penum;
mod services;
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
    services::penum_expand(attr, input)
}

/// Use this to express how `ToString` should be implemented through variants descriminant.
///
/// # Example
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
    services::to_string_expand(input)
}

/// Use this to express how `Display` should be implemented through variants descriminant.
///
/// # Example
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
    services::fmt_expand(input)
}

/// Use this to express how `Into<T>` should be implemented through variants descriminant.
///
/// # Example
///
/// ```rust
/// #[penum::into(String)]
/// enum EnumVariants {
///     Variant0 = "Return on match".into(),
///     Variant1(i32) = format!("Return {f0} on match"),
///     Variant2(i32, u32) = stringify!(f0, f1).to_string(),
///     Variant3 { name: String } = format!("My string {name}"),
///     Variant4 { age: u32 } =  age.to_string(),
/// }
/// let enum_variants = Enum::Variant0;
/// println!("{}", enum_variants.into());
/// ```
#[proc_macro_attribute]
pub fn into(attr: TokenStream, input: TokenStream) -> TokenStream {
    services::into_expand(attr, input)
}

/// Use this to express how `Deref<Target = T>` should be implemented through variants descriminant.
///
/// # Example
///
/// ```rust
/// #[penum::deref(str)]
/// enum EnumVariants {
///     Variant0 = "Return on match",
///     Variant1 = { "Evaluated" },
///     Variant2 = concat!(i32, hello),
///     Variant3(&'static str) = f0,
///     Variant4 = &EnumVariants::Variant0,
/// }
/// let enum_variants = Enum::Variant0;
/// println!("{}", &*enum_variants);
/// ```
#[proc_macro_attribute]
pub fn deref(attr: TokenStream, input: TokenStream) -> TokenStream {
    services::deref_expand(attr, input, None)
}

/// Use this to express that you want the enum to implement `deref() -> &str`, `as_str()` and `as_ref()`;
///
/// # Example
///
/// ```rust
/// #[penum::static_str]
/// enum EnumVariants {
///     Variant0 = "Return on match",
///     Variant1 = { "Evaluated" },
///     Variant2 = concat!(i32, hello),
///     Variant3(&'static str) = { f0 },
///     Variant4 = &EnumVariants::Variant0,
/// }
/// let enum_variants = Enum::Variant0;
/// assert_eq!("Return on match", &enum_variants);
/// assert_eq!("Return on match", enum_variants.as_str());
/// assert_eq!("Return on match", enum_variants.as_ref());
/// ```
#[proc_macro_attribute]
pub fn static_str(_: TokenStream, input: TokenStream) -> TokenStream {
    services::static_str(input)
}

/// Use this when you want to be able to associate a ...
/// UNDER DEVELOPMENT
/// # Example
///
/// ```rust
/// #[penum::lazy_string]
/// enum EnumVariants {
///     Variant0      = "Return on match",
///     Variant1(i32) = "{f0}"
/// }
/// let enum_variants = Enum::Variant1(10);
/// assert_eq!("Return on match", &enum_variants);
/// assert_eq!("Return on match", enum_variants.as_str());
/// assert_eq!("Return on match", enum_variants.as_ref());
/// ```
// #[proc_macro_attribute]
#[allow(unused)]
fn lazy_string(_: TokenStream, input: TokenStream) -> TokenStream {
    services::lazy_string(input)
}
