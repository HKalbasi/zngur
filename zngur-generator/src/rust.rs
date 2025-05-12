use std::{collections::HashSet, fmt::Write};

use itertools::Itertools;

use crate::{
    ZngurTrait, ZngurWellknownTrait, ZngurWellknownTraitData,
    cpp::{CppFnSig, CppLayoutPolicy, CppPath, CppTraitDefinition, CppTraitMethod, CppType},
};

use zngur_def::*;

pub trait IntoCpp {
    fn into_cpp(&self) -> CppType;
}

impl IntoCpp for RustPathAndGenerics {
    fn into_cpp(&self) -> CppType {
        let RustPathAndGenerics {
            path,
            generics,
            named_generics,
        } = self;
        let named_generics = named_generics.iter().sorted_by_key(|x| &x.0).map(|x| &x.1);
        CppType {
            rust_equivalent: None,
            path: CppPath::from_rust_path(path),
            generic_args: generics
                .iter()
                .chain(named_generics)
                .map(|x| x.into_cpp())
                .collect(),
        }
    }
}

impl IntoCpp for RustTrait {
    fn into_cpp(&self) -> CppType {
        match self {
            RustTrait::Normal(pg) => pg.into_cpp(),
            RustTrait::Fn {
                name,
                inputs,
                output,
            } => CppType {
                rust_equivalent: None,
                path: CppPath::from(&*format!("rust::{name}")),
                generic_args: inputs
                    .iter()
                    .chain(Some(&**output))
                    .map(|x| x.into_cpp())
                    .collect(),
            },
        }
    }
}

impl IntoCpp for RustType {
    fn into_cpp(&self) -> CppType {
        fn for_builtin(this: &RustType) -> Option<CppType> {
            match this {
                RustType::Primitive(s) => Some(match s {
                    CommonPrimitiveType::Uint(s) => CppType::from(&*format!("uint{s}_t")),
                    CommonPrimitiveType::Int(s) => CppType::from(&*format!("int{s}_t")),
                    CommonPrimitiveType::Float(32) => CppType::from("float_t"),
                    CommonPrimitiveType::Float(64) => CppType::from("double_t"),
                    CommonPrimitiveType::Float(_) => unreachable!(),
                    CommonPrimitiveType::Usize => CppType::from("size_t"),
                }),
                RustType::Raw(Mutability::Mut, t) => Some(CppType::from(&*format!(
                    "{}*",
                    for_builtin(t)?.to_string().strip_prefix("::")?
                ))),
                RustType::Raw(Mutability::Not, t) => Some(CppType::from(&*format!(
                    "{} const*",
                    for_builtin(t)?.to_string().strip_prefix("::")?
                ))),
                _ => None,
            }
        }

        fn inner(this: &RustType) -> CppType {
            if let Some(builtin) = for_builtin(this) {
                return builtin;
            }
            match this {
                RustType::Primitive(_) => unreachable!(),
                RustType::Bool => CppType::from("rust::Bool"),
                RustType::Str => CppType::from("rust::Str"),
                RustType::ZngurCppOpaqueOwnedObject => {
                    CppType::from("rust::ZngurCppOpaqueOwnedObject")
                }
                RustType::Boxed(t) => CppType {
                    rust_equivalent: None,
                    path: CppPath::from("rust::Box"),
                    generic_args: vec![t.into_cpp()],
                },
                RustType::Ref(m, t) => CppType {
                    rust_equivalent: None,
                    path: match m {
                        Mutability::Mut => CppPath::from("rust::RefMut"),
                        Mutability::Not => CppPath::from("rust::Ref"),
                    },
                    generic_args: vec![t.into_cpp()],
                },
                RustType::Slice(s) => CppType {
                    rust_equivalent: None,
                    path: CppPath::from("rust::Slice"),
                    generic_args: vec![s.into_cpp()],
                },
                RustType::Raw(_, _) => todo!(),
                RustType::Adt(pg) => pg.into_cpp(),
                RustType::Tuple(v) => {
                    if v.is_empty() {
                        return CppType::from("rust::Unit");
                    }
                    CppType {
                        rust_equivalent: None,
                        path: CppPath::from("rust::Tuple"),
                        generic_args: v.into_iter().map(|x| x.into_cpp()).collect(),
                    }
                }
                RustType::Dyn(tr, marker_bounds) => {
                    let tr_as_cpp_type = tr.into_cpp();
                    CppType {
                        rust_equivalent: None,
                        path: CppPath::from("rust::Dyn"),
                        generic_args: [tr_as_cpp_type]
                            .into_iter()
                            .chain(
                                marker_bounds
                                    .iter()
                                    .map(|x| CppType::from(&*format!("rust::{x}"))),
                            )
                            .collect(),
                    }
                }
            }
        }
        let mut r = inner(self);
        r.rust_equivalent = Some(self.clone());
        r
    }
}

