use std::collections::HashMap;

use crate::cpp::{
    CppExportedFnDefinition, CppExportedImplDefinition, CppFile, CppFnDefinition, CppLayoutPolicy,
    CppTraitDefinition, CppTypeDefinition,
};
use crate::rust::IntoCpp;
use askama::Template;
use zngur_def::{CppValue, Mutability, RustTrait, ZngurMethodReceiver, ZngurWellknownTraitData};

/// TODO: docs. What are the builtin types? What do they guarantee / represent?
/// When should we change this list?
fn builtin_types() -> Vec<String> {
    [8, 16, 32, 64]
        .into_iter()
        .flat_map(|x| [format!("int{x}_t"), format!("uint{x}_t")])
        .chain([8, 16, 32, 64].into_iter().flat_map(|x| {
            [
                format!("::rust::Ref<int{x}_t>"),
                format!("::rust::Ref<uint{x}_t>"),
                format!("::rust::RefMut<int{x}_t>"),
                format!("::rust::RefMut<uint{x}_t>"),
            ]
        }))
        .chain([
            "::rust::ZngurCppOpaqueOwnedObject".to_string(),
            "::double_t".to_string(),
            "::float_t".to_string(),
            "::size_t".to_string(),
        ])
        .collect()
}

#[derive(Template)]
#[template(path = "cpp_header.askama", escape = "none", print = "code")]
struct HeaderTemplate<'a> {
    panic_to_exception: bool,
    additional_includes: &'a str,
    builtin_types: Vec<String>,
    fn_defs: &'a [CppFnDefinition],
    type_defs: &'a [CppTypeDefinition],
    trait_defs: &'a HashMap<RustTrait, CppTraitDefinition>,
    exported_impls: &'a [CppExportedImplDefinition],
    exported_fn_defs: &'a [CppExportedFnDefinition],
}

impl<'a> HeaderTemplate<'a> {
    fn emit_cpp_fn_defs_for_type(&self, td: &CppTypeDefinition) -> String {
        td.emit_cpp_fn_defs_template(self.trait_defs, self.panic_to_exception)
    }
}

#[derive(Template)]
#[template(path = "cpp_source.askama", escape = "none", print = "code")]
struct CppFileTemplate<'a> {
    trait_defs: &'a HashMap<RustTrait, CppTraitDefinition>,
    exported_fn_defs: &'a [CppExportedFnDefinition],
    exported_impls: &'a [CppExportedImplDefinition],
}

impl CppFile {
    pub fn render_template(&self) -> (String, Option<String>) {
        let header = HeaderTemplate {
            panic_to_exception: self.panic_to_exception,
            additional_includes: &self.additional_includes,
            builtin_types: builtin_types(),
            fn_defs: &self.fn_defs,
            type_defs: &self.type_defs,
            trait_defs: &self.trait_defs,
            exported_impls: &self.exported_impls,
            exported_fn_defs: &self.exported_fn_defs,
        };

        let cpp_needed = !self.trait_defs.is_empty()
            || !self.exported_fn_defs.is_empty()
            || !self.exported_impls.is_empty();

        let cpp = CppFileTemplate {
            trait_defs: &self.trait_defs,
            exported_fn_defs: &self.exported_fn_defs,
            exported_impls: &self.exported_impls,
        };

        (
            header.render().unwrap(),
            cpp_needed.then_some(cpp.render().unwrap()),
        )
    }
}
