use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;

use cpp::CppExportedFnDefinition;
use cpp::CppExportedImplDefinition;
use cpp::CppFile;
use cpp::CppFnDefinition;
use cpp::CppFnSig;
use cpp::CppMethod;
use cpp::CppPath;
use cpp::CppTraitDefinition;
use cpp::CppType;
use cpp::CppTypeDefinition;
use cpp::cpp_handle_keyword;
use itertools::Itertools;
use rust::IntoCpp;

pub mod cpp;
mod rust;

pub use rust::RustFile;
pub use zngur_parser::ParsedZngFile;

pub use zngur_def::*;

pub struct ZngurGenerator(ZngurFile);

impl ZngurGenerator {
    pub fn build_from_zng(zng: ZngurFile) -> Self {
        ZngurGenerator(zng)
    }

    pub fn render(self) -> (String, String, Option<String>) {
        let mut zng = self.0;

        // Unit type is a bit special, and almost everyone needs it, so we add it ourself.
        zng.types.push(ZngurType {
            ty: RustType::UNIT,
            layout: Some(LayoutPolicy::StackAllocated { size: 0, align: 1 }),
            wellknown_traits: vec![ZngurWellknownTrait::Copy],
            methods: vec![],
            constructors: vec![],
            cpp_value: None,
            cpp_ref: None,
        });
        let mut cpp_file = CppFile::default();
        cpp_file.additional_includes = zng.additional_includes;
        let mut rust_file = RustFile::default();
        cpp_file.trait_defs = zng
            .traits
            .iter()
            .map(|(key, value)| (key.clone(), rust_file.add_builder_for_dyn_trait(value)))
            .collect();
        if zng.convert_panic_to_exception {
            rust_file.enable_panic_to_exception();
            cpp_file.panic_to_exception = true;
        }
        let defined_types: HashSet<_> = zng.types.iter().map(|ty| &ty.ty).cloned().collect();
        for ty_def in zng.types {
            let mut ty_def = augment_type_with_impls(ty_def, &zng.impls, &defined_types);

            let is_copy = ty_def.wellknown_traits.contains(&ZngurWellknownTrait::Copy);
            let is_unsized = ty_def
                .wellknown_traits
                .contains(&ZngurWellknownTrait::Unsized);
            if !is_copy && !is_unsized {
                ty_def.wellknown_traits.push(ZngurWellknownTrait::Drop);
            }

            let Some(layout) = ty_def.layout else {
                panic!("Missing layout for type {}", ty_def.ty)
            };
            match layout {
                LayoutPolicy::StackAllocated { size, align } => {
                    rust_file.add_static_size_assert(&ty_def.ty, size);
                    rust_file.add_static_align_assert(&ty_def.ty, align);
                }
                LayoutPolicy::HeapAllocated => (),
                LayoutPolicy::OnlyByRef => (),
            }
            if is_copy {
                rust_file.add_static_is_copy_assert(&ty_def.ty);
            }
            let mut cpp_methods = vec![];
            let mut constructors = vec![];
            let mut wellknown_traits = vec![];
            for constructor in ty_def.constructors {
                match constructor.name {
                    Some(name) => {
                        let rust_link_names = rust_file.add_constructor(
                            &format!("{}::{}", ty_def.ty, name),
                            &constructor.inputs,
                        );
                        cpp_methods.push(CppMethod {
                            name: cpp_handle_keyword(&name).to_owned(),
                            kind: ZngurMethodReceiver::Static,
                            sig: CppFnSig {
                                rust_link_name: rust_link_names.constructor,
                                inputs: constructor.inputs.iter().map(|x| x.1.into_cpp()).collect(),
                                output: ty_def.ty.into_cpp(),
                            },
                        });
                        cpp_methods.push(CppMethod {
                            name: format!("matches_{}", name),
                            kind: ZngurMethodReceiver::Ref(Mutability::Not),
                            sig: CppFnSig {
                                rust_link_name: rust_link_names.match_check,
                                inputs: vec![ty_def.ty.into_cpp().into_ref()],
                                output: CppType::from("uint8_t"),
                            },
                        });
                    }
                    None => {
                        let rust_link_name = rust_file
                            .add_constructor(&format!("{}", ty_def.ty), &constructor.inputs)
                            .constructor;
                        constructors.push(CppFnSig {
                            rust_link_name,
                            inputs: constructor.inputs.iter().map(|x| x.1.into_cpp()).collect(),
                            output: ty_def.ty.into_cpp(),
                        });
                    }
                }
            }
            if let RustType::Tuple(fields) = &ty_def.ty {
                if !fields.is_empty() {
                    let rust_link_name = rust_file.add_tuple_constructor(&fields);
                    constructors.push(CppFnSig {
                        rust_link_name,
                        inputs: fields.iter().map(|x| x.into_cpp()).collect(),
                        output: ty_def.ty.into_cpp(),
                    });
                }
            }
            for wellknown_trait in ty_def.wellknown_traits {
                let data = rust_file.add_wellknown_trait(&ty_def.ty, wellknown_trait, is_unsized);
                wellknown_traits.push(data);
            }
            for method_details in ty_def.methods {
                let ZngurMethodDetails {
                    data: method,
                    use_path,
                    deref,
                } = method_details;
                let (rusty_inputs, inputs) = real_inputs_of_method(&method, &ty_def.ty);
                let rust_link_name = rust_file.add_function(
                    &format!(
                        "<{}>::{}::<{}>",
                        deref.as_ref().unwrap_or(&ty_def.ty),
                        method.name,
                        method.generics.iter().join(", "),
                    ),
                    &rusty_inputs,
                    &method.output,
                    use_path,
                    deref.is_some(),
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(&method.name).to_owned(),
                    kind: method.receiver,
                    sig: CppFnSig {
                        rust_link_name,
                        inputs,
                        output: method.output.into_cpp(),
                    },
                });
            }
            cpp_file.type_defs.push(CppTypeDefinition {
                ty: ty_def.ty.into_cpp(),
                layout: rust_file.add_layout_policy_shim(&ty_def.ty, layout),
                constructors,
                methods: cpp_methods,
                wellknown_traits,
                cpp_value: ty_def.cpp_value.map(|(field, cpp_type)| {
                    let rust_link_name = rust_file.add_cpp_value_bridge(&ty_def.ty, &field);
                    (rust_link_name, cpp_type)
                }),
                cpp_ref: ty_def.cpp_ref,
                from_trait: if let RustType::Boxed(b) = &ty_def.ty {
                    if let RustType::Dyn(tr, _) = b.as_ref() {
                        if let RustTrait::Fn {
                            name,
                            inputs,
                            output,
                        } = tr
                        {
                            if let Entry::Vacant(e) = cpp_file.trait_defs.entry(tr.clone()) {
                                let rust_link_name =
                                    rust_file.add_builder_for_dyn_fn(name, inputs, output);
                                e.insert(CppTraitDefinition::Fn {
                                    sig: CppFnSig {
                                        rust_link_name,
                                        inputs: inputs.iter().map(|x| x.into_cpp()).collect(),
                                        output: output.into_cpp(),
                                    },
                                });
                            }
                        }
                        Some(tr.clone())
                    } else {
                        None
                    }
                } else {
                    None
                },
                from_trait_ref: if let RustType::Dyn(tr, _) = &ty_def.ty {
                    Some(tr.clone())
                } else {
                    None
                },
            });
        }
        for func in zng.funcs {
            let rust_link_name = rust_file.add_function(
                &func.path.to_string(),
                &func.inputs,
                &func.output,
                None,
                false,
            );
            cpp_file.fn_defs.push(CppFnDefinition {
                name: CppPath::from_rust_path(&func.path.path),
                sig: CppFnSig {
                    rust_link_name,
                    inputs: func.inputs.into_iter().map(|x| x.into_cpp()).collect(),
                    output: func.output.into_cpp(),
                },
            });
        }
        for func in zng.extern_cpp_funcs {
            let rust_link_name =
                rust_file.add_extern_cpp_function(&func.name, &func.inputs, &func.output);
            cpp_file.exported_fn_defs.push(CppExportedFnDefinition {
                name: func.name.clone(),
                sig: CppFnSig {
                    rust_link_name,
                    inputs: func.inputs.into_iter().map(|x| x.into_cpp()).collect(),
                    output: func.output.into_cpp(),
                },
            });
        }
        for impl_block in zng.extern_cpp_impls {
            let rust_link_names = rust_file.add_extern_cpp_impl(
                &impl_block.ty,
                impl_block.tr.as_ref(),
                &impl_block.methods,
            );
            cpp_file.exported_impls.push(CppExportedImplDefinition {
                tr: impl_block.tr.map(|x| x.into_cpp()),
                ty: impl_block.ty.into_cpp(),
                methods: impl_block
                    .methods
                    .iter()
                    .zip(&rust_link_names)
                    .map(|(method, link_name)| {
                        let (_, inputs) = real_inputs_of_method(method, &impl_block.ty);
                        (
                            cpp_handle_keyword(&method.name).to_owned(),
                            CppFnSig {
                                rust_link_name: link_name.clone(),
                                inputs,
                                output: method.output.into_cpp(),
                            },
                        )
                    })
                    .collect(),
            });
        }
        let (h, cpp) = cpp_file.render();
        (rust_file.text, h, cpp)
    }
}

