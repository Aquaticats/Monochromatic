//! What:
//!  The identities a client and a server compare in the handshake:
//! build identity and worktree identity.
//! Why:
//!  A server left running by an older build,
//! or serving a different or replaced worktree,
//! must not answer requests;
//!  the client drains it and starts a fresh one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function buildIdentity(): string; function worktreeIdentity(root: string): { path: string; dev: number; ino: number }
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `object` from `crate::json_build`.
use crate::json_build::object;
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `MetadataExt` from `std::os::unix::fs`.
use std::os::unix::fs::MetadataExt;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// What:
///  The running executable's identity:
/// crate version,
/// size and modification time.
/// Why:
///  A rebuilt binary changes size or mtime,
/// so a server started from the old build is recognized without hashing
///  the whole executable on every invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildIdentity(): string
/// ```
pub fn build_identity() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let (size, modified) = match std::env::current_exe().and_then(std::fs::metadata) {
        Ok(metadata) => (metadata.len(), metadata.mtime_nsec() + metadata.mtime() * 1_000_000_000),
        Err(_) => (0, 0),
    };
    return format!("{version}:{size}:{modified}");
}

/// What:
///  The canonical path plus directory identity (device and inode) of a worktree root.
///  `Value` is a JSON value (TS:
/// `unknown` parsed JSON).
/// Why:
///  The same path can name a different directory after a delete and recreate;
/// device and inode tell them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeIdentity(root: string): { path: string; dev: number; ino: number }
/// ```
pub fn worktree_identity(root: &Path) -> Result<Value, Diagnostic> {
    let canonical = match std::fs::canonicalize(root) {
        Ok(canonical) => canonical,
        Err(error) => {
            let message = format!("cannot resolve the worktree root {}: {error}", root.display());
            return Err(Diagnostic::new("root-missing", &message));
        }
    };
    let metadata = match std::fs::metadata(&canonical) {
        Ok(metadata) => metadata,
        Err(error) => {
            let message = format!("cannot inspect the worktree root {}: {error}", canonical.display());
            return Err(Diagnostic::new("root-missing", &message));
        }
    };
    let pairs = [
        ("path", Value::from(canonical.display().to_string())),
        ("dev", Value::from(metadata.dev())),
        ("ino", Value::from(metadata.ino())),
    ];
    return Ok(object(&pairs));
}
