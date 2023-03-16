#![allow(dead_code)]
extern crate penum;
use penum::penum;


#[penum( (T) where T: AsRef<str> )]
enum Foo {
    Bar(String),
}

fn main() {}
