#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum]
trait Trait {
    fn grab<'a>(&'a self, input: &'a i32) -> &'a i32;
}

impl Trait for i32 {
    fn grab<'a>(&'a self, input: &'a i32) -> &'a i32 {
        input
    }
}

#[penum( unit | () | (T, ..) where T: ^Trait )]
enum Foo {
    Bar(i32),
    Ber(i32, usize),
    Bur(),
}

fn main() {}
