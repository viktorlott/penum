#![allow(dead_code)]
extern crate penum;
// use penum::penum;

trait Trait {}
trait Random {}

// TODO: This doesn't work, please fix
//       message: `"dyn Trait"` is not a valid identifier
// impl Random for dyn Trait {}
// #[penum((T) where dyn Trait: Random )]
// enum Foo<'a> {
//     Bar(&'a dyn Trait),
// }

fn main() {}
