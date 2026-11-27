//! What:
//!  The spelling work a discovery needs before it can compare paths:
//!  the absolute form of the walked directory,
//!  its lexical normalization without touching a filesystem entry,
//!  and the `/`-separated scope prefix an index path is matched against.
//! Why:
//!  Git compares and reports paths as `/`-separated bytes,
//!  a scan may start anywhere below the working tree root,
//!  and the walked directory arrives relative or holding `.` and `..`,
//!  so every comparison needs one normalized spelling first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // absolutePath(p) -> lexicallyNormalized(p) -> scopeBytes(below)
//! ```

/// Imports the path component type the normalization classifies,
///  and the two path types every caller passes.
use std::path::{
    Component,
    Path,
    PathBuf,
};

/// Imports the platform-aware separator normalization,
///  so one scope prefix compares against Git's always-`/`-separated index
///  paths.
use crate::path_name_bytes::normalized_path;

/// The walked directory's path below the working tree root,
///  `/`-separated with a trailing separator,
///  empty when the two are the same directory.
pub(super) fn scope_bytes(below: &Path) -> Vec<u8> {
    if below.as_os_str().is_empty() {
        return Vec::new();
    }
    let mut bytes: Vec<u8> = normalized_path(below);
    bytes.push(b'/');
    return bytes;
}

/// The absolute form of `path`,
///  resolved against the current directory when relative,
///  with `.` and `..` dealt with lexically and no symlink followed.
pub(super) fn absolute_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return Some(lexically_normalized(path));
    }
    let current: PathBuf = std::env::current_dir().ok()?;
    return Some(lexically_normalized(&current.join(path)));
}

/// `path` with `.` components dropped and `..` resolved against the
///  component before it,
///  touching no filesystem entry.
///
/// Every input is absolute,
///  so a `..` that climbs past the filesystem root is dropped,
///  which is what Git and the kernel do with such a path.
///  This resolves spelling only:
///  a `..` after a symlink names a different directory on disk,
///  so nothing that must match the kernel's own resolution is normalized
///  here,
/// and `gitdir:` and `commondir` paths keep their spelling.
pub(crate) fn lexically_normalized(path: &Path) -> PathBuf {
    let mut parts = path.components().peekable();
    // A volume prefix is the base a root component keeps: `PathBuf::push` of a
    // rooted path replaces everything but the prefix, so the prefix starts the
    // result and the root is pushed onto it.
    let mut normalized: PathBuf = match parts.peek() {
        Some(Component::Prefix(prefix)) => {
            let base: PathBuf = PathBuf::from(prefix.as_os_str());
            parts.next();
            base
        }
        _ => PathBuf::new(),
    };
    for part in parts {
        match part {
            Component::Prefix(_) => {}
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    return normalized;
}
