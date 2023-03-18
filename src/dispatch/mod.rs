use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
    str::FromStr,
};

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse_str, Arm, Field, Fields, ItemTrait, Signature, TraitItem, TraitItemMethod};

use crate::{factory::TraitBound, penum::Stringify};

#[derive(Default)]
pub struct DispatchMap(pub BTreeMap<Schematic, BTreeSet<Dispatchalor>>);

#[repr(transparent)]
#[derive(Clone)]
pub struct Schematic(pub ItemTrait);

#[derive(Clone)]
pub struct Blueprint<'bound> {
    pub schematic: Schematic,
    pub bound: &'bound TraitBound,
    pub arms_map: BTreeMap<String, Vec<Arm>>,
}

pub struct FieldInformation<'info> {
    enum_ident: &'info Ident,
    variant_ident: &'info Ident,
    position: &'info Position,
    len: usize,
}

impl<'bound> Blueprint<'bound> {
    pub fn from(bound: &'bound TraitBound) -> Self {
        let schematic = Std::from(bound.get_ident()).into();
        Self {
            schematic,
            bound,
            arms_map: Default::default(),
        }
    }
}

impl<'info> FieldInformation<'info> {
    pub fn new(
        enum_ident: &'info Ident,
        variant_ident: &'info Ident,
        position: &'info Position,
        len: usize,
    ) -> Self {
        Self {
            enum_ident,
            variant_ident,
            position,
            len,
        }
    }
}

impl<'bound> Blueprint<'bound> {
    pub fn attatch(&mut self, field_info: &FieldInformation) {
        let FieldInformation {
            enum_ident,
            variant_ident,
            position,
            len,
        } = field_info;

        let mut arms: BTreeMap<String, Vec<Arm>> = Default::default();

        for item in self.schematic.items.iter() {
            let TraitItem::Method(method) = item else {
                continue
            };

            let TraitItemMethod {
                sig: Signature { ident, inputs, .. },
                ..
            } = method;

            let inpts = inputs
                .iter()
                .filter_map(|arg| match arg {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(t) => Some(t.pat.get_string()),
                })
                .collect::<Vec<_>>()
                .join(", ");

            let formatted_position = position.format_fields(*len);
            let caller = position.get_caller();
            let formatted_arm = format!(
                "{enum_ident}::{variant_ident}{formatted_position} => {caller}.{ident}({inpts})"
            );
            let parsed_arm = parse_str::<Arm>(&formatted_arm).expect("Should have arm");
            
            let method_name = ident.to_string();

            if let Some(arm_vec) = arms.get_mut(&method_name) {
                arm_vec.push(parsed_arm)
            } else {
                arms.insert(method_name, vec![parsed_arm]);
            }
        }

        arms.into_iter().for_each(|(method_name, mut am)| {
            if let Some(arm_vec) = self.arms_map.get_mut(&method_name) {
                arm_vec.append(&mut am);
            } else {
                self.arms_map.insert(method_name, am);
            }
        })
    }
}

pub type BlueprintMap<'bound> = BTreeMap<String, Vec<Blueprint<'bound>>>;

/// For each <Dispatchable> -> <{ position, ident, fields }>
/// Used for dispatching
#[derive(Debug)]
pub enum Position {
    /// The index of the field being dispatched
    Index(usize),

    /// The key of the field being dispatched
    Key(String),
}

impl Position {
    pub fn from(field: &Field, fallback: usize) -> Self {
        field
            .ident
            .as_ref()
            .map(|ident| Position::Key(ident.to_string()))
            .unwrap_or(Position::Index(fallback))
    }

    pub fn get_caller(&self) -> String {
        if let Position::Key(k) = self {
            k.clone()
        } else {
            "val".to_owned()
        }
    }
}
/// This one is important. Use fields and position to create a pattern.
/// e.g. ident + position + fields + "bound signature" = `Ident::(_, X, ..) => X.method_call(<args if any>)`
pub struct Dispatchalor {
    /// The name of the variant
    pub ident: Ident,

    /// Used for dispatching
    pub position: Position,

    /// Wowawiwa
    pub fields: Fields,
}

