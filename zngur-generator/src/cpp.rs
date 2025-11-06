use std::{
    collections::HashMap,
    fmt::{Display, Write},
    iter,
};

use itertools::Itertools;
use zngur_def::{CppRef, CppValue, RustTrait, ZngurField, ZngurMethodReceiver};

use crate::{
    ZngurWellknownTraitData,
    template::{CppHeaderTemplate, CppSourceTemplate, ZngurHeaderTemplate},
};
use sailfish::Template;

#[derive(Debug)]
pub struct CppPath(pub Vec<String>);

impl CppPath {
    fn namespace(&self) -> &[String] {
        self.0.split_last().unwrap().1
    }

    pub(crate) fn open_namespace(&self) -> String {
        self.namespace()
            .iter()
            .enumerate()
            .map(|(i, x)| format!("{:indent$}namespace {} {{", "", x, indent = i * 4))
            .join("\n")
    }

    pub(crate) fn close_namespace(&self) -> String {
        self.namespace()
            .iter()
            .enumerate()
            .map(|(i, x)| format!("{:indent$}}} // namespace {}", "", x, indent = i * 4))
            .join("\n")
    }

    pub(crate) fn name(&self) -> &str {
        self.0.split_last().unwrap().0
    }

    /// Returns true if this is an infrastructure template type defined in zngur.h.
    ///
    /// These types have generic template definitions in zngur.h and never appear
    /// in type_defs as they don't need per-type specialization in the generated header.
    pub(crate) fn is_infrastructure_template(&self) -> bool {
        matches!(
            self.0
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
            ["rust", "Ref"]
                | ["rust", "RefMut"]
                | ["rust", "Raw"]
                | ["rust", "RawMut"]
                | ["rust", "FieldOwned"]
                | ["rust", "FieldRef"]
                | ["rust", "FieldRefMut"]
                | ["rust", "ZngurCppOpaqueOwnedObject"]
                | ["rust", "Tuple"] // generic Tuple, not Unit
        )
    }

    /// Returns true if this type is rust::Unit (the empty tuple).
    pub(crate) fn is_unit(&self) -> bool {
        self.0.len() == 2 && self.0[0] == "rust" && self.0[1] == "Unit"
    }

    /// Returns true if this type is rust::Bool.
    pub(crate) fn is_bool(&self) -> bool {
        self.0.len() == 2 && self.0[0] == "rust" && self.0[1] == "Bool"
    }

    /// Returns true if this type is rust::Str.
    pub(crate) fn is_str(&self) -> bool {
        self.0.len() == 2 && self.0[0] == "rust" && self.0[1] == "Str"
    }

    /// Returns whether this type needs a forward declaration in the generated header.
    ///
    /// Forward declarations are needed for user-defined types so they can be
    /// referenced before their full definition. Returns false for:
    /// - Primitive types (uint8_t, int32_t) - built-in to C++
    /// - Infrastructure types (Ref, RefMut, Raw, etc.) - defined in zngur.h as generic templates
    /// - Unit type - fully defined in zngur.h as Tuple<> specialization
    fn needs_forward_declaration(&self) -> bool {
        // Primitive types (like uint8_t, int32_t, etc.) have no namespace - just a single component
        if self.0.len() == 1 {
            return false;
        }

        // Skip infrastructure types defined in zngur.h
        if self.is_infrastructure_template() || self.is_unit() {
            return false;
        }

        // User types need forward declarations
        true
    }

    pub(crate) fn from_rust_path(path: &[String], namespace: &str) -> CppPath {
        CppPath(
            iter::once(namespace)
                .chain(path.iter().map(|x| x.as_str()))
                .map(cpp_handle_keyword)
                .map(|x| x.to_owned())
                .collect(),
        )
    }
}

impl<const N: usize> From<[&str; N]> for CppPath {
    fn from(value: [&str; N]) -> Self {
        CppPath(value.iter().map(|x| x.to_string()).collect())
    }
}

impl From<&str> for CppPath {
    fn from(value: &str) -> Self {
        let value = value.trim();
        CppPath(value.split("::").map(|x| x.to_owned()).collect())
    }
}

impl Display for CppPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "::{}", self.0.iter().join("::"))
    }
}

#[derive(Debug)]
pub struct CppType {
    pub path: CppPath,
    pub generic_args: Vec<CppType>,
}

impl sailfish::runtime::Render for CppType {
    fn render(
        &self,
        b: &mut sailfish::runtime::Buffer,
    ) -> std::result::Result<(), sailfish::runtime::RenderError> {
        write!(b, "{}", self.path)?;
        if !self.generic_args.is_empty() {
            write!(b, "< {} >", self.generic_args.iter().join(", "))?;
        }
        Ok(())
    }
}

