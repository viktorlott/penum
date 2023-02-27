#![allow(dead_code)]
extern crate enpat;

use enpat::shape;

trait Trait {}
impl Trait for f32 {}
impl Trait for i32 {}

trait Advanced {}
impl Advanced for usize {}

#[shape[(T, T, U) | (T, U) | { name: T } where T: Trait, U: Advanced]]
enum Vector3 {
    Integer(i32, f32, usize),
    Float(f32, i32, usize),
}

#[shape[{ name: _, age: usize } where usize: Advanced]]
enum Strategy<'a> {
    V1 { name: String, age: usize },
    V2 { name: usize, age: usize },
    V3 { name: &'a str, age: usize },
}

#[shape[{ name: &'a str, age: usize }]]
enum Concrete<'a> {
    Static { name: &'a str, age: usize },
}

// #[shape[tuple(_)]]
// enum Must<'a> {
//     Static { name: &'a str, age: usize }
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^
//             `Static { name : & 'a str, age : usize }` doesn't match pattern `tuple(_)`
// }

// #[shape[tuple(T) where T: Trait]]
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// `the trait bound `usize: Trait` is not satisfied`
// enum Must {
//     Static (usize)
// }

fn main() {
    match Vector3::Integer(10, 10.0, 10) {
        Vector3::Integer(num, _, _) => num,
        Vector3::Float(num, _, _) => num as i32,
    };
}
