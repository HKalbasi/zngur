use cpp::CppExportedFnDefinition;
use cpp::CppExportedImplDefinition;
use cpp::CppFile;
use cpp::CppFnDefinition;
use cpp::CppFnSig;
use cpp::CppMethod;
use cpp::CppPath;
use cpp::CppTraitDefinition;
use cpp::CppTypeDefinition;
use cpp::CppVariant;
use cpp::cpp_handle_keyword;
use indexmap::map::Entry;
use itertools::Itertools;
use rust::CoroSupport;
use rust::IntoCpp;

pub mod cpp;
mod rust;
mod template;

use askama::Template;
pub use rust::RustFile;
pub use zngur_parser::{
    ParseReport, ParseResult, ParsedZngFile, ReportEntry, ReportSink, ReportSpan, SourceCache,
    SourceId, StdErrReportSink, cfg,
};

pub use zngur_def::*;

use crate::template::ZngHeaderTemplate;

#[derive(Default)]
struct CppModuleTree {
    children: indexmap::IndexMap<String, CppModuleTree>,
    content: String,
}

impl CppModuleTree {
    fn insert(&mut self, path: &[String], content: &str) {
        match path.split_first() {
            None => self.content.push_str(content),
            Some((head, rest)) => self
                .children
                .entry(head.clone())
                .or_default()
                .insert(rest, content),
        }
    }

    fn render(&self) -> String {
        let mut out = self.content.clone();
        for (name, child) in &self.children {
            out.push_str(&format!("\npub mod {name} {{\n{}\n}}\n", child.render()));
        }
        out
    }
}

pub struct ZngurGenerator(pub ZngurSpec, pub String);

