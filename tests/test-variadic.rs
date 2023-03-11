#![allow(dead_code)]
extern crate penum;
use penum::penum;

#[penum[(..)]]
enum Foo {
    Bar(i32, i32, i32, i32, i32, i32, usize, String, Vec<String>),
    Ber(String, Vec<String>),
    Bur(),
}

fn main() {}
