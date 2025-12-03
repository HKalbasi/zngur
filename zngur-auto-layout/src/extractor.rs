//! Core layout extraction logic via rustc compilation.
//!
//! This module implements layout extraction by generating and compiling a
//! temporary Rust object file with embedded layout data in object file sections.
//! This approach supports cross-compilation because we read the object file
//! directly rather than executing a compiled program.

use crate::types::{Layout, LayoutError, LayoutResult};
use object::{Object, ObjectSection};
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
    /// Generates a temporary Rust program with layout data embedded in object
    /// file sections using `#[link_section]`. The program is compiled (but not
    /// executed), and layouts are extracted by reading the object file directly.
    ///
    /// This approach supports cross-compilation because we read the object file
    /// for the target platform rather than executing code.
    ///
    /// - Precondition: The crate has been compiled and artifacts exist.
    /// - Precondition: All types are public and accessible.
    /// - Postcondition: Returns a map from types to their layouts, or an error
    ///   describing the first failure encountered.
    pub(crate) fn extract(&self, types: &[RustType]) -> LayoutResult<HashMap<RustType, Layout>> {
        // Find the compiled crate artifacts
        let crate_lib = self.find_crate_lib()?;

        // Generate the extraction program with #[link_section] breadcrumbs
        let extraction_code = self.generate_extraction_code(types)?;

        // Create a temporary directory for compilation
        let temp_dir = env::temp_dir().join(format!("zngur-layout-{}", std::process::id()));
        fs::create_dir_all(&temp_dir)?;

        let temp_file = temp_dir.join("extract_layouts.rs");
        fs::write(&temp_file, &extraction_code)?;

        // Compile to object file (don't link or run)
        let obj_file = temp_dir.join("extract_layouts.o");
        self.compile_to_object(&temp_file, &crate_lib, &obj_file)?;

        // Extract layouts from object file sections
        let layouts = self.extract_from_object(&obj_file, types)?;

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(layouts)
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
        // Priority: OUT_DIR (for build.rs context) > CARGO_TARGET_DIR > workspace root > crate root
        let target_dirs = if let Ok(out_dir) = env::var("OUT_DIR") {
            // In build.rs context, OUT_DIR is like target/release/build/<pkg>/out
            // The deps are in target/release/deps/
            let out_path = PathBuf::from(&out_dir);
            let mut dirs = vec![];

            // Walk up from OUT_DIR to find target directory
            let mut current = out_path.as_path();
            while let Some(parent) = current.parent() {
                if parent.file_name().map_or(false, |n| n == "target") {
                    dirs.push(parent.to_path_buf());
                    break;
                }
                // Also check if this is the profile directory (release/debug)
                if parent.join("deps").exists() {
                    // We're at the profile level, parent is target
                    if let Some(target_dir) = parent.parent() {
                        dirs.push(target_dir.to_path_buf());
                    }
                    break;
                }
                current = parent;
            }

            // Also check workspace and crate targets as fallback
            let mut check_dir = self.crate_path.as_path();
            while let Some(parent) = check_dir.parent() {
                let potential_target = parent.join("target");
                if potential_target.exists() && !dirs.contains(&potential_target) {
                    dirs.push(potential_target);
                }
                check_dir = parent;
            }
            dirs.push(self.crate_path.join("target"));
            dirs
        } else if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
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
                // Check deps directory first (for build.rs dependency context)
                let deps_path = target_dir.join(profile).join("deps");
                if deps_path.exists() {
                    // Look for lib<name>-<hash>.rlib pattern
                    if let Ok(entries) = fs::read_dir(&deps_path) {
                        for entry in entries.flatten() {
                            let filename = entry.file_name().to_string_lossy().to_string();
                            // Match lib<name>-<hash>.rlib or lib<name>.rlib
                            if filename.starts_with(&format!("lib{}", lib_name))
                                && (filename.ends_with(".rlib") || filename.ends_with(".rmeta"))
                            {
                                return Ok(entry.path());
                            }
                        }
                    }
                }

                // Also check direct lib path (for non-deps context)
                let lib_path = target_dir.join(profile);
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

    /// Generates Rust source code with layout data embedded in object file sections.
    ///
    /// For each type, generates a static array with `#[link_section]` attribute
    /// containing [size, align]. The section name encodes the type index.
    fn generate_extraction_code(&self, types: &[RustType]) -> LayoutResult<String> {
        let mut code = String::new();

        // Add necessary imports
        code.push_str("#![allow(dead_code)]\n");
        code.push_str("use std::mem::{size_of, align_of};\n\n");

        // Determine section name format based on target
        // Mach-O (macOS/iOS) requires "segment,section" format
        // ELF and PE/COFF use ".section" format
        let is_macho = self.is_macho_target();

        // Generate a link_section static for each type
        for (idx, ty) in types.iter().enumerate() {
            let type_str = rust_type_to_string(ty);

            // Use platform-appropriate section naming
            let section_name = if is_macho {
                // Mach-O format: __DATA,__zngur_N (segment,section)
                // Section names in Mach-O are limited to 16 chars, so keep it short
                format!("__DATA,__zngur{}", idx)
            } else {
                // ELF/PE format: .zngur_N
                format!(".zngur_{}", idx)
            };

            code.push_str(&format!(
                r#"#[used]
#[link_section = "{section}"]
static ZNGUR_LAYOUT_{idx}: [usize; 2] = [
    size_of::<{ty}>(),
    align_of::<{ty}>(),
];

"#,
                section = section_name,
                idx = idx,
                ty = type_str
            ));
        }

        // Add a dummy main for crate-type bin (helps with some linking scenarios)
        code.push_str("fn main() {}\n");

        Ok(code)
    }

    /// Returns true if the target uses Mach-O object format (macOS, iOS, etc.)
    fn is_macho_target(&self) -> bool {
        match &self.target {
            Some(target) => {
                target.contains("apple") || target.contains("darwin") || target.contains("ios")
            }
            None => {
                // No explicit target - check host platform
                cfg!(target_vendor = "apple")
            }
        }
    }

    /// Compiles the extraction source to an object file.
    ///
    /// Uses `rustc --emit=obj` to produce an object file without linking.
    /// This allows us to read the embedded layout data from sections.
    fn compile_to_object(
        &self,
        source: &Path,
        crate_lib: &Path,
        output: &Path,
    ) -> LayoutResult<()> {
        // Get the crate name for the --extern flag
        let crate_name = self.get_crate_name()?;
        let lib_name = crate_name.replace('-', "_");

        // Build rustc command to emit object file
        let mut cmd = Command::new("rustc");
        cmd.arg(source)
            .arg("--emit=obj")
            .arg("--crate-type=bin")
            .arg("--edition=2021")
            .arg("-o")
            .arg(output)
            .arg("--extern")
            .arg(format!("{}={}", lib_name, crate_lib.display()));

        // Add target if specified (critical for cross-compilation)
        if let Some(target) = &self.target {
            cmd.arg("--target").arg(target);
        }

        let result = cmd.output()?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(LayoutError::CompilationFailed(format!(
                "Failed to compile layout extraction program:\n{}",
                stderr
            )));
        }

        Ok(())
    }

    /// Extracts layout data from object file sections.
    ///
    /// Reads sections named `.zngur_N` (or platform variants like `__DATA,.zngur_N`)
    /// and parses the embedded [size, align] arrays.
    fn extract_from_object(
        &self,
        obj_path: &Path,
        types: &[RustType],
    ) -> LayoutResult<HashMap<RustType, Layout>> {
        let data = fs::read(obj_path)?;
        let obj = object::File::parse(&*data).map_err(|e| {
            LayoutError::ObjectParseError(format!("Failed to parse object file: {}", e))
        })?;

        // Determine pointer size from the object file
        let ptr_size = if obj.is_64() { 8usize } else { 4usize };
        let endian = obj.endianness();

        let mut layouts = HashMap::new();

        // Look for our layout sections
        for (idx, ty) in types.iter().enumerate() {
            let section_suffix = format!("zngur_{}", idx);

            // Find the section - handle platform-specific naming
            let section_data = self.find_layout_section(&obj, &section_suffix)?;

            // Parse the [size, align] array from section data
            let layout = parse_layout_from_bytes(&section_data, ptr_size, endian)?;
            layouts.insert(ty.clone(), layout);
        }

        Ok(layouts)
    }

    /// Finds a layout section by index, handling platform-specific section naming.
    ///
    /// - ELF: `.zngur_N`
    /// - Mach-O: `__zngurN` (in __DATA segment)
    /// - PE/COFF: `.zngur_N`
    fn find_layout_section(
        &self,
        obj: &object::File,
        section_suffix: &str,
    ) -> LayoutResult<Vec<u8>> {
        // Extract the index number from the suffix (e.g., "zngur_0" -> "0")
        let idx = section_suffix
            .strip_prefix("zngur_")
            .unwrap_or(section_suffix);

        for section in obj.sections() {
            if let Ok(name) = section.name() {
                // Check various naming patterns:
                // ELF/PE: ".zngur_N" or ends with "zngur_N"
                // Mach-O: "__zngurN" (section name only, segment is __DATA)
                let matches = name.ends_with(section_suffix)
                    || name.contains(&format!(".{}", section_suffix))
                    || name == format!("__zngur{}", idx)
                    || name.ends_with(&format!("__zngur{}", idx));

                if matches {
                    let data = section.data().map_err(|e| {
                        LayoutError::ObjectParseError(format!(
                            "Failed to read section '{}': {}",
                            name, e
                        ))
                    })?;
                    return Ok(data.to_vec());
                }
            }
        }

        Err(LayoutError::ObjectParseError(format!(
            "Could not find layout section for '{}'. Available sections: {:?}",
            section_suffix,
            obj.sections()
                .filter_map(|s| s.name().ok())
                .collect::<Vec<_>>()
        )))
    }
}

