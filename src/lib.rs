use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attribute;
mod utils;
use attribute::{VariantPattern, EnumShape};


#[proc_macro_attribute]
pub fn shape(attr: TokenStream, input: TokenStream) -> TokenStream {
    let shape = parse_macro_input!(attr as VariantPattern);
    let input =  parse_macro_input!(input as DeriveInput);

    // Loop through enum definition and match each variant with each shape pattern.
    // for each variant => pattern.find(variant)
    EnumShape::new(shape, input).matcher().collect_tokens()
}
