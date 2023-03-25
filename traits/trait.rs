use std::str::FromStr;
use std::ops::Deref;

use proc_macro2::Ident;
use syn::parse_quote;
use syn::ItemTrait;

#[derive(Debug)]
pub enum StandardTrait {
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

#[repr(transparent)]
#[derive(Clone, Hash, Debug)]
pub struct TraitSchematic(pub ItemTrait);

impl From<StandardTrait> for TraitSchematic {
    fn from(value: StandardTrait) -> Self {
        TraitSchematic(
            match value {
                StandardTrait::Any => parse_quote!(
pub trait Any: 'static {    fn type_id(&self) -> TypeId;}),StandardTrait::Borrow => parse_quote!(
pub trait Borrow<Borrowed>where    Borrowed: ?Sized,{    fn borrow(&self) -> &Borrowed;}),StandardTrait::BorrowMut => parse_quote!(
pub trait BorrowMut<Borrowed>: Borrow<Borrowed>where    Borrowed: ?Sized,{    fn borrow_mut(&mut self) -> &mut Borrowed;}),StandardTrait::Eq => parse_quote!(
pub trait Eq: PartialEq<Self> {}),StandardTrait::AsMut => parse_quote!(
pub trait AsMut<T>where    T: ?Sized,{    fn as_mut(&mut self) -> &mut T;}),StandardTrait::AsRef => parse_quote!(
pub trait AsRef<T>where    T: ?Sized,{    fn as_ref(&self) -> &T;}),StandardTrait::From => parse_quote!(
pub trait From<T>: Sized {    fn from(value: T) -> Self;}),StandardTrait::Into => parse_quote!(
pub trait Into<T>: Sized {    fn into(self) -> T;}),StandardTrait::TryFrom => parse_quote!(
pub trait TryFrom<T>: Sized {    type Error;    fn try_from(value: T) -> Result<Self, Self::Error>;}),StandardTrait::TryInto => parse_quote!(
pub trait TryInto<T>: Sized {    type Error;    fn try_into(self) -> Result<T, Self::Error>;}),StandardTrait::Default => parse_quote!(
pub trait Default: Sized {    fn default() -> Self;}),StandardTrait::Binary => parse_quote!(
pub trait Binary {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::Debug => parse_quote!(
pub trait Debug {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::Display => parse_quote!(
pub trait Display {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::LowerExp => parse_quote!(
pub trait LowerExp {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::LowerHex => parse_quote!(
pub trait LowerHex {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::Octal => parse_quote!(
pub trait Octal {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::Pointer => parse_quote!(
pub trait Pointer {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::UpperExp => parse_quote!(
pub trait UpperExp {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::UpperHex => parse_quote!(
pub trait UpperHex {    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;}),StandardTrait::Future => parse_quote!(
pub trait Future {    type Output;    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;}),StandardTrait::IntoFuture => parse_quote!(
pub trait IntoFuture {    type Output;    type IntoFuture: Future<Output = Self::Output>;    fn into_future(self) -> Self::IntoFuture;}),StandardTrait::FromIterator => parse_quote!(
pub trait FromIterator<A>: Sized {    fn from_iter<T>(iter: T) -> Self    where        T: IntoIterator<Item = A>;}),StandardTrait::FusedIterator => parse_quote!(
pub trait FusedIterator: Iterator {}),StandardTrait::IntoIterator => parse_quote!(
pub trait IntoIterator {    type Item;    type IntoIter: Iterator<Item = Self::Item>;    fn into_iter(self) -> Self::IntoIter;}),StandardTrait::Product => parse_quote!(
pub trait Product<A = Self>: Sized {    fn product<I>(iter: I) -> Self    where        I: Iterator<Item = A>;}),StandardTrait::Sum => parse_quote!(
pub trait Sum<A = Self>: Sized {    fn sum<I>(iter: I) -> Self    where        I: Iterator<Item = A>;}),StandardTrait::Copy => parse_quote!(
pub trait Copy: Clone {}),StandardTrait::Sized => parse_quote!(
pub trait Sized {}),StandardTrait::ToSocketAddrs => parse_quote!(
pub trait ToSocketAddrs {    type Iter: Iterator<Item = SocketAddr>;    fn to_socket_addrs(&self) -> Result<Self::Iter>;}),StandardTrait::Add => parse_quote!(
pub trait Add<Rhs = Self> {    type Output;    fn add(self, rhs: Rhs) -> Self::Output;}),StandardTrait::AddAssign => parse_quote!(
pub trait AddAssign<Rhs = Self> {    fn add_assign(&mut self, rhs: Rhs);}),StandardTrait::BitAnd => parse_quote!(
pub trait BitAnd<Rhs = Self> {    type Output;    fn bitand(self, rhs: Rhs) -> Self::Output;}),StandardTrait::BitAndAssign => parse_quote!(
pub trait BitAndAssign<Rhs = Self> {    fn bitand_assign(&mut self, rhs: Rhs);}),StandardTrait::BitOr => parse_quote!(
pub trait BitOr<Rhs = Self> {    type Output;    fn bitor(self, rhs: Rhs) -> Self::Output;}),StandardTrait::BitOrAssign => parse_quote!(
pub trait BitOrAssign<Rhs = Self> {    fn bitor_assign(&mut self, rhs: Rhs);}),StandardTrait::BitXor => parse_quote!(
pub trait BitXor<Rhs = Self> {    type Output;    fn bitxor(self, rhs: Rhs) -> Self::Output;}),StandardTrait::BitXorAssign => parse_quote!(
pub trait BitXorAssign<Rhs = Self> {    fn bitxor_assign(&mut self, rhs: Rhs);}),StandardTrait::Deref => parse_quote!(
pub trait Deref {    type Target: ?Sized;    fn deref(&self) -> &Self::Target;}),StandardTrait::DerefMut => parse_quote!(
pub trait DerefMut: Deref {    fn deref_mut(&mut self) -> &mut Self::Target;}),StandardTrait::Div => parse_quote!(
pub trait Div<Rhs = Self> {    type Output;    fn div(self, rhs: Rhs) -> Self::Output;}),StandardTrait::DivAssign => parse_quote!(
pub trait DivAssign<Rhs = Self> {    fn div_assign(&mut self, rhs: Rhs);}),StandardTrait::Drop => parse_quote!(
pub trait Drop {    fn drop(&mut self);}),StandardTrait::Fn => parse_quote!(
pub trait Fn<Args>: FnMut<Args> {    extern "rust-call" fn call(&self, args: Args) -> Self::Output;}),StandardTrait::FnMut => parse_quote!(
pub trait FnMut<Args>: FnOnce<Args> {    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;}),StandardTrait::FnOnce => parse_quote!(
pub trait FnOnce<Args> {    type Output;    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;}),StandardTrait::Index => parse_quote!(
pub trait Index<Idx>where    Idx: ?Sized,{    type Output: ?Sized;    fn index(&self, index: Idx) -> &Self::Output;}),StandardTrait::IndexMut => parse_quote!(
pub trait IndexMut<Idx>: Index<Idx>where    Idx: ?Sized,{    fn index_mut(&mut self, index: Idx) -> &mut Self::Output;}),StandardTrait::Mul => parse_quote!(
pub trait Mul<Rhs = Self> {    type Output;    fn mul(self, rhs: Rhs) -> Self::Output;}),StandardTrait::MulAssign => parse_quote!(
pub trait MulAssign<Rhs = Self> {    fn mul_assign(&mut self, rhs: Rhs);}),StandardTrait::Neg => parse_quote!(
pub trait Neg {    type Output;    fn neg(self) -> Self::Output;}),StandardTrait::Not => parse_quote!(
pub trait Not {    type Output;    fn not(self) -> Self::Output;}),StandardTrait::Rem => parse_quote!(
pub trait Rem<Rhs = Self> {    type Output;    fn rem(self, rhs: Rhs) -> Self::Output;}),StandardTrait::RemAssign => parse_quote!(
pub trait RemAssign<Rhs = Self> {    fn rem_assign(&mut self, rhs: Rhs);}),StandardTrait::Shl => parse_quote!(
pub trait Shl<Rhs = Self> {    type Output;    fn shl(self, rhs: Rhs) -> Self::Output;}),StandardTrait::ShlAssign => parse_quote!(
pub trait ShlAssign<Rhs = Self> {    fn shl_assign(&mut self, rhs: Rhs);}),StandardTrait::Shr => parse_quote!(
pub trait Shr<Rhs = Self> {    type Output;    fn shr(self, rhs: Rhs) -> Self::Output;}),StandardTrait::ShrAssign => parse_quote!(
pub trait ShrAssign<Rhs = Self> {    fn shr_assign(&mut self, rhs: Rhs);}),StandardTrait::Sub => parse_quote!(
pub trait Sub<Rhs = Self> {    type Output;    fn sub(self, rhs: Rhs) -> Self::Output;}),StandardTrait::SubAssign => parse_quote!(
pub trait SubAssign<Rhs = Self> {    fn sub_assign(&mut self, rhs: Rhs);}),StandardTrait::Termination => parse_quote!(
pub trait Termination {    fn report(self) -> ExitCode;}),StandardTrait::SliceIndex => parse_quote!(
pub unsafe trait SliceIndex<T>: Sealedwhere    T: ?Sized,{    type Output: ?Sized;    fn get(self, slice: &T) -> Option<&Self::Output>;    fn get_mut(self, slice: &mut T) -> Option<&mut Self::Output>;    unsafe fn get_unchecked(self, slice: *const T) -> *const Self::Output;    unsafe fn get_unchecked_mut(self, slice: *mut T) -> *mut Self::Output;    fn index(self, slice: &T) -> &Self::Output;    fn index_mut(self, slice: &mut T) -> &mut Self::Output;}),StandardTrait::FromStr => parse_quote!(
pub trait FromStr: Sized {    type Err;    fn from_str(s: &str) -> Result<Self, Self::Err>;}),StandardTrait::ToString => parse_quote!(
pub trait ToString {    fn to_string(&self) -> String;})
            }
            .expect("Std trait file should exist"),
        )
    }
}

impl FromStr for StandardTrait {
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
            _ => panic!("no match found, StandardTrait", value),
        })
    }
}

impl From<&Ident> for StandardTrait {
    fn from(value: &Ident) -> Self {
        StandardTrait::from_str(value.to_string().as_str()).expect("Expect to find a match")
    }
}

impl Deref for TraitSchematic {
    type Target = ItemTrait;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
