use std::ops::Deref;

use proc_macro2::Span;

use syn::Signature;
use syn::Pat;
use syn::FnArg;
use syn::spanned::Spanned;
use syn::token;
use syn::token::Comma;
use syn::punctuated::Punctuated;
use syn::Arm;
use syn::TraitItemMethod;
use syn::parse_quote_spanned;
use syn::Field;
use syn::Ident;

use quote::ToTokens;


pub struct VariantSignature<'info> {
    enum_ident: &'info Ident,
    variant_ident: &'info Ident,
    caller: Ident,
    params: Composite,
    span: Span,
}

/// For each <Dispatchable> -> <{ position, ident, fields }> Used to
/// know the position of a field.
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
/// e.g. ident + position + fields + "bound signature" = `Ident::(_, X,
/// ..) => X.method_call(<args if any>)`
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
            Position::Index(_, field) => parse_quote_spanned! {field.span()=> val },
            Position::Key(key) => parse_quote_spanned! {key.span()=> #key },
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

    /// To be able to construct a dispatch arm we would need two things,
    /// a variant signature and a trait item containing a method ident
    /// and inputs.
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
            parse_quote_spanned! {span.span()=> #enum_ident :: #variant_ident #fields => #caller . #method_ident (#sanitized_input)},
        )
    }
}


impl<'a> Position<'a> {
    /// We use this to format the call signature of the variant. It
    /// basically picks the value that is being dispatch and excludes
    /// the rest of the input fields.
    ///
    /// e.g. if we have a variant that contains 4 fields where the
    /// second field is to be dispatched, it'd look something like this:  
    /// - (_, val, ..) => val.<disptch>()
    /// - { somefield, ..} => somefield.<dispatch>()
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
