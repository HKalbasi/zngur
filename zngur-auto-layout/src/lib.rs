//! Automatic layout extraction for Zngur types.
//!
//! This crate provides functionality to automatically determine the size and alignment
//! of Rust types at build time by compiling and executing a helper program via rustc.
//!
//! # Examples
//!
//! ```no_run
//! use zngur_auto_layout::LayoutExtractor;
//! use zngur_def::RustType;
//! use std::path::Path;
//!
//! let extractor = LayoutExtractor::new(Path::new("."))
//!     .with_cache_dir(Path::new("target/zngur-cache").to_path_buf());
//!
//! let types = vec![/* RustTypes to extract */];
//! let layouts = extractor.extract_layouts(&types)?;
//! # Ok::<(), zngur_auto_layout::LayoutError>(())
//! ```

mod cache;
mod extractor;
mod types;

pub use types::{Layout, LayoutError, LayoutResult};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use zngur_def::RustType;

/// Extracts type layout information from compiled Rust crates.
///
/// The extractor compiles a temporary program that reports size and alignment
/// for requested types, with automatic caching to minimize build overhead.
///
/// # Invariants
///
/// - If `cache_dir` is `Some`, it must be a valid, writable directory path
/// - `crate_path` must point to a valid Cargo project root
pub struct LayoutExtractor {
    crate_path: PathBuf,
    cache_dir: Option<PathBuf>,
    target: Option<String>,
}

impl LayoutExtractor {
    /// Creates a layout extractor for the crate at `crate_path`.
    ///
    /// The crate must have been compiled (via `cargo build`) before extraction.
    ///
    /// - Postcondition: Returns an extractor with default settings (no cache,
    ///   host target).
    pub fn new(crate_path: impl AsRef<Path>) -> Self {
        Self {
            crate_path: crate_path.as_ref().to_path_buf(),
            cache_dir: None,
            target: None,
        }
    }

    /// Sets the cache directory to `dir`.
    ///
    /// Cached layouts are automatically invalidated when the compiler version,
    /// target, source files, or features change.
    ///
    /// - Postcondition: Subsequent calls to `extract_layouts` will use `dir`
    ///   for caching.
    pub fn with_cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = Some(dir);
        self
    }

    /// Sets the target triple for cross-compilation to `target`.
    ///
    /// The target's standard library must be installed via rustup.
    ///
    /// - Postcondition: Subsequent calls to `extract_layouts` will extract
    ///   layouts for `target`.
    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    /// Extracts size and alignment for all `types`.
    ///
    /// Uses cached values when available and valid, otherwise compiles a
    /// temporary program to extract fresh values.
    ///
    /// - Precondition: The crate at `crate_path` has been successfully compiled.
    /// - Precondition: All types in `types` are public and accessible from the
    ///   crate root.
    /// - Postcondition: Returns a map from each type to its layout, or an error
    ///   describing what failed.
    /// - Complexity: O(1) when cached and valid; O(n) compilation and execution
    ///   time when cache miss, where n is the crate size.
    pub fn extract_layouts(&self, types: &[RustType]) -> LayoutResult<HashMap<RustType, Layout>> {
        let cache = cache::Cache::new(self.cache_dir.as_deref(), self.target.as_deref())?;

        // Check if we can use cached values
        if let Some(cached_layouts) = cache.load(&self.crate_path)? {
            // Verify all requested types are in cache
            let all_cached = types.iter().all(|ty| cached_layouts.contains_key(ty));
            if all_cached {
                return Ok(types
                    .iter()
                    .map(|ty| (ty.clone(), cached_layouts[ty].clone()))
                    .collect());
            }
        }

        // Need to extract layouts
        let extractor = extractor::Extractor::new(&self.crate_path, self.target.as_deref())?;

        let layouts = extractor.extract(types)?;

        // Save to cache
        cache.save(&self.crate_path, &layouts)?;

        Ok(layouts)
    }

    /// Formats layouts for `types` as `.zng` syntax.
    ///
    /// The output can be copied directly into a `.zng` file to replace
    /// `#layout(auto)` directives with explicit values.
    ///
    /// - Precondition: Same as `extract_layouts`.
    /// - Postcondition: Returns a formatted string containing layout directives,
    ///   or an error if extraction fails.
    pub fn dump_layouts_zng(&self, types: &[RustType]) -> LayoutResult<String> {
        let layouts = self.extract_layouts(types)?;

        let mut output = String::new();
        output.push_str(&format!(
            "# Extracted layouts for {} (rustc {})\n",
            self.target.as_deref().unwrap_or("host"),
            extractor::get_rustc_version()?
        ));
        output.push_str(&format!(
            "# Generated at {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        for (ty, layout) in layouts.iter() {
            output.push_str(&format!("type {} {{\n", ty));
            output.push_str(&format!(
                "    #layout(size = {}, align = {});\n",
                layout.size, layout.align
            ));
            output.push_str("}\n\n");
        }

        Ok(output)
    }
}
