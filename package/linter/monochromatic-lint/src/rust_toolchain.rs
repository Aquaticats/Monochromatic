//! What:
//!  Resolve the selected project's compiler and already installed standard-library source.
//! Why:
//!  Semantic checking must not silently install rust-src or change the caller's working directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Execute rustc with explicit argv/cwd, validate its absolute paths, and require existing source files.
//! ```

/// Import the typed setup failure.
use crate::rust_semantic_error::SemanticError;
/// Import the backend's absolute UTF-8 path type.
use ra_ap_vfs::AbsPathBuf;
/// Import native paths and captured compiler-query output.
use std::path::Path;
/// Import the process types that query the selected toolchain for its compiler paths.
use std::process::{Command, Output};

/// Validated compiler/source paths,
///  without any executable override from linter configuration.
pub struct RustToolchain {
    /// Compiler's installed root,
    ///  as reported by that project's rustc selection.
    pub sysroot: AbsPathBuf,
    /// Existing standard-library source;
    ///  passed explicitly to prevent automatic rustup fallback.
    pub library: AbsPathBuf,
}

/// Convert native caller paths without the backend's panic-on-invalid-path helper.
pub(crate) fn absolute_utf8(path: &Path) -> Result<AbsPathBuf, SemanticError> {
    let Some(text): Option<&str> = path.to_str() else {
        return Err(SemanticError::new(format!("Rust semantic analysis requires a UTF-8 path: {}. Rename or relocate this input, or configure the semantic rule off.", path.display()).as_str()));
    };
    match AbsPathBuf::try_from(text) {
        Ok(absolute) => return Ok(absolute),
        Err(error) => {
            return Err(SemanticError::new(
                format!(
                    "Rust semantic path {} is not absolute: {error:?}.",
                    path.display()
                )
                .as_str(),
            ));
        }
    }
}

/// Query only the selected project's compiler;
///  missing rust-src is an actionable error,
///  not an installation request.
pub fn discover_toolchain(directory: &Path) -> Result<RustToolchain, SemanticError> {
    absolute_utf8(directory)?;
    let output: Output = match Command::new("rustc").args(["--print", "sysroot"]).current_dir(directory).output() {
        Ok(result) => result,
        Err(error) => return Err(SemanticError::new(format!("Cannot run rustc --print sysroot in {}: {error}. Make that project's Rust toolchain available before semantic checking.", directory.display()).as_str())),
    };
    if !output.status.success() {
        return Err(SemanticError::new(
            format!(
                "rustc --print sysroot failed in {} ({}): {}",
                directory.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )
            .as_str(),
        ));
    }
    let text: String = match String::from_utf8(output.stdout) {
        Ok(value) => value,
        Err(error) => {
            return Err(SemanticError::new(
                format!(
                    "rustc returned a non-UTF-8 sysroot for {}: {error}.",
                    directory.display()
                )
                .as_str(),
            ));
        }
    };
    let sysroot: AbsPathBuf = absolute_utf8(Path::new(text.trim()))?;
    let library: AbsPathBuf = sysroot.join("lib/rustlib/src/rust/library");
    let core: AbsPathBuf = library.join("core/src/lib.rs");
    let metadata: std::fs::Metadata = match std::fs::metadata(&core) {
        Ok(value) => value,
        Err(error) => return Err(SemanticError::new(format!("Cannot read the Rust standard-library source at {core}: {error}. Install the matching rust-src component with the toolchain manager, or configure this semantic rule off. The linter will not install components automatically.").as_str())),
    };
    if !metadata.is_file() {
        return Err(SemanticError::new(format!("Rust standard-library source {core} is not a regular file. Repair the matching rust-src installation before semantic checking.").as_str()));
    }
    return Ok(RustToolchain { sysroot, library });
}
