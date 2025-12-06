use std::panic::catch_unwind;

use expect_test::{Expect, expect};
use zngur_def::{LayoutPolicy, RustPathAndGenerics, RustType, ZngurSpec};

use crate::{
    ImportResolver, ParsedZngFile,
    cfg::{InMemoryRustCfgProvider, RustCfgProvider},
};

struct NullCfg;

impl RustCfgProvider for NullCfg {
    fn get_cfg(&self, _key: &str) -> Option<Vec<String>> {
        None
    }
    fn get_features(&self) -> Vec<String> {
        Vec::new()
    }
}

fn check_success(zng: &str) {
    let _ = ParsedZngFile::parse_str(zng, "test.zng".into(), &NullCfg);
}

pub struct ErrorText(pub String);

fn check_fail(zng: &str, error: Expect) {
    let r = catch_unwind(|| {
        let _ = ParsedZngFile::parse_str(zng, "test.zng".into(), &NullCfg);
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
    cfg: &(impl RustCfgProvider + std::panic::RefUnwindSafe),
    error: Expect,
) {
    let r = catch_unwind(|| {
        let _ = ParsedZngFile::parse_str(zng, "test.zng".into(), cfg);
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
        let _ = ParsedZngFile::parse_str_with_resolver(zng, "test.zng".into(), &NullCfg, resolver);
    });

    match r {
        Ok(_) => panic!("Parsing succeeded but we expected fail"),
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => error.assert_eq(&t.0),
            Err(e) => std::panic::resume_unwind(e),
        },
    }
}

// usefull for debugging a test that should succeeded on parse
fn catch_parse_fail(
    zng: &str,
    cfg: &(impl RustCfgProvider + std::panic::RefUnwindSafe),
) -> ZngurSpec {
    let r = catch_unwind(move || ParsedZngFile::parse_str(zng, "test.zng".into(), cfg));

    match r {
        Ok(spec) => spec,
        Err(e) => match e.downcast::<ErrorText>() {
            Ok(t) => {
                eprintln!("{}", &t.0);
                ZngurSpec::default()
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
            Error: found 'welcome_traits' expected '#', 'wellknown_traits', 'constructor', 'field', 'fn', or '}'
               ╭─[test.zng:4:5]
               │
             4 │     welcome_traits(Copy);
               │     ───────┬──────  
               │            ╰──────── found 'welcome_traits' expected '#', 'wellknown_traits', 'constructor', 'field', 'fn', or '}'
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
        "test.zng".into(),
        &NullCfg,
    );
    let ty = parsed.types.first().expect("no type parsed");
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
        "test.zng".into(),
        &NullCfg,
    );
    let ty = parsed.types.first().expect("no type parsed");
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
        "test.zng".into(),
        &NullCfg,
        &resolver,
    );
    assert_eq!(parsed.types.len(), 2);
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

fn assert_layout(wanted_size: usize, wanted_align: usize, layout: &LayoutPolicy) {
    if !matches!(layout, LayoutPolicy::StackAllocated { size, align } if *size == wanted_size && *align == wanted_align)
    {
        panic!(
            "no match: StackAllocated {{ size: {wanted_size}, align: {wanted_align} }} != {:?} ",
            layout
        );
    };
}

static EMPTY_FEATURES: [&str; 0] = [];
static EMPTY_CFG: [(&str, &[&str]); 0] = [];

#[test]
fn test_if_conditional_type_item() {
    let source = r#"
type ::std::string::String {
    #if #cfg(target_arch) = "64" {
        #layout(size = 24, align = 8);
    } #else if #cfg(target_arch) = "32" {
        #layout(size = 12, align = 4);
    } #else {
        // silly size for testing
        #layout(size = 27, align = 9);
    }
}
    "#;
    let parsed = catch_parse_fail(
        source,
        &InMemoryRustCfgProvider::new(&[("target_arch", &["64"])], &EMPTY_FEATURES),
    );
    let ty = parsed.types.first().expect("no type parsed");
    assert_layout(24, 8, &ty.layout);
    let parsed = catch_parse_fail(
        source,
        &InMemoryRustCfgProvider::new(&[("target_arch", &["32"])], &EMPTY_FEATURES),
    );
    let ty = parsed.types.first().expect("no type parsed");
    assert_layout(12, 4, &ty.layout);
    let parsed = catch_parse_fail(source, &NullCfg);
    let ty = parsed.types.first().expect("no type parsed");

    assert_layout(27, 9, &ty.layout);
}

