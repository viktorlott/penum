use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, ToTokens};
use syn::{
    parse_quote, parse_str,
    visit::{visit_type_path, Visit},
    ExprMacro, Type, TypePath, TypeReference, token,
};




// match &**ty {
//     // Owned return types without any references:
//     //
//     // - Types that can be proven implements Default
//     //   could be returned with `Default::default()`
//     //
//     // - Option<T> could automatically be defaulted
//     //   to `None`.
//     //
//     // - Result<T, U> needs to recursively check `U`
//     //   to find a defaultable type. If we could
//     //   prove that `U` implements Default, then we
//     //   could just `Err(Default::default())`.
//     Type::Path(_) => return_panic(),

//     // Referenced return types:
//     //
//     // - &T where T implements Default doesn't
//     //   really matter because it's not possible to
//     //   return `&Default::default()`, even if `T`
//     //   is a Copy type. `&0` would work, but
//     //   `&Default::default()` or `&i32::default()`
//     //   would not.`
//     //
//     // - &Option<T> could automatically be defaulted
//     //   to `&None`.
//     //
//     // - &Result<i32, Option<T>> could also be
//     //   defaulted to &Err(None)
//     Type::Reference(_) => return_panic(),

//     // Add support for these later.
//     Type::Paren(_) => return_panic(),
//     Type::Tuple(_) => return_panic(),
//     Type::ImplTrait(_) => return_panic(),
//     Type::Array(_) => return_panic(),
//     Type::BareFn(_) => return_panic(),

//     // Ouff, dunno
//     Type::Group(_) => return_panic(),
//     Type::Macro(_) => return_panic(),
//     Type::Never(_) => return_panic(),
//     Type::Ptr(_) => return_panic(),

//     // Some `Type`s can't even be considered as
//     // valid return types.
//     _ => return_panic(),
// }
pub fn handle_return_type(mut ty: &Type) -> Option<TokenStream> {
    let mut ctx = TokenStream::new();
    let mut error = false;
    loop {
        match ty {
            Type::Reference(ty_ref) => {
                if ty_ref.mutability.is_some() {
                    error = true;
                    break;
                }
                ctx.extend(quote::quote!(&));
                ty = &*ty_ref.elem;
            }
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
                    if let Some(tokens) = handle_return_type(ty) {
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


// #[cfg(test)]
// mod tests {
//     use proc_macro2::TokenStream;
//     use syn::{parse_quote, TypeReference, Type};

//     use crate::dispatch::ret::{handle_reference, handle_type};
//     #[test]
//     fn token_test() {
        
//         let ref_option: Type = parse_quote!(&Option<String>);
//         let ref_option_tuple: Type = parse_quote!(&(&Option<i32>, Option<i32>));

//         let mut group = TokenStream::new();
//         handle_type(&ref_option, &mut group);
//         println!("tuple {}", group);

//         // let ref_option_tuple: TypeReference = parse_quote!(&(Option<i32>, Option<i32>));

//         // println!("tuple {}", handle_reference(&ref_option_tuple));
//     }
// }


// trait Trait {
//     fn ret(&self) -> &(Option<i32>, Option<String>);
//     fn ret2(&self) -> &(&Option<i32>, Option<String>);
// }

// struct A(i32);

// impl Trait for A {
//     fn ret(&self) -> &(Option<i32>, Option<String>) {
//         &(None, None)
//     }

//     fn ret2(&self) -> &(&Option<i32>, Option<String>) {
//         &(&None, None)
//     }
    
// }