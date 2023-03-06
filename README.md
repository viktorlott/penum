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

`penum` is a procedural macro that is used to make an enum conform to a given pattern, which can include generics with trait bounds. Pattern matching, but for enums!

## Installation
This crate is available on [crates.io](https://crates.io/crates/penum) and can be used by adding the following to your project's Cargo.toml:
```toml
[dependencies]
penum = "0.1.11"
```
Or run this command in your cargo project:
```sh
$ cargo add penum
```

## Overview

1. A `penum` expression consists of a `pattern` and an optional `where clause`.
2. A `pattern` can consists of multiple `pattern fragments` that comes in three `group`-flavors, `Named {...}`, `Unnamed (...)` and `Unit`. They are then used for pattern matching against variants. 
4. A `group`'s inner parts can consist of:
    1. `generic` parameters and are used as `named polymorphic placeholders` that CAN have trait bounds, but **CAN ONLY** be declared with capital letters.
    2. `placeholder` parameters and are used as `unnamed polymorphic placeholders` that CANNOT have trait bounds, and are declared with undercore `_`.
    3. `variadic` parameters which are `polymorphic rest placeholders` and **CAN ONLY** be declared as the last argument, and only once (for now).
- `where clause` is used to bind generic parameters to traits which puts constraints on the concrete types that follows.

### Use case
Normally, using a generic in an enum means that it gets applied to the whole enum, and not per variant. For example, if I want to specify that all variants should be a `tuple(T)` where T must implement `Copy`, I'd have to specify a generic for all variants:
```rust
enum Foo where T: Copy, U: Copy, F: Copy {
    Bar(T), 
    Ber(U), 
    Bur(F)
    // But if I now want to add `Bor(D)` to this 
    // enum, I'd have to add it manually, and then
    // bind that generic to impl copy.

    // Also, there is nothing stopping me from 
    // changing the variant shape to `Bor(D, i32)`.
}
```
This seems kind of tedious, because all we want to do is to make the enum conform to a specific pattern, like this:
```rust
// This forces all current and future variants to 
// contain one field which must implement `Copy`.
#[penum( (T) where T: Copy )]
enum Foo {
    Bar(i32), 
    Ber(u32), 
    Bur(f32)
}
```
..which would expand to the first example above, but where T, U and F are replaced with i32, u32 and f32.

#### Supported
- Shapes `(...) | {...}`
- Generics `(T, U) | {num: T}`
- Placeholders `(_, _) | {num: _}`
- Bounds `(T, U) where T: Copy, U: Clone`
- Variadic `(T, U, ..) | {num: T, ..}`
#### Under development
- `Static dispatch` - auto implement `core`/`std`/`custom` traits ([read more](https://github.com/viktorlott/penum/blob/main/docs/static-dispatch.md)).


## Examples
It's also possible to make an enum conform to multiple shapes by seperating a `shape` with `|` symbol, for example:
```rust
#[penum( (T) | (T, T) | { num: T } where T: Copy )]
enum Foo {
    Bar(i32), 
    Bor(i32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```

Also, If an enum should break a `pattern`, like if a variant doesn't implement the correct `Trait`,
an error would occur:
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

Sometime we don't care about specifying a `where clause` and just want our enum to follow a specific `shape`.
This is done by specifing `_`:
```rust
#[penum( (_) | (_, _) | { num: _ } )]
enum Foo {
    Bar(u32), 
    Bor(i32, f32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```

Other times we only care about the first varaint field implementing a trait:
```rust
#[penum( (T, ..) | { num: T, .. } where T: Copy )]
enum Foo {
    Bar(u32), 
    Bor(i32, f32), 
    Ber(u32, i32), 
    Bur { num: f32 }
}
```

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
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
`the trait bound `usize: Trait` is not satisfied`
enum Must {
    Static (usize)
}
```

#### Unsupported
- `RangeLit` - variadic fields by range `(T, U, ..4) | {num: T, ..3}`
- `VariadicLit` - variadic fields with bounds `(T, U, ..Copy) | {num: T, ..Copy}` 
- `Discriminants` - support for `#ident(T) = func(#ident)`, or something..
 

<!-- [![Banner](https://raw.githubusercontent.com/viktorlott/penum/main/penum-logo.png)](https://github.com/viktorlott/penum) -->
<!-- [<img alt="Github" src="https://raw.githubusercontent.com/viktorlott/penum/main/penum-logo.png" height="100">](https://github.com/viktorlott/penum) -->



<!-- A `pattern` consists of one or more `shapes` and an optional `where clause`, that auto bind all concrete types that matches your shape(s)--with the trait bounds you've specified.
- `shapes` can either be `Named {...}`, `Unnamed (...)` or `Unit`, and are used to approve variants.
    - `generic` parameters are used as `named placeholders` which are polymorphic types that CAN have trait bounds, but **CAN ONLY** be declared with capital letters.
    - `placeholder` parameters are used as `unnamed placeholders` which are polymorphic types that CANNOT have trait bounds, and are declared with undercore `_`.
    - `variadic` parameter is used to express that a shape has a polymorphic rest parameter, and **CAN ONLY** be declared as the last argument--once.
- `where clause` is used to bind generic parameters to traits. -->
