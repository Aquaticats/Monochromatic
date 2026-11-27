//! What:
//! The comparison of two paths by the device and inode they name.
//! Why:
//! Two names of one program (hard links) have different resolved paths and one identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function isSameFile(first: string, second: string): boolean { /* stat both, compare dev and ino */ }
//! ```
/// Imports the file-system module that reads the two paths' metadata.
use std::fs;
/// Imports the metadata trait that gives a file's device and inode.
use std::os::unix::fs::MetadataExt;
/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `pub fn is_same_file(first: &Path, second: &Path) -> bool` says whether two paths name one file (the same device and inode).
/// Why:
/// Two names of one program (hard links) have different resolved paths and one identity.
pub fn is_same_file(
    first: &Path,
    second: &Path,
) -> bool {
    return match (fs::metadata(first), fs::metadata(second)) {
        (Ok(left), Ok(right)) => left.dev() == right.dev() && left.ino() == right.ino(),
        _ => false,
    };
}