impl ZngurGenerator {
    pub fn build_from_zng(zng: ZngurSpec, crate_name: String) -> Self {
        ZngurGenerator(zng, crate_name)
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
        let default_ns = zng.cpp_namespace.as_deref().unwrap_or("rust");
        let sanitized_crate_name = self.1.replace('-', "_");
        let mut rust_file = RustFile::new(&zng.mangling_base);
        rust_file.panic_to_exception = zng.convert_panic_to_exception.0;
        let coro_support = CoroSupport::from_types(
            zng.types.iter().map(|td| &td.ty),
            &zng.mangling_base,
            default_ns,
            &sanitized_crate_name,
        );
        if let Some(coro) = &coro_support {
            rust_file.add_coro_support(coro);
        }
        cpp_file.trait_defs = zng
            .traits
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    rust_file.add_builder_for_dyn_trait(value, default_ns, &sanitized_crate_name),
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
        let mut cpp_tree = CppModuleTree::default();
        cpp_file.coro_support = coro_support;
        // Flat instantiation names (`unique_ptr<Foo>` -> `unique_ptr_Foo`)
        // back the generated item names that cannot spell `<...>`; refuse
        // to silently merge two different instantiations that flatten alike.
        let mut cpp_mangled_names = std::collections::HashMap::new();
        // Generic structs and their `...Specializations` traits are emitted
        // once per C++ template, with one `Data` plus trait impl per
        // instantiation. All behavior impls are per-instantiation too, so
        // one template freely serves stack, heap, and borrowed layouts.
        let mut cpp_emitted_templates = std::collections::HashSet::new();
        for ty_def in zng.types {
            let ty = &ty_def.ty;
            let is_copy = ty_def.wellknown_traits.contains(&ZngurWellknownTrait::Copy);
            let Some(layout) = ty_def.layout else {
                unreachable!("Every type should have a defined layout policy before rendering")
            };
            match layout {
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
            let (type_name, is_cpp, module_path): (String, bool, Vec<String>) = match &ty {
                RustType::Cpp(pg) => {
                    let module_path = pg
                        .path
                        .split_last()
                        .expect("Cpp path must have at least one segment")
                        .1;
                    // Flat for every `c++::` type: plain paths reduce to
                    // their last segment, instantiations to the flattened
                    // suffix. The generic spelling from `Display` is used
                    // wherever real Rust code names the type.
                    (pg.flat_item_name(), true, module_path.to_vec())
                }
                _ => (
                    ty.to_string().split("::").last().unwrap().to_string(),
                    false,
                    vec![],
                ),
            };
            if is_cpp
                && let Some(previous) = cpp_mangled_names
                    .insert((module_path.clone(), type_name.clone()), format!("{ty:?}"))
                && previous != format!("{ty:?}")
            {
                panic!(
                    "c++ types {previous} and {ty:?} share generated item {type_name}; rename one of them",
                );
            }
            // Only C++-originated types need a generated Rust-side struct:
            // normal Rust types are defined by the user. The shape follows
            // the normal layout policy; the backing C++ type is derived
            // from the `c++::` segments at generation time.
            if is_cpp {
                // Single-argument templates share one generic struct plus
                // its `...Specializations` trait per template; each layout
                // adds its own `Data` plus trait impl below. `Data` is the
                // instantiation's inline bytes (stack), its heap box
                // (heap), or `()` (borrowed: the C++ side owns it).
                let single_template = match ty {
                    RustType::Cpp(pg) if pg.generics.len() == 1 && pg.named_generics.is_empty() => {
                        let struct_name = pg.path.last().expect("Cpp path is not empty").clone();
                        Some((
                            pg,
                            struct_name.clone(),
                            format!("{struct_name}Specializations"),
                            if module_path.is_empty() {
                                String::new()
                            } else {
                                format!("{}::", module_path.join("::"))
                            },
                        ))
                    }
                    _ => None,
                };
                if let Some((pg, struct_name, trait_name, _)) = &single_template {
                    if cpp_emitted_templates.insert(pg.path.clone()) {
                        cpp_tree.insert(
                            &module_path,
                            &format!(
                                r#"
    // `#[allow]` because the trait and struct deliberately mirror the
    // C++ template's own `snake_case` name.
    #[allow(non_camel_case_types)]
    pub trait {trait_name} {{
        type Data;
        /// Destroy the object. The shared `Drop` below calls this, so
        /// it must destroy exactly once; calling it directly and then
        /// dropping destroys twice. Borrowed types implement it as a
        /// no-op (nothing is owned).
        unsafe fn destruct(data: *mut Self::Data);
    }}

    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct {struct_name}<T: {trait_name}>(T::Data);

    // `Drop` cannot be specialized per instantiation, so it stays
    // generic and routes through the hook; markers are per
    // instantiation (stack only), so heap and borrowed stay unmarked
    // and mix with stack freely.
    impl<T: {trait_name}> Drop for {struct_name}<T> {{
        fn drop(&mut self) {{
            unsafe {{
                T::destruct((self as *mut Self as *mut u8) as *mut T::Data)
            }}
        }}
    }}
"#,
                            ),
                        );
                    }
                }
                match layout {
                    LayoutPolicy::StackAllocated { size, align }
                    | LayoutPolicy::Conservative { size, align } => {
                        let destructor_name = format!("_zngur_crate_{type_name}_destructor");
                        let mangled_name = rust_file.add_extern_cpp_function(
                            &destructor_name,
                            &[RustType::Ref(Mutability::Mut, Box::new(ty.clone()))],
                            &RustType::Tuple(vec![]),
                            true,
                        );
                        if let Some((pg, struct_name, trait_name, mod_prefix)) = &single_template {
                            // One generic struct `Name<T: NameSpecializations>(T::Data)`
                            // per template (emitted above), one `Data` plus
                            // trait impl per instantiation. Behavior impls
                            // are per-instantiation too, so one template
                            // freely serves every layout. Symbols and the
                            // C++ side keep the flat name; only Rust code
                            // spells `<...>`.
                            let data_name = format!("{type_name}_Data");
                            let arg = &pg.generics[0];
                            // The wrapper is `#[repr(C)]` over exactly
                            // `Data`, so its address is the data address.
                            let concrete = format!("{mod_prefix}{struct_name}<{arg}>");
                            cpp_tree.insert(
                                &module_path,
                                &format!(
                                    r#"
    #[repr(C)]
    #[repr(align({align}))]
    pub struct {data_name} {{
        buffer: ::core::cell::UnsafeCell<::core::mem::MaybeUninit<[u8; {size}]>>,
        _no_auto_traits: ::core::marker::PhantomData<*mut ()>,
        _pinned: ::core::marker::PhantomPinned,
    }}
"#
                                ),
                            );
                            cpp_tree.insert(
                                &[],
                                &format!(
                                    r#"impl {mod_prefix}{trait_name} for {arg} {{
    type Data = {mod_prefix}{data_name};
    unsafe fn destruct(data: *mut Self::Data) {{
        unsafe extern "C" {{
            fn {mangled_name}(i0: *mut u8, o: *mut u8);
        }}
        let mut dummy = ();
        unsafe {{
            {mangled_name}(data as *mut u8, &mut dummy as *mut () as *mut u8);
        }}
    }}
}}

    unsafe impl ::zngur_lib::ZngCppObject for {concrete} {{}}
    unsafe impl ::zngur_lib::ZngCppStackObject for {concrete} {{}}

    unsafe impl ::zngur_lib::ZngCppDestruct for {concrete} {{
        unsafe fn destruct(&mut self) {{
            unsafe extern "C" {{
                fn {mangled_name}(i0: *mut u8, o: *mut u8);
            }}
            let mut dummy = ();
            unsafe {{
                {mangled_name}(self as *mut Self as *mut u8, &mut dummy as *mut () as *mut u8);
            }}
        }}
    }}
"#
                                ),
                            );
                        } else {
                            cpp_tree.insert(
                                &module_path,
                                &format!(
                                    r#"
    #[repr(C)]
    #[repr(align({align}))]
    pub struct {type_name} {{
      pub(crate) buffer: core::cell::UnsafeCell<core::mem::MaybeUninit<[u8; {size}]>>,
      _no_auto_traits: core::marker::PhantomData<*mut ()>,
      _pinned: core::marker::PhantomPinned,
    }}

    unsafe impl ::zngur_lib::ZngCppObject for {type_name} {{}}
    unsafe impl ::zngur_lib::ZngCppStackObject for {type_name} {{}}

    unsafe impl ::zngur_lib::ZngCppDestruct for {type_name} {{
        unsafe fn destruct(&mut self) {{
            unsafe extern "C" {{
                fn {mangled_name}(i0: *mut u8, o: *mut u8);
            }}
            let mut dummy = ();
            unsafe {{
                {mangled_name}(self.buffer.get() as *mut _, &mut dummy as *mut () as *mut u8);
            }}
        }}
    }}

    impl Drop for {type_name} {{
        fn drop(&mut self) {{
            use ::zngur_lib::ZngCppDestruct;
            unsafe {{
                self.destruct();
            }}
        }}
    }}
"#
                                ),
                            );
                        }
                    }
                    LayoutPolicy::HeapAllocated => {
                        if let Some((pg, _, trait_name, mod_prefix)) = &single_template {
                            // The box has the same shape for every
                            // instantiation; only the constructor and the
                            // stored destructor vary, and both are already
                            // per-instantiation. The shared `Drop` destroys
                            // through the hook below, exactly once.
                            let data_name = format!("{type_name}_Data");
                            let arg = &pg.generics[0];
                            cpp_tree.insert(
                                &module_path,
                                &format!(
                                    r#"
    #[repr(C)]
    pub struct {data_name} {{
        pub(crate) data: *mut u8,
        pub(crate) destructor: extern "C" fn(*mut u8),
    }}
"#
                                ),
                            );
                            cpp_tree.insert(
                                &[],
                                &format!(
                                    r#"impl {mod_prefix}{trait_name} for {arg} {{
    type Data = {mod_prefix}{data_name};
    unsafe fn destruct(data: *mut Self::Data) {{
        unsafe {{
            let boxed = &*data;
            (boxed.destructor)(boxed.data)
        }}
    }}
}}
"#
                                ),
                            );
                        } else {
                            cpp_tree.insert(
                                &module_path,
                                &format!(
                                    r#"
    #[repr(C)]
    pub struct {type_name} {{
        pub(crate) data: *mut u8,
        pub(crate) destructor: extern "C" fn(*mut u8),
    }}

    impl Drop for {type_name} {{
        fn drop(&mut self) {{
            (self.destructor)(self.data)
        }}
    }}
"#
                                ),
                            );
                        }
                    }
                    LayoutPolicy::OnlyByRef => {
                        if let Some((pg, _, trait_name, mod_prefix)) = &single_template {
                            // Borrowed: nothing is owned, so `Data` is `()`
                            // and destruction is a no-op.
                            let arg = &pg.generics[0];
                            cpp_tree.insert(
                                &[],
                                &format!(
                                    r#"impl {mod_prefix}{trait_name} for {arg} {{
    type Data = ();
    unsafe fn destruct(_data: *mut Self::Data) {{}}
}}
"#
                                ),
                            );
                        } else {
                            cpp_tree.insert(
                                &module_path,
                                &format!(
                                    r#"
    pub struct {type_name}(());
"#
                                ),
                            );
                        }
                    }
                }
            }
            let mut cpp_methods = vec![];
            let mut constructor = None;
            let mut variants = vec![];
            let mut fields = vec![];
            let mut wellknown_traits = vec![];
            if let Some(constructor_def) = ty_def.constructor {
                let rust_link_name = rust_file.add_constructor(
                    &ty.to_string(),
                    constructor_def.inputs.iter().map(|(name, ty)| (name, ty)),
                );
                constructor = Some(CppFnSig {
                    rust_link_name,
                    inputs: constructor_def
                        .inputs
                        .iter()
                        .map(|x| x.1.into_cpp(default_ns, &sanitized_crate_name))
                        .collect(),
                    output: ty.into_cpp(default_ns, &sanitized_crate_name),
                });
            }
            let discriminant = if ty_def.exhaustive {
                rust_file.add_discriminant(&ty.to_string(), &ty_def.variants)
            } else {
                None
            };
            for variant in ty_def.variants {
                let rust_name = format!("{}::{}", ty, variant.name);
                let constructor = if variant.exhaustive {
                    let rust_link_name = rust_file.add_constructor(
                        &rust_name,
                        variant.fields.iter().map(|field| (&field.name, &field.ty)),
                    );
                    Some(CppFnSig {
                        rust_link_name,
                        inputs: variant
                            .fields
                            .iter()
                            .map(|x| x.ty.into_cpp(default_ns, &sanitized_crate_name))
                            .collect(),
                        output: ty
                            .into_cpp(default_ns, &sanitized_crate_name)
                            .with_tail(variant.name.clone()),
                    })
                } else {
                    None
                };
                let match_check = rust_file.add_match_check(&rust_name);
                let mut fields = vec![];
                for field in variant.fields {
                    let mn =
                        rust_file.add_variant_field_calculations(&field, &ty_def.ty, &variant.name);
                    fields.push(ZngurFieldData {
                        name: field.name,
                        ty: field.ty,
                        offset: ZngurFieldDataOffset::AutoDynamic(mn),
                    });
                }
                variants.push(CppVariant {
                    name: variant.name,
                    constructor,
                    match_check,
                    fields,
                });
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
                    constructor = Some(CppFnSig {
                        rust_link_name,
                        inputs: fields
                            .iter()
                            .map(|x| x.into_cpp(default_ns, &sanitized_crate_name))
                            .collect(),
                        output: ty.into_cpp(default_ns, &sanitized_crate_name),
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
                    cpp_name,
                } = method_details;
                let rusty_inputs = real_inputs_of_method(&method, &ty);
                let cpp_name = cpp_name.as_ref().unwrap_or(&method.name);

                let sig = rust_file.add_function(
                    cpp_name,
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
                    default_ns,
                    &sanitized_crate_name,
                );
                cpp_methods.push(CppMethod {
                    name: cpp_handle_keyword(cpp_name).to_owned(),
                    kind: method.receiver,
                    sig,
                });
            }
            let (layout_size, layout_align, is_layout_conservative) = match layout {
                LayoutPolicy::StackAllocated { size, align } => (size, align, false),
                LayoutPolicy::Conservative { size, align } => (size, align, true),
                _ => (0, 1, false),
            };
            cpp_file.type_defs.push(CppTypeDefinition {
                ty: ty.into_cpp(default_ns, &sanitized_crate_name),
                is_cpp: matches!(ty, RustType::Cpp(_)),
                rust_type_name: type_name.clone(),
                layout: rust_file.add_layout_policy_shim(&ty, layout),
                layout_size,
                layout_align,
                is_layout_conservative,
                constructor,
                variants,
                discriminant,
                fields,
                methods: cpp_methods,
                wellknown_traits,
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
                                        inputs: inputs
                                            .iter()
                                            .map(|x| x.into_cpp(default_ns, &sanitized_crate_name))
                                            .collect(),
                                        output: output.into_cpp(default_ns, &sanitized_crate_name),
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
        rust_file.text.push_str(&cpp_tree.render());
        for func in zng.funcs {
            let sig = rust_file.add_function(
                &func.path.to_string(),
                &func.path.to_string(),
                &func.inputs,
                &func.output,
                None,
                None,
                default_ns,
                &sanitized_crate_name,
            );
            cpp_file.fn_defs.push(CppFnDefinition {
                name: CppPath::from_rust_path(&func.path.path, default_ns, &sanitized_crate_name),
                sig,
            });
        }
        for func in zng.extern_cpp_funcs {
            let rust_link_name = rust_file.add_extern_cpp_function(
                &func.name,
                &func.inputs,
                &func.output,
                func.is_safe,
            );
            cpp_file.exported_fn_defs.push(CppExportedFnDefinition {
                name: func.name.clone(),
                sig: CppFnSig {
                    rust_link_name,
                    inputs: func
                        .inputs
                        .into_iter()
                        .map(|x| x.into_cpp(default_ns, &sanitized_crate_name))
                        .collect(),
                    output: func.output.into_cpp(default_ns, &sanitized_crate_name),
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
                tr: impl_block
                    .tr
                    .map(|x| x.into_cpp(default_ns, &sanitized_crate_name)),
                ty: impl_block.ty.into_cpp(default_ns, &sanitized_crate_name),
                methods: impl_block
                    .methods
                    .iter()
                    .zip(&rust_link_names)
                    .map(|(method, link_name)| {
                        let inputs = real_inputs_of_method(method, &impl_block.ty);
                        let inputs = inputs
                            .iter()
                            .map(|ty| ty.into_cpp(default_ns, &sanitized_crate_name))
                            .collect();
                        (
                            cpp_handle_keyword(&method.name).to_owned(),
                            CppFnSig {
                                rust_link_name: link_name.clone(),
                                inputs,
                                output: method.output.into_cpp(default_ns, &sanitized_crate_name),
                            },
                        )
                    })
                    .collect(),
            });
        }
        let (h, cpp) = cpp_file.render(default_ns, &sanitized_crate_name);
        (rust_file.text, h, cpp)
    }
}

pub struct ZngHeaderGenerator {
    pub panic_to_exception: bool,
    pub cpp_namespace: String,
}

impl ZngHeaderGenerator {
    /// Renders the zngur.h header
    pub fn render(&self) -> Result<String, askama::Error> {
        let zng_h = ZngHeaderTemplate {
            panic_to_exception: self.panic_to_exception,
            cpp_namespace: self.cpp_namespace.clone(),
        };
        zng_h.render()
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

#[cfg(test)]
mod tests {
    use zngur_def::*;

    use crate::ZngurGenerator;

    fn minimal_heap_allocated_type(ty: RustType) -> ZngurType {
        ZngurType {
            ty,
            layout: Some(LayoutPolicy::HeapAllocated),
            wellknown_traits: vec![],
            exhaustive: true,
            methods: vec![],
            constructor: None,
            variants: vec![],
            fields: vec![],
        }
    }

    fn minimal_stack_owned_type(ty: RustType, size: usize, align: usize) -> ZngurType {
        ZngurType {
            ty,
            layout: Some(LayoutPolicy::StackAllocated { size, align }),
            wellknown_traits: vec![],
            exhaustive: true,
            methods: vec![],
            constructor: None,
            variants: vec![],
            fields: vec![],
        }
    }

    fn minimal_only_by_ref_type(ty: RustType) -> ZngurType {
        ZngurType {
            ty,
            layout: Some(LayoutPolicy::OnlyByRef),
            wellknown_traits: vec![],
            exhaustive: true,
            methods: vec![],
            constructor: None,
            variants: vec![],
            fields: vec![],
        }
    }

    fn cpp(segments: &[&str]) -> RustType {
        RustType::Cpp(RustPathAndGenerics {
            path: segments.iter().map(|s| s.to_string()).collect(),
            generics: vec![],
            named_generics: vec![],
        })
    }

    fn cpp_with_arg(segments: &[&str], arg: RustType) -> RustType {
        RustType::Cpp(RustPathAndGenerics {
            path: segments.iter().map(|s| s.to_string()).collect(),
            generics: vec![arg],
            named_generics: vec![],
        })
    }

    #[test]
    fn cpp_heap_type_gets_nested_module() {
        let spec = ZngurSpec {
            types: vec![minimal_heap_allocated_type(cpp(&["a", "b", "Name"]))],
            ..Default::default()
        };
        let (rust_code, h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert!(rust_code.contains("pub mod a {"));
        assert!(rust_code.contains("pub mod b {"));
        assert!(rust_code.contains("pub struct Name"));
        // The C++ side names the real type directly: no `rust::` mirror of
        // it exists, but the origin trait is specialized for it.
        assert!(!h.contains("rust::a::b::Name"));
        assert!(h.contains("zngur_cpp_origin< ::a::b::Name >"));
    }

    #[test]
    fn cpp_stack_type_gets_nested_module() {
        let spec = ZngurSpec {
            types: vec![minimal_stack_owned_type(cpp(&["geo", "Point"]), 16, 8)],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert!(rust_code.contains("pub mod geo {"));
        assert!(rust_code.contains("pub struct Point"));
    }

    #[test]
    fn cpp_template_instantiation_is_generic() {
        // `c++::std::unique_ptr<c++::Foo>` becomes the generic struct
        // `std::unique_ptr<T: unique_ptrSpecializations>(T::Data)`, with the
        // storage plus trait impl per instantiation; the C++ side keeps
        // spelling the real instantiation and never forward-declares it
        // (the definition comes from the user's own headers).
        let spec = ZngurSpec {
            types: vec![minimal_stack_owned_type(
                cpp_with_arg(&["std", "unique_ptr"], cpp(&["Foo"])),
                8,
                8,
            )],
            ..Default::default()
        };
        let (rust_code, h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert!(rust_code.contains("pub mod std {"));
        assert!(
            rust_code.contains("pub struct unique_ptr<T: unique_ptrSpecializations>(T::Data);")
        );
        assert!(rust_code.contains("pub trait unique_ptrSpecializations"));
        assert!(rust_code.contains("pub struct unique_ptr_Foo_Data"));
        assert!(rust_code.contains("impl std::unique_ptrSpecializations for Foo"));
        assert!(rust_code.contains("type Data = std::unique_ptr_Foo_Data;"));
        assert!(!rust_code.contains("pub struct unique_ptr_Foo {"));
        assert!(h.contains("zngur_cpp_origin< ::std::unique_ptr< ::Foo > >"));
        assert!(
            !h.contains("struct unique_ptr;"),
            "must not forward-declare the real template in:\n{h}"
        );
    }

    #[test]
    fn cpp_template_struct_emitted_once_per_template() {
        // Two instantiations of one template share the generic struct and
        // trait, with separate storage plus trait impls.
        let spec = ZngurSpec {
            types: vec![
                minimal_stack_owned_type(cpp_with_arg(&["std", "unique_ptr"], cpp(&["Foo"])), 8, 8),
                minimal_stack_owned_type(cpp_with_arg(&["std", "unique_ptr"], cpp(&["Bar"])), 8, 8),
            ],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert_eq!(
            rust_code
                .matches("pub struct unique_ptr<T: unique_ptrSpecializations>(T::Data);")
                .count(),
            1,
            "generic struct must be emitted once in:\n{rust_code}"
        );
        assert_eq!(
            rust_code
                .matches("pub trait unique_ptrSpecializations")
                .count(),
            1,
            "trait must be emitted once in:\n{rust_code}"
        );
        assert!(rust_code.contains("pub struct unique_ptr_Foo_Data"));
        assert!(rust_code.contains("pub struct unique_ptr_Bar_Data"));
        assert!(rust_code.contains("impl std::unique_ptrSpecializations for Foo"));
        assert!(rust_code.contains("impl std::unique_ptrSpecializations for Bar"));
    }

    #[test]
    fn cpp_template_heap_uses_box_data() {
        // Heap templates share the generic struct and `Drop`; each
        // instantiation gets a box `Data` plus a trait impl whose hook
        // runs the stored destructor.
        let spec = ZngurSpec {
            types: vec![minimal_heap_allocated_type(cpp_with_arg(
                &["h", "Holder"],
                cpp(&["Foo"]),
            ))],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert!(rust_code.contains("pub struct Holder<T: HolderSpecializations>(T::Data);"));
        assert!(rust_code.contains("pub struct Holder_Foo_Data"));
        assert!(rust_code.contains("impl<T: HolderSpecializations> Drop for Holder<T>"));
        assert!(!rust_code.contains("impl Drop for Holder_Foo_Data"));
        assert!(rust_code.contains("impl h::HolderSpecializations for Foo"));
        assert!(rust_code.contains("type Data = h::Holder_Foo_Data;"));
        assert!(rust_code.contains("(boxed.destructor)(boxed.data)"));
        // No stack glue on a heap-only template: no markers, no
        // destructor wrapper.
        assert!(!rust_code.contains("ZngCppStackObject"));
        assert!(!rust_code.contains("destructor(i0"));
    }

    #[test]
    fn cpp_template_ref_uses_unit_data() {
        // Borrowed templates own nothing: `Data` is `()` and destruction
        // is a no-op, with no `Data` struct at all.
        let spec = ZngurSpec {
            types: vec![minimal_only_by_ref_type(cpp_with_arg(
                &["h", "Holder"],
                cpp(&["Foo"]),
            ))],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert!(rust_code.contains("pub struct Holder<T: HolderSpecializations>(T::Data);"));
        assert!(rust_code.contains("impl h::HolderSpecializations for Foo"));
        assert!(rust_code.contains("type Data = ();"));
        assert!(!rust_code.contains("Holder_Foo_Data"));
        // The shared `Drop` routes through the no-op hook; no markers.
        assert!(!rust_code.contains("ZngCppObject"));
        assert!(!rust_code.contains("destructor(i0"));
    }

    #[test]
    fn cpp_template_heap_and_ref_mix_freely() {
        // Heap and borrowed carry no generic impls on the wrapper, so one
        // template serves both layouts with separate `Data`s.
        let spec = ZngurSpec {
            types: vec![
                minimal_heap_allocated_type(cpp_with_arg(&["h", "Holder"], cpp(&["Foo"]))),
                minimal_only_by_ref_type(cpp_with_arg(&["h", "Holder"], cpp(&["Bar"]))),
            ],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert_eq!(
            rust_code
                .matches("pub struct Holder<T: HolderSpecializations>(T::Data);")
                .count(),
            1,
            "generic struct must be emitted once in:\n{rust_code}"
        );
        assert!(rust_code.contains("pub struct Holder_Foo_Data"));
        assert!(rust_code.contains("type Data = ();"));
    }

    #[test]
    fn cpp_template_layouts_mix_freely() {
        // `Drop` stays generic over the hook, so one template serves
        // stack, heap, and borrowed layouts at once; only the stack
        // instantiation gets the object markers.
        let spec = ZngurSpec {
            types: vec![
                minimal_stack_owned_type(cpp_with_arg(&["h", "Holder"], cpp(&["Foo"])), 8, 8),
                minimal_heap_allocated_type(cpp_with_arg(&["h", "Holder"], cpp(&["Bar"]))),
                minimal_only_by_ref_type(cpp_with_arg(&["h", "Holder"], cpp(&["Baz"]))),
            ],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert_eq!(
            rust_code
                .matches("pub struct Holder<T: HolderSpecializations>(T::Data);")
                .count(),
            1,
            "generic struct must be emitted once in:\n{rust_code}"
        );
        assert_eq!(
            rust_code
                .matches("impl<T: HolderSpecializations> Drop for Holder<T>")
                .count(),
            1,
            "generic Drop must be emitted once in:\n{rust_code}"
        );
        assert!(
            rust_code.contains("unsafe impl ::zngur_lib::ZngCppStackObject for h::Holder<Foo>")
        );
        assert!(!rust_code.contains("ZngCppStackObject for h::Holder<Bar>"));
        assert!(!rust_code.contains("ZngCppStackObject for h::Holder<Baz>"));
        assert!(rust_code.contains("pub struct Holder_Bar_Data"));
        assert!(rust_code.contains("type Data = ();"));
    }

    #[test]
    fn cpp_only_by_ref_type_gets_unit_struct_in_nested_module() {
        let spec = ZngurSpec {
            types: vec![minimal_only_by_ref_type(cpp(&["h", "Handle"]))],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert!(rust_code.contains("pub mod h {"));
        assert!(rust_code.contains("pub struct Handle(());"));
    }

    #[test]
    fn cpp_only_by_ref_type_has_single_ref_ctor_and_cpp_accessor() {
        // `c++::` reference-only types name the real C++ type directly, so
        // exactly one `Ref(const T&)`/`RefMut(const T&)` constructor may be
        // emitted, plus a `.cpp()` accessor returning the real object.
        let spec = ZngurSpec {
            types: vec![minimal_only_by_ref_type(cpp(&["h", "Handle"]))],
            ..Default::default()
        };
        let (_rust_code, h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        assert_eq!(
            h.matches("RefMut(const ::h::Handle&").count(),
            1,
            "expected exactly one RefMut ctor in:\n{h}"
        );
        assert_eq!(
            h.matches("Ref(const ::h::Handle&").count(),
            1,
            "expected exactly one Ref ctor in:\n{h}"
        );
        assert!(
            h.contains("::h::Handle& cpp()"),
            "expected a .cpp() accessor in:\n{h}"
        );
    }

    #[test]
    fn cpp_types_sharing_a_prefix_merge_into_a_single_module() {
        let spec = ZngurSpec {
            types: vec![
                minimal_heap_allocated_type(cpp(&["a", "Foo"])),
                minimal_heap_allocated_type(cpp(&["a", "Bar"])),
            ],
            ..Default::default()
        };
        let (rust_code, _h, _cpp) =
            ZngurGenerator::build_from_zng(spec, "test_crate".to_owned()).render(false);
        // Both structs must land inside a single, merged `pub mod a { ... }`.
        let mod_a_count = rust_code.matches("pub mod a {").count();
        assert_eq!(
            mod_a_count, 1,
            "expected exactly one `pub mod a {{` block, found {mod_a_count} in:\n{rust_code}"
        );
        assert!(rust_code.contains("pub struct Foo"));
        assert!(rust_code.contains("pub struct Bar"));
    }
}
