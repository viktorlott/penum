#![allow(dead_code)]
extern crate penum;
use penum::penum;

/// THIS IS A BUG
/// It should be possible to have a pattern containing conrete types that is ordered like this.
/// That is because we can do an identify check..
// #[penum( (i32, ..) | (..) )]
// enum Foo {
//     Bar(f32, i32),
//     Ber(String, Vec<String>),
//     Bur(),
// }

fn main() {}
