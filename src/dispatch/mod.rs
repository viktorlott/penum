use std::{
    borrow::{Borrow, BorrowMut},
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Comma},
    Arm, Binding, Field, FnArg, Pat, Signature, TraitItem, TraitItemMethod, TraitItemType, parse_str, ExprMacro, Macro, parse_quote, Type, TypeParam,
};

use crate::{factory::TraitBound, penum::Stringify};

use standard::{StandardTrait, TraitSchematic};

mod standard;

#[repr(transparent)]
#[derive(Default)]
pub struct Blueprints<'bound>(BTreeMap<String, Vec<Blueprint<'bound>>>);
// pub type Blueprints<'bound> = Vec<Blueprint<'bound>>;

/// This blueprint contains everything we need to construct an impl statement.
///
/// The trait bound will contain the actual trait bound (obviously).
/// ```rust
/// AsRef<str>
/// ```
///
/// The `schematic` contains a replica of the trait definition.
/// ```rust
/// trait AsRef<T> {
///     fn as_ref(&self) -> &T;
/// }
/// ```
///
/// The `methods` contains a map of variant arms that is used to dispatch a variant parameter.
/// For each method:
/// ```rust
/// Foo::Bar(_, val, ..) => val.as_ref()
/// ```
#[derive(Clone)]
pub struct Blueprint<'bound> {
    /// Trait bound
    pub bound: &'bound TraitBound,
    /// Trait definition
    pub schematic: TraitSchematic,
    /// `method_name -> [Arm]`
    pub methods: BTreeMap<Ident, Vec<Arm>>,
}

pub struct VariantSignature<'info> {
    enum_ident: &'info Ident,
    variant_ident: &'info Ident,
    caller: Ident,
    params: Composite,
    span: Span,
}

