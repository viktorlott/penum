#![allow(dead_code, unused_imports)]
use penum::penum;

struct Al;
struct Be(i32);
struct Ce(usize);

#[penum]
trait Echo {
    fn echo(&self) -> String;
}

#[penum]
trait AsInner<T> {
    fn as_inner(&self) -> &T;
}

impl Echo for Al {
    fn echo(&self) -> String {
        "A".to_string()
    }
}
impl Echo for Be {
    fn echo(&self) -> String {
        format!("B {}", self.0)
    }
}
impl Echo for Ce {
    fn echo(&self) -> String {
        format!("C {}", self.0)
    }
}

impl AsInner<i32> for Be {
    fn as_inner(&self) -> &i32 {
        &self.0
    }
}

impl AsInner<usize> for Ce {
    fn as_inner(&self) -> &usize {
        &self.0
    }
}

#[penum( (T) |  where T: ^Echo, Be: ^AsInner<i32>)]
enum Foo {
    V1(Al),
    V2(Be),
    V3(Ce),
}

fn main() {
    let foo_a = Foo::V1(Al);
    let foo_b = Foo::V2(Be(2));
    let foo_c = Foo::V3(Ce(3));

    assert_eq!("", foo_a.echo());
    assert_eq!("B 2", foo_b.echo());
    assert_eq!("", foo_c.echo());

    assert_eq!(&0, foo_a.as_inner());
    assert_eq!(&2, foo_b.as_inner());
    assert_eq!(&0, foo_c.as_inner());
}
