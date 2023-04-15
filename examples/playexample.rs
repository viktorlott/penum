#![allow(dead_code)]
#![allow(unused)]
use penum::penum;

#[penum]
trait Trait2 {}

#[penum]
trait Trait {
    fn go(&self) -> String;
}

impl Trait for i32 {
    fn go(&self) -> String {
        "todo!()".to_string()
    }
}

impl Trait for usize {
    fn go(&self) -> String {
        "todo!()".to_string()
    }
}

impl Trait2 for i32 {}
impl Trait2 for usize {}

#[penum( impl Trait for {usize, i32} )]
enum Mine {
    V1(i32),
    V2(i32),
    V3(usize, i32),
}

fn main() {
    let m = Mine::V2(20);

    let n = m.go();
}
