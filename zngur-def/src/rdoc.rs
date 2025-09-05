use crate::*;
use rustdoc_types::{Crate, GenericArg, GenericArgs, Id, ItemSummary, Struct, Type};

impl ZngurSpec {
    pub fn from_crate(value: Crate, layout_info: HashMap<String, LayoutPolicy>) -> Self {
        let crate_name = value.index.get(&value.root).unwrap().clone().name.unwrap();
        let mut spec = Self::default();
        for (id, item) in value.index.iter() {
            match &item.inner {
                ItemEnum::Struct(s) => {
                    if let Some(ztype) = convert_struct_to_zngur_type(s, id, &value, &layout_info) {
                        spec.types.push(ztype);
                    }
                }
                ItemEnum::Enum(e) => {
                    if let Some(ztype) = convert_enum_to_zngur_type(e, id, &value, &crate_name) {
                        spec.types.push(ztype);
                    }
                }
                ItemEnum::Function(_) => {
                    if let Some(zfn) = fn_to_zngfn(id, &value, &crate_name) {
                        spec.funcs.push(zfn);
                    }
                }
                ItemEnum::Module(_) => {
                    // Handle module items if needed
                }
                ItemEnum::ExternCrate { .. } => {
                    // Handle extern crate items if needed
                }
                ItemEnum::Union(_) => {}
                ItemEnum::StructField(_) => {
                    // Handle struct field items if needed
                }
                ItemEnum::Variant(_) => {
                    // Handle variant items if needed
                }
                ItemEnum::Trait(_) => {
                    // Handle trait items if needed
                }
                ItemEnum::TraitAlias(_) => {
                    // Handle trait alias items if needed
                }
                ItemEnum::Impl(_) => {
                    // Handle impl items if needed
                }
                ItemEnum::TypeAlias(_) => {
                    // Handle type alias items if needed
                }
                ItemEnum::Constant { .. } => {
                    // Handle constant items if needed
                }
                ItemEnum::Static(_) => {
                    // Handle static items if needed
                }
                ItemEnum::ExternType => {}
                ItemEnum::Macro(_) => {
                    // Handle macro items if needed
                }
                ItemEnum::ProcMacro(_) => {
                    // Handle proc macro items if needed
                }
                ItemEnum::Primitive(_) => {
                    // Handle primitive items if needed
                }
                ItemEnum::AssocConst { .. } => {
                    // Handle associated constant items if needed
                }
                ItemEnum::AssocType { .. } => {
                    // Handle associated type items if needed
                }
                ItemEnum::Use(u) => {
                    dbg!(u);
                    // Handle use items if needed
                }
            }
        }
        spec
    }
}

fn convert_struct_to_zngur_type(
    s: &Struct,
    id: &Id,
    value: &Crate,
    layout_info: &HashMap<String, LayoutPolicy>,
) -> Option<ZngurType> {
    let mut path_info = value.paths.get(id)?.clone();
    let is_local = convert_path(&mut path_info);
    let path = path_info.path;
    let layout = if is_local {
        layout_info.get(path.last().unwrap()).unwrap().clone()
    } else {
        layout_info.get(&path.join("::")).unwrap().clone()
    };

    let ty = RustType::Adt(RustPathAndGenerics {
        path,
        generics: vec![],
        named_generics: vec![],
    });

    let mut ztype = ZngurType {
        ty,
        layout,
        methods: vec![],
        //TODO:NRB handle dynamically
        wellknown_traits: vec![ZngurWellknownTrait::Drop],
        constructors: vec![],
        fields: vec![],
        cpp_value: None,
        cpp_ref: None,
    };

    process_impls(&mut ztype, &s.impls, value);
    Some(ztype)
}

fn convert_enum_to_zngur_type(
    e: &rustdoc_types::Enum,
    id: &rustdoc_types::Id,
    value: &rustdoc_types::Crate,
    crate_name: &String,
) -> Option<ZngurType> {
    let mut path_info = value.paths.get(id)?.clone();
    convert_path(&mut path_info);
    let path = path_info.path;

    let ty = RustType::Adt(RustPathAndGenerics {
        path,
        generics: vec![],
        named_generics: vec![],
    });

    let mut ztype = ZngurType {
        ty,
        layout: LayoutPolicy::HeapAllocated,
        methods: vec![],
        wellknown_traits: vec![],
        constructors: vec![],
        fields: vec![],
        cpp_value: None,
        cpp_ref: None,
    };

    process_impls(&mut ztype, &e.impls, value);
    Some(ztype)
}

