//! This crate contains an API for using the Zngur code generator inside build scripts. For more information
//! about the Zngur itself, see [the documentation](https://hkalbasi.github.io/zngur).

use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use zngur_generator::{LayoutPolicy, ParsedZngFile, ZngurGenerator};

#[must_use]
/// Builder for the Zngur generator.
///
/// Usage:
/// ```ignore
/// let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
/// let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
/// Zngur::from_zng_file(crate_dir.join("main.zng"))
///     .with_cpp_file(out_dir.join("generated.cpp"))
///     .with_h_file(out_dir.join("generated.h"))
///     .with_rs_file(out_dir.join("generated.rs"))
///     .generate();
/// ```
pub struct Zngur {
    zng_file: PathBuf,
    h_file_path: Option<PathBuf>,
    cpp_file_path: Option<PathBuf>,
    rs_file_path: Option<PathBuf>,
    mangling_base: Option<String>,
    cpp_namespace: Option<String>,
    layout_cache_dir: Option<PathBuf>,
    crate_path: Option<PathBuf>,
    target: Option<String>,
    standalone_output_dir: Option<PathBuf>,
}

impl Zngur {
    pub fn from_zng_file(zng_file_path: impl AsRef<Path>) -> Self {
        Zngur {
            zng_file: zng_file_path.as_ref().to_owned(),
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
            mangling_base: None,
            cpp_namespace: None,
            layout_cache_dir: None,
            crate_path: None,
            target: None,
            standalone_output_dir: None,
        }
    }

