use std::{collections::HashMap, fmt::Display, path::Component};

#[cfg(not(test))]
use std::process::exit;

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

/// Result of parsing a .zng file, containing both the spec and the list of all processed files.
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed Zngur specification
    pub spec: ZngurSpec,
    /// All .zng files that were processed (main file + transitive imports)
    pub processed_files: Vec<std::path::PathBuf>,
}

#[cfg(test)]
mod tests;

pub mod cfg;
mod conditional;

use crate::{
    cfg::{CfgConditional, RustCfgProvider},
    conditional::{Condition, ConditionalItem, NItems, conditional_item},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
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

#[derive(Default)]
pub struct UnstableFeatures {
    pub cfg_match: bool,
    pub cfg_if: bool,
}

#[derive(Default)]
pub struct ZngParserState {
    pub unstable_features: UnstableFeatures,
}

type ZngParserExtra<'a> =
    extra::Full<Rich<'a, Token<'a>, Span>, extra::SimpleState<ZngParserState>, ()>;

type BoxedZngParser<'a, Item> = chumsky::Boxed<'a, 'a, ParserInput<'a>, Item, ZngParserExtra<'a>>;

/// Effective trait alias for verbose chumsky Parser Trait
trait ZngParser<'a, Item>: Parser<'a, ParserInput<'a>, Item, ZngParserExtra<'a>> + Clone {}
impl<'a, T, Item> ZngParser<'a, Item> for T where
    T: Parser<'a, ParserInput<'a>, Item, ZngParserExtra<'a>> + Clone
{
}

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

#[derive(Debug, Clone)]
struct Scope<'a> {
    aliases: Vec<ParsedAlias<'a>>,
    base: Vec<String>,
}

