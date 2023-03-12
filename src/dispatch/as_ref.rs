use syn::{parse_quote, ItemTrait};

/// this is just for testing

pub fn construct() -> ItemTrait {
    parse_quote!(
        pub trait AsRef<T: ?Sized> {
            fn as_ref(&self) -> &T;
        }
    )
}
