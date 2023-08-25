use std::{
    fmt::{Display, Write},
    iter,
};

use iter_tools::Itertools;

use crate::{
    cpp::{cpp_handle_keyword, CppPath, CppType},
    parser::Mutability,
    ZngurWellknownTrait, ZngurWellknownTraitData,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScalarRustType {
    Uint(u32),
    Int(u32),
    Usize,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RustPathAndGenerics {
    pub path: Vec<String>,
    pub generics: Vec<RustType>,
    pub named_generics: Vec<(String, RustType)>,
}

impl RustPathAndGenerics {
    fn into_cpp(&self) -> CppType {
        let RustPathAndGenerics {
            path,
            generics,
            named_generics,
        } = self;
        let named_generics = named_generics.iter().sorted_by_key(|x| &x.0).map(|x| &x.1);
        CppType {
            path: CppPath(
                iter::once("rust")
                    .chain(path.iter().map(|x| x.as_str()))
                    .map(cpp_handle_keyword)
                    .map(|x| x.to_owned())
                    .collect(),
            ),
            generic_args: generics
                .iter()
                .chain(named_generics)
                .map(|x| x.into_cpp())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustTrait {
    Normal(RustPathAndGenerics),
    Fn {
        name: String,
        inputs: Vec<RustType>,
        output: Box<RustType>,
    },
}
impl RustTrait {
    pub fn into_cpp_type(&self) -> CppType {
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

    fn take_assocs(mut self) -> (Self, Vec<(String, RustType)>) {
        let assocs = match &mut self {
            RustTrait::Normal(p) => std::mem::take(&mut p.named_generics),
            RustTrait::Fn { .. } => vec![],
        };
        (self, assocs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustType {
    Scalar(ScalarRustType),
    Ref(Mutability, Box<RustType>),
    Raw(Mutability, Box<RustType>),
    Boxed(Box<RustType>),
    Slice(Box<RustType>),
    Dyn(RustTrait, Vec<String>),
    Tuple(Vec<RustType>),
    Adt(RustPathAndGenerics),
}

impl Display for RustPathAndGenerics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let RustPathAndGenerics {
            path,
            generics,
            named_generics,
        } = self;
        for p in path {
            if p != "crate" {
                write!(f, "::")?;
            }
            write!(f, "{p}")?;
        }
        if !generics.is_empty() || !named_generics.is_empty() {
            write!(
                f,
                "::<{}>",
                generics
                    .iter()
                    .map(|x| format!("{x}"))
                    .chain(named_generics.iter().map(|x| format!("{} = {}", x.0, x.1)))
                    .join(", ")
            )?;
        }
        Ok(())
    }
}

impl Display for RustTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustTrait::Normal(tr) => write!(f, "{tr}"),
            RustTrait::Fn {
                name,
                inputs,
                output,
            } => {
                write!(f, "{name}({})", inputs.iter().join(", "))?;
                if **output != RustType::UNIT {
                    write!(f, " -> {output}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for RustType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustType::Scalar(s) => match s {
                ScalarRustType::Uint(s) => write!(f, "u{s}"),
                ScalarRustType::Int(s) => write!(f, "i{s}"),
                ScalarRustType::Usize => write!(f, "usize"),
                ScalarRustType::Bool => write!(f, "bool"),
            },
            RustType::Ref(Mutability::Not, ty) => write!(f, "&{ty}"),
            RustType::Ref(Mutability::Mut, ty) => write!(f, "&mut {ty}"),
            RustType::Raw(Mutability::Not, ty) => write!(f, "*const {ty}"),
            RustType::Raw(Mutability::Mut, ty) => write!(f, "*mut {ty}"),
            RustType::Boxed(ty) => write!(f, "Box<{ty}>"),
            RustType::Tuple(v) => write!(f, "({})", v.iter().join(", ")),
            RustType::Adt(pg) => write!(f, "{pg}"),
            RustType::Dyn(tr, marker_bounds) => {
                write!(f, "dyn {tr}")?;
                for mb in marker_bounds {
                    write!(f, "+ {mb}")?;
                }
                Ok(())
            }
            RustType::Slice(s) => write!(f, "[{s}]"),
        }
    }
}

impl RustType {
    const UNIT: Self = RustType::Tuple(Vec::new());

    pub fn into_cpp_builtin(&self) -> Option<CppType> {
        match self {
            RustType::Scalar(s) => match s {
                ScalarRustType::Uint(s) => Some(CppType::from(&*format!("uint{s}_t"))),
                ScalarRustType::Int(s) => Some(CppType::from(&*format!("int{s}_t"))),
                ScalarRustType::Usize => Some(CppType::from("size_t")),
                ScalarRustType::Bool => None,
            },
            RustType::Raw(_, t) => Some(CppType::from(&*format!(
                "{}*",
                t.into_cpp_builtin()?.to_string().strip_prefix("::")?
            ))),
            _ => None,
        }
    }

    pub fn into_cpp(&self) -> CppType {
        if let Some(builtin) = self.into_cpp_builtin() {
            return builtin;
        }
        match self {
            RustType::Scalar(s) => match s {
                ScalarRustType::Bool => CppType::from("rust::Bool"),
                _ => unreachable!(),
            },
            RustType::Boxed(t) => CppType {
                path: CppPath::from("rust::Box"),
                generic_args: vec![t.into_cpp()],
            },
            RustType::Ref(_, t) => CppType {
                path: CppPath::from("rust::Ref"),
                generic_args: vec![t.into_cpp()],
            },
            RustType::Slice(s) => CppType {
                path: CppPath::from("rust::Slice"),
                generic_args: vec![s.into_cpp()],
            },
            RustType::Raw(_, t) => todo!(),
            RustType::Adt(pg) => pg.into_cpp(),
            RustType::Tuple(v) => {
                if v.is_empty() {
                    return CppType::from("rust::Unit");
                }
                todo!()
            }
            RustType::Dyn(tr, marker_bounds) => {
                let tr_as_cpp_type = tr.into_cpp_type();
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
        }
    }
}

pub struct RustFile(pub String);

impl Default for RustFile {
    fn default() -> Self {
        Self(
            r#"
struct ZngurCppOpaqueObject {
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
}

impl Drop for ZngurCppOpaqueObject {
    fn drop(&mut self) {
        (self.destructor)(self.data)
    }
}
"#
            .to_owned(),
        )
    }
}

impl Write for RustFile {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
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

impl RustFile {
    fn call_cpp_function(&mut self, name: &str, inputs: usize) {
        for n in 0..inputs {
            wln!(self, "let mut i{n} = ::core::mem::MaybeUninit::new(i{n});")
        }
        wln!(self, "let mut r = ::core::mem::MaybeUninit::uninit();");
        w!(self, "{name}(data");
        for n in 0..inputs {
            w!(self, ", i{n}.as_mut_ptr() as *mut u8");
        }
        wln!(self, ", r.as_mut_ptr() as *mut u8);");
        wln!(self, "r.assume_init()");
    }

    pub fn add_static_size_assert(&mut self, ty: &RustType, size: usize) {
        wln!(
            self,
            r#"const _: () = assert!(::std::mem::size_of::<{ty}>() == {size});"#
        );
    }

    pub fn add_static_align_assert(&mut self, ty: &RustType, align: usize) {
        wln!(
            self,
            r#"const _: () = assert!(::std::mem::align_of::<{ty}>() == {align});"#
        );
    }

    pub(crate) fn add_builder_for_dyn_trait(&mut self, tr: &crate::ZngurTrait) -> String {
        let trait_name = tr.tr.to_string();
        let (trait_without_assocs, assocs) = tr.tr.clone().take_assocs();
        let mangled_name = mangle_name(&trait_name);
        wln!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {mangled_name}(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    f_next: extern "C" fn(data: *mut u8, o: *mut u8),
    o: *mut u8,
) {{
    struct Wrapper {{ 
        value: ZngurCppOpaqueObject,
        f_next: extern "C" fn(data: *mut u8, o: *mut u8),
    }}
    impl {trait_without_assocs} for Wrapper {{
"#
        );
        for (name, ty) in assocs {
            wln!(self, "        type {name} = {ty};");
        }
        for method in &tr.methods {
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
            wln!(self, "            let data = self.value.data;");
            self.call_cpp_function("(self.f_next)", 0);
            wln!(self, "        }} }}");
        }
        wln!(
            self,
            r#"
    }}
    let this = Wrapper {{
        value: ZngurCppOpaqueObject {{ data, destructor }},
        f_next,
    }};
    let r: Box<dyn {trait_name}> = Box::new(this);
    unsafe {{ std::ptr::write(o as *mut _, r) }}
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
        wln!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {mangled_name}(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    call: extern "C" fn(data: *mut u8, i1: *mut u8, o: *mut u8),
    o: *mut u8,
) {{
    let this = ZngurCppOpaqueObject {{ data, destructor }};
    let r: Box<dyn {trait_str}> = Box::new(move |i0| unsafe {{
        _ = &this;
        let data = this.data;
"#,
        );
        self.call_cpp_function("call", 1);
        wln!(
            self,
            r#"
    }});
    unsafe {{ std::ptr::write(o as *mut _, r) }}
}}"#
        );
        mangled_name
    }

    pub fn add_constructor<'a>(
        &mut self,
        rust_name: &str,
        args: impl Iterator<Item = &'a str> + Clone,
    ) -> ConstructorMangledNames {
        let constructor = mangle_name(rust_name);
        let match_check = format!("{constructor}_check");
        w!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {constructor}("#
        );
        for name in args.clone() {
            w!(self, "f_{name}: *mut u8, ");
        }
        w!(
            self,
            r#"o: *mut u8) {{ unsafe {{
    ::std::ptr::write(o as *mut _, {rust_name} {{ "#
        );
        for name in args {
            w!(self, "{name}: ::std::ptr::read(f_{name} as *mut _), ");
        }
        wln!(self, "}}) }} }}");
        w!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {match_check}(i: *mut u8, o: *mut u8) {{ unsafe {{
    *o = matches!(&*(i as *mut &_), {rust_name} {{ .. }}) as u8;
}} }}"#
        );
        ConstructorMangledNames {
            constructor,
            match_check,
        }
    }

    pub fn add_function(
        &mut self,
        rust_name: &str,
        inputs: &[RustType],
        output: &RustType,
        use_path: Option<Vec<String>>,
    ) -> String {
        let mangled_name = mangle_name(rust_name);
        w!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {mangled_name}("#
        );
        for n in 0..inputs.len() {
            w!(self, "i{n}: *mut u8, ");
        }
        wln!(self, "o: *mut u8) {{ unsafe {{");
        if let Some(use_path) = use_path {
            wln!(self, "    use ::{};", use_path.iter().join("::"));
        }
        w!(
            self,
            "    ::std::ptr::write(o as *mut {output}, {rust_name}("
        );
        for (n, ty) in inputs.iter().enumerate() {
            w!(self, "::std::ptr::read(i{n} as *mut {ty}), ");
        }
        wln!(self, ")) }} }}");
        mangled_name
    }

    pub(crate) fn add_wellknown_trait(
        &mut self,
        ty: &RustType,
        wellknown_trait: ZngurWellknownTrait,
    ) -> ZngurWellknownTraitData {
        match wellknown_trait {
            ZngurWellknownTrait::Unsized => ZngurWellknownTraitData::Unsized,
            ZngurWellknownTrait::Debug => {
                let pretty_print = mangle_name(&format!("{ty}=debug_pretty"));
                let debug_print = mangle_name(&format!("{ty}=debug_print"));
                wln!(
                    self,
                    r#"
#[no_mangle]
pub extern "C" fn {pretty_print}(v: *mut u8) {{
    eprintln!("{{:#?}}", unsafe {{ &*(v as *mut {ty}) }});
}}"#
                );
                wln!(
                    self,
                    r#"
#[no_mangle]
pub extern "C" fn {debug_print}(v: *mut u8) {{
    eprintln!("{{:?}}", unsafe {{ &*(v as *mut {ty}) }});
}}"#
                );
                ZngurWellknownTraitData::Debug {
                    pretty_print,
                    debug_print,
                }
            }
        }
    }
}