#[test]
fn test_match_conditional_type_item() {
    // single item arm
    // optional comma seperator arm
    // empty pattern arm
    let source = r#"
type ::std::string::String {
    #match #cfg(target_arch) {
        "64" => #layout(size = 24, align = 8);
        "32" => {
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
        &InMemoryRustCfgProvider::new(&[("target_arch", &["64"])], &EMPTY_FEATURES),
    );
    let ty = parsed.types.first().expect("no type parsed");
    assert_layout(24, 8, &ty.layout);
    let parsed = catch_parse_fail(
        source,
        &InMemoryRustCfgProvider::new(&[("target_arch", &["32"])], &EMPTY_FEATURES),
    );
    let ty = parsed.types.first().expect("no type parsed");
    assert_layout(12, 4, &ty.layout);
    let parsed = catch_parse_fail(source, &NullCfg);
    let ty = parsed.types.first().expect("no type parsed");

    assert_layout(27, 9, &ty.layout);
}

macro_rules! test_paths_with_cfg {
    ($src:expr, $features_path_pairs:expr) => {
        for (cfg, features, path) in $features_path_pairs.iter().map(|(cfg, features, path)| {
            (
                cfg,
                features,
                path.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            )
        }) {
            let parsed = catch_parse_fail($src, &InMemoryRustCfgProvider::new(*cfg, *features));
            let ty = parsed.types.first().expect("no type parsed");
            assert_ty_path!(path, &ty.ty);
        }
    };
}

type CfgPathPairs<'a> = &'a [(&'a [(&'a str, &'a [&'a str])], &'a [&'a str], &'a [&'a str])];

#[test]
fn conditional_if_spec_item() {
    let source = r#"
#if #cfg(feature) = "foo" {
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
        (&EMPTY_CFG, &["foo"], &["crate", "Foo"]),
        (&EMPTY_CFG, &EMPTY_FEATURES, &["crate", "Bar"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn conditional_match_spec_item() {
    let source = r#"
#match #cfg(feature) {
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
        (&EMPTY_CFG, &["foo"], &["crate", "Foo"]),
        (&EMPTY_CFG, &EMPTY_FEATURES, &["crate", "Bar"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn match_pattern_single_cfg() {
    let source = r#"
#match #cfg(feature) {
    "bar" | "zigza" => type crate::BarZigZa {
        #layout(size = 1, align = 1);
    }
    // match two values from a cfg value as a set 
    // TODO:? (only works when matching a single cfg key)
    ("foo" , "baz") => type crate::FooBaz {
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
        (&EMPTY_CFG, &["foo"], &["crate", "Zoop"]),
        (&EMPTY_CFG, &["bar"], &["crate", "BarZigZa"]),
        (&EMPTY_CFG, &["zigza"], &["crate", "BarZigZa"]),
        (&EMPTY_CFG, &["foo", "baz"], &["crate", "FooBaz"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn match_pattern_multi_cfg() {
    let source = r#"
#match #cfg(feature.foo, target_arch) {
    // match two cfg keys as a set
    (Some, "32") => type crate::Foo32 {
        #layout(size = 1, align = 1);
    }
    (None, "64") => type crate::NoFoo64 {
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
        (&[("target_arch", &["32"])], &["foo"], &["crate", "Foo32"]),
        (&[("target_arch", &["64"])], &["bar"], &["crate", "NoFoo64"]),
        (&EMPTY_CFG, &["foo"], &["crate", "SpecialFoo"]),
        (&EMPTY_CFG, &EMPTY_FEATURES, &["crate", "SpecialFoo"]),
    ];
    test_paths_with_cfg!(source, pairs);
}

#[test]
fn match_pattern_multi_cfg_bad_pattern() {
    let source = r#"
#match #cfg(feature.foo, target_arch) {
    (Some, "32") => type crate::Foo32 { 
        // would succeed if cfg match attempted
        #layout(size = 1, align = 1);
    }
    "64" => type crate::NoFoo64 { 
        // will fail: cardinality of pattern and #cfg() don't match
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
        &InMemoryRustCfgProvider::new(&[("target_arch", &["64"])], &EMPTY_FEATURES),
        expect![[r#"
            Error: Can not match pattern against multiple cfg values.
               ╭─[test.zng:7:5]
               │
             7 │     "64" => type crate::NoFoo64 {
               │     ──┬─  
               │       ╰─── Can not match pattern against multiple cfg values.
            ───╯
        "#]],
    );
}
