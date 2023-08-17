use std::fmt::{Display, Write};

use iter_tools::Itertools;

pub struct CppPath(pub Vec<String>);

impl CppPath {
    fn namespace(&self) -> &[String] {
        self.0.split_last().unwrap().1
    }

    fn emit_in_namespace(
        &self,
        state: &mut State,
        f: impl FnOnce(&mut State) -> std::fmt::Result,
    ) -> std::fmt::Result {
        for p in self.namespace() {
            writeln!(state, "namespace {} {{", p)?;
        }
        f(state)?;
        for _ in self.namespace() {
            writeln!(state, "}}")?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        self.0.split_last().unwrap().0
    }

    fn is_rust(&self) -> bool {
        self.0.first().map(|x| x.as_str()) == Some("rust")
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

pub struct CppType {
    pub path: CppPath,
    pub generic_args: Vec<CppType>,
}

impl CppType {
    fn emit_header(&self, state: &mut State) -> std::fmt::Result {
        for x in &self.generic_args {
            x.emit_header(state)?;
        }
        if !self.path.is_rust() {
            return Ok(());
        }
        self.path.emit_in_namespace(state, |state| {
            if !self.generic_args.is_empty() {
                writeln!(
                    state,
                    "template<{}>",
                    (0..self.generic_args.len())
                        .map(|n| format!("typename T{n}"))
                        .join(", ")
                )?;
            }
            writeln!(state, "struct {};", self.path.name())
        })
    }
}

impl Display for CppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;
        if !self.generic_args.is_empty() {
            write!(f, "<{}>", self.generic_args.iter().join(", "))?;
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
        match value.split_once("<") {
            None => CppType {
                path: CppPath::from(value),
                generic_args: vec![],
            },
            Some((path, generics)) => {
                let generics = generics.strip_suffix(">").unwrap();
                CppType {
                    path: CppPath::from(path),
                    generic_args: split_string(generics).map(|x| CppType::from(&*x)).collect(),
                }
            }
        }
    }
}

struct State {
    text: String,
}

impl Write for State {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.text += s;
        Ok(())
    }
}

pub struct CppTraitMethod {
    pub name: String,
    pub inputs: Vec<CppType>,
    pub output: CppType,
}

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
        writeln!(state, "uint8_t* o);")?;
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
            "{output} {fn_name}({input_defs})
        {{
            {output} o;
            ::rust::__zngur_internal_assume_init(o);
            {rust_link_name}({input_args}::rust::__zngur_internal_data_ptr(o));
            {deinits}
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

impl CppFnDefinition {
    fn emit_cpp_def(&self, state: &mut State) -> std::fmt::Result {
        self.name.emit_in_namespace(state, |state| {
            self.sig.emit_cpp_def(state, self.name.name())
        })
    }
}

#[derive(PartialEq, Eq)]
pub enum CppMethodKind {
    StaticOnly,
    Lvalue,
    Rvalue,
}

pub struct CppMethod {
    pub name: String,
    pub kind: CppMethodKind,
    pub sig: CppFnSig,
}

pub struct BuildFromFunction {
    pub sig: CppFnSig,
}

pub struct CppTraitDefinition {
    pub as_ty: CppType,
    pub methods: Vec<CppTraitMethod>,
    pub link_name: String,
}

impl CppTraitDefinition {
    fn emit(&self, state: &mut State) -> std::fmt::Result {
        write!(
            state,
            r#"
namespace rust {{
template<typename T>
class Impl<T, {}> {{
public:
    T self;
    Impl(T&& val) : self(val) {{}}
"#,
            self.as_ty,
        )?;
        for method in &self.methods {
            write!(
                state,
                r#"
        {output} {name}({input});
"#,
                output = method.output,
                name = method.name,
                input = method
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(n, x)| format!("{x} i{n}"))
                    .join(", "),
            )?;
        }
        write!(
            state,
            r#"
}};
}};
"#,
        )?;
        Ok(())
    }
}

pub struct CppTypeDefinition {
    pub ty: CppType,
    pub size: usize,
    pub align: usize,
    pub is_copy: bool,
    pub methods: Vec<CppMethod>,
    pub from_function: Option<BuildFromFunction>,
    pub from_trait: Option<CppTraitDefinition>,
}

impl Default for CppTypeDefinition {
    fn default() -> Self {
        Self {
            ty: CppType::from("fill::me::you::forgot::it"),
            size: 0,
            align: 0,
            is_copy: false,
            methods: vec![],
            from_function: None,
            from_trait: None,
        }
    }
}

impl CppTypeDefinition {
    fn emit(&self, state: &mut State) -> std::fmt::Result {
        writeln!(
            state,
            r#"
namespace rust {{
    template<>
    uint8_t* __zngur_internal_data_ptr({ty}& t);
    template<>
    void __zngur_internal_assume_init({ty}& t);
    template<>
    void __zngur_internal_assume_deinit({ty}& t);
}}"#,
            ty = self.ty,
        )?;
        if let Some(from_trait) = &self.from_trait {
            from_trait.emit(state)?;
        }
        self.ty.path.emit_in_namespace(state, |state| {
            if self.ty.generic_args.is_empty() {
                write!(state, "struct {}", self.ty.path.name())?;
            } else {
                write!(
                    state,
                    "template<> struct {}<{}>",
                    self.ty.path.name(),
                    self.ty.generic_args.iter().join(", ")
                )?;
            }
            writeln!(
                state,
                r#"
{{
private:
    alignas({align}) ::std::array<uint8_t, {size}> data;
    template<typename T>
    friend uint8_t* ::rust::__zngur_internal_data_ptr(T& t);
    template<typename T>
    friend void ::rust::__zngur_internal_assume_init(T& t);
    template<typename T>
    friend void ::rust::__zngur_internal_assume_deinit(T& t);
"#,
                align = self.align,
                size = self.size,
            )?;
            if !self.is_copy {
                writeln!(state, "   bool drop_flag;")?;
            }
            writeln!(state, "public:")?;
            if !self.is_copy {
                writeln!(
                    state,
                    r#"
    {ty}() : drop_flag(false) {{}}
    ~{ty}() {{
        if (drop_flag) {{
            ::std::cout << "dropped";
        }}
    }}
    {ty}(const {ty}& other) = delete;
    {ty}& operator=(const {ty}& other) = delete;
    {ty}({ty}&& other) : drop_flag(true), data(other.data) {{
        if (!other.drop_flag) {{ ::std::terminate(); }}
        other.drop_flag = false;
    }}
    {ty}& operator=({ty}&& other) {{
        *this = {ty}(::std::move(other));
        return *this;
    }}
    "#,
                    ty = self.ty.path.name(),
                )?;
            }
            if let Some(from_function) = &self.from_function {
                // TODO: too special
                writeln!(
                    state,
                    r#"
    static {ty} build(::std::function<int32_t(int32_t)> f) {{
        auto data = new ::std::function<int32_t(int32_t)>(f);
        {ty} o;
        ::rust::__zngur_internal_assume_init(o);
        {link_name}(
            (uint8_t *)data,
            [](uint8_t *d) {{ delete (::std::function<int32_t(int32_t)> *)d; }},
            [](uint8_t *d, uint8_t *i1, uint8_t *o) {{
                int32_t *oo = (int32_t *)o;
                int32_t ii1 = *(int32_t *)i1;
                auto dd = (::std::function<int32_t(int32_t)> *)d;
                *oo = (*dd)(ii1);
            }},
            ::rust::__zngur_internal_data_ptr(o));
        return o;
    }}
    "#,
                    ty = self.ty.path.name(),
                    link_name = from_function.sig.rust_link_name,
                )?;
            }
            if let Some(from_trait) = &self.from_trait {
                // TODO: too special
                writeln!(
                    state,
                    r#"
    template<typename T>
    static {ty} build(T f) {{
        auto data = new {rust_impl}(::std::move(f));
        {ty} o;
        ::rust::__zngur_internal_assume_init(o);
        {link_name}(
            (uint8_t *)data,
            [](uint8_t *d) {{ delete ({rust_impl} *)d; }},
            [](uint8_t *d, uint8_t *o) {{
                ::std::array<uint8_t, 8> *oo = (::std::array<uint8_t, 8> *)o;
                auto dd = ({rust_impl} *)d;
                auto ooo = dd->next();
                *oo = *(::std::array<uint8_t, 8> *)::rust::__zngur_internal_data_ptr(ooo);
            }},
            ::rust::__zngur_internal_data_ptr(o));
        return o;
    }}
    "#,
                    rust_impl = format!("::rust::Impl<T, {}>", from_trait.as_ty),
                    ty = self.ty.path.name(),
                    link_name = from_trait.link_name,
                )?;
            }
            for method in &self.methods {
                write!(state, "static ")?;
                method.sig.emit_cpp_def(state, &method.name)?;
                if method.kind != CppMethodKind::StaticOnly {
                    let CppFnSig {
                        rust_link_name,
                        inputs,
                        output,
                    } = &method.sig;
                    writeln!(
                        state,
                        "{output} {fn_name}({input_defs})
                    {{
                        return {ty}::{fn_name}({this_arg}{input_args});
                    }}",
                        fn_name = &method.name,
                        ty = self.ty.path.name(),
                        this_arg = match method.kind {
                            CppMethodKind::Lvalue => "*this",
                            CppMethodKind::Rvalue => "::std::move(*this)",
                            CppMethodKind::StaticOnly => unreachable!(),
                        },
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
            writeln!(state, "}};")
        })?;
        writeln!(
            state,
            r#"
namespace rust {{
    template<>
    uint8_t* __zngur_internal_data_ptr({ty}& t) {{
        {check}
        return (uint8_t*)&t.data;
    }}

    template<>
    void __zngur_internal_assume_init({ty}& t) {{
        {assume_init}
    }}

    template<>
    void __zngur_internal_assume_deinit({ty}& t) {{
        {assume_deinit}
    }}
}}"#,
            ty = self.ty,
            check = if self.is_copy {
                ""
            } else {
                "if (!t.drop_flag) { ::std::terminate(); }"
            },
            assume_init = if self.is_copy {
                ""
            } else {
                "t.drop_flag = true;"
            },
            assume_deinit = if self.is_copy {
                ""
            } else {
                "t.drop_flag = false;"
            },
        )?;

        Ok(())
    }

    fn emit_rust_links(&self, state: &mut State) -> std::fmt::Result {
        for method in &self.methods {
            method.sig.emit_rust_link(state)?;
        }
        if let Some(ff) = &self.from_trait {
            let CppTraitDefinition {
                as_ty,
                methods,
                link_name,
            } = ff;
            // TODO: too special
            writeln!(
                state,
                "void {link_name}(uint8_t *data, void destructor(uint8_t *),
            void f_next(uint8_t *, uint8_t *),
            uint8_t *o);"
            )?;
        }
        if let Some(ff) = &self.from_function {
            let BuildFromFunction {
                sig:
                    CppFnSig {
                        rust_link_name,
                        inputs,
                        output,
                    },
            } = ff;
            // TODO: too special
            writeln!(
                state,
                "void {rust_link_name}(uint8_t *data, void destructor(uint8_t *),
            void call(uint8_t *, uint8_t *, uint8_t *),
            uint8_t *o);"
            )?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct CppFile {
    pub type_defs: Vec<CppTypeDefinition>,
    pub fn_defs: Vec<CppFnDefinition>,
}

impl CppFile {
    fn emit(&self, state: &mut State) -> std::fmt::Result {
        state.text += r#"
#include <cstddef>
#include <cstdint>
#include <array>
#include <iostream>
#include <functional>

namespace rust {
    template<typename T>
    uint8_t* __zngur_internal_data_ptr(T& t);

    template<typename T>
    void __zngur_internal_assume_init(T& t);

    template<typename T>
    void __zngur_internal_assume_deinit(T& t);

    template<typename T>
    struct Ref {
        Ref(T& t) {
            data = (size_t)__zngur_internal_data_ptr(t);
        }
        private:
            size_t data;
        template<typename T2>
        friend uint8_t* ::rust::__zngur_internal_data_ptr(::rust::Ref<T2>& t);
    };

    template<typename T>
    uint8_t* __zngur_internal_data_ptr(::rust::Ref<T>& t) {
        return (uint8_t*)&t.data;
    }

    template<typename T>
    void __zngur_internal_assume_init(::rust::Ref<T>& t) {}

    template<typename T>
    void __zngur_internal_assume_deinit(::rust::Ref<T>& t) {}

    uint8_t* __zngur_internal_data_ptr(int32_t& t) {
        return (uint8_t*)&t;
    }

    void __zngur_internal_assume_init(int32_t& t) {}
    void __zngur_internal_assume_deinit(int32_t& t) {}

    template<typename Type, typename Trait>
    class Impl;
}

"#;
        writeln!(state, "extern \"C\" {{")?;
        for f in &self.fn_defs {
            f.sig.emit_rust_link(state)?;
        }
        for td in &self.type_defs {
            td.emit_rust_links(state)?;
        }
        writeln!(state, "}}")?;
        for td in &self.type_defs {
            td.ty.emit_header(state)?;
        }
        // for fd in &self.trait_defs {
        //     fd.as_ty.emit_header(state)?;
        // }
        for td in &self.type_defs {
            td.emit(state)?;
        }
        for fd in &self.fn_defs {
            fd.emit_cpp_def(state)?;
        }
        // for fd in &self.trait_defs {
        //     fd.emit(state)?;
        // }
        Ok(())
    }

    pub fn render(self) -> String {
        let mut state = State {
            text: "".to_owned(),
        };
        self.emit(&mut state).unwrap();
        state.text
    }
}

pub fn cpp_handle_keyword(name: &str) -> &str {
    match name {
        "new" => "new_",
        x => x,
    }
}
