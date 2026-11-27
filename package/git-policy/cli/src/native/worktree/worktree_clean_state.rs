//! What:
//!  What a clean check reads before it judges:
//! the entries of a Git directory that mean an operation is in progress,
//! what `HEAD` names,
//! the tracked files of the index with their recorded stat fields,
//! the commit's flattened tree,
//! and the refusal each unreadable step words.
//! Why:
//!  The judgement of cleanliness and the snapshot of a worktree that is not clean start from the same reading,
//! so every shape the readers do not know is one refusal of kind `Unreadable` here
//! and the caller takes Git's own answer.
//! One open file gives both the index bytes and the time,
//! so the time belongs to the bytes that were proven.
//! A merge,
//! a rebase or a held index lock means the index is not the committed state yet,
//! and the lock a caller holds itself does not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = readState(gitDir, commonDir); // throws a CleanRefusal
//! const source = state.intoSource(indexBytes);
//! ```

use crate::repo_read_index::{
    IndexEntry,
    parse_index,
};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_refs::{
    RefValue,
    Refs,
};
use crate::repo_read_tree::{
    FlatEntry,
    flatten_tree,
    parse_commit,
};
use crate::subvolume_index::{
    StoredStat,
    be32,
    walk_entries,
};
use crate::subvolume_index_files::repository_hash_length;
use crate::worktree_clean_stat_verdicts::{
    CleanRefusal,
    StatEntry,
};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The entries of a Git directory that mean an operation is in progress and the index or tree is not a plain state.
/// Why a bisect has two of them:
///  `git bisect start` writes `BISECT_START`,
/// then `BISECT_NAMES`,
///  and opens `BISECT_LOG` only after both
///  (measured with Git 2.56 under `strace`),
///  so a process killed between those writes leaves a bisect whose mark is a file written before the kill,
///  and a list that knew only the log proved a worktree clean with that bisect still unfinished there.
const IN_PROGRESS: &[&str] = &[
    "index.lock",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "BISECT_START",
    "BISECT_LOG",
    "rebase-merge",
    "rebase-apply",
    "sequencer",
];

/// What:
///  What a clean worktree's `HEAD` names.
/// Why:
///   The two-tree update that follows a snapshot starts from this tree,
/// so a caller needs not read it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanSource = { headCommit: string; headTree: Uint8Array; tracked: number; indexSeconds: number; hashLength: number; indexBytes: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanSource {
    /// The commit `HEAD` resolves to,
    /// as hexadecimal.
    pub head_commit: String,
    /// The root tree of that commit,
    /// as a raw object name.
    pub head_tree: Vec<u8>,
    /// The number of tracked files.
    pub tracked: usize,
    /// The whole seconds of the index file's modification time,
    /// against which racy entries were judged.
    pub index_seconds: u32,
    /// The hash length of the repository in bytes.
    pub hash_length: usize,
    /// The `.gitattributes` files of the tree of `HEAD` as path and object name.
    pub attribute_blobs: Vec<(Vec<u8>, Vec<u8>)>,
    /// The bytes of the index file that were proven equal to the tree of `HEAD` and to the files.
    /// Every later step uses
    ///  these bytes,
    /// never a second read of the file,
    /// so what was proven is what is installed.
    pub index_bytes: Vec<u8>,
}

/// What:
///  A refusal that the index or the Git data could not be read,
/// when `found` holds nothing.
/// Why:
///   Every read of the repository's own files refuses in the same way,
/// saying which read failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// found ?? throw unreadable(reason)
/// ```
fn or_unreadable<T>(
    found: Option<T>,
    reason: &'static str,
) -> Result<T, CleanRefusal> {
    return match found {
        Some(value) => Ok(value),
        None => Err(CleanRefusal::unreadable(reason)),
    };
}

/// What:
///  A refusal that the index file could not be opened,
/// examined or read.
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
///  The first entry of the Git directory that marks an operation in progress,
/// if any.
/// Why:
///   A merge,
/// a rebase or a held index lock means the index is not the committed state yet;
/// the lock a caller holds itself
///  (`owned_index_lock`) does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function operationInProgress(gitDir: string): string | undefined;
/// ```
pub(crate) fn operation_in_progress(
    git_dir: &Path,
    owned_index_lock: bool,
) -> Option<&'static str> {
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
///  The commit `HEAD` resolves to,
/// read from the plain ref files.
/// Why:
///   An unusual ref storage answers "unknown",
/// which declines the shortcut rather than guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headCommit(gitDir: string, commonDir: string, format: ObjectFormat): string | undefined;
/// ```
pub fn head_commit(
    git_dir: &Path,
    common_dir: &Path,
    format: ObjectFormat,
) -> Option<String> {
    let refs: Refs = Refs::new(common_dir, format.hex_length());
    return match refs.lookup_head(git_dir)? {
        RefValue::Direct(object) => Some(object),
        RefValue::Symbolic(name) => refs.resolve(name.as_str()),
    };
}

