use std::{
    fmt::{Display, Write},
    iter,
};

use iter_tools::{Either, Itertools};

use crate::cpp::{cpp_handle_keyword, CppPath, CppType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarRustType {
    Uint(u32),
    Int(u32),
}

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
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
}

impl From<&str> for RustTrait {
    fn from(value: &str) -> Self {
        let ty = RustType::from(&*format!("dyn {value}"));
        match ty {
            RustType::Dyn(x) => x,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum RustType {
    Scalar(ScalarRustType),
    Ref(Box<RustType>),
    RefMut(Box<RustType>),
    Boxed(Box<RustType>),
    Dyn(RustTrait),
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
            },
            RustType::Ref(ty) => write!(f, "&{ty}"),
            RustType::RefMut(ty) => write!(f, "&mut {ty}"),
            RustType::Boxed(ty) => write!(f, "Box<{ty}>"),
            RustType::Tuple(v) => write!(f, "({})", v.iter().join(", ")),
            RustType::Adt(pg) => write!(f, "{pg}"),
            RustType::Dyn(tr) => write!(f, "dyn {tr}"),
        }
    }
}

impl From<&str> for RustType {
    fn from(value: &str) -> Self {
        use chumsky::prelude::*;

        #[derive(Debug, Clone, PartialEq, Eq)]
        enum Token<'a> {
            ColonColon,
            Arrow,
            BracketOpen,
            BracketClose,
            ParenOpen,
            ParenClose,
            And,
            Eq,
            Comma,
            Dyn,
            Ident(&'a str),
        }

        let lexer = choice([
            just::<_, _, extra::Err<Simple<'_, char>>>("::").to(Token::ColonColon),
            just("->").to(Token::Arrow),
            just("<").to(Token::BracketOpen),
            just(">").to(Token::BracketClose),
            just("(").to(Token::ParenOpen),
            just(")").to(Token::ParenClose),
            just("&").to(Token::And),
            just("=").to(Token::Eq),
            just(",").to(Token::Comma),
            just("dyn").to(Token::Dyn),
        ])
        .or(text::ident().map(|x| Token::Ident(x)))
        .padded()
        .repeated()
        .collect();

        let tokens: Vec<Token> = lexer.parse(value).into_output().unwrap();

        let as_scalar = |s: &str, head: char| -> Option<u32> {
            let s = s.strip_prefix(head)?;
            s.parse().ok()
        };

        let scalar = select! {
            Token::Ident(c) if as_scalar(c, 'u').is_some() => RustType::Scalar(ScalarRustType::Uint(as_scalar(c, 'u').unwrap())),
            Token::Ident(c) if as_scalar(c, 'i').is_some() => RustType::Scalar(ScalarRustType::Int(as_scalar(c, 'i').unwrap())),
        };

        let path = just(Token::ColonColon)
            .then(select! {
                Token::Ident(c) => c,
            })
            .map(|x| x.1.to_string())
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>();

        let parser = recursive::<_, RustType, extra::Err<Simple<'_, Token>>, _, _>(|parser| {
            let named_generic = select! {
                Token::Ident(c) => c.to_owned(),
            }
            .then_ignore(just(Token::Eq))
            .then(parser.clone())
            .map(Either::Left);
            let generics = just(Token::ColonColon).repeated().at_most(1).ignore_then(
                named_generic
                    .or(parser.clone().map(Either::Right))
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::BracketOpen), just(Token::BracketClose)),
            );
            let pg = path
                .then(generics.clone().repeated().at_most(1).collect::<Vec<_>>())
                .map(|x| {
                    let generics = x.1.into_iter().next().unwrap_or_default();
                    let (named_generics, generics) = generics.into_iter().partition_map(|x| x);
                    RustPathAndGenerics {
                        path: x.0,
                        generics,
                        named_generics,
                    }
                });
            let adt = pg.clone().map(RustType::Adt);

            let fn_args = parser
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClose))
                .then_ignore(just(Token::Arrow))
                .then(parser.clone());

            let fn_trait = select! {
                Token::Ident(c) => c,
            }
            .then(fn_args)
            .map(|x| RustTrait::Fn {
                name: x.0.to_owned(),
                inputs: x.1 .0,
                output: Box::new(x.1 .1),
            });

            let dyn_trait = just(Token::Dyn)
                .ignore_then(pg.map(RustTrait::Normal).or(fn_trait))
                .map(RustType::Dyn);
            let boxed = just(Token::Ident("Box")).then(generics).map(|(_, x)| {
                assert_eq!(x.len(), 1);
                RustType::Boxed(Box::new(x.into_iter().next().unwrap().right().unwrap()))
            });
            let unit = just(Token::ParenOpen)
                .then(just(Token::ParenClose))
                .map(|_| RustType::Tuple(vec![]));
            let reference = just(Token::And)
                .then(parser)
                .map(|x| RustType::Ref(Box::new(x.1)));
            scalar
                .or(unit)
                .or(adt)
                .or(reference)
                .or(boxed)
                .or(dyn_trait)
        });
        let (result, errors) = parser.parse(tokens.as_slice()).into_output_errors();
        match result {
            None => panic!("{errors:?}"),
            Some(x) => x,
        }
    }
}

