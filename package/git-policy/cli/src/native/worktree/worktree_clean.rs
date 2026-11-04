//! What:
//!  Whether a worktree is clean enough to be copied from, or removed without asking Git: no operation in progress, an
//!  index equal to the tree of `HEAD`, every tracked file unchanged by its recorded size, times and inode, and no
//!  untracked file that Git does not ignore.
//! Why:
//!  A snapshot or a clone carries the bytes of the source's files, so any change the source has not committed would
//!  reach the new worktree, and a removal by the subvolume ioctl cannot be undone. The tracked-file checks read the
//!  index in process (no Git process) with the same comparison Git's fast path makes, and refuse every shape they cannot
//!  prove: a split or sparse index, a skip-worktree or assume-valid entry, a submodule, an entry as new as the index
//!  file (Git re-hashes those, so their stat fields prove nothing). The one question the index cannot answer, whether
//!  an untracked file is ignored, goes to one `git ls-files` run. Every refusal returns its reason in words for the log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clean = await checkClean({ root, gitDir, commonDir }); // { headCommit, headTree } or throws a reason
//! ```

use crate::add_timing::{begin_step, begin_work, end_step, end_work};
use crate::scoped_start::start_or_run;
use crate::repo_read_index::{IndexEntry, parse_index};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_refs::{Refs, RefValue};
use crate::repo_read_tree::{FlatEntry, MODE_GITLINK, flatten_tree, parse_commit};
use crate::fs_beneath::{entry_metadata_at, open_directory_beneath, open_root, plain_components};
use crate::worktree_clean_content::{ContentJudge, ConversionEnvironment, Place};
use crate::subvolume_index::{StatFields, StoredStat, be32, walk_entries};
use crate::subvolume_index_files::{fields_of, matches_stored, repository_hash_length};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The entries of a Git directory that mean an operation is in progress and the index or tree is not a plain state.
const IN_PROGRESS: &[&str] =
    &["index.lock", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "BISECT_LOG", "rebase-merge", "rebase-apply", "sequencer"];

/// The flags bit of an index entry that tells Git to trust the file without looking (`CE_VALID`).
const ASSUME_VALID: u16 = 0x8000;

/// What:
///  Which kind of state kept a worktree from being proven clean.
/// Why:
///   Callers decide by kind, never by the words of the reason: the snapshot rung restores a worktree whose only trouble is an
///  ordinary change, and refuses every state a restore cannot be proven to undo; the sentence for the person is chosen by kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanRefusalKind = { kind: 'operation-in-progress'; name: string } | { kind: 'unreadable' } | { kind: 'not-plain' }
///   | { kind: 'staged-change' } | { kind: 'file-changed' } | { kind: 'stat-stale' } | { kind: 'racy' } | { kind: 'untracked' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanRefusalKind {
    /// A merge, cherry-pick, revert, bisect, rebase, sequencer or `index.lock` mark exists; the name is the mark's.
    OperationInProgress(&'static str),
    /// The index, the object database, the configuration or Git itself cannot be read in the way the checks need.
    Unreadable,
    /// An index or tree entry of a kind a restore cannot be proven to undo: unmerged, intent-to-add, skip-worktree,
    ///  assume-unchanged, or a submodule.
    NotPlain,
    /// The index differs from the tree of `HEAD`: a staged addition, deletion, change or rename.
    StagedChange,
    /// A tracked file is missing, changed its type or executable bit, or differs in content.
    FileChanged,
    /// A tracked file's size, times or inode differ from its entry, so it probably changed.
    StatStale,
    /// A tracked file is as new as the index, so its stat fields prove nothing.
    Racy,
    /// Git lists an untracked file that no ignore rule covers.
    Untracked,
}

impl CleanRefusalKind {
    /// Whether a reset to a commit can undo the state: only ordinary changes, not an operation in progress or a state Git
    ///  keeps outside the tree and the files.
    pub fn restorable(self) -> bool {
        return matches!(self, Self::StagedChange | Self::FileChanged | Self::StatStale | Self::Racy | Self::Untracked);
    }
}

/// What:
///  Why a worktree could not be proven clean: the kind and the reason in words.
/// Why:
///   The words are for logs and for people; the kind is for code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanRefusal = { kind: CleanRefusalKind; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanRefusal {
    /// What kind of state refused.
    pub kind: CleanRefusalKind,
    /// The reason, in words.
    pub reason: String,
}

impl CleanRefusal {
    /// A refusal of `kind` with its reason.
    fn new(kind: CleanRefusalKind, reason: impl Into<String>) -> CleanRefusal {
        return CleanRefusal { kind, reason: reason.into() };
    }

