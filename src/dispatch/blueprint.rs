use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::ops::DerefMut;

use proc_macro2::Ident;

use syn::parse_quote;
use syn::parse_str;
use syn::punctuated::Punctuated;
use syn::visit_mut::visit_angle_bracketed_generic_arguments_mut;
use syn::visit_mut::visit_type_mut;
use syn::visit_mut::VisitMut;
use syn::Arm;
use syn::Binding;
use syn::GenericArgument;
use syn::ItemTrait;
use syn::Token;
use syn::TraitBound as SynTraitBound;
use syn::TraitItem;
use syn::TraitItemMethod;
use syn::TraitItemType;
use syn::Type;
use syn::TypeParam;

use crate::factory::TraitBound;
use crate::utils::UniqueHashId;

use super::ret::return_default_ret_type;
use super::ret::return_panic;
use super::T_SHM;

use super::sig::VariantSig;
use super::standard::StandardTrait;
use super::standard::TraitSchematic;

/// This blueprint contains everything we need to construct an impl
/// statement.
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
/// The `methods` contains a map of variant arms that is used to
/// dispatch a variant parameter. For each method:
/// ```rust
/// Foo::Bar(_, val, ..) => val.as_ref()
/// ```
#[derive(Clone, Hash, Debug)]
pub struct Blueprint<'bound> {
    /// Trait bound
    pub bound: &'bound TraitBound,
    /// Trait definition
    pub schematic: TraitSchematic,
    /// `method_name -> [Arm]`
    pub methods: BTreeMap<Ident, Vec<Arm>>,
}
// FIXME: Should be by Trait bound instead of by Type?
// This will stop working when `impl Trait for {A, B}` because
// they are interpreted as two different impls.
// `_ where i32: ^Trait, usize: ^Trait`
#[repr(transparent)]
#[derive(Default, Hash, Debug)]
pub struct BlueprintsMap<'bound>(BTreeMap<UniqueHashId<Type>, Vec<Blueprint<'bound>>>);

/// Only use this for modifying methods trait generics. Should probably
/// use visit_mut more often..
///
/// ```text
///                           Currently no support for method generics...
/// trait A<T> {              |                  |
///     fn very_cool_function<U>(&self, a: T, b: U) -> &T;
/// }                                      |            |
///                                        We only do substitutions on trait generics.
/// ```
struct MonomorphizeFnSignature<'poly>(&'poly BTreeMap<Ident, &'poly Type>);

///        
/// ```text
/// T: Add<T, Output = T>
/// |      |           |
/// |      Replace these generics with concrete variant types
/// |
/// This one already gets replace during polymophic mapping step.
/// ```
struct MonomorphizeTraitBound<'poly>(&'poly BTreeMap<Ident, &'poly Type>);

///        
/// ```text
/// where T: Add<i32, Output = i32>
///                   ^^^^^^^^^^^^
///                   |
///                   Remove bindings form trait bound.
///                                        
/// ```
struct RemoveBoundBindings;

/// FIXME: USE VISITER PATTERN INSTEAD.
impl<'bound> Blueprint<'bound> {
    /// Should probably be using `visit_mut` more often......
    pub fn get_associated_methods(&self) -> Vec<TraitItemMethod> {
        let mut method_items = vec![];

        // This polymap only contains TRAIT GENERIC PARAM MAPPINGS e.g.
        // A<i32>
        let polymap = self.get_bound_generics().map(|types| {
            self.get_schematic_generics()
                .zip(types)
                .map(|(gen, ty)| (gen.ident.clone(), ty))
                .collect::<BTreeMap<_, _>>()
        });

        for method in self.get_schematic_methods() {
            if let Some(method_arms) = self.methods.get(&method.sig.ident) {
                let TraitItemMethod { ref sig, .. } = method;

                let mut signature = sig.clone();

                if let Some(polymap) = polymap.as_ref() {
                    MonomorphizeFnSignature(polymap).visit_signature_mut(&mut signature)
                }

                // Right now, we always default to a panic. But we could
                // consider other options here too. For example, if we
                // had an Option return type, we could default with
                // `None` instead. Read more /docs/static-dispatch.md

                // We should look for Default implementations on the
                // return type. Through, a `-> &T` where `T: Default`.
                // It's not possible to do `&Default::default()` or
                // `&T::default()` IIRC. A &T where T isn't owned by
                // self needs to be ZST to be able to be returned.
                let default_return = match signature.output.borrow() {
                    syn::ReturnType::Default => quote::quote!(()),
                    syn::ReturnType::Type(_, ty) => {
                        return_default_ret_type(ty).unwrap_or_else(return_panic)
                    }
                };

                // A method item that is ready to be implemented
                let item: TraitItemMethod = parse_quote!(
                    #signature { match self { #(#method_arms,)* _ => #default_return } }
                );

                method_items.push(item);
            }
        }
        method_items
    }

