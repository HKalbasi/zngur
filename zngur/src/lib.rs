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

        // Detect parent crate from zng file location
        let zng_file_abs = zng_file.canonicalize().unwrap_or(zng_file.clone());
        let zng_parent = zng_file_abs
            .parent()
            .expect("zng file has no parent directory");
        let crate_path = self.crate_path.unwrap_or_else(|| zng_parent.to_path_buf());

        // Get relative path from bridge project to zng file
        // The bridge project will be at output_dir, so we need "../<zng_filename>"
        let zng_filename = zng_file
            .file_name()
            .expect("zng file has no filename")
            .to_string_lossy()
            .to_string();

        // Compute path to zngur crate from the bridge project
        // For development (in the zngur workspace), we find the workspace root
        // For production use, we'd use the crates.io version
        let zngur_path = Self::find_zngur_path(&output_dir);

        // Create output directory structure
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        fs::create_dir_all(output_dir.join("src")).expect("Failed to create src directory");
        fs::create_dir_all(output_dir.join("cpp")).expect("Failed to create cpp directory");

        // Generate minimal src/lib.rs that includes the generated code
        let lib_rs = r#"mod generated;
pub use generated::*;
"#;
        File::create(output_dir.join("src/lib.rs"))
            .unwrap()
            .write_all(lib_rs.as_bytes())
            .unwrap();

        // Generate Cargo.toml
        let cargo_toml = Self::generate_cargo_toml(&crate_path, &zngur_path);
        File::create(output_dir.join("Cargo.toml"))
            .unwrap()
            .write_all(cargo_toml.as_bytes())
            .unwrap();

        // Generate build.rs that does layout extraction and code generation
        let build_rs = Self::generate_build_rs(
            &zng_filename,
            cpp_namespace.as_deref(),
            mangling_base.as_deref(),
        );
        File::create(output_dir.join("build.rs"))
            .unwrap()
            .write_all(build_rs.as_bytes())
            .unwrap();

        println!(
            "Generated standalone bridge project at: {}",
            output_dir.display()
        );
    }

    /// Find the path to the zngur crate from the bridge project directory.
    ///
    /// For development (in the zngur workspace), this finds the workspace root and returns
    /// a relative path. For production use, this would return a crates.io version string.
    fn find_zngur_path(output_dir: &Path) -> String {
        // Get absolute path of the output directory
        let output_abs = output_dir
            .canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().unwrap().join(output_dir));

        // Walk up from output_dir looking for a directory containing zngur/Cargo.toml
        let mut search_dir = output_abs.parent();
        let mut depth = 1; // Start at 1 since we're already one level up

        while let Some(dir) = search_dir {
            let potential_zngur = dir.join("zngur").join("Cargo.toml");
            if potential_zngur.exists() {
                // Found it! Return relative path
                let rel_prefix = "../".repeat(depth);
                return format!("{rel_prefix}zngur");
            }
            search_dir = dir.parent();
            depth += 1;

            // Don't search too far up
            if depth > 10 {
                break;
            }
        }

        // Fallback to crates.io version
        "0.7".to_string()
    }

    fn generate_cargo_toml(parent_crate_path: &Path, zngur_dep: &str) -> String {
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

        // Determine if zngur_dep is a path or a version
        let zngur_dep_str = if zngur_dep.starts_with("../") || zngur_dep.starts_with('/') {
            format!("{{ path = \"{zngur_dep}\" }}")
        } else {
            format!("\"{zngur_dep}\"")
        };

        format!(
            r#"[package]
name = "zngur-bridge"
version = "0.1.0"
edition = "2021"

# Opt out of parent workspace
[workspace]

[lib]
crate-type = ["staticlib"]

[dependencies]
{package_name} = {{ path = ".." }}

[build-dependencies]
zngur = {zngur_dep_str}
"#
        )
    }

    fn generate_build_rs(
        zng_filename: &str,
        cpp_namespace: Option<&str>,
        mangling_base: Option<&str>,
    ) -> String {
        let mut build_rs = format!(
            r#"fn main() {{
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Generate code (layout extraction happens inside)
    let zng = zngur::Zngur::from_zng_file("../{zng_filename}")
        .with_crate_path("..")
        .with_layout_cache_dir(&out_dir)
        .with_h_file(format!("{{manifest_dir}}/cpp/generated.h"))
        .with_cpp_file(format!("{{manifest_dir}}/cpp/generated.cpp"))
        .with_rs_file(format!("{{manifest_dir}}/src/generated.rs"))"#
        );

        if let Some(ns) = cpp_namespace {
            build_rs.push_str(&format!(
                r#"
        .with_cpp_namespace("{ns}")"#
            ));
        }

        if let Some(mb) = mangling_base {
            build_rs.push_str(&format!(
                r#"
        .with_mangling_base("{mb}")"#
            ));
        }

        build_rs.push_str(&format!(
            r#";

    zng.generate();

    // Tell Cargo to re-run if the .zng file or source files change
    println!("cargo::rerun-if-changed=../{zng_filename}");
    println!("cargo::rerun-if-changed=../src");
}}
"#
        ));

        build_rs
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