    /// A refusal because something could not be read.
    fn unreadable(reason: impl Into<String>) -> CleanRefusal {
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
///   `parse_index` leaves the stat fields out, and the comparison with the file on disk needs them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatEntry = { path: Uint8Array; flags: number; mode: number; stored: StoredStat };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatEntry {
    /// The path, `/` separated.
    pub path: Vec<u8>,
    /// The entry's flags (the assume-valid bit among them).
    pub flags: u16,
    /// The mode with its type bits.
    pub mode: u32,
    /// The recorded stat fields.
    pub stored: StoredStat,
}

/// What:
///  What a clean worktree's `HEAD` names.
/// Why:
///   The two-tree update that follows a snapshot starts from this tree, so a caller needs not read it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanSource = { headCommit: string; headTree: Uint8Array; tracked: number; indexSeconds: number; hashLength: number; indexBytes: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanSource {
    /// The commit `HEAD` resolves to, as hexadecimal.
    pub head_commit: String,
    /// The root tree of that commit, as a raw object name.
    pub head_tree: Vec<u8>,
    /// The number of tracked files.
    pub tracked: usize,
    /// The whole seconds of the index file's modification time, against which racy entries were judged.
    pub index_seconds: u32,
    /// The hash length of the repository in bytes.
    pub hash_length: usize,
    /// The `.gitattributes` files of the tree of `HEAD` as path and object name.
    pub attribute_blobs: Vec<(Vec<u8>, Vec<u8>)>,
    /// The bytes of the index file that were proven equal to the tree of `HEAD` and to the files. Every later step uses
    ///  these bytes, never a second read of the file, so what was proven is what is installed.
    pub index_bytes: Vec<u8>,
}

/// What:
///  A refusal that the index or the Git data could not be read, when `found` holds nothing.
/// Why:
///   Every read of the repository's own files refuses in the same way, saying which read failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// found ?? throw unreadable(reason)
/// ```
fn or_unreadable<T>(found: Option<T>, reason: &'static str) -> Result<T, CleanRefusal> {
    return match found {
        Some(value) => Ok(value),
        None => Err(CleanRefusal::unreadable(reason)),
    };
}

/// What:
///  A refusal that the index file could not be opened, examined or read.
/// Why:
///   Three steps of reading the index word their failure the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unreadable(`the index is unreadable (${error})`)
/// ```
fn index_unreadable(error: std::io::Error) -> CleanRefusal {
    return CleanRefusal::unreadable(format!("the index is unreadable ({error})"));
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
fn file_changed(reason: String) -> CleanRefusal {
    return CleanRefusal::new(CleanRefusalKind::FileChanged, reason);
}

/// What:
///  The refusal for a file that changed, as the `Some` an `Option` of refusals holds.
/// Why:
///   The directory walks return the first refusal, and a missing file is one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return refusal('fileChanged', reason)
/// ```
fn changed_file(reason: String) -> Option<CleanRefusal> {
    return Some(file_changed(reason));
}

/// What:
///  Whether the cached directory handle is not the one for `parents`, so the directory must be opened.
/// Why:
///   Entries come in path order, so most share the directory of the one before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cached === undefined || !sameNames(cached.names, parents)
/// ```
fn cache_misses(cached: &Option<(Vec<std::ffi::OsString>, std::fs::File)>, parents: &[std::ffi::OsString]) -> bool {
    return match cached {
        Some((known, _directory)) => known.as_slice() != parents,
        None => true,
    };
}

/// What:
///  Whether a byte is NUL, the end of each name in Git's `-z` output.
/// Why:
///   The untracked names are split on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The first entry of the Git directory that marks an operation in progress, if any.
/// Why:
///   A merge, a rebase or a held index lock means the index is not the committed state yet; the lock a caller holds itself
///  (`owned_index_lock`) does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function operationInProgress(gitDir: string): string | undefined;
/// ```
fn operation_in_progress(git_dir: &Path, owned_index_lock: bool) -> Option<&'static str> {
    for name in IN_PROGRESS.iter().copied() {
        if owned_index_lock && name == "index.lock" {
            continue;
        }
        if git_dir.join(name).symlink_metadata().is_ok() {
            return Some(name);
        }
    }
    return None;
}

/// What:
///  The commit `HEAD` resolves to, read from the plain ref files.
/// Why:
///   An unusual ref storage answers "unknown", which declines the shortcut rather than guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headCommit(gitDir: string, commonDir: string, format: ObjectFormat): string | undefined;
/// ```
pub fn head_commit(git_dir: &Path, common_dir: &Path, format: ObjectFormat) -> Option<String> {
    let refs: Refs = Refs::new(common_dir, format.hex_length());
    return match refs.lookup_head(git_dir)? {
        RefValue::Direct(object) => Some(object),
        RefValue::Symbolic(name) => refs.resolve(name.as_str()),
    };
}

