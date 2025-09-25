use std::{
    collections::HashMap,
    fmt::{Display, Write},
    iter,
};

use itertools::Itertools;
use zngur_def::{CppRef, CppValue, Mutability, RustTrait, ZngurField, ZngurMethodReceiver};

use crate::{ZngurWellknownTraitData, rust::IntoCpp, template::CppHeaderTemplate};
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

    fn emit_open_namespace(&self, state: &mut State) -> std::fmt::Result {
        for p in self.namespace() {
            writeln!(state, "namespace {} {{", p)?;
        }
        Ok(())
    }

    fn emit_close_namespace(&self, state: &mut State) -> std::fmt::Result {
        for _ in self.namespace() {
            writeln!(state, "}}")?;
        }
        Ok(())
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

impl CppFnSig {
    fn emit_rust_link(&self, state: &mut State) -> std::fmt::Result {
        write!(state, "void {}(", self.rust_link_name)?;
        for n in 0..self.inputs.len() {
            write!(state, "uint8_t* i{n},")?;
        }
        write!(state, "uint8_t* o)")?;
        Ok(())
    }

    fn emit_cpp_def(&self, state: &mut State, fn_name: &str) -> std::fmt::Result {
        let CppFnSig {
            inputs,
            output,
            rust_link_name,
        } = self;
        writeln!(
            state,
            "inline {output} {fn_name}({input_defs}) noexcept {{
            {output} o{{}};
            {deinits}
            {rust_link_name}({input_args}::rust::__zngur_internal_data_ptr(o));
            {panic_handler}
            ::rust::__zngur_internal_assume_init(o);
            return o;
        }}",
            input_defs = inputs
                .iter()
                .enumerate()
                .map(|(n, ty)| format!("{ty} i{n}"))
                .join(", "),
            input_args = (0..inputs.len())
                .map(|n| format!("::rust::__zngur_internal_data_ptr(i{n}), "))
                .join(""),
            panic_handler = state.panic_handler(),
            deinits = (0..inputs.len())
                .map(|n| format!("::rust::__zngur_internal_assume_deinit(i{n});"))
                .join("\n"),
        )
    }
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

