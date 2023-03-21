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

`penum` is a procedural macro that is used to make an enum conform to a
given pattern that can include generics with trait bounds, which then
allows for `static dispatching`. It's a tool for asserting how enums
should look and behave through simple expressive rust grammar.

- **Patterns** — can be though of a toy shape sorter, where the enum
  variants are shape pieces that are trying to fit the given pattern
  we've expressed. There are 3 shapes to choose from, *tuples* `()`,
  *structs* `{}` and *units*. 

- **Trait bounds** — are used in combination with *generic parameters*
  to assert what the matched variants field types should implement, and
  can be expressed like this `where T: Trait<Type>`. The *generic
  parameters* actually needs to be introduced inside a pattern fragment. 

- **Static dispatch** — lets us express how an enum should behave in
  respect to its variants. The symbol that is used to express this is
  `^` and should be put infront of the trait you wish to be dispatched,
  e.g. `(T) where T: ^AsRef<str>`. This is currently limited to rust std
  and core library traits, but there's plans to extend support for
  custom trait definitions soon.
  
- **Impls** — can be seen as a shorthand for *a concrete type that
  implements this trait*, and are primarily used as a substitute for
  regular *generic trait bound expressions*. The look something like
  this, `(impl Copy, impl Copy) | {name: impl Clone}`
  
- **Placeholders** — are single unbounded wildcards, or if you are
  familiar with rust, it's the underscore `_` identifier and usually
  means that something is ignored, which means that they will satisfy
  any type `(_, _) | {num: _}`.
  
- **Variadic** — are similar to placeholders, but instead of only being
  able to substitute one type, variadics can be substituted by 0 or more
  types. Like placeholders, they are a way to express that we don't care
  about the rest of the parameters in a pattern. The look something like
  this`(T, U, ..) | {num: T, ..}`

*Allowing developers to assert how an enum should look and behave.*