    pub fn with_h_file(mut self, path: impl AsRef<Path>) -> Self {
        self.h_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_cpp_file(mut self, path: impl AsRef<Path>) -> Self {
        self.cpp_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_rs_file(mut self, path: impl AsRef<Path>) -> Self {
        self.rs_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_mangling_base(mut self, mangling_base: &str) -> Self {
        self.mangling_base = Some(mangling_base.to_owned());
        self
    }

    pub fn with_cpp_namespace(mut self, cpp_namespace: &str) -> Self {
        self.cpp_namespace = Some(cpp_namespace.to_owned());
        self
    }

    /// Set the directory for caching layout information (default: OUT_DIR).
    ///
    /// The cache is automatically invalidated when the rustc version, target,
    /// source files, or features change.
    pub fn with_layout_cache_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.layout_cache_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Set the crate path for layout extraction (default: CARGO_MANIFEST_DIR).
    ///
    /// This is the directory containing the Cargo.toml of the crate whose
    /// types are being bridged.
    pub fn with_crate_path(mut self, path: impl AsRef<Path>) -> Self {
        self.crate_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the target triple for cross-compilation (default: auto-detect).
    pub fn with_target(mut self, target: impl AsRef<str>) -> Self {
        self.target = Some(target.as_ref().to_string());
        self
    }

    /// Enable standalone mode, generating a complete bridge project.
    ///
    /// In standalone mode, Zngur generates a complete Cargo project with:
    /// - Cargo.toml (with dependency on parent crate)
    /// - build.rs (for compiling C++ code)
    /// - src/lib.rs (FFI + layout assertions)
    /// - cpp/generated.h and cpp/generated.cpp
    ///
    /// This eliminates circular dependencies and provides a clean separation
    /// between user code and generated FFI code.
    pub fn standalone_mode(mut self, output_dir: impl AsRef<Path>) -> Self {
        self.standalone_output_dir = Some(output_dir.as_ref().to_path_buf());
        self
    }

    fn generate_standalone(self, output_dir: PathBuf) {
        use std::fs;

        // Extract all the data we need from self at the beginning
        let zng_file = self.zng_file;
        let cpp_namespace = self.cpp_namespace;
        let mangling_base = self.mangling_base;
        let crate_path_opt = self.crate_path;
        let layout_cache_dir = self.layout_cache_dir;
        let target = self.target;

        // Create output directory structure
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        fs::create_dir_all(output_dir.join("src")).expect("Failed to create src directory");
        fs::create_dir_all(output_dir.join("cpp")).expect("Failed to create cpp directory");

        // Parse and generate code
        let mut file = ZngurGenerator::build_from_zng(ParsedZngFile::parse(zng_file.clone()));

        // Set up file paths within the bridge project
        let h_file_path = output_dir.join("cpp/generated.h");
        let cpp_file_path = output_dir.join("cpp/generated.cpp");
        let rs_file_path = output_dir.join("src/lib.rs");

        file.0.cpp_include_header_name = "generated.h".to_string();
        let cpp_ns = cpp_namespace.clone().unwrap_or_else(|| "rust".to_string());
        file.0.cpp_namespace = cpp_ns.clone();

        if let Some(ref cpp_namespace_val) = cpp_namespace {
            file.0.mangling_base = cpp_namespace_val.clone();
        }

        if let Some(mangling_base_val) = mangling_base {
            file.0.mangling_base = mangling_base_val;
        }

        // Resolve auto layouts (detect parent crate from zng file location)
        // Canonicalize the zng_file path to get an absolute path
        let zng_file_abs = zng_file.canonicalize().unwrap_or(zng_file.clone());
        let zng_parent = zng_file_abs
            .parent()
            .expect("zng file has no parent directory");
        let crate_path = crate_path_opt.unwrap_or_else(|| zng_parent.to_path_buf());

        let cache_dir = layout_cache_dir.or_else(|| env::var("OUT_DIR").ok().map(PathBuf::from));

        if let Err(e) =
            file.resolve_auto_layouts(&crate_path, cache_dir.as_deref(), target.as_deref())
        {
            eprintln!("{}", e);
            eprintln!();
            eprintln!(
                "note: you can avoid auto-layout by using explicit #layout(size = X, align = Y)"
            );
            eprintln!("      or #heap_allocate instead of #layout(auto) in your .zng file");
            panic!("auto layout resolution failed");
        }

        let (rust, mut h, mut cpp) = file.render();

        // Namespace replacement
        h = h
            .replace("rust::", &format!("{cpp_ns}::"))
            .replace("namespace rust", &format!("namespace {cpp_ns}"));
        cpp = cpp.map(|cpp_code| cpp_code.replace("rust::", &format!("{cpp_ns}::")));

        // Check if C++ code exists
        let has_cpp = cpp.is_some();

        // Write generated files
        File::create(&rs_file_path)
            .unwrap()
            .write_all(rust.as_bytes())
            .unwrap();
        File::create(&h_file_path)
            .unwrap()
            .write_all(h.as_bytes())
            .unwrap();
        if let Some(cpp_code) = cpp {
            File::create(&cpp_file_path)
                .unwrap()
                .write_all(cpp_code.as_bytes())
                .unwrap();
        }

        // Generate Cargo.toml
        let cargo_toml = Self::generate_cargo_toml(&crate_path, has_cpp);
        File::create(output_dir.join("Cargo.toml"))
            .unwrap()
            .write_all(cargo_toml.as_bytes())
            .unwrap();

        // Generate build.rs if C++ code exists
        if has_cpp {
            let build_rs = Self::generate_build_rs();
            File::create(output_dir.join("build.rs"))
                .unwrap()
                .write_all(build_rs.as_bytes())
                .unwrap();
        }

        println!(
            "Generated standalone bridge project at: {}",
            output_dir.display()
        );
    }

    fn generate_cargo_toml(parent_crate_path: &Path, has_cpp: bool) -> String {
        // Read parent Cargo.toml to get crate name
        let parent_cargo_toml_path = parent_crate_path.join("Cargo.toml");
        let parent_cargo_toml = std::fs::read_to_string(&parent_cargo_toml_path)
            .expect("Failed to read parent Cargo.toml");

        // Simple parsing to extract package name
        let package_name = parent_cargo_toml
            .lines()
            .find(|line| line.trim().starts_with("name"))
            .and_then(|line| line.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"').to_string())
            .expect("Failed to find package name in parent Cargo.toml");

        let mut cargo_toml = format!(
            r#"[package]
name = "zngur-bridge"
version = "0.1.0"
edition = "2021"

# Opt out of parent workspace
[workspace]

[lib]
crate-type = ["staticlib"]

[dependencies]
{} = {{ path = ".." }}
"#,
            package_name
        );

        if has_cpp {
            cargo_toml.push_str(
                r#"
[build-dependencies]
cc = "1.0"
"#,
            );
        }

        cargo_toml
    }

    fn generate_build_rs() -> String {
        r#"fn main() {
    cc::Build::new()
        .cpp(true)
        .file("cpp/generated.cpp")
        .include("cpp")
        .std("c++17")
        .compile("zngur_bridge_cpp");
}
"#
        .to_string()
    }

    pub fn generate(mut self) {
        // Check if we're in standalone mode
        if let Some(output_dir) = self.standalone_output_dir.take() {
            self.generate_standalone(output_dir);
            return;
        }

        let mut file = ZngurGenerator::build_from_zng(ParsedZngFile::parse(self.zng_file));

        let rs_file_path = self.rs_file_path.expect("No rs file path provided");
        let h_file_path = self.h_file_path.expect("No h file path provided");

        file.0.cpp_include_header_name = h_file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        file.0.cpp_namespace = "rust".to_owned();

        if let Some(cpp_namespace) = self.cpp_namespace {
            file.0.mangling_base = cpp_namespace.clone();
            file.0.cpp_namespace = cpp_namespace;
        }

        if let Some(mangling_base) = self.mangling_base {
            file.0.mangling_base = mangling_base;
        }

        // Resolve auto layouts (only if there are types with #layout(auto))
        let has_auto_layouts = file
            .0
            .types
            .iter()
            .any(|ty_def| matches!(ty_def.layout, LayoutPolicy::Auto));

        if has_auto_layouts {
            let crate_path = self.crate_path.unwrap_or_else(|| {
                env::var("CARGO_MANIFEST_DIR")
                    .map(PathBuf::from)
                    .expect("CARGO_MANIFEST_DIR not set and no crate_path provided")
            });

            let cache_dir = self
                .layout_cache_dir
                .or_else(|| env::var("OUT_DIR").ok().map(PathBuf::from));

            if let Err(e) =
                file.resolve_auto_layouts(&crate_path, cache_dir.as_deref(), self.target.as_deref())
            {
                eprintln!("{}", e);
                eprintln!();
                eprintln!(
                    "note: you can avoid auto-layout by using explicit #layout(size = X, align = Y)"
                );
                eprintln!("      or #heap_allocate instead of #layout(auto) in your .zng file");
                panic!("auto layout resolution failed");
            }
        }

        let cpp_namespace = file.0.cpp_namespace.clone();

        let (rust, mut h, mut cpp) = file.render();

        // TODO: Don't hard code namespace as "::rust" and remove this replace
        h = h
            .replace("rust::", &format!("{cpp_namespace}::"))
            .replace("namespace rust", &format!("namespace {cpp_namespace}"));
        cpp = cpp.map(|cpp| cpp.replace("rust::", &format!("{cpp_namespace}::")));

        File::create(rs_file_path)
            .unwrap()
            .write_all(rust.as_bytes())
            .unwrap();
        File::create(h_file_path)
            .unwrap()
            .write_all(h.as_bytes())
            .unwrap();
        if let Some(cpp) = cpp {
            let cpp_file_path = self.cpp_file_path.expect("No cpp file path provided");
            File::create(cpp_file_path)
                .unwrap()
                .write_all(cpp.as_bytes())
                .unwrap();
        }
    }
}
