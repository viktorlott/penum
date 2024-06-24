// #![allow(dead_code)]
// use std::{
//     alloc::GlobalAlloc,
//     hash::{Hash, Hasher},
//     iter::Enumerate,
//     marker::PhantomData,
//     ptr::NonNull,
// };

// use penum::penum;
// use std::alloc::System;

// trait Trait {}
// trait Trait2 {}
// trait Trait3 {}

// impl Trait for i32 {}
// impl Trait for f32 {}

// impl Trait2 for i32 {}
// impl Trait2 for f32 {}

// impl Trait3 for i32 {}
// impl Trait3 for f32 {}

// struct Bufv<T, A: GlobalAlloc = System> {
//     pointer: *const T,
//     cap: usize,
//     alloc: A,
//     _marker: PhantomData<T>,
// }
// struct V<T> {
//     buf: Bufv<T>,
//     len: usize,
// }

// const VEC: Vec<i32> = {
//     const ARR: &[i32] = &[1; 10];

//     let pointer = ARR as *const [i32];
//     let ve = V::<i32> {
//         buf: Bufv {
//             pointer: unsafe { &*pointer as *const [i32] as *const i32 },
//             cap: 10,
//             alloc: System,
//             _marker: PhantomData,
//         },
//         len: 10,
//     };

//     let m = &ve as *const _ as *const Vec<i32>;

//     unsafe { m.read() }
// };

// #[penum::to_string]
// enum EnumVariants {
//     Variant0 = "Return on match",
//     Variant1(i32) = "Return {f0} on match",
//     Variant2(i32, u32) = stringify!(f0, f1).to_string(),
//     Variant3 {
//         name: String,
//     } = format!("My string {name}"),
//     Variant4 {
//         age: u32,
//     } = age.to_string(),
//     Variant5 = EnumVariants::Variant0.to_string(),
//     Variant6 {
//         list: Vec<u32>,
//     } = {
//         let string = list.iter().map(ToString::to_string).collect::<String>();

//         format!("List: ({string})")
//     },
//     Variant7,
//     Variant8,

//     // Note that default will not appear in the Enum, i.e `EnumVariants::default` will not exist.
//     default = "Variant7 and Variant8 will use this",
// }

// trait Advanced {}
// // impl Advanced for usize {}

// #[penum::static_str]
// enum ABC {
//     A = "HELLO",
//     B = concat!("OJ", "df"),
//     C = &ABC::A,
//     D,
//     E,
//     default = "D and E will fall through to this",
// }

// // Plan is to get lazy string slices to work.
// #[derive(Hash)]
// enum OP {
//     A(i32),    // = "A variant {f0}", // hello
//     B(String), // = { if f0.contains("hello") {"left {f0}"} else {"right {f0}"} } // Will work

//     C(String), // = { if Random::number() < 500 {"left {f0}"} else {"right {f0}"} } // Will not work
//                // The above will only cache the first value
// }

// fn get_store() -> &'static mut std::collections::hash_map::HashMap<u64, String> {
//     static mut STORE: std::sync::OnceLock<std::collections::hash_map::HashMap<u64, String>> =
//         std::sync::OnceLock::new();

//     // THIS IS SO UNSAFE; We can drop in-use values....
//     unsafe {
//         let _ = STORE.get_or_init(|| std::collections::hash_map::HashMap::new());
//         STORE.get_mut().unwrap()
//     }
// }

// impl std::ops::Deref for OP {
//     type Target = str;

//     fn deref(&self) -> &Self::Target {
//         let mut hasher = std::collections::hash_map::DefaultHasher::new();
//         let store = get_store();

//         std::mem::discriminant(self).hash(&mut hasher);
//         self.hash(&mut hasher);

//         let id = hasher.finish();

//         if !store.contains_key(&id) {
//             match self {
//                 OP::A(f0) => {
//                     let actual_disc_expr = format!("A variant {f0}");
//                     store.insert(id, actual_disc_expr);
//                 }
//                 OP::B(f0) => {
//                     let actual_disc_expr = format!("B variant {f0}");
//                     store.insert(id, actual_disc_expr);
//                 }
//                 _ => todo!(),
//             }
//         }