## Installation
This crate is available on [crates.io](https://crates.io/crates/penum)
and can be used by adding the following to your project's Cargo.toml:
```toml
[dependencies]
penum = "0.1.14"
```
Or run this command in your cargo project:
```sh
$ cargo add penum
```
## Overview 
- A `Penum` expression can look like this:
```text
#[penum( (T) where T: Trait )]
         ^^^       ^^^^^^^^
         |         |
         |         Predicate bound.
         |
         Pattern fragment.
```
*note that there can be multiple patterns fragments and predicate
bounds.*

- Here's how it would look like if we wanted to dispatch a trait.
```text
#[penum( (T) where T: ^Trait )]
                      |
                      Dispatch symbol
```
#### Super trivial example:
- Here he have an enum with two unary tuple variants where the parameter
  type `Struct` implements the trait `Trait`. The goal is to be able to
  call the trait `method` through `Foo`. This can be accomplished
  automatically marking the trait with a dispatch symbol `^`.
```rust
#[penum{ (T) where T: ^Trait }]
enum Foo {
    V1(Struct), 
    V2(Struct), 
}
```

- Will turn into this:
```rust
impl Trait for Foo {
    fn method(&self, text: &str) {
        match self {
            V1(val) => val.method(text),
            V2(val) => val.method(text),
        }
    }
}
```

*Boilerplate code for aboves examples*
```rust
struct Struct;
trait Trait {
    fn method(&self, text: &str);
}
impl Trait for Struct {}
```

## Examples
It's also possible to make an enum conform to multiple shapes by
seperating a `shape` with `|` symbol, for example:

```rust
#[penum( (T) | (T, T) | { num: T } where T: Copy )]
enum Foo {
    Bar(String), 
        ^^^^^^
    // ERROR: `String` doesn't implement `Copy`
    Bor(i32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```
..or if a variant doesn't match the specified `shape`:
```rust
#[penum( (T) | (T, T) | { num: T } where T: Copy )]
enum Foo {
    Bar(u32), 
    Bor(i32), 
    Ber(u32, i32, i32),
        ^^^^^^^^^^^^^
    // Found: `Ber(u32, i32, i32)` 
    // Expected: `(T) | (T, T) | { num: T }`
    Bwr(String),
        ^^^^^^
    // ERROR: `String` doesn't implement `Copy`
    Bur { num: f32 }
}
```

Sometime we don't care about specifying a `where clause` and just want
our enum to follow a specific `shape`. This is done by specifing `_`:
```rust
#[penum( (_) | (_, _) | { num: _ } )]
enum Foo {
    Bar(u32), 
    Bor(i32, f32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```

Other times we only care about the first varaint field implementing a
trait:
```rust
#[penum( (T, ..) | { num: T, .. } where T: Copy )]
enum Foo {
    Bar(u32), 
    Bor(i32, f32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```

..or you could just use `impl` expressions instead.
```rust
#[penum( (impl Copy, ..) | { num: f32 } )]
enum Foo {
    Bar(u32), 
    Bor(i32, f32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```


#### Under development
- `Static dispatch` - auto implement `core`/`std`/`custom` traits ([read
  more](https://github.com/viktorlott/penum/blob/main/docs/static-dispatch.md)).


## Demo
```rust
use penum::shape;

trait Trait {}
impl Trait for f32 {}
impl Trait for i32 {}

trait Advanced {}
impl Advanced for usize {}

// `(T, FOO, BAR)` are valid generic parameters, but `(t, Foo, BaR)` are not, 
// they are considered as **concrete** types. 
#[penum( (T, T, U) | (T, U) | { name: T } where T: Trait, U: Advanced )]
enum Vector3 {
    Integer(i32, f32, usize),
    Float(f32, i32, usize),
}

#[penum( { name: _, age: usize } where usize: Advanced )]
enum Strategy<'a> {
    V1 { name: String, age: usize },
    V2 { name: usize, age: usize },
    V3 { name: &'a str, age: usize },
}

#[penum( { name: &'a str, age: usize } )]
enum Concrete<'a> {
    Static { name: &'a str, age: usize },
}
```

```rust
#[penum( tuple(_) )]
enum Must<'a> {
    Static { name: &'a str, age: usize }
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // Found: `Static { name : & 'a str, age : usize }`
    // Expected: `tuple(_)`
}
// Note that this shape has a name (`tuple`). Right now 
// it doesn't do anything,but there is an idea of using 
// regexp to be able to validate on Variant names too.

// Also, there is thoughts about using these Idents to 
// specify other rules, like if penum should auto implement
// a static dispatch for a certain pattern. But this could 
// also be done by other rules.

#[penum( tuple(T) where T: Trait )]
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
`the trait bound `usize: Trait` is not satisfied`
enum Must {
    Static (usize)
}
```



|   Traits   |   Supported   |
| ---------- | ------------- |
|`Any`| :heavy_check_mark: supported |
|`Borrow`| :heavy_check_mark: supported |
|`BorrowMut`| :heavy_check_mark: supported |
|`Eq`| :heavy_check_mark: supported |
|`AsMut`| :heavy_check_mark: supported |
|`AsRef`| :heavy_check_mark: supported |
|`From`| :heavy_check_mark: supported |
|`Into`| :heavy_check_mark: supported |
|`TryFrom`| :heavy_check_mark: supported |
|`TryInto`| :heavy_check_mark: supported |
|`Default`| :heavy_check_mark: supported |
|`Binary`| :heavy_check_mark: supported |
|`Debug`| :heavy_check_mark: supported |
|`Display`| :heavy_check_mark: supported |
|`LowerExp`| :heavy_check_mark: supported |
|`LowerHex`| :heavy_check_mark: supported |
|`Octal`| :heavy_check_mark: supported |
|`Pointer`| :heavy_check_mark: supported |
|`UpperExp`| :heavy_check_mark: supported |
|`UpperHex`| :heavy_check_mark: supported |
|`Future`| :heavy_check_mark: supported |
|`IntoFuture`| :heavy_check_mark: supported |
|`FromIterator`| :heavy_check_mark: supported |
|`FusedIterator`| :heavy_check_mark: supported |
|`IntoIterator`| :heavy_check_mark: supported |
|`Product`| :heavy_check_mark: supported |
|`Sum`| :heavy_check_mark: supported |
|`Copy`| :heavy_check_mark: supported |
|`Sized`| :heavy_check_mark: supported |
|`ToSocketAddrs`| :heavy_check_mark: supported |
|`Add`| :heavy_check_mark: supported |
|`AddAssign`| :heavy_check_mark: supported |
|`BitAnd`| :heavy_check_mark: supported |
|`BitAndAssign`| :heavy_check_mark: supported |
|`BitOr`| :heavy_check_mark: supported |
|`BitOrAssign`| :heavy_check_mark: supported |
|`BitXor`| :heavy_check_mark: supported |
|`BitXorAssign`| :heavy_check_mark: supported |
|`Deref`| :heavy_check_mark: supported |
|`DerefMut`| :heavy_check_mark: supported |
|`Div`| :heavy_check_mark: supported |
|`DivAssign`| :heavy_check_mark: supported |
|`Drop`| :heavy_check_mark: supported |
|`Fn`| :heavy_check_mark: supported |
|`FnMut`| :heavy_check_mark: supported |
|`FnOnce`| :heavy_check_mark: supported |
|`Index`| :heavy_check_mark: supported |
|`IndexMut`| :heavy_check_mark: supported |
|`Mul`| :heavy_check_mark: supported |
|`MulAssign`| :heavy_check_mark: supported |
|`MultiMethod`| :heavy_check_mark: supported |
|`Neg`| :heavy_check_mark: supported |
|`Not`| :heavy_check_mark: supported |
|`Rem`| :heavy_check_mark: supported |
|`RemAssign`| :heavy_check_mark: supported |
|`Shl`| :heavy_check_mark: supported |
|`ShlAssign`| :heavy_check_mark: supported |
|`Shr`| :heavy_check_mark: supported |
|`ShrAssign`| :heavy_check_mark: supported |
|`Sub`| :heavy_check_mark: supported |
|`SubAssign`| :heavy_check_mark: supported |
|`Termination`| :heavy_check_mark: supported |
|`SliceIndex`| :heavy_check_mark: supported |
|`FromStr`| :heavy_check_mark: supported |
|`ToString`| :heavy_check_mark: supported |


#### Unsupported
- `RangeLit` - variadic fields by range `(T, U, ..4) | {num: T, ..3}` -
  `VariadicLit` - variadic fields with bounds `(T, U, ..Copy) | {num:
T, ..Copy}` 
- `Discriminants` - support for `#ident(T) = func(#ident)`, or
  something..
 