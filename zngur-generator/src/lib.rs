use cpp::cpp_handle_keyword;
use cpp::CppExportedFnDefinition;
use cpp::CppFile;
use cpp::CppFnDefinition;
use cpp::CppFnSig;
use cpp::CppMethod;
use cpp::CppMethodKind;
use cpp::CppPath;
use cpp::CppTraitDefinition;
use cpp::CppTraitMethod;
use cpp::CppType;
use cpp::CppTypeDefinition;
use iter_tools::Itertools;
use rust::IntoCpp;

pub mod cpp;
mod rust;

pub use rust::RustFile;
pub use zngur_parser::ParsedZngFile;

pub use zngur_def::*;

pub struct ZngurGenerator(ZngurFile);

impl ZngurGenerator {
    pub fn build_from_zng(zng: ParsedZngFile<'_>) -> Self {
        ZngurGenerator(zng.into_zngur_file())
    }

    pub fn render(self) -> (String, String, Option<String>) {
        let zng = self.0;
        let mut cpp_file = CppFile::default();
        let mut rust_file = RustFile::default();
        for ty_def in zng.types {
            let is_unsized = ty_def
                .wellknown_traits
                .contains(&ZngurWellknownTrait::Unsized);
            if !is_unsized {
                rust_file.add_static_size_assert(&ty_def.ty, ty_def.size);
                rust_file.add_static_align_assert(&ty_def.ty, ty_def.align);
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
                            kind: CppMethodKind::StaticOnly,
                            sig: CppFnSig {
                                rust_link_name: rust_link_names.constructor,
                                inputs: constructor.inputs.iter().map(|x| x.1.into_cpp()).collect(),
                                output: ty_def.ty.into_cpp(),
                            },
                        });
                        cpp_methods.push(CppMethod {
                            name: format!("matches_{}", cpp_handle_keyword(&name)),
                            kind: CppMethodKind::Lvalue,
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
            for wellknown_trait in ty_def.wellknown_traits {
                let data = rust_file.add_wellknown_trait(&ty_def.ty, wellknown_trait);
                wellknown_traits.push(data);
            }
            for (method, use_path) in ty_def.methods {
                let receiver_type = match method.receiver {
                    ZngurMethodReceiver::Static => None,
                    ZngurMethodReceiver::Ref(m) => {
                        Some(RustType::Ref(m, Box::new(ty_def.ty.clone())))
                    }
                    ZngurMethodReceiver::Move => Some(ty_def.ty.clone()),
                };
                let rusty_inputs = receiver_type
                    .into_iter()
                    .chain(method.inputs)
                    .collect::<Vec<_>>();
                let inputs = rusty_inputs.iter().map(|x| x.into_cpp()).collect_vec();
                let rust_link_name = rust_file.add_function(
                    &format!(
                        "<{}>::{}::<{}>",
                        ty_def.ty,
                        method.name,
                        method.generics.iter().join(", "),
                    ),
                    &rusty_inputs,
                    &method.output,
                    use_path,
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(&method.name).to_owned(),
                    kind: match method.receiver {
                        ZngurMethodReceiver::Static => CppMethodKind::StaticOnly,
                        ZngurMethodReceiver::Ref(_) => CppMethodKind::Lvalue,
                        ZngurMethodReceiver::Move => CppMethodKind::Rvalue,
                    },
                    sig: CppFnSig {
                        rust_link_name,
                        inputs,
                        output: method.output.into_cpp(),
                    },
                });
            }
            cpp_file.type_defs.push(CppTypeDefinition {
                ty: ty_def.ty.into_cpp(),
                size: ty_def.size,
                align: ty_def.align,
                is_copy: ty_def.is_copy,
                constructors,
                methods: cpp_methods,
                wellknown_traits,
                from_trait: if let RustType::Boxed(b) = &ty_def.ty {
                    if let RustType::Dyn(tr, _) = b.as_ref() {
                        match tr {
                            RustTrait::Normal(_) => {
                                if let Some(ztr) = zng.traits.get(tr) {
                                    let link_name = rust_file.add_builder_for_dyn_trait(ztr);
                                    Some(CppTraitDefinition::Normal {
                                        as_ty: ztr.tr.into_cpp(),
                                        methods: ztr
                                            .methods
                                            .clone()
                                            .into_iter()
                                            .map(|x| CppTraitMethod {
                                                name: x.name,
                                                inputs: x
                                                    .inputs
                                                    .into_iter()
                                                    .map(|x| x.into_cpp())
                                                    .collect(),
                                                output: x.output.into_cpp(),
                                            })
                                            .collect(),
                                        link_name: link_name.clone(),
                                    })
                                } else {
                                    None
                                }
                            }
                            RustTrait::Fn {
                                name,
                                inputs,
                                output,
                            } => {
                                let rust_link_name =
                                    rust_file.add_builder_for_dyn_fn(name, inputs, output);
                                Some(CppTraitDefinition::Fn {
                                    sig: CppFnSig {
                                        rust_link_name,
                                        inputs: inputs.iter().map(|x| x.into_cpp()).collect(),
                                        output: output.into_cpp(),
                                    },
                                })
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                },
            });
        }
        for func in zng.funcs {
            let rust_link_name =
                rust_file.add_function(&func.path.to_string(), &func.inputs, &func.output, None);
            cpp_file.fn_defs.push(CppFnDefinition {
                name: CppPath(func.path.path),
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
        let (h, cpp) = cpp_file.render();
        (rust_file.0, h, cpp)
    }
}
