#![allow(dead_code)]
#![allow(unused)]
extern crate penum;
use penum::penum;

// #[penum]
// pub trait TraitBlank {}

// #[penum]
// pub trait TraitWithGenericT<T> {}

// #[penum]
// pub trait TraitWithAssocType {
//     type Type;
// }

// #[penum]
// pub trait TraitWithAssocMethodVoid {
//     fn method(&self);
// }

// #[penum]
// pub trait TraitWithAssocMethodSelf {
//     fn method(&self) -> Self;
// }

// #[penum]
// pub trait TraitWithAssocMethodString {
//     fn method(&self) -> String;
// }

// #[penum]
// pub trait TraitWithGenericTAssocMethodT<T> {
//     fn method(&self) -> T;
// }

// // -------------------------------

// #[penum]
// pub trait TraitAssocReturn {
//     type Return;

//     fn method(&self) -> &Self::Return;
// }

// #[penum]
// pub trait TraitComposed<T>: TraitAssocReturn<Return = T> {
//     fn input(&self, input: T) -> Self::Return;
// }

// impl TraitAssocReturn for String {
//     type Return = Self;

//     fn method(&self) -> &Self::Return {
//         self
//     }
// }

// impl<T> TraitComposed<T> for String
// where
//     Self: TraitAssocReturn<Return = T>,
// {
//     fn input(&self, input: T) -> Self::Return {
//         input
//     }
// }

// #[penum( impl TraitComposed<String> for String )]
// #[penum( impl ^AsRef<str> for String )]
#[penum( impl AsRef<str> for String )]
enum Abc {
    V0(String),
    V1(i32, String),
    V2,
}

fn main() {
    let s = Abc::V0("hello".to_string());

    let s = s.as_ref();
}
