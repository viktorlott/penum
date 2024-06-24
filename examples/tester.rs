// #![allow(dead_code)]

// use std::io::BufRead;
// use std::ops::Add;
// use std::thread::spawn;

// struct Vuc<T> {
//     pointer: *const T,
//     cap: usize,
//     len: usize,
// }

// const fn vec_const<T>(arr: &[T]) -> Vec<T> {
//     unsafe {
//         (&(&*(arr as *const [T] as *const T), arr.len(), arr.len()) as *const _ as *const Vec<T>)
//             .read()
//     }
// }

// const VEC: &Vec<i32> = &vec_const(&[1, 2, 3, 4]);

// fn len_from_raw<T>(s: &[T]) -> usize {
//     unsafe {
//         let x = *(&s as *const _ as *const (*const T, usize));
//         let raw_slice = std::mem::transmute::<&[T], (*const T, usize)>(s);
//         // raw_slice.1
//         x.1
//     }
// }

// // trait BufReader {
// //     fn read(&self);
// // }

// // impl<T: BufRead> BufReader for T {
// //     default fn read(&self) {
// //         todo!()
// //     }
// // }

// fn main() {
//     let len = unsafe {
//         (*(&(&[1, 1, 1, 1, 1] as *const [i32]) as *const _ as *const (*const i32, usize))).1
//     };

//     let y = &[1, 2, 3, 4];
//     let s = y
//         .iter()
//         .enumerate()
//         .map(|(i, _)| y[..i].iter().sum())
//         .collect::<Vec<i32>>();

//     println!("{:?}", len_from_raw(&vec!(1, 2, 3)));
//     // unsafe {
//     //     let fat_pointer = &(x as *const [i32]) as *const _ as *const (*const i32, usize);
//     //     println!("{:?}", &*fat_pointer);
//     // }

//     println!("{:?}", s);
//     println!("{:?} {} {}", VEC, VEC.capacity(), VEC.len());
// }

fn main() {}
