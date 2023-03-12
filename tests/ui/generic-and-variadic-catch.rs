#![allow(dead_code)]
extern crate penum;
use penum::penum;

/// This is different though.
/// Because `impl Copy` will implicitly turn into `__IMPL_Copy: Copy`, we count this as a generic,
/// and generics cannot know before hand if there monomorphic counterparts implements the trait specified.
#[penum( (impl Copy, ..) | (..) )]
enum Moo {
    Bar(String, i32),
    Ber(String, Vec<String>),
    Bur(),
}

fn main() {}