pub struct RustFile<'a> {
    unsized_types: &'a HashSet<RustType>,
    pub text: String,
    pub panic_to_exception: bool,
}

impl Write for RustFile<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.text.write_str(s)
    }
}

macro_rules! w {
    ($dst:expr, $($arg:tt)*) => {
        { let _ = write!($dst, $($arg)*); }
    };
}

macro_rules! wln {
    ($dst:expr, $($arg:tt)*) => {
        { let _ = writeln!($dst, $($arg)*); }
    };
}

fn mangle_name(name: &str) -> String {
    let mut name = "__zngur_"
        .chars()
        .chain(name.chars().filter(|c| !c.is_whitespace()))
        .chain(Some('_'))
        .collect::<String>();
    let bads = [
        (1, "::<", 'm'),
        (1, ">::", 'n'),
        (1, "->", 'a'),
        (2, "&", 'r'),
        (2, "=", 'e'),
        (2, "<", 'x'),
        (2, ">", 'y'),
        (2, "[", 'j'),
        (2, "]", 'k'),
        (2, "::", 's'),
        (2, ",", 'c'),
        (2, "+", 'l'),
        (2, "(", 'p'),
        (2, ")", 'q'),
    ];
    while let Some((pos, which)) = bads.iter().filter_map(|x| Some((name.find(x.1)?, x))).min() {
        name.replace_range(pos..pos + which.1.len(), "_");
        w!(name, "{}{pos}", which.2);
    }
    name
}

pub struct ConstructorMangledNames {
    pub constructor: String,
    pub match_check: String,
}

