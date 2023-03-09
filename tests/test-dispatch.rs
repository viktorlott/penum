#![allow(dead_code)]
extern crate penum;
use penum::penum;
use std::ops::Add;

#[penum((T) where T: ^Copy + ^Add<i32>)]
enum Foo {
    Bar(i32),
}

fn main() {}