/// What:
///  The tracked files of the index paired with their recorded stat fields, or nothing when the layout is not plain.
/// Why:
///   The comparison with `lstat` needs the fields `parse_index` leaves out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function storedStats(bytes: Uint8Array, hashLength: number): { path: Uint8Array; flags: number; mode: number; stored: StoredStat }[] | undefined;
/// ```
pub fn stored_stats(bytes: &[u8], hash_length: usize) -> Option<Vec<StatEntry>> {
    let (entries, _end) = walk_entries(bytes, hash_length)?;
    let mut found: Vec<StatEntry> = Vec::with_capacity(entries.len());
    for entry in entries {
        let stored: StoredStat = StoredStat {
            ctime_seconds: be32(bytes, entry.start)?,
            ctime_nanos: be32(bytes, entry.start + 4)?,
            mtime_seconds: be32(bytes, entry.start + 8)?,
            mtime_nanos: be32(bytes, entry.start + 12)?,
            ino: be32(bytes, entry.start + 20)?,
            uid: be32(bytes, entry.start + 28)?,
            gid: be32(bytes, entry.start + 32)?,
            size: be32(bytes, entry.start + 36)?,
        };
        found.push(StatEntry { path: entry.path, flags: entry.flags, mode: entry.mode, stored });
    }
    return Some(found);
}

/// What:
///  The reason the index differs from the tree of `HEAD`, or nothing when the two are the same files.
/// Why:
///   A staged change, a staged deletion or an intent-to-add entry is an uncommitted state the copy must not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexDiffersFromTree(index: IndexEntry[], tree: FlatEntry[]): string | undefined;
/// ```
fn index_differs_from_tree(index: &[IndexEntry], tree: &[FlatEntry]) -> Option<CleanRefusal> {
    if index.len() != tree.len() {
        return Some(CleanRefusal::new(CleanRefusalKind::StagedChange, "the index and the tree of HEAD hold different numbers of files"));
    }
    for (staged, committed) in index.iter().zip(tree) {
        if staged.stage != 0 || staged.extended != 0 {
            return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("the index entry {} is unmerged, intent-to-add or skip-worktree", String::from_utf8_lossy(&staged.path))));
        }
        if staged.mode == MODE_GITLINK {
            return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("{} is a submodule", String::from_utf8_lossy(&staged.path))));
        }
        if staged.path != committed.path || staged.mode != committed.mode || staged.id != committed.id {
            return Some(CleanRefusal::new(CleanRefusalKind::StagedChange, format!("{} is staged with a change", String::from_utf8_lossy(&staged.path))));
        }
    }
    return None;
}

/// What:
///  What the stat data of an index entry says about a file.
/// Why:
///   A file the data cannot vouch for (a racily clean entry, a stale stat) is not the same as a file that certainly changed:
///  Git settles the first by content, so a caller that can compare content takes the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatVerdict = { kind: 'agrees' } | { kind: 'unproven'; reason: string } | { kind: 'differs'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatVerdict {
    /// Size, times, inode, type and mode all equal the entry.
    Agrees,
    /// The stat data proves nothing either way; the content decides.
    Unproven(String),
    /// The file certainly differs: missing type, mode or assume-unchanged mark.
    Differs(String),
}

/// What:
///  What the stat data of the entry says about a file whose metadata is `metadata`.
/// Why:
///   The comparison takes the metadata of an inode already in hand, so a caller that holds a descriptor judges the file it
///  holds, not whatever the path names a moment later. Type and executable bit come before the time checks: they differ
///  whatever the content is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function metadataVerdict(entry: StatEntry, metadata: Stats, indexSeconds: number): StatVerdict;
/// ```
pub fn metadata_verdict(entry: &StatEntry, metadata: &std::fs::Metadata, index_seconds: u32) -> StatVerdict {
    return match stat_finding(entry, metadata, index_seconds) {
        None => StatVerdict::Agrees,
        Some(CleanRefusal { kind: CleanRefusalKind::Racy | CleanRefusalKind::StatStale, reason }) => StatVerdict::Unproven(reason),
        Some(refusal) => StatVerdict::Differs(refusal.reason),
    };
}