/// For each <Dispatchable> -> <{ position, ident, fields }>
/// Used to know the position of a field.
pub enum Position<'a> {
    /// The index of the field being dispatched
    Index(usize, &'a Field),

    /// The key of the field being dispatched
    Key(&'a Ident),
}

pub enum Param {
    Ident(Ident),
    Placeholder,
    Rest,
}

pub enum Composite {
    Named(Punctuated<Param, Comma>, token::Brace),
    Unnamed(Punctuated<Param, Comma>, token::Paren),
}

/// This one is important. Use fields and position to create a pattern.
/// e.g. ident + position + fields + "bound signature" = `Ident::(_, X, ..) => X.method_call(<args if any>)`
impl<'a> Position<'a> {
    pub fn from_field(field: &'a Field, fallback: usize) -> Self {
        field
            .ident
            .as_ref()
            .map(Position::Key)
            .unwrap_or(Position::Index(fallback, field))
    }

    pub fn get_caller(&self) -> Ident {
        match self {
            Position::Index(_, field) => parse_quote_spanned! { field.span() => val },
            Position::Key(key) => parse_quote_spanned! { key.span() => #key },
        }
    }
}


impl<'bound> From<&'bound TraitBound> for Blueprint<'bound> {
    fn from(bound: &'bound TraitBound) -> Self {
        let schematic = StandardTrait::from(bound.get_ident()).into();
        Self {
            schematic,
            bound,
            methods: Default::default(),
        }
    }
}


/// FIXME: USE VISITER PATTERN INSTEAD.
impl<'bound> Blueprint<'bound> {

    
    pub fn get_associated_methods(&self) -> Vec<TraitItemMethod> {
        let mut method_items = vec![];
        let mut polymap = BTreeMap::<String, &Type>::new();

        if let Some(types) = self.get_bound_generics() {
            let generics = self.get_schematic_generics();
            // Generic -> concrete
            // T       -> str
            polymap = generics.zip(types).map(|(gen, ty)| (gen.get_string(), ty)).collect::<BTreeMap<_, _>>();
        }
        // match return_type {
        //     Type::Array(_) => todo!(),
        //     Type::BareFn(_) => todo!(),
        //     Type::Group(_) => todo!(),
        //     Type::ImplTrait(_) => todo!(),
        //     Type::Infer(_) => todo!(),
        //     Type::Macro(_) => todo!(),
        //     Type::Never(_) => todo!(),
        //     Type::Paren(_) => todo!(),
        //     Type::Path(_) => todo!(),
        //     Type::Ptr(_) => todo!(),
        //     Type::Reference(_) => todo!(),
        //     Type::Slice(_) => todo!(),
        //     Type::TraitObject(_) => todo!(),
        //     Type::Tuple(_) => todo!(),
        //     Type::Verbatim(_) => todo!(),
        //     _ => todo!(),
        // }

        for method in self.get_schematic_methods() {
            if let Some(method_arms) = self.methods.get(&method.sig.ident) {
                let TraitItemMethod { ref sig, .. } = method;

                let default_return = if matches!(sig.output, syn::ReturnType::Default) {
                    quote::quote!(())
                } else {
                    // Right now, we always default to a panic. But we could consider other options here too.
                    // For example, 
                    //  if we had an Option return type, we could default with `None` instead.
                    //  if we had an 
                    parse_str::<ExprMacro>("panic!(\"Missing arm\")").unwrap().to_token_stream()
                };

                let item: TraitItemMethod = parse_quote!(
                    #sig {
                        match self {
                            #(#method_arms,)*
                            _ => #default_return
                        }
                    }
                );

                method_items.push(item);
            }
        }
        method_items
    }

    /// Used to zip `get_bound_bindings` and `get_schematic_types` together.
    ///
    /// ```rust
    /// struct A where i32: Deref<Target = i32>;
    /// //                        ^^^^^^^^^^^^
    /// //                        |
    /// //                        get_bound_bindings()
    /// trait Deref for A {
    ///     type Target;
    /// //       ^^^^^^
    /// //       |
    /// //       get_schematic_types()
    /// 
    ///     fn deref(&self) -> &Target;
    /// }
    /// 
    /// type Target = i32;
    /// //   ^^^^^^^^^^^^
    /// //   |
    /// //   get_bound_bindings() <> get_schematic_types()
    /// ``
    pub fn get_mapped_bindings(&self) -> Option<Vec<TraitItemType>> {
        let Some(bindings) = self.get_bound_bindings() else {
            return None
        };
        
        let mut types = self.get_schematic_types().collect::<Vec<_>>();

        for binding in bindings {
            let Some(matc) = types.iter_mut().find_map(|assoc| assoc.ident.eq(&binding.ident).then_some(assoc)) else {
                panic!("Missing associated trait bindings")
            };

            if matc.default.is_none() {
                matc.default = Some((binding.eq_token, binding.ty.clone()));
            }
        }

        Some(types)
    }

    
    /// Used to extract all bindings in a trait bound
    ///
    /// ```rust
    /// struct A where i32: Deref<Target = i32>; 
    /// //                        ^^^^^^^^^^^^
    /// //                        |
    /// //                        Binding
    /// ``
    fn get_bound_bindings(&self) -> Option<impl Iterator<Item = &Binding>> {
        let path_segment = self.bound.path.segments.last().unwrap();
        match path_segment.arguments.borrow() {
            syn::PathArguments::AngleBracketed(angle) => {
                Some(angle.args.iter().filter_map(|arg| match arg {
                    syn::GenericArgument::Binding(binding) => Some(binding),
                    _ => None,
                }))
            }
            _ => None,
        }
    }

    /// Used to extract all generics in a trait bound.
    /// Though, we are more picking out the concrete types
    /// that substitute the generics.
    ///
    /// ```rust
    /// struct A where i32: AsRef<i32>; // <-- Trait bound
    /// //                        ^^^
    /// //                        |
    /// //                        Concrete type
    /// ```
    fn get_bound_generics(&self) -> Option<impl Iterator<Item = &Type>> {
        let path_segment = self.bound.path.segments.last().unwrap();
        match path_segment.arguments.borrow() {
            syn::PathArguments::AngleBracketed(angle) => {
                Some(angle.args.iter().filter_map(|arg| match arg {
                    syn::GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                }))
            }
            _ => None,
        }
    }


    /// Used to extract all generic types in a trait
    ///
    /// ```rust
    /// trait AsRef<T> for A {
    /// //          ^
    /// //          |
    /// //          Generic type (Type Param)
    ///     fn as_ref(&self) -> &T;
    /// }
    /// ```
    fn get_schematic_generics(&self) -> impl Iterator<Item = &TypeParam> {
        self.schematic.generics.params.iter().filter_map(|param| match param {
            syn::GenericParam::Type(ty) => Some(ty),
            _ => None
        })
    }


    /// Used to extract all associated types in a trait
    ///
    /// ```rust
    /// trait Deref for A {
    ///     type Target; 
    /// //       ^^^^^^
    /// //       |
    /// //       Associated type
    ///     fn deref(&self) -> &Target;
    /// }
    /// ```
    fn get_schematic_types(&self) -> impl Iterator<Item = TraitItemType> + '_ {
        self.schematic.items.iter().filter_map(|item| match item {
            TraitItem::Type(ty) => Some(ty.clone()),
            _ => None,
        })
    }

      /// Used to extract all associated methods in a trait
    ///
    /// ```rust
    /// trait Deref for A {
    ///     type Target; 
    ///     fn deref(&self) -> &Target;
    /// //  ^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// //  |
    /// //  Associated method
    /// }
    /// ```
    fn get_schematic_methods(&self) -> impl Iterator<Item = TraitItemMethod> + '_ {
        self.schematic.items.iter().filter_map(|item| match item {
            TraitItem::Method(method) => Some(method.clone()),
            _ => None,
        })
    }
}

