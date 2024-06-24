#![allow(dead_code)]
extern crate penum;

#[penum::static_str]
enum Foo {
    Bar = "Bar",
    Ber = &Foo::Bar,
    Bur(&'static str) = f0,
}

fn accept_str(_input: &str) {}

fn main() {
    let bar = Foo::Bur("Bur");
    assert_eq!(&*bar, "Bur");

    accept_str(&bar);
    accept_str(bar.as_str());
    accept_str(bar.as_ref());
}
