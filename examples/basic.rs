#![allow(dead_code)]
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

#[penum::static_str]
enum ABC {
    A = "HELLO",
    B = concat!("OJ", "df"),
    C = &ABC::A,
}

// enum ABC {
//     A,
//     B,
//     C,
// }

// impl std::ops::Deref for ABC {
//     type Target = str;
//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::A => "HELLO",
//             Self::B => {
//                 concat!("OJ", "df")
//             }
//             Self::C => "PEE",
//             _ => Default::default(),
//         }
//     }
// }
// enum ABC {
//     A,
//     B,
//     C,
// }
// impl std::ops::Deref for ABC {
//     type Target = str;
//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::A => "HELLO",
//             Self::B => "OJ",
//             Self::C => "PEE",
//             _ => Default::default(),
//         }
//     }
// }

struct A<T>(T);

impl<T> Trait for A<T> {}

#[penum((T) where T: Copy)]
enum Foo {
    Bar(i32),
    Bor(i32),
}

#[penum( _ where i32: Trait )]
enum B {
    V1(usize),
    V2(usize, i32),
    V3(usize, usize, i32),
}

enum Opt<T> {
    Some(T),
    None,
}
struct Abc(String);

impl Abc {
    fn a(&self) -> &Opt<i32> {
        &Opt::None
    }
    fn b(&self) -> &Option<i32> {
        &None
    }
    fn c(&self) -> &Result<i32, ()> {
        &Err(())
    }
    fn d(&self) -> &i32 {
        // &i32::default() Doesn't work (cannot return reference to temporary value)
        &10 // Work
    }
    fn e(&self) -> &str {
        {
            use std::cell::UnsafeCell;
            struct Static<T: Default>(UnsafeCell<Option<T>>);
            unsafe impl<T: Default> Sync for Static<T> {}
            impl<T: Default> Static<T> {
                pub const fn new() -> Self {
                    Self(UnsafeCell::new(None))
                }
                fn get(&self) -> &'static T {
                    unsafe { &mut *self.0.get() }.get_or_insert_with(|| T::default())
                }
            }
            static RETURN: Static<String> = Static::new();
            RETURN.get()
        }
    }

    fn f(&self) -> &String {
        thread_local! {}
        {
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
            RETURN.get()
        }
    }
}

fn accept_str(_input: &str) {}

fn main() {
    let _x = Abc("23".to_string());

    let nn = ABC::A;

    accept_str(&nn);
    accept_str(nn.as_str());
    accept_str(nn.as_ref());
}
