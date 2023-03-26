use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_str;
use syn::spanned::Spanned;
use syn::ExprMacro;
use syn::GenericArgument;
use syn::PathArguments;
use syn::Type;

/// This is kind of a redundant solution..
fn static_return<T: ToTokens + Spanned>(ty: &T) -> TokenStream {
    quote::quote_spanned!(ty.span()=>
        {
            use std::cell::UnsafeCell;
            use std::sync::Once;
            struct Static<T: Default>(UnsafeCell<Option<T>>, Once);
            unsafe impl<T: Default> Sync for Static<T> {}
            impl<T: Default> Static<T> {
                pub const fn new() -> Self {
                    Self(UnsafeCell::new(None), Once::new())
                }
                fn get(&self) -> &'static T {
                    // SAFETY: Firstly, this static isn't available to
                    //         the user directly because it's scoped and
                    //         is only generated through macro expansion
                    //         at return positions. Secondly, the type
                    //         we are returning is an immutable static
                    //         reference which means that it's not
                    //         possible to mutate it directly, unless
                    //         the user has an interior mutability type.
                    //         It's up to the user to make sure that T
                    //         doesn't contain any unsound datastructure
                    //         that would break this implementation.
                    //         Note that T needs to implement Default.
                    //         Thirdly, this type is meant to return
                    //         non-const reference types, so to make
                    //         this work we have to do a lazy
                    //         initialization, which means that it needs
                    //         to be thread safe. This is done through a
                    //         sync primitive that ensures us that it
                    //         can only be initialized once, and that
                    //         other threads are blocked from reading it
                    //         if it's being written to.
                    self.1.call_once(|| unsafe { *self.0.get() = Some(T::default()) });
                    unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
                }
            }
            static RETURN: Static<#ty> = Static::new();
            RETURN.get()
        }
    )
}

// We could use Visitor pattern here, but it was easier to do it like
// this. TODO: Refactor please
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
            Type::Path(path) => {
                if let Some(path_seg) = path.path.segments.last() {
                    match path_seg.ident.to_string().as_str() {
                        "Result" => {
                            if let PathArguments::AngleBracketed(ref abga) = path_seg.arguments {
                                if let Some(GenericArgument::Type(err_ty)) = abga.args.last() {
                                    // FIXME: Search `err_ty` and check
                                    // if it implements Default.
                                    if let Some(toks) = handle_default_ret_type(err_ty) {
                                        tokens.extend(quote::quote!(Err(#toks)));
                                        return Some(tokens);
                                    } else {
                                        return None;
                                    }
                                }
                            }

                            return None;
                        }
                        "Option" => {
                            tokens.extend(quote::quote!(None));
                            return Some(tokens);
                        }
                        "String" => {
                            if is_ref {
                                return Some(static_return(&path_seg.ident));
                            } else {
                                return Some(quote::quote!("".to_string()));
                            }
                        }
                        "str" => return Some(quote::quote!("")),
                        "char" => return Some(quote::quote!('\x00')),
                        "bool" => {
                            tokens.extend(quote::quote!(false));
                            return Some(tokens);
                        }
                        "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64"
                        | "i128" | "usize" | "isize" => {
                            tokens.extend(quote::quote!(0));
                            return Some(tokens);
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
    use crate::dispatch::ret::handle_default_ret_type;
    use syn::{parse_quote, Type};

    #[test]
    fn owned_result() {
        let ty: Type = parse_quote!(Result<T, String>);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("Err (\"\" . to_string ())", result.as_str())
    }

    #[test]
    fn ref_result() {
        let ty: Type = parse_quote!(&Result<T, String>);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("& Err (\"\" . to_string ())", result.as_str())
    }

    #[test]
    fn owned_option() {
        let ty: Type = parse_quote!(Option<T>);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("None", result.as_str())
    }

    #[test]
    fn ref_option() {
        let ty: Type = parse_quote!(&Option<T>);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("& None", result.as_str())
    }

    #[test]
    fn ref_tuple_ref_option_and_option() {
        let ty: Type = parse_quote!(&(&Option<T>, Option<T>));
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("& (& None , None)", result.as_str())
    }

    #[test]
    fn ref_char() {
        let ty: Type = parse_quote!(&char);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("'\\x00'", result.as_str())
    }

    #[test]
    fn owned_bool() {
        let ty: Type = parse_quote!(bool);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("false", result.as_str())
    }

    #[test]
    fn ref_bool() {
        let ty: Type = parse_quote!(&bool);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("& false", result.as_str())
    }

    #[test]
    fn owned_string() {
        let ty: Type = parse_quote!(String);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("\"\" . to_string ()", result.as_str())
    }

    #[test]
    fn ref_string() {
        let ty: Type = parse_quote!(&String);
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!("{ use std :: cell :: UnsafeCell ; use std :: sync :: Once ; struct Static < T : Default > (UnsafeCell < Option < T >> , Once) ; unsafe impl < T : Default > Sync for Static < T > { } impl < T : Default > Static < T > { pub const fn new () -> Self { Self (UnsafeCell :: new (None) , Once :: new ()) } fn get (& self) -> & 'static T { self . 1 . call_once (|| unsafe { * self . 0 . get () = Some (T :: default ()) }) ; unsafe { (* self . 0 . get ()) . as_ref () . unwrap_unchecked () } } } static RETURN : Static < String > = Static :: new () ; RETURN . get () }", result.as_str())
    }

    #[test]
    fn tuple_numbers() {
        let ty: Type =
            parse_quote!((u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize));
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!(
            "(0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0)",
            result.as_str()
        )
    }

    #[test]
    fn ref_tuple_numbers() {
        let ty: Type =
            parse_quote!(&(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize));
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!(
            "& (0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0)",
            result.as_str()
        )
    }

    #[test]
    fn ref_tuple_ref_numbers() {
        let ty: Type = parse_quote!(&(
            &u8, &u16, &u32, &u64, &u128, &usize, &i8, &i16, &i32, &i64, &i128, &isize
        ));
        let result = handle_default_ret_type(&ty).expect("to parse").to_string();

        assert_eq!(
            "& (& 0 , & 0 , & 0 , & 0 , & 0 , & 0 , & 0 , & 0 , & 0 , & 0 , & 0 , & 0)",
            result.as_str()
        )
    }
}
