extern crate penum;

use penum::penum;

#[penum[ (T) where T: ^Trait ]]
enum Must {
    Static(usize),
}

#[penum]
trait Trait {}

impl Trait for usize {}

fn main() {}
