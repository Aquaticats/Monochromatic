//! What:
//!  One path with `.` dropped and `..` resolved against the components before it,
//!  and nothing read from the filesystem.
//! Why:
//!  Git writes a worktree's whole path in a registration's `gitdir` and `../..` in its `commondir`,
//!  and a registration another command wrote may hold a relative spelling of either.
//!  A comparison of those records against a recorded path must not go through `canonicalize`:
//!  the path they name may be gone (the slot whose pointer the lock outlives),
//!  and a name that cannot be resolved would then refuse a registration the add wrote.
//!  Git writes no `..` in the back-link it makes itself,
//!  so a lexical resolution is the whole of the crossing this route makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const resolved = path.resolve('/a/b/../c'); // the lexical answer, without touching the disk
//! ```

use std::path::{
    Component,
    Path,
    PathBuf,
};

/// What:
///  One path with its `.` components dropped and its `..` components resolved.
/// Why:
///  A relative record resolved against its registration and an absolute one compared as it stands
///  both read through this,
///  so a `..` in either spelling names the path it stands for.
pub fn the_lexical_path(path: &Path) -> PathBuf {
    let mut resolved: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let the_last = resolved.components().next_back();
                if matches!(the_last, Some(Component::Normal(_))) {
                    resolved.pop();
                } else if !resolved.has_root() {
                    resolved.push(component.as_os_str());
                }
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    return resolved;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lexical_path_resolution_tests.rs"]
mod tests;