/// What:
///  The tracked files of the index paired with their recorded stat fields,
/// or nothing when the layout is not plain.
/// Why:
///   The comparison with `lstat` needs the fields `parse_index` leaves out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function storedStats(bytes: Uint8Array, hashLength: number): { path: Uint8Array; flags: number; mode: number; stored: StoredStat }[] | undefined;
/// ```
pub fn stored_stats(
    bytes: &[u8],
    hash_length: usize,
) -> Option<Vec<StatEntry>> {
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
        found.push(StatEntry {
            path: entry.path,
            flags: entry.flags,
            mode: entry.mode,
            stored,
        });
    }
    return Some(found);
}

/// What the checks read before they judge:
/// the index with its stat data,
/// the commit `HEAD` names and its flattened tree.
pub(crate) struct State {
    /// The bytes of the index file.
    pub(crate) bytes: Vec<u8>,
    /// The whole seconds of the index file's modification time.
    pub(crate) index_seconds: u32,
    /// The hash length of the repository in bytes.
    hash_length: usize,
    /// The object format.
    pub(crate) format: ObjectFormat,
    /// The parsed index entries.
    pub(crate) entries: Vec<IndexEntry>,
    /// The stat fields of the entries.
    pub(crate) stats: Vec<StatEntry>,
    /// The commit `HEAD` names.
    commit: String,
    /// The root tree of that commit.
    tree_id: Vec<u8>,
    /// The flattened tree.
    pub(crate) tree: Vec<FlatEntry>,
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

    /// What `HEAD` names,
    /// with the index bytes the caller proved (or none,
    /// for a worktree that is not clean).
    pub(crate) fn into_source(
        self,
        index_bytes: Vec<u8>,
    ) -> CleanSource {
        let attribute_blobs: Vec<(Vec<u8>, Vec<u8>)> = self.attribute_blobs();
        return CleanSource {
            head_commit: self.commit,
            head_tree: self.tree_id,
            tracked: self.entries.len(),
            index_seconds: self.index_seconds,
            hash_length: self.hash_length,
            attribute_blobs,
            index_bytes,
        };
    }
}

/// What:
///  Read the index and the tree of `HEAD` of a worktree,
/// in process,
///  from the repository's own store.
/// Why:
///   The judgement of cleanliness and the snapshot of a worktree that is not clean start from the same reading;
/// every shape
///  the readers do not know is a refusal of kind `Unreadable`.
///  The question this answers is about this worktree's own files and its own index,
/// so it reads the repository's own store,
/// as `main`'s reader did,
///  and a store or a base a command's environment names does not move that reading:
/// the objects it asks for are this repository's,
/// and their names are their bytes.
/// Where the repository's own store lacks them,
/// the proof refuses as `main` refused,
/// and the caller takes Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readState(gitDir: string, commonDir: string): State; // throws a CleanRefusal
/// ```
pub(crate) fn read_state(
    git_dir: &Path,
    common_dir: &Path,
) -> Result<State, CleanRefusal> {
    let hash_length: usize = or_unreadable(
        repository_hash_length(common_dir),
        "the repository configuration is unreadable",
    )?;
    let format: ObjectFormat = if hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    // One open file gives both the bytes and the time, so the time belongs to the bytes that were proven.
    let index_path = git_dir.join("index");
    let mut index_file: std::fs::File =
        std::fs::File::open(index_path.as_path()).map_err(index_unreadable)?;
    let index_seconds: u32 = index_file.metadata().map_err(index_unreadable)?.mtime() as u32;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut index_file, &mut bytes).map_err(index_unreadable)?;
    let entries: Vec<IndexEntry> = or_unreadable(
        parse_index(bytes.as_slice(), format),
        "the index has a layout the reader does not know",
    )?;
    let stats = or_unreadable(
        stored_stats(bytes.as_slice(), hash_length),
        "the index entries have no plain layout",
    )?;
    let commit: String = or_unreadable(
        head_commit(git_dir, common_dir, format),
        "HEAD is not a plain ref to a commit",
    )?;
    let opened: Option<ObjectDatabase> =
        ObjectDatabase::open_of_the_repository_alone(common_dir, None, format);
    let database: ObjectDatabase = or_unreadable(
        opened,
        "the object database has a layout the reader does not know",
    )?;
    let commit_name: Vec<u8> = or_unreadable(
        crate::repo_read_object::from_hex(commit.as_bytes(), format),
        "HEAD names no object",
    )?;
    let commit_object = or_unreadable(
        database.read(commit_name.as_slice()),
        "the commit of HEAD is unreadable",
    )?;
    let commit_fields = or_unreadable(
        parse_commit(commit_object.bytes.as_slice(), format),
        "the commit of HEAD has a layout the reader does not know",
    )?;
    let tree: Vec<FlatEntry> = or_unreadable(
        flatten_tree(&database, commit_fields.tree.as_slice(), format),
        "the tree of HEAD is unreadable",
    )?;
    return Ok(State {
        bytes,
        index_seconds,
        hash_length,
        format,
        entries,
        stats,
        commit,
        tree_id: commit_fields.tree,
        tree,
    });
}