    /// Used to zip `get_bound_bindings` and `get_schematic_types`
    /// together.
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
    /// ```
    pub fn get_mapped_bindings(&self) -> Option<Vec<TraitItemType>> {
        let Some(bindings) = self.get_bound_bindings() else {
            return None
        };

        let mut types = self.get_schematic_types().collect::<Vec<_>>();

        for binding in bindings {
            let Some(matc) = types.iter_mut()
                .find_map(|assoc| assoc.ident.eq(&binding.ident)
                .then_some(assoc)) else {
                panic!("Missing associated trait bindings")
            };

            if matc.default.is_none() {
                matc.default = Some((binding.eq_token, binding.ty.clone()));
            }
        }

        Some(types)
    }

    /// Fill our blueprint with dispatchable variant arms that we later
    /// use to contruct an impl statement.
    pub fn attach(&mut self, variant_sig: &VariantSig) {
        let mut arms: BTreeMap<Ident, Vec<Arm>> = Default::default();

        for item in self.schematic.items.iter() {
            let TraitItem::Method(method) = item else {
                continue
            };

            // FIXME: FILTER RECEIVER METHODS.

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

    pub fn get_sanatized_impl_path(&self) -> SynTraitBound {
        let tb = self.bound.clone();
        let mut tb: SynTraitBound = parse_quote!(#tb);
        RemoveBoundBindings.visit_trait_bound_mut(&mut tb);
        tb
    }
}

impl<'bound> Blueprint<'bound> {
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

    /// Used to extract all generics in a trait bound. Though, we are
    /// more picking out the concrete types that substitute the
    /// generics.
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
        self.schematic
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Type(ty) => Some(ty),
                _ => None,
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

impl<'bound> TryFrom<&'bound TraitBound> for Blueprint<'bound> {
    type Error = syn::Error;
    fn try_from(bound: &'bound TraitBound) -> Result<Self, Self::Error> {
        let b_name = bound.get_ident();

        if let Ok(schematic) = StandardTrait::try_from(b_name) {
            Ok(Self {
                schematic: schematic.into(),
                bound,
                methods: Default::default(),
            })
        } else if let Some(Ok(schematic)) = T_SHM
            .find(&b_name.to_string())
            .as_ref()
            .map(|result| parse_str::<ItemTrait>(result))
        {
            Ok(Self {
                schematic: TraitSchematic(schematic),
                bound,
                methods: Default::default(),
            })
        } else {
            Err(syn::Error::new_spanned(bound, trait_not_found(bound)))
        }
    }
}

fn trait_not_found(bound: &TraitBound) -> String {
    format!("`{}` cannot be found. Make sure the trait is tagged with the `#[penum]` attribute, and is invoked before your enum.", bound.get_ident())
}

impl<'bound> BlueprintsMap<'bound> {
    /// This flattens values in the map.
    /// ty: [blueprint] -> [[blueprint]] -> [blueprint]
    ///
    /// NOTE: Some of these blueprints might be duplicates, meaning that we implement two or more
    /// times for the same trait. So what we have to do here is deduplicate the blueprints by
    /// mapping over the trait bounds instead of the concrete types.
    ///
    /// FIXME: Change so that we can map on trait bounds instead of just concrete types. Each
    /// implementation needs to be unique, i.e. there can only be one trait implementation per type.
    /// Note, Trait<U> and Trait<T> are considered different, so we should support generic traits.
    pub fn for_each_blueprint(&self, mut f: impl FnMut(&Blueprint)) {
        // TODO: We could probably just use a HashSet instead and implement Hash for Blueprint->bound.
        let mut deduplicates: BTreeMap<UniqueHashId<Type>, Blueprint<'bound>> = Default::default();

        for item in self.0.iter() {
            for blueprint in item.1.iter() {
                let bound = blueprint.bound;
                let id: Type = parse_quote!(#bound);
                let id_unique = UniqueHashId::new(&id);

                // FIXME: TEMP, should fix this copy mess
                if let Some(unique_entry) = deduplicates.get_mut(&id_unique) {
                    unique_entry
                        .methods
                        .extend(blueprint.methods.clone().into_iter());
                } else {
                    deduplicates.insert(id_unique, blueprint.clone());
                }
            }
        }

        deduplicates.iter().for_each(|m| f(m.1))
    }

    pub fn find_and_attach(&mut self, id: &UniqueHashId<Type>, variant_sig: &VariantSig) -> bool {
        if let Some(bp_list) = self.get_mut(id) {
            for blueprint in bp_list.iter_mut() {
                blueprint.attach(variant_sig)
            }
            true
        } else {
            false
        }
    }
}

impl<'bound> Deref for BlueprintsMap<'bound> {
    type Target = BTreeMap<UniqueHashId<Type>, Vec<Blueprint<'bound>>>;

    fn deref(&self) -> &Self::Target {
        self.0.borrow()
    }
}

impl<'bound> DerefMut for BlueprintsMap<'bound> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.borrow_mut()
    }
}

impl VisitMut for MonomorphizeFnSignature<'_> {
    /// Skip mutating generic parameter in method signature
    fn visit_generics_mut(&mut self, _: &mut syn::Generics) {}

    /// We only care about mutating path types
    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        if let Type::Path(typath) = node {
            // assuming it's always a generic parameter.
            if let Some(&ty) = typath.path.get_ident().and_then(|ident| self.0.get(ident)) {
                *node = ty.clone();
            }
        }
        visit_type_mut(self, node);
    }
}