impl RustType {
    const UNIT: Self = RustType::Tuple(Vec::new());

    pub fn into_cpp(&self) -> CppType {
        match self {
            RustType::Scalar(s) => match s {
                ScalarRustType::Uint(s) => CppType::from(&*format!("uint{s}_t")),
                ScalarRustType::Int(s) => CppType::from(&*format!("int{s}_t")),
            },
            RustType::Boxed(t) => CppType {
                path: CppPath::from("rust::Box"),
                generic_args: vec![t.into_cpp()],
            },
            RustType::Ref(t) => CppType {
                path: CppPath::from("rust::Ref"),
                generic_args: vec![t.into_cpp()],
            },
            RustType::RefMut(t) => CppType {
                path: CppPath::from("rust::Ref"),
                generic_args: vec![t.into_cpp()],
            },
            RustType::Adt(pg) => pg.into_cpp(),
            RustType::Tuple(v) => {
                if v.is_empty() {
                    return CppType::from("rust::Unit");
                }
                todo!()
            }
            RustType::Dyn(tr) => {
                let tr_as_cpp_type = tr.into_cpp_type();
                CppType {
                    path: CppPath::from("rust::Dyn"),
                    generic_args: vec![tr_as_cpp_type],
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
        (2, "<", 'x'),
        (2, ">", 'y'),
        (2, "::", 's'),
        (2, "->", 'a'),
        (2, ",", 'c'),
        (2, "(", 'p'),
        (2, ")", 'q'),
    ];
    while let Some((pos, which)) = bads.iter().filter_map(|x| Some((name.find(x.1)?, x))).min() {
        name.replace_range(pos..pos + which.1.len(), "_");
        w!(name, "{}{pos}", which.2);
    }
    name
}

impl RustFile {
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

    pub fn add_builder_for_dyn_fn(
        &mut self,
        name: &str,
        inputs: &[RustType],
        output: &RustType,
    ) -> String {
        let mangled_name = mangle_name(&inputs.iter().chain(Some(output)).join(", "));
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
    let r: Box<dyn Fn(i32) -> i32> = Box::new(move |i1| unsafe {{
        _ = &this;
        let mut i1 = ::core::mem::ManuallyDrop::new(i1);
        let mut r = ::core::mem::MaybeUninit::uninit();
        call(
            this.data,
            &mut *i1 as *mut _ as *mut _,
            r.as_mut_ptr() as *mut _,
        );
        r.assume_init()
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
    ) -> String {
        let mangled_name = mangle_name(rust_name);
        w!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {mangled_name}("#
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
        mangled_name
    }

    pub fn add_function(&mut self, rust_name: &str, arg_count: usize) -> String {
        let mangled_name = mangle_name(rust_name);
        w!(
            self,
            r#"
#[no_mangle]
pub extern "C" fn {mangled_name}("#
        );
        for n in 0..arg_count {
            w!(self, "i{n}: *mut u8, ");
        }
        w!(
            self,
            r#"o: *mut u8) {{ unsafe {{
    ::std::ptr::write(o as *mut _, {rust_name}("#
        );
        for n in 0..arg_count {
            w!(self, "::std::ptr::read(i{n} as *mut _), ");
        }
        wln!(self, ")) }} }}");
        mangled_name
    }
}

#[cfg(test)]
mod tests {
    use super::RustType;

    fn parse_pretty(s: &str) {
        let ty = RustType::from(s);
        let pretty = ty.to_string();
        assert_eq!(s, pretty);
    }

    #[test]
    fn scalar() {
        parse_pretty("&u32");
        parse_pretty("Box<i64>");
        parse_pretty("Box<dyn ::std::fmt::Debug>");
        parse_pretty("Box<dyn ::std::iter::Iterator::<Item = i32>>");
        parse_pretty("Box<dyn Fn(i32, ::hello::World) -> Box<u64>>");
        parse_pretty("::Vec::<i32>");
        parse_pretty("::std::result::Result::<::Vec::<&u32>, ::Err>");
    }
}