//         store.get(&id).unwrap()
//     }
// }

// impl Drop for OP {
//     fn drop(&mut self) {
//         let mut hasher = std::collections::hash_map::DefaultHasher::new();
//         std::mem::discriminant(self).hash(&mut hasher);
//         self.hash(&mut hasher);
//         let id = hasher.finish();
//         let _ = get_store().remove(&id);
//     }
// }

// // enum ABC {
// //     A,
// //     B,
// //     C,
// // }

// // impl std::ops::Deref for ABC {
// //     type Target = str;
// //     fn deref(&self) -> &Self::Target {
// //         match self {
// //             Self::A => "HELLO",
// //             Self::B => {
// //                 concat!("OJ", "df")
// //             }
// //             Self::C => "PEE",
// //             _ => Default::default(),
// //         }
// //     }
// // }
// // enum ABC {
// //     A,
// //     B,
// //     C,
// // }
// // impl std::ops::Deref for ABC {
// //     type Target = str;
// //     fn deref(&self) -> &Self::Target {
// //         match self {
// //             Self::A => "HELLO",
// //             Self::B => "OJ",
// //             Self::C => "PEE",
// //             _ => Default::default(),
// //         }
// //     }
// // }

// struct A<T>(T);

// impl<T> Trait for A<T> {}

// #[penum((T) where T: Copy)]
// enum Foo {
//     Bar(i32),
//     Bor(i32),
// }

// #[penum( _ where i32: Trait )]
// enum B {
//     V1(usize),
//     V2(usize, i32),
//     V3(usize, usize, i32),
// }

// enum Opt<T> {
//     Some(T),
//     None,
// }
// struct Abc(String);

// impl Abc {
//     fn a(&self) -> &Opt<i32> {
//         &Opt::None
//     }
//     fn b(&self) -> &Option<i32> {
//         &None
//     }
//     fn c(&self) -> &Result<i32, ()> {
//         &Err(())
//     }
//     fn d(&self) -> &i32 {
//         // &i32::default() Doesn't work (cannot return reference to temporary value)
//         &10 // Work
//     }
//     fn e(&self) -> &str {
//         {
//             use std::cell::UnsafeCell;
//             struct Static<T: Default>(UnsafeCell<Option<T>>);
//             unsafe impl<T: Default> Sync for Static<T> {}
//             impl<T: Default> Static<T> {
//                 pub const fn new() -> Self {
//                     Self(UnsafeCell::new(None))
//                 }
//                 fn get(&self) -> &'static T {
//                     unsafe { &mut *self.0.get() }.get_or_insert_with(|| T::default())
//                 }
//             }
//             static RETURN: Static<String> = Static::new();
//             RETURN.get()
//         }
//     }

//     fn f(&self) -> &String {
//         thread_local! {}
//         {
//             use core::cell::UnsafeCell;
//             use std::sync::Once;
//             struct Static<T: Default, F = fn() -> T>(UnsafeCell<Option<T>>, F);
//             unsafe impl<T: Default> Sync for Static<T> {}
//             static RETURN: Static<String> = Static::new();
//             impl<T: Default> Static<T> {
//                 pub const fn new() -> Self {
//                     Self(UnsafeCell::new(None), || T::default())
//                 }
//                 fn get(&self) -> &'static T {
//                     static INIT: Once = Once::new();
//                     INIT.call_once(|| unsafe { *self.0.get() = Some(self.1()) });
//                     unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
//                 }
//             }
//             RETURN.get()
//         }
//     }
// }

// fn accept_str(_input: &str) {}

// fn main() {
//     let _x = Abc("23".to_string());

//     let nn = ABC::A;

//     accept_str(&nn);
//     accept_str(nn.as_str());
//     accept_str(nn.as_ref());

//     let mn = ABC::D;
//     println!("{}", mn.as_str());

//     let ooo = OP::A(100);
//     let ooo2 = OP::A(102);

//     println!("{} {}", &*ooo, &*ooo2);
// }

fn main() {}
