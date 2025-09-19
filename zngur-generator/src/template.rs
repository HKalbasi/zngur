use std::collections::HashMap;

use crate::cpp::{
    CppExportedImplDefinition, CppFnDefinition, CppLayoutPolicy, CppTraitDefinition,
    CppTypeDefinition, PanicToExceptionSymbols,
};
use sailfish::Template;
use zngur_def::{RustTrait, ZngurWellknownTraitData};

use itertools::Itertools;

#[derive(Template)]
#[template(path = "cpp_header.sptl", escape = false)]
pub(crate) struct CppHeaderTemplate<'a> {
    pub(crate) panic_to_exception: &'a Option<PanicToExceptionSymbols>,
    pub(crate) additional_includes: &'a String,
    pub(crate) fn_deps: &'a Vec<CppFnDefinition>,
    pub(crate) type_defs: &'a Vec<CppTypeDefinition>,
    pub(crate) trait_defs: &'a HashMap<RustTrait, CppTraitDefinition>,
    pub(crate) exported_impls: &'a Vec<CppExportedImplDefinition>,
}

impl<'a> CppHeaderTemplate<'a> {
    // TODO: Docs - what do these represent? When will we change this list?
    fn builtin_types(&self) -> Vec<String> {
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

    fn emit_type_old(&self, td: &CppTypeDefinition) -> String {
        let mut state = crate::cpp::State {
            text: String::new(),
            panic_to_exception: self.panic_to_exception.clone(),
        };
        td.emit(&mut state).unwrap();
        state.text
    }
}
