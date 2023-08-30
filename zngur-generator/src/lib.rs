use std::collections::HashMap;

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
use parser::Mutability;
use rust::RustPathAndGenerics;
pub use rust::{RustTrait, RustType};

pub mod cpp;
mod parser;
mod rust;

pub use parser::ParsedZngFile;
pub use rust::RustFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZngurMethodReceiver {
    Static,
    Ref(Mutability),
    Move,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurMethod {
    pub name: String,
    pub generics: Vec<RustType>,
    pub receiver: ZngurMethodReceiver,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurFn {
    pub path: RustPathAndGenerics,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurExternCppFn {
    pub name: String,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

pub struct ZngurConstructor {
    pub name: String,
    pub inputs: Vec<(String, RustType)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZngurWellknownTrait {
    Debug,
    Drop,
    Unsized,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ZngurWellknownTraitData {
    Debug {
        pretty_print: String,
        debug_print: String,
    },
    Drop {
        drop_in_place: String,
    },
    Unsized,
}

pub struct ZngurType {
    pub ty: RustType,
    pub size: usize,
    pub align: usize,
    pub is_copy: bool,
    pub wellknown_traits: Vec<ZngurWellknownTrait>,
    pub methods: Vec<(ZngurMethod, Option<Vec<String>>)>,
    pub constructors: Vec<ZngurConstructor>,
}

pub struct ZngurTrait {
    pub tr: RustTrait,
    pub methods: Vec<ZngurMethod>,
}

#[derive(Default)]
pub struct ZngurFile {
    pub types: Vec<ZngurType>,
    pub traits: HashMap<RustTrait, ZngurTrait>,
    pub funcs: Vec<ZngurFn>,
    pub extern_cpp_funcs: Vec<ZngurExternCppFn>,
}

impl ZngurFile {
    pub fn build_from_zng(zng: ParsedZngFile<'_>) -> Self {
        zng.into_zngur_file()
    }

    pub fn render(self) -> (String, String, Option<String>) {
        let mut cpp_file = CppFile::default();
        let mut rust_file = RustFile::default();
        for ty_def in self.types {
            let is_unsized = ty_def
                .wellknown_traits
                .contains(&ZngurWellknownTrait::Unsized);
            if !is_unsized {
                rust_file.add_static_size_assert(&ty_def.ty, ty_def.size);
                rust_file.add_static_align_assert(&ty_def.ty, ty_def.align);
            }
            let mut cpp_methods = vec![];
            let mut wellknown_traits = vec![];
            for constructor in ty_def.constructors {
                let rust_link_names = rust_file.add_constructor(
                    &format!("{}::{}", ty_def.ty, constructor.name),
                    constructor.inputs.iter().map(|x| &*x.0),
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(&constructor.name).to_owned(),
                    kind: CppMethodKind::StaticOnly,
                    sig: CppFnSig {
                        rust_link_name: rust_link_names.constructor,
                        inputs: constructor.inputs.iter().map(|x| x.1.into_cpp()).collect(),
                        output: ty_def.ty.into_cpp(),
                    },
                });
                cpp_methods.push(CppMethod {
                    name: format!("matches_{}", cpp_handle_keyword(&constructor.name)),
                    kind: CppMethodKind::Lvalue,
                    sig: CppFnSig {
                        rust_link_name: rust_link_names.match_check,
                        inputs: vec![ty_def.ty.into_cpp().into_ref()],
                        output: CppType::from("uint8_t"),
                    },
                });
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
                methods: cpp_methods,
                wellknown_traits,
                from_trait: if let RustType::Boxed(b) = &ty_def.ty {
                    if let RustType::Dyn(tr, _) = b.as_ref() {
                        match tr {
                            RustTrait::Normal(_) => {
                                if let Some(ztr) = self.traits.get(tr) {
                                    let link_name = rust_file.add_builder_for_dyn_trait(ztr);
                                    Some(CppTraitDefinition::Normal {
                                        as_ty: ztr.tr.into_cpp_type(),
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
        for func in self.funcs {
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
        for func in self.extern_cpp_funcs {
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
