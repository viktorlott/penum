use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Ident;
use syn::{Fields, ItemTrait};

mod as_ref;

pub use as_ref::construct;

#[derive(Default)]
pub struct DispatchMap(pub BTreeMap<Dispatchable, BTreeSet<Dispatchalor>>);

pub struct Dispatchable {
    pub trait_decl: ItemTrait,
}

/// For each <Dispatchable> -> <{ position, ident, fields }>
/// Used for dispatching
pub enum Position {
    /// The index of the field being dispatched
    Index(usize),

    /// The key of the field being dispatched
    Key(String),
}

/// This one is important. Use fields and position to create a pattern.
/// e.g. ident + position + fields + "bound signature" = `Ident::(_, X, ..) => X.method_call(<args if any>)`
pub struct Dispatchalor {
    /// The name of the variant
    pub ident: Ident,

    /// Used for dispatching
    pub position: Position,

    pub fields: Fields,
}

// impl DispatchMap {
//     pub fn polymap_insert(&mut self, pty: String, ity: String) {
//         if let Some(set) = self.0.get_mut(pty.as_str()) {
//             set.insert(ity);
//         } else {
//             self.0.insert(pty, vec![ity].into_iter().collect());
//         }
//     }
// }
