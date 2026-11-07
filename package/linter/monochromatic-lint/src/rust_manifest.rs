//! What:
//!  The Cargo-manifest facts of a Rust input that need no compiler front end:
//!  which preparation a workspace load uses and which manifest owns an input.
//! Why:
//!  The closure check and the build without the `semantic` feature need manifest discovery,
//!  and only the semantic engine needs the rust-analyzer backend that loads a workspace.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // discoverManifest(input) -> nearest Cargo.toml path; WorkspacePreparation = 'source-only' | 'build-generated'
//! ```

/// Import typed setup failures.
use crate::rust_semantic_error::SemanticError;
/// Import native manifest discovery and explicit I/O failure kinds.
use std::io::ErrorKind;
/// Import the path types used to name the manifest.
use std::path::{Path, PathBuf};

/// Refuse a path that is not absolute UTF-8,
/// as the backend's path type does,
///  without panicking.
fn require_absolute_utf8(path: &Path) -> Result<(), SemanticError> {
    if path.to_str().is_none() {
        return Err(SemanticError::new(format!("Rust semantic analysis requires a UTF-8 path: {}. Rename or relocate this input, or configure the semantic rule off.", path.display()).as_str()));
    }
    if !path.is_absolute() {
        return Err(SemanticError::new(
            format!("Rust semantic path {} is not absolute.", path.display()).as_str(),
        ));
    }
    return Ok(());
}

/// Preparation is fixed by the host workflow,
///  never an arbitrary executable from JSONC.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspacePreparation {
    /// Read source/metadata only;
    ///  unavailable generated definitions remain explicit resolution failures.
    SourceOnly,
    /// Run locked/offline Cargo checking and load its macro artifacts for complete generated-source context.
    BuildGenerated,
}

/// Find the nearest Cargo manifest without searching unrelated child projects or rust-project command descriptors.
pub fn discover_manifest(input: &Path) -> Result<PathBuf, SemanticError> {
    require_absolute_utf8(input)?;
    let Some(directory): Option<&Path> = input.parent() else {
        return Err(SemanticError::new(
            "Rust input has no parent directory in which to discover Cargo.toml.",
        ));
    };
    for ancestor in directory.ancestors() {
        let candidate: PathBuf = ancestor.join("Cargo.toml");
        match std::fs::metadata(&candidate) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err(SemanticError::new(
                        format!(
                            "Rust workspace manifest {} is not a regular file.",
                            candidate.display()
                        )
                        .as_str(),
                    ));
                }
                return Ok(candidate);
            }
            Err(error) => {
                if error.kind() != ErrorKind::NotFound && error.kind() != ErrorKind::NotADirectory {
                    return Err(SemanticError::new(
                        format!(
                            "Cannot inspect Rust workspace manifest {}: {error}.",
                            candidate.display()
                        )
                        .as_str(),
                    ));
                }
            }
        }
    }
    return Err(SemanticError::new(format!("No Cargo.toml owns Rust input {}. Provide a Cargo workspace context or configure this semantic rule off for an intentional standalone snippet.", input.display()).as_str()));
}
