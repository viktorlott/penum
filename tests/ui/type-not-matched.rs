extern crate penum;

use penum::penum;

trait Trait {}

#[penum[ (i32) ]]
enum Foo {
    Bar(usize),
}

fn main() {}
