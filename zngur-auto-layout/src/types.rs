//! Shared types for layout extraction.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Size and alignment of a Rust type.
///
/// # Invariants
///
/// - `size` must be a valid type size (0 or greater)
/// - `align` must be a power of 2 (enforced by Rust's type system)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The size in bytes.
    pub size: usize,
    /// The alignment requirement in bytes (always a power of 2).
    pub align: usize,
}

/// Result type for layout operations.
pub type LayoutResult<T> = Result<T, LayoutError>;

/// Describes failures during layout extraction.
///
/// Each variant includes contextual information and a helpful hint
/// for resolving the error.
#[derive(Debug)]
pub enum LayoutError {
    /// The crate's compiled artifacts could not be located.
    ///
    /// - Hint: Run `cargo build` before attempting layout extraction.
    CrateNotFound(String),

    /// Compilation of the layout extraction program failed.
    ///
    /// - Hint: Ensure the target crate compiles successfully.
    CompilationFailed(String),

    /// Execution of the layout extraction program failed.
    ///
    /// - Hint: Check for linking or runtime issues.
    ExecutionFailed(String),

    /// Parsing the extraction program's output failed.
    ///
    /// - Hint: This likely indicates a bug in zngur-auto-layout.
    ParseError(String),

    /// A requested type was not found in the compiled crate.
    ///
    /// - Hint: Ensure the type is public and the path is correct.
    TypeNotFound(String),

    /// An I/O operation failed.
    ///
    /// - Hint: Check file permissions and disk space.
    IoError(std::io::Error),

    /// A cache operation failed.
    ///
    /// - Hint: Try deleting the cache directory.
    CacheError(String),

    /// Cargo is not available in PATH but is required for auto-layout.
    ///
    /// - Hint: Install Cargo or use explicit layouts.
    CargoNotFound,

    /// Failed to parse the compiled object file.
    ///
    /// - Hint: This may indicate a toolchain issue or unsupported platform.
    ObjectParseError(String),

    /// An unspecified error occurred.
    Other(String),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::CrateNotFound(msg) => {
                writeln!(f, "error: could not find compiled library")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: run `cargo build` first to compile your crate before running zngur"
                )
            }
            LayoutError::CompilationFailed(msg) => {
                writeln!(f, "error: failed to compile layout extraction program")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: check that your crate compiles successfully with `cargo build`"
                )
            }
            LayoutError::ExecutionFailed(msg) => {
                writeln!(f, "error: failed to execute layout extraction program")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: this may indicate a linking or runtime issue with your crate"
                )
            }
            LayoutError::ParseError(msg) => {
                writeln!(f, "error: failed to parse layout extraction output")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: this is likely a bug in zngur-auto-layout, please report it"
                )
            }
            LayoutError::TypeNotFound(msg) => {
                writeln!(f, "error: type not found in compiled crate")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: ensure the type is public and the path is correct in your .zng file"
                )
            }
            LayoutError::IoError(e) => {
                writeln!(f, "error: IO operation failed")?;
                writeln!(f, "  {}", e)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: check file permissions and available disk space"
                )
            }
            LayoutError::CacheError(msg) => {
                writeln!(f, "error: cache operation failed")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: try deleting the cache directory and rebuilding"
                )
            }
            LayoutError::CargoNotFound => {
                writeln!(f, "error: cargo is not available")?;
                writeln!(f, "  #layout(auto) requires Cargo to extract type layouts")?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: install Cargo from https://rustup.rs or use explicit #layout(size = X, align = Y)"
                )
            }
            LayoutError::ObjectParseError(msg) => {
                writeln!(f, "error: failed to parse compiled object file")?;
                writeln!(f, "  {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: this may indicate a toolchain issue or unsupported platform"
                )
            }
            LayoutError::Other(msg) => {
                writeln!(f, "error: {}", msg)?;
                writeln!(f)?;
                write!(
                    f,
                    "  = hint: for more details, please check the error message above"
                )
            }
        }
    }
}

impl std::error::Error for LayoutError {}

impl From<std::io::Error> for LayoutError {
    fn from(e: std::io::Error) -> Self {
        LayoutError::IoError(e)
    }
}

impl From<serde_json::Error> for LayoutError {
    fn from(e: serde_json::Error) -> Self {
        LayoutError::ParseError(e.to_string())
    }
}
