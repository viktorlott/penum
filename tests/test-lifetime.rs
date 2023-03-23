#![allow(dead_code)]
extern crate penum;
use penum::penum;

// struct A where for<'a, 'b: 'a> &'b str: AsRef<&'a str>;
// fn tester<'a, 'b: 'a, T>() where &'a T: AsRef<&'a str> + 'a {}

// fn sdsd() {
//     tester::<&str>()
// }

// #[penum((T) where T: Copy)]
enum Foo<'a, 'b: 'a> {
    Bar(&'a i32),
    Bar2(&'b i32),
}

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
