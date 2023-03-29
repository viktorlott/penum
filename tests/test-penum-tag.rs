#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum]
trait Trait {
    fn a(&self) -> &String;
    fn b(&self, x: String) -> &String;
}

fn main() {}
