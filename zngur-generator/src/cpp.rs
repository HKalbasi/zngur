use std::{
    collections::HashMap,
    fmt::{Display, Write},
    iter,
};

use itertools::Itertools;
use zngur_def::{CppRef, CppValue, RustTrait, ZngurFieldData, ZngurMethodReceiver};

use crate::{
    ZngurWellknownTraitData,
    template::{CppHeaderTemplate, CppSourceTemplate},
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

    fn need_header(&self) -> bool {
        self.0.first().map(|x| x.as_str()) == Some("rust")
            && self.0 != ["rust", "Unit"]
            && self.0 != ["rust", "Ref"]
            && self.0 != ["rust", "RefMut"]
    }

    pub(crate) fn from_rust_path(path: &[String]) -> CppPath {
        CppPath(
            iter::once("rust")
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

        if !self.path.need_header() {
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
            self.text = self.text.replace(" noexcept ", " ");
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
    pub fields: Vec<ZngurFieldData>,
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
    pub rust_cfg_defines: Vec<String>,
}

impl CppFile {
    fn emit_h_file(&self, state: &mut State) -> std::fmt::Result {
        let template = CppHeaderTemplate {
            panic_to_exception: &self.panic_to_exception,
            additional_includes: &self.additional_includes,
            fn_deps: &self.fn_defs,
            type_defs: &self.type_defs,
            trait_defs: &self.trait_defs,
            exported_impls: &self.exported_impls,
            exported_fn_defs: &self.exported_fn_defs,
            rust_cfg_defines: &self.rust_cfg_defines,
        };
        state.text += template.render().unwrap().as_str();
        Ok(())
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

    pub fn render(self) -> (String, Option<String>) {
        let mut h_file = State {
            text: "".to_owned(),
            panic_to_exception: self.panic_to_exception.clone(),
        };
        let mut cpp_file = State {
            text: "".to_owned(),
            panic_to_exception: self.panic_to_exception.clone(),
        };
        self.emit_h_file(&mut h_file).unwrap();
        let mut is_cpp_needed = false;
        self.emit_cpp_file(&mut cpp_file, &mut is_cpp_needed)
            .unwrap();
        h_file.remove_no_except_in_panic();
        (h_file.text, is_cpp_needed.then_some(cpp_file.text))
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
