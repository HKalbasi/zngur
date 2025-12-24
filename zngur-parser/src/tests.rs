use std::panic::catch_unwind;

use expect_test::{Expect, expect};
use zngur_def::{LayoutPolicy, RustPathAndGenerics, RustType, ZngurSpec};

use crate::{
    ImportResolver, ParsedZngFile,
    cfg::{InMemoryRustCfgProvider, NullCfg, RustCfgProvider},
};

fn check_success(zng: &str) {
    let _ = ParsedZngFile::parse_str(zng, NullCfg);
}

pub struct ErrorText(pub String);

fn check_fail(zng: &str, error: Expect) {
    let r = catch_unwind(|| {
        let _ = ParsedZngFile::parse_str(zng, NullCfg);
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
        let _ = ParsedZngFile::parse_str(zng, cfg);
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
        let _ = ParsedZngFile::parse_str_with_resolver(zng, NullCfg, resolver);
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
    let r = catch_unwind(move || ParsedZngFile::parse_str(zng, cfg));

    match r {
        Ok(r) => r,
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => {
                eprintln!("{}", &t.0);
                crate::ParseResult {
                    spec: ZngurSpec::default(),
                    processed_files: Vec::new(),
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
               ╭─[test.zng:2:6]
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
            Error: found 'welcome_traits' expected '#', 'wellknown_traits', 'constructor', 'field', 'async', 'fn', or '}'
               ╭─[test.zng:4:5]
               │
             4 │     welcome_traits(Copy);
               │     ───────┬──────  
               │            ╰──────── found 'welcome_traits' expected '#', 'wellknown_traits', 'constructor', 'field', 'async', 'fn', or '}'
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
               ╭─[test.zng:4:5]
               │
             4 │     #heap_allocated;
               │     ───────┬───────  
               │            ╰───────── Duplicate layout policy found
            ───╯
        "#]],
    );
}

#[test]
fn cpp_ref_should_not_need_layout_info() {
    check_fail(
        r#"
type crate::Way {
    #layout(size = 1, align = 2);

    #cpp_ref "::osmium::Way";
}
    "#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[test.zng:3:5]
               │
             3 │     #layout(size = 1, align = 2);
               │     ──────────────┬─────────────  
               │                   ╰─────────────── Duplicate layout policy found
            ───╯
        "#]],
    );
    check_success(
        r#"
type crate::Way {
    #cpp_ref "::osmium::Way";
}
    "#,
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
    let parsed = ParsedZngFile::parse_str(
        r#"
use ::std::string::String as MyString;
type MyString {
    #layout(size = 24, align = 8);
}
    "#,
        NullCfg,
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    let RustType::Adt(RustPathAndGenerics { path: p, .. }) = &ty.ty else {
        panic!("no match?");
    };
    assert_eq!(p.as_slice(), ["std", "string", "String"]);
}

#[test]
fn alias_expands_nearest_scope_first() {
    let parsed = ParsedZngFile::parse_str(
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
    );
    let ty = parsed.spec.types.first().expect("no type parsed");
    let RustType::Adt(RustPathAndGenerics { path: p, .. }) = &ty.ty else {
        panic!("no match?");
    };
    assert_eq!(p.as_slice(), ["crate", "MyLocalString"]);
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

    let parsed = ParsedZngFile::parse_str_with_resolver(
        r#"
import "./relative/path.zng";
type Example {
    #layout(size = 1, align = 1);
}
    "#,
        NullCfg,
        &resolver,
    );
    assert_eq!(parsed.spec.types.len(), 2);
}

#[test]
fn module_import_prohibited() {
    let resolver = MockFilesystem::new(vec![] as Vec<(&str, &str)>);

    check_import_fail(
        r#"
    import "foo/bar.zng";
    "#,
        expect![[r#"
            Error: Module import is not supported. Use a relative path instead.
               ╭─[test.zng:2:5]
               │
             2 │     import "foo/bar.zng";
               │     ──────────┬──────────  
               │               ╰──────────── Module import is not supported. Use a relative path instead.
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
    import "./a.zng";
    type A {
      #layout(size = 2, align = 2);
    }
"#,
        expect![[r#"
            Error: Duplicate layout policy found
               ╭─[a.zng:2:12]
               │
             2 │       type A {
               │            ┬  
               │            ╰── Duplicate layout policy found
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
    import "./a.zng";
    "#,
        expect![[r#"
            Error: Import path not found: ./a.zng
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
  import "./a.zng";
  type A {
    #layout(size = 1, align = 1);
    fn foo(i64) -> i64;
  }
  "#,
        expect![[r#"
            Error: Method mismatch
               ╭─[a.zng:1:6]
               │
             1 │ type A { #layout(size = 1, align = 1); fn foo(i32) -> i32; }
               │      ┬  
               │      ╰── Method mismatch
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
  import "./a.zng";
  type A {
    #layout(size = 1, align = 1);
    field x (offset = 0, type = i64);
  }
  "#,
        expect![[r#"
            Error: Field mismatch
               ╭─[a.zng:1:6]
               │
             1 │ type A {
               │      ┬  
               │      ╰── Field mismatch
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
import "./imported.zng";
type B {
    #layout(size = 1, align = 1);
}
        "#,
        expect![[r#"
            Error: Using `#convert_panic_to_exception` in imported zngur files is not supported. This directive can only be used in the main zngur file.
               ╭─[imported.zng:2:10]
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
    let parsed = ParsedZngFile::parse_str(
        r#"
type A {
    #layout(size = 1, align = 1);
}
        "#,
        NullCfg,
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

    let parsed = ParsedZngFile::parse_str_with_resolver(
        r#"
import "./imported.zng";
type Main {
    #layout(size = 1, align = 1);
}
        "#,
        NullCfg,
        &resolver,
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
            r#"import "./b.zng"; type A { #layout(size = 1, align = 1); }"#,
        ),
        (
            "./b.zng",
            r#"import "./c.zng"; type B { #layout(size = 1, align = 1); }"#,
        ),
        ("./c.zng", "type C { #layout(size = 1, align = 1); }"),
    ]);

    let parsed = ParsedZngFile::parse_str_with_resolver(
        r#"
import "./a.zng";
type Main {
    #layout(size = 1, align = 1);
}
        "#,
        NullCfg,
        &resolver,
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

fn assert_layout(wanted_size: usize, wanted_align: usize, layout: &LayoutPolicy) {
    if !matches!(layout, LayoutPolicy::StackAllocated { size, align } if *size == wanted_size && *align == wanted_align)
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
               ╭─[test.zng:9:5]
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
               ╭─[test.zng:9:5]
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
            Error: `#match` statements are unstable. Enable them by using `#unstable(cgf_match)` at the top of the file.
                ╭─[test.zng:2:1]
                │
              2 │ ╭─▶ #match cfg!(feature) {
                ┆ ┆   
             11 │ ├─▶ }
                │ │       
                │ ╰─────── `#match` statements are unstable. Enable them by using `#unstable(cgf_match)` at the top of the file.
            ────╯
        "#]],
    );
}
