#![allow(dead_code)]
extern crate penum;
use penum::penum;
use std::ops::Add;

pub trait AbcTrait {
    fn a(&self) -> Option<i32>;
    fn b(&self) -> &Option<i32>;
    fn c(&self) -> (Option<i32>, &Option<&String>);
}


impl AbcTrait for String {
    fn a(&self) -> Option<i32> {
        Some(10)
    }

    fn b(&self) -> &Option<i32> {
        &Some(20)
    }

    fn c(&self) -> (Option<i32>, &Option<&String>) {
        (Some(30), &None)
    }
}

#[penum( (T) where T: ^AsRef<str> )]
enum Foo {
    Bar(String),
}

#[penum( (T) where T: ^AbcTrait )]
enum Foo1 {
    Bar(String),
}

#[penum((T) where T:  ^Add<i32, Output = i32>)]
enum Foo2 {
    Bar(i32),
    Bor(i32)
}

fn main() {
    let foot = Foo::Bar("Word".to_owned());
    assert_eq!("word", foot.as_ref());

    let foot1 = Foo1::Bar("Word".to_owned());
    assert_eq!(Some(10), foot1.a());
    assert_eq!(&Some(20), foot1.b());
    assert_eq!((Some(30), &None), foot1.c());


    let foot2 = Foo2::Bar(100);
    assert_eq!(300, foot2 + 200);
}
