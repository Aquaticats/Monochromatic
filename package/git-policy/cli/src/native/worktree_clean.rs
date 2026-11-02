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

use super::repo_read_index::{IndexEntry, parse_index};
use super::repo_read_layout::ObjectFormat;
use super::repo_read_odb::ObjectDatabase;
use super::repo_read_refs::{Refs, RefValue};
use super::repo_read_tree::{FlatEntry, MODE_GITLINK, flatten_tree, parse_commit};
use super::subvolume_index::{StatFields, StoredStat, be32, walk_entries};
use super::subvolume_index_files::{matches_stored, repository_hash_length};
use super::transaction_git::{GitContext, GitRequest, run_git};
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
    /// The bytes of the index file that were proven equal to the tree of `HEAD` and to the files. Every later step uses
    ///  these bytes, never a second read of the file, so what was proven is what is installed.
    pub index_bytes: Vec<u8>,
}

/// What:
///  The first entry of the Git directory that marks an operation in progress, if any.
/// Why:
///   A merge, a rebase or a held index lock means the index is not the committed state yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function operationInProgress(gitDir: string): string | undefined;
/// ```
fn operation_in_progress(git_dir: &Path) -> Option<&'static str> {
    return IN_PROGRESS.iter().copied().find(|name: &&str| { return git_dir.join(name).symlink_metadata().is_ok(); });
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
fn index_differs_from_tree(index: &[IndexEntry], tree: &[FlatEntry]) -> Option<String> {
    if index.len() != tree.len() {
        return Some(String::from("the index and the tree of HEAD hold different numbers of files"));
    }
    for (staged, committed) in index.iter().zip(tree) {
        if staged.stage != 0 || staged.extended != 0 {
            return Some(format!("the index entry {} is unmerged, intent-to-add or skip-worktree", String::from_utf8_lossy(&staged.path)));
        }
        if staged.mode == MODE_GITLINK {
            return Some(format!("{} is a submodule", String::from_utf8_lossy(&staged.path)));
        }
        if staged.path != committed.path || staged.mode != committed.mode || staged.id != committed.id {
            return Some(format!("{} is staged with a change", String::from_utf8_lossy(&staged.path)));
        }
    }
    return None;
}

/// What:
///  The reason a tracked file differs from its index entry, or nothing when `lstat` agrees with the entry.
/// Why:
///   This is Git's fast path: equal size, times, inode, owner and mode mean Git would not look at the content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileDiffersFromEntry(root: string, entry: StatEntry, indexSeconds: number): string | undefined;
/// ```
fn file_differs_from_entry(root: &Path, entry: &StatEntry, index_seconds: u32) -> Option<String> {
    let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
    let Ok(metadata) = std::fs::symlink_metadata(root.join(OsStr::from_bytes(entry.path.as_slice()))) else {
        return Some(format!("{shown} is missing from the worktree"));
    };
    return metadata_differs_from_entry(entry, &metadata, index_seconds);
}

