#![allow(unused)]

enum Stage {
    Before,
    After,
}

#[penum::fmt]
enum EnumVariants {
    Variant0 = "Return on match",
    Variant1(i32) = "Return {f0} on match",
    Variant2(i32, u32) = stringify!(f0, f1).to_string().fmt(f),
    Variant3 { name: String } = format!("My string {name}").fmt(f),
    Variant4 { age: u32 } = age.to_string().fmt(f),
}

fn main() {
    let enum_variants = EnumVariants::Variant0;
    println!("{}", enum_variants);
}
