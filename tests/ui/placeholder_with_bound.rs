extern crate penum;
use penum::penum;

trait Trait {}

#[penum( (_) where i32: Trait ) ]
enum Must {
    V1(i32)
}

fn main() {}
