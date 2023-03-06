#![allow(dead_code)]
extern crate penum;
use penum::penum;

// There's no current support for having variant conform to a naming convention
#[penum(rAnDomWordHeRe)]
enum Foo {
    Bar,
    Bor,
}

fn main() {}
