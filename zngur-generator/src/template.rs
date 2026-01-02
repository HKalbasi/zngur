use std::collections::HashMap;

use crate::cpp::{
    CppExportedFnDefinition, CppExportedImplDefinition, CppFnDefinition, CppFnSig, CppLayoutPolicy,
    CppTraitDefinition, CppTypeDefinition, PanicToExceptionSymbols, cpp_handle_field_name,
};
use sailfish::Template;
use zngur_def::*;
use zngur_def::{Mutability, ZngurMethodReceiver, ZngurWellknownTraitData};

use crate::rust::IntoCpp;
use itertools::Itertools;

/// Macro for template string interpolation over iterables with enumeration
///
/// Usage:
/// - splat!(items, |idx, ty|, "{ty} param{idx}")
/// - splat!(items.iter().skip(1), |n, t|, "{t} i{n}")
/// - splat!(items, "{el} i{n}")  // Default names: n (index), el (element)
macro_rules! splat {
    // Closure-style with custom variable names
    ($inputs:expr, |$n:ident, $el:ident|, $pattern:literal) => {{
        use itertools::Itertools;
        $inputs
            .into_iter()
            .enumerate()
            .map(|($n, $el)| format!($pattern))
            .join(", ")
    }};

    ($inputs:expr, |$n:ident, _|, $pattern:literal) => {{
      use itertools::Itertools;
      $inputs
          .into_iter()
          .enumerate()
          .map(|($n, _)| format!($pattern))
          .join(", ")
  }};

    // Default names fallback
    ($inputs:expr, $pattern:literal) => {
        splat!($inputs, |n, el|, $pattern)
    };
}

#[derive(Template)]
#[template(path = "cpp_header.sptl", escape = false)]
pub(crate) struct CppHeaderTemplate<'a> {
    pub(crate) panic_to_exception: &'a Option<PanicToExceptionSymbols>,
    pub(crate) additional_includes: &'a String,
    pub(crate) fn_deps: &'a Vec<CppFnDefinition>,
    pub(crate) type_defs: &'a Vec<CppTypeDefinition>,
    pub(crate) trait_defs: &'a HashMap<RustTrait, CppTraitDefinition>,
    pub(crate) exported_impls: &'a Vec<CppExportedImplDefinition>,
    pub(crate) exported_fn_defs: &'a Vec<CppExportedFnDefinition>,
    pub(crate) rust_cfg_defines: &'a Vec<String>,
}

impl<'a> CppHeaderTemplate<'a> {
    // TODO: Docs - what do these represent? When will we change this list?
    fn builtin_types(&self) -> Vec<String> {
        let builtins = [8, 16, 32, 64]
            .into_iter()
            .flat_map(|x| [format!("int{x}_t"), format!("uint{x}_t")])
            .chain(["::double_t".to_owned(), "::float_t".to_owned()])
            .flat_map(|x| {
                [
                    x.clone(),
                    format!("::rust::Ref<{x}>"),
                    format!("::rust::RefMut<{x}>"),
                ]
            });
        builtins
            .chain([
                "::rust::ZngurCppOpaqueOwnedObject".to_owned(),
                "::size_t".to_owned(),
            ])
            .collect()
    }

    fn panic_handler(&self) -> String {
        if let Some(symbols) = &self.panic_to_exception {
            format!(
                r#"
            if ({}()) {{
                {}();
                throw ::rust::Panic{{}};
            }}
            "#,
                symbols.detect_panic, symbols.take_panic,
            )
        } else {
            "".to_owned()
        }
    }
}

#[derive(Template)]
#[template(path = "cpp_source.sptl", escape = false)]
pub(crate) struct CppSourceTemplate<'a> {
    pub(crate) header_file_name: &'a String,
    pub(crate) trait_defs: &'a HashMap<RustTrait, CppTraitDefinition>,
    pub(crate) exported_fn_defs: &'a Vec<CppExportedFnDefinition>,
    pub(crate) exported_impls: &'a Vec<CppExportedImplDefinition>,
}
