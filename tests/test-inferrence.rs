#![allow(dead_code)]
extern crate penum;
use penum::penum;

struct Al;
struct Be(i32);
struct Ce(String);

#[penum]
trait Echo {
    fn echo(&self) -> String;
}

#[penum]
trait Special {
    fn ret(&self) -> Option<&String>;
}

#[penum]
trait AsInner<T> {
    fn as_inner(&self) -> &T;
}

impl Special for Ce {
    fn ret(&self) -> Option<&String> {
        Some(&self.0)
    }
}

impl AsInner<i32> for Be {
    fn as_inner(&self) -> &i32 {
        &self.0
    }
}

#[penum(_ where Ce: ^Special, Be: ^AsInner<i32>)]
enum Foo {
    V1(Al),
    V2(i32, Be),
    V3(Ce),
    V4 { name: String, age: Be },
}

fn main() {}
