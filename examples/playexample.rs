#![allow(dead_code)]
#![allow(unused)]
use penum::penum;

#[penum]
trait Trait {
    fn go(&self) -> String;
}

#[penum]
trait Trait2 {
    fn go2(&self) -> String;
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

impl Trait2 for i32 {
    fn go2(&self) -> String {
        "todo!()".to_string()
    }
}
impl Trait2 for usize {
    fn go2(&self) -> String {
        "todo!()".to_string()
    }
}

#[penum( impl Trait for {usize, i32} )]
enum Mine {
    V1(i32),
    V2(i32),
    V3(usize, i32),
}

#[penum( (T) | (U, T) where usize: ^Trait, i32: ^Trait2 )]
enum Mine2 {
    V1(i32),
    V2(i32),
    V3(usize, i32),
}

#[penum( (T) | (U, T) where usize: ^Trait, usize: ^Trait2 )]
enum Mine3 {
    V1(i32),
    V2(i32),
    V3(usize, i32),
}

// FIXME: This skips the T dispatch.
#[penum( (T) | (U, T) where T: ^Trait, T: ^Trait2, usize: ^Trait2 )]
enum Mine4 {
    V1(i32),
    V2(i32),
    V3(usize, i32),
}

#[penum( (T) | (T, U) where T: ^Trait, T: ^Trait2)]
enum Mine5 {
    V1(i32),
    V2(i32),
    V3(i32, i32),
}
// impl Trait2 for Mine4 {
//     fn go2(&self) -> String {
//         match self {
//             Mine4::V3(val, ..) => val.go2(),
//             _ => "".to_string(),
//         }
//     }
// }

fn main() {
    let m = Mine::V2(20);
    let n = m.go();

    let m = Mine2::V2(20);
    let n = m.go();

    let m = Mine3::V2(20);
    let n = m.go2();
}
