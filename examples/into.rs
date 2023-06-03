#![allow(unused)]

#[penum::into(String)]
enum EnumVariants {
    Variant0 = "Return on match".into(),
    Variant1(i32) = format!("Return {f0} on match"),
    Variant2(i32, u32) = stringify!(f0, f1).to_string(),
    Variant3 { name: String } = format!("My string {name}"),
    Variant4 { age: u32 } = age.to_string(),
}

fn main() {
    let enum_variants = EnumVariants::Variant0;
    let string: String = enum_variants.into();
    println!("{string}");
}
