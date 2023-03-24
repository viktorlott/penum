use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_str, ExprMacro, Type};

// We could use Visit pattern here, but it was easier to do it like this.
pub fn handle_default_ret_type(mut ty: &Type) -> Option<TokenStream> {
    let mut ctx = TokenStream::new();
    let mut error = false;

    loop {
        match ty {
            // Referenced return types:
            //
            // - &T where T implements Default doesn't
            //   really matter because it's not possible to
            //   return `&Default::default()`, even if `T`
            //   is a Copy type. `&0` would work, but
            //   `&Default::default()` or `&i32::default()`
            //   would not.`
            //
            // - &Option<T> could automatically be defaulted
            //   to `&None`.
            //
            // - &Result<i32, Option<T>> could also be
            //   defaulted to &Err(None)
            Type::Reference(ty_ref) => {
                if ty_ref.mutability.is_some() {
                    error = true;
                    break;
                }
                ctx.extend(quote::quote!(&));
                ty = &*ty_ref.elem;
            }

            // Owned return types without any references:
            //
            // - Types that can be proven implements Default
            //   could be returned with `Default::default()`
            //
            // - Option<T> could automatically be defaulted
            //   to `None`.
            //
            // - Result<T, U> needs to recursively check `U`
            //   to find a defaultable type. If we could
            //   prove that `U` implements Default, then we
            //   could just `Err(Default::default())`.
            Type::Path(path) => {
                if let Some(path_seg) = path.path.segments.last() {
                    if path_seg.ident.to_string().eq("Option") {
                        ctx.extend(quote::quote!(None));
                        break;
                    }
                };

                error = true;
                break;
            }

            Type::Tuple(tuple) => {
                let len = tuple.elems.len();

                if len == 0 {
                    error = true;
                    break;
                }

                let mut group = TokenStream::new();

                for (i, ty) in tuple.elems.iter().enumerate() {
                    if let Some(tokens) = handle_default_ret_type(ty) {
                        group.extend(tokens);
                    } else {
                        error = true;
                        break;
                    }
                    if i != len - 1 {
                        group.extend(quote::quote!(,));
                    }
                }

                ctx.extend(quote::quote!((#group)));

                break;
            }
            // Some `Type`s can't even be considered as
            // valid return types.
            _ => break,
        }
    }

    if error {
        None
    } else {
        Some(ctx)
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