/// What:
///  What the stat data of the entry says about a file whose metadata is `metadata`, as a refusal with its kind, or nothing
///  when they agree.
/// Why:
///   One comparison serves the verdict that callers of the stat-only checks use and the kind that the snapshot rung decides
///  by. Type and executable bit come before the time checks: they differ whatever the content is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFinding(entry: StatEntry, metadata: Stats, indexSeconds: number): CleanRefusal | undefined;
/// ```
fn stat_finding(entry: &StatEntry, metadata: &std::fs::Metadata, index_seconds: u32) -> Option<CleanRefusal> {
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
///  The reason a file whose metadata is `metadata` differs from its index entry, or nothing when the two agree.
/// Why:
///   Callers that rely on the stat data alone (the snapshot and clone rungs) treat a file the data cannot vouch for like a
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
///  The first reason a tracked file of `root` differs from its index entry, or nothing when every file agrees.
/// Why:
///   After a snapshot the same comparison runs against the snapshot's own files, so a source that changed while the
///  snapshot was taken is caught on the copy. The files are reached below a held root with no link followed, so a directory
///  replaced by a link is not read through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndex(root: string, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index(root: &Path, entries: &[StatEntry], index_seconds: u32) -> Option<String> {
    return match open_root(root) {
        Ok(held) => tree_differs_from_index_beneath(&held, entries, index_seconds),
        Err(error) => Some(format!("{} cannot be opened without following a link ({error})", root.display())),
    };
}

/// The first refusal of a tracked file of `root` against its index entry, with its kind; the files are reached below a held
///  root with no link followed.
fn tree_refusal(root: &Path, entries: &[StatEntry], index_seconds: u32) -> Option<CleanRefusal> {
    return match open_root(root) {
        Ok(held) => tree_refusal_beneath(&held, entries, index_seconds),
        Err(error) => Some(CleanRefusal::unreadable(format!("{} cannot be opened without following a link ({error})", root.display()))),
    };
}

/// What the content decision needs besides the entries: the worktree, its Git directories, the environment and the format.
pub struct ContentTarget<'a> {
    /// The worktree root.
    pub root: &'a Path,
    /// The worktree's Git directory.
    pub git_dir: &'a Path,
    /// The shared Git directory.
    pub common_dir: &'a Path,
    /// The configuration and attribute locations.
    pub environment: &'a ConversionEnvironment,
    /// The object format of the repository.
    pub format: ObjectFormat,
}

/// What:
///  The first reason a tracked file of the worktree differs from its index entry, deciding by content where the stat data
///  cannot vouch for the file.
/// Why:
///   Git hashes a racily clean entry and an entry whose stat data is stale; this does the same in process, so a clean
///  worktree that was copied, cloned or written in the same second as its index is not refused. Every file is reached below a
///  held root with no link followed, so a directory replaced by a link is refused instead of being read through. A file that
///  really differs, or one a content conversion could change, is refused with the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndexByContent(target: ContentTarget, entries: StatEntry[], indexed: IndexEntry[], indexSeconds: number): string | undefined;
/// ```
fn tree_differs_from_index_by_content(target: &ContentTarget<'_>, entries: &[StatEntry], indexed: &[IndexEntry], index_seconds: u32) -> Option<String> {
    if entries.len() != indexed.len() {
        return Some(String::from("the index entries and their stat data are not the same files"));
    }
    let held: std::fs::File = match open_root(target.root) {
        Ok(opened) => opened,
        Err(error) => return Some(format!("{} cannot be opened without following a link ({error})", target.root.display())),
    };
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, target.git_dir, target.common_dir, target.environment, target.format);
    let mut cached: Option<(Vec<std::ffi::OsString>, std::fs::File)> = None;
    for (entry, committed) in entries.iter().zip(indexed) {
        if entry.path != committed.path {
            return Some(String::from("the index entries and their stat data are not the same files"));
        }
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice()))) else { return Some(format!("{shown} is not a plain path")) };
        let Some((name, parents)) = names.split_last() else { return Some(format!("{shown} is not a plain path")) };
        if cache_misses(&cached, parents) {
            let Ok(directory) = open_directory_beneath(&held, parents) else { return Some(format!("{shown} is missing from the worktree or reached through a link")) };
            cached = Some((parents.to_vec(), directory));
        }
        let Some((_known, directory)) = cached.as_ref() else { return Some(format!("{shown} is missing from the worktree")) };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else { return Some(format!("{shown} is missing from the worktree")) };
        match metadata_verdict(entry, &metadata, index_seconds) {
            StatVerdict::Agrees => {}
            StatVerdict::Differs(reason) => return Some(reason),
            StatVerdict::Unproven(_reason) => {
                let place: Place<'_> = Place { directory, parents, name: name.as_os_str() };
                if let Err(reason) = judge.agrees(&place, entry, committed.id.as_slice(), index_seconds) {
                    return Some(reason);
                }
            }
        }
    }
    return None;
}