fn real_inputs_of_method(method: &ZngurMethod, ty: &RustType) -> (Vec<RustType>, Vec<CppType>) {
    let receiver_type = match method.receiver {
        ZngurMethodReceiver::Static => None,
        ZngurMethodReceiver::Ref(m) => Some(RustType::Ref(m, Box::new(ty.clone()))),
        ZngurMethodReceiver::Move => Some(ty.clone()),
    };
    let rusty_inputs = receiver_type
        .into_iter()
        .chain(method.inputs.clone())
        .collect::<Vec<_>>();
    let inputs = rusty_inputs.iter().map(|x| x.into_cpp()).collect_vec();
    (rusty_inputs, inputs)
}

fn matches_generic<'a, 'b>(ty: &'a RustType, generic: &'b RustType, mapping: &mut HashMap<&'b str, &'a RustType>) -> bool {
    fn match_lists<'a, 'b>(v1: &'a [RustType], v2: &'b [RustType], mapping: &mut HashMap<&'b str, &'a RustType>) -> bool {
        v1.len() == v2.len() && match_iters(v1, v2, mapping)
    }

    fn match_iters<'a, 'b>(i1: impl IntoIterator<Item = &'a RustType>,  i2: impl IntoIterator<Item = &'b RustType>, mapping: &mut HashMap<&'b str, &'a RustType>) -> bool {
        i1.into_iter().zip(i2).all(|(ty1, ty2)| matches_generic(ty1, ty2, mapping))
    }

    match (ty, generic) {
        (RustType::TypeVar(_), _) => unreachable!(),
        (ty, RustType::TypeVar(v)) => {
            mapping.insert(v, ty).map(|prev_binding| prev_binding == ty).unwrap_or(true)
        }
        (RustType::Primitive(p1), RustType::Primitive(p2)) => p1 == p2,
        (RustType::Ref(m1, t1), RustType::Ref(m2, t2)) 
        | (RustType::Raw(m1, t1), RustType::Raw(m2, t2)) => m1 == m2 && matches_generic(t1, t2, mapping),
        (RustType::Boxed(t1), RustType::Boxed(t2))
        | (RustType::Slice(t1), RustType::Slice(t2)) => matches_generic(t1, t2, mapping),
        (RustType::Dyn(_, _), RustType::Dyn(_, _)) => todo!(),
        (RustType::Tuple(tys1), RustType::Tuple(tys2)) => match_lists(tys1, tys2, mapping),
        (RustType::Adt(adt1), RustType::Adt(adt2)) => {
            // For now named generics must be in the same order
            adt1.path == adt2.path && match_lists(&adt1.generics, &adt2.generics, mapping) 
            && adt1.named_generics.len() == adt2.named_generics.len()
            && adt1.named_generics.iter().zip(adt2.named_generics.iter())
                .all(|((n1, t1), (n2, t2))| n1 == n2 && matches_generic(t1, t2, mapping))
        }
        (_, _) => false
    }
}

