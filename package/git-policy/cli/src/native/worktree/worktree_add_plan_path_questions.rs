//! What:
//!  Small questions about a path or a byte that the `git worktree add` plans ask:
//! whether it is a slash,
//! a link,
//! a `..`
//!  component,
//! and which ancestor exists.
//! Why:
//!  The plans of the ladder and of the ignored-state copy read the same arguments with the same path rules,
//! and the rules
//!  stand alone as functions of a path or a byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isSymbolicLink(path); namesParentDirectory(path); nearestExistingAncestor(path);
//! ```

use std::path::Path;

/// What:
///  Whether the byte is a slash.
/// Why:
///   An operand is split on its slashes to find its last component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === '/'.charCodeAt(0)
/// ```
pub fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Whether a path is a symbolic link,
/// without following it;
/// a path that cannot be examined is not one.
/// Why:
///   Git records the resolved path of a destination that is a link,
/// so the ladder declines such a destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return lstatSync(path).isSymbolicLink(); } catch { return false; }
/// ```
pub fn is_symbolic_link(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(found) => found.file_type().is_symlink(),
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether a path has a `..` component.
/// Why:
///   A `-C` directory with `..` cannot be resolved without the directories it leaves,
/// so the plan declines it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.split('/').includes('..')
/// ```
pub fn names_parent_directory(path: &Path) -> bool {
    for part in path.components() {
        if part == std::path::Component::ParentDir {
            return true;
        }
    }
    return false;
}

/// What:
///  The first of a path and its ancestors,
/// nearest first,
/// that exists.
/// Why:
///   A destination's missing parents are appended as written to the nearest one that can be resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ancestors(path).find((ancestor) => existsSync(ancestor))
/// ```
pub fn nearest_existing_ancestor(path: &Path) -> Option<&Path> {
    for ancestor in path.ancestors() {
        if ancestor.exists() {
            return Some(ancestor);
        }
    }
    return None;
}