#[derive(Debug)]
pub enum Std {
    Any,
    Borrow,
    BorrowMut,
    Eq,
    AsMut,
    AsRef,
    From,
    Into,
    TryFrom,
    TryInto,
    Default,
    Binary,
    Debug,
    Display,
    LowerExp,
    LowerHex,
    Octal,
    Pointer,
    UpperExp,
    UpperHex,
    Future,
    IntoFuture,
    FromIterator,
    FusedIterator,
    IntoIterator,
    Product,
    Sum,
    Copy,
    Sized,
    ToSocketAddrs,
    Add,
    AddAssign,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
    Deref,
    DerefMut,
    Div,
    DivAssign,
    Drop,
    Fn,
    FnMut,
    FnOnce,
    Index,
    IndexMut,
    Mul,
    MulAssign,
    MultiMethod,
    Neg,
    Not,
    Rem,
    RemAssign,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
    Sub,
    SubAssign,
    Termination,
    SliceIndex,
    FromStr,
    ToString,
}

impl Position {
    pub fn format_fields(&self, arity: usize) -> String {
        match self {
            Position::Index(index) => {
                let mut v = vec![];
                for _ in 0..*index {
                    v.push("_");
                }
                v.push("val");

                if arity > index + 1 {
                    v.push("..")
                }

                format!("({})", v.join(", "))
            }
            Position::Key(k) => format!(" {{ {}{} }}", k, if arity > 1 { ", .." } else { "" }),
        }
    }
}

impl From<Std> for Schematic {
    fn from(value: Std) -> Self {
        Schematic(
            match value {
                Std::Any => parse_str(include_str!("./std/Any.rs")),
                Std::Borrow => parse_str(include_str!("./std/Borrow.rs")),
                Std::BorrowMut => parse_str(include_str!("./std/BorrowMut.rs")),
                Std::Eq => parse_str(include_str!("./std/Eq.rs")),
                Std::AsMut => parse_str(include_str!("./std/AsMut.rs")),
                Std::AsRef => parse_str(include_str!("./std/AsRef.rs")),
                Std::From => parse_str(include_str!("./std/From.rs")),
                Std::Into => parse_str(include_str!("./std/Into.rs")),
                Std::TryFrom => parse_str(include_str!("./std/TryFrom.rs")),
                Std::TryInto => parse_str(include_str!("./std/TryInto.rs")),
                Std::Default => parse_str(include_str!("./std/Default.rs")),
                Std::Binary => parse_str(include_str!("./std/Binary.rs")),
                Std::Debug => parse_str(include_str!("./std/Debug.rs")),
                Std::Display => parse_str(include_str!("./std/Display.rs")),
                Std::LowerExp => parse_str(include_str!("./std/LowerExp.rs")),
                Std::LowerHex => parse_str(include_str!("./std/LowerHex.rs")),
                Std::Octal => parse_str(include_str!("./std/Octal.rs")),
                Std::Pointer => parse_str(include_str!("./std/Pointer.rs")),
                Std::UpperExp => parse_str(include_str!("./std/UpperExp.rs")),
                Std::UpperHex => parse_str(include_str!("./std/UpperHex.rs")),
                Std::Future => parse_str(include_str!("./std/Future.rs")),
                Std::IntoFuture => parse_str(include_str!("./std/IntoFuture.rs")),
                Std::FromIterator => parse_str(include_str!("./std/FromIterator.rs")),
                Std::FusedIterator => parse_str(include_str!("./std/FusedIterator.rs")),
                Std::IntoIterator => parse_str(include_str!("./std/IntoIterator.rs")),
                Std::Product => parse_str(include_str!("./std/Product.rs")),
                Std::Sum => parse_str(include_str!("./std/Sum.rs")),
                Std::Copy => parse_str(include_str!("./std/Copy.rs")),
                Std::Sized => parse_str(include_str!("./std/Sized.rs")),
                Std::ToSocketAddrs => parse_str(include_str!("./std/ToSocketAddrs.rs")),
                Std::Add => parse_str(include_str!("./std/Add.rs")),
                Std::AddAssign => parse_str(include_str!("./std/AddAssign.rs")),
                Std::BitAnd => parse_str(include_str!("./std/BitAnd.rs")),
                Std::BitAndAssign => parse_str(include_str!("./std/BitAndAssign.rs")),
                Std::BitOr => parse_str(include_str!("./std/BitOr.rs")),
                Std::BitOrAssign => parse_str(include_str!("./std/BitOrAssign.rs")),
                Std::BitXor => parse_str(include_str!("./std/BitXor.rs")),
                Std::BitXorAssign => parse_str(include_str!("./std/BitXorAssign.rs")),
                Std::Deref => parse_str(include_str!("./std/Deref.rs")),
                Std::DerefMut => parse_str(include_str!("./std/DerefMut.rs")),
                Std::Div => parse_str(include_str!("./std/Div.rs")),
                Std::DivAssign => parse_str(include_str!("./std/DivAssign.rs")),
                Std::Drop => parse_str(include_str!("./std/Drop.rs")),
                Std::Fn => parse_str(include_str!("./std/Fn.rs")),
                Std::FnMut => parse_str(include_str!("./std/FnMut.rs")),
                Std::FnOnce => parse_str(include_str!("./std/FnOnce.rs")),
                Std::Index => parse_str(include_str!("./std/Index.rs")),
                Std::IndexMut => parse_str(include_str!("./std/IndexMut.rs")),
                Std::Mul => parse_str(include_str!("./std/Mul.rs")),
                Std::MulAssign => parse_str(include_str!("./std/MulAssign.rs")),
                Std::MultiMethod => parse_str(include_str!("./std/MultiMethod.rs")),
                Std::Neg => parse_str(include_str!("./std/Neg.rs")),
                Std::Not => parse_str(include_str!("./std/Not.rs")),
                Std::Rem => parse_str(include_str!("./std/Rem.rs")),
                Std::RemAssign => parse_str(include_str!("./std/RemAssign.rs")),
                Std::Shl => parse_str(include_str!("./std/Shl.rs")),
                Std::ShlAssign => parse_str(include_str!("./std/ShlAssign.rs")),
                Std::Shr => parse_str(include_str!("./std/Shr.rs")),
                Std::ShrAssign => parse_str(include_str!("./std/ShrAssign.rs")),
                Std::Sub => parse_str(include_str!("./std/Sub.rs")),
                Std::SubAssign => parse_str(include_str!("./std/SubAssign.rs")),
                Std::Termination => parse_str(include_str!("./std/Termination.rs")),
                Std::SliceIndex => parse_str(include_str!("./std/SliceIndex.rs")),
                Std::FromStr => parse_str(include_str!("./std/FromStr.rs")),
                Std::ToString => parse_str(include_str!("./std/ToString.rs")),
            }
            .expect("Std trait file should exist"),
        )
    }
}