impl CppType {
    pub fn into_ref(self) -> CppType {
        CppType {
            path: CppPath::from("rust::Ref"),
            generic_args: vec![self],
        }
    }

    /// Returns true if this type is rust::Bool.
    pub(crate) fn is_bool(&self) -> bool {
        self.path.is_bool()
    }

    /// Returns true if this type is rust::Str.
    pub(crate) fn is_str(&self) -> bool {
        self.path.is_str()
    }

    pub(crate) fn specialization_decl(&self) -> String {
        if self.generic_args.is_empty() {
            format!("struct {}", self.path.name())
        } else {
            format!(
                "template<> struct {}< {} >",
                self.path.name(),
                self.generic_args.iter().join(", ")
            )
        }
    }

    fn header_helper(&self, state: &mut impl Write) -> std::fmt::Result {
        // Note: probably need to keep this out of the template because it's recursive.
        for x in &self.generic_args {
            x.header_helper(state)?;
        }

        if !self.path.needs_forward_declaration() {
            return Ok(());
        }

        for p in self.path.namespace() {
            writeln!(state, "namespace {} {{", p)?;
        }

        if !self.generic_args.is_empty() {
            writeln!(state, "template<typename ...T>")?;
        }

        writeln!(state, "struct {};", self.path.name())?;

        for _ in self.path.namespace() {
            writeln!(state, "}}")?;
        }

        Ok(())
    }

    pub(crate) fn header(&self) -> String {
        let mut state = String::new();

        self.header_helper(&mut state).unwrap();

        state
    }
}

impl Display for CppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;
        if !self.generic_args.is_empty() {
            write!(f, "< {} >", self.generic_args.iter().join(", "))?;
        }
        Ok(())
    }
}

fn split_string(input: &str) -> impl Iterator<Item = String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut parentheses_count = 0;

    for c in input.chars() {
        match c {
            ',' if parentheses_count == 0 => {
                parts.push(current_part.clone());
                current_part.clear();
            }
            '<' => {
                parentheses_count += 1;
                current_part.push(c);
            }
            '>' => {
                parentheses_count -= 1;
                current_part.push(c);
            }
            _ => {
                current_part.push(c);
            }
        }
    }

    if !current_part.is_empty() {
        parts.push(current_part);
    }

    parts.into_iter()
}

impl From<&str> for CppType {
    fn from(value: &str) -> Self {
        let value = value.trim();
        match value.split_once('<') {
            None => CppType {
                path: CppPath::from(value),
                generic_args: vec![],
            },
            Some((path, generics)) => {
                let generics = generics.strip_suffix('>').unwrap();
                CppType {
                    path: CppPath::from(path),
                    generic_args: split_string(generics).map(|x| CppType::from(&*x)).collect(),
                }
            }
        }
    }
}

// pub(crate) just for migration
pub(crate) struct State {
    pub(crate) text: String,
    pub(crate) panic_to_exception: Option<PanicToExceptionSymbols>,
}

impl State {
    fn remove_no_except_in_panic(&mut self) {
        if self.panic_to_exception.is_some() {
            // Remove noexcept when converting panics to exceptions
            // Handle various cases: " noexcept ", " noexcept;", " noexcept{", etc.
            self.text = self.text.replace(" noexcept ", " ");
            self.text = self.text.replace(" noexcept;", ";");
            self.text = self.text.replace(" noexcept{", "{");
            self.text = self.text.replace(" noexcept}", "}");
        }
    }
}

