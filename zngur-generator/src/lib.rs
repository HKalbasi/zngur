use cpp::cpp_handle_keyword;
use cpp::CppFile;
use cpp::CppFnSig;
use cpp::CppMethod;
use cpp::CppMethodKind;
use cpp::CppTraitDefinition;
use cpp::CppTraitMethod;
use cpp::CppTypeDefinition;
use iter_tools::Itertools;
pub use rust::{RustTrait, RustType};

pub mod cpp;
mod rust;

pub use rust::RustFile;

pub enum ZngurMethodReceiver {
    Static,
    Ref,
    RefMut,
    Move,
}

pub struct ZngurMethod {
    pub name: String,
    pub generics: Vec<RustType>,
    pub receiver: ZngurMethodReceiver,
    pub inputs: Vec<RustType>,
    pub output: RustType,
}

pub struct ZngurConstructor {
    pub name: String,
    pub inputs: Vec<(String, RustType)>,
}

impl From<&str> for ZngurConstructor {
    fn from(value: &str) -> Self {
        let (name, fields) = value.split_once("{").unwrap();
        ZngurConstructor {
            name: name.trim().to_owned(),
            inputs: fields
                .strip_suffix("}")
                .unwrap()
                .split(",")
                .map(|x| x.trim())
                .filter(|x| !x.is_empty())
                .map(|x| {
                    let (name, ty) = x.split_once(":").unwrap();
                    (name.trim().to_owned(), RustType::from(ty))
                })
                .collect(),
        }
    }
}

pub struct ZngurType {
    pub ty: RustType,
    pub size: usize,
    pub align: usize,
    pub is_copy: bool,
    pub methods: Vec<ZngurMethod>,
    pub constructors: Vec<ZngurConstructor>,
}

pub struct ZngurTrait {
    pub tr: RustTrait,
    pub methods: Vec<ZngurMethod>,
}

pub struct ZngurFile {
    pub types: Vec<ZngurType>,
    pub traits: Vec<ZngurTrait>,
}

impl ZngurFile {
    pub fn render(self) -> (String, String) {
        let mut cpp_file = CppFile::default();
        let mut rust_file = RustFile::default();
        for ty_def in self.types {
            rust_file.add_static_size_assert(&ty_def.ty, ty_def.size);
            rust_file.add_static_align_assert(&ty_def.ty, ty_def.align);
            let mut cpp_methods = vec![];
            for constructor in ty_def.constructors {
                let rust_link_name = rust_file.add_constructor(
                    &format!("{}::{}", ty_def.ty, constructor.name),
                    constructor.inputs.iter().map(|x| &*x.0),
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(&constructor.name).to_owned(),
                    kind: CppMethodKind::StaticOnly,
                    sig: CppFnSig {
                        rust_link_name,
                        inputs: constructor.inputs.iter().map(|x| x.1.into_cpp()).collect(),
                        output: ty_def.ty.into_cpp(),
                    },
                });
            }
            for method in ty_def.methods {
                let receiver_type = match method.receiver {
                    ZngurMethodReceiver::Static => None,
                    ZngurMethodReceiver::Ref => Some(RustType::Ref(Box::new(ty_def.ty.clone()))),
                    ZngurMethodReceiver::RefMut => Some(RustType::Ref(Box::new(ty_def.ty.clone()))),
                    ZngurMethodReceiver::Move => Some(ty_def.ty.clone()),
                };
                let inputs = receiver_type
                    .into_iter()
                    .chain(method.inputs)
                    .map(|x| x.into_cpp())
                    .collect_vec();
                let rust_link_name = rust_file.add_function(
                    &format!(
                        "{}::{}::<{}>",
                        ty_def.ty,
                        method.name,
                        method.generics.iter().join(", ")
                    ),
                    inputs.len(),
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(&method.name).to_owned(),
                    kind: match method.receiver {
                        ZngurMethodReceiver::Static => CppMethodKind::StaticOnly,
                        ZngurMethodReceiver::Ref | ZngurMethodReceiver::RefMut => {
                            CppMethodKind::Lvalue
                        }
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
                from_function: if let RustType::Boxed(b) = &ty_def.ty {
                    if let RustType::Dyn(rust::RustTrait::Fn {
                        name,
                        inputs,
                        output,
                    }) = b.as_ref()
                    {
                        let rust_link_name = rust_file.add_builder_for_dyn_fn(name, inputs, output);
                        Some(cpp::BuildFromFunction {
                            sig: CppFnSig {
                                rust_link_name,
                                inputs: inputs.iter().map(|x| x.into_cpp()).collect(),
                                output: output.into_cpp(),
                            },
                        })
                    } else {
                        None
                    }
                } else {
                    None
                },
            });
        }
        for tr in self.traits {
            cpp_file.trait_defs.push(CppTraitDefinition {
                as_ty: tr.tr.into_cpp_type(),
                methods: tr
                    .methods
                    .into_iter()
                    .map(|x| CppTraitMethod {
                        name: x.name,
                        inputs: x.inputs.into_iter().map(|x| x.into_cpp()).collect(),
                        output: x.output.into_cpp(),
                    })
                    .collect(),
            })
        }
        (rust_file.0, cpp_file.render())
    }
}
