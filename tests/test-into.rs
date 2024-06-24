#![allow(dead_code)]
extern crate penum;

#[penum::into(String)]
enum Foo {
    Bar = "Bar".to_string(),
    Ber = "Ber".to_string(),
    Bur(&'static str) = format!("{f0}"),
}

fn main() {
    let bar = Foo::Bur("10");
    let string: String = bar.into();
    assert_eq!(string, "10");
}
