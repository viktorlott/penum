# penum

[<img alt="github" src="https://img.shields.io/github/languages/code-size/viktorlott/penum?style=flat-square&logo=github" height="20">](https://github.com/viktorlott/penum)
[<img alt="crates.io" src="https://img.shields.io/crates/v/penum?style=flat-square&logo=rust" height="20">](https://crates.io/crates/penum)

`penum` is a procedural macro that is used to make an enum follow a given pattern, which can include generics with trait bounds.

A `pattern` consists of one or more `shapes` and an optional `where clause`, which will autobind the concrete types specified for you.
  - `shape` can either be `Named`, `Unnamed` or `Unit`, and are used to validate variants.
  - `where clause` are used to bind the generic parameters to a traits.
  
Normally, using a generic in an enum means that it gets applied to the whole enum, and not per variant. 
For example, if I want to specify that all variants should be a `tuple(T)` where T must implement `Copy`, 
I'd have to specify a generic for all variants:

```rust
enum Foo where T: Copy, U: Copy, F: Copy {
    Bar(T), Ber(U), Bur(F), 
    // But if I now want to add `Bor(D)` to this 
    // enum, I'd have to add it manually, and then
    // bind that generic to impl copy.
    // Also, there is nothing stopping me from 
    // changing the variant shape to `Bor(D, i32)`.
}
```

This seems kind of tedious, because all we want to do is to be able to make the enum conform to a specific pattern, 
like this:
```rust
#[shape[ (T) where T: Copy ]]
enum Foo {
    Bar(i32), Ber(u32), Bur(f32),
}
```
..which would expand to:
```rust
#[shape[ (T) where T: Copy ]]
enum Foo {
    Bar(i32), Ber(u32), Bur(f32),
}
```

There are much more one could do with this, for example, one could specify that an enum should follow a pattern 
with multiple different shapes:
```rust
#[shape[ (T) | (T, T) | { number: T } where T: Copy ]]
enum Foo {
    Bar(i32), Ber(u32, i32), Bur { number: f32 },
}
```

Also, If an enum should break a pattern, then an error will occur:
```rust
#[shape[ (T) | (T, T) | { number: T } where T: Copy ]]
enum Foo {
    Bar(String), Ber(u32, i32), Bur { number: f32 },
        ^^^^^^
       ERROR: `String` doesn't implement `Copy`
}
```

```rust
#[shape[ (T) | (T, T) | { number: T } where T: Copy ]]
enum Foo {
    Bar(u32), Ber(u32, i32, i32), Bur { number: f32 },
                  ^^^^^^^^^^^^^
`Ber(u32, i32, i32)` doesn't match pattern `(T) | (T, T) | { number: T }`
}
```


# Examples
```rust
use penum::shape;

trait Trait {}
impl Trait for f32 {}
impl Trait for i32 {}

trait Advanced {}
impl Advanced for usize {}

#[shape[(T, T, U) | (T, U) | { name: T } where T: Trait, U: Advanced]]
enum Vector3 {
    Integer(i32, f32, usize),
    Float(f32, i32, usize),
}

#[shape[{ name: _, age: usize } where usize: Advanced]]
enum Strategy<'a> {
    V1 { name: String, age: usize },
    V2 { name: usize, age: usize },
    V3 { name: &'a str, age: usize },
}

#[shape[{ name: &'a str, age: usize }]]
enum Concrete<'a> {
    Static { name: &'a str, age: usize },
}
```


```rust
#[shape[tuple(_)]]
enum Must<'a> {
    Static { name: &'a str, age: usize }
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^
`Static { name : & 'a str, age : usize }` doesn't match pattern `tuple(_)`
}

#[shape[tuple(T) where T: Trait]]
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
`the trait bound `usize: Trait` is not satisfied`
enum Must {
    Static (usize)
}
```

## Future support
- Discriminants
- Implement