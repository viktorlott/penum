#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum]
trait Trait {
    fn grab(&self) -> i32;
}

impl Trait for i32 {
    fn grab(&self) -> i32 {
        20
    }
}

#[penum( unit | () | (T, ..) where T: ^Trait )]
enum Foo<T: Trait> {
    // FIXME: should work without specifying T: Trait twice
    Bar(T),
    Ber(i32, usize),
    Bur(),
}

fn main() {}
