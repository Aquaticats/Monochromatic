//! What:
//!  Small operations on repo-relative paths held as text with `/` separators.
//! Why:
//!  Rules and the publisher all need "the directory part of this path";
//! one function keeps the empty-parent case
//!  (a file at the root has directory `""`) the same everywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function parentDirectory(path: string): string
//! ```

/// What:
///  The directory part of a repo-relative path:
///  everything before the last `/`,
///  or the empty string for a path with no slash.
///  `&str` borrowed in,
/// `&str` borrowed out:
/// the result is a slice of the input,
/// so nothing is copied.
/// Why:
///  A manifest at `package/a/b/Cargo.toml` lives in `package/a/b`;
/// a file at the worktree root lives in `""`,
///  which the rules treat as the root directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parentDirectory(path: string): string { const i = path.lastIndexOf('/'); return i < 0 ? '' : path.slice(0, i); }
/// ```
pub fn parent_directory(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((parent, _)) => return parent,
        None => return "",
    }
}