impl RustFile<'_> {
    pub fn new<'a>(unsized_types: &'a HashSet<RustType>) -> RustFile<'a> {
        RustFile {
            unsized_types,
            text: r#"
    #[allow(dead_code)]
    mod zngur_types {
        pub struct ZngurCppOpaqueBorrowedObject(());
    
        #[repr(C)]
        pub struct ZngurCppOpaqueOwnedObject {
            data: *mut u8,
            destructor: extern "C" fn(*mut u8),
        }
    
        impl ZngurCppOpaqueOwnedObject {
            pub unsafe fn new(
                data: *mut u8,
                destructor: extern "C" fn(*mut u8),            
            ) -> Self {
                Self { data, destructor }
            }
    
            pub fn ptr(&self) -> *mut u8 {
                self.data
            }
        }
    
        impl Drop for ZngurCppOpaqueOwnedObject {
            fn drop(&mut self) {
                (self.destructor)(self.data)
            }
        }
    }
    
    #[allow(unused_imports)]
    pub use zngur_types::ZngurCppOpaqueOwnedObject;
    #[allow(unused_imports)]
    pub use zngur_types::ZngurCppOpaqueBorrowedObject;
    "#
            .to_owned(),
            panic_to_exception: false,
        }
    }

    fn call_cpp_function(&mut self, name: &str, inputs: &[RustType]) {
        for (n, ty) in inputs.iter().enumerate() {
            match ty {
                RustType::Primitive(_) => (),
                _ => wln!(
                    self,
                    "#[allow(unused_mut)] let mut i{n} = ::core::mem::MaybeUninit::new(i{n});"
                ),
            }
        }
        wln!(self, "let mut r = ::core::mem::MaybeUninit::uninit();");
        w!(self, "{name}");
        for (n, ty) in inputs.iter().enumerate() {
            match ty {
                RustType::Primitive(_) => w!(self, "i{n}, "),
                RustType::Ref(_, inner) if !self.unsized_types.contains(&inner) => {
                    w!(
                        self,
                        "::std::mem::transmute::<_, *mut ::std::ffi::c_void>(i{n}), "
                    );
                }
                _ => w!(self, "i{n}.as_mut_ptr() as *mut u8, "),
            }
        }
        wln!(self, "r.as_mut_ptr() as *mut u8);");
        wln!(self, "r.assume_init()");
    }

    pub fn add_static_is_copy_assert(&mut self, ty: &RustType) {
        wln!(
            self,
            r#"const _: () = {{
                const fn static_assert_is_copy<T: Copy>() {{}}
                static_assert_is_copy::<{ty}>();
            }};"#
        );
    }

    pub fn add_static_size_assert(&mut self, ty: &RustType, size: usize) {
        wln!(
            self,
            r#"const _: [(); {size}] = [(); ::std::mem::size_of::<{ty}>()];"#
        );
    }

    pub fn add_static_align_assert(&mut self, ty: &RustType, align: usize) {
        wln!(
            self,
            r#"const _: [(); {align}] = [(); ::std::mem::align_of::<{ty}>()];"#
        );
    }

    pub(crate) fn add_builder_for_dyn_trait(&mut self, tr: &ZngurTrait) -> CppTraitDefinition {
        assert!(matches!(tr.tr, RustTrait::Normal { .. }));
        let mut method_mangled_name = vec![];
        wln!(self, r#"unsafe extern "C" {{"#);
        for method in &tr.methods {
            let name = mangle_name(&tr.tr.to_string()) + "_" + &method.name;
            let inputs = method
                .inputs
                .iter()
                .enumerate()
                .map(|(n, ty)| self.generate_rust_call_type(n, ty))
                .join(" ");
            wln!(self, r#"fn {name}(data: *mut u8, {inputs} o: *mut u8);"#,);
            method_mangled_name.push(name);
        }
        wln!(self, "}}");
        let link_name = self.add_builder_for_dyn_trait_owned(tr, &method_mangled_name);
        let link_name_ref = self.add_builder_for_dyn_trait_borrowed(tr, &method_mangled_name);
        CppTraitDefinition::Normal {
            as_ty: tr.tr.into_cpp(),
            methods: tr
                .methods
                .clone()
                .into_iter()
                .zip(method_mangled_name)
                .map(|(x, rust_link_name)| CppTraitMethod {
                    name: x.name,
                    sig: CppFnSig {
                        rust_link_name,
                        inputs: x.inputs.into_iter().map(|x| x.into_cpp()).collect(),
                        output: x.output.into_cpp(),
                    },
                })
                .collect(),
            link_name,
            link_name_ref,
        }
    }

    fn add_builder_for_dyn_trait_owned(
        &mut self,
        tr: &ZngurTrait,
        method_mangled_name: &[String],
    ) -> String {
        let trait_name = tr.tr.to_string();
        let (trait_without_assocs, assocs) = tr.tr.clone().take_assocs();
        let mangled_name = mangle_name(&trait_name);
        wln!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {mangled_name}(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    o: *mut u8,
) {{
    struct Wrapper {{ 
        value: ZngurCppOpaqueOwnedObject,
    }}
    impl {trait_without_assocs} for Wrapper {{
"#
        );
        for (name, ty) in assocs {
            wln!(self, "        type {name} = {ty};");
        }
        for (method, rust_link_name) in tr.methods.iter().zip(method_mangled_name) {
            w!(self, "        fn {}(", method.name);
            match method.receiver {
                crate::ZngurMethodReceiver::Static => {
                    panic!("traits with static methods are not object safe");
                }
                crate::ZngurMethodReceiver::Ref(Mutability::Not) => w!(self, "&self"),
                crate::ZngurMethodReceiver::Ref(Mutability::Mut) => w!(self, "&mut self"),
                crate::ZngurMethodReceiver::Move => w!(self, "self"),
            }
            for (i, ty) in method.inputs.iter().enumerate() {
                w!(self, ", i{i}: {ty}");
            }
            wln!(self, ") -> {} {{ unsafe {{", method.output);
            wln!(self, "            let data = self.value.ptr();");
            self.call_cpp_function(&format!("{rust_link_name}(data, "), &method.inputs);
            wln!(self, "        }} }}");
        }
        wln!(
            self,
            r#"
    }}
    unsafe {{ 
        let this = Wrapper {{
            value: ZngurCppOpaqueOwnedObject::new(data, destructor),
        }};
        let r: Box<dyn {trait_name}> = Box::new(this);
        std::ptr::write(o as *mut _, r)
    }}
}}"#
        );
        mangled_name
    }

    fn add_builder_for_dyn_trait_borrowed(
        &mut self,
        tr: &ZngurTrait,
        method_mangled_name: &[String],
    ) -> String {
        let trait_name = tr.tr.to_string();
        let (trait_without_assocs, assocs) = tr.tr.clone().take_assocs();
        let mangled_name = mangle_name(&trait_name) + "_borrowed";
        wln!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {mangled_name}(
    data: *mut u8,
    o: *mut u8,
) {{
    struct Wrapper(ZngurCppOpaqueBorrowedObject);
    impl {trait_without_assocs} for Wrapper {{
"#
        );
        for (name, ty) in assocs {
            wln!(self, "        type {name} = {ty};");
        }
        for (method, rust_link_name) in tr.methods.iter().zip(method_mangled_name) {
            w!(self, "        fn {}(", method.name);
            match method.receiver {
                crate::ZngurMethodReceiver::Static => {
                    panic!("traits with static methods are not object safe");
                }
                crate::ZngurMethodReceiver::Ref(Mutability::Not) => w!(self, "&self"),
                crate::ZngurMethodReceiver::Ref(Mutability::Mut) => w!(self, "&mut self"),
                crate::ZngurMethodReceiver::Move => w!(self, "self"),
            }
            for (i, ty) in method.inputs.iter().enumerate() {
                w!(self, ", i{i}: {ty}");
            }
            wln!(self, ") -> {} {{ unsafe {{", method.output);
            wln!(
                self,
                "            let data = ::std::mem::transmute::<_, *mut u8>(self);"
            );
            self.call_cpp_function(&format!("{rust_link_name}(data, "), &method.inputs);
            wln!(self, "        }} }}");
        }
        wln!(
            self,
            r#"
    }}
    unsafe {{ 
        let this = data as *mut Wrapper;
        let r: &dyn {trait_name} = &*this;
        std::ptr::write(o as *mut _, r)
    }}
}}"#
        );
        mangled_name
    }

    pub fn add_builder_for_dyn_fn(
        &mut self,
        name: &str,
        inputs: &[RustType],
        output: &RustType,
    ) -> String {
        let mangled_name = mangle_name(&inputs.iter().chain(Some(output)).join(", "));
        let trait_str = format!("{name}({}) -> {output}", inputs.iter().join(", "));
        let c_args = inputs
            .iter()
            .enumerate()
            .map(|(n, ty)| self.generate_rust_call_type(n, ty))
            .join(" ");
        wln!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {mangled_name}(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    call: extern "C" fn(data: *mut u8, {c_args} o: *mut u8),
    o: *mut u8,
) {{
    let this = unsafe {{ ZngurCppOpaqueOwnedObject::new(data, destructor) }};
    let r: Box<dyn {trait_str}> = Box::new(move |{}| unsafe {{
        _ = &this;
        let data = this.ptr();
"#,
            inputs
                .iter()
                .enumerate()
                .map(|(n, ty)| format!("i{n}: {ty}"))
                .join(", "),
        );
        self.call_cpp_function("call(data, ", &inputs);
        wln!(
            self,
            r#"
    }});
    unsafe {{ std::ptr::write(o as *mut _, r) }}
}}"#
        );
        mangled_name
    }

    pub fn add_tuple_constructor(&mut self, fields: &[RustType]) -> String {
        let constructor = mangle_name(&fields.iter().join("&"));
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {constructor}("#
        );
        for (name, ty) in fields.iter().enumerate() {
            let gty = self.generate_rust_call_type(name, ty);
            w!(self, "{gty}");
        }
        w!(
            self,
            r#"o: *mut u8) {{ unsafe {{
    ::std::ptr::write(o as *mut _, ("#
        );
        for (name, ty) in fields.iter().enumerate() {
            self.generate_rust_call_value_from_argument(name, ty);
        }
        wln!(self, ")) }} }}");
        constructor
    }

    pub fn add_constructor(
        &mut self,
        rust_name: &str,
        args: &[(String, RustType)],
    ) -> ConstructorMangledNames {
        let constructor = mangle_name(rust_name);
        let match_check = format!("{constructor}_check");
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {constructor}("#
        );
        for (name, (_, ty)) in args.iter().enumerate() {
            let gty = self.generate_rust_call_type(name, ty);
            w!(self, "{gty}");
        }
        w!(
            self,
            r#"o: *mut u8) {{ unsafe {{
    ::std::ptr::write(o as *mut _, {rust_name} {{ "#
        );
        for (n, (name, ty)) in args.iter().enumerate() {
            w!(self, "{name}: ");
            self.generate_rust_call_value_from_argument(n, ty);
        }
        wln!(self, "}}) }} }}");
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {match_check}(i: *mut u8, o: *mut u8) {{ unsafe {{
    *o = matches!(::std::mem::transmute::<_, &_>(i), {rust_name} {{ .. }}) as u8;
}} }}"#
        );
        ConstructorMangledNames {
            constructor,
            match_check,
        }
    }

    pub(crate) fn add_field_assertions(&mut self, field: &ZngurField, owner: &RustType) {
        let ZngurField { name, ty, offset } = field;
        wln!(
            self,
            r#"
            const _: [(); {offset}] = [(); ::std::mem::offset_of!({owner}, {name})];
            const _: () = {{
                fn check_field(value: {owner}) -> {ty} {{
                    value.{name}
                }}
            }};
            "#
        );
    }

    pub fn add_extern_cpp_impl(
        &mut self,
        owner: &RustType,
        tr: Option<&RustTrait>,
        methods: &[ZngurMethod],
    ) -> Vec<String> {
        let mut mangled_names = vec![];
        w!(self, r#"unsafe extern "C" {{"#);
        for method in methods {
            let mn = mangle_name(&format!("{}_extern_method_{}", owner, method.name));
            w!(
                self,
                r#"
    fn {mn}("#
            );
            let receiver_ty = match method.receiver {
                ZngurMethodReceiver::Static => None,
                ZngurMethodReceiver::Ref(mutability) => {
                    Some(RustType::Ref(mutability, Box::new(owner.clone())))
                }
                ZngurMethodReceiver::Move => Some(owner.clone()),
            };
            for (n, ty) in receiver_ty.iter().chain(&method.inputs).enumerate() {
                let gty = self.generate_rust_call_type(n, ty);
                w!(self, "{gty}");
            }
            wln!(self, r#"o: *mut u8);"#);
            mangled_names.push(mn);
        }
        w!(self, r#"}}"#);
        match tr {
            Some(tr) => {
                let (tr, assocs) = tr.clone().take_assocs();
                w!(self, r#"impl {tr} for {owner} {{"#);
                for (name, ty) in assocs {
                    w!(self, r#"type {name} = {ty};"#);
                }
            }
            None => w!(self, r#"impl {owner} {{"#),
        }
        for (mn, method) in mangled_names.iter().zip(methods) {
            if tr.is_none() {
                w!(self, "pub ");
            }
            w!(self, r#"fn {}("#, method.name);
            match method.receiver {
                ZngurMethodReceiver::Static => (),
                ZngurMethodReceiver::Ref(Mutability::Mut) => w!(self, "&mut self, "),
                ZngurMethodReceiver::Ref(Mutability::Not) => w!(self, "&self, "),
                ZngurMethodReceiver::Move => w!(self, "self, "),
            }
            let receiver_ty = match method.receiver {
                ZngurMethodReceiver::Static => None,
                ZngurMethodReceiver::Ref(mutability) => {
                    Some(RustType::Ref(mutability, Box::new(owner.clone())))
                }
                ZngurMethodReceiver::Move => Some(owner.clone()),
            };
            let input_offset = if method.receiver == ZngurMethodReceiver::Static {
                0
            } else {
                1
            };
            for (ty, n) in method.inputs.iter().zip(input_offset..) {
                w!(self, "i{n}: {ty}, ");
            }
            wln!(self, ") -> {} {{ unsafe {{", method.output);
            if method.receiver != ZngurMethodReceiver::Static {
                wln!(self, "let i0 = self;");
            }
            self.call_cpp_function(
                &format!("{mn}("),
                &receiver_ty
                    .into_iter()
                    .chain(method.inputs.clone())
                    .collect::<Vec<_>>(),
            );
            wln!(self, "}} }}");
        }
        w!(self, r#"}}"#);
        mangled_names
    }

    pub fn add_extern_cpp_function(
        &mut self,
        rust_name: &str,
        inputs: &[RustType],
        output: &RustType,
    ) -> String {
        let mangled_name = mangle_name(rust_name);
        w!(
            self,
            r#"
unsafe extern "C" {{ fn {mangled_name}("#
        );
        for (n, ty) in inputs.iter().enumerate() {
            let gty = self.generate_rust_call_type(n, ty);
            w!(self, "{gty}");
        }
        wln!(self, r#"o: *mut u8); }}"#);
        w!(
            self,
            r#"
pub(crate) fn {rust_name}("#
        );
        for (n, ty) in inputs.iter().enumerate() {
            w!(self, "i{n}: {ty}, ");
        }
        wln!(self, ") -> {output} {{ unsafe {{");
        self.call_cpp_function(&format!("{mangled_name}("), &inputs);
        wln!(self, "}} }}");
        mangled_name
    }

    pub fn add_cpp_value_bridge(&mut self, ty: &RustType, field: &str) -> String {
        let mangled_name = mangle_name(&format!("{ty}_cpp_value_{field}"));
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {mangled_name}(d: *mut u8) -> *mut ZngurCppOpaqueOwnedObject {{
    unsafe {{ &mut (*(d as *mut {ty})).{field} }}
}}"#
        );
        mangled_name
    }

    pub fn add_function(
        &mut self,
        rust_name: &str,
        inputs: &[RustType],
        output: &RustType,
        use_path: Option<Vec<String>>,
        deref: bool,
    ) -> String {
        let mut mangled_name = mangle_name(rust_name);
        if deref {
            mangled_name += "_deref_";
            mangled_name += &mangle_name(&inputs[0].to_string());
        }
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {mangled_name}("#
        );
        for (n, ty) in inputs.iter().enumerate() {
            let gty = self.generate_rust_call_type(n, ty);
            w!(self, "{gty}");
        }
        wln!(self, "o: *mut u8) {{ unsafe {{");
        self.wrap_in_catch_unwind(|this| {
            if let Some(use_path) = use_path {
                if use_path.first().is_some_and(|x| x == "crate") {
                    wln!(this, "    use {};", use_path.iter().join("::"));
                } else {
                    wln!(this, "    use ::{};", use_path.iter().join("::"));
                }
            }
            w!(
                this,
                "    ::std::ptr::write(o as *mut {output}, {rust_name}("
            );
            if deref {
                w!(this, "&");
            }
            for (n, ty) in inputs.iter().enumerate() {
                this.generate_rust_call_value_from_argument(n, ty);
            }
            wln!(this, "));");
        });
        wln!(self, " }} }}");
        mangled_name
    }

    pub(crate) fn add_wellknown_trait(
        &mut self,
        ty: &RustType,
        wellknown_trait: ZngurWellknownTrait,
        is_unsized: bool,
    ) -> ZngurWellknownTraitData {
        match wellknown_trait {
            ZngurWellknownTrait::Unsized => ZngurWellknownTraitData::Unsized,
            ZngurWellknownTrait::Copy => ZngurWellknownTraitData::Copy,
            ZngurWellknownTrait::Drop => {
                let drop_in_place = mangle_name(&format!("{ty}=drop_in_place"));
                wln!(
                    self,
                    r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {drop_in_place}(v: *mut u8) {{ unsafe {{
    ::std::ptr::drop_in_place(v as *mut {ty});
}} }}"#
                );
                ZngurWellknownTraitData::Drop { drop_in_place }
            }
            ZngurWellknownTrait::Debug => {
                let pretty_print = mangle_name(&format!("{ty}=debug_pretty"));
                let debug_print = mangle_name(&format!("{ty}=debug_print"));
                let dbg_ty = if !is_unsized {
                    format!("{ty}")
                } else {
                    format!("&{ty}")
                };
                wln!(
                    self,
                    r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {pretty_print}(v: *mut u8) {{
    eprintln!("{{:#?}}", unsafe {{ &*(v as *mut {dbg_ty}) }});
}}"#
                );
                wln!(
                    self,
                    r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {debug_print}(v: *mut u8) {{
    eprintln!("{{:?}}", unsafe {{ &*(v as *mut {dbg_ty}) }});
}}"#
                );
                ZngurWellknownTraitData::Debug {
                    pretty_print,
                    debug_print,
                }
            }
        }
    }

    pub(crate) fn enable_panic_to_exception(&mut self) {
        wln!(
            self,
            r#"thread_local! {{
            pub static PANIC_PAYLOAD: ::std::cell::Cell<Option<()>> = ::std::cell::Cell::new(None);
        }}
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub fn __zngur_detect_panic() -> u8 {{
            PANIC_PAYLOAD.with(|p| {{
                let pp = p.take();
                let r = if pp.is_some() {{ 1 }} else {{ 0 }};
                p.set(pp);
                r
            }})
        }}

        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub fn __zngur_take_panic() {{
            PANIC_PAYLOAD.with(|p| {{
                p.take();
            }})
        }}
        "#
        );
        self.panic_to_exception = true;
    }

    fn wrap_in_catch_unwind(&mut self, f: impl FnOnce(&mut RustFile)) {
        if !self.panic_to_exception {
            f(self);
        } else {
            wln!(self, "let e = ::std::panic::catch_unwind(|| {{");
            f(self);
            wln!(self, "}});");
            wln!(
                self,
                "if let Err(_) = e {{ PANIC_PAYLOAD.with(|p| p.set(Some(()))) }}"
            );
        }
    }

    pub(crate) fn add_layout_policy_shim(
        &mut self,
        ty: &RustType,
        layout: LayoutPolicy,
    ) -> CppLayoutPolicy {
        match layout {
            LayoutPolicy::StackAllocated { size, align } => {
                CppLayoutPolicy::StackAllocated { size, align }
            }
            LayoutPolicy::HeapAllocated => {
                let size_fn = mangle_name(&format!("{ty}_size_fn"));
                let alloc_fn = mangle_name(&format!("{ty}_alloc_fn"));
                let free_fn = mangle_name(&format!("{ty}_free_fn"));
                wln!(
                    self,
                    r#"
                #[allow(non_snake_case)]
                #[unsafe(no_mangle)]
                pub fn {size_fn}() -> usize {{
                    ::std::mem::size_of::<{ty}>()
                }}
        
                #[allow(non_snake_case)]
                #[unsafe(no_mangle)]
                pub fn {alloc_fn}() -> *mut u8 {{
                    unsafe {{ ::std::alloc::alloc(::std::alloc::Layout::new::<{ty}>()) }}
                }}

                #[allow(non_snake_case)]
                #[unsafe(no_mangle)]
                pub fn {free_fn}(p: *mut u8) {{
                    unsafe {{ ::std::alloc::dealloc(p, ::std::alloc::Layout::new::<{ty}>()) }}
                }}
                "#
                );
                CppLayoutPolicy::HeapAllocated {
                    size_fn,
                    alloc_fn,
                    free_fn,
                }
            }
            LayoutPolicy::OnlyByRef => CppLayoutPolicy::OnlyByRef,
        }
    }

    fn generate_rust_call_value_from_argument(&mut self, name: usize, ty: &RustType) {
        match ty {
            RustType::Primitive(_) => w!(self, "i{name},"),
            RustType::Ref(_, inner) if !self.unsized_types.contains(inner) => {
                w!(self, "::std::mem::transmute::<_, {ty}>(i{name}),")
            }
            _ => w!(self, "::std::ptr::read(i{name} as *mut {ty}), "),
        }
    }

    fn generate_rust_call_type(&mut self, name: usize, ty: &RustType) -> String {
        match ty {
            RustType::Primitive(_) => format!("i{name}: {ty},"),
            RustType::Ref(_, inner) if !self.unsized_types.contains(inner) => {
                format!("i{name}: *mut ::std::ffi::c_void,")
            }
            _ => format!("i{name}: *mut u8, "),
        }
    }
}
