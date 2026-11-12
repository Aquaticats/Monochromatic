//! What:
//!  What a clean check says about one tracked file:
//! the refusal types,
//! one index entry's recorded stat data,
//! and the verdict of
//!  comparing that data with the file's metadata.
//! Why:
//!  The proof that a worktree is clean compares every file with what the index recorded;
//! the content judge and the tree walks
//!  both need the same entry and verdict types,
//! so they live apart from either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = metadataVerdict(entry, metadata, indexSeconds); // 'agrees' | { unproven } | { differs }
//! ```

use crate::subvolume_index::{StatFields, StoredStat};
use crate::subvolume_index_files::{fields_of, matches_stored, matches_stored_exactly};
use std::os::unix::fs::MetadataExt;

/// The flags bit of an index entry that tells Git to trust the file without looking (`CE_VALID`).
pub const ASSUME_VALID: u16 = 0x8000;

/// What:
///  Which kind of state kept a worktree from being proven clean.
/// Why:
///   Callers decide by kind,
/// never by the words of the reason:
/// the snapshot rung restores a worktree whose only trouble is an
///  ordinary change,
/// and refuses every state a restore cannot be proven to undo;
/// the sentence for the person is chosen by kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanRefusalKind = { kind: 'operation-in-progress'; name: string } | { kind: 'unreadable' } | { kind: 'not-plain' }
///   | { kind: 'staged-change' } | { kind: 'file-changed' } | { kind: 'stat-stale' } | { kind: 'racy' } | { kind: 'untracked' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanRefusalKind {
    /// A merge,
    /// cherry-pick,
    /// revert,
    /// bisect,
    /// rebase,
    /// sequencer or `index.lock` mark exists;
    /// the name is the mark's.
    OperationInProgress(&'static str),
    /// The index,
    /// the object database,
    /// the configuration or Git itself cannot be read in the way the checks need.
    Unreadable,
    /// An index or tree entry of a kind a restore cannot be proven to undo:
    /// unmerged,
    /// intent-to-add,
    /// skip-worktree,
    ///  assume-unchanged,
    /// or a submodule.
    NotPlain,
    /// The index differs from the tree of `HEAD`:
    /// a staged addition,
    /// deletion,
    /// change or rename.
    StagedChange,
    /// A tracked file is missing,
    /// changed its type or executable bit,
    /// or differs in content.
    FileChanged,
    /// A tracked file's size,
    /// times or inode differ from its entry,
    /// so it probably changed.
    StatStale,
    /// A tracked file is as new as the index,
    /// so its stat fields prove nothing.
    Racy,
    /// Git lists an untracked file that no ignore rule covers.
    Untracked,
}

impl CleanRefusalKind {
    /// Whether a reset to a commit can undo the state:
    /// only ordinary changes,
    /// not an operation in progress or a state Git
    ///  keeps outside the tree and the files.
    pub fn restorable(self) -> bool {
        return matches!(self, Self::StagedChange | Self::FileChanged | Self::StatStale | Self::Racy | Self::Untracked);
    }
}

/// What:
///  Why a worktree could not be proven clean:
/// the kind and the reason in words.
/// Why:
///   The words are for logs and for people;
/// the kind is for code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanRefusal = { kind: CleanRefusalKind; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanRefusal {
    /// What kind of state refused.
    pub kind: CleanRefusalKind,
    /// The reason,
    /// in words.
    pub reason: String,
}

impl CleanRefusal {
    /// A refusal of `kind` with its reason.
    pub fn new(kind: CleanRefusalKind, reason: impl Into<String>) -> CleanRefusal {
        return CleanRefusal { kind, reason: reason.into() };
    }

    /// A refusal because something could not be read.
    pub fn unreadable(reason: impl Into<String>) -> CleanRefusal {
        return CleanRefusal::new(CleanRefusalKind::Unreadable, reason);
    }
}

impl std::fmt::Display for CleanRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.reason.as_str());
    }
}

/// What:
///  One tracked file of an index with the stat fields the index recorded for it.
/// Why:
///   `parse_index` leaves the stat fields out,
/// and the comparison with the file on disk needs them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatEntry = { path: Uint8Array; flags: number; mode: number; stored: StoredStat };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatEntry {
    /// The path,
    /// `/` separated.
    pub path: Vec<u8>,
    /// The entry's flags (the assume-valid bit among them).
    pub flags: u16,
    /// The mode with its type bits.
    pub mode: u32,
    /// The recorded stat fields.
    pub stored: StoredStat,
}

/// What:
///  What the stat data of an index entry says about a file.
/// Why:
///   A file the data cannot vouch for (a racily clean entry,
/// a stale stat) is not the same as a file that certainly changed:
///  Git settles the first by content,
/// so a caller that can compare content takes the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatVerdict = { kind: 'agrees' } | { kind: 'unproven'; reason: string } | { kind: 'differs'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatVerdict {
    /// Size,
    /// times,
    /// inode,
    /// type and mode all equal the entry,
    /// each nanosecond count the entry recorded equal to the
    ///  file's own,
    /// so no field was matched as a wildcard and the entry names the bytes the file holds.
    Agrees,
    /// The stat data proves nothing either way;
    /// the content decides.
    /// A count the entry recorded as zero,
    ///  which a file whose own count is not zero cannot equal,
    /// is one way here:
    /// a Git built without
    ///  nanosecond support wrote it for a time it could not express.
    Unproven(String),
    /// The file certainly differs:
    /// missing type,
    /// mode or assume-unchanged mark.
    Differs(String),
}

