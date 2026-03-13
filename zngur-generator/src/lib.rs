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
use indexmap::map::Entry;
use itertools::Itertools;
use rust::IntoCpp;

pub mod cpp;
mod rust;
mod template;

pub use rust::RustFile;
use sailfish::Template;
pub use zngur_parser::{ParseResult, ParsedZngFile, cfg};

pub use zngur_def::*;

use crate::template::ZngHeaderTemplate;

pub struct ZngurGenerator(pub ZngurSpec);

impl ZngurGenerator {
    pub fn build_from_zng(zng: ZngurSpec) -> Self {
        ZngurGenerator(zng)
    }

    pub fn render(self, zng_header_in_place: bool) -> (String, String, Option<String>) {
        let zng = self.0;
        let mut cpp_file = CppFile::default();
        cpp_file.header_file_name = zng.cpp_include_header_name.clone();
        cpp_file.additional_includes = zng.additional_includes.0;
        cpp_file.zng_header_in_place = zng_header_in_place;
        for module in &zng.imported_modules {
            cpp_file
                .additional_includes
                .push_str(&format!("\n#include \"{}.h\"", module.path.display()));
        }
        let mut module_namespaces = std::collections::HashMap::new();
        for module in &zng.imported_modules {
            if let Some(ns) = &module.cpp_namespace {
                module_namespaces.insert(module.alias.clone(), ns.clone());
            }
        }
        let default_ns = zng.cpp_namespace.as_deref().unwrap_or("rust");
        let ctx = rust::GeneratorContext {
            module_namespaces: &module_namespaces,
            default_namespace: default_ns,
        };
        let mut rust_file = RustFile::new(&zng.mangling_base);
        rust_file.panic_to_exception = zng.convert_panic_to_exception.0;
        cpp_file.trait_defs = zng
            .traits
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    rust_file.add_builder_for_dyn_trait(value, &ctx),
                )
            })
            .collect();
        cpp_file.panic_to_exception = zng.convert_panic_to_exception.0;
        cpp_file
            .rust_cfg_defines
            .extend(zng.rust_cfg.iter().map(|(key, value)| {
                format!(
                    "ZNGUR_CFG_{}{}",
                    key.to_uppercase(),
                    value
                        .as_ref()
                        .and_then(|value| if value.trim().is_empty() {
                            None
                        } else {
                            Some(format!(
                                "_{}",
                                value
                                    .chars()
                                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                                    .collect::<String>()
                                    .to_uppercase()
                            ))
                        })
                        .unwrap_or_default()
                )
            }));
        for ty_def in zng.types {
            let ty = &ty_def.ty;
            let is_copy = ty_def.wellknown_traits.contains(&ZngurWellknownTrait::Copy);
            match ty_def.layout {
                LayoutPolicy::StackAllocated { size, align } => {
                    rust_file.add_static_size_assert(&ty, size);
                    rust_file.add_static_align_assert(&ty, align);
                }
                LayoutPolicy::Conservative { size, align } => {
                    rust_file.add_static_size_upper_bound_assert(&ty, size);
                    rust_file.add_static_align_upper_bound_assert(&ty, align);
                }
                LayoutPolicy::HeapAllocated => (),
                LayoutPolicy::OnlyByRef => (),
            }
            if is_copy {
                rust_file.add_static_is_copy_assert(&ty);
            }
            let mut cpp_methods = vec![];
            let mut constructors = vec![];
            let mut fields = vec![];
            let mut wellknown_traits = vec![];
            for constructor in ty_def.constructors {
                match constructor.name {
                    Some(name) => {
                        let rust_link_names = rust_file
                            .add_constructor(&format!("{}::{}", ty, name), &constructor.inputs);
                        cpp_methods.push(CppMethod {
                            name: cpp_handle_keyword(&name).to_owned(),
                            kind: ZngurMethodReceiver::Static,
                            sig: CppFnSig {
                                rust_link_name: rust_link_names.constructor,
                                inputs: constructor
                                    .inputs
                                    .iter()
                                    .map(|x| x.1.into_cpp(&ctx))
                                    .collect(),
                                output: ty.into_cpp(&ctx),
                            },
                        });
                        cpp_methods.push(CppMethod {
                            name: format!("matches_{}", name),
                            kind: ZngurMethodReceiver::Ref(Mutability::Not),
                            sig: CppFnSig {
                                rust_link_name: rust_link_names.match_check,
                                inputs: vec![ty.into_cpp(&ctx).into_ref()],
                                output: CppType::from("uint8_t"),
                            },
                        });
                    }
                    None => {
                        let rust_link_name = rust_file
                            .add_constructor(&format!("{}", ty), &constructor.inputs)
                            .constructor;
                        constructors.push(CppFnSig {
                            rust_link_name,
                            inputs: constructor
                                .inputs
                                .iter()
                                .map(|x| x.1.into_cpp(&ctx))
                                .collect(),
                            output: ty.into_cpp(&ctx),
                        });
                    }
                }
            }
            for field in ty_def.fields {
                let extern_mn = rust_file.add_field_assertions(&field, &ty_def.ty);
                let field = ZngurFieldData {
                    name: field.name,
                    ty: field.ty,
                    offset: match field.offset {
                        Some(offset) => ZngurFieldDataOffset::Offset(offset),
                        None => ZngurFieldDataOffset::Auto(
                            extern_mn.expect("auto offset did not provide extern name"),
                        ),
                    },
                };
                fields.push(field);
            }
            if let RustType::Tuple(fields) = &ty_def.ty {
                if !fields.is_empty() {
                    let rust_link_name = rust_file.add_tuple_constructor(&fields);
                    constructors.push(CppFnSig {
                        rust_link_name,
                        inputs: fields.iter().map(|x| x.into_cpp(&ctx)).collect(),
                        output: ty.into_cpp(&ctx),
                    });
                }
            }
            let is_unsized = ty_def
                .wellknown_traits
                .contains(&ZngurWellknownTrait::Unsized);
            for wellknown_trait in ty_def.wellknown_traits {
                let data = rust_file.add_wellknown_trait(&ty, wellknown_trait, is_unsized);
                wellknown_traits.push(data);
            }
            for method_details in ty_def.methods {
                let ZngurMethodDetails {
                    data: method,
                    use_path,
                    deref,
                } = method_details;
                let rusty_inputs = real_inputs_of_method(&method, &ty);

                let sig = rust_file.add_function(
                    &format!(
                        "<{}>::{}::<{}>",
                        deref.as_ref().map(|x| &x.0).unwrap_or(&ty),
                        method.name,
                        method.generics.iter().join(", "),
                    ),
                    &rusty_inputs,
                    &method.output,
                    use_path,
                    deref.map(|x| x.1),
                    &ctx,
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(&method.name).to_owned(),
                    kind: method.receiver,
                    sig,
                });
            }
            cpp_file.type_defs.push(CppTypeDefinition {
                ty: ty.into_cpp(&ctx),
                layout: rust_file.add_layout_policy_shim(&ty, ty_def.layout),
                constructors,
                fields,
                methods: cpp_methods,
                wellknown_traits,
                cpp_value: ty_def.cpp_value.map(|mut cpp_value| {
                    cpp_value.0 = rust_file.add_cpp_value_bridge(&ty, &cpp_value.0);
                    cpp_value
                }),
                cpp_ref: ty_def.cpp_ref,
                from_trait: if let RustType::Boxed(b) = &ty {
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
                                        inputs: inputs.iter().map(|x| x.into_cpp(&ctx)).collect(),
                                        output: output.into_cpp(&ctx),
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
                from_trait_ref: if let RustType::Dyn(tr, _) = &ty {
                    Some(tr.clone())
                } else {
                    None
                },
            });
        }
        for func in zng.funcs {
            let sig = rust_file.add_function(
                &func.path.to_string(),
                &func.inputs,
                &func.output,
                None,
                None,
                &ctx,
            );
            cpp_file.fn_defs.push(CppFnDefinition {
                name: CppPath::from_rust_path(&func.path.path, default_ns),
                sig,
            });
        }
        for func in zng.extern_cpp_funcs {
            let rust_link_name =
                rust_file.add_extern_cpp_function(&func.name, &func.inputs, &func.output);
            cpp_file.exported_fn_defs.push(CppExportedFnDefinition {
                name: func.name.clone(),
                sig: CppFnSig {
                    rust_link_name,
                    inputs: func.inputs.into_iter().map(|x| x.into_cpp(&ctx)).collect(),
                    output: func.output.into_cpp(&ctx),
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
                tr: impl_block.tr.map(|x| x.into_cpp(&ctx)),
                ty: impl_block.ty.into_cpp(&ctx),
                methods: impl_block
                    .methods
                    .iter()
                    .zip(&rust_link_names)
                    .map(|(method, link_name)| {
                        let inputs = real_inputs_of_method(method, &impl_block.ty);
                        let inputs = inputs.iter().map(|ty| ty.into_cpp(&ctx)).collect();
                        (
                            cpp_handle_keyword(&method.name).to_owned(),
                            CppFnSig {
                                rust_link_name: link_name.clone(),
                                inputs,
                                output: method.output.into_cpp(&ctx),
                            },
                        )
                    })
                    .collect(),
            });
        }
        let (h, cpp) = cpp_file.render(&ctx);
        (rust_file.text, h, cpp)
    }
}

pub struct ZngHeaderGenerator {
    pub panic_to_exception: bool,
}

impl ZngHeaderGenerator {
    /// Renders the zngur.h header
    pub fn render(&self) -> String {
        let zng_h = ZngHeaderTemplate {
            panic_to_exception: self.panic_to_exception,
        };
        zng_h.render().unwrap()
    }
}

fn real_inputs_of_method(method: &ZngurMethod, ty: &RustType) -> Vec<RustType> {
    let receiver_type = match method.receiver {
        ZngurMethodReceiver::Static => None,
        ZngurMethodReceiver::Ref(m) => Some(RustType::Ref(m, Box::new(ty.clone()))),
        ZngurMethodReceiver::Move => Some(ty.clone()),
    };
    let rusty_inputs = receiver_type
        .into_iter()
        .chain(method.inputs.clone())
        .collect::<Vec<_>>();
    rusty_inputs
}