/// Parses a [size, align] layout from raw bytes.
///
/// Handles both 32-bit and 64-bit targets, and both endiannesses.
fn parse_layout_from_bytes(
    data: &[u8],
    ptr_size: usize,
    endian: object::Endianness,
) -> LayoutResult<Layout> {
    let expected_len = ptr_size * 2; // [usize; 2]
    if data.len() < expected_len {
        return Err(LayoutError::ObjectParseError(format!(
            "Section data too short: expected {} bytes, got {}",
            expected_len,
            data.len()
        )));
    }

    let (size, align) = match (ptr_size, endian) {
        (8, object::Endianness::Little) => {
            let size = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
            let align = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
            (size, align)
        }
        (8, object::Endianness::Big) => {
            let size = u64::from_be_bytes(data[0..8].try_into().unwrap()) as usize;
            let align = u64::from_be_bytes(data[8..16].try_into().unwrap()) as usize;
            (size, align)
        }
        (4, object::Endianness::Little) => {
            let size = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
            let align = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
            (size, align)
        }
        (4, object::Endianness::Big) => {
            let size = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
            let align = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
            (size, align)
        }
        _ => {
            return Err(LayoutError::ObjectParseError(format!(
                "Unsupported pointer size: {}",
                ptr_size
            )));
        }
    };

    Ok(Layout { size, align })
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
