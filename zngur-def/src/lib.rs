use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

mod merge;
pub use merge::{Merge, MergeFailure, MergeResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Mut,
    Not,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZngurExternCppImpl {
    pub tr: Option<RustTrait>,
    pub ty: RustType,
    pub methods: Vec<ZngurMethod>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurConstructor {
    pub name: Option<String>,
    pub inputs: Vec<(String, RustType)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurField {
    pub name: String,
    pub ty: RustType,
    pub offset: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurFieldData {
    pub name: String,
    pub ty: RustType,
    pub offset: ZngurFieldDataOffset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ZngurFieldDataOffset {
    Offset(usize),
    Auto(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZngurWellknownTrait {
    Debug,
    Drop,
    Unsized,
    Copy,
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
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPolicy {
    StackAllocated { size: usize, align: usize },
    Conservative { size: usize, align: usize },
    HeapAllocated,
    OnlyByRef,
}

impl LayoutPolicy {
    pub const ZERO_SIZED_TYPE: Self = LayoutPolicy::StackAllocated { size: 0, align: 1 };
}

#[derive(Debug, PartialEq, Eq)]
pub struct ZngurMethodDetails {
    pub data: ZngurMethod,
    pub use_path: Option<Vec<String>>,
    pub deref: Option<(RustType, Mutability)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CppValue(pub String, pub String);

#[derive(Debug, PartialEq, Eq)]
pub struct CppRef(pub String);

impl Display for CppRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ZngurType {
    pub ty: RustType,
    pub layout: LayoutPolicy,
    pub wellknown_traits: Vec<ZngurWellknownTrait>,
    pub methods: Vec<ZngurMethodDetails>,
    pub constructors: Vec<ZngurConstructor>,
    pub fields: Vec<ZngurField>,
    pub cpp_value: Option<CppValue>,
    pub cpp_ref: Option<CppRef>,
}

#[derive(Debug)]
pub struct ZngurTrait {
    pub tr: RustTrait,
    pub methods: Vec<ZngurMethod>,
}

#[derive(Debug, Default)]
pub struct AdditionalIncludes(pub String);

#[derive(Debug, Default)]
pub struct ConvertPanicToException(pub bool);

#[derive(Clone, Debug, Default)]
pub struct Import(pub std::path::PathBuf);
#[derive(Debug, Default)]
pub struct ZngurSpec {
    pub imports: Vec<Import>,
    pub types: Vec<ZngurType>,
    pub traits: HashMap<RustTrait, ZngurTrait>,
    pub funcs: Vec<ZngurFn>,
    pub extern_cpp_funcs: Vec<ZngurExternCppFn>,
    pub extern_cpp_impls: Vec<ZngurExternCppImpl>,
    pub additional_includes: AdditionalIncludes,
    pub convert_panic_to_exception: ConvertPanicToException,
    pub cpp_include_header_name: String,
    pub mangling_base: String,
    pub cpp_namespace: String,
    pub rust_cfg: Vec<(String, Option<String>)>,
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
    pub fn take_assocs(mut self) -> (Self, Vec<(String, RustType)>) {
        let assocs = match &mut self {
            RustTrait::Normal(p) => std::mem::take(&mut p.named_generics),
            RustTrait::Fn { .. } => vec![],
        };
        (self, assocs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveRustType {
    Uint(u32),
    Int(u32),
    Float(u32),
    Usize,
    Bool,
    Str,
    ZngurCppOpaqueOwnedObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RustPathAndGenerics {
    pub path: Vec<String>,
    pub generics: Vec<RustType>,
    pub named_generics: Vec<(String, RustType)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustType {
    Primitive(PrimitiveRustType),
    Ref(Mutability, Box<RustType>),
    Raw(Mutability, Box<RustType>),
    Boxed(Box<RustType>),
    Slice(Box<RustType>),
    Dyn(RustTrait, Vec<String>),
    Impl(RustTrait, Vec<String>),
    Tuple(Vec<RustType>),
    Adt(RustPathAndGenerics),
}

impl RustType {
    pub const UNIT: Self = RustType::Tuple(Vec::new());
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
            RustType::Primitive(s) => match s {
                PrimitiveRustType::Uint(s) => write!(f, "u{s}"),
                PrimitiveRustType::Int(s) => write!(f, "i{s}"),
                PrimitiveRustType::Float(s) => write!(f, "f{s}"),
                PrimitiveRustType::Usize => write!(f, "usize"),
                PrimitiveRustType::Bool => write!(f, "bool"),
                PrimitiveRustType::Str => write!(f, "str"),
                PrimitiveRustType::ZngurCppOpaqueOwnedObject => {
                    write!(f, "ZngurCppOpaqueOwnedObject")
                }
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
            RustType::Impl(tr, marker_bounds) => {
                write!(f, "impl {tr}")?;
                for mb in marker_bounds {
                    write!(f, "+ {mb}")?;
                }
                Ok(())
            }
            RustType::Slice(s) => write!(f, "[{s}]"),
        }
    }
}
