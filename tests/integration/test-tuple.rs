#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum((_,_))]
enum Foo {
    Bar(i32, usize),
    Bor(usize, i32),
}

fn main() {}
