#![allow(dead_code)]
extern crate penum;
use penum::penum;
use std::ops::Add;

#[penum( (T) where T: ^AsRef<str> )]
enum Foo {
    Bar(String),
}

#[penum((T) where T:  ^Add<i32, Output = i32>)]
enum Foo2 {
    Bar(i32),
}

fn main() {
    let foot = Foo::Bar("Word".to_owned());
    assert_eq!("word", foot.as_ref());

    let foot2 = Foo2::Bar(100);
    assert_eq!(300, foot2 + 200);
}
