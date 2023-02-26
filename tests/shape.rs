#![allow(dead_code)]
extern crate enpat;

use enpat::shape;

trait Trait {}

impl Trait for f32 {}
impl Trait for i32 {}

#[shape(variant(T) where T: Trait)]
enum Shape { 
    Integer(i32), 
    Float(f32) 
}

fn main() {
    let shape = Shape::Integer(10);

    println!("{}", match shape { Shape::Integer(num) => num, Shape::Float(num) => num as i32 });
}