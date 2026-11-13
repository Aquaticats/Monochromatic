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

/// What:
///  The suffix of the record that says a stage was examined and kept,
/// beside the claim that names it.
/// Why:
///   The comparison against the record of what the copy was writing costs one lookup per payload entry,
/// and a kept stage is named on every command while its claim stands,
/// so the walk is made once for a folder that does not change and its answer is written down:
/// the person reads the same reason each time and no command pays the walk again.
/// The record can only ever save a keep and never a removal:
/// a removal is decided by a fresh walk every time,
/// so a record out of date keeps a folder longer and never deletes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEPT_SUFFIX = '.kept';
/// ```
pub const KEPT_SUFFIX: &str = ".kept";

/// What:
///  The line that holds the reason the person reads.
/// Why:
///   One field per line, read by its own key, as every record of this package is.
const REASON_LINE_PREFIX: &str = "reason=";

/// What:
///  The line that holds the state of the payload the reason was decided in.
const CHANGED_LINE_PREFIX: &str = "changed=";

/// What:
///  When the payload was last changed,
/// as the seconds of its change and modification times,
/// or nothing when it cannot be examined.
/// Why:
///   Any entry added to or removed from the payload changes the directory's change time,
/// and a person's file is such an entry;
/// whole seconds are enough because a record that misses a change only saves a keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = (payload) => { const found = statSync(payload); return { ctime: found.ctime, mtime: found.mtime }; };
/// ```
fn payload_changed(container: &Path) -> Option<(i64, i64)> {
    use std::os::unix::fs::MetadataExt;
    let found: std::fs::Metadata = std::fs::symlink_metadata(container.join(crate::worktree_copy_stage::PAYLOAD_NAME).as_path()).ok()?;
    return Some((found.ctime(), found.mtime()));
}

/// What:
///  The reason a kept stage was kept,
/// where the payload has not changed since that decision,
/// or nothing when no such record stands or the folder moved on.
/// Why:
///   The answer is the words the person read the first time,
/// and a payload whose times differ is examined again,
/// so a person who removes their file lets the next command settle the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = reasonTheStageWasKept(claim, container); // undefined before the first look, or after a change
/// ```
pub fn reason_the_stage_was_kept(claim: &Path, container: &Path) -> Option<String> {
    let recorded: (i64, i64) = payload_changed(container)?;
    let text: String = std::fs::read_to_string(kept_record_beside(claim)).ok()?;
    let mut reason: Option<&str> = None;
    let mut changed: Option<(i64, i64)> = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix(REASON_LINE_PREFIX) {
            reason = Some(value);
            continue;
        }
        if let Some(value) = line.strip_prefix(CHANGED_LINE_PREFIX) {
            let (left, right) = value.split_once('.')?;
            changed = Some((left.parse().ok()?, right.parse().ok()?));
            continue;
        }
        // A line this release does not write proves nothing about the folder.
        return None;
    }
    if changed != Some(recorded) {
        return None;
    }
    return reason.map(str::to_string).filter(|reason_text: &String| { return !reason_text.is_empty(); });
}

/// What:
///  Writes down that this stage was examined and kept,
/// with the reason and the state of the payload it was decided in,
/// published whole by one rename and flushed by nobody.
/// Why:
///   It saves only the walk, never the decision,
/// and a write that fails only means the next command walks again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordTheStageWasKept(claim, container, reason);
/// ```
pub fn record_the_stage_was_kept(claim: &Path, container: &Path, reason: &str) -> Result<(), WorktreeCopyError> {
    let Some((changed_seconds, modified_seconds)): Option<(i64, i64)> = payload_changed(container) else {
        return Ok(());
    };
    let recorded: String = format!("{REASON_LINE_PREFIX}{reason}\n{CHANGED_LINE_PREFIX}{changed_seconds}.{modified_seconds}\n");
    return write_atomic_unsynced(kept_record_beside(claim).as_path(), recorded.as_bytes(), "stage kept record");
}

/// What:
///  The path of the record that says a stage was kept,
/// for the claim that names the stage.
/// Why:
///   It keeps the claim's own name and changes only its suffix,
/// so one reader finds it and one release removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim.withExtension(KEPT_SUFFIX)
/// ```
pub fn kept_record_beside(claim: &Path) -> PathBuf {
    return claim.with_extension(&KEPT_SUFFIX[1..]);
}
