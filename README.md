<div align="center">
    <img alt="Github" src="https://raw.githubusercontent.com/viktorlott/penum/main/penum-logo.png" height="103">
</div>

<div align="center">
    <a href="https://github.com/viktorlott/penum">
        <img alt="Github" src="https://img.shields.io/github/languages/code-size/viktorlott/penum?style=flat-square&logo=github" height="20">
        <img alt="Download" src="https://img.shields.io/crates/d/penum.svg?style=flat-square&logo=rust" height="20">
        <img alt="Tests" src="https://img.shields.io/github/actions/workflow/status/viktorlott/penum/test.yaml?branch=main&style=flat-square&logo=github">
    </a>
    <a href="https://crates.io/crates/penum">
        <img alt="crates.io" src="https://img.shields.io/crates/v/penum.svg?style=flat-square&logo=rust" height="20">
    </a>
    <a href="https://docs.rs/penum/latest/penum/">
        <img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-penum-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">
    </a>
</div>

`penum` is a procedural macro that is used for **enum conformity** and
**static dispatch**. This is done by specifying a declarative pattern
that expresses how we should interpret the enum. It's a tool for
asserting how enums should **look** and **behave** through simple
expressive rust grammar.

- **Patterns** — can be thought of as a *toy shape sorter* that sorts
  through enum variants and makes sure they fit. So each variant has a
  certain shape that must satisfy the patterns we've specified. There
  are 3 shapes to choose from, *tuples* `()`, *structs* `{}` and
  *units*.

- **Predicates** — are used in combination with *patterns* to assert
  what the matched variants field types should implement. They can be
  expressed like a regular where clause, e.g `where T: Trait<Type>`. The
  *generic parameters* needs to be introduced inside a pattern fragment.

- **Smart dispatch** — lets us express how an enum should **behave** in
  respect to its variants. The symbol that is used to express this is
  `^` and should be put in front of the trait you wish to be dispatched.

