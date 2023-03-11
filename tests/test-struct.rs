#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum({ metadata: _ })]
enum Foo {
    Bar { metadata: String },
    Bor { metadata: usize },
}

fn main() {}
