#![allow(dead_code)]
extern crate penum;
use penum::penum;
use std::convert::AsRef;

#[penum( (T) where T: ^AsRef<str> )]
enum Foo {
    Bar(String),
}

fn main() {}