/// What:
///  The first reason a tracked file below the held directory `root` differs from its index entry, or nothing when every
///  file agrees; each file is looked at below handles opened without following a link.
/// Why:
///   A snapshot is checked against the index it will carry; a path walk through a link inside the snapshot would compare
///  a file outside it, so every directory on the way must be a real one on the snapshot's device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndexBeneath(root: Dir, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index_beneath(root: &std::fs::File, entries: &[StatEntry], index_seconds: u32) -> Option<String> {
    return match tree_refusal_beneath(root, entries, index_seconds) {
        Some(refusal) => Some(refusal.reason),
        None => None,
    };
}

/// The first refusal of a tracked file below the held directory `root` against its index entry, with its kind.
fn tree_refusal_beneath(root: &std::fs::File, entries: &[StatEntry], index_seconds: u32) -> Option<CleanRefusal> {
    let mut cached: Option<(Vec<std::ffi::OsString>, std::fs::File)> = None;
    for entry in entries {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice()))) else { return Some(CleanRefusal::unreadable(format!("{shown} is not a plain path"))) };
        let Some((name, parents)) = names.split_last() else { return Some(CleanRefusal::unreadable(format!("{shown} is not a plain path"))) };
        if cache_misses(&cached, parents) {
            let Ok(directory) = open_directory_beneath(root, parents) else { return changed_file(format!("{shown} is missing from the worktree or reached through a link")) };
            cached = Some((parents.to_vec(), directory));
        }
        let Some((_held, directory)) = cached.as_ref() else { return changed_file(format!("{shown} is missing from the worktree")) };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else { return changed_file(format!("{shown} is missing from the worktree")) };
        if let Some(refusal) = stat_finding(entry, &metadata, index_seconds) {
            return Some(refusal);
        }
    }
    return None;
}

/// What:
///  A walk below a held directory that keeps the last directory it opened.
/// Why:
///   The paths of an index come in order, so most of them share their parent with the one before; one `openat2` chain per
///  directory instead of per file is what the walk costs, and each file is still reached through handles that follow no link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cursor = new StatCursor(root); cursor.fields(path); // StatFields | undefined
/// ```
pub struct StatCursor<'a> {
    /// The directory every path is found below.
    root: &'a std::fs::File,
    /// The components and the handle of the last directory opened.
    held: Option<(Vec<std::ffi::OsString>, std::fs::File)>,
}

impl<'a> StatCursor<'a> {
    /// A cursor below `root` that has opened nothing.
    pub fn new(root: &'a std::fs::File) -> Self {
        return Self { root, held: None };
    }

    /// The metadata of the file `path` names below the root, or the reason it cannot be found through plain directories.
    fn metadata(&mut self, path: &[u8]) -> Result<std::fs::Metadata, String> {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(path))) else { return Err(format!("{shown} is not a plain path")) };
        let Some((name, parents)) = names.split_last() else { return Err(format!("{shown} is not a plain path")) };
        if cache_misses(&self.held, parents) {
            let Ok(directory) = open_directory_beneath(self.root, parents) else { return Err(format!("{shown} is missing from the worktree or reached through a link")) };
            self.held = Some((parents.to_vec(), directory));
        }
        let Some((_known, directory)) = self.held.as_ref() else { return Err(format!("{shown} is missing from the worktree")) };
        return match entry_metadata_at(directory, name.as_os_str()) {
            Ok(found) => Ok(found),
            Err(_missing) => Err(format!("{shown} is missing from the worktree")),
        };
    }

    /// The stat fields of the tracked file `path` below the root, or nothing when it cannot be found through plain directories.
    pub fn fields(&mut self, path: &[u8]) -> Option<StatFields> {
        return match self.metadata(path) {
            Ok(found) => Some(fields_of(&found)),
            Err(_refused) => None,
        };
    }
}

/// What:
///  Judge every tracked file below the held directory `root` against its index entry and return the stat fields of each, in
///  entry order, in one walk.
/// Why:
///   The snapshot's index takes the stat fields of the snapshot's own files, and the snapshot is proven equal to the source's
///  index by the same stat data; the same `statx` serves both, so the files are looked at once, and the fields are the ones the
///  verdict was given on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fields = treeFieldsBeneath(root, entries, indexSeconds); // throws a CleanRefusal
/// ```
pub fn tree_fields_beneath(root: &std::fs::File, entries: &[StatEntry], index_seconds: u32) -> Result<Vec<StatFields>, CleanRefusal> {
    let mut cursor: StatCursor<'_> = StatCursor::new(root);
    let mut fields: Vec<StatFields> = Vec::with_capacity(entries.len());
    for entry in entries {
        let metadata: std::fs::Metadata = match cursor.metadata(entry.path.as_slice()) {
            Ok(found) => found,
            Err(reason) if reason.ends_with("is not a plain path") => return Err(CleanRefusal::unreadable(reason)),
            Err(reason) => return Err(CleanRefusal::new(CleanRefusalKind::FileChanged, reason)),
        };
        if let Some(refusal) = stat_finding(entry, &metadata, index_seconds) {
            return Err(refusal);
        }
        fields.push(fields_of(&metadata));
    }
    return Ok(fields);
}

