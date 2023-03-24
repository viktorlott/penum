#![allow(dead_code)]
extern crate penum;
use penum::penum;

trait Trait {}
trait Trait2 {}
trait Trait3 {}

impl Trait for i32 {}
impl Trait for f32 {}

impl Trait2 for i32 {}
impl Trait2 for f32 {}

impl Trait3 for i32 {}
impl Trait3 for f32 {}

trait Advanced {}
// impl Advanced for usize {}

struct A<T>(T);

impl<T> Trait for A<T> {}

pub trait AbcTrait {
    fn a(&self) -> Option<i32>;
    fn b(&self) -> &Option<i32>;
}

fn main() {}
