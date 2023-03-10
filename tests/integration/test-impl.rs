#![allow(dead_code)]
extern crate penum;
use penum::penum;

trait Trait {}
impl Trait for i32 {}
impl Trait for usize {}

#[penum((impl Trait, impl Copy))]
enum Foo {
    Bar(i32, i32),
    Bor(usize, i32),
}

fn main() {}