impl FromStr for Std {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "Any" => Self::Any,
            "Borrow" => Self::Borrow,
            "BorrowMut" => Self::BorrowMut,
            "Eq" => Self::Eq,
            "AsMut" => Self::AsMut,
            "AsRef" => Self::AsRef,
            "From" => Self::From,
            "Into" => Self::Into,
            "TryFrom" => Self::TryFrom,
            "TryInto" => Self::TryInto,
            "Default" => Self::Default,
            "Binary" => Self::Binary,
            "Debug" => Self::Debug,
            "Display" => Self::Display,
            "LowerExp" => Self::LowerExp,
            "LowerHex" => Self::LowerHex,
            "Octal" => Self::Octal,
            "Pointer" => Self::Pointer,
            "UpperExp" => Self::UpperExp,
            "UpperHex" => Self::UpperHex,
            "Future" => Self::Future,
            "IntoFuture" => Self::IntoFuture,
            "FromIterator" => Self::FromIterator,
            "FusedIterator" => Self::FusedIterator,
            "IntoIterator" => Self::IntoIterator,
            "Product" => Self::Product,
            "Sum" => Self::Sum,
            "Copy" => Self::Copy,
            "Sized" => Self::Sized,
            "ToSocketAddrs" => Self::ToSocketAddrs,
            "Add" => Self::Add,
            "AddAssign" => Self::AddAssign,
            "BitAnd" => Self::BitAnd,
            "BitAndAssign" => Self::BitAndAssign,
            "BitOr" => Self::BitOr,
            "BitOrAssign" => Self::BitOrAssign,
            "BitXor" => Self::BitXor,
            "BitXorAssign" => Self::BitXorAssign,
            "Deref" => Self::Deref,
            "DerefMut" => Self::DerefMut,
            "Div" => Self::Div,
            "DivAssign" => Self::DivAssign,
            "Drop" => Self::Drop,
            "Fn" => Self::Fn,
            "FnMut" => Self::FnMut,
            "FnOnce" => Self::FnOnce,
            "Index" => Self::Index,
            "IndexMut" => Self::IndexMut,
            "Mul" => Self::Mul,
            "MulAssign" => Self::MulAssign,
            "MultiMethod" => Self::MultiMethod,
            "Neg" => Self::Neg,
            "Not" => Self::Not,
            "Rem" => Self::Rem,
            "RemAssign" => Self::RemAssign,
            "Shl" => Self::Shl,
            "ShlAssign" => Self::ShlAssign,
            "Shr" => Self::Shr,
            "ShrAssign" => Self::ShrAssign,
            "Sub" => Self::Sub,
            "SubAssign" => Self::SubAssign,
            "Termination" => Self::Termination,
            "SliceIndex" => Self::SliceIndex,
            "FromStr" => Self::FromStr,
            "ToString" => Self::ToString,
            _ => panic!("no match found, {}", value),
        })
    }
}
impl From<&Ident> for Std {
    fn from(value: &Ident) -> Self {
        match value.to_string().as_str() {
            "Any" => Self::Any,
            "Borrow" => Self::Borrow,
            "BorrowMut" => Self::BorrowMut,
            "Eq" => Self::Eq,
            "AsMut" => Self::AsMut,
            "AsRef" => Self::AsRef,
            "From" => Self::From,
            "Into" => Self::Into,
            "TryFrom" => Self::TryFrom,
            "TryInto" => Self::TryInto,
            "Default" => Self::Default,
            "Binary" => Self::Binary,
            "Debug" => Self::Debug,
            "Display" => Self::Display,
            "LowerExp" => Self::LowerExp,
            "LowerHex" => Self::LowerHex,
            "Octal" => Self::Octal,
            "Pointer" => Self::Pointer,
            "UpperExp" => Self::UpperExp,
            "UpperHex" => Self::UpperHex,
            "Future" => Self::Future,
            "IntoFuture" => Self::IntoFuture,
            "FromIterator" => Self::FromIterator,
            "FusedIterator" => Self::FusedIterator,
            "IntoIterator" => Self::IntoIterator,
            "Product" => Self::Product,
            "Sum" => Self::Sum,
            "Copy" => Self::Copy,
            "Sized" => Self::Sized,
            "ToSocketAddrs" => Self::ToSocketAddrs,
            "Add" => Self::Add,
            "AddAssign" => Self::AddAssign,
            "BitAnd" => Self::BitAnd,
            "BitAndAssign" => Self::BitAndAssign,
            "BitOr" => Self::BitOr,
            "BitOrAssign" => Self::BitOrAssign,
            "BitXor" => Self::BitXor,
            "BitXorAssign" => Self::BitXorAssign,
            "Deref" => Self::Deref,
            "DerefMut" => Self::DerefMut,
            "Div" => Self::Div,
            "DivAssign" => Self::DivAssign,
            "Drop" => Self::Drop,
            "Fn" => Self::Fn,
            "FnMut" => Self::FnMut,
            "FnOnce" => Self::FnOnce,
            "Index" => Self::Index,
            "IndexMut" => Self::IndexMut,
            "Mul" => Self::Mul,
            "MulAssign" => Self::MulAssign,
            "MultiMethod" => Self::MultiMethod,
            "Neg" => Self::Neg,
            "Not" => Self::Not,
            "Rem" => Self::Rem,
            "RemAssign" => Self::RemAssign,
            "Shl" => Self::Shl,
            "ShlAssign" => Self::ShlAssign,
            "Shr" => Self::Shr,
            "ShrAssign" => Self::ShrAssign,
            "Sub" => Self::Sub,
            "SubAssign" => Self::SubAssign,
            "Termination" => Self::Termination,
            "SliceIndex" => Self::SliceIndex,
            "FromStr" => Self::FromStr,
            "ToString" => Self::ToString,
            _ => panic!("no match found, {}", value),
        }
    }
}

impl Deref for Schematic {
    type Target = ItemTrait;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
