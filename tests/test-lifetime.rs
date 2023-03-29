#![allow(dead_code)]
extern crate penum;
use penum::penum;

// This should work right?
// #[penum((T) where T: Copy)]
// enum Foo<'a, 'b: 'a> {
//     Bar(&'a i32),
//     Bar2(&'b i32),
// }

// enum Foo<'a, 'b: 'a> where &'a i32: Copy {
//     Bar(&'a i32),
//     Bar2(&'b i32),
// }

#[penum((T) where T: Copy)]
enum Foo2<'a, 'b, T> {
    Bar(&'a T),
    Bar2(&'b i32),
}

#[penum((&'a i32))]
enum Foo3<'a> {
    Bar(&'a i32),
}

fn main() {}
