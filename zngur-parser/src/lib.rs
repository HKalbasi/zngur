use std::{fmt::Display, path::Component, process::exit};

use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::prelude::*;
use itertools::{Either, Itertools};

use zngur_def::{
    AdditionalIncludes, ConvertPanicToException, CppRef, CppValue, Import, LayoutPolicy, Merge,
    MergeFailure, Mutability, PrimitiveRustType, RustPathAndGenerics, RustTrait, RustType,
    ZngurConstructor, ZngurExternCppFn, ZngurExternCppImpl, ZngurField, ZngurFn, ZngurMethod,
    ZngurMethodDetails, ZngurMethodReceiver, ZngurSpec, ZngurTrait, ZngurType, ZngurWellknownTrait,
};

pub type Span = SimpleSpan<usize>;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Spanned<T> {
    inner: T,
    span: Span,
}

type ParserInput<'a> = chumsky::input::MappedInput<
    Token<'a>,
    Span,
    &'a [(Token<'a>, Span)],
    Box<
        dyn for<'x> Fn(
            &'x (Token<'_>, chumsky::span::SimpleSpan),
        ) -> (&'x Token<'x>, &'x SimpleSpan),
    >,
>;

#[derive(Debug)]
pub struct ParsedZngFile<'a>(Vec<ParsedItem<'a>>);

