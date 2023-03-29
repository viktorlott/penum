#![allow(dead_code, unused_imports)]
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

#[
penum(
    (T) where
        Ce: ^Special,
        T: ^Echo,
        Be: ^AsInner<i32>,
)]
enum Foo {
    V1(Al),
    V2(Be),
    V3(Ce),
}

fn main() {
    let foo_a = Foo::V1(Al);
    let foo_b = Foo::V2(Be(2));
    let foo_c = Foo::V3(Ce("hello".to_string()));

    println!("{}", foo_a.echo());
    println!("{}", foo_b.echo());
    println!("{}", foo_c.echo());

    println!("{}", foo_a.as_inner());
    println!("{}", foo_b.as_inner());
    println!("{}", foo_c.as_inner());

    println!("{:?}", foo_a.ret());
    println!("{:?}", foo_b.ret());
    println!("{:?}", foo_c.ret());
}