/// What:
///  The stat fields of the tracked file `path` below the held directory `root`, found through handles, or nothing.
/// Why:
///   The index of a snapshot takes the stat fields of the snapshot's own files; reading them by a path with several
///  components would follow a link substituted below the root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFieldsBeneath(root: Dir, path: Uint8Array): StatFields | undefined;
/// ```
pub fn stat_fields_beneath(root: &std::fs::File, path: &[u8]) -> Option<StatFields> {
    return StatCursor::new(root).fields(path);
}

/// What:
///  Whether Git lists an untracked file that no ignore rule covers.
/// Why:
///   Only Git's ignore rules can tell an untracked file that matters from one it ignores; `--directory` stops at the
///  first untracked directory and `--no-empty-directory` leaves out directories without files, as `git status` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function untrackedFile(context: GitContext, root: string): Promise<string | undefined>;
/// ```
fn untracked_file(context: &GitContext, root: &Path) -> Result<Option<String>, CleanRefusal> {
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["ls-files", "--others", "--exclude-standard", "--directory", "--no-empty-directory", "-z"].map(Into::into).to_vec(),
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(CleanRefusal::unreadable(format!("Git could not list untracked files ({error})"))),
    };
    if output.code != Some(0) {
        return Err(CleanRefusal::unreadable(format!("Git could not list untracked files: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())));
    }
    // The first name that is not empty, as `find` chose it.
    for name in output.stdout.split(is_nul_byte) {
        if !name.is_empty() {
            return Ok(Some(String::from_utf8_lossy(name).into_owned()));
        }
    }
    return Ok(None);
}

/// What:
///  The checks of `check_clean` that read no untracked file: no operation in progress, an index equal to the tree of `HEAD`,
///  and every tracked file with the size, times and inode its index entry recorded.
/// Why:
///   A caller that copies the whole worktree, untracked files included (the conversion of a worktree into a subvolume),
///  needs a quiet and unmodified tracked state but has no reason to refuse for an untracked file, and needs no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCleanTracked(root: string, gitDir: string, commonDir: string): CleanSource; // throws the reason
/// ```
pub fn check_clean_tracked(root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, String> {
    return match check_tracked(root, git_dir, common_dir, None, false) {
        Ok(source) => Ok(source),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What:
///  `check_clean_tracked` that settles an entry the stat data cannot vouch for by comparing the file with the committed
///  blob in process, under the configuration and attribute locations of `environment`.
/// Why:
///   The conversion copies the whole worktree and rewrites the index afterwards, and an entry it could not prove keeps its old
///  stat data, so Git hashes it at the first status; the snapshot and clone rungs install the index they proved and use the
///  stat-only check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCleanTrackedByContent(root: string, gitDir: string, commonDir: string, environment: ConversionEnvironment, ownedIndexLock: boolean): CleanSource; // throws the reason
/// ```
pub fn check_clean_tracked_by_content(root: &Path, git_dir: &Path, common_dir: &Path, environment: &ConversionEnvironment, owned_index_lock: bool) -> Result<CleanSource, String> {
    return match check_tracked(root, git_dir, common_dir, Some(environment), owned_index_lock) {
        Ok(source) => Ok(source),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What the checks read before they judge: the index with its stat data, the commit `HEAD` names and its flattened tree.
struct State {
    /// The bytes of the index file.
    bytes: Vec<u8>,
    /// The whole seconds of the index file's modification time.
    index_seconds: u32,
    /// The hash length of the repository in bytes.
    hash_length: usize,
    /// The object format.
    format: ObjectFormat,
    /// The parsed index entries.
    entries: Vec<IndexEntry>,
    /// The stat fields of the entries.
    stats: Vec<StatEntry>,
    /// The commit `HEAD` names.
    commit: String,
    /// The root tree of that commit.
    tree_id: Vec<u8>,
    /// The flattened tree.
    tree: Vec<FlatEntry>,
}

impl State {
    /// The `.gitattributes` files of the tree as path and object name.
    fn attribute_blobs(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut blobs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for entry in &self.tree {
            if entry.path == b".gitattributes" || entry.path.ends_with(b"/.gitattributes") {
                blobs.push((entry.path.clone(), entry.id.clone()));
            }
        }
        return blobs;
    }

    /// What `HEAD` names, with the index bytes the caller proved (or none, for a worktree that is not clean).
    fn into_source(self, index_bytes: Vec<u8>) -> CleanSource {
        let attribute_blobs: Vec<(Vec<u8>, Vec<u8>)> = self.attribute_blobs();
        return CleanSource { head_commit: self.commit, head_tree: self.tree_id, tracked: self.entries.len(), index_seconds: self.index_seconds, hash_length: self.hash_length, attribute_blobs, index_bytes };
    }
}

/// What:
///  Read the index and the tree of `HEAD` of a worktree, in process.
/// Why:
///   The judgement of cleanliness and the snapshot of a worktree that is not clean start from the same reading; every shape
///  the readers do not know is a refusal of kind `Unreadable`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readState(gitDir: string, commonDir: string): State; // throws a CleanRefusal
/// ```
fn read_state(git_dir: &Path, common_dir: &Path) -> Result<State, CleanRefusal> {
    let hash_length: usize = or_unreadable(repository_hash_length(common_dir), "the repository configuration is unreadable")?;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    // One open file gives both the bytes and the time, so the time belongs to the bytes that were proven.
    let index_path = git_dir.join("index");
    let mut index_file: std::fs::File = std::fs::File::open(index_path.as_path()).map_err(index_unreadable)?;
    let index_seconds: u32 = index_file.metadata().map_err(index_unreadable)?.mtime() as u32;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut index_file, &mut bytes).map_err(index_unreadable)?;
    let entries: Vec<IndexEntry> = or_unreadable(parse_index(bytes.as_slice(), format), "the index has a layout the reader does not know")?;
    let stats = or_unreadable(stored_stats(bytes.as_slice(), hash_length), "the index entries have no plain layout")?;
    let commit: String = or_unreadable(head_commit(git_dir, common_dir, format), "HEAD is not a plain ref to a commit")?;
    let database: ObjectDatabase = or_unreadable(ObjectDatabase::open(common_dir, None, format), "the object database has a layout the reader does not know")?;
    let commit_name: Vec<u8> = or_unreadable(crate::repo_read_object::from_hex(commit.as_bytes(), format), "HEAD names no object")?;
    let commit_object = or_unreadable(database.read(commit_name.as_slice()), "the commit of HEAD is unreadable")?;
    let commit_fields = or_unreadable(parse_commit(commit_object.bytes.as_slice(), format), "the commit of HEAD has a layout the reader does not know")?;
    let tree: Vec<FlatEntry> = or_unreadable(flatten_tree(&database, commit_fields.tree.as_slice(), format), "the tree of HEAD is unreadable")?;
    return Ok(State { bytes, index_seconds, hash_length, format, entries, stats, commit, tree_id: commit_fields.tree, tree });
}

/// The checks of `check_clean_tracked`, by stat data alone or, given a `scope`, by content where the stat data is not enough.
fn check_tracked(root: &Path, git_dir: &Path, common_dir: &Path, environment: Option<&ConversionEnvironment>, owned_index_lock: bool) -> Result<CleanSource, CleanRefusal> {
    if let Some(name) = operation_in_progress(git_dir, owned_index_lock) {
        return Err(CleanRefusal::new(CleanRefusalKind::OperationInProgress(name), format!("{name} exists in {}", git_dir.display())));
    }
    let read_index_and_tree_entered: bool = begin_step("readIndexAndTree");
    let read_index_and_tree_outcome = read_state(git_dir, common_dir);
    end_step(read_index_and_tree_entered);
    let state: State = read_index_and_tree_outcome?;
    let index_against_tree_entered: bool = begin_step("indexAgainstTree");
    let index_against_tree_outcome = index_differs_from_tree(state.entries.as_slice(), state.tree.as_slice());
    end_step(index_against_tree_entered);
    if let Some(refusal) = index_against_tree_outcome {
        return Err(refusal);
    }
    let differs: Option<CleanRefusal> = match environment {
        None => {
            let walk_source_entered: bool = begin_step("walkSource");
            let walked: Option<CleanRefusal> = tree_refusal(root, state.stats.as_slice(), state.index_seconds);
            end_step(walk_source_entered);
            walked
        }
        Some(found) => tree_differs_from_index_by_content(&ContentTarget { root, git_dir, common_dir, environment: found, format: state.format }, state.stats.as_slice(), state.entries.as_slice(), state.index_seconds)
            .map(file_changed),
    };
    if let Some(refusal) = differs {
        return Err(refusal);
    }
    let bytes: Vec<u8> = state.bytes.clone();
    return Ok(state.into_source(bytes));
}

/// What:
///  The first entry of a worktree's index or tree that a reset to a commit cannot be proven to undo, if any: unmerged,
///  intent-to-add and skip-worktree entries, assume-unchanged entries, and submodules in the index or in the tree of `HEAD`.
/// Why:
///   The first refusal of the clean check can be an ordinary change that hides one of these further on; a restore needs all of
///  the index looked at, not the first offender.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryAResetCannotUndo(state: State): CleanRefusal | undefined;
/// ```
fn entry_a_reset_cannot_undo(state: &State) -> Option<CleanRefusal> {
    for staged in &state.entries {
        let shown: String = String::from_utf8_lossy(&staged.path).into_owned();
        if staged.stage != 0 || staged.extended != 0 {
            return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("the index entry {shown} is unmerged, intent-to-add or skip-worktree")));
        }
        if staged.mode == MODE_GITLINK {
            return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("{shown} is a submodule")));
        }
    }
    for entry in &state.tree {
        if entry.mode == MODE_GITLINK {
            return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("{} is a submodule in HEAD", String::from_utf8_lossy(&entry.path))));
        }
    }
    for entry in &state.stats {
        if entry.flags & ASSUME_VALID != 0 {
            return Some(CleanRefusal::new(CleanRefusalKind::NotPlain, format!("{} is marked assume-unchanged", String::from_utf8_lossy(&entry.path))));
        }
    }
    return None;
}

