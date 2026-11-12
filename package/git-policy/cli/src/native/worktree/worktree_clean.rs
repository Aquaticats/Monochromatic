//! What:
//!  Whether a worktree is clean enough to be copied from,
//! or removed without asking Git:
//! no operation in progress,
//! an
//!  index equal to the tree of `HEAD`,
//! every tracked file unchanged by its recorded size,
//! times and inode,
//! and no
//!  untracked file that Git does not ignore.
//! Why:
//!  A snapshot or a clone carries the bytes of the source's files,
//! so any change the source has not committed would
//!  reach the new worktree,
//! and a removal by the subvolume ioctl cannot be undone.
//! The tracked-file checks read the
//!  index in process (no Git process) with the same comparison Git's fast path makes,
//! and refuse every shape they cannot
//!  prove:
//! a split or sparse index,
//! a skip-worktree or assume-valid entry,
//! a submodule,
//! an entry as new as the index
//!  file (Git re-hashes those,
//! so their stat fields prove nothing).
//! The one question the index cannot answer,
//! whether
//!  an untracked file is ignored,
//! goes to one `git ls-files` run.
//! Every refusal returns its reason in words for the log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clean = await checkClean({ root, gitDir, commonDir }); // { headCommit, headTree } or throws a reason
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::lazy_runtime::run_on_runtime;
use crate::repo_read_index::{IndexEntry, parse_index};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_refs::{Refs, RefValue};
use crate::repo_read_tree::{FlatEntry, MODE_GITLINK, flatten_tree, parse_commit};
use crate::worktree_clean_content::ConversionEnvironment;
pub use crate::worktree_clean_stat_verdicts::{CleanRefusal, CleanRefusalKind, StatEntry, StatVerdict, metadata_differs_from_entry, metadata_verdict};
use crate::worktree_clean_stat_verdicts::{ASSUME_VALID, file_changed};
pub use crate::worktree_clean_tree_walk::{ContentTarget, StatCursor, paths_whose_content_differs, stat_fields_beneath, tree_differs_from_index, tree_differs_from_index_beneath, tree_fields_by_content};
use crate::worktree_clean_tree_walk::{tree_differs_from_index_by_content, tree_refusal};
use crate::subvolume_index::{StoredStat, be32, walk_entries};
use crate::subvolume_index_files::repository_hash_length;
use crate::transaction_git::GitContext;
use crate::worktree_clean_untracked_scan::{scan_for_untracked_file, scan_for_untracked_file_awaiting};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The entries of a Git directory that mean an operation is in progress and the index or tree is not a plain state.
const IN_PROGRESS: &[&str] =
    &["index.lock", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "BISECT_LOG", "rebase-merge", "rebase-apply", "sequencer"];

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
fn or_unreadable<T>(found: Option<T>, reason: &'static str) -> Result<T, CleanRefusal> {
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
pub fn head_commit(git_dir: &Path, common_dir: &Path, format: ObjectFormat) -> Option<String> {
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
///  The reason the index differs from the tree of `HEAD`,
/// or nothing when the two are the same files.
/// Why:
///   A staged change,
/// a staged deletion or an intent-to-add entry is an uncommitted state the copy must not carry.
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
///  The checks of `check_clean` that read no untracked file:
/// no operation in progress,
/// an index equal to the tree of `HEAD`,
///  and every tracked file with the size,
/// times and inode its index entry recorded.
/// Why:
///   A caller that copies the whole worktree,
/// untracked files included (the conversion of a worktree into a subvolume),
///  needs a quiet and unmodified tracked state but has no reason to refuse for an untracked file,
/// and needs no Git process.
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
///  blob in process,
/// under the configuration and attribute locations of `environment`.
/// Why:
///   The conversion copies the whole worktree and rewrites the index afterwards,
/// and an entry it could not prove keeps its old
///  stat data,
/// so Git hashes it at the first status;
/// the snapshot rung settles the entries of the copy the same way before it
///  installs the index (`worktree_snapshot.rs`).
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

/// What the checks read before they judge:
/// the index with its stat data,
/// the commit `HEAD` names and its flattened tree.
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

    /// What `HEAD` names,
    /// with the index bytes the caller proved (or none,
    /// for a worktree that is not clean).
    fn into_source(self, index_bytes: Vec<u8>) -> CleanSource {
        let attribute_blobs: Vec<(Vec<u8>, Vec<u8>)> = self.attribute_blobs();
        return CleanSource { head_commit: self.commit, head_tree: self.tree_id, tracked: self.entries.len(), index_seconds: self.index_seconds, hash_length: self.hash_length, attribute_blobs, index_bytes };
    }
}

/// What:
///  Read the index and the tree of `HEAD` of a worktree,
/// in process.
/// Why:
///   The judgement of cleanliness and the snapshot of a worktree that is not clean start from the same reading;
/// every shape
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

/// The checks of `check_clean_tracked`,
/// by stat data alone or,
/// given a `scope`,
/// by content where the stat data is not enough.
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
///  The first entry of a worktree's index or tree that a reset to a commit cannot be proven to undo,
/// if any:
/// unmerged,
///  intent-to-add and skip-worktree entries,
/// assume-unchanged entries,
/// and submodules in the index or in the tree of `HEAD`.
/// Why:
///   The first refusal of the clean check can be an ordinary change that hides one of these further on;
/// a restore needs all of
///  the index looked at,
/// not the first offender.
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

/// What:
///  The tracked-file proof of `check_clean`,
/// which runs as a job of the blocking pool while the untracked scan is awaited.
/// Why:
///   The proof reads the index,
/// the tree of `HEAD` and a stat of every tracked file,
/// none of which is a question
///  for a task that also awaits a bounded Git child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTrackedFiles([root, gitDir, commonDir]: [string, string, string]): CleanSource;
/// ```
fn check_tracked_files((root, git_dir, common_dir): (&Path, &Path, &Path)) -> Result<CleanSource, CleanRefusal> {
    return check_tracked(root, git_dir, common_dir, None, false);
}

/// What:
///  `check_tracked_files` for a job of the blocking pool,
/// which owns the paths it reads.
/// Why:
///   A job of the pool must own everything it reads,
/// and the caller's paths live on a stack that the job outlives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTrackedFilesInAJob([root, gitDir, commonDir]: [string, string, string]): CleanSource; // throws the reason
/// ```
fn check_tracked_files_in_a_job((root, git_dir, common_dir): (PathBuf, PathBuf, PathBuf)) -> Result<CleanSource, CleanRefusal> {
    return check_tracked_files((root.as_path(), git_dir.as_path(), common_dir.as_path()));
}

/// What:
///  `check_tracked_files_in_a_job` that settles every entry the stat data cannot vouch for by comparing the file with
///  the committed blob,
/// under the configuration and attributes of `context`.
/// Why:
///   A worktree that was checked out in the same instant its index was written records stat data for every file that
/// proves nothing (`worktree_clean_stat_verdicts.rs`,
/// `Racy`),
/// and the snapshot rung and the conversion settle exactly that state by content.
/// A removal that wants Git's own answer for such a worktree asks the same question the same way,
/// so it costs one hash of the files stat cannot vouch for instead of Git's file-by-file walk of the whole tree.
/// The environment is read inside the job,
/// because the facts it holds come from a Git start and the blocking pool is where a Git start belongs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTrackedFilesByContentInAJob([root, gitDir, commonDir, context]): CleanSource; // throws the reason
/// ```
fn check_tracked_files_by_content_in_a_job((root, git_dir, common_dir, context): (PathBuf, PathBuf, PathBuf, GitContext)) -> Result<CleanSource, CleanRefusal> {
    let environment: ConversionEnvironment = crate::worktree_clean_content_attribute_sources::for_context(&context);
    return check_tracked(root.as_path(), git_dir.as_path(), common_dir.as_path(), Some(&environment), false);
}

/// What:
///  The clean check of a worktree as Git itself would answer it about the tracked files:
/// the same checks as `check_clean`,
/// except that a file the recorded stat data cannot vouch for is read and compared with the blob its entry names.
/// Why:
///   A freshly added worktree is exactly the state the stat-only check refuses,
/// and a removal that refused it handed the whole tree to Git's file-by-file walk,
/// where anything still writing into the tree leaves a directory Git cannot empty
/// (`SPEC.md`,
/// "Cheap worktree creation and removal on btrfs,
/// XFS and overlayfs").
/// Every other refusal stays a refusal:
/// only the two stat verdicts a content comparison settles are read,
/// and a file whose bytes differ is refused with the reason the comparison gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkCleanByContentAwaiting({ context, root, gitDir, commonDir }): Promise<CleanSource>; // throws the reason
/// ```
pub async fn check_clean_by_content_awaiting(context: &GitContext, root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, CleanRefusal> {
    let (untracked, tracked): (Result<Option<String>, CleanRefusal>, Result<Result<CleanSource, CleanRefusal>, BlockingJobError>) = tokio::join!(
        scan_for_untracked_file_awaiting((context, root)),
        run_a_job_or_in_turn((root.to_path_buf(), git_dir.to_path_buf(), common_dir.to_path_buf(), context.clone()), check_tracked_files_by_content_in_a_job),
    );
    let proof: Result<CleanSource, CleanRefusal> = match tracked {
        Ok(verdict) => verdict,
        Err(refused) => Err(a_refused_job_is_a_refusal(refused)),
    };
    return the_clean_verdict((proof, untracked));
}

/// What:
///  `check_clean_by_content_awaiting` for a caller that is not a task of the runtime,
/// with the same answer as `check_clean` where no runtime can be built.
/// Why:
///   The removal of a worktree is a synchronous caller,
/// and it enters the runtime for this proof the way it does for the stat-only one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCleanByContent({ context, root, gitDir, commonDir }): CleanSource; // throws the reason
/// ```
pub fn check_clean_by_content(context: &GitContext, root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, CleanRefusal> {
    return match run_on_runtime(check_clean_by_content_awaiting(context, root, git_dir, common_dir)) {
        Ok(verdict) => verdict,
        Err(refusal) => {
            crate::diagnostic_log::debug("checkClean", format!("the content proof and the untracked scan run one after another ({refusal})").as_str());
            let environment: ConversionEnvironment = crate::worktree_clean_content_attribute_sources::for_context(context);
            the_clean_verdict((check_tracked(root, git_dir, common_dir, Some(&environment), false), scan_for_untracked_file((context, root))))
        }
    };
}

/// What:
///  Whether a refusal is one a comparison with the committed blob could settle,
/// which is the only state `check_clean_by_content` is asked about.
/// Why:
///   The two stat verdicts are the ones Git re-hashes;
/// every other refusal is a state no read of the file's bytes can clear,
/// so the removal keeps Git's own answer for those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settlesByContent = (kind) => kind === 'racy' || kind === 'stat-stale';
/// ```
pub fn settles_by_content(kind: CleanRefusalKind) -> bool {
    return matches!(kind, CleanRefusalKind::Racy | CleanRefusalKind::StatStale);
}

/// What:
///  The answer two verdicts make:
/// the tracked proof's refusal first,
/// then the untracked file's.
/// Why:
///   The two checks read different things and their verdicts were always read in this order,
/// so both faces of the check answer the
///  same state with the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCleanVerdict([tracked, untracked]: [CleanSource | Refusal, string | undefined | Refusal]): CleanSource;
/// ```
fn the_clean_verdict((tracked, untracked): (Result<CleanSource, CleanRefusal>, Result<Option<String>, CleanRefusal>)) -> Result<CleanSource, CleanRefusal> {
    let clean: CleanSource = tracked?;
    if let Some(name) = untracked? {
        return Err(CleanRefusal::new(CleanRefusalKind::Untracked, format!("{name} is untracked and not ignored")));
    }
    return Ok(clean);
}

/// What:
///  Keep the reason of a tracked proof that could not run as a job of the pool.
/// Why:
///   A proof that panicked proves nothing,
/// and a pool that refused the work says which state it is in;
/// the check reports
///  either as a refusal rather than unwinding a caller or claiming a verdict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRefusedJobIsARefusal(error: BlockingJobError): Refusal;
/// ```
fn a_refused_job_is_a_refusal(error: BlockingJobError) -> CleanRefusal {
    return match error {
        BlockingJobError::Panicked => CleanRefusal::unreadable("the proof of the tracked files stopped unexpectedly"),
        other => CleanRefusal::unreadable(format!("the tracked files could not be proven ({other})")),
    };
}

/// What:
///  Check that the worktree at `root`,
/// whose Git directory is `git_dir` and whose shared directory is `common_dir`,
/// is
///  clean,
/// on the runtime:
/// the untracked scan awaited on this task and the tracked proof a job of the blocking pool beside it.
/// Why:
///   The two read different things and write nothing,
/// so neither waits for the other;
/// the scan's wait is bounded by its step
///  terms and the proof walks every tracked file,
/// which must never run on the task that awaits that bounded wait.
/// A task of the runtime cannot start
///  this child through the synchronous face,
/// so the scan is the awaited one.
/// Where the pool refuses a thread,
/// the proof runs in turn under a place of the
///  process's bound (`blocking_jobs_within_the_budget.rs`,
/// `run_a_job_or_in_turn`),
/// so the answer is the same and only the overlap is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkCleanAwaiting({ context, root, gitDir, commonDir }): Promise<CleanSource>; // throws the reason
/// ```
pub async fn check_clean_awaiting(context: &GitContext, root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, CleanRefusal> {
    let (untracked, tracked): (Result<Option<String>, CleanRefusal>, Result<Result<CleanSource, CleanRefusal>, BlockingJobError>) = tokio::join!(
        scan_for_untracked_file_awaiting((context, root)),
        run_a_job_or_in_turn((root.to_path_buf(), git_dir.to_path_buf(), common_dir.to_path_buf()), check_tracked_files_in_a_job),
    );
    let proof: Result<CleanSource, CleanRefusal> = match tracked {
        Ok(verdict) => verdict,
        Err(refused) => Err(a_refused_job_is_a_refusal(refused)),
    };
    return the_clean_verdict((proof, untracked));
}

/// What:
///  `check_clean_awaiting` for a caller that is not a task of the runtime:
/// it enters the process's runtime once and waits there,
/// and where no runtime can be built the two checks run one
///  after another on this thread.
/// Why:
///   The callers of this face are synchronous (the removal of a worktree and the choice of a snapshot's source),
/// so one
///  of them enters the runtime for them;
/// from inside the runtime it refuses,
/// and `check_clean_awaiting` is the face such a caller uses.
/// The
///  work done without a runtime is the same work,
/// with nothing running beside it,
/// so every verdict is the one the runtime gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkClean(args: { context: GitContext; root: string; gitDir: string; commonDir: string }): CleanSource; // throws a reason
/// ```
pub fn check_clean(context: &GitContext, root: &Path, git_dir: &Path, common_dir: &Path) -> Result<CleanSource, CleanRefusal> {
    return match run_on_runtime(check_clean_awaiting(context, root, git_dir, common_dir)) {
        Ok(verdict) => verdict,
        Err(refusal) => {
            crate::diagnostic_log::debug("checkClean", format!("the tracked files and the untracked scan run one after another ({refusal})").as_str());
            the_clean_verdict((check_tracked_files((root, git_dir, common_dir)), scan_for_untracked_file((context, root))))
        }
    };
}

/// What:
///  What `HEAD` names for a worktree that is not clean,
/// and why it is not,
/// once it is known that the only trouble is ordinary
///  changes.
/// Why:
///   The snapshot rung copies such a worktree and restores the copy to its `HEAD`;
/// the copy's own index is installed,
/// so the
///  index bytes here are only as read:
/// they name the tracked paths and prove nothing about the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtySource = { source: CleanSource; why: CleanRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirtySource {
    /// What `HEAD` names;
    /// its `index_bytes` are the index as read,
    /// not proven equal to the files.
    pub source: CleanSource,
    /// The first reason the worktree is not clean.
    pub why: CleanRefusal,
}

/// What:
///  For a worktree `check_clean` refused with the ordinary change `why`,
/// what `HEAD` names,
/// or the refusal that a restore
///  cannot undo when the index holds one anywhere.
/// Why:
///   The first refusal hides the rest:
/// a staged edit can come before a submodule or a skip-worktree entry,
/// and only a look at
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