/// What:
///  The reason a file whose metadata is `metadata` differs from its index entry, or nothing when the two agree.
/// Why:
///   The comparison takes the metadata of an inode already in hand, so a caller that holds a descriptor judges the file it
///  holds, not whatever the path names a moment later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function metadataDiffersFromEntry(entry: StatEntry, metadata: Stats, indexSeconds: number): string | undefined;
/// ```
pub fn metadata_differs_from_entry(entry: &StatEntry, metadata: &std::fs::Metadata, index_seconds: u32) -> Option<String> {
    let StatEntry { path, flags, mode, stored }: &StatEntry = entry;
    let shown: String = String::from_utf8_lossy(path).into_owned();
    if flags & ASSUME_VALID != 0 {
        return Some(format!("{shown} is marked assume-unchanged"));
    }
    if stored.mtime_seconds >= index_seconds {
        return Some(format!("{shown} is as new as the index, so its stat fields prove nothing"));
    }
    let symbolic: bool = *mode & 0o170_000 == 0o120_000;
    if symbolic != metadata.file_type().is_symlink() || (!symbolic && !metadata.file_type().is_file()) {
        return Some(format!("{shown} changed its type"));
    }
    if !symbolic && (*mode & 0o100 != 0) != (metadata.mode() & 0o100 != 0) {
        return Some(format!("{shown} changed its executable bit"));
    }
    let now: StatFields = StatFields {
        ctime_seconds: metadata.ctime() as u32,
        ctime_nanos: metadata.ctime_nsec() as u32,
        mtime_seconds: metadata.mtime() as u32,
        mtime_nanos: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
    if !matches_stored(&now, stored) {
        return Some(format!("{shown} differs from its index entry by size, times or inode"));
    }
    return None;
}

/// What:
///  The first reason a tracked file of `root` differs from its index entry, or nothing when every file agrees.
/// Why:
///   After a snapshot the same comparison runs against the snapshot's own files, so a source that changed while the
///  snapshot was taken is caught on the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndex(root: string, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index(root: &Path, entries: &[StatEntry], index_seconds: u32) -> Option<String> {
    return entries.iter().find_map(|entry: &StatEntry| { return file_differs_from_entry(root, entry, index_seconds); });
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
    use super::fs_beneath::{entry_metadata_at, open_directory_beneath, plain_components};
    let mut cached: Option<(Vec<std::ffi::OsString>, std::fs::File)> = None;
    for entry in entries {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice()))) else { return Some(format!("{shown} is not a plain path")) };
        let Some((name, parents)) = names.split_last() else { return Some(format!("{shown} is not a plain path")) };
        if cached.as_ref().is_none_or(|(held, _directory): &(Vec<std::ffi::OsString>, std::fs::File)| { return held.as_slice() != parents; }) {
            let Ok(directory) = open_directory_beneath(root, parents) else { return Some(format!("{shown} is missing from the worktree or reached through a link")) };
            cached = Some((parents.to_vec(), directory));
        }
        let Some((_held, directory)) = cached.as_ref() else { return Some(format!("{shown} is missing from the worktree")) };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else { return Some(format!("{shown} is missing from the worktree")) };
        if let Some(reason) = metadata_differs_from_entry(entry, &metadata, index_seconds) {
            return Some(reason);
        }
    }
    return None;
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
fn untracked_file(context: &GitContext, root: &Path) -> Result<Option<String>, String> {
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["ls-files", "--others", "--exclude-standard", "--directory", "--no-empty-directory", "-z"].map(Into::into).to_vec(),
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = run_git(context, &request).map_err(|error| { return format!("Git could not list untracked files ({error})"); })?;
    if output.code != Some(0) {
        return Err(format!("Git could not list untracked files: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim()));
    }
    return Ok(output.stdout.split(|byte: &u8| { return *byte == 0; }).find(|name: &&[u8]| { return !name.is_empty(); }).map(|name: &[u8]| { return String::from_utf8_lossy(name).into_owned(); }));
}

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
pub fn check_clean(context: &GitContext, root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, String> {
    if let Some(name) = operation_in_progress(git_dir) {
        return Err(format!("{name} exists in {}", git_dir.display()));
    }
    let hash_length: usize = repository_hash_length(common_dir).ok_or_else(|| { return String::from("the repository configuration is unreadable"); })?;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    // One open file gives both the bytes and the time, so the time belongs to the bytes that were proven.
    let index_path = git_dir.join("index");
    let mut index_file: std::fs::File = std::fs::File::open(index_path.as_path()).map_err(|error| { return format!("the index is unreadable ({error})"); })?;
    let index_seconds: u32 = index_file.metadata().map_err(|error| { return format!("the index is unreadable ({error})"); })?.mtime() as u32;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut index_file, &mut bytes).map_err(|error| { return format!("the index is unreadable ({error})"); })?;
    let entries: Vec<IndexEntry> = parse_index(bytes.as_slice(), format).ok_or_else(|| { return String::from("the index has a layout the reader does not know"); })?;
    let stats = stored_stats(bytes.as_slice(), hash_length).ok_or_else(|| { return String::from("the index entries have no plain layout"); })?;
    let commit: String = head_commit(git_dir, common_dir, format).ok_or_else(|| { return String::from("HEAD is not a plain ref to a commit"); })?;
    let database: ObjectDatabase = ObjectDatabase::open(common_dir, None, format).ok_or_else(|| { return String::from("the object database has a layout the reader does not know"); })?;
    let commit_name: Vec<u8> = super::repo_read_object::from_hex(commit.as_bytes(), format).ok_or_else(|| { return String::from("HEAD names no object"); })?;
    let commit_fields = parse_commit(database.read(commit_name.as_slice()).ok_or_else(|| { return String::from("the commit of HEAD is unreadable"); })?.bytes.as_slice(), format)
        .ok_or_else(|| { return String::from("the commit of HEAD has a layout the reader does not know"); })?;
    let tree: Vec<FlatEntry> = flatten_tree(&database, commit_fields.tree.as_slice(), format).ok_or_else(|| { return String::from("the tree of HEAD is unreadable"); })?;
    if let Some(reason) = index_differs_from_tree(entries.as_slice(), tree.as_slice()) {
        return Err(reason);
    }
    if let Some(reason) = tree_differs_from_index(root, stats.as_slice(), index_seconds) {
        return Err(reason);
    }
    if let Some(name) = untracked_file(context, root)? {
        return Err(format!("{name} is untracked and not ignored"));
    }
    return Ok(CleanSource { head_commit: commit, head_tree: commit_fields.tree, tracked: entries.len(), index_seconds, hash_length, index_bytes: bytes });
}

/// Cleanliness controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clean_tests.rs"]
mod tests;
