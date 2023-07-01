#![allow(dead_code)]
extern crate penum;
use penum::strukter;

struct Name {
    name: String,
}

#[strukter]
struct A {
    name: x!(String = ""),
}

fn main() {}