impl VisitMut for MonomorphizeTraitBound<'_> {
    /// Skip mutating generic parameter in method signature
    fn visit_generics_mut(&mut self, _: &mut syn::Generics) {}

    /// We only care about mutating path types
    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        if let Type::Path(typath) = node {
            // assuming it's always a generic parameter.
            if let Some(&ty) = typath.path.get_ident().and_then(|ident| self.0.get(ident)) {
                *node = ty.clone();
            }
        }
        visit_type_mut(self, node);
    }
}

impl VisitMut for RemoveBoundBindings {
    fn visit_angle_bracketed_generic_arguments_mut(
        &mut self,
        node: &mut syn::AngleBracketedGenericArguments,
    ) {
        let mut rep_gas: Punctuated<GenericArgument, Token![,]> = Default::default();

        let mut args = node.args.iter().peekable();

        // Ugh, refactor this
        loop {
            let (Some(gen), s) = (args.next(), args.peek()) else {
                break
            };

            if !matches!(gen, GenericArgument::Binding(_)) {
                rep_gas.push_value(gen.clone());

                if let Some(GenericArgument::Binding(_)) = s {
                    break;
                } else {
                    rep_gas.push_punct(parse_quote!(,));
                }
            };
        }

        if args.count() != 0 {
            node.args = rep_gas;
        }

        visit_angle_bracketed_generic_arguments_mut(self, node);
    }
}
