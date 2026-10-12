use std::panic::catch_unwind;

use expect_test::{Expect, expect};
use zngur_def::{LayoutPolicy, RustPathAndGenerics, RustType, ZngurSpec};

use crate::{
    DefaultImportResolver, ImportResolver, ParsedZngFile, ReportSink,
    cfg::{InMemoryRustCfgProvider, NullCfg, RustCfgProvider},
};

/// Parse a .zng file from a string.
fn parse_str(
    text: &str,
    cfg: impl RustCfgProvider + 'static,
    warning_sink: impl FnMut(&str),
) -> crate::ParseResult {
    let mut error_sink = TestErrorSink::new(warning_sink);
    ParsedZngFile::parse_str_with_resolver(
        text,
        "test.zng",
        cfg,
        &DefaultImportResolver,
        &mut error_sink,
    )
}

fn parse_str_with_resolver(
    text: &str,
    cfg: impl RustCfgProvider + 'static,
    resolver: &impl ImportResolver,
    warning_sink: impl FnMut(&str),
) -> crate::ParseResult {
    let mut report_sink = TestErrorSink::new(warning_sink);
    ParsedZngFile::parse_str_with_resolver(text, "test.zng", cfg, resolver, &mut report_sink)
}

#[derive(Debug)]
struct TestErrorSink<F>
where
    F: FnMut(&str),
{
    error_buffer: Vec<u8>,
    warning_sink: F,
}

impl<F> TestErrorSink<F>
where
    F: FnMut(&str),
{
    fn new(f: F) -> Self {
        Self {
            error_buffer: Default::default(),
            warning_sink: f,
        }
    }
}

impl<F> ReportSink for TestErrorSink<F>
where
    F: FnMut(&str),
{
    fn sink_report(&mut self, report: &crate::ReportEntry, source_cache: crate::SourceCache) {
        if report.fatal {
            report
                .report
                .write(source_cache, &mut self.error_buffer)
                .unwrap();
        } else {
            let mut buf = Vec::<u8>::new();
            report.report.write(source_cache, &mut buf).unwrap();
            (self.warning_sink)(&String::from_utf8(strip_ansi_escapes::strip(buf)).unwrap());
        };
    }
}

/// unwind with the full error on drop if not empty
impl<F> Drop for TestErrorSink<F>
where
    F: FnMut(&str),
{
    fn drop(&mut self) {
        if !self.error_buffer.is_empty() {
            std::panic::resume_unwind(Box::new(ErrorText(
                String::from_utf8(strip_ansi_escapes::strip(&self.error_buffer)).unwrap(),
            )))
        }
    }
}

fn check_success(zng: &str) {
    let _ = parse_str(zng, NullCfg, |msg| {
        unreachable!("an unexpected warning was emitted: {msg}");
    });
}

pub struct ErrorText(pub String);

