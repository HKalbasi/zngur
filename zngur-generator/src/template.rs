use crate::cpp::PanicToExceptionSymbols;
use sailfish::Template;

#[derive(Template)]
#[template(path = "cpp_header.sptl", escape = false)]
pub(crate) struct CppHeaderTemplate {
    pub(crate) panic_to_exception: Option<PanicToExceptionSymbols>,
    pub(crate) additional_includes: String,
}

impl CppHeaderTemplate {
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
}