// Unfortunately we won't catch all unbound var errors because an undefiend type might appear first
enum SubstitutionError<'a> {
    UnboundVar(&'a str),
    UndefinedType,
}

fn map_substitute<'a>(i: impl IntoIterator<Item = &'a RustType>, mapping: &HashMap<&str, &RustType>, validate: &impl Fn(&RustType) -> bool) -> Result<Vec<RustType>, SubstitutionError<'a>> {
    i.into_iter().map(|ty| substitute_vars(ty, mapping, validate)).collect::<Result<_,_>>()
}

fn substitute_vars<'a>(ty: &'a RustType, mapping: &HashMap<&str, &RustType>, validate: &impl Fn(&RustType) -> bool) -> Result<RustType, SubstitutionError<'a>> {
    fn ident(_: &RustType) -> bool {
        true
    }
    let ty = match ty {
        RustType::TypeVar(v) => mapping.get(v.as_str()).map_or(Err(SubstitutionError::UnboundVar(v.as_str())), |ty| Ok((*ty).to_owned()))?,
        p @ RustType::Primitive(_) => p.to_owned(),
        RustType::Ref(m, t) => RustType::Ref(*m, Box::new(substitute_vars(t, mapping, &ident)?)),
        RustType::Raw(m, t) => RustType::Raw(*m, Box::new(substitute_vars(t, mapping, &ident)?)),
        RustType::Boxed(t) => RustType::Boxed(Box::new(substitute_vars(t, mapping, &ident)?)),
        RustType::Slice(t) => RustType::Slice(Box::new(substitute_vars(t, mapping, &ident)?)),
        RustType::Dyn(_, _) => todo!(),
        RustType::Tuple(tys) => RustType::Tuple(map_substitute(tys, mapping, &ident)?),
        RustType::Adt(RustPathAndGenerics { path, generics, named_generics }) => {
            RustType::Adt(RustPathAndGenerics { 
                path: path.to_owned(), 
                generics: map_substitute(generics, mapping, &ident)?, 
                named_generics: named_generics.iter().map(|(name, ty)| {
                    substitute_vars(ty, mapping, &ident).map(|ty| (name.to_owned(), ty))
                }).collect::<Result<_,_>>()?,
            })
        }
    };
    validate(&ty).then_some(ty).ok_or(SubstitutionError::UndefinedType)
}