fn process_impls(ztype: &mut ZngurType, impls: &[rustdoc_types::Id], value: &rustdoc_types::Crate) {
    //TODO:NRB dumb impl, fix logic. pass items directly
    for impl_id in impls {
        let impl_item = match value.index.get(impl_id) {
            Some(item) => item,
            None => continue,
        };

        let ItemEnum::Impl(i) = &impl_item.inner else {
            continue;
        };

        match &i.trait_ {
            None => {
                // Native impl, add methods
                for method_id in &i.items {
                    if let Some(mut method) = fn_to_zngmethod(method_id, value) {
                        // outputs self
                        if let RustType::Adt(s) = &method.output
                            && s.path == vec!["Self"]
                        {
                            method.output = ztype.ty.clone();
                        }
                        ztype.methods.push(ZngurMethodDetails {
                            data: method,
                            use_path: None,
                            deref: None,
                        });
                    }
                }
            }
            Some(t) => {
                // Trait impl, add to wellknown_traits
                match t.path.as_str() {
                    "Debug" => {
                        ztype.wellknown_traits.push(ZngurWellknownTrait::Debug);
                    }
                    "Drop" => {
                        ztype.wellknown_traits.push(ZngurWellknownTrait::Drop);
                    }
                    "Unsized" => {
                        ztype.wellknown_traits.push(ZngurWellknownTrait::Unsized);
                    }
                    "Copy" => {
                        ztype.wellknown_traits.push(ZngurWellknownTrait::Copy);
                    }
                    _ => {
                        println!("Unsupported trait impl: {}", t.path)
                    }
                }
            }
        }
    }
}

fn fn_to_zngmethod(
    func_id: &rustdoc_types::Id,
    value: &rustdoc_types::Crate,
) -> Option<ZngurMethod> {
    let crate_name = value.index.get(&value.root).unwrap().clone().name.unwrap();
    let item = value.index.get(func_id)?;
    let ItemEnum::Function(func) = &item.inner else {
        return None;
    };

    let selff = func
        .sig
        .inputs
        .iter()
        .find(|(name, _ty)| name == "self")
        .map(|(_name, ty)| ty);
    let receiver = match selff {
        None => ZngurMethodReceiver::Static,
        Some(rec) => {
            match rec {
                Type::BorrowedRef {
                    lifetime,
                    is_mutable,
                    type_,
                } => {
                    if *is_mutable {
                        ZngurMethodReceiver::Ref(Mutability::Mut)
                    } else {
                        ZngurMethodReceiver::Ref(Mutability::Not)
                    }
                }
                Type::Generic(s) => {
                    // check s == "Self"?
                    ZngurMethodReceiver::Move
                }
                // Maybe check for borrow ref and all others default to move?
                _ => panic!(),
            }
        }
    };

    //TODO:NRB fix crate local type names
    let inputs = func
        .sig
        .inputs
        .iter()
        .filter(|(name, _ty)| name != "self")
        .map(|(_name, ty)| RustType::try_from(ty.clone()).unwrap())
        .collect::<Vec<_>>();

    let output = if func.sig.output.is_some() {
        func.sig.output.clone().unwrap().try_into().unwrap()
    } else {
        RustType::UNIT
    };
    Some(ZngurMethod {
        name: item.name.clone()?,
        inputs,
        generics: vec![],
        receiver,
        output,
    })
}

fn fn_to_zngfn(
    func_id: &rustdoc_types::Id,
    value: &rustdoc_types::Crate,
    crate_name: &String,
) -> Option<ZngurFn> {
    let item = value.index.get(func_id)?;
    let mut path_info = value.paths.get(func_id)?.clone();
    convert_path(&mut path_info);
    let path = path_info.path;
    let ItemEnum::Function(func) = &item.inner else {
        return None;
    };

    let inputs = func
        .sig
        .inputs
        .iter()
        .map(|(_name, ty)| ty.clone().try_into().unwrap())
        .collect::<Vec<_>>();

    let output = if func.sig.output.is_some() {
        func.sig.output.clone().unwrap().try_into().unwrap()
    } else {
        RustType::UNIT
    };

    Some(ZngurFn {
        path: RustPathAndGenerics {
            path,
            generics: vec![],
            named_generics: vec![],
        },
        inputs,
        output,
    })
}

// HELPER FUNCTIONS

fn convert_path(summ: &mut ItemSummary) -> bool {
    if summ.crate_id == 0 {
        let f = summ.path.first_mut().unwrap();
        *f = "crate".into();
        return true;
    }
    false
}

// fn item_enum_to_rusttype(item: &ItemEnum, id: &Id, cr: &Crate) -> Option<RustType> {
//     let crate_name = cr.index.get(&cr.root).unwrap().clone().name.unwrap();
//     let mut path = cr.paths.get(id).unwrap().clone().path;
//     convert_path(&mut path, &crate_name);
//     match item {
//         ItemEnum::Struct(s) => {
//             // Just reference the generics, don't move them
//             let _gens = &s.generics;
//             Some(RustType::Adt(RustPathAndGenerics {
//                 path,
//                 generics: vec![],
//                 named_generics: vec![],
//             }))
//         }
//         ItemEnum::Enum(e) => Some(RustType::Adt(RustPathAndGenerics {
//             path,
//             generics: vec![],
//             named_generics: vec![],
//         })),
//         ItemEnum::Function(f) => {
//             let inputs = f
//                 .sig
//                 .inputs
//                 .iter()
//                 .map(|(name, ty)| ty.clone().try_into().unwrap())
//                 .collect::<Vec<RustType>>();
//
//             let output = if let Some(t) = &f.sig.output {
//                 t.clone().try_into().unwrap()
//             } else {
//                 RustType::UNIT
//             };
//             None
//         }
//         _ => None,
//     }
// }