/// Cleanliness controls stay out of the release executable.
/// What:
///  Check that the worktree at `root`, whose Git directory is `git_dir` and whose shared directory is `common_dir`, is
///  clean; return what its `HEAD` names, or the reason it cannot be proven clean.
/// Why:
///   The caller turns a refusal into a quiet fall to the next rung (or to Git's own removal) and logs the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkClean(args: { context: GitContext; root: string; gitDir: string; commonDir: string }): Promise<CleanSource>; // throws a reason
/// ```
pub fn check_clean(context: &GitContext, root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, CleanRefusal> {
    // The tracked files (the index, the tree of `HEAD`, a stat of every file) and the untracked scan (one Git run over the
    //  directory tree) read different things and write nothing, so they run side by side; the tracked refusal still comes first.
    let (tracked, untracked): (Result<CleanSource, CleanRefusal>, Result<Option<String>, CleanRefusal>) = std::thread::scope(|scope| {
        let scan = start_or_run(scope, || {
            let began: std::time::Instant = begin_work();
            let scanned: Result<Option<String>, CleanRefusal> = untracked_file(context, root);
            end_work("untrackedScan", began);
            return scanned;
        });
        let tracked = check_tracked(root, git_dir, common_dir, None, false);
        let scanned = match scan.join() {
            Ok(found) => found,
            Err(_panic) => Err(CleanRefusal::unreadable("the scan for untracked files stopped unexpectedly")),
        };
        return (tracked, scanned);
    });
    let clean: CleanSource = tracked?;
    if let Some(name) = untracked? {
        return Err(CleanRefusal::new(CleanRefusalKind::Untracked, format!("{name} is untracked and not ignored")));
    }
    return Ok(clean);
}

/// What:
///  What `HEAD` names for a worktree that is not clean, and why it is not, once it is known that the only trouble is ordinary
///  changes.
/// Why:
///   The snapshot rung copies such a worktree and restores the copy to its `HEAD`; the copy's own index is installed, so the
///  index bytes here are only as read: they name the tracked paths and prove nothing about the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtySource = { source: CleanSource; why: CleanRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirtySource {
    /// What `HEAD` names; its `index_bytes` are the index as read, not proven equal to the files.
    pub source: CleanSource,
    /// The first reason the worktree is not clean.
    pub why: CleanRefusal,
}

/// What:
///  For a worktree `check_clean` refused with the ordinary change `why`, what `HEAD` names, or the refusal that a restore
///  cannot undo when the index holds one anywhere.
/// Why:
///   The first refusal hides the rest: a staged edit can come before a submodule or a skip-worktree entry, and only a look at
///  every entry tells a change a reset undoes from a state it does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtySource(args: { gitDir: string; commonDir: string; why: CleanRefusal }): DirtySource; // throws a CleanRefusal
/// ```
pub fn dirty_source(git_dir: &Path, common_dir: &Path, why: CleanRefusal) -> Result<DirtySource, CleanRefusal> {
    if !why.kind.restorable() {
        return Err(why);
    }
    let state: State = read_state(git_dir, common_dir)?;
    if let Some(refusal) = entry_a_reset_cannot_undo(&state) {
        return Err(refusal);
    }
    let read: Vec<u8> = state.bytes.clone();
    return Ok(DirtySource { source: state.into_source(read), why });
}

#[cfg(test)]
#[path = "worktree_clean_tests.rs"]
mod tests;
