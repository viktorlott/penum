#![allow(dead_code)]
extern crate penum;
use penum::to_string;

#[to_string]
enum Foo {
    Bar(i32) = "{f0}",
    Ber {
        name: String,
    } = "{name}",
    Bur(&'static str) = {
        let x = f0;
        x.to_string()
    },
}

fn main() {
    let bar = Foo::Bar(10);
    assert_eq!(bar.to_string(), "10");
}
