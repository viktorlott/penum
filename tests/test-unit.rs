#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum(unit)]
enum Foo {
    Bar,
    Bor,
}

fn main() {}
