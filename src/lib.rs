use proc_macro::TokenStream;
use syn::{
    parse_macro_input, 
    spanned::Spanned,
    Data, DeriveInput, Error,
};

mod attribute;

use attribute::{VariantPattern, State};


#[proc_macro_attribute]
pub fn shape(attr: TokenStream, input: TokenStream) -> TokenStream {
    let derived_input = parse_macro_input!(input as DeriveInput);

    let Data::Enum(enum_definition) = &derived_input.data else {
        return Error::new(derived_input.ident.span(), "Expected an enum.").to_compile_error().into();
    };

    if enum_definition.variants.is_empty() {
        return Error::new(
            enum_definition.variants.span(),
            "Expected to find at least one variant.",
        )
        .to_compile_error()
        .into();
    }


    let mut state = State::new(parse_macro_input!(attr as VariantPattern), derived_input.clone());

    // Loop through enum definition and match each variant with each shape pattern.
    // for each variant => pattern.find(variant)
    for variant in enum_definition.variants.iter() {
        state.matcher(variant);
    }

    state.collect_tokens()
}
