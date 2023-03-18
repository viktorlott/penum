use std::{collections::BTreeMap, ops::Deref};

use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Comma},
    Arm, Field, FnArg, Pat, Signature, TraitItem, TraitItemMethod,
};

use crate::{factory::TraitBound};

use standard::{StandardTrait, TraitSchematic};

mod standard;

pub trait Sanitize {
    fn sanitize(&self) -> String;
}

pub type BlueprintMap<'bound> = BTreeMap<String, Vec<Blueprint<'bound>>>;
pub type Blueprints<'bound> = Vec<Blueprint<'bound>>;

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

impl<'bound> Blueprint<'bound> {
    pub fn from(bound: &'bound TraitBound) -> Self {
        let schematic = StandardTrait::from(bound.get_ident()).into();
        Self {
            schematic,
            bound,
            methods: Default::default(),
        }
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
        // println!("{} :: {} {} => {} . {} ({})", enum_ident.to_token_stream(), variant_ident.to_token_stream(), fields.to_token_stream(), caller.to_token_stream(), method_ident.to_token_stream(), sanitized_input.to_token_stream());
        (
            method_ident,
            parse_quote_spanned! {span.span() => #enum_ident :: #variant_ident #fields => #caller . #method_ident (#sanitized_input)},
        )
    }
}

impl<'bound> Blueprint<'bound> {
    pub fn attatch(&mut self, variant_sig: &VariantSignature) {
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
