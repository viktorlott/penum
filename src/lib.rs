use proc_macro::TokenStream;
use syn::{parse_macro_input};

use factory::{Penum, Subject, Pattern};

mod factory;
mod utils;
mod error;


#[proc_macro_attribute]
pub fn shape(attr: TokenStream, input: TokenStream) -> TokenStream {
    let pattern = parse_macro_input!(attr as Pattern);
    let input =  parse_macro_input!(input as Subject);

    // Loop through enum definition and match each variant with each shape pattern.
    // for each variant => pattern.find(variant)
    Penum::from(pattern, input).assemble().unwrap_or_error()
}
