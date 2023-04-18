#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_patterns)]
extern crate penum;
use penum::penum;

#[penum]
trait Cool {
    type Target;
    fn mine(&self, value: Self::Target) -> &i32;
}

impl Cool for i32 {
    type Target = String;
    fn mine(&self, value: String) -> &i32 {
        self
    }
}

#[penum( _ where i32: ^Cool )]
enum Mine5 {
    V1(i32),
    V2(i32),
    V3(i32, i32),
}

fn main() {
    let m = Mine5::V1(10);

    let s = m.mine("wewe".to_string());

    assert_eq!(&10, s);
}