/// What:
///  What the stat data of the entry says about a file whose metadata is `metadata`,
/// for a caller deciding whether the
///  content has to be read.
/// Why:
///   The comparison takes the metadata of an inode already in hand,
/// so a caller that holds a descriptor judges the file it
///  holds,
/// not whatever the path names a moment later.
/// Type and executable bit come before the time checks:
/// they differ
///  whatever the content is.
/// An entry whose recorded nanosecond count the file does not equal is unproven,
/// not agreed,
///  even when every other field agrees:
/// zero is what a Git built without nanosecond support wrote for a time it could
///  not express,
/// and zero matches any present count,
/// so only the content tells what the file holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function metadataVerdict(entry: StatEntry, metadata: Stats, indexSeconds: number): StatVerdict;
/// ```
pub fn metadata_verdict(entry: &StatEntry, metadata: &std::fs::Metadata, index_seconds: u32) -> StatVerdict {
    return match stat_finding(entry, metadata, index_seconds) {
        Some(CleanRefusal { kind: CleanRefusalKind::Racy | CleanRefusalKind::StatStale, reason }) => StatVerdict::Unproven(reason),
        Some(refusal) => StatVerdict::Differs(refusal.reason),
        None if matches_stored_exactly(&fields_of(metadata), &entry.stored) => StatVerdict::Agrees,
        None => StatVerdict::Unproven(format!(
            "{} is recorded without the nanoseconds the file has, so only its content can prove which bytes it holds",
            String::from_utf8_lossy(entry.path.as_slice())
        )),
    };
}

/// What:
///  What the stat data of the entry says about a file whose metadata is `metadata`,
/// as a refusal with its kind,
/// or nothing
///  when they agree.
/// Why:
///   One comparison serves the stat-only checks,
/// whose kind the snapshot rung decides by,
/// and it reads the recorded
///  fields the way Git does:
/// a count it recorded as zero matches any present count.
/// A caller deciding whether
///  the content must be read uses `metadata_verdict`,
/// which does not take such an entry as proof.
/// Type and executable bit come before the time checks:
/// they differ whatever the content is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFinding(entry: StatEntry, metadata: Stats, indexSeconds: number): CleanRefusal | undefined;
/// ```
pub fn stat_finding(entry: &StatEntry, metadata: &std::fs::Metadata, index_seconds: u32) -> Option<CleanRefusal> {
    let StatEntry { path, flags, mode, stored }: &StatEntry = entry;
    let shown: String = String::from_utf8_lossy(path).into_owned();
    if flags & ASSUME_VALID != 0 {
        return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("{shown} is marked assume-unchanged")));
    }
    let symbolic: bool = *mode & 0o170_000 == 0o120_000;
    if symbolic != metadata.file_type().is_symlink() || (!symbolic && !metadata.file_type().is_file()) {
        return Some(CleanRefusal::new(CleanRefusalKind::FileChanged, format!("{shown} changed its type")));
    }
    if !symbolic && (*mode & 0o100 != 0) != (metadata.mode() & 0o100 != 0) {
        return Some(CleanRefusal::new(CleanRefusalKind::FileChanged, format!("{shown} changed its executable bit")));
    }
    if stored.mtime_seconds >= index_seconds {
        return Some(CleanRefusal::new(CleanRefusalKind::Racy, format!("{shown} is as new as the index, so its stat fields prove nothing")));
    }
    let now: StatFields = fields_of(metadata);
    if !matches_stored(&now, stored) {
        return Some(CleanRefusal::new(CleanRefusalKind::StatStale, format!("{shown} differs from its index entry by size, times or inode")));
    }
    return None;
}

/// What:
///  The reason a file whose metadata is `metadata` differs from its index entry,
/// or nothing when the two agree.
/// Why:
///   Callers that rely on the stat data alone (the clone rung) treat a file the data cannot vouch for like a
///  file that changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function metadataDiffersFromEntry(entry: StatEntry, metadata: Stats, indexSeconds: number): string | undefined;
/// ```
pub fn metadata_differs_from_entry(entry: &StatEntry, metadata: &std::fs::Metadata, index_seconds: u32) -> Option<String> {
    return match metadata_verdict(entry, metadata, index_seconds) {
        StatVerdict::Agrees => None,
        StatVerdict::Unproven(reason) | StatVerdict::Differs(reason) => Some(reason),
    };
}

/// What:
///  A refusal that a tracked file differs from the index in content.
/// Why:
///   The content check reports its reason as a file that changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Refusal('fileChanged', reason)
/// ```
pub fn file_changed(reason: String) -> CleanRefusal {
    return CleanRefusal::new(CleanRefusalKind::FileChanged, reason);
}

/// What:
///  The refusal for a file that changed,
/// as the `Some` an `Option` of refusals holds.
/// Why:
///   The directory walks return the first refusal,
/// and a missing file is one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return refusal('fileChanged', reason)
/// ```
pub fn changed_file(reason: String) -> Option<CleanRefusal> {
    return Some(file_changed(reason));
}
