//! What:
//!  Which of a repository's registrations names one destination.
//! Why:
//!  A linked worktree is registered by its administrative directory under `<common>/worktrees`:
//! the `gitdir` file of that directory names the worktree's `.git` file,
//! and it does so whatever a command is doing to that `.git` file at the time.
//!  Recovery decided a copy was abandoned by reading the destination's `.git` pointer,
//! and a command that unlinks,
//! truncates or rewrites that file leaves it missing,
//! cut short or naming something else while the worktree stays registered;
//! the registrations answer the same question without reading the pointer at all.
//!  Nothing here waits or fails:
//! a `gitdir` that cannot be read is one Git would prune,
//! and a directory that cannot be listed is reported as unknown rather than as an absence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const naming = registrationNaming(commonDir, destination); // { named } | none | unknown
//! ```

use crate::plain_file::SMALL_FILE_LIMIT;
use crate::plain_file::read_plain_file;
use crate::worktree_add_binding::resolve_lexically;
use std::path::{Path, PathBuf};

/// What:
///  The registration that names a destination,
/// or why none does.
/// Why:
///  `read_registration` discards a pending copy only on a proven absence,
/// so the answer that proves nothing is kept apart from the answer that proves there is no registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamedRegistration = { named: string } | 'none' | 'unknown';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedRegistration {
    /// The administrative directory whose `gitdir` file names this destination.
    Named(PathBuf),
    /// No registration names it.
    None,
    /// The registrations cannot all be read,
    /// so whether one names it is unknown.
    Unknown,
}

/// What:
///  Which registration under `<common>/worktrees` names `destination`.
/// Why:
///   The destination's own `.git` file is not evidence:
/// a command that rewrites it leaves it empty,
/// half written or naming another directory,
/// and the registration stays.
/// A registration whose `gitdir` is missing or names no worktree is one Git would prune,
/// so it names nothing.
///  The destination is compared by the one canonical spelling both exist under,
/// because a registration written before the repository moved behind a link names the worktree all the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const admin of readdir(join(commonDir, 'worktrees'))) if (namesDestination(admin, destination)) return { named: admin }
/// ```
pub fn registration_naming(common_dir: &Path, destination: &Path) -> NamedRegistration {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(worktrees.as_path()) {
        Ok(listing) => listing,
        // No `worktrees` directory is a repository with no linked worktree at all.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return NamedRegistration::None,
        // A repository whose registrations cannot be listed proves nothing, so a copy is kept rather than discarded.
        Err(_unreadable) => return NamedRegistration::Unknown,
    };
    let mut unproven: bool = false;
    for listed in listing {
        let admin: PathBuf = match listed {
            Ok(entry) => entry.path(),
            Err(_unreadable) => {
                unproven = true;
                continue;
            }
        };
        match read_plain_file(admin.join("gitdir").as_path(), SMALL_FILE_LIMIT) {
            Ok(recorded) => {
                if names_destination(admin.as_path(), recorded.as_slice(), destination) {
                    return NamedRegistration::Named(admin);
                }
            }
            // An entry without a `gitdir` file is one Git would prune, and it names no worktree.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_unreadable) => unproven = true,
        }
    }
    return if unproven { NamedRegistration::Unknown } else { NamedRegistration::None };
}

/// What:
///  Whether one registration's `gitdir` names `destination`.
/// Why:
///  `gitdir` holds the path of the worktree's `.git` file,
/// absolute or relative to the administrative directory,
/// and its last component is always `.git`;
/// the directory that holds it is the worktree.
/// A registration whose recorded worktree is gone names nothing this copy could fill,
/// and one that cannot be resolved for another reason is not this destination's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marker = resolveLexically(admin, read('gitdir').trim()); return canonicalize(dirname(marker)) === destination
/// ```
fn names_destination(admin: &Path, recorded: &[u8], destination: &Path) -> bool {
    let line: &[u8] = recorded.trim_ascii();
    if line.is_empty() {
        return false;
    }
    let marker: PathBuf = match resolve_lexically(admin, line) {
        Ok(resolved) => resolved,
        Err(()) => return false,
    };
    let Some(named) = marker.parent() else {
        return false;
    };
    let Ok(canonical) = std::fs::canonicalize(named) else {
        return false;
    };
    return canonical.as_path() == destination;
}

/// Registration naming controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_registration_naming_tests.rs"]
mod tests;
