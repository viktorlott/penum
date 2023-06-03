#![allow(unused)]

enum Stage {
    Before,
    After,
}

#[penum::to_string]
enum Enum {
    // Regular str literal
    Variant0 = "Return on match",

    // Format str literal
    Variant1(i32) = "Return {f0} on match",

    // Macro calls
    Variant2(String) = format!("My string {f0}"),
    Variant3(i32, u32) = stringify!(f0, f1).to_string(),

    // Field highlighing
    Variant4 {
        name: String,
        age: u8,
    } = "My name is {name} and I'm {age} old",

    // Expression blocks
    Variant5(Vec<String>, String) = {
        let result = f0.iter().find(|s| *s == f1);
        result.unwrap().to_string()
    },

    // Call expressions
    Variant6 {
        name: String,
    } = name.to_string(),

    // Match expressions
    Variant8 {
        state: Stage,
    } = match state {
        Stage::Before => "Before".to_string(),
        Stage::After => "After".to_string(),
    },
}

#[penum::to_string]
enum EnumVariants {
    Variant0 = "Return on match",
    Variant1(i32) = "Return {f0} on match",
    Variant2(i32, u32) = stringify!(f0, f1).to_string(),
    Variant3 { name: String } = format!("My string {name}"),
    Variant4 { age: u32 } = age.to_string(),
}

fn main() {
    let enum_variants = Enum::Variant0;
    println!("{}", enum_variants.to_string());
}
