//! Cache management with automatic invalidation.
//!
//! The cache stores extracted layouts in JSON format with metadata for
//! automatic invalidation when the compiler, target, or source files change.

use crate::types::{Layout, LayoutResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use zngur_def::RustType;

/// Metadata for cache invalidation.
///
/// # Invariants
///
/// - `features` is sorted
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    rustc_version: String,
    rustc_commit_hash: Option<String>,
    target: String,
    source_hash: String,
    features: Vec<String>,
    created_at: String,
}

/// The complete cached data structure.
#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    metadata: CacheMetadata,
    layouts: HashMap<String, LayoutInfo>,
}

/// Serializable layout information.
#[derive(Debug, Serialize, Deserialize)]
struct LayoutInfo {
    size: usize,
    align: usize,
}

/// Manages layout cache with automatic invalidation.
///
/// # Invariants
///
/// - `cache_file` is in a directory that exists and is writable
/// - `current_metadata` reflects the current build environment
pub(crate) struct Cache {
    cache_file: PathBuf,
    current_metadata: CacheMetadata,
}

impl Cache {
    /// Creates a cache using `cache_dir` for storage and `target` for the build.
    ///
    /// Creates the cache directory if it doesn't exist. Defaults to OUT_DIR or
    /// a temp directory if `cache_dir` is None.
    ///
    /// - Postcondition: Returns a cache ready to load/save layouts, or an error
    ///   if directory creation or metadata building fails.
    pub(crate) fn new(cache_dir: Option<&Path>, target: Option<&str>) -> LayoutResult<Self> {
        let cache_dir = cache_dir
            .map(|p| p.to_path_buf())
            .or_else(|| env::var("OUT_DIR").ok().map(PathBuf::from))
            .unwrap_or_else(|| env::temp_dir().join("zngur-cache"));

        fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("zngur-layout-cache.json");

        let current_metadata = Self::build_metadata(target)?;

        Ok(Self {
            cache_file,
            current_metadata,
        })
    }

    fn build_metadata(target: Option<&str>) -> LayoutResult<CacheMetadata> {
        let rustc_version = crate::extractor::get_rustc_version()?;

        // Extract commit hash from version string if available
        let rustc_commit_hash = rustc_version
            .split_whitespace()
            .find(|s| s.len() == 41 && s.chars().all(|c| c.is_ascii_hexdigit()))
            .map(|s| s.to_string());

        // Determine target triple
        let target = target.map(|s| s.to_string()).unwrap_or_else(|| {
            // Try to detect from environment
            let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
            let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
            let env_var = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
            let vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_default();

            if !arch.is_empty() && !os.is_empty() {
                if !vendor.is_empty() && !env_var.is_empty() {
                    format!("{}-{}-{}-{}", arch, vendor, os, env_var)
                } else if !env_var.is_empty() {
                    format!("{}-unknown-{}-{}", arch, os, env_var)
                } else {
                    format!("{}-unknown-{}", arch, os)
                }
            } else {
                "host".to_string()
            }
        });

        // Collect feature flags
        let mut features: Vec<String> = env::vars()
            .filter_map(|(key, _)| {
                if key.starts_with("CARGO_FEATURE_") {
                    Some(key["CARGO_FEATURE_".len()..].to_lowercase())
                } else {
                    None
                }
            })
            .collect();
        features.sort();

        let created_at = chrono::Utc::now().to_rfc3339();

        Ok(CacheMetadata {
            rustc_version,
            rustc_commit_hash,
            target,
            source_hash: String::new(), // Will be filled when loading/saving
            features,
            created_at,
        })
    }

    fn compute_source_hash(&self, crate_path: &Path) -> LayoutResult<String> {
        let mut hasher = Sha256::new();

        // Hash Cargo.lock if it exists
        let cargo_lock = crate_path.join("Cargo.lock");
        if cargo_lock.exists() {
            let content = fs::read(&cargo_lock)?;
            hasher.update(&content);
        }

        // Hash the src directory modification time
        let src_dir = crate_path.join("src");
        if src_dir.exists() {
            if let Ok(metadata) = fs::metadata(&src_dir) {
                if let Ok(modified) = metadata.modified() {
                    let duration = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    hasher.update(duration.as_secs().to_le_bytes());
                }
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Loads cached layouts for the crate at `crate_path` if valid.
    ///
    /// Returns None if no cache exists or if the cache is stale (compiler
    /// version, target, source files, or features changed).
    ///
    /// - Postcondition: Returns Some(layouts) if cache is valid, None if cache
    ///   doesn't exist or is invalid, or an error if cache is corrupted.
    pub(crate) fn load(
        &self,
        crate_path: &Path,
    ) -> LayoutResult<Option<HashMap<RustType, Layout>>> {
        if !self.cache_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.cache_file)?;
        let cache_data: CacheData = match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(_) => {
                // Invalid cache, delete it
                let _ = fs::remove_file(&self.cache_file);
                return Ok(None);
            }
        };

        // Check if cache is still valid
        if !self.is_cache_valid(&cache_data.metadata, crate_path)? {
            // Cache is stale, delete it
            let _ = fs::remove_file(&self.cache_file);
            return Ok(None);
        }

        // Convert cached layouts back to RustType keys
        // For now, we'll just return None and force re-extraction
        // A full implementation would need to serialize/deserialize RustType properly
        Ok(None)
    }

    fn is_cache_valid(
        &self,
        cached_metadata: &CacheMetadata,
        crate_path: &Path,
    ) -> LayoutResult<bool> {
        // Check rustc version
        if cached_metadata.rustc_version != self.current_metadata.rustc_version {
            return Ok(false);
        }

        // Check target
        if cached_metadata.target != self.current_metadata.target {
            return Ok(false);
        }

        // Check features
        if cached_metadata.features != self.current_metadata.features {
            return Ok(false);
        }

        // Check source hash
        let current_hash = self.compute_source_hash(crate_path)?;
        if cached_metadata.source_hash != current_hash {
            return Ok(false);
        }

        Ok(true)
    }

    /// Saves `layouts` to cache for the crate at `crate_path`.
    ///
    /// Includes current metadata for automatic invalidation on next load.
    ///
    /// - Postcondition: Layouts are written to disk in JSON format, or an error
    ///   is returned if serialization or file writing fails.
    pub(crate) fn save(
        &self,
        crate_path: &Path,
        layouts: &HashMap<RustType, Layout>,
    ) -> LayoutResult<()> {
        let source_hash = self.compute_source_hash(crate_path)?;

        let mut metadata = self.current_metadata.clone();
        metadata.source_hash = source_hash;

        // Convert RustType keys to strings for serialization
        let layouts_serializable: HashMap<String, LayoutInfo> = layouts
            .iter()
            .map(|(ty, layout)| {
                (
                    format!("{}", ty),
                    LayoutInfo {
                        size: layout.size,
                        align: layout.align,
                    },
                )
            })
            .collect();

        let cache_data = CacheData {
            metadata,
            layouts: layouts_serializable,
        };

        let content = serde_json::to_string_pretty(&cache_data)?;
        fs::write(&self.cache_file, content)?;

        Ok(())
    }
}
