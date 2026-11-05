//! What:
//!  Lexical path normalization, the way Node's `path.resolve` does it.
//! Why:
//!  `git monochromatic-git-policy-cli doctor` scans each hooks directory once, so one directory named two ways (`./hooks`, `hooks/`,
//!  `a/../hooks`) must compare equal without touching the file system.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! path.resolve(path);
//! ```

/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Component, Path, PathBuf};

/// What:
///  Normalize a path lexically: drop `.`, fold `..` into the component before it, never above the root.
/// Why:
///   `PathBuf::join` keeps `.` and `..`, and a `..` above the root stays at the root, as Node clamps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.resolve(path);
/// ```
pub fn normalized_path(path: &Path) -> PathBuf {
    // `mut` lets the result grow and shrink as components arrive.
    let mut result: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                result.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                let at_root: bool = result.parent().is_none();
                if !at_root && result.file_name().is_some() {
                    result.pop();
                }
            }
            Component::Normal(name) => {
                result.push(name);
            }
        }
    }
    return result;
}

/// Normalization is controlled in the test module.
#[cfg(test)]
#[path = "lexical_path_tests.rs"]
mod tests;
