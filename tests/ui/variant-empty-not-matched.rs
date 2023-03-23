extern crate penum;
use penum::penum;

#[penum[tuple(_)]]
enum Must {
    Variant1(i32),
    Variant2()
}

fn main() {}