## Installation
This crate is available on [crates.io](https://crates.io/crates/penum)
and can be used by adding the following to your project's Cargo.toml:
```toml
[dependencies]
penum = "0.1.27"
```
Or run this command in your cargo project:
```sh
$ cargo add penum
```

## Latest feature

You can now use enum `descriminants` as expression blocks for `ToString` and `Display`.

```rust
#[penum::to_string]
enum EnumVariants {
    Variant0                    = "Return on match",
    Variant1(i32)               = "Return {f0} on match",
    Variant2(i32, u32)          = stringify!(f0, f1).to_string(),
    Variant3 { name: String }   = format!("My string {name}"),
    Variant4 { age: u32 }       = age.to_string(),
    Variant5 { list: Vec<u32> } = {
        let string = list
            .iter()
            .map(ToString::to_string)
            .join(", ");

        format!("List: ({string})")
    },
}

let enum_variants = Enum::Variant0;
println!("{}", enum_variants.to_string());
```

Add the attribute `#[penum::to_string]` or `#[penum::fmt]` to replace strict enum descriminant.

------------------------------------------------------------------

## Overview 

A `Penum` expression can look like this:
```console
                      Dispatch symbol.
                      |
#[penum( (T) where T: ^Trait )]
         ^^^       ^^^^^^^^^
         |         |
         |         Predicate bound.
         |
         Pattern fragment.
```

A `Penum` expression without specifying a pattern:
```console
#[penum( impl Trait for Type )]
         ^^^^^^^^^^^^^^^^^^^
```
*Shorthand syntax for `_ where Type: ^Trait`*

<details>
<summary>More details</summary>

Important to include `^` for traits that you want to dispatch.
```rust
#[penum( impl Type: ^Trait )]
```

Note that in a penum impl for expression, no `^` is needed.
```rust
#[penum( impl Trait for Type )]
```
In Rust 1.68.0, `From<bool>` for `{f32,f64}` has stabilized. 
That means you can do this.
```rust
#[penum( impl From<bool> for {f32,f64} )]
```

</details>

<br />

### Trivial example
Use `Penum` to automatically `implement` a trait for the enum. 

```rust
#[penum(impl String: ^AsRef<str>)]
enum Store {
    V0(),
    V1(i32),
    V2(String, i32),
    V3(i32, usize, String),
    V4(i32, String, usize),
    V5 { age: usize, name: String },
    V6,
}
```
- Will turn into this:
```rust
impl AsRef<str> for Store {
    fn as_ref(&self) -> &str {
        match self {
            Store::V2(val, ..) => val.as_ref(),
            Store::V3(_, _, val) => val.as_ref(),
            Store::V4(_, val, ..) => val.as_ref(),
            Store::V5 { name, .. } => name.as_ref(),
            _ => "",
        }
    }
}
```

There is also support for user defined traits, but make sure that they
are tagged before the enum.
```rust
#[penum]
trait Trait {
    fn method(&self, text: &str) -> &Option<&str>;
}
```

<br />

<details>
<summary>Supported std traits</summary>

`Any`, `Borrow`, `BorrowMut`, `Eq`, `AsMut`, `AsRef`, `From`, `Into`,
`TryFrom`, `TryInto`, `Default`, `Binary`, `Debug`, `Display`,
`LowerExp`, `LowerHex`, `Octal`, `Pointer`, `UpperExp`, `UpperHex`,
`Future`, `IntoFuture`, `FromIterator`, `FusedIterator`, `IntoIterator`,
`Product`, `Sum`, `Sized`, `ToSocketAddrs`, `Add`, `AddAssign`,
`BitAnd`, `BitAndAssign`, `BitOr`, `BitOrAssign`, `BitXor`,
`BitXorAssign`, `Deref`, `DerefMut`, `Div`, `DivAssign`, `Drop`,
`Index`, `IndexMut`, `Mul`, `MulAssign`, `MultiMethod`, `Neg`, `Not`,
`Rem`, `RemAssign`, `Shl`, `ShlAssign`, `Shr`, `ShrAssign`, `Sub`,
`SubAssign`, `Termination`, `SliceIndex`, `FromStr`, `ToString`

</details>

`Penum` is smart enough to infer certain return types for non-matching
variants. e.g `Option<T>`, `&Option<T>`, `String`, `&str`. It can even
handle `&String`, referenced non-const types. The goal is to support any
type, which we could potentially do by checking for types implementing
the `Default` trait.

Note, when dispatching traits with associated types, it's important to
declare them. e.g `Add<i32, Output = i32>`.

## Examples
Used penum to force every variant to be a tuple with one field that must
implement `Trait`.

```rust
#[penum( (T, ..) where T: Trait )]
enum Guard {
    Bar(String), 
        ^^^^^^
    // ERROR: `String` doesn't implement `Trait`

    Bor(Option<String>), 
        ^^^^^^^^^^^^^^
    // ERROR: `Option<String>` doesn't implement `Trait`

    Bur(Vec<String>), 
        ^^^^^^^^^^^
    // ERROR: `Vec<String>` doesn't implement `Trait`

    Byr(), 
    ^^^^^
    // ERROR: `Byr()` doesn't match pattern `(T)`

    Bxr { name: usize }, 
        ^^^^^^^^^^^^^^^
    // ERROR: `{ nname: usize }` doesn't match pattern `(T)`

    Brr,
    ^^^
    // ERROR: `Brr` doesn't match pattern `(T)`

    Bir(i32, String), // Works!
    Beer(i32)         // Works!
}
```

If you don't care about the actual pattern matching, then you could use
`_` to automatically infer every shape and field. Combine this with
concrete dispatch types, and you got yourself a auto dispatcher.


<details>
<summary>Under development</summary>

For non-std types we rely on the `Default` trait, which means, if we can
prove that a type implements `Default` we can automatically add them as
return types for non-matching variants,

</details>



```rust
#[penum( _ where Ce: ^Special, Be: ^AsInner<i32> )]
enum Foo {
    V1(Al),
    V2(i32, Be),
    V3(Ce),
    V4 { name: String, age: Be },
}

// Will create these implementations
impl Special for Foo {
    fn ret(&self) -> Option<&String> {
        match self {
            Foo::V3(val) => val.ret(),
            _ => None,
        }
    }
}

impl AsInner<i32> for Foo {
    fn as_inner(&self) -> &i32 {
        match self {
            Foo::V2(_, val) => val.as_inner(),
            Foo::V4 { age, .. } => age.as_inner(),
            _ => &0,
        }
    }
}
```

- It's identical to this:
```rust
#[penum(impl Ce: ^Special, Be: ^AsInner<i32>)]
```

#### More details

- **Impls** — can be seen as a shorthand for *a concrete type that
  implements this trait*, and are primarily used as a substitute for
  regular *generic trait bound expressions*. They look something like
  this, `(impl Copy, impl Copy) | {name: impl Clone}`

- **Placeholders** — are single unbounded wildcards, or if you are
  familiar with rust, it's the underscore `_` identifier and usually
  means that something is ignored, which means that they will satisfy
  any type `(_, _) | {num: _}`.

- **Variadic** — are similar to placeholders, but instead of only being
  able to substitute one type, variadics can be substituted by 0 or more
  types. Like placeholders, they are a way to express that we don't care
  about the rest of the parameters in a pattern. The look something like
  this`(T, U, ..) | {num: T, ..}`.




  <!-- ,
  e.g. `(T) where T: ^AsRef<str>`. The dispatcher is smart enough to
  figure out certain return types for methods such that non-matching
  variants can be assigned with a *default* return statement. i.e types
  like `Option<_>`, `Result<_, E>` and many other types (*including
  Primitive Types*) can get defaulted automatically for us instead of
  returning them with panic. *This is currently limited to rust std
  library traits, but there are plans to extend support for custom trait
  definitions soon.* -->