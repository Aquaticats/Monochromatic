//! What:
//!  One look at one path,
//!  and what it answered:
//!  a file with its owner and mode,
//!  a folder with its owner and mode,
//!  nothing at the path,
//!  an object of another kind,
//!  or a look this account was refused.
//! Why:
//!  Every handler and staging check judges presence,
//!  kind,
//!  owner and mode,
//!  and the four are answers a person repairs differently:
//!  a file that is not there is an install nobody ran,
//!  a folder at a file's path is a collision,
//!  another owner is a file the route does not control,
//!  and a refused look proves nothing at all
//!  (`crate::doctor_zfs_route::handler_checks_paths_and_modes`).
//!  The look reads the link itself rather than what it points at,
//!  because a link at an install path is the one state a person may not mean
//!  and a check must name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = thePathState('/usr/libexec/…/history_event-…'); // { kind: 'file', owner: 0, mode: 0o755 } | { kind: 'absent' } | …
//! ```

/// What one look at a path answered,
/// and the path a finding names beside it.
use crate::doctor_zfs_route::handler_checks::{
    PathFacts,
    PathState,
};
/// The owner and permission bits a file's own metadata carries,
/// and the kinds a look can be refused to name.
use std::os::unix::fs::{
    FileTypeExt,
    MetadataExt,
    PermissionsExt,
};
/// A borrowed path,
/// which the facts own a copy of.
use std::path::Path;

/// The permission bits a `PathState` keeps,
/// which is every bit a mode may carry and no file-type bit.
const PERMISSION_BITS: u32 = 0o7777;

/// What:
///  One path and what the look at it answered.
/// Why:
///  The path travels with the state so a finding names the file a person
///  opens,
///  and the install paths are constants of the build or of the places,
///  so a report names the same file an install writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = thePathFacts('/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc');
/// ```
pub(crate) fn the_path_facts(path: &Path) -> PathFacts {
    return PathFacts {
        path: path.to_path_buf(),
        state: the_path_state(path),
    };
}

/// What:
///  What one look at one path answered.
/// Why:
///  The look decides presence and kind,
///  and only its own answer can tell a refused look from an object that is not
///  there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = thePathState('/run/git-worktree-clone-mount-readiness'); // { kind: 'directory', owner, mode } | …
/// ```
pub(crate) fn the_path_state(path: &Path) -> PathState {
    let facts: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(held) => held,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return PathState::Absent,
        Err(error) => {
            return PathState::Unreadable {
                why: error.to_string(),
            };
        }
    };
    let owner: u32 = facts.uid();
    let mode: u32 = facts.permissions().mode() & PERMISSION_BITS;
    if facts.is_dir() {
        return PathState::Directory {
            owner,
            mode,
        };
    }
    if facts.is_file() {
        return PathState::File {
            owner,
            mode,
        };
    }
    return PathState::WrongKind {
        found: what_stands_there(&facts),
    };
}

/// What:
///  What kind of object a look found,
///  in words a person reads.
/// Why:
///  A finding must say what stands at the path,
///  and the kind is the only thing the look proves about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = whatStandsThere(facts); // 'a symbolic link'
/// ```
fn what_stands_there(facts: &std::fs::Metadata) -> String {
    let kind: std::fs::FileType = facts.file_type();
    if kind.is_symlink() {
        return String::from("a symbolic link");
    }
    if kind.is_fifo() {
        return String::from("a named pipe");
    }
    if kind.is_socket() {
        return String::from("a socket");
    }
    if kind.is_block_device() {
        return String::from("a block device node");
    }
    if kind.is_char_device() {
        return String::from("a character device node");
    }
    return String::from("an object of a kind this look does not name");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "one_look_at_a_path_tests.rs"]
mod tests;
