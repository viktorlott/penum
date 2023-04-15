#![allow(dead_code)]
extern crate penum;
use penum::penum;

/// THIS IS A BUG
///
/// Foo should have where String: AsRef<str> bound...............
///
/// UPDATE: Should variadic be interpreted as a rest placeholder?
#[penum( (..) where String: AsRef<str> )]
enum Foo {
    Bar(f32, i32),
    Ber(String, Vec<String>),
    Bur(),
}

fn main() {}
