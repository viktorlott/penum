extern crate penum;

use penum::penum;

trait Trait {}

#[penum[ (T) where T: Trait ]]
enum Must {
    Static(usize),
}

fn main() {}