impl TryFrom<rustdoc_types::Type> for RustType {
    type Error = ();

    fn try_from(value: rustdoc_types::Type) -> Result<Self, Self::Error> {
        match value {
            Type::ResolvedPath(path) => {
                // Convert rustdoc_types::Path to RustPathAndGenerics
                let path_vec = path.path.split("::").map(|s| s.to_string()).collect();
                let mut generics = vec![];
                if let Some(args) = path.args {
                    match *args {
                        GenericArgs::AngleBracketed { args, constraints } => {
                            args.iter().for_each(|x| {
                                if let GenericArg::Type(t) = x {
                                    generics.push(RustType::try_from(t.clone()).unwrap());
                                }
                            })
                        }
                        //TODO:NRB finish impl
                        GenericArgs::Parenthesized { inputs, output } => {}
                        GenericArgs::ReturnTypeNotation => {}
                    }
                }
                let named_generics = vec![];
                Ok(RustType::Adt(RustPathAndGenerics {
                    path: path_vec,
                    generics,
                    named_generics,
                }))
            }
            Type::DynTrait(dyn_trait) => {
                // Convert the first trait to RustTrait, ignore others for now
                if let Some(first_trait) = dyn_trait.traits.first() {
                    let trait_path = first_trait
                        .trait_
                        .path
                        .split("::")
                        .map(|s| s.to_string())
                        .collect();
                    let rust_trait = RustTrait::Normal(RustPathAndGenerics {
                        path: trait_path,
                        generics: vec![],
                        named_generics: vec![],
                    });
                    Ok(RustType::Dyn(rust_trait, vec![]))
                } else {
                    Ok(RustType::UNIT)
                }
            }
            Type::Generic(name) => {
                // For generic types, we'll need to handle them in context
                // For now, return a placeholder
                Ok(RustType::Adt(RustPathAndGenerics {
                    path: vec![name],
                    generics: vec![],
                    named_generics: vec![],
                }))
            }
            Type::Primitive(p) => Ok(RustType::Primitive(PrimitiveRustType::from(p))),
            Type::FunctionPointer(function_pointer) => {
                // Convert function pointer signature to RustType
                let inputs = function_pointer
                    .sig
                    .inputs
                    .iter()
                    .map(|(_, ty)| ty.clone().try_into().unwrap())
                    .collect();
                let output = function_pointer
                    .sig
                    .output
                    .as_ref()
                    .map(|ty| Box::new(ty.clone().try_into().unwrap()))
                    .unwrap_or(Box::new(RustType::UNIT));

                Ok(RustType::Dyn(
                    RustTrait::Fn {
                        name: "fn".to_string(),
                        inputs,
                        output,
                    },
                    vec![],
                ))
            }
            Type::Tuple(items) => {
                let converted_items = items
                    .iter()
                    .map(|ty| ty.clone().try_into().unwrap())
                    .collect();
                Ok(RustType::Tuple(converted_items))
            }
            Type::Slice(item_type) => {
                let converted_type = Box::new((*item_type).clone().try_into().unwrap());
                Ok(RustType::Slice(converted_type))
            }
            Type::Array { type_, len: _ } => {
                // For arrays, we'll treat them as slices for now
                // TODO: Consider adding a proper Array variant to RustType
                let converted_type = Box::new((*type_).clone().try_into().unwrap());
                Ok(RustType::Slice(converted_type))
            }
            Type::Pat {
                type_,
                __pat_unstable_do_not_use: _,
            } => {
                // Pattern types are experimental, treat as the underlying type
                Ok((*type_).clone().try_into().unwrap())
            }
            Type::ImplTrait(_generic_bounds) => {
                // Impl trait types are opaque, treat as unit for now
                // TODO: Implement proper impl trait handling
                Ok(RustType::UNIT)
            }
            Type::Infer => {
                // Inferred types, treat as unit for now
                Ok(RustType::UNIT)
            }
            Type::RawPointer { is_mutable, type_ } => {
                let mutability = if is_mutable {
                    Mutability::Mut
                } else {
                    Mutability::Not
                };
                let converted_type = Box::new((*type_).clone().try_into().unwrap());
                Ok(RustType::Raw(mutability, converted_type))
            }
            Type::BorrowedRef {
                lifetime: _,
                is_mutable,
                type_,
            } => {
                let mutability = if is_mutable {
                    Mutability::Mut
                } else {
                    Mutability::Not
                };
                let converted_type = Box::new((*type_).clone().try_into().unwrap());
                Ok(RustType::Ref(mutability, converted_type))
            }
            Type::QualifiedPath {
                name: _,
                args: _,
                self_type,
                trait_: _,
            } => {
                // Qualified paths like <T as Trait>::Item
                // For now, treat as the self type
                Ok((*self_type).clone().try_into().unwrap())
            }
        }
    }
}
