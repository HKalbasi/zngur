//! Core layout extraction logic via rustc compilation.
//!
//! This module implements layout extraction by generating and compiling a
//! temporary Rust program that reports type sizes and alignments.

use crate::types::{Layout, LayoutError, LayoutResult};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use zngur_def::RustType;

/// Extracts layouts by compiling temporary programs.
///
/// # Invariants
///
/// - `crate_path` points to a valid Cargo project with compiled artifacts
/// - `rustc_version` matches the currently installed rustc
pub(crate) struct Extractor {
    crate_path: PathBuf,
    target: Option<String>,
    #[allow(dead_code)]
    rustc_version: String,
}

impl Extractor {
    /// Creates an extractor for the crate at `crate_path` targeting `target`.
    ///
    /// - Precondition: rustc and cargo are available on PATH.
    /// - Postcondition: Returns an extractor configured for the given crate
    ///   and target, or an error if rustc version cannot be determined or cargo is not available.
    pub(crate) fn new(crate_path: &Path, target: Option<&str>) -> LayoutResult<Self> {
        // Check if cargo is available
        if Command::new("cargo").arg("--version").output().is_err() {
            return Err(LayoutError::CargoNotFound);
        }

        Ok(Self {
            crate_path: crate_path.to_path_buf(),
            target: target.map(|s| s.to_string()),
            rustc_version: get_rustc_version()?,
        })
    }

    /// Extracts layouts for all `types`.
    ///
    /// Generates a temporary Rust program, compiles it against the target crate,
    /// executes it, and parses its JSON output to extract size/align pairs.
    ///
    /// - Precondition: The crate has been compiled and artifacts exist.
    /// - Precondition: All types are public and accessible.
    /// - Postcondition: Returns a map from types to their layouts, or an error
    ///   describing the first failure encountered.
    /// - Complexity: O(n) where n is the compilation and execution time of the
    ///   extraction program.
    pub(crate) fn extract(&self, types: &[RustType]) -> LayoutResult<HashMap<RustType, Layout>> {
        // Find the compiled crate artifacts
        let crate_lib = self.find_crate_lib()?;

        // Generate the extraction program
        let extraction_code = self.generate_extraction_code(types)?;

        // Create a temporary directory for compilation
        let temp_dir = env::temp_dir().join(format!("zngur-layout-{}", std::process::id()));
        fs::create_dir_all(&temp_dir)?;

        let temp_file = temp_dir.join("extract_layouts.rs");
        fs::write(&temp_file, extraction_code)?;

        // Compile and run the extraction program
        let output = self.compile_and_run(&temp_file, &crate_lib, &temp_dir)?;

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);

