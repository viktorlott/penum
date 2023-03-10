extern crate penum;

use penum::penum;

trait Trait {}

#[penum[ (T) where T: Trait, 'a: 'b ]]
enum Must {
    Static(usize),
}

fn main() {}
