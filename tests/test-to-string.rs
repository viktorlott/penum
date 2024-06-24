#![allow(dead_code)]
extern crate penum;

#[penum::to_string]
enum Foo {
    Bar(i32) = "{f0}",
    Ber(String) = "{f0}",
    Bur(&'static str) = "{f0}",
    Baz {
        name: String,
    } = "{name}",
    Bez(&'static str) = {
        let x = f0;
        x.to_string()
    },
    Buz,
    default = "fallback for Buz",
}

fn main() {
    let bar = Foo::Bar(10);
    assert_eq!(bar.to_string(), "10");

    let baz = Foo::Baz {
        name: "10".to_string(),
    };
    assert_eq!(baz.to_string(), "10");

    let buz = Foo::Buz;
    assert_eq!(buz.to_string(), "fallback for Buz");
}