        // Parse the output
        self.parse_output(&output, types)
    }

    fn find_crate_lib(&self) -> LayoutResult<PathBuf> {
        // Try to find an existing compiled library
        if let Ok(lib_path) = self.try_find_crate_lib() {
            return Ok(lib_path);
        }

        // No library found - try to build it
        let crate_name = self.get_crate_name()?;

        eprintln!(
            "No compiled library found for '{}', building...",
            crate_name
        );

        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(&self.crate_path);

        // Add target if specified
        if let Some(target) = &self.target {
            cmd.arg("--target").arg(target);
        }

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LayoutError::CargoNotFound
            } else {
                LayoutError::IoError(e)
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LayoutError::CompilationFailed(format!(
                "Failed to build crate '{}':\n{}",
                crate_name, stderr
            )));
        }

        // Try to find the library again after building
        self.try_find_crate_lib().map_err(|_| {
            LayoutError::CrateNotFound(format!(
                "Built crate '{}' successfully, but could not find compiled library artifacts. \
                 This may indicate the crate has no library target (crate-type).",
                crate_name
            ))
        })
    }

    fn try_find_crate_lib(&self) -> LayoutResult<PathBuf> {
        // Get the crate name from Cargo.toml
        let crate_name = self.get_crate_name()?;
        let lib_name = crate_name.replace('-', "_");

        // Determine the target directory
        // Priority: CARGO_TARGET_DIR env var > workspace root > crate root
        let target_dirs = if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
            vec![PathBuf::from(target_dir)]
        } else {
            let mut dirs = vec![];

            // Check for workspace target (look for target/ in parent directories)
            let mut current = self.crate_path.as_path();
            while let Some(parent) = current.parent() {
                let potential_target = parent.join("target");
                if potential_target.exists() {
                    dirs.push(potential_target);
                }
                current = parent;
            }

            // Also check the crate's own target
            dirs.push(self.crate_path.join("target"));
            dirs
        };

        // Try both release and debug profiles in each target directory
        let profiles = vec!["release", "debug"];

        for target_dir in &target_dirs {
            for profile in &profiles {
                let lib_path = target_dir.join(profile);

                // Try to find the rlib or dylib
                let extensions = vec!["rlib", "so", "dylib", "dll", "a"];
                for ext in &extensions {
                    let candidate = lib_path.join(format!("lib{}.{}", lib_name, ext));
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        Err(LayoutError::CrateNotFound(format!(
            "Could not find compiled library for crate '{}' in {:?}.",
            crate_name, target_dirs
        )))
    }

    fn get_crate_name(&self) -> LayoutResult<String> {
        let cargo_toml = self.crate_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml)
            .map_err(|e| LayoutError::CrateNotFound(format!("Could not read Cargo.toml: {}", e)))?;

        // Simple parsing - look for name = "..." in [package] section
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") && line.contains('=') {
                if let Some(name) = line.split('=').nth(1) {
                    let name = name.trim().trim_matches('"').trim_matches('\'');
                    return Ok(name.to_string());
                }
            }
        }

        Err(LayoutError::CrateNotFound(
            "Could not find crate name in Cargo.toml".to_string(),
        ))
    }

    fn generate_extraction_code(&self, types: &[RustType]) -> LayoutResult<String> {
        let mut code = String::new();

        // Add necessary imports
        code.push_str("use std::mem::{size_of, align_of};\n\n");

        code.push_str("fn main() {\n");

        // Generate simple output format: TYPE|SIZE|ALIGN per line
        for ty in types {
            let type_str = rust_type_to_string(ty);
            code.push_str(&format!(
                "    println!(\"{{}}|{{}}|{{}}\", \"{}\", size_of::<{}>(), align_of::<{}>());\n",
                type_str, type_str, type_str
            ));
        }

        code.push_str("}\n");

        Ok(code)
    }

    fn compile_and_run(
        &self,
        source: &Path,
        crate_lib: &Path,
        temp_dir: &Path,
    ) -> LayoutResult<String> {
        let output_bin = temp_dir.join("extract_layouts");

        // Get the crate name for the --extern flag
        let crate_name = self.get_crate_name()?;
        let lib_name = crate_name.replace('-', "_");

        // Build rustc command
        let mut cmd = Command::new("rustc");
        cmd.arg(source)
            .arg("--crate-type")
            .arg("bin")
            .arg("--edition")
            .arg("2021")
            .arg("-o")
            .arg(&output_bin)
            .arg("--extern")
            .arg(format!("{}={}", lib_name, crate_lib.display()));

        // Add target if specified
        if let Some(target) = &self.target {
            cmd.arg("--target").arg(target);
        }

        // Compile
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LayoutError::CompilationFailed(format!(
                "Failed to compile layout extraction program:\n{}",
                stderr
            )));
        }

        // Run the compiled program
        let output = Command::new(&output_bin).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LayoutError::ExecutionFailed(format!(
                "Failed to execute layout extraction program:\n{}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_output(
        &self,
        output: &str,
        types: &[RustType],
    ) -> LayoutResult<HashMap<RustType, Layout>> {
        // Parse simple format: TYPE|SIZE|ALIGN per line
        let mut parsed: HashMap<String, Layout> = HashMap::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() != 3 {
                continue; // Skip malformed lines
            }

            let type_str = parts[0].to_string();
            let size = parts[1]
                .parse::<usize>()
                .map_err(|_| LayoutError::ParseError(format!("Invalid size: {}", parts[1])))?;
            let align = parts[2]
                .parse::<usize>()
                .map_err(|_| LayoutError::ParseError(format!("Invalid align: {}", parts[2])))?;

            parsed.insert(type_str, Layout { size, align });
        }

        // Map to result with original RustType keys
        let mut result = HashMap::new();
        for ty in types {
            let type_str = rust_type_to_string(ty);
            if let Some(layout) = parsed.get(&type_str) {
                result.insert(ty.clone(), layout.clone());
            } else {
                return Err(LayoutError::TypeNotFound(format!(
                    "Type '{}' not found. Check if it's public and the path is correct.",
                    type_str
                )));
            }
        }

        Ok(result)
    }
}