impl Write for State {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.text += s;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CppTraitMethod {
    pub name: String,
    pub rust_link_name: String,
    pub inputs: Vec<CppType>,
    pub output: CppType,
}

#[derive(Debug)]
pub struct CppFnSig {
    pub rust_link_name: String,
    pub inputs: Vec<CppType>,
    pub output: CppType,
}

pub struct CppFnDefinition {
    pub name: CppPath,
    pub sig: CppFnSig,
}

pub struct CppExportedFnDefinition {
    pub name: String,
    pub sig: CppFnSig,
}

pub struct CppExportedImplDefinition {
    pub tr: Option<CppType>,
    pub ty: CppType,
    pub methods: Vec<(String, CppFnSig)>,
}

#[derive(Debug)]
pub struct CppMethod {
    pub name: String,
    pub kind: ZngurMethodReceiver,
    pub sig: CppFnSig,
}

#[derive(Debug)]
pub enum CppTraitDefinition {
    Fn {
        sig: CppFnSig,
    },
    Normal {
        as_ty: CppType,
        methods: Vec<CppTraitMethod>,
        link_name: String,
        link_name_ref: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CppLayoutPolicy {
    StackAllocated {
        size: usize,
        align: usize,
    },
    HeapAllocated {
        size_fn: String,
        alloc_fn: String,
        free_fn: String,
    },
    OnlyByRef,
}

#[derive(Debug)]
pub struct CppTypeDefinition {
    pub ty: CppType,
    pub layout: CppLayoutPolicy,
    pub methods: Vec<CppMethod>,
    pub constructors: Vec<CppFnSig>,
    pub fields: Vec<ZngurField>,
    pub from_trait: Option<RustTrait>,
    pub from_trait_ref: Option<RustTrait>,
    pub wellknown_traits: Vec<ZngurWellknownTraitData>,
    pub cpp_value: Option<CppValue>,
    pub cpp_ref: Option<CppRef>,
}

impl Default for CppTypeDefinition {
    fn default() -> Self {
        Self {
            ty: CppType::from("fill::me::you::forgot::it"),
            layout: CppLayoutPolicy::OnlyByRef,
            methods: vec![],
            constructors: vec![],
            fields: vec![],
            wellknown_traits: vec![],
            from_trait: None,
            from_trait_ref: None,
            cpp_value: None,
            cpp_ref: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PanicToExceptionSymbols {
    pub detect_panic: String,
    pub take_panic: String,
}

#[derive(Default)]
pub struct CppFile {
    pub header_file_name: String,
    pub type_defs: Vec<CppTypeDefinition>,
    pub trait_defs: HashMap<RustTrait, CppTraitDefinition>,
    pub fn_defs: Vec<CppFnDefinition>,
    pub exported_fn_defs: Vec<CppExportedFnDefinition>,
    pub exported_impls: Vec<CppExportedImplDefinition>,
    pub additional_includes: String,
    pub panic_to_exception: Option<PanicToExceptionSymbols>,
}

impl CppFile {
    fn emit_h_file(&self, state: &mut State, cpp_namespace: &str) -> std::fmt::Result {
        let template = CppHeaderTemplate {
            cpp_namespace: &cpp_namespace.to_owned(),
            panic_to_exception: &self.panic_to_exception,
            fn_deps: &self.fn_defs,
            type_defs: &self.type_defs,
            trait_defs: &self.trait_defs,
            exported_impls: &self.exported_impls,
            exported_fn_defs: &self.exported_fn_defs,
        };
        state.text += template.render().unwrap().as_str();
        Ok(())
    }

    fn emit_zngur_h_file(&self) -> String {
        let template = ZngurHeaderTemplate {
            additional_includes: &self.additional_includes,
        };
        template.render().unwrap()
    }

    fn emit_cpp_file(&self, state: &mut State, is_really_needed: &mut bool) -> std::fmt::Result {
        let template = CppSourceTemplate {
            header_file_name: &self.header_file_name,
            trait_defs: &self.trait_defs,
            exported_fn_defs: &self.exported_fn_defs,
            exported_impls: &self.exported_impls,
        };
        state.text += template.render().unwrap().as_str();

        *is_really_needed = !self.trait_defs.is_empty()
            || !self.exported_fn_defs.is_empty()
            || !self.exported_impls.is_empty();

        Ok(())
    }

    pub fn render(self, cpp_namespace: &str) -> (String, String, Option<String>) {
        let mut h_file = State {
            text: "".to_owned(),
            panic_to_exception: self.panic_to_exception.clone(),
        };
        let mut cpp_file = State {
            text: "".to_owned(),
            panic_to_exception: self.panic_to_exception.clone(),
        };
        let mut zngur_h_state = State {
            text: self.emit_zngur_h_file(),
            panic_to_exception: self.panic_to_exception.clone(),
        };
        self.emit_h_file(&mut h_file, cpp_namespace).unwrap();
        let mut is_cpp_needed = false;
        self.emit_cpp_file(&mut cpp_file, &mut is_cpp_needed)
            .unwrap();
        h_file.remove_no_except_in_panic();
        zngur_h_state.remove_no_except_in_panic();
        (
            h_file.text,
            zngur_h_state.text,
            is_cpp_needed.then_some(cpp_file.text),
        )
    }
}

pub fn cpp_handle_keyword(name: &str) -> &str {
    match name {
        "new" => "new_",
        "default" => "default_",
        x => x,
    }
}

pub fn cpp_handle_field_name(name: &str) -> String {
    if name.parse::<u32>().is_ok() {
        return format!("f{name}");
    }
    cpp_handle_keyword(name).to_owned()
}
