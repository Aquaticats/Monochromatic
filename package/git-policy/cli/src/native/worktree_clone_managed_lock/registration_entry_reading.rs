//! What:
//!  Read one `worktrees/<name>` entry as the facts the add's finder decides from.
//! Why:
//!  The finder's rows are the binding design's,
//!  and this route calls them rather than restating them
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The finder, and the lock's part in it").
//!  The finder needs one entry's lock bytes,
//!  its `gitdir` target,
//!  whether `commondir` and `HEAD` stand,
//!  the slot's pointer bytes,
//!  and how many entries the slot holds besides `.git`,
//!  and the entry-reader step 10 builds for the add's resume is not on this branch,
//!  so this file reads them.
//!  A `locked` the read refuses is reported as an empty one:
//!  it attributes nothing,
//!  while `file_exists` still keeps the registration from a prune (`F1`).
//!  The lock's own state and its refusal's words are read by `read_the_lock`,
//!  never here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = readOneEntry(worktrees, name, slot); // EntryFacts
//! ```

use super::lexical_path_resolution::the_lexical_path;
use super::lock_and_pointer_readings::{LOCK_FILE_NAME, SLOT_POINTER_NAME, read_the_slot_pointer};
use super::reading_a_bounded_file::{Reading, read_a_bounded_file};
use super::registration_proof_two_facts::REGISTRATION_RECORD_MAXIMUM_BYTES;
use crate::hooks_path_values::without_trailing_newlines;
use crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_MAXIMUM_BYTES;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// The registration's name for its back-link,
/// read here only to decide whether the finder may attribute the entry.
pub const GITDIR_FILE_NAME: &str = "gitdir";

/// The registration's name for the common directory it belongs to.
pub const COMMONDIR_FILE_NAME: &str = "commondir";

/// The registration's name for the head it checked out.
pub const HEAD_FILE_NAME: &str = "HEAD";

/// What:
///  The bytes one name holds,
///  absence as nothing,
///  and a refused read as nothing a finder may attribute.
/// Why:
///  The finder compares bytes and matches paths,
///  and it has no state for a link or a pipe,
///  so a name this route refuses to read is one it cannot attribute.
fn bytes_or_nothing(path: &Path, bound: u64) -> Option<Vec<u8>> {
    return match read_a_bounded_file(path, bound) {
        Reading::Bytes(bytes) => Some(bytes),
        Reading::Absent | Reading::Refused(_) => None,
    };
}

/// Whether an entry stands at one name,
/// with `lstat`'s own answer,
/// which is the test Git's `file_exists` makes.
fn an_entry_stands(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// The path one `gitdir` file names,
/// its ending newline trimmed and a relative spelling resolved against the entry.
fn back_link_of(entry: &Path) -> Option<PathBuf> {
    let bytes: Vec<u8> = bytes_or_nothing(entry.join(GITDIR_FILE_NAME).as_path(), REGISTRATION_RECORD_MAXIMUM_BYTES)?;
    let named: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(without_trailing_newlines(bytes.as_slice())));
    let spelled: PathBuf = if named.is_absolute() { named } else { entry.join(named) };
    return Some(the_lexical_path(spelled.as_path()));
}

/// How many entries the slot holds besides its `.git`,
/// which is what tells a bare registration (row 1) from one beside a filled slot.
fn other_entries_of_slot(slot: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(slot) else { return 0 };
    let mut count: usize = 0;
    for entry in entries.flatten() {
        if entry.file_name() != std::ffi::OsStr::new(SLOT_POINTER_NAME) {
            count += 1;
        }
    }
    return count;
}

/// What:
///  Read one `worktrees/<name>` entry into the facts the finder decides from.
/// Why:
///  The five names the rows turn on are read once here,
///  and a name that is absent is nothing rather than a refusal,
///  because every row is a state in which some of them are still missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = { name, locked, gitdir, slotGitPointer, commondir, head, slotOtherEntries };
/// ```
pub fn read_one_entry(worktrees: &Path, name: &str, slot: &Path) -> EntryFacts {
    let entry: PathBuf = worktrees.join(name);
    let locked: Option<Vec<u8>> = match read_a_bounded_file(entry.join(LOCK_FILE_NAME).as_path(), LOCK_WORDS_MAXIMUM_BYTES) {
        Reading::Absent => None,
        Reading::Bytes(bytes) => Some(bytes),
        Reading::Refused(_cause) => Some(Vec::new()),
    };
    let pointer: Option<Vec<u8>> = match read_the_slot_pointer(slot) {
        Reading::Bytes(bytes) => Some(bytes),
        Reading::Absent | Reading::Refused(_) => None,
    };
    return EntryFacts {
        name: String::from(name),
        locked,
        gitdir: back_link_of(entry.as_path()),
        slot_git_pointer: pointer,
        commondir: an_entry_stands(entry.join(COMMONDIR_FILE_NAME).as_path()),
        head: an_entry_stands(entry.join(HEAD_FILE_NAME).as_path()),
        slot_other_entries: other_entries_of_slot(slot),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_entry_reading_tests.rs"]
mod tests;