/// Returns the rustc version string.
///
/// Executes `rustc --version` and returns its output.
///
/// - Precondition: rustc is available on PATH.
/// - Postcondition: Returns the version string (e.g., "rustc 1.75.0"), or
///   an error if rustc cannot be executed.
pub fn get_rustc_version() -> LayoutResult<String> {
    let output = Command::new("rustc").arg("--version").output()?;

    if !output.status.success() {
        return Err(LayoutError::Other(
            "Failed to get rustc version".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Converts `ty` to valid Rust source code.
///
/// Handles all RustType variants including primitives, references, ADTs,
/// and generic types with proper syntax.
///
/// - Postcondition: Returns a string that can be used in Rust source code
///   to name the type.
fn rust_type_to_string(ty: &RustType) -> String {
    use zngur_def::{Mutability, PrimitiveRustType};

    match ty {
        RustType::Primitive(p) => match p {
            PrimitiveRustType::Uint(bits) => format!("u{}", bits),
            PrimitiveRustType::Int(bits) => format!("i{}", bits),
            PrimitiveRustType::Float(bits) => format!("f{}", bits),
            PrimitiveRustType::Usize => "usize".to_string(),
            PrimitiveRustType::Bool => "bool".to_string(),
            PrimitiveRustType::Str => "str".to_string(),
            PrimitiveRustType::ZngurCppOpaqueOwnedObject => {
                "zngur_types::ZngurCppOpaqueOwnedObject".to_string()
            }
        },
        RustType::Ref(mutability, inner) => {
            let mut_str = match mutability {
                Mutability::Mut => "mut ",
                Mutability::Not => "",
            };
            format!("&{}{}", mut_str, rust_type_to_string(inner))
        }
        RustType::Raw(mutability, inner) => {
            let mut_str = match mutability {
                Mutability::Mut => "mut ",
                Mutability::Not => "const ",
            };
            format!("*{}{}", mut_str, rust_type_to_string(inner))
        }
        RustType::Boxed(inner) => {
            format!("Box<{}>", rust_type_to_string(inner))
        }
        RustType::Slice(inner) => {
            format!("[{}]", rust_type_to_string(inner))
        }
        RustType::Dyn(trait_obj, _) => {
            format!("dyn {}", trait_obj)
        }
        RustType::Tuple(elements) => {
            if elements.is_empty() {
                "()".to_string()
            } else {
                let elements_str: Vec<_> = elements.iter().map(rust_type_to_string).collect();
                format!("({})", elements_str.join(", "))
            }
        }
        RustType::Adt(path_and_generics) => {
            let path_str = path_and_generics.path.join("::");
            if path_and_generics.generics.is_empty() && path_and_generics.named_generics.is_empty()
            {
                path_str
            } else {
                let mut generics_str: Vec<_> = path_and_generics
                    .generics
                    .iter()
                    .map(rust_type_to_string)
                    .collect();
                generics_str.extend(
                    path_and_generics
                        .named_generics
                        .iter()
                        .map(|(name, ty)| format!("{} = {}", name, rust_type_to_string(ty))),
                );
                format!("{}<{}>", path_str, generics_str.join(", "))
            }
        }
    }
}
