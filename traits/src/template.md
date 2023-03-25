use std::str::FromStr;
use std::ops::Deref;

use proc_macro2::Ident;
use syn::parse_quote;
use syn::ItemTrait;

#[derive(Debug)]
pub enum {enum_name} {{
    {enum_variants}
}}

#[repr(transparent)]
#[derive(Clone, Hash, Debug)]
pub struct TraitSchematic(pub ItemTrait);

impl From<{enum_name}> for TraitSchematic {{
    fn from(value: {enum_name}) -> Self {{
        TraitSchematic(
            match value {{
                {from_enum_to_item}
            }}
            .expect("Std trait file should exist"),
        )
    }}
}}

impl FromStr for {enum_name} {{
    type Err = ();
    fn from_str(value: &str) -> Result<Self, ()> {{
        Ok(match value {{
            {from_str_to_enum},
            _ => panic!("no match found, {}", value),
        }})
    }}
}}

impl From<&Ident> for {enum_name} {{
    fn from(value: &Ident) -> Self {{
        StandardTrait::from_str(value.to_string().as_str())
            .expect("Expect to find a match")
    }}
}}

impl Deref for TraitSchematic {{
    type Target = ItemTrait;

    fn deref(&self) -> &Self::Target {{
        &self.0
    }}
}}
