extern crate penum;

use penum::penum;

trait Trait {}

#[penum[ (impl !Sized) ]]
enum Must {
    Static(usize),
}

fn main() {}