impl CppFnDefinition {
    fn emit_cpp_def(&self, state: &mut State) -> std::fmt::Result {
        self.name.emit_open_namespace(state)?;
        self.sig.emit_cpp_def(state, self.name.name())?;
        self.name.emit_close_namespace(state)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CppMethod {
    pub name: String,
    pub kind: ZngurMethodReceiver,
    pub sig: CppFnSig,
}

#[derive(Debug)]
pub struct BuildFromFunction {
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

impl CppTraitDefinition {
    fn emit_cpp(&self, state: &mut State) -> std::fmt::Result {
        match self {
            CppTraitDefinition::Fn { .. } => (),
            CppTraitDefinition::Normal {
                as_ty,
                methods,
                link_name: _,
                link_name_ref: _,
            } => {
                for method in methods {
                    write!(state, "void {}(uint8_t* data", method.rust_link_name)?;
                    for arg in 0..method.inputs.len() {
                        write!(state, ", uint8_t* i{arg}")?;
                    }
                    writeln!(state, ", uint8_t* o) {{")?;
                    writeln!(
                        state,
                        "   {as_ty}* data_typed = reinterpret_cast< {as_ty}* >(data);"
                    )?;
                    write!(
                        state,
                        "   {} oo = data_typed->{}({});",
                        method.output,
                        method.name,
                        method
                            .inputs
                            .iter()
                            .enumerate()
                            .map(|(n, ty)| {
                                format!("::rust::__zngur_internal_move_from_rust< {ty} >(i{n})")
                            })
                            .join(", ")
                    )?;
                    writeln!(state, "   ::rust::__zngur_internal_move_to_rust(o, oo);")?;
                    writeln!(state, "}}")?;
                }
            }
        }
        Ok(())
    }
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

impl CppTypeDefinition {
    fn emit_field_specialization(&self, state: &mut State) -> std::fmt::Result {
        for field_kind in ["FieldOwned", "FieldRef", "FieldRefMut"] {
            writeln!(
                state,
                r#"
    namespace rust {{
    template<size_t OFFSET>
    struct {field_kind}< {ty}, OFFSET > {{
                "#,
                ty = self.ty,
            )?;
            for field in &self.fields {
                writeln!(
                    state,
                    "[[no_unique_address]] {field_kind}<{}, OFFSET + {}> {};",
                    field.ty.into_cpp(),
                    field.offset,
                    cpp_handle_field_name(&field.name),
                )?;
            }
            for method in &self.methods {
                if let ZngurMethodReceiver::Ref(m) = method.kind {
                    if m == Mutability::Mut && field_kind == "FieldRef" {
                        continue;
                    }
                    let CppFnSig {
                        rust_link_name: _,
                        inputs,
                        output,
                    } = &method.sig;
                    writeln!(
                        state,
                        "{output} {fn_name}({input_defs}) const noexcept ;",
                        fn_name = &method.name,
                        input_defs = inputs
                            .iter()
                            .skip(1)
                            .enumerate()
                            .map(|(n, ty)| format!("{ty} i{n}"))
                            .join(", "),
                    )?;
                }
            }
            writeln!(state, "}};\n}}")?;
        }
        Ok(())
    }

    fn emit_ref_specialization(&self, state: &mut State) -> std::fmt::Result {
        let is_unsized = self
            .wellknown_traits
            .contains(&ZngurWellknownTraitData::Unsized);
        if self.ty.path.to_string() == "::rust::Str" {
            writeln!(
                state,
                r#"
    auto operator""_rs(const char* input, size_t len) -> ::rust::Ref<::rust::Str>;
"#,
            )?;
        }
        if is_unsized {
            writeln!(
                state,
                r#"
namespace rust {{
template<>
struct Ref< {ty} > {{
    Ref() {{
        data = {{0, 0}};
    }}
private:
    ::std::array<size_t, 2> data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< {ty} > >(const ::rust::Ref< {ty} >& t) noexcept ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< {ty} > >;
"#,
                ty = self.ty,
            )?;
        } else {
            writeln!(
                state,
                r#"
namespace rust {{
template<>
struct Ref< {ty} > {{
    Ref() {{
        data = 0;
    }}
"#,
                ty = self.ty,
            )?;
            if !matches!(self.layout, CppLayoutPolicy::OnlyByRef) {
                writeln!(
                    state,
                    r#"
    Ref(const {ty}& t) {{
        ::rust::__zngur_internal_check_init< {ty} >(t);
        data = reinterpret_cast<size_t>(__zngur_internal_data_ptr(t));
    }}
"#,
                    ty = self.ty,
                )?;
            }
            for field in &self.fields {
                writeln!(
                    state,
                    "[[no_unique_address]] ::rust::FieldRef<{}, {}> {};",
                    field.ty.into_cpp(),
                    field.offset,
                    cpp_handle_field_name(&field.name),
                )?;
            }
            writeln!(
                state,
                r#"
private:
    size_t data;
    friend uint8_t* ::rust::__zngur_internal_data_ptr< ::rust::Ref< {ty} > >(const ::rust::Ref< {ty} >& t) noexcept ;
    friend ::rust::ZngurPrettyPrinter< ::rust::Ref< {ty} > >;
"#,
                ty = self.ty,
            )?;
        }
        writeln!(state, "public:")?;
        writeln!(
            state,
            r#"
    Ref(RefMut< {ty} > rm) {{
        data = rm.data;
    }}
    "#,
            ty = self.ty,
        )?;
        if !is_unsized {
            writeln!(
                state,
                r#"
    template<size_t OFFSET>
    Ref(const FieldOwned< {ty}, OFFSET >& f) {{
        data = reinterpret_cast<size_t>(&f) + OFFSET;
    }}

    template<size_t OFFSET>
    Ref(const FieldRef< {ty}, OFFSET >& f) {{
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }}

    template<size_t OFFSET>
    Ref(const FieldRefMut< {ty}, OFFSET >& f) {{
        data = *reinterpret_cast<const size_t*>(&f) + OFFSET;
    }}
    "#,
                ty = self.ty,
            )?;
        }
        match &self.from_trait_ref {
            Some(RustTrait::Fn { inputs, output, .. }) => {
                let as_std_function = format!(
                    "::std::function< {}({})>",
                    output.into_cpp(),
                    inputs.iter().map(|x| x.into_cpp()).join(", ")
                );
                writeln!(
                    state,
                    r#"
inline {ty}({as_std_function} f);
"#,
                    ty = self.ty.path.name(),
                )?;
            }
            Some(tr @ RustTrait::Normal { .. }) => {
                let tr = tr.into_cpp();
                writeln!(
                    state,
                    r#"
            inline Ref({tr}& arg);
            "#,
                )?;
            }
            None => (),
        }
        if let Some(CppValue(rust_link_name, cpp_ty)) = &self.cpp_value {
            writeln!(
                state,
                r#"
                inline {cpp_ty}& cpp() {{
                    return (*{rust_link_name}(reinterpret_cast<uint8_t*>(data))).as_cpp< {cpp_ty} >();
                }}"#
            )?;
        }
        if let Some(cpp_ty) = &self.cpp_ref {
            writeln!(
                state,
                r#"
                inline {cpp_ty}& cpp() {{
                    return *reinterpret_cast< {cpp_ty}* >(data);
                }}"#
            )?;
            writeln!(
                state,
                r#"
                inline Ref(const {cpp_ty}& t) : data(reinterpret_cast<size_t>(&t)) {{}}"#
            )?;
        }
        for method in &self.methods {
            if let ZngurMethodReceiver::Ref(m) = method.kind {
                if m == Mutability::Mut {
                    continue;
                }
                let CppFnSig {
                    rust_link_name: _,
                    inputs,
                    output,
                } = &method.sig;
                writeln!(
                    state,
                    "{output} {fn_name}({input_defs}) const noexcept ;",
                    fn_name = &method.name,
                    input_defs = inputs
                        .iter()
                        .skip(1)
                        .enumerate()
                        .map(|(n, ty)| format!("{ty} i{n}"))
                        .join(", "),
                )?;
            }
        }
        if self.ty.path.to_string() == "::rust::Str" {
            writeln!(
                state,
                r#"
    friend auto ::operator""_rs(const char* input, size_t len) -> ::rust::Ref<::rust::Str>;
}};
"#,
            )?;
        } else {
            writeln!(state, "}};")?;
        }
        writeln!(
            state,
            r#"
template<>
inline uint8_t* __zngur_internal_data_ptr< Ref < {ty} > >(const Ref< {ty} >& t) noexcept {{
    return const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&t.data));
}}

template<>
inline void __zngur_internal_assume_init< Ref < {ty} > >(Ref< {ty} >&) noexcept {{
}}

template<>
inline void __zngur_internal_check_init< Ref < {ty} > >(const Ref< {ty} >&) noexcept {{
}}

template<>
inline void __zngur_internal_assume_deinit< Ref < {ty} > >(Ref< {ty} >&) noexcept {{
}}

template<>
inline size_t __zngur_internal_size_of< Ref < {ty} > >() noexcept {{
    return {size};
}}
}}"#,
            ty = self.ty,
            size = if is_unsized { 16 } else { 8 },
        )?;
        if self.ty.path.to_string() == "::rust::Str" {
            writeln!(
                state,
                r#"
inline ::rust::Ref<::rust::Str> operator""_rs(const char* input, size_t len) {{
    ::rust::Ref<::rust::Str> o;
    o.data[0] = reinterpret_cast<size_t>(input);
    o.data[1] = len;
    return o;
}}
                    "#,
            )?;
        }
        Ok(())
    }

    pub(crate) fn emit(&self, state: &mut State) -> std::fmt::Result {
        self.emit_ref_specialization(state)?;
        self.emit_field_specialization(state)?;
        Ok(())
    }

    fn emit_cpp_fn_defs(
        &self,
        state: &mut State,
        traits: &HashMap<RustTrait, CppTraitDefinition>,
    ) -> std::fmt::Result {
        let is_unsized = self
            .wellknown_traits
            .contains(&ZngurWellknownTraitData::Unsized);
        let cpp_type = &self.ty.to_string();
        let my_name = cpp_type.strip_prefix("::").unwrap();
        for c in &self.constructors {
            let fn_name = my_name.to_owned() + "::" + self.ty.path.0.last().unwrap();
            let CppFnSig {
                inputs,
                output: _,
                rust_link_name,
            } = c;
            writeln!(
                state,
                "inline {fn_name}({input_defs}) noexcept {{
            ::rust::__zngur_internal_assume_init(*this);
            {rust_link_name}({input_args}::rust::__zngur_internal_data_ptr(*this));
            {deinits}
        }}",
                input_defs = inputs
                    .iter()
                    .enumerate()
                    .map(|(n, ty)| format!("{ty} i{n}"))
                    .join(", "),
                input_args = (0..inputs.len())
                    .map(|n| format!("::rust::__zngur_internal_data_ptr(i{n}), "))
                    .join(""),
                deinits = (0..inputs.len())
                    .map(|n| format!("::rust::__zngur_internal_assume_deinit(i{n});"))
                    .join("\n"),
            )?;
        }
        match self.from_trait.as_ref().and_then(|k| traits.get(k)) {
            Some(CppTraitDefinition::Fn { sig }) => {
                let as_std_function = format!(
                    "::std::function< {}({})>",
                    sig.output,
                    sig.inputs.iter().join(", ")
                );
                let ii_names = sig
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(n, x)| format!("::rust::__zngur_internal_move_from_rust< {x} >(i{n})"))
                    .join(", ");
                let uint8_t_ix = sig
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(n, _)| format!("uint8_t* i{n},"))
                    .join(" ");
                let out_ty = &sig.output;
                writeln!(
                    state,
                    r#"
{my_name} {my_name}::make_box({as_std_function} f) {{
auto data = new {as_std_function}(f);
{my_name} o;
::rust::__zngur_internal_assume_init(o);
{link_name}(
reinterpret_cast<uint8_t*>(data),
[](uint8_t *d) {{ delete reinterpret_cast< {as_std_function}*>(d); }},
[](uint8_t *d, {uint8_t_ix} uint8_t *o) {{
auto dd = reinterpret_cast< {as_std_function} *>(d);
{out_ty} oo = (*dd)({ii_names});
::rust::__zngur_internal_move_to_rust< {out_ty} >(o, oo);
}},
::rust::__zngur_internal_data_ptr(o));
return o;
}}
"#,
                    link_name = sig.rust_link_name,
                )?;
            }
            Some(CppTraitDefinition::Normal {
                as_ty,
                methods: _,
                link_name,
                link_name_ref: _,
            }) => {
                writeln!(
                    state,
                    r#"
template<typename T, typename... Args>
{my_name} {my_name}::make_box(Args&&... args) {{
auto data = new T(::std::forward<Args>(args)...);
auto data_as_impl = dynamic_cast< {as_ty}*>(data);
{my_name} o;
::rust::__zngur_internal_assume_init(o);
{link_name}(
reinterpret_cast<uint8_t*>(data_as_impl),
[](uint8_t *d) {{ delete reinterpret_cast< {as_ty} *>(d); }},
"#,
                )?;
                writeln!(
                    state,
                    r#"
::rust::__zngur_internal_data_ptr(o));
return o;
}}
"#,
                )?;
            }
            None => (),
        }
        match self.from_trait_ref.as_ref().and_then(|k| traits.get(k)) {
            Some(CppTraitDefinition::Fn { .. }) => {
                todo!()
            }
            Some(CppTraitDefinition::Normal {
                as_ty,
                methods: _,
                link_name: _,
                link_name_ref,
            }) => {
                for ref_kind in ["Ref", " RefMut"] {
                    writeln!(
                        state,
                        r#"
rust::{ref_kind}< {my_name} >::{ref_kind}({as_ty}& args) {{
auto data_as_impl = &args;
::rust::__zngur_internal_assume_init(*this);
{link_name_ref}(
(uint8_t *)data_as_impl,
"#,
                    )?;
                    writeln!(
                        state,
                        r#"
::rust::__zngur_internal_data_ptr(*this));
}}
"#,
                    )?;
                }
            }
            None => (),
        }
        for method in &self.methods {
            let fn_name = my_name.to_owned() + "::" + &method.name;
            method.sig.emit_cpp_def(state, &fn_name)?;
            if let ZngurMethodReceiver::Ref(m) = method.kind {
                let ref_kinds: &[&str] = match m {
                    Mutability::Mut => &["RefMut"],
                    Mutability::Not => &["Ref", "RefMut"],
                };
                let field_kinds: &[&str] = match m {
                    Mutability::Mut => &["FieldOwned", "FieldRefMut"],
                    Mutability::Not => &["FieldOwned", "FieldRefMut", "FieldRef"],
                };
                for field_kind in field_kinds {
                    let CppFnSig {
                        rust_link_name: _,
                        inputs,
                        output,
                    } = &method.sig;
                    writeln!(
                        state,
                        "template<size_t OFFSET>
                        inline {output} rust::{field_kind}< {ty}, OFFSET >::{method_name}({input_defs}) const noexcept {{
                    return {fn_name}(*this{input_args});
                }}",
                        ty = &self.ty,
                        method_name = &method.name,
                        input_defs = inputs
                            .iter()
                            .skip(1)
                            .enumerate()
                            .map(|(n, ty)| format!("{ty} i{n}"))
                            .join(", "),
                        input_args = (0..inputs.len() - 1)
                            .map(|n| format!(", ::std::move(i{n})"))
                            .join("")
                    )?;
                }
                for ref_kind in ref_kinds {
                    let CppFnSig {
                        rust_link_name: _,
                        inputs,
                        output,
                    } = &method.sig;
                    writeln!(
                        state,
                        "inline {output} rust::{ref_kind}< {ty} >::{method_name}({input_defs}) const noexcept {{
                    return {fn_name}(*this{input_args});
                }}",
                        ty = &self.ty,
                        method_name = &method.name,
                        input_defs = inputs
                            .iter()
                            .skip(1)
                            .enumerate()
                            .map(|(n, ty)| format!("{ty} i{n}"))
                            .join(", "),
                        input_args = (0..inputs.len() - 1)
                            .map(|n| format!(", ::std::move(i{n})"))
                            .join("")
                    )?;
                }
            }
            if !is_unsized
                && !matches!(self.layout, CppLayoutPolicy::OnlyByRef)
                && method.kind != ZngurMethodReceiver::Static
            {
                let CppFnSig {
                    rust_link_name: _,
                    inputs,
                    output,
                } = &method.sig;
                writeln!(
                    state,
                    "inline {output} {fn_name}({input_defs}) {const_kw} noexcept {{
                    return {fn_name}({this_arg}{input_args});
                }}",
                    this_arg = match method.kind {
                        ZngurMethodReceiver::Ref(_) => "*this",
                        ZngurMethodReceiver::Move => "::std::move(*this)",
                        ZngurMethodReceiver::Static => unreachable!(),
                    },
                    input_defs = inputs
                        .iter()
                        .skip(1)
                        .enumerate()
                        .map(|(n, ty)| format!("{ty} i{n}"))
                        .join(", "),
                    input_args = (0..inputs.len() - 1)
                        .map(|n| format!(", ::std::move(i{n})"))
                        .join(""),
                    const_kw = if method.kind != ZngurMethodReceiver::Ref(Mutability::Not) {
                        ""
                    } else {
                        "const"
                    },
                )?;
            }
        }
        let is_unsized = self
            .wellknown_traits
            .contains(&ZngurWellknownTraitData::Unsized);
        for tr in &self.wellknown_traits {
            match tr {
                ZngurWellknownTraitData::Debug {
                    pretty_print,
                    debug_print: _, // TODO: use it
                } => {
                    if !is_unsized {
                        writeln!(
                            state,
                            r#"
            namespace rust {{
                template<>
                struct ZngurPrettyPrinter< {ty} > {{
                    static inline void print({ty} const& t) {{
                        ::rust::__zngur_internal_check_init< {ty} >(t);
                        {pretty_print}(&t.data[0]);
                    }}
                }};

                template<>
                struct ZngurPrettyPrinter< Ref< {ty} > > {{
                    static inline void print(Ref< {ty} > const& t) {{
                        ::rust::__zngur_internal_check_init< Ref< {ty} > >(t);
                        {pretty_print}(reinterpret_cast<uint8_t*>(t.data));
                    }}
                }};

                template<>
                struct ZngurPrettyPrinter< RefMut< {ty} > > {{
                    static inline void print(RefMut< {ty} > const& t) {{
                        ::rust::__zngur_internal_check_init< RefMut< {ty} > >(t);
                        {pretty_print}(reinterpret_cast<uint8_t*>(t.data));
                    }}
                }};

                template<size_t OFFSET>
                struct ZngurPrettyPrinter< FieldOwned< {ty}, OFFSET > > {{
                    static inline void print(FieldOwned< {ty}, OFFSET > const& t) {{
                        ZngurPrettyPrinter< Ref< {ty} > >::print(t);
                    }}
                }};

                template<size_t OFFSET>
                struct ZngurPrettyPrinter< FieldRef< {ty}, OFFSET > > {{
                    static inline void print(FieldRef< {ty}, OFFSET > const& t) {{
                        ZngurPrettyPrinter< Ref< {ty} > >::print(t);
                    }}
                }};

                template<size_t OFFSET>
                struct ZngurPrettyPrinter< FieldRefMut< {ty}, OFFSET > > {{
                    static inline void print(FieldRefMut< {ty}, OFFSET > const& t) {{
                        ZngurPrettyPrinter< Ref< {ty} > >::print(t);
                    }}
                }};
            }}"#,
                            ty = self.ty,
                        )?;
                    } else {
                        writeln!(
                            state,
                            r#"
            namespace rust {{
                template<>
                struct ZngurPrettyPrinter< Ref< {ty} > > {{
                    static inline void print(Ref< {ty} > const& t) {{
                        ::rust::__zngur_internal_check_init< Ref< {ty} > >(t);
                        {pretty_print}(::rust::__zngur_internal_data_ptr< Ref< {ty} > >(t));
                    }}
                }};

                template<>
                struct ZngurPrettyPrinter< RefMut< {ty} > > {{
                    static inline void print(RefMut< {ty} > const& t) {{
                        ::rust::__zngur_internal_check_init< RefMut< {ty} > >(t);
                        {pretty_print}(::rust::__zngur_internal_data_ptr< RefMut< {ty} > >(t));
                    }}
                }};
            }}"#,
                            ty = self.ty,
                        )?;
                    }
                }
                ZngurWellknownTraitData::Unsized
                | ZngurWellknownTraitData::Copy
                | ZngurWellknownTraitData::Drop { .. } => {}
            }
        }
        Ok(())
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
    fn emit_h_file(&self, state: &mut State) -> std::fmt::Result {
        let template = CppHeaderTemplate {
            panic_to_exception: &self.panic_to_exception,
            additional_includes: &self.additional_includes,
            fn_deps: &self.fn_defs,
            type_defs: &self.type_defs,
            trait_defs: &self.trait_defs,
            exported_impls: &self.exported_impls,
        };
        state.text += template.render().unwrap().as_str();
        for td in &self.type_defs {
            td.emit_cpp_fn_defs(state, &self.trait_defs)?;
        }
        for fd in &self.fn_defs {
            fd.emit_cpp_def(state)?;
        }
        for func in &self.exported_fn_defs {
            writeln!(state, "namespace rust {{ namespace exported_functions {{")?;
            write!(state, "   {} {}(", func.sig.output, func.name)?;
            for (n, ty) in func.sig.inputs.iter().enumerate() {
                if n != 0 {
                    write!(state, ", ")?;
                }
                write!(state, "{ty} i{n}")?;
            }
            writeln!(state, ");")?;
            writeln!(state, "}} }}")?;
        }
        for imp in &self.exported_impls {
            writeln!(
                state,
                "namespace rust {{ template<> class Impl< {}, {} > {{ public:",
                imp.ty,
                match &imp.tr {
                    Some(x) => format!("{x}"),
                    None => "::rust::Inherent".to_string(),
                }
            )?;
            for (name, sig) in &imp.methods {
                write!(state, "   static {} {}(", sig.output, name)?;
                for (n, ty) in sig.inputs.iter().enumerate() {
                    if n != 0 {
                        write!(state, ", ")?;
                    }
                    write!(state, "{ty} i{n}")?;
                }
                writeln!(state, ");")?;
            }
            writeln!(state, "}}; }}")?;
        }
        Ok(())
    }

    fn emit_cpp_file(&self, state: &mut State, is_really_needed: &mut bool) -> std::fmt::Result {
        writeln!(state, r#"#include "{}""#, self.header_file_name)?;
        writeln!(state, "extern \"C\" {{")?;
        for t in &self.trait_defs {
            *is_really_needed = true;
            t.1.emit_cpp(state)?;
        }
        for func in &self.exported_fn_defs {
            *is_really_needed = true;
            func.sig.emit_rust_link(state)?;
            writeln!(state, "{{")?;
            writeln!(
                state,
                "   {} oo = ::rust::exported_functions::{}({});",
                func.sig.output,
                func.name,
                func.sig
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(n, ty)| {
                        format!("::rust::__zngur_internal_move_from_rust< {ty} >(i{n})")
                    })
                    .join(", "),
            )?;
            writeln!(state, "   ::rust::__zngur_internal_move_to_rust(o, oo);")?;
            writeln!(state, "}}")?;
        }
        for imp in &self.exported_impls {
            *is_really_needed = true;
            for (name, sig) in &imp.methods {
                sig.emit_rust_link(state)?;
                writeln!(state, "{{")?;
                writeln!(
                    state,
                    "   {} oo = ::rust::Impl< {}, {} >::{}({});",
                    sig.output,
                    imp.ty,
                    match &imp.tr {
                        Some(x) => format!("{x}"),
                        None => "::rust::Inherent".to_string(),
                    },
                    name,
                    sig.inputs
                        .iter()
                        .enumerate()
                        .map(|(n, ty)| {
                            format!("::rust::__zngur_internal_move_from_rust< {ty} >(i{n})")
                        })
                        .join(", "),
                )?;
                writeln!(state, "   ::rust::__zngur_internal_move_to_rust(o, oo);")?;
                writeln!(state, "}}")?;
            }
        }
        writeln!(state, "}}")?;
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
