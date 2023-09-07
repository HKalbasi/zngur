use std::{fmt::Display, process::exit};

use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::prelude::*;
use iter_tools::{Either, Itertools};

use zngur_def::{
    Mutability, PrimitiveRustType, RustPathAndGenerics, RustTrait, RustType, ZngurConstructor,
    ZngurExternCppFn, ZngurExternCppImpl, ZngurFile, ZngurFn, ZngurMethod, ZngurMethodReceiver,
    ZngurTrait, ZngurType, ZngurWellknownTrait,
};

pub type Span = SimpleSpan<usize>;

type ParserInput<'a> = chumsky::input::SpannedInput<Token<'a>, Span, &'a [(Token<'a>, Span)]>;

#[derive(Debug)]
pub struct ParsedZngFile<'a>(Vec<ParsedItem<'a>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParsedPathStart {
    Absolute,
    Relative,
    Crate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPath<'a> {
    start: ParsedPathStart,
    segments: Vec<&'a str>,
    span: Span,
}

impl ParsedPath<'_> {
    fn to_zngur(self, base: &[String]) -> Vec<String> {
        match self.start {
            ParsedPathStart::Absolute => self.segments.into_iter().map(|x| x.to_owned()).collect(),
            ParsedPathStart::Relative => base
                .iter()
                .map(|x| x.as_str())
                .chain(self.segments.into_iter())
                .map(|x| x.to_owned())
                .collect(),
            ParsedPathStart::Crate => ["crate"]
                .into_iter()
                .chain(self.segments)
                .map(|x| x.to_owned())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedItem<'a> {
    CppAdditionalInclude(&'a str),
    Mod {
        path: ParsedPath<'a>,
        items: Vec<ParsedItem<'a>>,
    },
    Type {
        ty: ParsedRustType<'a>,
        items: Vec<ParsedTypeItem<'a>>,
    },
    Trait {
        tr: ParsedRustTrait<'a>,
        methods: Vec<ParsedMethod<'a>>,
    },
    Fn(ParsedMethod<'a>),
    ExternCpp(Vec<ParsedExternCppItem<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedExternCppItem<'a> {
    Function(ParsedMethod<'a>),
    Impl {
        ty: ParsedRustType<'a>,
        methods: Vec<ParsedMethod<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedConstructorArgs<'a> {
    Unit,
    Tuple(Vec<ParsedRustType<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedTypeItem<'a> {
    Properties(Vec<(&'a str, usize)>),
    Traits(Vec<ZngurWellknownTrait>),
    Constructor {
        name: Option<&'a str>,
        args: ParsedConstructorArgs<'a>,
    },
    Method(ParsedMethod<'a>, Option<ParsedPath<'a>>),
    CppValue {
        field: &'a str,
        cpp_type: &'a str,
    },
    CppRef {
        cpp_type: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedMethod<'a> {
    name: &'a str,
    receiver: ZngurMethodReceiver,
    generics: Vec<ParsedRustType<'a>>,
    inputs: Vec<ParsedRustType<'a>>,
    output: ParsedRustType<'a>,
}

impl ParsedMethod<'_> {
    fn to_zngur(self, base: &[String]) -> ZngurMethod {
        ZngurMethod {
            name: self.name.to_owned(),
            generics: self
                .generics
                .into_iter()
                .map(|x| x.to_zngur(base))
                .collect(),
            receiver: self.receiver,
            inputs: self.inputs.into_iter().map(|x| x.to_zngur(base)).collect(),
            output: self.output.to_zngur(base),
        }
    }
}

impl ParsedItem<'_> {
    fn add_to_zngur_file(self, r: &mut ZngurFile, base: &[String]) {
        match self {
            ParsedItem::Mod { path, items } => {
                let base = path.to_zngur(base);
                for item in items {
                    item.add_to_zngur_file(r, &base);
                }
            }
            ParsedItem::Type { ty, items } => {
                let mut methods = vec![];
                let mut constructors = vec![];
                let mut wellknown_traits = vec![];
                let mut size = 0;
                let mut align = 0;
                let mut is_copy = false;
                let mut cpp_value = None;
                let mut cpp_ref = None;
                for item in items {
                    match item {
                        ParsedTypeItem::Properties(p) => {
                            for (key, value) in p {
                                match key {
                                    "size" => size = value,
                                    "align" => align = value,
                                    "is_copy" => {
                                        is_copy = match value {
                                            0 => false,
                                            1 => true,
                                            _ => todo!(),
                                        };
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                        ParsedTypeItem::Traits(tr) => {
                            wellknown_traits.extend(tr);
                        }
                        ParsedTypeItem::Constructor { name, args } => {
                            constructors.push(ZngurConstructor {
                                name: name.map(|x| x.to_owned()),
                                inputs: match args {
                                    ParsedConstructorArgs::Unit => vec![],
                                    ParsedConstructorArgs::Tuple(t) => t
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, t)| (i.to_string(), t.to_zngur(base)))
                                        .collect(),
                                },
                            })
                        }
                        ParsedTypeItem::Method(m, u) => {
                            methods.push((m.to_zngur(base), u.map(|x| x.to_zngur(base))));
                        }
                        ParsedTypeItem::CppValue { field, cpp_type } => {
                            cpp_value = Some((field.to_owned(), cpp_type.to_owned()));
                        }
                        ParsedTypeItem::CppRef { cpp_type } => {
                            cpp_ref = Some(cpp_type.to_owned());
                        }
                    }
                }
                let is_unsized = wellknown_traits.contains(&ZngurWellknownTrait::Unsized);
                if !is_copy && !is_unsized {
                    wellknown_traits.push(ZngurWellknownTrait::Drop);
                }
                r.types.push(ZngurType {
                    ty: ty.to_zngur(base),
                    size,
                    align,
                    is_copy,
                    methods,
                    wellknown_traits,
                    constructors,
                    cpp_value,
                    cpp_ref,
                });
            }
            ParsedItem::Trait { tr, methods } => {
                let tr = tr.to_zngur(base);
                r.traits.insert(
                    tr.clone(),
                    ZngurTrait {
                        tr,
                        methods: methods.into_iter().map(|m| m.to_zngur(base)).collect(),
                    },
                );
            }
            ParsedItem::Fn(f) => {
                let method = f.to_zngur(base);
                r.funcs.push(ZngurFn {
                    path: RustPathAndGenerics {
                        path: base.iter().chain(Some(&method.name)).cloned().collect(),
                        generics: method.generics,
                        named_generics: vec![],
                    },
                    inputs: method.inputs,
                    output: method.output,
                })
            }
            ParsedItem::ExternCpp(items) => {
                for item in items {
                    match item {
                        ParsedExternCppItem::Function(method) => {
                            let method = method.to_zngur(base);
                            r.extern_cpp_funcs.push(ZngurExternCppFn {
                                name: method.name.to_string(),
                                inputs: method.inputs,
                                output: method.output,
                            });
                        }
                        ParsedExternCppItem::Impl { ty, methods } => {
                            r.extern_cpp_impls.push(ZngurExternCppImpl {
                                ty: ty.to_zngur(base),
                                methods: methods.into_iter().map(|x| x.to_zngur(base)).collect(),
                            });
                        }
                    }
                }
            }
            ParsedItem::CppAdditionalInclude(s) => {
                r.additional_includes += s;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedRustType<'a> {
    Primitive(PrimitiveRustType),
    Ref(Mutability, Box<ParsedRustType<'a>>),
    Raw(Mutability, Box<ParsedRustType<'a>>),
    Boxed(Box<ParsedRustType<'a>>),
    Slice(Box<ParsedRustType<'a>>),
    Dyn(ParsedRustTrait<'a>, Vec<&'a str>),
    Tuple(Vec<ParsedRustType<'a>>),
    Adt(ParsedRustPathAndGenerics<'a>),
}

impl ParsedRustType<'_> {
    fn to_zngur(self, base: &[String]) -> RustType {
        match self {
            ParsedRustType::Primitive(s) => RustType::Primitive(s),
            ParsedRustType::Ref(m, s) => RustType::Ref(m, Box::new(s.to_zngur(base))),
            ParsedRustType::Raw(m, s) => RustType::Raw(m, Box::new(s.to_zngur(base))),
            ParsedRustType::Boxed(s) => RustType::Boxed(Box::new(s.to_zngur(base))),
            ParsedRustType::Slice(s) => RustType::Slice(Box::new(s.to_zngur(base))),
            ParsedRustType::Dyn(tr, bounds) => RustType::Dyn(
                tr.to_zngur(base),
                bounds.into_iter().map(|x| x.to_owned()).collect(),
            ),
            ParsedRustType::Tuple(v) => {
                RustType::Tuple(v.into_iter().map(|s| s.to_zngur(base)).collect())
            }
            ParsedRustType::Adt(s) => RustType::Adt(s.to_zngur(base)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedRustTrait<'a> {
    Normal(ParsedRustPathAndGenerics<'a>),
    Fn {
        name: &'a str,
        inputs: Vec<ParsedRustType<'a>>,
        output: Box<ParsedRustType<'a>>,
    },
}

impl ParsedRustTrait<'_> {
    fn to_zngur(self, base: &[String]) -> RustTrait {
        match self {
            ParsedRustTrait::Normal(s) => RustTrait::Normal(s.to_zngur(base)),
            ParsedRustTrait::Fn {
                name,
                inputs,
                output,
            } => RustTrait::Fn {
                name: name.to_owned(),
                inputs: inputs.into_iter().map(|s| s.to_zngur(base)).collect(),
                output: Box::new(output.to_zngur(base)),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRustPathAndGenerics<'a> {
    path: ParsedPath<'a>,
    generics: Vec<ParsedRustType<'a>>,
    named_generics: Vec<(&'a str, ParsedRustType<'a>)>,
}

impl ParsedRustPathAndGenerics<'_> {
    fn to_zngur(self, base: &[String]) -> RustPathAndGenerics {
        RustPathAndGenerics {
            path: self.path.to_zngur(base),
            generics: self
                .generics
                .into_iter()
                .map(|x| x.to_zngur(base))
                .collect(),
            named_generics: self
                .named_generics
                .into_iter()
                .map(|(name, x)| (name.to_owned(), x.to_zngur(base)))
                .collect(),
        }
    }
}

impl ParsedZngFile<'_> {
    pub fn parse<T>(
        filename: &str,
        text: &str,
        then: impl for<'a> Fn(ParsedZngFile<'a>) -> T,
    ) -> T {
        let (tokens, errs) = lexer().parse(text).into_output_errors();
        let tokens = match tokens {
            Some(tokens) => tokens,
            None => {
                let errs = errs.into_iter().map(|e| e.map_token(|c| c.to_string()));
                handle_error(errs, filename, text);
                exit(0);
            }
        };
        let (ast, errs) = file_parser()
            .map_with_span(|ast, span| (ast, span))
            .parse(tokens.as_slice().spanned((text.len()..text.len()).into()))
            .into_output_errors();
        let ast = match ast {
            Some(x) => x,
            None => {
                let errs = errs.into_iter().map(|e| e.map_token(|c| c.to_string()));
                handle_error(errs, filename, text);
                exit(0);
            }
        };
        then(ast.0)
    }

    pub fn into_zngur_file(self) -> ZngurFile {
        let mut r = ZngurFile::default();
        for item in self.0 {
            item.add_to_zngur_file(&mut r, &[]);
        }
        r
    }
}

fn handle_error<'a>(errs: impl Iterator<Item = Rich<'a, String>>, filename: &str, text: &str) {
    for e in errs {
        Report::build(ReportKind::Error, filename, e.span().start)
            .with_message(e.to_string())
            .with_label(
                Label::new((filename.to_string(), e.span().into_range()))
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .with_labels(e.contexts().map(|(label, span)| {
                Label::new((filename.to_string(), span.into_range()))
                    .with_message(format!("while parsing this {}", label))
                    .with_color(Color::Yellow)
            }))
            .finish()
            .print(sources([(filename.to_string(), text)]))
            .unwrap();
    }
    exit(101);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token<'a> {
    ColonColon,
    Arrow,
    AngleOpen,
    AngleClose,
    BracketOpen,
    BracketClose,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    And,
    Star,
    Sharp,
    Plus,
    Eq,
    Question,
    Comma,
    Semicolon,
    KwDyn,
    KwUse,
    KwMod,
    KwCrate,
    KwType,
    KwTrait,
    KwFn,
    KwMut,
    KwConst,
    KwExtern,
    KwImpl,
    Ident(&'a str),
    Str(&'a str),
    Number(usize),
}

impl<'a> Token<'a> {
    fn ident_or_kw(ident: &'a str) -> Self {
        match ident {
            "dyn" => Token::KwDyn,
            "mod" => Token::KwMod,
            "type" => Token::KwType,
            "trait" => Token::KwTrait,
            "crate" => Token::KwCrate,
            "fn" => Token::KwFn,
            "mut" => Token::KwMut,
            "const" => Token::KwConst,
            "use" => Token::KwUse,
            "extern" => Token::KwExtern,
            "impl" => Token::KwImpl,
            x => Token::Ident(x),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::ColonColon => write!(f, "::"),
            Token::Arrow => write!(f, "->"),
            Token::AngleOpen => write!(f, "<"),
            Token::AngleClose => write!(f, ">"),
            Token::BracketOpen => write!(f, "["),
            Token::BracketClose => write!(f, "]"),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::BraceOpen => write!(f, "{{"),
            Token::BraceClose => write!(f, "}}"),
            Token::And => write!(f, "&"),
            Token::Star => write!(f, "*"),
            Token::Sharp => write!(f, "#"),
            Token::Plus => write!(f, "+"),
            Token::Eq => write!(f, "="),
            Token::Question => write!(f, "?"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::KwDyn => write!(f, "dyn"),
            Token::KwUse => write!(f, "use"),
            Token::KwMod => write!(f, "mod"),
            Token::KwCrate => write!(f, "crate"),
            Token::KwType => write!(f, "type"),
            Token::KwTrait => write!(f, "trait"),
            Token::KwFn => write!(f, "fn"),
            Token::KwMut => write!(f, "mut"),
            Token::KwConst => write!(f, "const"),
            Token::KwExtern => write!(f, "extern"),
            Token::KwImpl => write!(f, "impl"),
            Token::Ident(i) => write!(f, "{i}"),
            Token::Number(n) => write!(f, "{n}"),
            Token::Str(s) => write!(f, r#""{s}""#),
        }
    }
}

fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    let token = choice([
        just("::").to(Token::ColonColon),
        just("->").to(Token::Arrow),
        just("<").to(Token::AngleOpen),
        just(">").to(Token::AngleClose),
        just("[").to(Token::BracketOpen),
        just("]").to(Token::BracketClose),
        just("(").to(Token::ParenOpen),
        just(")").to(Token::ParenClose),
        just("{").to(Token::BraceOpen),
        just("}").to(Token::BraceClose),
        just("&").to(Token::And),
        just("*").to(Token::Star),
        just("#").to(Token::Sharp),
        just("+").to(Token::Plus),
        just("=").to(Token::Eq),
        just("?").to(Token::Question),
        just(",").to(Token::Comma),
        just(";").to(Token::Semicolon),
    ])
    .or(text::ident().map(Token::ident_or_kw))
    .or(text::int(10).map(|x: &str| Token::Number(x.parse().unwrap())))
    .or(just('"')
        .ignore_then(none_of('"').repeated().map_slice(Token::Str))
        .then_ignore(just('"')));

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
        .collect()
}

fn file_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedZngFile<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    item().repeated().collect::<Vec<_>>().map(ParsedZngFile)
}

fn rust_type<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedRustType<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    let as_scalar = |s: &str, head: char| -> Option<u32> {
        let s = s.strip_prefix(head)?;
        s.parse().ok()
    };

    let scalar = select! {
        Token::Ident("bool") => PrimitiveRustType::Bool,
        Token::Ident("str") => PrimitiveRustType::Str,
        Token::Ident("ZngurCppOpaqueOwnedObject") => PrimitiveRustType::ZngurCppOpaqueOwnedObject,
        Token::Ident("usize") => PrimitiveRustType::Usize,
        Token::Ident(c) if as_scalar(c, 'u').is_some() => PrimitiveRustType::Uint(as_scalar(c, 'u').unwrap()),
        Token::Ident(c) if as_scalar(c, 'i').is_some() => PrimitiveRustType::Int(as_scalar(c, 'i').unwrap()),
        Token::Ident(c) if as_scalar(c, 'f').is_some() => PrimitiveRustType::Float(as_scalar(c, 'f').unwrap()),
    }.map(ParsedRustType::Primitive);

    recursive(|parser| {
        let pg = rust_path_and_generics(parser.clone());
        let adt = pg.clone().map(ParsedRustType::Adt);

        let dyn_trait = just(Token::KwDyn)
            .ignore_then(rust_trait(parser.clone()))
            .then(
                just(Token::Plus)
                    .ignore_then(select! {
                        Token::Ident(c) => c,
                    })
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(x, y)| ParsedRustType::Dyn(x, y));
        let boxed = just(Token::Ident("Box"))
            .then(rust_generics(parser.clone()))
            .map(|(_, x)| {
                assert_eq!(x.len(), 1);
                ParsedRustType::Boxed(Box::new(x.into_iter().next().unwrap().right().unwrap()))
            });
        let unit = just(Token::ParenOpen)
            .then(just(Token::ParenClose))
            .map(|_| ParsedRustType::Tuple(vec![]));
        let slice = parser
            .clone()
            .map(|x| ParsedRustType::Slice(Box::new(x)))
            .delimited_by(just(Token::BracketOpen), just(Token::BracketClose));
        let reference = just(Token::And)
            .ignore_then(
                just(Token::KwMut)
                    .to(Mutability::Mut)
                    .or(empty().to(Mutability::Not)),
            )
            .then(parser.clone())
            .map(|(m, x)| ParsedRustType::Ref(m, Box::new(x)));
        let raw_ptr = just(Token::Star)
            .ignore_then(
                just(Token::KwMut)
                    .to(Mutability::Mut)
                    .or(just(Token::KwConst).to(Mutability::Not)),
            )
            .then(parser)
            .map(|(m, x)| ParsedRustType::Raw(m, Box::new(x)));
        scalar
            .or(boxed)
            .or(unit)
            .or(slice)
            .or(adt)
            .or(reference)
            .or(raw_ptr)
            .or(dyn_trait)
    })
}

fn rust_generics<'a>(
    rust_type: impl Parser<'a, ParserInput<'a>, ParsedRustType<'a>, extra::Err<Rich<'a, Token<'a>, Span>>>
        + Clone,
) -> impl Parser<
    'a,
    ParserInput<'a>,
    Vec<Either<(&'a str, ParsedRustType<'a>), ParsedRustType<'a>>>,
    extra::Err<Rich<'a, Token<'a>, Span>>,
> + Clone {
    let named_generic = select! {
        Token::Ident(c) => c,
    }
    .then_ignore(just(Token::Eq))
    .then(rust_type.clone())
    .map(Either::Left);
    just(Token::ColonColon).repeated().at_most(1).ignore_then(
        named_generic
            .or(rust_type.clone().map(Either::Right))
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::AngleOpen), just(Token::AngleClose)),
    )
}

fn rust_path_and_generics<'a>(
    rust_type: impl Parser<'a, ParserInput<'a>, ParsedRustType<'a>, extra::Err<Rich<'a, Token<'a>, Span>>>
        + Clone,
) -> impl Parser<
    'a,
    ParserInput<'a>,
    ParsedRustPathAndGenerics<'a>,
    extra::Err<Rich<'a, Token<'a>, Span>>,
> + Clone {
    let generics = rust_generics(rust_type.clone());
    path()
        .then(generics.clone().repeated().at_most(1).collect::<Vec<_>>())
        .map(|x| {
            let generics = x.1.into_iter().next().unwrap_or_default();
            let (named_generics, generics) = generics.into_iter().partition_map(|x| x);
            ParsedRustPathAndGenerics {
                path: x.0,
                generics,
                named_generics,
            }
        })
}

fn fn_args<'a>(
    rust_type: impl Parser<'a, ParserInput<'a>, ParsedRustType<'a>, extra::Err<Rich<'a, Token<'a>, Span>>>
        + Clone,
) -> impl Parser<
    'a,
    ParserInput<'a>,
    (Vec<ParsedRustType<'a>>, ParsedRustType<'a>),
    extra::Err<Rich<'a, Token<'a>, Span>>,
> + Clone {
    rust_type
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::ParenOpen), just(Token::ParenClose))
        .then(
            just(Token::Arrow)
                .ignore_then(rust_type)
                .or(empty().to(ParsedRustType::Tuple(vec![]))),
        )
}

fn rust_trait<'a>(
    rust_type: impl Parser<'a, ParserInput<'a>, ParsedRustType<'a>, extra::Err<Rich<'a, Token<'a>, Span>>>
        + Clone,
) -> impl Parser<'a, ParserInput<'a>, ParsedRustTrait<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    let fn_trait = select! {
        Token::Ident(c) => c,
    }
    .then(fn_args(rust_type.clone()))
    .map(|x| ParsedRustTrait::Fn {
        name: x.0,
        inputs: x.1 .0,
        output: Box::new(x.1 .1),
    });

    let rust_trait = fn_trait.or(rust_path_and_generics(rust_type).map(ParsedRustTrait::Normal));
    rust_trait
}

fn method<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedMethod<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    just(Token::KwFn)
        .ignore_then(select! {
            Token::Ident(c) => c,
        })
        .then(
            rust_type()
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::AngleOpen), just(Token::AngleClose))
                .or(empty().to(vec![])),
        )
        .then(fn_args(rust_type()))
        .map(|((name, generics), args)| {
            let is_self = |c: &ParsedRustType<'_>| {
                if let ParsedRustType::Adt(c) = c {
                    c.path.start == ParsedPathStart::Relative
                        && &c.path.segments == &["self"]
                        && c.generics.is_empty()
                } else {
                    false
                }
            };
            let (inputs, receiver) = match args.0.get(0) {
                Some(x) if is_self(&x) => (args.0[1..].to_vec(), ZngurMethodReceiver::Move),
                Some(ParsedRustType::Ref(m, x)) if is_self(&x) => {
                    (args.0[1..].to_vec(), ZngurMethodReceiver::Ref(*m))
                }
                _ => (args.0, ZngurMethodReceiver::Static),
            };
            ParsedMethod {
                name,
                receiver,
                generics,
                inputs,
                output: args.1,
            }
        })
}

fn type_item<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    fn inner_item<'a>(
    ) -> impl Parser<'a, ParserInput<'a>, ParsedTypeItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>>
           + Clone {
        let property_item = (select! {
            Token::Ident(c) => c,
        })
        .then_ignore(just(Token::Eq))
        .then(select! {
            Token::Number(c) => c,
        });
        let properties = just(Token::Ident("properties"))
            .ignore_then(
                property_item
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(ParsedTypeItem::Properties);
        let trait_item = select! {
            Token::Ident("Debug") => ZngurWellknownTrait::Debug,
        }
        .or(just(Token::Question)
            .then(just(Token::Ident("Sized")))
            .to(ZngurWellknownTrait::Unsized));
        let traits = just(Token::Ident("wellknown_traits"))
            .ignore_then(
                trait_item
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(ParsedTypeItem::Traits);
        let constructor_args = rust_type()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::ParenOpen), just(Token::ParenClose))
            .map(ParsedConstructorArgs::Tuple)
            .or(empty().to(ParsedConstructorArgs::Unit));
        let constructor = just(Token::Ident("constructor")).ignore_then(
            (select! {
                Token::Ident(c) => Some(c),
            })
            .or(empty().to(None))
            .then(constructor_args)
            .map(|(name, args)| ParsedTypeItem::Constructor { name, args }),
        );
        let cpp_value = just(Token::Sharp)
            .then(just(Token::Ident("cpp_value")))
            .ignore_then(select! {
                Token::Str(c) => c,
            })
            .then(select! {
                Token::Str(c) => c,
            })
            .map(|x| ParsedTypeItem::CppValue {
                field: x.0,
                cpp_type: x.1,
            });
        let cpp_ref = just(Token::Sharp)
            .then(just(Token::Ident("cpp_ref")))
            .ignore_then(select! {
                Token::Str(c) => c,
            })
            .map(|x| ParsedTypeItem::CppRef { cpp_type: x });
        properties
            .or(traits)
            .or(constructor)
            .or(cpp_value)
            .or(cpp_ref)
            .or(method()
                .then(
                    just(Token::KwUse)
                        .ignore_then(path())
                        .map(Some)
                        .or(empty().to(None)),
                )
                .map(|(m, u)| ParsedTypeItem::Method(m, u)))
            .then_ignore(just(Token::Semicolon))
    }
    just(Token::KwType)
        .ignore_then(rust_type())
        .then(
            inner_item()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(ty, items)| ParsedItem::Type { ty, items })
}

fn trait_item<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    just(Token::KwTrait)
        .ignore_then(rust_trait(rust_type()))
        .then(
            method()
                .then_ignore(just(Token::Semicolon))
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(tr, methods)| ParsedItem::Trait { tr, methods })
}

fn fn_item<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    method()
        .then_ignore(just(Token::Semicolon))
        .map(ParsedItem::Fn)
}

fn additional_include_item<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    just(Token::Sharp)
        .then(just(Token::Ident("cpp_additional_includes")))
        .ignore_then(select! {
            Token::Str(c) => ParsedItem::CppAdditionalInclude(c),
        })
}

fn extern_cpp_item<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    let function = method()
        .then_ignore(just(Token::Semicolon))
        .map(ParsedExternCppItem::Function);
    let impl_block = just(Token::KwImpl)
        .ignore_then(rust_type())
        .then(
            method()
                .then_ignore(just(Token::Semicolon))
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(ty, methods)| ParsedExternCppItem::Impl { ty, methods });
    just(Token::KwExtern)
        .then(just(Token::Str("C++")))
        .ignore_then(
            function
                .or(impl_block)
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(ParsedItem::ExternCpp)
}

fn item<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    recursive(|item| {
        just(Token::KwMod)
            .ignore_then(path())
            .then(
                item.repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
            )
            .map(|(path, items)| ParsedItem::Mod { path, items })
            .or(type_item())
            .or(trait_item())
            .or(extern_cpp_item())
            .or(fn_item())
            .or(additional_include_item())
    })
}

fn path<'a>(
) -> impl Parser<'a, ParserInput<'a>, ParsedPath<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    let start = just(Token::ColonColon)
        .to(ParsedPathStart::Absolute)
        .or(just(Token::KwCrate)
            .then(just(Token::ColonColon))
            .to(ParsedPathStart::Crate))
        .or(empty().to(ParsedPathStart::Relative));
    start
        .then(
            (select! {
                Token::Ident(c) => c,
            })
            .separated_by(just(Token::ColonColon))
            .at_least(1)
            .collect::<Vec<_>>(),
        )
        .or(just(Token::KwCrate).to((ParsedPathStart::Crate, vec![])))
        .map_with_span(|(start, segments), span| ParsedPath {
            start,
            segments,
            span,
        })
}
