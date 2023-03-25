use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_str;
use syn::spanned::Spanned;
use syn::ExprMacro;
use syn::Type;

// This is kind of a redundant solution..
fn static_return<T: ToTokens + Spanned>(ty: &T) -> TokenStream {
    quote::quote_spanned!(ty.span()=>
        {
            use std::cell::UnsafeCell;
            struct Static<T: Default>(UnsafeCell<Option<T>>);
            unsafe impl<T: Default> Sync for Static<T> {}
            impl<T: Default> Static<T> {
                pub const fn new() -> Self {
                    Self(UnsafeCell::new(None))
                }
                fn get(&self) -> &'static T {
                    unsafe { &mut *self.0.get() }.get_or_insert_with(|| T::default())
                }
            }
            static RETURN: Static<#ty> = Static::new();
            RETURN.get()
        }
    )
}

// We could use Visit pattern here, but it was easier to do it like
// this.
pub fn handle_default_ret_type(mut ty: &Type) -> Option<TokenStream> {
    let mut tokens = TokenStream::new();
    let mut is_ref = false;
    loop {
        match ty {
            // Referenced return types:
            //
            // - &T where T implements Default doesn't really matter
            //   because it's not possible to return
            //   `&Default::default()`, even if `T` is a Copy type. `&0`
            //   would work, but `&Default::default()` or
            //   `&i32::default()` would not.`
            //
            // - &Option<T> could automatically be defaulted to `&None`.
            //
            // - &Result<i32, Option<T>> could also be defaulted to
            //   &Err(None)
            Type::Reference(ty_ref) => {
                if ty_ref.mutability.is_some() {
                    return None;
                }

                is_ref = true;

                tokens.extend(quote::quote!(&));
                ty = &*ty_ref.elem;
            }

            // Owned return types without any references:
            //
            // - Types that can be proven implements Default could be
            //   returned with `Default::default()`
            //
            // - Option<T> could automatically be defaulted to `None`.
            //
            // - Result<T, U> needs to recursively check `U` to find a
            //   defaultable type. If we could prove that `U` implements
            //   Default, then we could just `Err(Default::default())`.
            //
            //   | "bool" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16"
            //   | "i32" | "i64"
            Type::Path(path) => {
                if let Some(path_seg) = path.path.segments.last() {
                    match path_seg.ident.to_string().as_str() {
                        "Option" => {
                            tokens.extend(quote::quote!(None));
                            return Some(tokens);
                        }
                        "str" => return Some(quote::quote!("")),
                        "String" => {
                            if is_ref {
                                return Some(static_return(&path_seg.ident));
                            } else {
                                return Some(quote::quote!("".to_string()));
                            }
                        }
                        // "Result" => {}
                        _ => return None,
                    }
                };

                return None;
            }

            Type::Tuple(tuple) => {
                let len = tuple.elems.len();

                if len == 0 {
                    return None;
                }

                let mut group = TokenStream::new();

                for (i, ty) in tuple.elems.iter().enumerate() {
                    if let Some(tokens) = handle_default_ret_type(ty) {
                        group.extend(tokens);
                    } else {
                        return None;
                    }
                    if i != len - 1 {
                        group.extend(quote::quote!(,));
                    }
                }

                tokens.extend(quote::quote!((#group)));

                return Some(tokens);
            }
            // Some `Type`s can't even be considered as valid return
            // types.
            _ => return None,
        }
    }
}

pub fn return_panic() -> TokenStream {
    // Might be better ways of parsing macros.
    parse_str::<ExprMacro>("panic!(\"Missing arm\")")
        .unwrap()
        .to_token_stream()
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Type};

    use crate::dispatch::ret::handle_default_ret_type;
    #[test]
    fn token_test() {
        let ref_option: Type = parse_quote!(&Option<String>);
        let _result = handle_default_ret_type(&ref_option);

        let ref_option_tuple: Type = parse_quote!(&(&Option<i32>, Option<i32>));
        let _result = handle_default_ret_type(&ref_option_tuple);
    }
}
