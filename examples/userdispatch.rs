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

impl AsInner<i32> for Be {
    fn as_inner(&self) -> &i32 {
        &self.0
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

#[
penum(_ where Ce: ^Special, Be: ^AsInner<i32>)]
enum Foo {
    V1(Al),
    V2(i32, Be),
    V3(Ce),
    V4 { name: String, age: Be },
}

use core::cell::UnsafeCell;
use std::sync::Once;
struct Static<T: Default, F = fn() -> T>(UnsafeCell<Option<T>>, F);
unsafe impl<T: Default> Sync for Static<T> {}
static RETURN: Static<String> = Static::new();
impl<T: Default> Static<T> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None), || T::default())
    }
    fn get(&self) -> &'static T {
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe { *self.0.get() = Some(self.1()) });
        unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
    }
}

struct Inner(String);

impl AsRef<str> for Inner {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[penum(_ where String: ^AsRef<str> )]
enum Store {
    V0(Inner),

    // #[as_ref("Tuple")]
    V1(i32),

    V2(String, i32),
    V3(i32, usize, String),
    V4(i32, String, usize),
    V5 { name: String, age: usize },

    // #[as_ref("Unit")]
    V6,
}

fn main() {
    // let foo_a = Foo::V1(Al);
    // let foo_b = Foo::V2(Be(2));
    // let foo_c = Foo::V3(Ce("hello".to_string()));

    // println!("{}", foo_a.echo());
    // println!("{}", foo_b.echo());
    // println!("{}", foo_c.echo());

    // println!("{}", foo_a.as_inner());
    // println!("{}", foo_b.as_inner());
    // println!("{}", foo_c.as_inner());

    // println!("{:?}", foo_a.ret());
    // println!("{:?}", foo_b.ret());
    // println!("{:?}", foo_c.ret());
}
