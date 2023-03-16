#![allow(dead_code)]
use penum::{penum};

trait Trait {}
impl Trait for f32 {}
impl Trait for i32 {}

trait Advanced {}
impl Advanced for usize {}


#[penum[(T, T, U) | (T, U) | { name: T } where T: Trait, U: Advanced]]
enum Vector3 {
    Integer(i32, f32, usize),
    Float(f32, i32, usize),
}

#[penum[{ name: _, age: usize } where usize: Advanced]]
enum Strategy<'a> {
    V1 { name: String, age: usize },
    V2 { name: usize, age: usize },
    V3 { name: &'a str, age: usize },
}

#[penum[{ name: &'a str, age: usize }]]
enum Concrete<'a> {
    Static { name: &'a str, age: usize },
}

#[penum[(T, U, ..) where T: Trait, U: Advanced]]
enum Variadic {
    V1(i32, usize, String, u8, u16),
}

fn main() {
    match Vector3::Integer(10, 10.0, 10) {
        Vector3::Integer(num, _, _) => num,
        Vector3::Float(num, _, _) => num as i32,
    };

}
