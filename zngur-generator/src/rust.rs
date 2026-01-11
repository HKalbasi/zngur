use std::fmt::Write;

use itertools::Itertools;
use sha2::{Digest, Sha256};

use crate::{
    ZngurTrait, ZngurWellknownTrait, ZngurWellknownTraitData,
    cpp::{
        CppFnSig, CppLayoutPolicy, CppPath, CppTraitDefinition, CppTraitMethod, CppType,
        PanicToExceptionSymbols,
    },
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
                RustType::Primitive(s) => match s {
                    PrimitiveRustType::Uint(s) => Some(CppType::from(&*format!("uint{s}_t"))),
                    PrimitiveRustType::Int(s) => Some(CppType::from(&*format!("int{s}_t"))),
                    PrimitiveRustType::Float(32) => Some(CppType::from("float_t")),
                    PrimitiveRustType::Float(64) => Some(CppType::from("double_t")),
                    PrimitiveRustType::Float(_) => unreachable!(),
                    PrimitiveRustType::Usize => Some(CppType::from("size_t")),
                    PrimitiveRustType::Bool | PrimitiveRustType::Str => None,
                    PrimitiveRustType::ZngurCppOpaqueOwnedObject => {
                        Some(CppType::from("rust::ZngurCppOpaqueOwnedObject"))
                    }
                },
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
        if let Some(builtin) = for_builtin(self) {
            return builtin;
        }
        match self {
            RustType::Primitive(s) => match s {
                PrimitiveRustType::Bool => CppType::from("rust::Bool"),
                PrimitiveRustType::Str => CppType::from("rust::Str"),
                _ => unreachable!(),
            },
            RustType::Boxed(t) => CppType {
                path: CppPath::from("rust::Box"),
                generic_args: vec![t.into_cpp()],
            },
            RustType::Ref(m, t) => CppType {
                path: match m {
                    Mutability::Mut => CppPath::from("rust::RefMut"),
                    Mutability::Not => CppPath::from("rust::Ref"),
                },
                generic_args: vec![t.into_cpp()],
            },
            RustType::Slice(s) => CppType {
                path: CppPath::from("rust::Slice"),
                generic_args: vec![s.into_cpp()],
            },
            RustType::Raw(m, t) => CppType {
                path: match m {
                    Mutability::Mut => CppPath::from("rust::RawMut"),
                    Mutability::Not => CppPath::from("rust::Raw"),
                },
                generic_args: vec![t.into_cpp()],
            },
            RustType::Adt(pg) => pg.into_cpp(),
            RustType::Tuple(v) => {
                if v.is_empty() {
                    return CppType::from("rust::Unit");
                }
                CppType {
                    path: CppPath::from("rust::Tuple"),
                    generic_args: v.into_iter().map(|x| x.into_cpp()).collect(),
                }
            }
            RustType::Dyn(tr, marker_bounds) => {
                let tr_as_cpp_type = tr.into_cpp();
                CppType {
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
            RustType::Impl(_, _) => panic!("impl Trait is invalid in C++"),
        }
    }
}

pub struct RustFile {
    pub text: String,
    pub panic_to_exception: bool,
    pub mangling_base: String,
}

impl RustFile {
    pub fn new(mangling_base: &str) -> Self {
        Self {
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
            mangling_base: mangling_base.to_owned(),
        }
    }
}

impl Write for RustFile {
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

pub fn hash_of_sig(sig: &[RustType]) -> String {
    let mut text = "".to_owned();
    for elem in sig {
        text += &format!("{elem}+");
    }

    let digset = Sha256::digest(&text);
    hex::encode(&digset[..5])
}

fn mangle_name(name: &str, mangling_base: &str) -> String {
    let mut name = "_zngur_"
        .chars()
        .chain(mangling_base.chars())
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
        (2, "@", 'z'),
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

impl RustFile {
    fn mangle_name(&self, name: &str) -> String {
        mangle_name(name, &self.mangling_base)
    }

    fn call_cpp_function(&mut self, name: &str, inputs: usize) {
        for n in 0..inputs {
            wln!(self, "let mut i{n} = ::core::mem::MaybeUninit::new(i{n});")
        }
        wln!(self, "let mut r = ::core::mem::MaybeUninit::uninit();");
        w!(self, "{name}");
        for n in 0..inputs {
            w!(self, "i{n}.as_mut_ptr() as *mut u8, ");
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

    pub fn add_static_size_upper_bound_assert(&mut self, ty: &RustType, size: usize) {
        wln!(
            self,
            r#"const _: () = assert!({size} >= ::std::mem::size_of::<{ty}>());"#
        );
    }

    pub fn add_static_align_upper_bound_assert(&mut self, ty: &RustType, align: usize) {
        wln!(
            self,
            r#"const _: () = assert!({align} >= ::std::mem::align_of::<{ty}>());"#
        );
    }

    pub(crate) fn add_builder_for_dyn_trait(&mut self, tr: &ZngurTrait) -> CppTraitDefinition {
        assert!(matches!(tr.tr, RustTrait::Normal { .. }));
        let mut method_mangled_name = vec![];
        wln!(self, r#"unsafe extern "C" {{"#);
        for method in &tr.methods {
            let name = self.mangle_name(&tr.tr.to_string())
                + "_"
                + &method.name
                + "_"
                + &hash_of_sig(&method.generics)
                + "_"
                + &hash_of_sig(&method.inputs);
            wln!(
                self,
                r#"fn {name}(data: *mut u8, {} o: *mut u8);"#,
                method
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(n, _)| format!("i{n}: *mut u8,"))
                    .join(" ")
            );
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
                    rust_link_name,
                    inputs: x.inputs.into_iter().map(|x| x.into_cpp()).collect(),
                    output: x.output.into_cpp(),
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
        let mangled_name = self.mangle_name(&trait_name);
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
            self.call_cpp_function(&format!("{rust_link_name}(data, "), method.inputs.len());
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
        let mangled_name = self.mangle_name(&trait_name) + "_borrowed";
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
            self.call_cpp_function(&format!("{rust_link_name}(data, "), method.inputs.len());
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
        let mangled_name = self.mangle_name(&inputs.iter().chain(Some(output)).join(", "));
        let trait_str = format!("{name}({}) -> {output}", inputs.iter().join(", "));
        wln!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {mangled_name}(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    call: extern "C" fn(data: *mut u8, {} o: *mut u8),
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
                .map(|(n, _)| format!("i{n}: *mut u8, "))
                .join(" "),
            inputs
                .iter()
                .enumerate()
                .map(|(n, ty)| format!("i{n}: {ty}"))
                .join(", "),
        );
        self.call_cpp_function("call(data, ", inputs.len());
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
        let constructor = self.mangle_name(&fields.iter().join("&"));
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {constructor}("#
        );
        for name in 0..fields.len() {
            w!(self, "f_{name}: *mut u8, ");
        }
        w!(
            self,
            r#"o: *mut u8) {{ unsafe {{
    ::std::ptr::write(o as *mut _, ("#
        );
        for (name, ty) in fields.iter().enumerate() {
            w!(self, "::std::ptr::read(f_{name} as *mut {ty}), ");
        }
        wln!(self, ")) }} }}");
        constructor
    }

    pub fn add_constructor(
        &mut self,
        rust_name: &str,
        args: &[(String, RustType)],
    ) -> ConstructorMangledNames {
        let constructor = self.mangle_name(rust_name);
        let match_check = format!("{constructor}_check");
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {constructor}("#
        );
        for (name, _) in args {
            w!(self, "f_{name}: *mut u8, ");
        }
        w!(
            self,
            r#"o: *mut u8) {{ unsafe {{
    ::std::ptr::write(o as *mut _, {rust_name} {{ "#
        );
        for (name, ty) in args {
            w!(self, "{name}: ::std::ptr::read(f_{name} as *mut {ty}), ");
        }
        wln!(self, "}}) }} }}");
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn {match_check}(i: *mut u8, o: *mut u8) {{ unsafe {{
    *o = matches!(&*(i as *mut &_), {rust_name} {{ .. }}) as u8;
}} }}"#
        );
        ConstructorMangledNames {
            constructor,
            match_check,
        }
    }

    pub(crate) fn add_field_assertions(
        &mut self,
        field: &ZngurField,
        owner: &RustType,
    ) -> Option<String> {
        let ZngurField { name, ty, offset } = field;
        wln!(
            self,
            r#"
const _: () = {{
    #[allow(dead_code)]
    fn check_field(value: {owner}) -> {ty} {{
        value.{name}
    }}
}};
            "#
        );
        if let Some(offset) = offset {
            wln!(
                self,
                r#"
const _: [(); {offset}] = [(); ::std::mem::offset_of!({owner}, {name})];
                "#
            );
            None
        } else {
            let mn = self.mangle_name(&format!("{}_field_{}_offset", &owner, &name));
            wln!(
                self,
                r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub static {mn}: usize = ::std::mem::offset_of!({owner}, {name});
                "#
            );
            Some(mn)
        }
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
            let mn = self.mangle_name(&format!("{}_extern_method_{}", owner, method.name));
            w!(
                self,
                r#"
    fn {mn}("#
            );
            let input_offset = if method.receiver == ZngurMethodReceiver::Static {
                0
            } else {
                1
            };
            for n in 0..method.inputs.len() + input_offset {
                w!(self, "i{n}: *mut u8, ");
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
            self.call_cpp_function(&format!("{mn}("), method.inputs.len() + input_offset);
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
        let mangled_name = self.mangle_name(rust_name);
        w!(
            self,
            r#"
unsafe extern "C" {{ fn {mangled_name}("#
        );
        for (n, _) in inputs.iter().enumerate() {
            w!(self, "i{n}: *mut u8, ");
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
        self.call_cpp_function(&format!("{mangled_name}("), inputs.len());
        wln!(self, "}} }}");
        mangled_name
    }

    pub fn add_cpp_value_bridge(&mut self, ty: &RustType, field: &str) -> String {
        let mangled_name = self.mangle_name(&format!("{ty}_cpp_value_{field}"));
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
        deref: Option<Mutability>,
    ) -> CppFnSig {
        let mut mangled_name = self.mangle_name(rust_name) + "_" + &hash_of_sig(&inputs);
        if deref.is_some() {
            mangled_name += "_deref";
        }
        w!(
            self,
            r#"
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[allow(unused_parens)]
pub extern "C" fn {mangled_name}("#
        );
        for n in 0..inputs.len() {
            w!(self, "i{n}: *mut u8, ");
        }
        let (modified_output, is_impl_trait) = if let RustType::Impl(tr, bounds) = output {
            (
                RustType::Boxed(Box::new(RustType::Dyn(tr.clone(), bounds.clone()))),
                true,
            )
        } else {
            (output.clone(), false)
        };
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
                "    ::std::ptr::write(o as *mut {modified_output}, {impl_trait} {rust_name}(",
                impl_trait = if is_impl_trait { "Box::new( " } else { "" },
            );
            match deref {
                Some(Mutability::Mut) => w!(this, "::std::ops::DerefMut::deref_mut"),
                Some(Mutability::Not) => w!(this, "::std::ops::Deref::deref"),
                None => {}
            }
            for (n, ty) in inputs.iter().enumerate() {
                w!(this, "(::std::ptr::read(i{n} as *mut {ty})), ");
            }
            if is_impl_trait {
                wln!(this, ")));");
            } else {
                wln!(this, "));");
            }
        });
        wln!(self, " }} }}");
        CppFnSig {
            rust_link_name: mangled_name,
            inputs: inputs.iter().map(|ty| ty.into_cpp()).collect(),
            output: modified_output.into_cpp(),
        }
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
                let drop_in_place = self.mangle_name(&format!("{ty}=drop_in_place"));
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
                let pretty_print = self.mangle_name(&format!("{ty}=debug_pretty"));
                let debug_print = self.mangle_name(&format!("{ty}=debug_print"));
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

    pub(crate) fn enable_panic_to_exception(&mut self) -> PanicToExceptionSymbols {
        let detect_panic = self.mangle_name("@detect_panic");
        let take_panic = self.mangle_name("@take_panic");
        wln!(
            self,
            r#"thread_local! {{
            pub static PANIC_PAYLOAD: ::std::cell::Cell<Option<()>> = ::std::cell::Cell::new(None);
        }}
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub fn {detect_panic}() -> u8 {{
            PANIC_PAYLOAD.with(|p| {{
                let pp = p.take();
                let r = if pp.is_some() {{ 1 }} else {{ 0 }};
                p.set(pp);
                r
            }})
        }}

        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub fn {take_panic}() {{
            PANIC_PAYLOAD.with(|p| {{
                p.take();
            }})
        }}
        "#,
        );
        self.panic_to_exception = true;
        PanicToExceptionSymbols {
            detect_panic,
            take_panic,
        }
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
            LayoutPolicy::Conservative { size, align } => {
                CppLayoutPolicy::StackAllocated { size, align }
            }
            LayoutPolicy::HeapAllocated => {
                let size_fn = self.mangle_name(&format!("{ty}_size_fn"));
                let alloc_fn = self.mangle_name(&format!("{ty}_alloc_fn"));
                let free_fn = self.mangle_name(&format!("{ty}_free_fn"));
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
}