fn substitute_method_vars<'a>(m: &'a ZngurMethodDetails, mapping: &HashMap<&str, &RustType>, validate: &impl Fn(&RustType) -> bool) -> Result<ZngurMethodDetails, SubstitutionError<'a>> {
    Ok(ZngurMethodDetails { 
        data: ZngurMethod { 
            name: m.data.name.to_owned(), 
            generics: map_substitute(&m.data.generics, mapping, validate)?, 
            receiver: m.data.receiver, 
            inputs: map_substitute(&m.data.inputs, mapping, validate)?, 
            output: substitute_vars(&m.data.output, mapping, validate)?,
        }, 
        use_path: m.use_path.to_owned(), 
        deref: m.deref.as_ref().map(|ty| substitute_vars(&ty, mapping, validate)).transpose()?,
    })
}

fn augment_type_with_impls(mut ty: ZngurType, impls: &[ZngurType], defined_types: &HashSet<RustType>) -> ZngurType {
    fn validate(defined_types: &HashSet<RustType>) -> impl Fn(&RustType) -> bool {
        |ty| {  
            let ty = match ty {
                RustType::Raw(_, ty)
                | RustType::Ref(_, ty) => ty,
                ty => ty
            };
            match ty {
                RustType::Primitive(_) => true,
                ty => defined_types.contains(ty),
            }
        }
    }

    for zng_impl in impls {
        let mut mapping = HashMap::new();
        if !matches_generic(&ty.ty, &zng_impl.ty, &mut mapping) {
            continue;
        }
        
        // For these we just choose the first match layout. Worst case is a compile error
        ty.layout = ty.layout.or(zng_impl.layout);
        if ty.cpp_ref.is_none() {
            ty.cpp_ref = zng_impl.cpp_ref.clone()
        }
        if ty.cpp_value.is_none() {
            ty.cpp_value= zng_impl.cpp_value.clone()
        }
        for t in &zng_impl.wellknown_traits {
            if !ty.wellknown_traits.contains(t) {
                ty.wellknown_traits.push(*t);
            }
        }
        for m in &zng_impl.methods {
            if ty.methods.iter().any(|x| x.data.name == m.data.name) {
                continue;
            }
            match substitute_method_vars(m, &mapping, &validate(defined_types)){
                Ok(m) => ty.methods.push(m),
                // Do nothing if we reference an undefined type in an `impl`
                Err(SubstitutionError::UndefinedType) => (),
                Err(SubstitutionError::UnboundVar(v)) => panic!("Failed to substitute type variable {} in method {} in impl {} for type {}", v, m.data.name, zng_impl.ty, ty.ty)
            }
        }
        for c in &zng_impl.constructors {
            if ty.constructors.iter().any(|x| x.name == c.name) {
                continue;
            }
            let new_inputs: Result<Vec<_>, _> = c.inputs.iter().map(|(name, ty)| substitute_vars(ty, &mapping, &validate(defined_types)).map(|ty| (name.to_owned(), ty))).collect();
            match new_inputs {
                Ok(inputs) => ty.constructors.push(ZngurConstructor { name: c.name.to_owned(), inputs }),
                Err(SubstitutionError::UndefinedType) => (),
                Err(SubstitutionError::UnboundVar(v)) => panic!("Failed to substitute type variable {} in constructor {:?} in impl {} for type {}", v, c.name, zng_impl.ty, ty.ty)
            }
        }
    }
    ty
}