fn check_fail(zng: &str, error: Expect) {
    let r = catch_unwind(|| {
        let _ = parse_str(zng, NullCfg, |_| {});
    });
    match r {
        Ok(_) => panic!("Parsing succeeded but we expected fail"),
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => error.assert_eq(&t.0),
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

fn check_fail_with_cfg(
    zng: &str,
    cfg: impl RustCfgProvider + std::panic::UnwindSafe + 'static,
    error: Expect,
) {
    let r = catch_unwind(|| {
        let _ = parse_str(zng, cfg, |_| {});
    });
    match r {
        Ok(_) => panic!("Parsing succeeded but we expected fail"),
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => error.assert_eq(&t.0),
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

fn check_import_fail(zng: &str, error: Expect, resolver: &MockFilesystem) {
    let r = catch_unwind(|| {
        let _ = parse_str_with_resolver(zng, NullCfg, resolver, |_| {});
    });

    match r {
        Ok(_) => panic!("Parsing succeeded but we expected fail"),
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => error.assert_eq(&t.0),
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

// useful for debugging a test that should succeeded on parse
fn catch_parse_fail(
    zng: &str,
    cfg: impl RustCfgProvider + std::panic::UnwindSafe + 'static,
) -> crate::ParseResult {
    let r = catch_unwind(move || parse_str(zng, cfg, |_| {}));

    match r {
        Ok(r) => r,
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => {
                eprintln!("{}", t.0);
                crate::ParseResult {
                    spec: ZngurSpec::default(),
                    processed_files: Vec::new(),
                    errors: 1,
                    warnings: 0,
                }
            }
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

#[test]
fn parse_unit() {
    check_fail(
        r#"
type () {
    #layout(size = 0, align = 1);
    wellknown_traits(Copy);
}
    "#,
        expect![[r#"
            Error: Unit type is declared implicitly. Remove this entirely.
               ╭─[ test.zng:2:6 ]
               │
             2 │ type () {
               │      ─┬  
               │       ╰── Unit type is declared implicitly. Remove this entirely.
            ───╯
        "#]],
    );
}

#[test]
fn parse_tuple() {
    check_success(
        r#"
type (i8, u8) {
    #layout(size = 0, align = 1);
}
    "#,
    );
}

#[test]
fn typo_in_wellknown_trait() {
    check_fail(
        r#"
type () {
    #layout(size = 0, align = 1);
    welcome_traits(Copy);
}
    "#,
        expect![[r#"
            Error: found 'welcome_traits' expected '#', 'variant', 'wellknown_traits', 'non_exhaustive', 'constructor', 'field', 'async', 'fn', or '}'
               ╭─[ test.zng:4:5 ]
               │
             4 │     welcome_traits(Copy);
               │     ───────┬──────  
               │            ╰──────── found 'welcome_traits' expected '#', 'variant', 'wellknown_traits', 'non_exhaustive', 'constructor', 'field', 'async', 'fn', or '}'
            ───╯
        "#]],
    );
}

#[test]
fn multiple_layout_policies() {
    check_fail(
        r#"
type ::std::string::String {
    #layout(size = 24, align = 8);
    #heap_allocated;
}
    "#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[ test.zng:4:5 ]
               │
             4 │     #heap_allocated;
               │     ───────┬───────  
               │            ╰───────── Duplicate layout policy found
            ───╯
        "#]],
    );
}

#[test]
fn only_by_ref_needs_no_layout_info() {
    check_fail(
        r#"
type c++::osmium::Way {
    #layout(size = 1, align = 2);

    #only_by_ref;
}
    "#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[ test.zng:5:5 ]
               │
             5 │     #only_by_ref;
               │     ──────┬─────  
               │           ╰─────── Duplicate layout policy found
            ───╯
        "#]],
    );
    check_success(
        r#"
type c++::osmium::Way {
    #only_by_ref;
}
    "#,
    );
}

#[test]
fn cpp_type_with_heap_allocated_layout() {
    let result = parse_str(
        r#"
type c++::osmium::Way {
    #heap_allocated;
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = result.spec.types.first().expect("no type parsed");
    assert_eq!(ty.layout, Some(LayoutPolicy::HeapAllocated));
}

#[test]
fn cpp_type_with_exact_layout() {
    let parsed = parse_str(
        r#"
type c++::osmium::Way {
    #layout(size = 16, align = 8);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_eq!(
        ty.layout,
        Some(LayoutPolicy::StackAllocated { size: 16, align: 8 })
    );
}

#[test]
fn cpp_type_with_conservative_layout() {
    let parsed = parse_str(
        r#"
type c++::osmium::Way {
    #layout_conservative(size = 32, align = 16);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_eq!(
        ty.layout,
        Some(LayoutPolicy::Conservative {
            size: 32,
            align: 16
        })
    );
}

#[test]
fn cpp_type_without_layout_is_an_error() {
    check_fail(
        r#"
type c++::osmium::Way {
}
    "#,
        expect![[r#"
            Error: No layout policy found for type osmium::Way.
               ╭─[ test.zng:1:1 ]
               │
             2 │ type c++::osmium::Way {
               │      ────────┬───────  
               │              ╰───────── Type first declared here.
               │ 
               │ Note: Use one of `#layout(size = X, align = Y)`, `#heap_allocated` or `#only_by_ref`.
            ───╯
        "#]],
    );
}

macro_rules! assert_ty_path {
    ($path_expected:expr, $ty:expr) => {{
        let RustType::Adt(RustPathAndGenerics { path: p, .. }) = $ty else {
            panic!("type `{:?}` is not a path", $ty);
        };
        assert_eq!(p.as_slice(), $path_expected);
    }};
}

#[test]
fn alias_expands_correctly() {
    let parsed = parse_str(
        r#"
use ::std::string::String as MyString;
type MyString {
    #layout(size = 24, align = 8);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    let RustType::Adt(RustPathAndGenerics { path: p, .. }) = &ty.ty else {
        panic!("no match?");
    };
    assert_eq!(p.as_slice(), ["std", "string", "String"]);
}

#[test]
fn alias_expands_nearest_scope_first() {
    let parsed = parse_str(
        r#"
use ::std::string::String as MyString;
mod crate {
    use MyLocalString as MyString;
    type MyString {
        #layout(size = 24, align = 8);
    }
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    let RustType::Adt(RustPathAndGenerics { path: p, .. }) = &ty.ty else {
        panic!("no match?");
    };
    assert_eq!(p.as_slice(), ["crate", "MyLocalString"]);
}

#[test]
fn parse_variants() {
    println!("meow");

    let parsed = parse_str(
        r#"
type Option<i32> {
    #layout(size = 8, align = 4);
    variant None { }
    variant Some {
        field 0 (offset = auto, type = i32);
    }
}
        "#,
        NullCfg,
        |_| {},
    );

    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_eq!(ty.variants.len(), 2, "should have two variants");

    assert_eq!(ty.variants[0].name, "None");
    assert_eq!(ty.variants[0].fields.len(), 0);

    assert_eq!(ty.variants[1].name, "Some");
    assert_eq!(ty.variants[1].fields.len(), 1);
    assert_eq!(ty.variants[1].fields[0].name, "0");
    assert_eq!(
        ty.variants[1].fields[0].ty,
        RustType::Primitive(zngur_def::PrimitiveRustType::Int(32))
    );
    assert_eq!(ty.variants[1].fields[0].offset, None);
}

struct MockFilesystem {
    files: std::collections::HashMap<std::path::PathBuf, String>,
}

impl MockFilesystem {
    fn new(
        files: impl IntoIterator<Item = (impl Into<std::path::PathBuf>, impl Into<String>)>,
    ) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

impl ImportResolver for MockFilesystem {
    fn resolve_import(
        &self,
        cwd: &std::path::Path,
        relpath: &std::path::Path,
    ) -> Result<String, String> {
        let path = cwd.join(relpath);
        self.files
            .get(&path)
            .cloned()
            .ok_or_else(|| format!("File not found: {}", path.display()))
    }
}

#[test]
fn import_parser_test() {
    let resolver = MockFilesystem::new(vec![(
        "./relative/path.zng",
        "type Imported { #layout(size = 1, align = 1); }",
    )]);

    let parsed = parse_str_with_resolver(
        r#"
merge "./relative/path.zng";
type Example {
    #layout(size = 1, align = 1);
}
    "#,
        NullCfg,
        &resolver,
        |_| {},
    );
    assert_eq!(parsed.spec.types.len(), 2);
}

#[test]
fn module_import_prohibited() {
    let resolver = MockFilesystem::new(vec![] as Vec<(&str, &str)>);

    check_import_fail(
        r#"
    merge "foo/bar.zng";
    "#,
        expect![[r#"
            Error: Module import is not supported. Use a relative path instead.
               ╭─[ test.zng:2:5 ]
               │
             2 │     merge "foo/bar.zng";
               │     ──────────┬─────────  
               │               ╰─────────── Module import is not supported. Use a relative path instead.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn import_has_conflict() {
    // Test that an import which introduces a conflict produces a reasonable error message.
    let resolver = MockFilesystem::new(vec![(
        "./a.zng",
        r#"
      type A {
        #layout(size = 1, align = 1);
      }
    "#,
    )]);

    check_import_fail(
        r#"
    merge "./a.zng";
    type A {
      #layout(size = 2, align = 2);
    }
"#,
        expect![[r#"
            Error: Conflicting layout policy found
               ╭─[ ./a.zng:2:12 ]
               │
             2 │       type A {
               │            ┬  
               │            ╰── Conflicting layout policy found
             3 │         #layout(size = 1, align = 1);
               │         ──────────────┬─────────────  
               │                       ╰─────────────── declaration here ...
               │
               ├─[ test.zng:4:7 ]
               │
             4 │       #layout(size = 2, align = 2);
               │       ──────────────┬─────────────  
               │                     ╰─────────────── conflicts with the declaration here.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn missing_layout_across_files() {
    // Test that a type defined in multiple places without a layout produces a reasonable error message.
    let resolver = MockFilesystem::new(vec![(
        "./a.zng",
        r#"
      type A {
      }
    "#,
    )]);

    check_import_fail(
        r#"
    merge "./a.zng";
    type A {}
"#,
        expect![[r#"
            Error: No layout policy found for type ::A.
               ╭─[ test.zng:1:1 ]
               │
             3 │     type A {}
               │          ┬  
               │          ╰── Type first declared here.
               │
               ├─[ ./a.zng:2:12 ]
               │
             2 │       type A {
               │            ┬  
               │            ╰── Type also declared here
               │ 
               │ Note: Use one of `#layout(size = X, align = Y)`, `#heap_allocated` or `#only_by_ref`.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn missing_layout_across_files_with_template() {
    // Test that a type defined in multiple places without a layout produces a reasonable error message.
    let resolver = MockFilesystem::new(vec![(
        "./a.zng",
        r#"
      type A<B> {
      }
    "#,
    )]);

    check_import_fail(
        r#"
    #unstable(template_types)
    merge "./a.zng";
    type<T> A<T> {}
    type A<B> {}
"#,
        expect![[r#"
            Error: No layout policy found for type ::A::<::B>.
               ╭─[ test.zng:1:1 ]
               │
             5 │     type A<B> {}
               │          ──┬─  
               │            ╰─── Type first declared here.
               │
               ├─[ ./a.zng:2:12 ]
               │
             2 │       type A<B> {
               │            ──┬─  
               │              ╰─── Type also declared here
               │
               ├─[ test.zng:1:1 ]
               │
             4 │     type<T> A<T> {}
               │             ──┬─  
               │               ╰─── Matching template defined here
               │ 
               │ Note: Use one of `#layout(size = X, align = Y)`, `#heap_allocated` or `#only_by_ref`.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn import_not_found() {
    let resolver = MockFilesystem::new(vec![] as Vec<(&str, &str)>);
    check_import_fail(
        r#"
    merge "./a.zng";
    "#,
        expect![[r#"
            Error: Failed to process merge file `./a.zng`: File not found: ./a.zng
        "#]],
        &resolver,
    );
}

#[test]
fn import_has_mismatched_method_signature() {
    let resolver = MockFilesystem::new(vec![(
        "./a.zng",
        "type A { #layout(size = 1, align = 1); fn foo(i32) -> i32; }",
    )]);

    check_import_fail(
        r#"
  merge "./a.zng";
  type A {
    #layout(size = 1, align = 1);
    fn foo(i64) -> i64;
  }
  "#,
        expect![[r#"
            Error: Method mismatch
               ╭─[ ./a.zng:1:6 ]
               │
             1 │ type A { #layout(size = 1, align = 1); fn foo(i32) -> i32; }
               │      ┬                                 ─────────┬─────────  
               │      ╰────────────────────────────────────────────────────── Method mismatch
               │                                                 │           
               │                                                 ╰─────────── declaration here ...
               │
               ├─[ test.zng:5:5 ]
               │
             5 │     fn foo(i64) -> i64;
               │     ─────────┬─────────  
               │              ╰─────────── conflicts with the declaration here.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn import_has_mismatched_field() {
    let resolver = MockFilesystem::new(vec![(
        "./a.zng",
        "type A {
        #layout(size = 1, align = 1);
        field x (offset = 0, type = i32);
    }",
    )]);

    check_import_fail(
        r#"
  merge "./a.zng";
  type A {
    #layout(size = 1, align = 1);
    field x (offset = 0, type = i64);
  }
  "#,
        expect![[r#"
            Error: Field mismatch
               ╭─[ ./a.zng:1:6 ]
               │
             1 │ type A {
               │      ┬  
               │      ╰── Field mismatch
               │ 
             3 │         field x (offset = 0, type = i32);
               │         ────────────────┬────────────────  
               │                         ╰────────────────── declaration here ...
               │
               ├─[ test.zng:5:5 ]
               │
             5 │     field x (offset = 0, type = i64);
               │     ────────────────┬────────────────  
               │                     ╰────────────────── conflicts with the declaration here.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn convert_panic_to_exception_in_imported_file_fails() {
    let resolver = MockFilesystem::new(vec![(
        "./imported.zng",
        r#"
        #convert_panic_to_exception
        type A {
            #layout(size = 1, align = 1);
        }
        "#,
    )]);

    check_import_fail(
        r#"
merge "./imported.zng";
type B {
    #layout(size = 1, align = 1);
}
        "#,
        expect![[r#"
            Error: Using `#convert_panic_to_exception` in imported zngur files is not supported. This directive can only be used in the main zngur file.
               ╭─[ ./imported.zng:2:10 ]
               │
             2 │         #convert_panic_to_exception
               │          ─────────────┬────────────  
               │                       ╰────────────── Using `#convert_panic_to_exception` in imported zngur files is not supported. This directive can only be used in the main zngur file.
            ───╯
        "#]],
        &resolver,
    );
}

#[test]
fn convert_panic_to_exception_in_main_file_succeeds() {
    check_success(
        r#"
#convert_panic_to_exception
type A {
    #layout(size = 1, align = 1);
}
        "#,
    );
}

// Tests for processed_files tracking (depfile support)

#[test]
fn processed_files_single_file() {
    let parsed = parse_str(
        r#"
type A {
    #layout(size = 1, align = 1);
}
        "#,
        NullCfg,
        |_| {},
    );
    // Should have exactly one file (test.zng)
    assert_eq!(parsed.processed_files.len(), 1);
    assert_eq!(
        parsed.processed_files[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        "test.zng"
    );
}

#[test]
fn processed_files_with_import() {
    let resolver = MockFilesystem::new(vec![(
        "./imported.zng",
        "type Imported { #layout(size = 1, align = 1); }",
    )]);

    let parsed = parse_str_with_resolver(
        r#"
merge "./imported.zng";
type Main {
    #layout(size = 1, align = 1);
}
        "#,
        NullCfg,
        &resolver,
        |_| {},
    );
    // Should have two files: main (test.zng) + imported
    assert_eq!(parsed.processed_files.len(), 2);
    let file_names: Vec<_> = parsed
        .processed_files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(file_names.contains(&"test.zng"));
    assert!(file_names.contains(&"imported.zng"));
}

#[test]
fn processed_files_with_nested_imports() {
    let resolver = MockFilesystem::new(vec![
        (
            "./a.zng",
            r#"merge "./b.zng"; type A { #layout(size = 1, align = 1); }"#,
        ),
        (
            "./b.zng",
            r#"merge "./c.zng"; type B { #layout(size = 1, align = 1); }"#,
        ),
        ("./c.zng", "type C { #layout(size = 1, align = 1); }"),
    ]);

    let parsed = parse_str_with_resolver(
        r#"
merge "./a.zng";
type Main {
    #layout(size = 1, align = 1);
}
        "#,
        NullCfg,
        &resolver,
        |_| {},
    );
    // Should have four files: main + a + b + c
    assert_eq!(parsed.processed_files.len(), 4);
    let file_names: Vec<_> = parsed
        .processed_files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(file_names.contains(&"test.zng"));
    assert!(file_names.contains(&"a.zng"));
    assert!(file_names.contains(&"b.zng"));
    assert!(file_names.contains(&"c.zng"));
}

fn assert_layout(wanted_size: usize, wanted_align: usize, layout: &Option<LayoutPolicy>) {
    if !matches!(layout, Some(LayoutPolicy::StackAllocated { size, align }) if *size == wanted_size && *align == wanted_align)
    {
        panic!(
            "no match: StackAllocated {{ size: {wanted_size}, align: {wanted_align} }} != {:?} ",
            layout
        );
    };
}

static EMPTY_CFG: [(&str, &[&str]); 0] = [];

#[test]
fn test_if_conditional_type_item() {
    let source = r#"
#unstable(cfg_if)

type ::std::string::String {
    #if cfg!(target_pointer_width = "64") {
        #layout(size = 24, align = 8);
    } #else if cfg!(target_pointer_width = "32") {
        #layout(size = 12, align = 4);
    } #else {
        // silly size for testing
        #layout(size = 27, align = 9);
    }
}
    "#;
    let parsed = catch_parse_fail(
        source,
        InMemoryRustCfgProvider::default().with_values([("target_pointer_width", &["64"])]),
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_layout(24, 8, &ty.layout);
    let parsed = catch_parse_fail(
        source,
        InMemoryRustCfgProvider::default().with_values([("target_pointer_width", &["32"])]),
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_layout(12, 4, &ty.layout);
    let parsed = catch_parse_fail(source, NullCfg);
    let ty = parsed.spec.types.first().expect("no type parsed");

    assert_layout(27, 9, &ty.layout);
}

#[test]
fn test_match_conditional_type_item() {
    let source = r#"
#unstable(cfg_match)

type ::std::string::String {
    #match cfg!(target_pointer_width) {
        // single item arm
        "64" => #layout(size = 24, align = 8);
        // match usize numbers
        32 => {
            #layout(size = 12, align = 4);
        },

        _ => {
            // silly size for testing
            #layout(size = 27, align = 9);
        }
    }
}
    "#;
    let parsed = catch_parse_fail(
        source,
        InMemoryRustCfgProvider::default().with_values([("target_pointer_width", &["64"])]),
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_layout(24, 8, &ty.layout);
    let parsed = catch_parse_fail(
        source,
        InMemoryRustCfgProvider::default().with_values([("target_pointer_width", &["32"])]),
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_layout(12, 4, &ty.layout);
    let parsed = catch_parse_fail(source, NullCfg);
    let ty = parsed.spec.types.first().expect("no type parsed");

    assert_layout(27, 9, &ty.layout);
}

macro_rules! test_paths_with_cfg {
    ($src:expr, $features_path_pairs:expr) => {
        for (cfg, path) in $features_path_pairs.iter().map(|(cfg, path)| {
            (
                cfg.into_iter().copied().collect::<Vec<_>>(),
                path.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            )
        }) {
            let parsed =
                catch_parse_fail($src, InMemoryRustCfgProvider::default().with_values(cfg));
            let ty = parsed.spec.types.first().expect("no type parsed");
            assert_ty_path!(path, &ty.ty);
        }
    };
}

type CfgPathPairs<'a> = &'a [(&'a [(&'a str, &'a [&'a str])], &'a [&'a str])];

#[test]
fn conditional_if_spec_item() {
    let source = r#"
#unstable(cfg_if)

#if cfg!(feature = "foo") {
    type crate::Foo {
        #layout(size = 1, align = 1);
    }
} #else {
    type crate::Bar {
        #layout(size = 1, align = 1);
    }
}
    "#;

    let pairs: CfgPathPairs = &[
        (&[("feature", &["foo"])], &["crate", "Foo"]),
        (&EMPTY_CFG, &["crate", "Bar"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn conditional_match_spec_item() {
    let source = r#"
#unstable(cfg_match)

#match cfg!(feature) {
    "foo" => type crate::Foo {
        #layout(size = 1, align = 1);
    }
    _ => {
        type crate::Bar {
            #layout(size = 1, align = 1);
        }
    }
}
    "#;
    let pairs: CfgPathPairs = &[
        (&[("feature", &["foo"])], &["crate", "Foo"]),
        (&EMPTY_CFG, &["crate", "Bar"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn match_pattern_single_cfg() {
    let source = r#"
#unstable(cfg_match)

#match cfg!(feature) {
    "bar" | "zigza" => type crate::BarZigZa {
        #layout(size = 1, align = 1);
    }
    // match two values from a cfg value as a set
    "foo" & "baz" => type crate::FooBaz {
        #layout(size = 1, align = 1);
    }
    // negative matching (no feature baz)
    "foo" & !"baz" => type crate::FooNoBaz {
        #layout(size = 1, align = 1);
    }
    _ => {
        type crate::Zoop {
            #layout(size = 1, align = 1);
        }
    }
}
    "#;
    let pairs: CfgPathPairs = &[
        (&EMPTY_CFG, &["crate", "Zoop"]),
        (&[("feature", &["foo"])], &["crate", "FooNoBaz"]),
        (&[("feature", &["bar"])], &["crate", "BarZigZa"]),
        (&[("feature", &["zigza"])], &["crate", "BarZigZa"]),
        (&[("feature", &["foo", "baz"])], &["crate", "FooBaz"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn if_pattern_multi_cfg() {
    let source = r#"
#unstable(cfg_if)
// match two cfg keys as a set
#if cfg!(feature.foo) && cfg!(target_pointer_width = 32) {
    type crate::Foo32 {
        #layout(size = 1, align = 1);
    }
} #else if cfg!(feature.foo = None) && cfg!(target_pointer_width = 64) {
    type crate::NoFoo64 {
        #layout(size = 1, align = 1);
    }
} #else if (cfg!(feature.foo = Some) && cfg!(target_pointer_width = 64)) || cfg!(feature.baz) {
    type crate::Foo64_OrBaz {
        #layout(size = 1, align = 1);
    }
} #else {
    type crate::SpecialFoo {
        #layout(size = 1, align = 1);
    }
}
    "#;
    let pairs: CfgPathPairs = &[
        (
            &[("target_pointer_width", &["32"]), ("feature", &["foo"])],
            &["crate", "Foo32"],
        ),
        (
            &[("target_pointer_width", &["64"]), ("feature", &["bar"])],
            &["crate", "NoFoo64"],
        ),
        (
            &[("target_pointer_width", &["64"]), ("feature", &["foo"])],
            &["crate", "Foo64_OrBaz"],
        ),
        (
            &[("target_pointer_width", &["32"]), ("feature", &["baz"])],
            &["crate", "Foo64_OrBaz"],
        ),
        (&[("feature", &["foo"])], &["crate", "SpecialFoo"]),
        (&EMPTY_CFG, &["crate", "SpecialFoo"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn match_pattern_multi_cfg() {
    let source = r#"
#unstable(cfg_match)

#match (cfg!(feature.foo), cfg!(target_pointer_width)) {
    // match two cfg keys as a set
    (Some, "32") => type crate::Foo32 {
        #layout(size = 1, align = 1);
    }
    (None, 64) => type crate::NoFoo64 {
        #layout(size = 1, align = 1);
    }
    _ => {
        type crate::SpecialFoo {
            #layout(size = 1, align = 1);
        }
    }
}
    "#;
    let pairs: CfgPathPairs = &[
        (
            &[("target_pointer_width", &["32"]), ("feature", &["foo"])],
            &["crate", "Foo32"],
        ),
        (
            &[("target_pointer_width", &["64"]), ("feature", &["bar"])],
            &["crate", "NoFoo64"],
        ),
        (&[("feature", &["foo"])], &["crate", "SpecialFoo"]),
        (&EMPTY_CFG, &["crate", "SpecialFoo"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn match_pattern_multi_cfg_bad_pattern() {
    let source = r#"
#unstable(cfg_match)

#match (cfg!(feature.foo), cfg!(target_pointer_width)) {
    (Some, "32") => type crate::Foo32 {
        // would succeed if cfg match attempted
        #layout(size = 1, align = 1);
    }
    "64" => type crate::NoFoo64 {
        // will fail: cardinality of pattern and tuple don't match
        #layout(size = 1, align = 1);
    }
    _ => {
        type crate::SpecialFoo {
            #layout(size = 1, align = 1);
        }
    }
}
    "#;
    check_fail_with_cfg(
        source,
        InMemoryRustCfgProvider::default().with_values([("target_pointer_width", &["64"])]),
        expect![[r#"
            Error: Can not match single pattern against multiple cfg values.
               ╭─[ test.zng:9:5 ]
               │
             9 │     "64" => type crate::NoFoo64 {
               │     ──┬─  
               │       ╰─── Can not match single pattern against multiple cfg values.
            ───╯
        "#]],
    );
}

#[test]
fn match_pattern_multi_cfg_bad_pattern2() {
    let source = r#"
#unstable(cfg_match)

#match (cfg!(feature.foo), cfg!(target_pointer_width), cfg!(target_feature) ) {
    (Some, "32", "avx" & "avx2") => type crate::Foo32 {
        // would succeed if cfg match attempted
        #layout(size = 1, align = 1);
    }
    (None, "64") => type crate::NoFoo64 {
        // will fail: cardinality of pattern and tuple don't match
        #layout(size = 1, align = 1);
    }
    _ => {
        type crate::SpecialFoo {
            #layout(size = 1, align = 1);
        }
    }
}
    "#;
    let cfg: [(&str, &[&str]); 2] = [
        ("target_pointer_width", &["64"]),
        ("target_feature", &["avx", "avx2"]),
    ];
    check_fail_with_cfg(
        source,
        InMemoryRustCfgProvider::default().with_values(cfg),
        expect![[r#"
            Error: Number of patterns and number of scrutinees do not match.
               ╭─[ test.zng:9:5 ]
               │
             9 │     (None, "64") => type crate::NoFoo64 {
               │     ──────┬─────  
               │           ╰─────── Number of patterns and number of scrutinees do not match.
            ───╯
        "#]],
    );
}

#[test]
fn cfg_match_unstable() {
    let source = r#"
#match cfg!(feature) {
    "foo" => type crate::Foo {
        #layout(size = 1, align = 1);
    }
    _ => {
        type crate::Bar {
            #layout(size = 1, align = 1);
        }
    }
}
    "#;
    check_fail_with_cfg(
        source,
        InMemoryRustCfgProvider::default().with_values([("feature", &["foo"])]),
        expect![[r#"
            Error: `#match` statements are unstable. Enable them by using `#unstable(cfg_match)` at the top of the file.
                ╭─[ test.zng:2:1 ]
                │
              2 │ ╭─▶ #match cfg!(feature) {
                ┆ ┆   
             11 │ ├─▶ }
                │ │       
                │ ╰─────── `#match` statements are unstable. Enable them by using `#unstable(cfg_match)` at the top of the file.
            ────╯
        "#]],
    );
}

#[test]
fn module_import_parser_test() {
    let parsed = parse_str(
        r#"
import "module.zng";
"#,
        NullCfg,
        |_| {},
    );
    assert_eq!(parsed.spec.imported_modules.len(), 1);
    assert_eq!(
        parsed.spec.imported_modules[0].path.to_str().unwrap(),
        "module.zng"
    );
}

#[test]
fn extern_cpp_requires_safety() {
    let source = r#"
extern "C++" {
    fn foo();
}
    "#;
    check_fail(
        source,
        expect![[r#"
            Error: found 'fn' expected 'safe', 'unsafe', 'impl', or '}'
               ╭─[ test.zng:3:5 ]
               │
             3 │     fn foo();
               │     ─┬  
               │      ╰── found 'fn' expected 'safe', 'unsafe', 'impl', or '}'
            ───╯
        "#]],
    );
    let source = r#"
extern "C++" {
    impl crate::Foo {
        fn foo();
    }
}
    "#;
    check_fail(
        source,
        expect![[r#"
            Error: found 'fn' expected 'safe', 'unsafe', or '}'
               ╭─[ test.zng:4:9 ]
               │
             4 │         fn foo();
               │         ─┬  
               │          ╰── found 'fn' expected 'safe', 'unsafe', or '}'
            ───╯
        "#]],
    );
}

#[test]
fn cpp_additional_includes() {
    let parsed = parse_str(
        r#"
#cpp_additional_includes "
    // comment
    stuff
"
    "#,
        NullCfg,
        |_| {},
    );
    assert_eq!(
        parsed.spec.additional_includes.0,
        "\n    // comment\n    stuff\n"
    );

    let parsed = parse_str(
        r##"
#cpp_additional_includes r#"
    // comment
    "stuff"
"#
    "##,
        NullCfg,
        |_| {},
    );
    assert_eq!(
        parsed.spec.additional_includes.0,
        "\n    // comment\n    \"stuff\"\n"
    );
}

// Tests for `c++::a::b::Name` paths.

fn assert_cpp(expected: &[&str], ty: &RustType) {
    let RustType::Cpp(pg) = ty else {
        panic!("type `{:?}` is not a c++ type", ty);
    };
    assert!(pg.generics.is_empty(), "expected no generics in {ty:?}");
    let got: Vec<&str> = pg.path.iter().map(String::as_str).collect();
    assert_eq!(got.as_slice(), expected);
}

#[test]
fn mod_cpp_shorthand_matches_direct_cpp_path() {
    let direct = parse_str(
        r#"
type c++::a::b::Name {
    #heap_allocated;
}
    "#,
        NullCfg,
        |_| {},
    );
    let via_mod = parse_str(
        r#"
mod c++::a::b {
    type Name {
        #heap_allocated;
    }
}
    "#,
        NullCfg,
        |_| {},
    );
    let direct_ty = &direct.spec.types.first().expect("no type parsed").ty;
    let via_mod_ty = &via_mod.spec.types.first().expect("no type parsed").ty;
    assert_cpp(&["a", "b", "Name"], direct_ty);
    assert_eq!(direct_ty, via_mod_ty);
}

#[test]
fn alias_can_target_a_cpp_path() {
    // Unlike a method's `use` path or a trait bound, a `use ... as` alias
    // target can legitimately be a `c++::` path -- referencing the alias
    // elsewhere should resolve to exactly that `c++::` path.
    let parsed = parse_str(
        r#"
use c++::a::Foo as MyFoo;

type MyFoo {
    #heap_allocated;
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["a", "Foo"], &ty.ty);
}

#[test]
fn cpp_template_args_are_preserved() {
    let parsed = parse_str(
        r#"
type c++::std::unique_ptr<c++::Foo> {
    #layout(size = 8, align = 8);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    let RustType::Cpp(pg) = &ty.ty else {
        panic!("type `{:?}` is not a c++ type", ty.ty);
    };
    assert_eq!(pg.path, vec!["std".to_owned(), "unique_ptr".to_owned()]);
    assert_eq!(pg.generics.len(), 1);
    let RustType::Cpp(arg) = &pg.generics[0] else {
        panic!("arg `{:?}` is not a c++ type", pg.generics[0]);
    };
    assert_eq!(arg.path, vec!["Foo".to_owned()]);
}

#[test]
fn cpp_path_rejected_in_method_use_path() {
    check_fail(
        r#"
type crate::Foo {
    #layout(size = 1, align = 1);
    fn bar(self) -> usize use c++::a::Bar;
}
    "#,
        expect![[r#"
            Error: `c++::` paths cannot be used in a method's `use` path
               ╭─[ test.zng:4:42 ]
               │
             4 │     fn bar(self) -> usize use c++::a::Bar;
               │                                          ┬  
               │                                          ╰── `c++::` paths cannot be used in a method's `use` path
            ───╯
        "#]],
    );
}

#[test]
fn cpp_path_rejected_as_trait() {
    check_fail(
        r#"
extern "C++" {
    impl c++::Foo for crate::X {
    }
}
    "#,
        expect![[r#"
            Error: `c++::` paths cannot be used as a trait
               ╭─[ test.zng:3:19 ]
               │
             3 │     impl c++::Foo for crate::X {
               │                   ─┬─  
               │                    ╰─── `c++::` paths cannot be used as a trait
            ───╯
        "#]],
    );
}

#[test]
#[should_panic(expected = "a c++::-only path was used somewhere that can't support it")]
fn cpp_path_via_alias_indirection_as_trait_is_a_known_ice() {
    // The syntactic check above only catches an *explicit* `c++::` prefix in
    // trait position. An alias that itself targets a `c++::` path slips
    // past it (nothing about `MyTrait` looks like a `c++::` path until it's
    // resolved), and hits a deliberate `todo!()` instead of a clean
    // diagnostic -- see the `RustPathAndGenerics::to_zngur` `Cpp` arm.
    let _ = parse_str(
        r#"
use c++::Foo as MyTrait;

extern "C++" {
    impl MyTrait for crate::X {
    }
}
    "#,
        NullCfg,
        |_| {},
    );
}

#[test]
fn cpp_path_allowed_as_impl_target() {
    check_success(
        r#"
extern "C++" {
    impl c++::Foo {
    }
}
    "#,
    );
}

#[test]
fn cpp_path_allowed_as_impl_for_trait_target() {
    check_success(
        r#"
extern "C++" {
    impl crate::SomeTrait for c++::Foo {
    }
}
    "#,
    );
}

#[test]
fn cpp_path_lexer_tolerates_whitespace_between_tokens() {
    // `c++` is lexed as a single, indivisible token (like `->` or `::`), so
    // whitespace *inside* it (`c ++`) does not lex as `Token::CppPathStart` --
    // it falls back to `Ident("c")` + `Plus` + `Plus`, same as any other
    // unrecognized punctuation sequence would. But, like every other token in
    // this grammar (e.g. `crate ::`), ordinary whitespace *between* the
    // `c++` token and the following `::`/segments is fine.
    let normal = parse_str(
        r#"
type c++::Name {
    #heap_allocated;
}
    "#,
        NullCfg,
        |_| {},
    );
    let spaced = parse_str(
        r#"
type c++ :: Name {
    #heap_allocated;
}
    "#,
        NullCfg,
        |_| {},
    );
    let normal_ty = &normal.spec.types.first().expect("no type parsed").ty;
    let spaced_ty = &spaced.spec.types.first().expect("no type parsed").ty;
    assert_cpp(&["Name"], normal_ty);
    assert_eq!(normal_ty, spaced_ty);
}

#[test]
fn cpp_path_with_space_inside_token_is_a_clean_syntax_error_not_a_panic() {
    // `c ++ :: Name` (space between `c` and `++`) does NOT lex as the `c++`
    // path-start token -- it lexes as `Ident("c")` followed by `Plus`, `Plus`,
    // which is a plain syntax error (not a panic).
    check_fail(
        r#"
type c ++ :: Name {
    #layout(size = 16, align = 8);
}
    "#,
        expect![[r#"
            Error: found '+' expected '::', '<', or '{'
               ╭─[ test.zng:2:8 ]
               │
             2 │ type c ++ :: Name {
               │        ┬  
               │        ╰── found '+' expected '::', '<', or '{'
            ───╯
        "#]],
    );
}

#[test]
fn ordinary_path_starting_with_c_is_unaffected() {
    let parsed = parse_str(
        r#"
type crate::config::Foo {
    #layout(size = 1, align = 1);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_ty_path!(["crate", "config", "Foo"], &ty.ty);
}

#[test]
fn cpp_mod_rejected_when_nested_inside_another_module() {
    check_fail(
        r#"
mod crate::foo {
    mod c++::a::b {
        type Name {
            #layout(size = 16, align = 8);
        }
    }
}
    "#,
        expect![[r#"
            Error: `c++::` modules can only appear at the top level of a file, not nested inside another module
               ╭─[ test.zng:3:9 ]
               │
             3 │     mod c++::a::b {
               │         ────┬────  
               │             ╰────── `c++::` modules can only appear at the top level of a file, not nested inside another module
            ───╯
        "#]],
    );
}

#[test]
fn free_fn_rejected_inside_cpp_scope() {
    // There's no real Rust function living in the generated c++::-only
    // module tree, so a top-level `fn` declaration inside `mod c++::a {
    // ... }` must be rejected rather than silently misinterpreting the
    // c++::-only path as if it were a real, absolute Rust path.
    check_fail(
        r#"
mod c++::a {
    fn foo(i32) -> bool;
}
    "#,
        expect![[r#"
            Error: a free function cannot be declared inside a c++:: scope
               ╭─[ test.zng:3:5 ]
               │
             3 │     fn foo(i32) -> bool;
               │     ─────────┬─────────  
               │              ╰─────────── a free function cannot be declared inside a c++:: scope
            ───╯
        "#]],
    );
}

#[test]
fn crate_mod_rejected_when_nested_inside_another_module() {
    check_fail(
        r#"
mod crate::foo {
    mod crate::bar {
        type Name {
            #layout(size = 16, align = 8);
        }
    }
}
    "#,
        expect![[r#"
            Error: `crate::` modules can only appear at the top level of a file, not nested inside another module
               ╭─[ test.zng:3:9 ]
               │
             3 │     mod crate::bar {
               │         ─────┬────  
               │              ╰────── `crate::` modules can only appear at the top level of a file, not nested inside another module
            ───╯
        "#]],
    );
}

#[test]
fn absolute_mod_rejected_when_nested_inside_another_module() {
    check_fail(
        r#"
mod crate::foo {
    mod ::std::bar {
        type Name {
            #layout(size = 16, align = 8);
        }
    }
}
    "#,
        expect![[r#"
            Error: `::` modules can only appear at the top level of a file, not nested inside another module
               ╭─[ test.zng:3:9 ]
               │
             3 │     mod ::std::bar {
               │         ─────┬────  
               │              ╰────── `::` modules can only appear at the top level of a file, not nested inside another module
            ───╯
        "#]],
    );
}

#[test]
fn relative_mod_nested_in_cpp_mod_extends_the_cpp_prefix() {
    // `mod c++::a { mod b { type Name { ... } } }` is `c++::a::b::Name` --
    // a plain relative `mod` nested inside a `c++::` scope composes onto the
    // same `c++::` prefix rather than starting a fresh Rust module path.
    let parsed = parse_str(
        r#"
mod c++::a {
    mod b {
        type Name {
            #heap_allocated;
        }
    }
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["a", "b", "Name"], &ty.ty);
}

#[test]
fn aliased_type_referenced_inside_cpp_mod_still_resolves_via_the_alias() {
    // A bare relative name that matches an alias must still expand via that
    // alias, even when referenced from inside a `c++::` scope -- it must
    // NOT get the `c++::` prefix composed onto it instead.
    let parsed = parse_str(
        r#"
use ::std::string::String as MyString;

mod c++::a {
    type Name {
        #heap_allocated;
        field s (offset = auto, type = MyString);
    }
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    // The type's own declaration still composes onto the c++:: prefix:
    assert_cpp(&["a", "Name"], &ty.ty);
    // But the aliased field type resolves as the real Rust path, not
    // `Cpp(["a", "s"])`:
    let field = ty.fields.first().expect("no field parsed");
    assert_ty_path!(["std", "string", "String"], &field.ty);
}

#[test]
fn scope_reference_to_cpp_target_from_top_level_cpp_base_is_bare() {
    // At the root of the c++::-only tree (an empty `Cpp` base), there's
    // nothing to climb out of, so the reference is just the target's own
    // segments.
    let scope = scope_with_base(crate::EntityPath::Cpp(Vec::new()));
    assert_eq!(
        scope.reference_to(&crate::EntityPath::cpp(["foo", "Bar"])),
        Some("foo::Bar".to_owned()),
    );
}

fn scope_with_base(base: crate::EntityPath) -> crate::Scope<'static> {
    crate::Scope {
        aliases: Vec::new(),
        base,
        type_vars: Default::default(),
    }
}

#[test]
fn scope_reference_to_rust_target_is_always_reachable_regardless_of_base() {
    // A `Rust` target is crate- or globally-qualified, so it's reachable the
    // same way no matter what the referencing scope's own base is -- even
    // from a `Cpp` base.
    let scope = scope_with_base(crate::EntityPath::cpp(["a", "b"]));
    assert_eq!(
        scope.reference_to(&crate::EntityPath::crate_relative(["foo", "Bar"])),
        Some("crate::foo::Bar".to_owned()),
    );
    assert_eq!(
        scope.reference_to(&crate::EntityPath::rust(["foo", "Bar"])),
        Some("::foo::Bar".to_owned()),
    );
}

#[test]
fn scope_reference_to_cpp_target_from_rust_base_is_impossible() {
    // We don't know which module the c++::-only tree will itself be
    // generated into relative to an arbitrary Rust path, so there's no way
    // to reference it from a `Rust` base.
    let scope = scope_with_base(crate::EntityPath::Rust(Vec::new()));
    assert_eq!(
        scope.reference_to(&crate::EntityPath::cpp(["a", "Foo"])),
        None
    );
}

#[test]
fn scope_reference_to_cpp_target_from_cpp_base_uses_super_as_needed() {
    let scope = scope_with_base(crate::EntityPath::cpp(["a", "b"]));
    // Different branch entirely: climb out twice, then descend.
    assert_eq!(
        scope.reference_to(&crate::EntityPath::cpp(["x", "y"])),
        Some("super::super::x::y".to_owned()),
    );
    // Sibling module under the shared parent `a`: climb out once.
    assert_eq!(
        scope.reference_to(&crate::EntityPath::cpp(["a", "c"])),
        Some("super::c".to_owned()),
    );
    // Child of the current base: no climbing needed at all.
    assert_eq!(
        scope.reference_to(&crate::EntityPath::cpp(["a", "b", "Name"])),
        Some("Name".to_owned()),
    );
}

#[test]
fn cpp_type_with_only_by_ref_layout() {
    let parsed = parse_str(
        r#"
type c++::osmium::Way {
    #only_by_ref;
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["osmium", "Way"], &ty.ty);
    assert_eq!(ty.layout, Some(LayoutPolicy::OnlyByRef));
}

#[test]
fn cpp_type_with_stack_layout() {
    let parsed = parse_str(
        r#"
type c++::osmium::Way {
    #layout(size = 16, align = 8);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["osmium", "Way"], &ty.ty);
    assert_eq!(
        ty.layout,
        Some(LayoutPolicy::StackAllocated { size: 16, align: 8 })
    );
}

#[test]
fn cpp_type_with_conservative_layout_derives_path() {
    let parsed = parse_str(
        r#"
type c++::osmium::Way {
    #layout_conservative(size = 32, align = 16);
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["osmium", "Way"], &ty.ty);
    assert_eq!(
        ty.layout,
        Some(LayoutPolicy::Conservative {
            size: 32,
            align: 16
        })
    );
}

#[test]
fn cpp_type_with_heap_layout() {
    let parsed = parse_str(
        r#"
type c++::osmium::Way {
    #heap_allocated;
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["osmium", "Way"], &ty.ty);
    assert_eq!(ty.layout, Some(LayoutPolicy::HeapAllocated));
}

#[test]
fn normal_layout_on_rust_type_passes_through() {
    let parsed = parse_str(
        r#"
type crate::Foo {
    #only_by_ref;
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_eq!(ty.layout, Some(LayoutPolicy::OnlyByRef));
}

#[test]
fn mod_cpp_shorthand_with_normal_layout() {
    let parsed = parse_str(
        r#"
mod c++::osmium {
    type Way {
        #only_by_ref;
    }
}
    "#,
        NullCfg,
        |_| {},
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    assert_cpp(&["osmium", "Way"], &ty.ty);
    assert_eq!(ty.layout, Some(LayoutPolicy::OnlyByRef));
}