impl<'info> VariantSignature<'info> {
    pub fn new(
        enum_ident: &'info Ident,
        variant_ident: &'info Ident,
        field: &Field,
        max_length: usize,
    ) -> Self {
        let position = Position::from_field(field, max_length);
        let caller = position.get_caller();
        let fields = position.format_fields_pattern(max_length);

        Self {
            enum_ident,
            variant_ident,
            caller,
            params: fields,
            span: field.span(),
        }
    }

    pub fn parse_arm(&'info self, method: &'info TraitItemMethod) -> (&Ident, Arm) {
        let Self {
            enum_ident,
            variant_ident,
            caller,
            params: fields,
            span,
        } = self;

        let (method_ident, sanitized_input) = get_method_parts(method);

        (
            method_ident,
            parse_quote_spanned! {span.span() => #enum_ident :: #variant_ident #fields => #caller . #method_ident (#sanitized_input)},
        )
    }
}

impl<'bound> Blueprint<'bound> {
    pub fn attach(&mut self, variant_sig: &VariantSignature) {
        let mut arms: BTreeMap<Ident, Vec<Arm>> = Default::default();

        for item in self.schematic.items.iter() {
            let TraitItem::Method(method) = item else {
                continue
            };

            let (method_name, parsed_arm) = variant_sig.parse_arm(method);

            if let Some(arm_vec) = arms.get_mut(method_name) {
                arm_vec.push(parsed_arm)
            } else {
                arms.insert(method_name.clone(), vec![parsed_arm]);
            }
        }

        arms.into_iter().for_each(|(method_name, mut am)| {
            if let Some(arm_vec) = self.methods.get_mut(&method_name) {
                arm_vec.append(&mut am);
            } else {
                self.methods.insert(method_name, am);
            }
        })
    }
}

impl<'a> Position<'a> {
    pub fn format_fields_pattern(&self, arity: usize) -> Composite {
        let mut punc = Punctuated::<Param, Comma>::new();

        match self {
            Position::Index(index, field) => {
                for _ in 1..*index {
                    punc.push_value(Param::Placeholder);
                    punc.push_punct(Comma(field.span()));
                }

                punc.push_value(Param::Ident(Ident::new("val", field.span())));

                if arity > index + 1 {
                    punc.push_punct(Comma(field.span()));
                    punc.push_value(Param::Rest);
                }

                Composite::Unnamed(punc, token::Paren(field.span()))
            }
            Position::Key(key) => {
                punc.push_value(Param::Ident(key.deref().clone()));
                if arity > 1 {
                    punc.push_punct(Comma(key.span()));
                    punc.push_value(Param::Rest);
                }

                Composite::Named(punc, token::Brace(key.span()))
            }
        }
    }
}

impl ToTokens for Composite {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Composite::Named(param, brace) => {
                brace.surround(tokens, |tokens| param.to_tokens(tokens))
            }
            Composite::Unnamed(param, paren) => {
                paren.surround(tokens, |tokens| param.to_tokens(tokens))
            }
        }
    }
}

impl ToTokens for Param {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Param::Ident(ident) => ident.to_tokens(tokens),
            Param::Placeholder => token::Underscore(Span::call_site()).to_tokens(tokens),
            Param::Rest => token::Dot2(Span::call_site()).to_tokens(tokens),
        }
    }
}

impl<'bound> Deref for Blueprints<'bound> {
    type Target = BTreeMap<String, Vec<Blueprint<'bound>>>;

    fn deref(&self) -> &Self::Target {
        self.0.borrow()
    }
}

impl<'bound> DerefMut for Blueprints<'bound> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.borrow_mut()
    }
}

fn sanitize(inputs: &Punctuated<FnArg, Comma>) -> Punctuated<Pat, Comma> {
    let mut san = Punctuated::new();
    let max = inputs.len();
    inputs.iter().enumerate().for_each(|(i, arg)| match arg {
        syn::FnArg::Receiver(_) => (),
        syn::FnArg::Typed(typed) => {
            san.push_value(typed.pat.deref().clone());
            if i != max - 1 {
                san.push_punct(Comma(Span::call_site()));
            }
        }
    });
    san
}

fn get_method_parts(method: &TraitItemMethod) -> (&Ident, Punctuated<Pat, Comma>) {
    let TraitItemMethod { sig, .. } = method;
    let Signature { ident, inputs, .. } = sig;
    (ident, sanitize(inputs))
}