impl<'a> Scope<'a> {
    /// Create a new root scope containing the specified aliases.
    fn new_root(aliases: Vec<ParsedAlias<'a>>) -> Scope<'a> {
        Scope {
            aliases,
            base: Vec::new(),
        }
    }

    /// Resolve a path according to the current scope.
    fn resolve_path(&self, path: ParsedPath<'a>) -> Vec<String> {
        // Check to see if the path refers to an alias:
        if let Some(expanded_alias) = self
            .aliases
            .iter()
            .find_map(|alias| alias.expand(&path, &self.base))
        {
            expanded_alias
        } else {
            path.to_zngur(&self.base)
        }
    }

    /// Create a fully-qualified path relative to this scope's base path.
    fn simple_relative_path(&self, relative_item_name: &str) -> Vec<String> {
        self.base
            .iter()
            .cloned()
            .chain(Some(relative_item_name.to_string()))
            .collect()
    }

    fn sub_scope(&self, new_aliases: &[ParsedAlias<'a>], nested_path: ParsedPath<'a>) -> Scope<'_> {
        let base = nested_path.to_zngur(&self.base);
        let mut mod_aliases = new_aliases.to_vec();
        mod_aliases.extend_from_slice(&self.aliases);

        Scope {
            aliases: mod_aliases,
            base,
        }
    }
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
    UnstableFeature(&'a str),
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
    MatchOnCfg(Condition<CfgConditional<'a>, ParsedItem<'a>, NItems>),
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
    Conservative(Vec<(Spanned<&'a str>, usize)>),
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
        offset: Option<usize>,
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
    MatchOnCfg(Condition<CfgConditional<'a>, ParsedTypeItem<'a>, NItems>),
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
    fn to_zngur(self, scope: &Scope<'_>) -> ZngurMethod {
        ZngurMethod {
            name: self.name.to_owned(),
            generics: self
                .generics
                .into_iter()
                .map(|x| x.to_zngur(scope))
                .collect(),
            receiver: self.receiver,
            inputs: self.inputs.into_iter().map(|x| x.to_zngur(scope)).collect(),
            output: self.output.to_zngur(scope),
        }
    }
}

fn checked_merge<T, U>(src: T, dst: &mut U, span: Span, ctx: &mut ParseContext)
where
    T: Merge<U>,
{
    match src.merge(dst) {
        Ok(()) => {}
        Err(e) => match e {
            MergeFailure::Conflict(s) => {
                ctx.add_error_str(&s, span);
            }
        },
    }
}

impl ProcessedItem<'_> {
    fn add_to_zngur_spec(self, r: &mut ZngurSpec, scope: &Scope<'_>, ctx: &mut ParseContext) {
        match self {
            ProcessedItem::Mod {
                path,
                items,
                aliases,
            } => {
                let sub_scope = scope.sub_scope(&aliases, path);
                for item in items {
                    item.add_to_zngur_spec(r, &sub_scope, ctx);
                }
            }
            ProcessedItem::Import(path) => {
                if path.path.is_absolute() {
                    ctx.add_error_str("Absolute paths imports are not supported.", path.span)
                }
                match path.path.components().next() {
                    Some(Component::CurDir) | Some(Component::ParentDir) => {
                        r.imports.push(Import(path.path));
                    }
                    _ => ctx.add_error_str(
                        "Module import is not supported. Use a relative path instead.",
                        path.span,
                    ),
                }
            }
            ProcessedItem::Type { ty, items } => {
                if ty.inner == ParsedRustType::Tuple(vec![]) {
                    // We add unit type implicitly.
                    ctx.add_error_str(
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
                let mut to_process = items;
                to_process.reverse(); // create a stack of items to process
                while let Some(item) = to_process.pop() {
                    let item_span = item.span;
                    let item = item.inner;
                    match item {
                        ParsedTypeItem::Layout(span, p) => {
                            let mut check_size_align = |props: Vec<(Spanned<&str>, usize)>| {
                                let mut size = None;
                                let mut align = None;
                                for (key, value) in props {
                                    match key.inner {
                                        "size" => size = Some(value),
                                        "align" => align = Some(value),
                                        _ => ctx.add_error_str("Unknown property", key.span),
                                    }
                                }
                                let Some(size) = size else {
                                    ctx.add_error_str(
                                        "Size is not declared for this type",
                                        ty.span,
                                    );
                                    return None;
                                };
                                let Some(align) = align else {
                                    ctx.add_error_str(
                                        "Align is not declared for this type",
                                        ty.span,
                                    );
                                    return None;
                                };
                                Some((size, align))
                            };
                            layout = Some(match p {
                                ParsedLayoutPolicy::StackAllocated(p) => {
                                    let Some((size, align)) = check_size_align(p) else {
                                        continue;
                                    };
                                    LayoutPolicy::StackAllocated { size, align }
                                }
                                ParsedLayoutPolicy::Conservative(p) => {
                                    let Some((size, align)) = check_size_align(p) else {
                                        continue;
                                    };
                                    LayoutPolicy::Conservative { size, align }
                                }
                                ParsedLayoutPolicy::HeapAllocated => LayoutPolicy::HeapAllocated,
                                ParsedLayoutPolicy::OnlyByRef => LayoutPolicy::OnlyByRef,
                            });
                            match layout_span {
                                Some(_) => {
                                    ctx.add_error_str("Duplicate layout policy found", span);
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
                                        .map(|(i, t)| (i.to_string(), t.to_zngur(scope)))
                                        .collect(),
                                    ParsedConstructorArgs::Named(t) => t
                                        .into_iter()
                                        .map(|(i, t)| (i.to_owned(), t.to_zngur(scope)))
                                        .collect(),
                                },
                            })
                        }
                        ParsedTypeItem::Field { name, ty, offset } => {
                            fields.push(ZngurField {
                                name: name.to_owned(),
                                ty: ty.to_zngur(scope),
                                offset,
                            });
                        }
                        ParsedTypeItem::Method {
                            data,
                            use_path,
                            deref,
                        } => {
                            let deref = deref.and_then(|x| {
                                let deref_type = x.to_zngur(scope);
                                let receiver_mutability = match data.receiver {
                                    ZngurMethodReceiver::Ref(mutability) => mutability,
                                    ZngurMethodReceiver::Static | ZngurMethodReceiver::Move => {
                                        ctx.add_error_str(
                                            "Deref needs reference receiver",
                                            item_span,
                                        );
                                        return None;
                                    }
                                };
                                Some((deref_type, receiver_mutability))
                            });
                            methods.push(ZngurMethodDetails {
                                data: data.to_zngur(scope),
                                use_path: use_path.map(|x| scope.resolve_path(x)),
                                deref,
                            });
                        }
                        ParsedTypeItem::CppValue { field, cpp_type } => {
                            cpp_value = Some(CppValue(field.to_owned(), cpp_type.to_owned()));
                        }
                        ParsedTypeItem::CppRef { cpp_type } => {
                            match layout_span {
                                Some(span) => {
                                    ctx.add_error_str("Duplicate layout policy found", span);
                                    continue;
                                }
                                None => {
                                    layout = Some(LayoutPolicy::ZERO_SIZED_TYPE);
                                    layout_span = Some(item_span);
                                }
                            }
                            cpp_ref = Some(CppRef(cpp_type.to_owned()));
                        }
                        ParsedTypeItem::MatchOnCfg(match_) => {
                            let result = match_.eval(ctx);
                            if let Some(result) = result {
                                to_process.extend(result);
                            }
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
                        ctx.add_report(
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
                if let Some(layout) = layout {
                    checked_merge(
                        ZngurType {
                            ty: ty.inner.to_zngur(scope),
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
                } else {
                    ctx.add_error_str(
                        "No layout policy found for this type. \
Use one of `#layout(size = X, align = Y)`, `#heap_allocated` or `#only_by_ref`.",
                        ty.span,
                    );
                };
            }
            ProcessedItem::Trait { tr, methods } => {
                checked_merge(
                    ZngurTrait {
                        tr: tr.inner.to_zngur(scope),
                        methods: methods.into_iter().map(|m| m.to_zngur(scope)).collect(),
                    },
                    r,
                    tr.span,
                    ctx,
                );
            }
            ProcessedItem::Fn(f) => {
                let method = f.inner.to_zngur(scope);
                checked_merge(
                    ZngurFn {
                        path: RustPathAndGenerics {
                            path: scope.simple_relative_path(&method.name),
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
                            let method = method.inner.to_zngur(scope);
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
                                    tr: tr.map(|x| x.to_zngur(scope)),
                                    ty: ty.inner.to_zngur(scope),
                                    methods: methods
                                        .into_iter()
                                        .map(|x| x.to_zngur(scope))
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
                    ctx.add_error_str(
                        "Using `#convert_panic_to_exception` in imported zngur files is not supported. This directive can only be used in the main zngur file.",
                        span,
                    );
                    return;
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
    Impl(ParsedRustTrait<'a>, Vec<&'a str>),
    Tuple(Vec<ParsedRustType<'a>>),
    Adt(ParsedRustPathAndGenerics<'a>),
}

impl ParsedRustType<'_> {
    fn to_zngur(self, scope: &Scope<'_>) -> RustType {
        match self {
            ParsedRustType::Primitive(s) => RustType::Primitive(s),
            ParsedRustType::Ref(m, s) => RustType::Ref(m, Box::new(s.to_zngur(scope))),
            ParsedRustType::Raw(m, s) => RustType::Raw(m, Box::new(s.to_zngur(scope))),
            ParsedRustType::Boxed(s) => RustType::Boxed(Box::new(s.to_zngur(scope))),
            ParsedRustType::Slice(s) => RustType::Slice(Box::new(s.to_zngur(scope))),
            ParsedRustType::Dyn(tr, bounds) => RustType::Dyn(
                tr.to_zngur(scope),
                bounds.into_iter().map(|x| x.to_owned()).collect(),
            ),
            ParsedRustType::Impl(tr, bounds) => RustType::Impl(
                tr.to_zngur(scope),
                bounds.into_iter().map(|x| x.to_owned()).collect(),
            ),
            ParsedRustType::Tuple(v) => {
                RustType::Tuple(v.into_iter().map(|s| s.to_zngur(scope)).collect())
            }
            ParsedRustType::Adt(s) => RustType::Adt(s.to_zngur(scope)),
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
    fn to_zngur(self, scope: &Scope<'_>) -> RustTrait {
        match self {
            ParsedRustTrait::Normal(s) => RustTrait::Normal(s.to_zngur(scope)),
            ParsedRustTrait::Fn {
                name,
                inputs,
                output,
            } => RustTrait::Fn {
                name: name.to_owned(),
                inputs: inputs.into_iter().map(|s| s.to_zngur(scope)).collect(),
                output: Box::new(output.to_zngur(scope)),
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
    fn to_zngur(self, scope: &Scope<'_>) -> RustPathAndGenerics {
        RustPathAndGenerics {
            path: scope.resolve_path(self.path),
            generics: self
                .generics
                .into_iter()
                .map(|x| x.to_zngur(scope))
                .collect(),
            named_generics: self
                .named_generics
                .into_iter()
                .map(|(name, x)| (name.to_owned(), x.to_zngur(scope)))
                .collect(),
        }
    }
}

struct ParseContext<'a, 'b> {
    path: std::path::PathBuf,
    text: &'a str,
    depth: usize,
    reports: Vec<Report<'b, (String, std::ops::Range<usize>)>>,
    source_cache: std::collections::HashMap<std::path::PathBuf, String>,
    /// All .zng files processed during parsing (main file + imports)
    processed_files: Vec<std::path::PathBuf>,
    cfg_provider: Box<dyn RustCfgProvider>,
}

impl<'a, 'b> ParseContext<'a, 'b> {
    fn new(path: std::path::PathBuf, text: &'a str, cfg: Box<dyn RustCfgProvider>) -> Self {
        let processed_files = vec![path.clone()];
        Self {
            path,
            text,
            depth: 0,
            reports: Vec::new(),
            source_cache: HashMap::new(),
            processed_files,
            cfg_provider: cfg,
        }
    }

    fn with_depth(
        path: std::path::PathBuf,
        text: &'a str,
        depth: usize,
        cfg: Box<dyn RustCfgProvider>,
    ) -> Self {
        let processed_files = vec![path.clone()];
        Self {
            path,
            text,
            depth,
            reports: Vec::new(),
            source_cache: HashMap::new(),
            processed_files,
            cfg_provider: cfg,
        }
    }

    fn filename(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }

    fn add_report(&mut self, report: Report<'b, (String, std::ops::Range<usize>)>) {
        self.reports.push(report);
    }
    fn add_errors<'err_src>(&mut self, errs: impl Iterator<Item = Rich<'err_src, String>>) {
        let filename = self.filename().to_string();
        self.reports.extend(errs.map(|e| {
            Report::build(ReportKind::Error, &filename, e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(label, span)| {
                    Label::new((filename.clone(), span.into_range()))
                        .with_message(format!("while parsing this {}", label))
                        .with_color(Color::Yellow)
                }))
                .finish()
        }));
    }

    fn add_error_str(&mut self, error: &str, span: Span) {
        self.add_errors([Rich::custom(span, error)].into_iter());
    }

    fn consume_from(&mut self, mut other: ParseContext<'_, 'b>) {
        // Always merge processed files, regardless of errors
        self.processed_files.append(&mut other.processed_files);
        if other.has_errors() {
            self.reports.extend(other.reports);
            self.source_cache.insert(other.path, other.text.to_string());
            self.source_cache.extend(other.source_cache);
        }
    }

    fn has_errors(&self) -> bool {
        !self.reports.is_empty()
    }

    #[cfg(test)]
    fn emit_ariadne_errors(&self) -> ! {
        let mut r = Vec::<u8>::new();
        for err in &self.reports {
            err.write(
                sources(
                    [(self.filename().to_string(), self.text)]
                        .into_iter()
                        .chain(
                            self.source_cache
                                .iter()
                                .map(|(path, text)| {
                                    (
                                        path.file_name().unwrap().to_str().unwrap().to_string(),
                                        text.as_str(),
                                    )
                                })
                                .collect::<Vec<_>>()
                                .into_iter(),
                        ),
                ),
                &mut r,
            )
            .unwrap();
        }
        std::panic::resume_unwind(Box::new(tests::ErrorText(
            String::from_utf8(strip_ansi_escapes::strip(r)).unwrap(),
        )));
    }

    #[cfg(not(test))]
    fn emit_ariadne_errors(&self) -> ! {
        for err in &self.reports {
            err.eprint(sources(
                [(self.filename().to_string(), self.text)]
                    .into_iter()
                    .chain(
                        self.source_cache
                            .iter()
                            .map(|(path, text)| {
                                (
                                    path.file_name().unwrap().to_str().unwrap().to_string(),
                                    text.as_str(),
                                )
                            })
                            .collect::<Vec<_>>()
                            .into_iter(),
                    ),
            ))
            .unwrap();
        }
        exit(101);
    }

    fn get_config_provider(&self) -> &dyn RustCfgProvider {
        self.cfg_provider.as_ref()
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
    fn parse_into(zngur: &mut ZngurSpec, ctx: &mut ParseContext, resolver: &impl ImportResolver) {
        let (tokens, errs) = lexer().parse(ctx.text).into_output_errors();
        let Some(tokens) = tokens else {
            ctx.add_errors(errs.into_iter().map(|e| e.map_token(|c| c.to_string())));
            ctx.emit_ariadne_errors();
        };
        let tokens: ParserInput<'_> = tokens.as_slice().map(
            (ctx.text.len()..ctx.text.len()).into(),
            Box::new(|(t, s)| (t, s)),
        );
        let (ast, errs) = file_parser()
            .map_with(|ast, extra| (ast, extra.span()))
            .parse_with_state(tokens, &mut extra::SimpleState(ZngParserState::default()))
            .into_output_errors();
        let Some(ast) = ast else {
            ctx.add_errors(errs.into_iter().map(|e| e.map_token(|c| c.to_string())));
            ctx.emit_ariadne_errors();
        };

        let (aliases, items) = partition_parsed_items(
            ast.0
                .0
                .into_iter()
                .map(|item| process_parsed_item(item, ctx)),
        );
        ProcessedZngFile::new(aliases, items).into_zngur_spec(zngur, ctx);

        if let Some(dirname) = ctx.path.to_owned().parent() {
            for import in std::mem::take(&mut zngur.imports) {
                match resolver.resolve_import(dirname, &import.0) {
                    Ok(text) => {
                        let mut nested_ctx = ParseContext::with_depth(
                            dirname.join(&import.0),
                            &text,
                            ctx.depth + 1,
                            ctx.get_config_provider().clone_box(),
                        );
                        Self::parse_into(zngur, &mut nested_ctx, resolver);
                        ctx.consume_from(nested_ctx);
                    }
                    Err(_) => {
                        // TODO: emit a better error. How should we get a span here?
                        // I'd like to avoid putting a ParsedImportPath in ZngurSpec, and
                        // also not have to pass a filename to add_to_zngur_spec.
                        ctx.add_report(
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

    /// Parse a .zng file and return both the spec and list of all processed files.
    pub fn parse(path: std::path::PathBuf, cfg: Box<dyn RustCfgProvider>) -> ParseResult {
        let mut zngur = ZngurSpec::default();
        zngur.rust_cfg.extend(cfg.get_cfg_pairs());
        let text = std::fs::read_to_string(&path).unwrap();
        let mut ctx = ParseContext::new(path.clone(), &text, cfg.clone_box());
        Self::parse_into(&mut zngur, &mut ctx, &DefaultImportResolver);
        if ctx.has_errors() {
            // add report of cfg values used
            ctx.add_report(
                Report::build(
                    ReportKind::Custom("cfg values", ariadne::Color::Green),
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    0,
                )
                .with_message(
                    cfg.get_cfg_pairs()
                        .into_iter()
                        .map(|(key, value)| match value {
                            Some(value) => format!("{key}=\"{value}\""),
                            None => key,
                        })
                        .join("\n")
                        .to_string(),
                )
                .finish(),
            );
            ctx.emit_ariadne_errors();
        }
        ParseResult {
            spec: zngur,
            processed_files: ctx.processed_files,
        }
    }

    /// Parse a .zng file from a string. Mainly useful for testing.
    pub fn parse_str(text: &str, cfg: impl RustCfgProvider + 'static) -> ParseResult {
        let mut zngur = ZngurSpec::default();
        let mut ctx = ParseContext::new(std::path::PathBuf::from("test.zng"), text, Box::new(cfg));
        Self::parse_into(&mut zngur, &mut ctx, &DefaultImportResolver);
        if ctx.has_errors() {
            ctx.emit_ariadne_errors();
        }
        ParseResult {
            spec: zngur,
            processed_files: ctx.processed_files,
        }
    }

    #[cfg(test)]
    pub(crate) fn parse_str_with_resolver(
        text: &str,
        cfg: impl RustCfgProvider + 'static,
        resolver: &impl ImportResolver,
    ) -> ParseResult {
        let mut zngur = ZngurSpec::default();
        let mut ctx = ParseContext::new(std::path::PathBuf::from("test.zng"), text, Box::new(cfg));
        Self::parse_into(&mut zngur, &mut ctx, resolver);
        if ctx.has_errors() {
            ctx.emit_ariadne_errors();
        }
        ParseResult {
            spec: zngur,
            processed_files: ctx.processed_files,
        }
    }
}

pub(crate) enum ProcessedItemOrAlias<'a> {
    Ignore,
    Processed(ProcessedItem<'a>),
    Alias(ParsedAlias<'a>),
    ChildItems(Vec<ProcessedItemOrAlias<'a>>),
}

fn process_parsed_item<'a>(
    item: ParsedItem<'a>,
    ctx: &mut ParseContext,
) -> ProcessedItemOrAlias<'a> {
    use ProcessedItemOrAlias as Ret;
    match item {
        ParsedItem::Alias(alias) => Ret::Alias(alias),
        ParsedItem::ConvertPanicToException(span) => {
            Ret::Processed(ProcessedItem::ConvertPanicToException(span))
        }
        ParsedItem::UnstableFeature(_) => {
            // ignore
            Ret::Ignore
        }
        ParsedItem::CppAdditionalInclude(inc) => {
            Ret::Processed(ProcessedItem::CppAdditionalInclude(inc))
        }
        ParsedItem::Mod { path, items } => {
            let (aliases, items) = partition_parsed_items(
                items.into_iter().map(|item| process_parsed_item(item, ctx)),
            );
            Ret::Processed(ProcessedItem::Mod {
                path,
                items,
                aliases,
            })
        }
        ParsedItem::Type { ty, items } => Ret::Processed(ProcessedItem::Type { ty, items }),
        ParsedItem::Trait { tr, methods } => Ret::Processed(ProcessedItem::Trait { tr, methods }),
        ParsedItem::Fn(method) => Ret::Processed(ProcessedItem::Fn(method)),
        ParsedItem::ExternCpp(items) => Ret::Processed(ProcessedItem::ExternCpp(items)),
        ParsedItem::Import(path) => Ret::Processed(ProcessedItem::Import(path)),
        ParsedItem::MatchOnCfg(match_) => Ret::ChildItems(
            match_
                .eval(ctx)
                .unwrap_or_default() // unwrap or empty
                .into_iter()
                .map(|item| item.inner)
                .collect(),
        ),
    }
}

fn partition_parsed_items<'a>(
    items: impl IntoIterator<Item = ProcessedItemOrAlias<'a>>,
) -> (Vec<ParsedAlias<'a>>, Vec<ProcessedItem<'a>>) {
    let mut aliases = Vec::new();
    let mut processed = Vec::new();
    for item in items.into_iter() {
        match item {
            ProcessedItemOrAlias::Ignore => continue,
            ProcessedItemOrAlias::Processed(p) => processed.push(p),
            ProcessedItemOrAlias::Alias(a) => aliases.push(a),
            ProcessedItemOrAlias::ChildItems(children) => {
                let (child_aliases, child_items) = partition_parsed_items(children);
                aliases.extend(child_aliases);
                processed.extend(child_items);
            }
        }
    }
    (aliases, processed)
}

impl<'a> ProcessedZngFile<'a> {
    fn new(aliases: Vec<ParsedAlias<'a>>, items: Vec<ProcessedItem<'a>>) -> Self {
        ProcessedZngFile { aliases, items }
    }

    fn into_zngur_spec(self, zngur: &mut ZngurSpec, ctx: &mut ParseContext) {
        let root_scope = Scope::new_root(self.aliases);

        for item in self.items {
            item.add_to_zngur_spec(zngur, &root_scope, ctx);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token<'a> {
    Arrow,
    ArrowArm,
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
    Pipe,
    Underscore,
    Dot,
    Bang,
    KwAs,
    KwAsync,
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
    KwIf,
    KwElse,
    KwMatch,
    Ident(&'a str),
    Str(&'a str),
    Number(usize),
}

impl<'a> Token<'a> {
    fn ident_or_kw(ident: &'a str) -> Self {
        match ident {
            "as" => Token::KwAs,
            "async" => Token::KwAsync,
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
            "if" => Token::KwIf,
            "else" => Token::KwElse,
            "match" => Token::KwMatch,
            x => Token::Ident(x),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Arrow => write!(f, "->"),
            Token::ArrowArm => write!(f, "=>"),
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
            Token::Pipe => write!(f, "|"),
            Token::Underscore => write!(f, "_"),
            Token::Dot => write!(f, "."),
            Token::Bang => write!(f, "!"),
            Token::KwAs => write!(f, "as"),
            Token::KwAsync => write!(f, "async"),
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
            Token::KwIf => write!(f, "if"),
            Token::KwElse => write!(f, "else"),
            Token::KwMatch => write!(f, "match"),
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
            just("=>").to(Token::ArrowArm),
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
            just("|").to(Token::Pipe),
            just("_").to(Token::Underscore),
            just(".").to(Token::Dot),
            just("!").to(Token::Bang),
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

fn alias<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
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
-> impl Parser<'a, ParserInput<'a>, ParsedZngFile<'a>, ZngParserExtra<'a>> + Clone {
    item().repeated().collect::<Vec<_>>().map(ParsedZngFile)
}

fn rust_type<'a>() -> Boxed<'a, 'a, ParserInput<'a>, ParsedRustType<'a>, ZngParserExtra<'a>> {
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
            .or(just(Token::KwImpl))
            .then(rust_trait(parser.clone()))
            .then(
                just(Token::Plus)
                    .ignore_then(select! {
                        Token::Ident(c) => c,
                    })
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|((token, first), rest)| match token {
                Token::KwDyn => ParsedRustType::Dyn(first, rest),
                Token::KwImpl => ParsedRustType::Impl(first, rest),
                _ => unreachable!(),
            });
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
    rust_type: Boxed<'a, 'a, ParserInput<'a>, ParsedRustType<'a>, ZngParserExtra<'a>>,
) -> impl Parser<
    'a,
    ParserInput<'a>,
    Vec<Either<(&'a str, ParsedRustType<'a>), ParsedRustType<'a>>>,
    ZngParserExtra<'a>,
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
    rust_type: Boxed<'a, 'a, ParserInput<'a>, ParsedRustType<'a>, ZngParserExtra<'a>>,
) -> impl Parser<'a, ParserInput<'a>, ParsedRustPathAndGenerics<'a>, ZngParserExtra<'a>> + Clone {
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
    rust_type: Boxed<'a, 'a, ParserInput<'a>, ParsedRustType<'a>, ZngParserExtra<'a>>,
) -> impl Parser<'a, ParserInput<'a>, (Vec<ParsedRustType<'a>>, ParsedRustType<'a>), ZngParserExtra<'a>>
+ Clone {
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
    parser: impl Parser<'a, ParserInput<'a>, T, ZngParserExtra<'a>> + Clone,
) -> impl Parser<'a, ParserInput<'a>, Spanned<T>, ZngParserExtra<'a>> + Clone {
    parser.map_with(|inner, extra| Spanned {
        inner,
        span: extra.span(),
    })
}

fn rust_trait<'a>(
    rust_type: Boxed<'a, 'a, ParserInput<'a>, ParsedRustType<'a>, ZngParserExtra<'a>>,
) -> impl Parser<'a, ParserInput<'a>, ParsedRustTrait<'a>, ZngParserExtra<'a>> + Clone {
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

fn method<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedMethod<'a>, ZngParserExtra<'a>> + Clone {
    spanned(just(Token::KwAsync))
        .or_not()
        .then_ignore(just(Token::KwFn))
        .then(select! {
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
        .map(|(((opt_async, name), generics), args)| {
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
            let mut output = args.1;
            if let Some(async_kw) = opt_async {
                output = ParsedRustType::Impl(
                    ParsedRustTrait::Normal(ParsedRustPathAndGenerics {
                        path: ParsedPath {
                            start: ParsedPathStart::Absolute,
                            segments: vec!["std", "future", "Future"],
                            span: async_kw.span,
                        },
                        generics: vec![],
                        named_generics: vec![("Output", output)],
                    }),
                    vec![],
                )
            }
            ParsedMethod {
                name,
                receiver,
                generics,
                inputs,
                output,
            }
        })
}

fn inner_type_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedTypeItem<'a>, ZngParserExtra<'a>> + Clone {
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
                .clone()
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
        )
        .map(ParsedLayoutPolicy::StackAllocated)
        .or(just([Token::Sharp, Token::Ident("layout_conservative")])
            .ignore_then(
                property_item
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(ParsedLayoutPolicy::Conservative))
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
                    Token::Number(c) => Some(c),
                    Token::Ident("auto") => None,
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
    recursive(|item| {
        let inner_item = choice((
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
        ));

        let match_stmt =
            conditional_item::<_, CfgConditional<'a>, NItems>(item).map(ParsedTypeItem::MatchOnCfg);

        choice((match_stmt, inner_item.then_ignore(just(Token::Semicolon))))
    })
}

fn type_item<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
    just(Token::KwType)
        .ignore_then(spanned(rust_type()))
        .then(
            spanned(inner_type_item())
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(ty, items)| ParsedItem::Type { ty, items })
        .boxed()
}

fn trait_item<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone
{
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

fn fn_item<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
    spanned(method())
        .then_ignore(just(Token::Semicolon))
        .map(ParsedItem::Fn)
}

fn additional_include_item<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
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
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
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

fn unstable_feature<'a>()
-> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
    just([Token::Sharp, Token::Ident("unstable")]).ignore_then(
        select! { Token::Ident(feat) => feat }
            .delimited_by(just(Token::ParenOpen), just(Token::ParenClose))
            .try_map_with(|feat, e| match feat {
                "cfg_match" => {
                    let ctx: &mut extra::SimpleState<ZngParserState> = e.state();
                    ctx.unstable_features.cfg_match = true;
                    Ok(ParsedItem::UnstableFeature("cfg_match"))
                }
                "cfg_if" => {
                    let ctx: &mut extra::SimpleState<ZngParserState> = e.state();
                    ctx.unstable_features.cfg_if = true;
                    Ok(ParsedItem::UnstableFeature("cfg_if"))
                }
                _ => Err(Rich::custom(
                    e.span(),
                    format!("unknown unstable feature '{feat}'"),
                )),
            }),
    )
}

fn item<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone {
    recursive(|item| {
        choice((
            unstable_feature(),
            just(Token::KwMod)
                .ignore_then(path())
                .then(
                    item.clone()
                        .repeated()
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
            conditional_item::<_, CfgConditional<'a>, NItems>(item).map(ParsedItem::MatchOnCfg),
        ))
    })
    .boxed()
}

fn import_item<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedItem<'a>, ZngParserExtra<'a>> + Clone
{
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

fn path<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedPath<'a>, ZngParserExtra<'a>> + Clone {
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

impl<'a> conditional::BodyItem for crate::ParsedTypeItem<'a> {
    type Processed = Self;

    fn process(self, _ctx: &mut ParseContext) -> Self::Processed {
        self
    }
}

impl<'a> conditional::BodyItem for crate::ParsedItem<'a> {
    type Processed = ProcessedItemOrAlias<'a>;

    fn process(self, ctx: &mut ParseContext) -> Self::Processed {
        crate::process_parsed_item(self, ctx)
    }
}