#[derive(Debug)]
pub struct ProcessedZngFile<'a> {
    aliases: Vec<ParsedAlias<'a>>,
    items: Vec<ProcessedItem<'a>>,
}

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
                .chain(self.segments)
                .map(|x| x.to_owned())
                .collect(),
            ParsedPathStart::Crate => ["crate"]
                .into_iter()
                .chain(self.segments)
                .map(|x| x.to_owned())
                .collect(),
        }
    }

    fn matches_alias(&self, alias: &ParsedAlias<'_>) -> bool {
        match self.start {
            ParsedPathStart::Absolute | ParsedPathStart::Crate => false,
            ParsedPathStart::Relative => self
                .segments
                .first()
                .is_some_and(|part| *part == alias.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAlias<'a> {
    name: &'a str,
    path: ParsedPath<'a>,
    span: Span,
}

impl ParsedAlias<'_> {
    fn expand(&self, path: &ParsedPath<'_>, base: &[String]) -> Option<Vec<String>> {
        if path.matches_alias(self) {
            match self.path.start {
                ParsedPathStart::Absolute => Some(
                    self.path
                        .segments
                        .iter()
                        .chain(path.segments.iter().skip(1))
                        .map(|seg| (*seg).to_owned())
                        .collect(),
                ),
                ParsedPathStart::Crate => Some(
                    ["crate"]
                        .into_iter()
                        .chain(self.path.segments.iter().cloned())
                        .chain(path.segments.iter().skip(1).cloned())
                        .map(|seg| (*seg).to_owned())
                        .collect(),
                ),
                ParsedPathStart::Relative => Some(
                    base.iter()
                        .map(|x| x.as_str())
                        .chain(self.path.segments.iter().cloned())
                        .chain(path.segments.iter().skip(1).cloned())
                        .map(|seg| (*seg).to_owned())
                        .collect(),
                ),
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedImportPath {
    path: std::path::PathBuf,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedItem<'a> {
    ConvertPanicToException(Span),
    CppAdditionalInclude(&'a str),
    Mod {
        path: ParsedPath<'a>,
        items: Vec<ParsedItem<'a>>,
    },
    Type {
        ty: Spanned<ParsedRustType<'a>>,
        items: Vec<Spanned<ParsedTypeItem<'a>>>,
    },
    Trait {
        tr: Spanned<ParsedRustTrait<'a>>,
        methods: Vec<ParsedMethod<'a>>,
    },
    Fn(Spanned<ParsedMethod<'a>>),
    ExternCpp(Vec<ParsedExternCppItem<'a>>),
    Alias(ParsedAlias<'a>),
    Import(ParsedImportPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessedItem<'a> {
    ConvertPanicToException(Span),
    CppAdditionalInclude(&'a str),
    Mod {
        path: ParsedPath<'a>,
        items: Vec<ProcessedItem<'a>>,
        aliases: Vec<ParsedAlias<'a>>,
    },
    Type {
        ty: Spanned<ParsedRustType<'a>>,
        items: Vec<Spanned<ParsedTypeItem<'a>>>,
    },
    Trait {
        tr: Spanned<ParsedRustTrait<'a>>,
        methods: Vec<ParsedMethod<'a>>,
    },
    Fn(Spanned<ParsedMethod<'a>>),
    ExternCpp(Vec<ParsedExternCppItem<'a>>),
    Import(ParsedImportPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedExternCppItem<'a> {
    Function(Spanned<ParsedMethod<'a>>),
    Impl {
        tr: Option<ParsedRustTrait<'a>>,
        ty: Spanned<ParsedRustType<'a>>,
        methods: Vec<ParsedMethod<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedConstructorArgs<'a> {
    Unit,
    Tuple(Vec<ParsedRustType<'a>>),
    Named(Vec<(&'a str, ParsedRustType<'a>)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedLayoutPolicy<'a> {
    StackAllocated(Vec<(Spanned<&'a str>, usize)>),
    HeapAllocated,
    OnlyByRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedTypeItem<'a> {
    Layout(Span, ParsedLayoutPolicy<'a>),
    Traits(Vec<Spanned<ZngurWellknownTrait>>),
    Constructor {
        name: Option<&'a str>,
        args: ParsedConstructorArgs<'a>,
    },
    Field {
        name: String,
        ty: ParsedRustType<'a>,
        offset: usize,
    },
    Method {
        data: ParsedMethod<'a>,
        use_path: Option<ParsedPath<'a>>,
        deref: Option<ParsedRustType<'a>>,
    },
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
    fn to_zngur(self, aliases: &[ParsedAlias<'_>], base: &[String]) -> ZngurMethod {
        ZngurMethod {
            name: self.name.to_owned(),
            generics: self
                .generics
                .into_iter()
                .map(|x| x.to_zngur(aliases, base))
                .collect(),
            receiver: self.receiver,
            inputs: self
                .inputs
                .into_iter()
                .map(|x| x.to_zngur(aliases, base))
                .collect(),
            output: self.output.to_zngur(aliases, base),
        }
    }
}

fn checked_merge<T, U>(src: T, dst: &mut U, span: Span, ctx: &ParseContext)
where
    T: Merge<U>,
{
    match src.merge(dst) {
        Ok(()) => {}
        Err(e) => match e {
            MergeFailure::Conflict(s) => {
                create_and_emit_error(ctx, &s, span);
            }
        },
    }
}

impl ProcessedItem<'_> {
    fn add_to_zngur_spec(
        self,
        r: &mut ZngurSpec,
        aliases: &[ParsedAlias],
        base: &[String],
        ctx: &ParseContext,
    ) {
        match self {
            ProcessedItem::Mod {
                path,
                items,
                aliases: mut mod_aliases,
            } => {
                let base = path.to_zngur(base);
                mod_aliases.extend_from_slice(aliases);
                for item in items {
                    item.add_to_zngur_spec(r, &mod_aliases, &base, ctx);
                }
            }
            ProcessedItem::Import(path) => {
                if path.path.is_absolute() {
                    create_and_emit_error(
                        ctx,
                        "Absolute paths imports are not supported.",
                        path.span,
                    )
                }
                match path.path.components().next() {
                    Some(Component::CurDir) | Some(Component::ParentDir) => {}
                    _ => create_and_emit_error(
                        ctx,
                        "Module import is not supported. Use a relative path instead.",
                        path.span,
                    ),
                }

                r.imports.push(Import(path.path));
            }
            ProcessedItem::Type { ty, items } => {
                if ty.inner == ParsedRustType::Tuple(vec![]) {
                    // We add unit type implicitly.
                    create_and_emit_error(
                        ctx,
                        "Unit type is declared implicitly. Remove this entirely.",
                        ty.span,
                    );
                }

                let mut methods = vec![];
                let mut constructors = vec![];
                let mut fields = vec![];
                let mut wellknown_traits = vec![];
                let mut layout = None;
                let mut layout_span = None;
                let mut cpp_value = None;
                let mut cpp_ref = None;
                for item in items {
                    let item_span = item.span;
                    let item = item.inner;
                    match item {
                        ParsedTypeItem::Layout(span, p) => {
                            layout = Some(match p {
                                ParsedLayoutPolicy::StackAllocated(p) => {
                                    let mut size = None;
                                    let mut align = None;
                                    for (key, value) in p {
                                        match key.inner {
                                            "size" => size = Some(value),
                                            "align" => align = Some(value),
                                            _ => create_and_emit_error(
                                                ctx,
                                                "Unknown property",
                                                key.span,
                                            ),
                                        }
                                    }
                                    let Some(size) = size else {
                                        create_and_emit_error(
                                            ctx,
                                            "Size is not declared for this type",
                                            ty.span,
                                        );
                                    };
                                    let Some(align) = align else {
                                        create_and_emit_error(
                                            ctx,
                                            "Align is not declared for this type",
                                            ty.span,
                                        );
                                    };
                                    LayoutPolicy::StackAllocated { size, align }
                                }
                                ParsedLayoutPolicy::HeapAllocated => LayoutPolicy::HeapAllocated,
                                ParsedLayoutPolicy::OnlyByRef => LayoutPolicy::OnlyByRef,
                            });
                            match layout_span {
                                Some(_) => {
                                    create_and_emit_error(
                                        ctx,
                                        "Duplicate layout policy found",
                                        span,
                                    );
                                }
                                None => layout_span = Some(span),
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
                                        .map(|(i, t)| (i.to_string(), t.to_zngur(aliases, base)))
                                        .collect(),
                                    ParsedConstructorArgs::Named(t) => t
                                        .into_iter()
                                        .map(|(i, t)| (i.to_owned(), t.to_zngur(aliases, base)))
                                        .collect(),
                                },
                            })
                        }
                        ParsedTypeItem::Field { name, ty, offset } => {
                            fields.push(ZngurField {
                                name: name.to_owned(),
                                ty: ty.to_zngur(aliases, base),
                                offset,
                            });
                        }
                        ParsedTypeItem::Method {
                            data,
                            use_path,
                            deref,
                        } => {
                            let deref = deref.map(|x| {
                                let deref_type = x.to_zngur(aliases, base);
                                let receiver_mutability = match data.receiver {
                                    ZngurMethodReceiver::Ref(mutability) => mutability,
                                    ZngurMethodReceiver::Static | ZngurMethodReceiver::Move => {
                                        create_and_emit_error(
                                            ctx,
                                            "Deref needs reference receiver",
                                            item_span,
                                        );
                                    }
                                };
                                (deref_type, receiver_mutability)
                            });
                            methods.push(ZngurMethodDetails {
                                data: data.to_zngur(aliases, base),
                                use_path: use_path.map(|x| {
                                    aliases
                                        .iter()
                                        .filter_map(|alias| alias.expand(&x, base))
                                        .collect::<Vec<_>>()
                                        .first()
                                        .cloned()
                                        .unwrap_or_else(|| x.to_zngur(base))
                                }),
                                deref,
                            });
                        }
                        ParsedTypeItem::CppValue { field, cpp_type } => {
                            cpp_value = Some(CppValue(field.to_owned(), cpp_type.to_owned()));
                        }
                        ParsedTypeItem::CppRef { cpp_type } => {
                            match layout_span {
                                Some(span) => {
                                    create_and_emit_error(
                                        ctx,
                                        "Duplicate layout policy found",
                                        span,
                                    );
                                }
                                None => {
                                    layout = Some(LayoutPolicy::ZERO_SIZED_TYPE);
                                    layout_span = Some(item_span);
                                }
                            }
                            cpp_ref = Some(CppRef(cpp_type.to_owned()));
                        }
                    }
                }
                let is_unsized = wellknown_traits
                    .iter()
                    .find(|x| x.inner == ZngurWellknownTrait::Unsized)
                    .cloned();
                let is_copy = wellknown_traits
                    .iter()
                    .find(|x| x.inner == ZngurWellknownTrait::Copy)
                    .cloned();
                let mut wt = wellknown_traits
                    .into_iter()
                    .map(|x| x.inner)
                    .collect::<Vec<_>>();
                if is_copy.is_none() && is_unsized.is_none() {
                    wt.push(ZngurWellknownTrait::Drop);
                }
                if let Some(is_unsized) = is_unsized {
                    if let Some(span) = layout_span {
                        emit_ariadne_error(
                            ctx,
                            Report::build(
                                ReportKind::Error,
                                ctx.filename().to_string(),
                                span.start,
                            )
                            .with_message("Duplicate layout policy found for unsized type.")
                            .with_label(
                                Label::new((ctx.filename().to_string(), span.start..span.end))
                                    .with_message(
                                        "Unsized types have implicit layout policy, remove this.",
                                    )
                                    .with_color(Color::Red),
                            )
                            .with_label(
                                Label::new((
                                    ctx.filename().to_string(),
                                    is_unsized.span.start..is_unsized.span.end,
                                ))
                                .with_message("Type declared as unsized here.")
                                .with_color(Color::Blue),
                            )
                            .finish(),
                        )
                    }
                    layout = Some(LayoutPolicy::OnlyByRef);
                }
                let Some(layout) = layout else {
                    create_and_emit_error(
                        ctx,
                        "No layout policy found for this type. \
Use one of `#layout(size = X, align = Y)`, `#heap_allocated` or `#only_by_ref`.",
                        ty.span,
                    );
                };
                checked_merge(
                    ZngurType {
                        ty: ty.inner.to_zngur(aliases, base),
                        layout,
                        methods,
                        wellknown_traits: wt,
                        constructors,
                        fields,
                        cpp_value,
                        cpp_ref,
                    },
                    r,
                    ty.span,
                    ctx,
                );
            }
            ProcessedItem::Trait { tr, methods } => {
                checked_merge(
                    ZngurTrait {
                        tr: tr.inner.to_zngur(aliases, base),
                        methods: methods
                            .into_iter()
                            .map(|m| m.to_zngur(aliases, base))
                            .collect(),
                    },
                    r,
                    tr.span,
                    ctx,
                );
            }
            ProcessedItem::Fn(f) => {
                let method = f.inner.to_zngur(aliases, base);
                checked_merge(
                    ZngurFn {
                        path: RustPathAndGenerics {
                            path: base.iter().chain(Some(&method.name)).cloned().collect(),
                            generics: method.generics,
                            named_generics: vec![],
                        },
                        inputs: method.inputs,
                        output: method.output,
                    },
                    r,
                    f.span,
                    ctx,
                );
            }
            ProcessedItem::ExternCpp(items) => {
                for item in items {
                    match item {
                        ParsedExternCppItem::Function(method) => {
                            let span = method.span;
                            let method = method.inner.to_zngur(aliases, base);
                            checked_merge(
                                ZngurExternCppFn {
                                    name: method.name.to_string(),
                                    inputs: method.inputs,
                                    output: method.output,
                                },
                                r,
                                span,
                                ctx,
                            );
                        }
                        ParsedExternCppItem::Impl { tr, ty, methods } => {
                            checked_merge(
                                ZngurExternCppImpl {
                                    tr: tr.map(|x| x.to_zngur(aliases, base)),
                                    ty: ty.inner.to_zngur(aliases, base),
                                    methods: methods
                                        .into_iter()
                                        .map(|x| x.to_zngur(aliases, base))
                                        .collect(),
                                },
                                r,
                                ty.span,
                                ctx,
                            );
                        }
                    }
                }
            }
            ProcessedItem::CppAdditionalInclude(s) => {
                match AdditionalIncludes(s.to_owned()).merge(r) {
                    Ok(()) => {}
                    Err(_) => {
                        unreachable!() // For now, additional includes can't have conflicts.
                    }
                }
            }
            ProcessedItem::ConvertPanicToException(span) => {
                if ctx.depth > 0 {
                    create_and_emit_error(
                        ctx,
                        "Using `#convert_panic_to_exception` in imported zngur files is not supported. This directive can only be used in the main zngur file.",
                        span,
                    );
                }
                match ConvertPanicToException(true).merge(r) {
                    Ok(()) => {}
                    Err(_) => {
                        unreachable!() // For now, CPtE also can't have conflicts.
                    }
                }
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
    fn to_zngur(self, aliases: &[ParsedAlias<'_>], base: &[String]) -> RustType {
        match self {
            ParsedRustType::Primitive(s) => RustType::Primitive(s),
            ParsedRustType::Ref(m, s) => RustType::Ref(m, Box::new(s.to_zngur(aliases, base))),
            ParsedRustType::Raw(m, s) => RustType::Raw(m, Box::new(s.to_zngur(aliases, base))),
            ParsedRustType::Boxed(s) => RustType::Boxed(Box::new(s.to_zngur(aliases, base))),
            ParsedRustType::Slice(s) => RustType::Slice(Box::new(s.to_zngur(aliases, base))),
            ParsedRustType::Dyn(tr, bounds) => RustType::Dyn(
                tr.to_zngur(aliases, base),
                bounds.into_iter().map(|x| x.to_owned()).collect(),
            ),
            ParsedRustType::Tuple(v) => {
                RustType::Tuple(v.into_iter().map(|s| s.to_zngur(aliases, base)).collect())
            }
            ParsedRustType::Adt(s) => RustType::Adt(s.to_zngur(aliases, base)),
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
    fn to_zngur(self, aliases: &[ParsedAlias<'_>], base: &[String]) -> RustTrait {
        match self {
            ParsedRustTrait::Normal(s) => RustTrait::Normal(s.to_zngur(aliases, base)),
            ParsedRustTrait::Fn {
                name,
                inputs,
                output,
            } => RustTrait::Fn {
                name: name.to_owned(),
                inputs: inputs
                    .into_iter()
                    .map(|s| s.to_zngur(aliases, base))
                    .collect(),
                output: Box::new(output.to_zngur(aliases, base)),
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
    fn to_zngur(self, aliases: &[ParsedAlias<'_>], base: &[String]) -> RustPathAndGenerics {
        RustPathAndGenerics {
            path: aliases
                .iter()
                .filter_map(|alias| alias.expand(&self.path, base))
                .collect::<Vec<_>>()
                .first()
                .cloned()
                .unwrap_or_else(|| self.path.to_zngur(base)),
            generics: self
                .generics
                .into_iter()
                .map(|x| x.to_zngur(aliases, base))
                .collect(),
            named_generics: self
                .named_generics
                .into_iter()
                .map(|(name, x)| (name.to_owned(), x.to_zngur(aliases, base)))
                .collect(),
        }
    }
}

struct ParseContext<'a> {
    path: std::path::PathBuf,
    text: &'a str,
    depth: usize,
}

impl<'a> ParseContext<'a> {
    fn new(path: std::path::PathBuf, text: &'a str) -> Self {
        Self {
            path,
            text,
            depth: 0,
        }
    }

    fn with_depth(path: std::path::PathBuf, text: &'a str, depth: usize) -> Self {
        Self { path, text, depth }
    }

    fn filename(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}

/// A trait for types which can resolve filesystem-like paths relative to a given directory.
pub trait ImportResolver {
    fn resolve_import(
        &self,
        cwd: &std::path::Path,
        relpath: &std::path::Path,
    ) -> Result<String, String>;
}

/// A default implementation of ImportResolver which uses conventional filesystem paths and semantics.
struct DefaultImportResolver;

impl ImportResolver for DefaultImportResolver {
    fn resolve_import(
        &self,
        cwd: &std::path::Path,
        relpath: &std::path::Path,
    ) -> Result<String, String> {
        let path = cwd
            .join(relpath)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }
}

impl<'a> ParsedZngFile<'a> {
    fn parse_into(zngur: &mut ZngurSpec, ctx: &ParseContext, resolver: &impl ImportResolver) {
        let (tokens, errs) = lexer().parse(ctx.text).into_output_errors();
        let Some(tokens) = tokens else {
            let errs = errs.into_iter().map(|e| e.map_token(|c| c.to_string()));
            emit_error(&ctx, errs);
        };
        let tokens: ParserInput<'_> = tokens.as_slice().map(
            (ctx.text.len()..ctx.text.len()).into(),
            Box::new(|(t, s)| (t, s)),
        );
        let (ast, errs) = file_parser()
            .map_with(|ast, extra| (ast, extra.span()))
            .parse(tokens)
            .into_output_errors();
        let Some(ast) = ast else {
            let errs = errs.into_iter().map(|e| e.map_token(|c| c.to_string()));
            emit_error(&ctx, errs);
        };

        let (aliases, items) = ast.0.0.into_iter().partition_map(partition_parsed_item_vec);
        ProcessedZngFile::new(aliases, items).into_zngur_spec(zngur, &ctx);

        if let Some(dirname) = ctx.path.parent() {
            for import in std::mem::take(&mut zngur.imports) {
                match resolver.resolve_import(dirname, &import.0) {
                    Ok(text) => {
                        Self::parse_into(
                            zngur,
                            &ParseContext::with_depth(
                                dirname.join(&import.0),
                                &text,
                                ctx.depth + 1,
                            ),
                            resolver,
                        );
                    }
                    Err(_) => {
                        // TODO: emit a better error. How should we get a span here?
                        // I'd like to avoid putting a ParsedImportPath in ZngurSpec, and
                        // also not have to pass a filename to add_to_zngur_spec.
                        emit_ariadne_error(
                            &ctx,
                            Report::build(ReportKind::Error, ctx.filename(), 0)
                                .with_message(format!(
                                    "Import path not found: {}",
                                    import.0.display()
                                ))
                                .finish(),
                        );
                    }
                }
            }
        }
    }

    pub fn parse(path: std::path::PathBuf) -> ZngurSpec {
        let mut zngur = ZngurSpec::default();
        let text = std::fs::read_to_string(&path).unwrap();
        Self::parse_into(
            &mut zngur,
            &ParseContext::new(path, &text),
            &DefaultImportResolver,
        );
        zngur
    }

    pub fn parse_str(text: &str) -> ZngurSpec {
        let mut zngur = ZngurSpec::default();
        Self::parse_into(
            &mut zngur,
            &ParseContext::new(std::path::PathBuf::from("test.zng"), text),
            &DefaultImportResolver,
        );
        zngur
    }

    #[cfg(test)]
    pub(crate) fn parse_str_with_resolver(text: &str, resolver: &impl ImportResolver) -> ZngurSpec {
        let mut zngur = ZngurSpec::default();
        Self::parse_into(
            &mut zngur,
            &ParseContext::new(std::path::PathBuf::from("test.zng"), text),
            resolver,
        );
        zngur
    }
}

fn partition_parsed_item_vec(item: ParsedItem<'_>) -> Either<ParsedAlias<'_>, ProcessedItem<'_>> {
    match item {
        ParsedItem::Alias(alias) => Either::Left(alias),
        ParsedItem::ConvertPanicToException(span) => {
            Either::Right(ProcessedItem::ConvertPanicToException(span))
        }
        ParsedItem::CppAdditionalInclude(inc) => {
            Either::Right(ProcessedItem::CppAdditionalInclude(inc))
        }
        ParsedItem::Mod { path, items } => {
            let (aliases, items): (Vec<ParsedAlias<'_>>, Vec<ProcessedItem<'_>>) =
                items.into_iter().partition_map(partition_parsed_item_vec);
            Either::Right(ProcessedItem::Mod {
                path,
                items,
                aliases,
            })
        }
        ParsedItem::Type { ty, items } => Either::Right(ProcessedItem::Type { ty, items }),
        ParsedItem::Trait { tr, methods } => Either::Right(ProcessedItem::Trait { tr, methods }),
        ParsedItem::Fn(method) => Either::Right(ProcessedItem::Fn(method)),
        ParsedItem::ExternCpp(items) => Either::Right(ProcessedItem::ExternCpp(items)),
        ParsedItem::Import(path) => Either::Right(ProcessedItem::Import(path)),
    }
}

impl<'a> ProcessedZngFile<'a> {
    fn new(aliases: Vec<ParsedAlias<'a>>, items: Vec<ProcessedItem<'a>>) -> Self {
        ProcessedZngFile { aliases, items }
    }

    fn into_zngur_spec(self, zngur: &mut ZngurSpec, ctx: &ParseContext) {
        for item in self.items {
            item.add_to_zngur_spec(zngur, &self.aliases, &[], ctx);
        }
    }
}

fn create_and_emit_error(ctx: &ParseContext, error: &str, span: Span) -> ! {
    emit_error(ctx, [Rich::custom(span, error)].into_iter())
}

#[cfg(test)]
fn emit_ariadne_error(ctx: &ParseContext, err: Report<'_, (String, std::ops::Range<usize>)>) -> ! {
    let mut r = Vec::<u8>::new();
    err.write(sources([(ctx.filename().to_string(), ctx.text)]), &mut r)
        .unwrap();

    std::panic::resume_unwind(Box::new(tests::ErrorText(
        String::from_utf8(strip_ansi_escapes::strip(r)).unwrap(),
    )));
}

#[cfg(not(test))]
fn emit_ariadne_error(ctx: &ParseContext, err: Report<'_, (String, std::ops::Range<usize>)>) -> ! {
    err.eprint(sources([(ctx.filename().to_string(), ctx.text)]))
        .unwrap();
    exit(101);
}

fn emit_error<'a>(ctx: &ParseContext, errs: impl Iterator<Item = Rich<'a, String>>) -> ! {
    for e in errs {
        emit_ariadne_error(
            ctx,
            Report::build(ReportKind::Error, ctx.filename(), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((ctx.filename().to_string(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(label, span)| {
                    Label::new((ctx.filename().to_string(), span.into_range()))
                        .with_message(format!("while parsing this {}", label))
                        .with_color(Color::Yellow)
                }))
                .finish(),
        )
    }
    exit(101);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token<'a> {
    Arrow,
    AngleOpen,
    AngleClose,
    BracketOpen,
    BracketClose,
    Colon,
    ColonColon,
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
    KwAs,
    KwDyn,
    KwUse,
    KwFor,
    KwMod,
    KwCrate,
    KwType,
    KwTrait,
    KwFn,
    KwMut,
    KwConst,
    KwExtern,
    KwImpl,
    KwImport,
    Ident(&'a str),
    Str(&'a str),
    Number(usize),
}

impl<'a> Token<'a> {
    fn ident_or_kw(ident: &'a str) -> Self {
        match ident {
            "as" => Token::KwAs,
            "dyn" => Token::KwDyn,
            "mod" => Token::KwMod,
            "type" => Token::KwType,
            "trait" => Token::KwTrait,
            "crate" => Token::KwCrate,
            "fn" => Token::KwFn,
            "mut" => Token::KwMut,
            "const" => Token::KwConst,
            "use" => Token::KwUse,
            "for" => Token::KwFor,
            "extern" => Token::KwExtern,
            "impl" => Token::KwImpl,
            "import" => Token::KwImport,
            x => Token::Ident(x),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Arrow => write!(f, "->"),
            Token::AngleOpen => write!(f, "<"),
            Token::AngleClose => write!(f, ">"),
            Token::BracketOpen => write!(f, "["),
            Token::BracketClose => write!(f, "]"),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::BraceOpen => write!(f, "{{"),
            Token::BraceClose => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::ColonColon => write!(f, "::"),
            Token::And => write!(f, "&"),
            Token::Star => write!(f, "*"),
            Token::Sharp => write!(f, "#"),
            Token::Plus => write!(f, "+"),
            Token::Eq => write!(f, "="),
            Token::Question => write!(f, "?"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::KwAs => write!(f, "as"),
            Token::KwDyn => write!(f, "dyn"),
            Token::KwUse => write!(f, "use"),
            Token::KwFor => write!(f, "for"),
            Token::KwMod => write!(f, "mod"),
            Token::KwCrate => write!(f, "crate"),
            Token::KwType => write!(f, "type"),
            Token::KwTrait => write!(f, "trait"),
            Token::KwFn => write!(f, "fn"),
            Token::KwMut => write!(f, "mut"),
            Token::KwConst => write!(f, "const"),
            Token::KwExtern => write!(f, "extern"),
            Token::KwImpl => write!(f, "impl"),
            Token::KwImport => write!(f, "import"),
            Token::Ident(i) => write!(f, "{i}"),
            Token::Number(n) => write!(f, "{n}"),
            Token::Str(s) => write!(f, r#""{s}""#),
        }
    }
}

fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    let token = choice((
        choice([
            just("->").to(Token::Arrow),
            just("<").to(Token::AngleOpen),
            just(">").to(Token::AngleClose),
            just("[").to(Token::BracketOpen),
            just("]").to(Token::BracketClose),
            just("(").to(Token::ParenOpen),
            just(")").to(Token::ParenClose),
            just("{").to(Token::BraceOpen),
            just("}").to(Token::BraceClose),
            just("::").to(Token::ColonColon),
            just(":").to(Token::Colon),
            just("&").to(Token::And),
            just("*").to(Token::Star),
            just("#").to(Token::Sharp),
            just("+").to(Token::Plus),
            just("=").to(Token::Eq),
            just("?").to(Token::Question),
            just(",").to(Token::Comma),
            just(";").to(Token::Semicolon),
        ]),
        text::ident().map(Token::ident_or_kw),
        text::int(10).map(|x: &str| Token::Number(x.parse().unwrap())),
        just('"')
            .ignore_then(none_of('"').repeated().to_slice().map(Token::Str))
            .then_ignore(just('"')),
    ));

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with(|tok, extra| (tok, extra.span()))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
        .collect()
}

fn alias<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    just(Token::KwUse)
        .ignore_then(path())
        .then_ignore(just(Token::KwAs))
        .then(select! {
            Token::Ident(c) => c,
        })
        .then_ignore(just(Token::Semicolon))
        .map_with(|(path, name), extra| {
            ParsedItem::Alias(ParsedAlias {
                name,
                path,
                span: extra.span(),
            })
        })
        .boxed()
}

fn file_parser<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedZngFile<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    item().repeated().collect::<Vec<_>>().map(ParsedZngFile)
}

fn rust_type<'a>()
-> Boxed<'a, 'a, ParserInput<'a>, ParsedRustType<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> {
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
        let parser = parser.boxed();
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
        let tuple = parser
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::ParenOpen), just(Token::ParenClose))
            .map(|xs| ParsedRustType::Tuple(xs));
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
        choice((
            scalar, boxed, unit, tuple, slice, adt, reference, raw_ptr, dyn_trait,
        ))
    })
    .boxed()
}

fn rust_generics<'a>(
    rust_type: Boxed<
        'a,
        'a,
        ParserInput<'a>,
        ParsedRustType<'a>,
        extra::Err<Rich<'a, Token<'a>, Span>>,
    >,
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
    rust_type: Boxed<
        'a,
        'a,
        ParserInput<'a>,
        ParsedRustType<'a>,
        extra::Err<Rich<'a, Token<'a>, Span>>,
    >,
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
    rust_type: Boxed<
        'a,
        'a,
        ParserInput<'a>,
        ParsedRustType<'a>,
        extra::Err<Rich<'a, Token<'a>, Span>>,
    >,
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
        .boxed()
}

fn spanned<'a, T>(
    parser: impl Parser<'a, ParserInput<'a>, T, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone,
) -> impl Parser<'a, ParserInput<'a>, Spanned<T>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    parser.map_with(|inner, extra| Spanned {
        inner,
        span: extra.span(),
    })
}

fn rust_trait<'a>(
    rust_type: Boxed<
        'a,
        'a,
        ParserInput<'a>,
        ParsedRustType<'a>,
        extra::Err<Rich<'a, Token<'a>, Span>>,
    >,
) -> impl Parser<'a, ParserInput<'a>, ParsedRustTrait<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
{
    let fn_trait = select! {
        Token::Ident(c) => c,
    }
    .then(fn_args(rust_type.clone()))
    .map(|x| ParsedRustTrait::Fn {
        name: x.0,
        inputs: x.1.0,
        output: Box::new(x.1.1),
    });

    let rust_trait = fn_trait.or(rust_path_and_generics(rust_type).map(ParsedRustTrait::Normal));
    rust_trait
}

fn method<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedMethod<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone
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

fn type_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    fn inner_item<'a>()
    -> impl Parser<'a, ParserInput<'a>, ParsedTypeItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>>
    + Clone {
        let property_item = (spanned(select! {
            Token::Ident(c) => c,
        }))
        .then_ignore(just(Token::Eq))
        .then(select! {
            Token::Number(c) => c,
        });
        let layout = just([Token::Sharp, Token::Ident("layout")])
            .ignore_then(
                property_item
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(ParsedLayoutPolicy::StackAllocated)
            .or(just([Token::Sharp, Token::Ident("only_by_ref")]).to(ParsedLayoutPolicy::OnlyByRef))
            .or(just([Token::Sharp, Token::Ident("heap_allocated")])
                .to(ParsedLayoutPolicy::HeapAllocated))
            .map_with(|x, extra| ParsedTypeItem::Layout(extra.span(), x))
            .boxed();
        let trait_item = select! {
            Token::Ident("Debug") => ZngurWellknownTrait::Debug,
            Token::Ident("Copy") => ZngurWellknownTrait::Copy,
        }
        .or(just(Token::Question)
            .then(just(Token::Ident("Sized")))
            .to(ZngurWellknownTrait::Unsized));
        let traits = just(Token::Ident("wellknown_traits"))
            .ignore_then(
                spanned(trait_item)
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(ParsedTypeItem::Traits)
            .boxed();
        let constructor_args = rust_type()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::ParenOpen), just(Token::ParenClose))
            .map(ParsedConstructorArgs::Tuple)
            .or((select! {
                Token::Ident(c) => c,
            })
            .boxed()
            .then_ignore(just(Token::Colon))
            .then(rust_type())
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::BraceOpen), just(Token::BraceClose))
            .map(ParsedConstructorArgs::Named))
            .or(empty().to(ParsedConstructorArgs::Unit))
            .boxed();
        let constructor = just(Token::Ident("constructor")).ignore_then(
            (select! {
                Token::Ident(c) => Some(c),
            })
            .or(empty().to(None))
            .then(constructor_args)
            .map(|(name, args)| ParsedTypeItem::Constructor { name, args }),
        );
        let field = just(Token::Ident("field")).ignore_then(
            (select! {
                Token::Ident(c) => c.to_owned(),
                Token::Number(c) => c.to_string(),
            })
            .then(
                just(Token::Ident("offset"))
                    .then(just(Token::Eq))
                    .ignore_then(select! {
                        Token::Number(c) => c,
                    })
                    .then(
                        just(Token::Comma)
                            .then(just(Token::KwType))
                            .then(just(Token::Eq))
                            .ignore_then(rust_type()),
                    )
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(|(name, (offset, ty))| ParsedTypeItem::Field { name, ty, offset }),
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
        choice((
            layout,
            traits,
            constructor,
            field,
            cpp_value,
            cpp_ref,
            method()
                .then(
                    just(Token::KwUse)
                        .ignore_then(path())
                        .map(Some)
                        .or(empty().to(None)),
                )
                .then(
                    just(Token::Ident("deref"))
                        .ignore_then(rust_type())
                        .map(Some)
                        .or(empty().to(None)),
                )
                .map(|((data, use_path), deref)| ParsedTypeItem::Method {
                    deref,
                    use_path,
                    data,
                }),
        ))
        .then_ignore(just(Token::Semicolon))
    }
    just(Token::KwType)
        .ignore_then(spanned(rust_type()))
        .then(
            spanned(inner_item())
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(ty, items)| ParsedItem::Type { ty, items })
        .boxed()
}

fn trait_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    just(Token::KwTrait)
        .ignore_then(spanned(rust_trait(rust_type())))
        .then(
            method()
                .then_ignore(just(Token::Semicolon))
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(tr, methods)| ParsedItem::Trait { tr, methods })
        .boxed()
}

fn fn_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    spanned(method())
        .then_ignore(just(Token::Semicolon))
        .map(ParsedItem::Fn)
}

fn additional_include_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    just(Token::Sharp)
        .ignore_then(
            just(Token::Ident("cpp_additional_includes"))
                .ignore_then(select! {
                    Token::Str(c) => ParsedItem::CppAdditionalInclude(c),
                })
                .or(just(Token::Ident("convert_panic_to_exception"))
                    .map_with(|_, extra| ParsedItem::ConvertPanicToException(extra.span()))),
        )
        .boxed()
}

fn extern_cpp_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    let function = spanned(method())
        .then_ignore(just(Token::Semicolon))
        .map(ParsedExternCppItem::Function);
    let impl_block = just(Token::KwImpl)
        .ignore_then(
            rust_trait(rust_type())
                .then_ignore(just(Token::KwFor))
                .map(Some)
                .or(empty().to(None))
                .then(spanned(rust_type())),
        )
        .then(
            method()
                .then_ignore(just(Token::Semicolon))
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|((tr, ty), methods)| ParsedExternCppItem::Impl { tr, ty, methods });
    just(Token::KwExtern)
        .then(just(Token::Str("C++")))
        .ignore_then(
            function
                .or(impl_block)
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose))
                .boxed(),
        )
        .map(ParsedItem::ExternCpp)
        .boxed()
}

fn item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    recursive(|item| {
        choice((
            just(Token::KwMod)
                .ignore_then(path())
                .then(
                    item.repeated()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
                )
                .map(|(path, items)| ParsedItem::Mod { path, items }),
            type_item(),
            trait_item(),
            extern_cpp_item(),
            fn_item(),
            additional_include_item(),
            import_item(),
            alias(),
        ))
    })
    .boxed()
}

fn import_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    just(Token::KwImport)
        .ignore_then(select! {
            Token::Str(path) => path,
        })
        .then_ignore(just(Token::Semicolon))
        .map_with(|path, extra| {
            ParsedItem::Import(ParsedImportPath {
                path: std::path::PathBuf::from(path),
                span: extra.span(),
            })
        })
        .boxed()
}

fn path<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedPath<'a>, extra::Err<Rich<'a, Token<'a>, Span>>> + Clone {
    let start = choice((
        just(Token::ColonColon).to(ParsedPathStart::Absolute),
        just(Token::KwCrate)
            .then(just(Token::ColonColon))
            .to(ParsedPathStart::Crate),
        empty().to(ParsedPathStart::Relative),
    ));
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
        .map_with(|(start, segments), extra| ParsedPath {
            start,
            segments,
            span: extra.span(),
        })
        .boxed()
}
