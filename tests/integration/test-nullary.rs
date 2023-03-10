#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum[()]]
enum Foo {
    Bar(),
    Bor(),
}

fn main() {}
