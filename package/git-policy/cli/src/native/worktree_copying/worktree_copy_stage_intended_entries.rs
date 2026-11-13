//! What:
//!  The record a copy writes into its stage before the first byte of the payload:
//! the repository paths the payload is about to hold,
//!  one per line,
//! and the reader that answers what a stage's payload may hold.
//! Why:
//!  A stage a copy left is removed only when nothing in it can be a person's,
//! and the wrapper cannot tell its own copy of a source's ignored entry from a file a person put at a path
//!  nobody copied:
//! the copied file and the person's file look alike,
//! and most of them are a copy of something the source holds.
//! A birth time does not separate them either:
//! a copy is killed at an arbitrary instruction,
//! so the instant after which nothing of its own was written is not on disk anywhere,
//! while a person's file is written after that instant and after the kill.
//! What does separate them is the copy's own statement of what it is writing,
//!  written before it writes anything:
//! a path the record names is one this copy meant to write,
//! and anything else inside the stage is somebody else's and keeps the folder
//!  (`worktree_copy_stage_claim_reaping.rs`).
//! The record is not installed:
//!  installation reads `payload/` alone, so a file of the container never reaches a worktree,
//! which `worktree_copy_stage_binding.rs` says of its own record too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordTheEntriesThePayloadWillHold(container, entries);
//! const intended = entriesThePayloadMayHold(container); // undefined: no record, or one that does not read
//! ```

use crate::path_bytes_in_records::{bytes_of_hex, hex_of_bytes};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_journal::write_atomic_unsynced;
use crate::worktree_copy_manifest::WorktreeCopyEntry;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// What:
///  The name of the record inside a stage container.
/// Why:
///   It sits beside `payload/` and the registration,
/// so no entry of the payload can ever have this name,
/// and a reader of the stage's top level knows it is the wrapper's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INTENDED_NAME = 'intended';
/// ```
pub const INTENDED_NAME: &str = "intended";

/// What:
///  The text of the record:
/// every repository path the payload will hold,
/// hex-encoded,
///  one per line,
/// in the order the manifest walked them.
/// Why:
///   A path is written as its bytes,
/// so a name that is not text is recorded whole and one line is always one path.
/// The list is the copy's statement of intent and not a promise of what stands there:
/// a kill leaves some of these paths missing,
/// which is why the reader adds the directories a path needs to the set it answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.map((entry) => hexOfBytes(entry.relativePath)).join('\n') + '\n'
/// ```
fn intended_text(entries: &[WorktreeCopyEntry]) -> String {
    let mut text: String = String::with_capacity(entries.len().saturating_mul(48));
    for entry in entries {
        text.push_str(hex_of_bytes(entry.relative_path.as_slice()).as_str());
        text.push('\n');
    }
    return text;
}

/// What:
///  Records the paths the payload of this stage is about to hold,
/// published whole by one rename and flushed by nobody.
/// Why:
///   It is written after the manifest is walked and before the first entry is copied,
/// so a kill at any later instruction leaves a stage that says what it was writing;
/// a kill before it leaves a stage holding no payload entry at all,
/// which loses nothing and is removed.
/// It flushes nothing for the reason `SPEC.md` gives under "The records an add writes for itself":
/// a kill keeps the write in the page cache,
/// and a power cut that loses it only leaves a stage the next command keeps and names,
/// which is the safe direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordTheEntriesThePayloadWillHold(container, entries);
/// ```
pub fn record_intended_entries(container: &Path, entries: &[WorktreeCopyEntry]) -> Result<(), WorktreeCopyError> {
    let path: PathBuf = container.join(INTENDED_NAME);
    return write_atomic_unsynced(path.as_path(), intended_text(entries).as_bytes(), "stage payload entries");
}

/// What:
///  The relative paths a stage's payload may hold,
/// with every directory one of them needs,
/// or nothing when the stage holds no record this release writes.
/// Why:
///   A directory is the wrapper's own when a recorded path lies below it,
/// so the set answers for the directories the copy made as well as for the entries themselves.
/// A record that does not read proves nothing,
/// and a stage holding one is kept rather than removed:
/// a person's file there exists nowhere else, and a wrong line here deletes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intended = entriesThePayloadMayHold(container); // undefined: no record, or one that does not read
/// ```
pub fn entries_the_payload_may_hold(container: &Path) -> Option<HashSet<Vec<u8>>> {
    let path: PathBuf = container.join(INTENDED_NAME);
    let bytes: Vec<u8> = std::fs::read(path.as_path()).ok()?;
    let text: &str = std::str::from_utf8(bytes.as_slice()).ok()?;
    let mut allowed: HashSet<Vec<u8>> = HashSet::new();
    for line in text.lines() {
        let path_bytes: Vec<u8> = bytes_of_hex(line)?;
        if path_bytes.is_empty() || path_bytes.contains(&0) {
            return None;
        }
        for relative in ancestors_and_self(path_bytes.as_slice()) {
            allowed.insert(relative);
        }
    }
    return Some(allowed);
}

/// What:
///  A path and every directory above it,
/// relative to the payload's root.
/// Why:
///   The copy creates the directory of every entry it writes,
/// so each of those directories is the wrapper's own although no recorded path names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['cache/npm/a.json', 'cache/npm', 'cache']
/// ```
fn ancestors_and_self(path: &[u8]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
    let mut end: usize = path.len();
    loop {
        out.push(path[..end].to_vec());
        let Some(separator) = path[..end].iter().rposition(|byte: &u8| { return *byte == b'/'; }) else {
            return out;
        };
        if separator == 0 {
            return out;
        }
        end = separator;
    }
}
