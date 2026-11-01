//! What:
//!  The capture under capture order:
//!       the per-worktree capture lock,
//!       the store identity,
//!       the sequence allocation and its durable write,
//!       the listing of the paths the capture read from the worktree,
//!       and `captured.json`.
//! Why:
//!  Every commit transaction captures its content while holding the short per-worktree capture lock
//!  and takes the next sequence number under it,
//!  so sequence numbers give one total order of the captured disk states of a worktree.
//!  The incumbent and this wrapper share the store,
//!  so the lock, the identity and the write order must match exactly
//!  (`src/policy-engine/commit-capture-order-store.ts`,
//!  `commit-capture-order-capture.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stamp, value } = await captureInOrder({ gitDir, capture });
//! ```

/// Store layout names, the sequence reader and the store read.
use super::capture_store::{
    CAPTURE_LOCK_NAME, CaptureError, SEQUENCE_FILENAME, WORKTREE_ID_FILENAME,
    ensure_capture_store, read_next_capture_sequence, read_store_file,
};
/// Record encoding and the `captured.json` name.
use super::capture_records::{CapturedRecord, encode_captured_record};
/// Exclusive writes, durable and not, and directory sync.
use super::private_storage::{
    sync_directory, write_exclusive_unsynced, write_private_file,
};
/// Owner lock acquisition, its poll delay, its silent wait notice and its failure.
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::OwnerLockError;
/// Random identity bytes formatted as a UUID.
use super::random_id::random_uuid;
/// One Git start and its failure text.
use super::transaction_phase::{Phase, PhaseMarkerError, reach_transaction_phase};
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// Journal enums: the commit selection mode and the schema constant.
use super::transaction_journal::TransactionMode;
/// Trim with JavaScript's whitespace rules.
use super::js_text::trim_javascript;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// Sorted byte-keyed map of the staged entries.
use std::collections::{BTreeMap, BTreeSet};
/// `OsStr` is the operating-system text type used by process arguments.
use std::ffi::OsStr;

/// Name of the captured record inside a transaction directory.
pub const CAPTURED_FILENAME: &str = "captured.json";

/// Failure of a capture under capture order.
///  Each variant keeps the incumbent's wording,
///  so a diagnostic reads the same from either wrapper.
#[derive(Debug)]
pub enum CaptureOrderError {
    /// The store, its identity or its sequence file is unusable.
    Store(CaptureError),
    /// The capture lock could not be taken.
    Lock(OwnerLockError),
    /// A Git read of the capture failed.
    Git(GitFailure),
    /// The transaction directory could not receive `captured.json`.
    Write(String),
    /// The test-only phase marker variable is malformed or its marker could not be written.
    Phase(PhaseMarkerError),
}

impl From<CaptureError> for CaptureOrderError {
    /// Wraps a store failure.
    fn from(error: CaptureError) -> Self {
        return Self::Store(error);
    }
}

impl From<GitFailure> for CaptureOrderError {
    /// Wraps a Git failure.
    fn from(error: GitFailure) -> Self {
        return Self::Git(error);
    }
}

impl std::fmt::Display for CaptureOrderError {
    /// Writes the wording of the wrapped failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Store(error) => formatter.write_str(error.0.as_str()),
            Self::Lock(error) => write!(formatter, "{error}"),
            Self::Git(error) => write!(formatter, "{error}"),
            Self::Write(text) => formatter.write_str(text.as_str()),
            Self::Phase(error) => write!(formatter, "{error}"),
        };
    }
}

/// What:
///  Position of one capture in its worktree's capture order.
/// Why:
///   Sequence numbers are only comparable within one store generation,
///  so the identity travels with the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CaptureStamp = { worktreeId: string; sequence: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureStamp {
    /// Identity of the store that allocated the number.
    pub worktree_id: String,
    /// Capture sequence number, starting at 1.
    pub sequence: i64,
}

/// What:
///  Reads the store identity, creating it on the first capture.
/// Why:
///   A deleted store starts again with a new identity,
///  so numbers of different generations are never compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureWorktreeId(store);
/// ```
fn ensure_worktree_id(store: &Path) -> Result<String, CaptureError> {
    let path: PathBuf = store.join(WORKTREE_ID_FILENAME);
    if let Some(text) = read_store_file(path.as_path())? {
        let recorded: &str = trim_javascript(text.as_str());
        if recorded.is_empty() {
            return Err(CaptureError(format!(
                "Capture store worktree identity is empty: {}",
                path.display()
            )));
        }
        return Ok(String::from(recorded));
    }
    let identity: String = random_uuid().map_err(|error| { return CaptureError(error.0); })?;
    write_private_file(path.as_path(), format!("{identity}\n").as_bytes())
        .map_err(|error| { return CaptureError(format!("writing {} failed: {error}", path.display())); })?;
    sync_directory(store)
        .map_err(|error| { return CaptureError(format!("syncing {} failed: {error}", store.display())); })?;
    return Ok(identity);
}

/// What:
///  Writes the sequence file through a temporary file and a rename,
///  so a reader sees the old number or the new one.
/// Why:
///   A crash after the rename leaves only a gap in the numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePrivateFile({ path: tmp, bytes }); await rename(tmp, join(store, 'sequence'));
/// ```
pub fn write_sequence(store: &Path, sequence: i64) -> Result<(), CaptureError> {
    let temporary: PathBuf = store.join(format!(
        "{SEQUENCE_FILENAME}.{}.tmp",
        random_uuid().map_err(|error| { return CaptureError(error.0); })?
    ));
    let written: Result<(), CaptureError> = write_private_file(
        temporary.as_path(),
        format!("{sequence}\n").as_bytes(),
    )
    .map_err(|error| { return CaptureError(format!("writing {} failed: {error}", temporary.display())); })
    .and_then(|()| {
        return std::fs::rename(&temporary, store.join(SEQUENCE_FILENAME)).map_err(|error| {
            return CaptureError(format!("renaming {} failed: {error}", temporary.display()));
        });
    });
    if let Err(error) = written {
        // Removing the temporary file is best effort; the original failure is the answer.
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    return sync_directory(store)
        .map_err(|error| { return CaptureError(format!("syncing {} failed: {error}", store.display())); });
}

/// What:
///  Allocates the next sequence number durably.
/// Why:
///   The caller holds the capture lock,
///  so the read and the write cannot race another capture of this worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sequence = await readNextCaptureSequence(gitDir); await writeSequence(store, sequence);
/// ```
fn allocate_sequence(git_dir: &Path, store: &Path) -> Result<i64, CaptureError> {
    let sequence: i64 = read_next_capture_sequence(git_dir)?;
    write_sequence(store, sequence)?;
    return Ok(sequence);
}

/// What:
///  Runs one capture under the per-worktree capture lock and stamps it with the next sequence number.
/// Why:
///   The lock is held only for the capture itself,
///  never across hooks, the editor or landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stamp, value } = await captureInOrder({ gitDir, capture });
/// ```
pub fn capture_in_order<Value, Failure: From<CaptureOrderError>>(
    git_dir: &Path,
    capture: impl FnOnce() -> Result<Value, Failure>,
) -> Result<(CaptureStamp, Value), Failure> {
    let store: PathBuf = ensure_capture_store(git_dir).map_err(CaptureOrderError::from)?;
    // Released when this function returns, through the owner lock's drop.
    let lock = acquire_live_owner_lock(store.join(CAPTURE_LOCK_NAME).as_path(), "the capture lock")
        .map_err(CaptureOrderError::Lock)?;
    // The test-only marker of the incumbent's `captureInOrder`: reached once the capture lock is held.
    reach_transaction_phase(Phase::CaptureLocked, None).map_err(CaptureOrderError::Phase)?;
    let worktree_id: String = ensure_worktree_id(store.as_path()).map_err(CaptureOrderError::from)?;
    let sequence: i64 = allocate_sequence(git_dir, store.as_path()).map_err(CaptureOrderError::from)?;
    let value: Value = capture()?;
    drop(lock);
    return Ok((
        CaptureStamp {
            worktree_id,
            sequence,
        },
        value,
    ));
}

/// What:
///  Writes `captured.json` exclusively and whole, without syncing it.
/// Why:
///   A transaction directory holds one captured record,
///  and a second write means a bug, so the exclusive open refuses it.
///  A torn or lost record only costs a warning and subsumption in later replays (the tearing table in
///  `doc/planning/cli-git-spawn-and-sync-reduction.md`), so no flush is worth waiting for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeCapturedRecord({ directory, record });
/// ```
pub fn write_captured_record(directory: &Path, record: &CapturedRecord) -> Result<(), CaptureOrderError> {
    // Unsynced: a torn or lost record costs a warning and subsumption in later replays, not the landing.
    return write_exclusive_unsynced(
        directory.join(CAPTURED_FILENAME).as_path(),
        encode_captured_record(record).as_bytes(),
    )
    .map_err(|error| {
        return CaptureOrderError::Write(format!(
            "writing {} failed: {error}",
            directory.join(CAPTURED_FILENAME).display()
        ));
    });
}

/// What:
///  Decodes a Git path byte-for-byte as Latin-1,
///  the incumbent's text for paths in JSON records.
/// Why:
///   Latin-1 maps each byte to one code point,
///  so sorting the text sorts the raw bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeLatin1(bytes)
/// ```
fn latin1(bytes: &[u8]) -> String {
    return bytes.iter().map(|byte| { return char::from(*byte); }).collect();
}

/// What:
///  Splits NUL-terminated Git output into its nonempty records.
/// Why:
///   `-z` output ends every record with NUL and never contains an empty record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.split('\0').filter((record) => record !== '')
/// ```
fn nul_records(output: &[u8]) -> Vec<&[u8]> {
    return output
        .split(|byte| { return *byte == 0; })
        .filter(|record| { return !record.is_empty(); })
        .collect();
}

/// What:
///  The index and object-store inputs of one listing.
/// Why:
///   Staging and listing read the private index,
///  and only staging writes objects into the shadow store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Workspace = Pick<CommitTransactionWorkspace, 'capturedIndexPath' | 'commitIndexPath' | 'objectDirectory'>;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureIndexes {
    /// Real index copied at invocation.
    pub captured_index: PathBuf,
    /// Private commit index.
    pub commit_index: PathBuf,
    /// Shadow object store.
    pub object_directory: PathBuf,
}

/// What:
///  A Git request that reads or writes one private index,
///  ignoring the invocation's global options as the incumbent does.
/// Why:
///   `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY` select the private state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args })
/// ```
pub fn indexed_request<S: AsRef<OsStr>>(
    cwd: &Path,
    arguments: &[S],
    index: Option<&Path>,
    object_directory: Option<&Path>,
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, arguments);
    request.without_prefix = true;
    request.index = index.map(Path::to_path_buf);
    request.object_directory = object_directory.map(Path::to_path_buf);
    return request;
}

/// What:
///  Reads an index as `<mode> <object> <stage>` per path, stages joined by `;`.
/// Why:
///   Comparing two indexes needs each path's staged state,
///  including every conflict stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stagedEntries({ gitPath, cwd, indexPath });
/// ```
fn staged_entries(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
) -> Result<BTreeMap<Vec<u8>, String>, GitFailure> {
    let output: GitOutput = run_git_checked(
        context,
        &indexed_request(cwd, &["ls-files", "--stage", "-z"], Some(index), None),
    )?;
    let mut entries: BTreeMap<Vec<u8>, String> = BTreeMap::new();
    for record in nul_records(output.stdout.as_slice()) {
        // Git always separates the entry from its path with a tab, so a missing tab cannot come from Git.
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            return Err(GitFailure(String::from("git ls-files --stage returned an unparsable record.")));
        };
        let staged: &mut String = entries.entry(record[tab + 1..].to_vec()).or_default();
        staged.push_str(latin1(&record[..tab]).as_str());
        staged.push(';');
    }
    return Ok(entries);
}

/// What:
///  Lists the paths the capture read from the worktree,
///  in the incumbent's order.
/// Why:
///   Explicit-path commits read every path the private index changes against the base;
///  index commits that stage worktree content read only the paths their staging changed;
///  any other index commit read no worktree bytes at capture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listWorktreeCapturedPaths({ gitPath, cwd, workspace, mode, stagesWorktree, baseRevision });
/// ```
pub fn list_worktree_captured_paths(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    mode: TransactionMode,
    stages_worktree: bool,
    base_revision: &str,
) -> Result<Vec<String>, GitFailure> {
    if mode == TransactionMode::ExplicitPath {
        let output: GitOutput = run_git_checked(
            context,
            &indexed_request(
                cwd,
                &[
                    "diff-index",
                    "--cached",
                    "--name-only",
                    "-z",
                    "--no-renames",
                    base_revision,
                ],
                Some(indexes.commit_index.as_path()),
                Some(indexes.object_directory.as_path()),
            ),
        )?;
        return Ok(nul_records(output.stdout.as_slice())
            .into_iter()
            .map(latin1)
            .collect());
    }
    if !stages_worktree {
        return Ok(Vec::new());
    }
    let before: BTreeMap<Vec<u8>, String> =
        staged_entries(context, cwd, indexes.captured_index.as_path())?;
    let after: BTreeMap<Vec<u8>, String> =
        staged_entries(context, cwd, indexes.commit_index.as_path())?;
    // Union of both key sets, in byte order, keeping paths whose staged state changed.
    let mut changed: BTreeSet<Vec<u8>> = BTreeSet::new();
    for path in before.keys().chain(after.keys()) {
        if before.get(path) != after.get(path) {
            changed.insert(path.clone());
        }
    }
    return Ok(changed.iter().map(|path| { return latin1(path.as_slice()); }).collect());
}

/// What:
///  Runs the capture under capture order and journals it.
/// Why:
///   The sequence is taken before the capture reads any bytes,
///  and `captured.json` is written only after the worktree listing is known.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await captureCommitContent({ gitPath, cwd, gitDir, workspace, mode, stagesWorktree, baseRevision, nextSequenceBeforeBase, capture });
/// ```
pub fn capture_commit_content(
    context: &GitContext,
    cwd: &Path,
    git_dir: &Path,
    directory: &Path,
    indexes: &CaptureIndexes,
    mode: TransactionMode,
    stages_worktree: bool,
    base_revision: &str,
    next_sequence_before_base: i64,
    capture: impl FnOnce() -> Result<(), CaptureOrderError>,
) -> Result<CapturedRecord, CaptureOrderError> {
    let (stamp, ()) = capture_in_order(git_dir, capture)?;
    let worktree_paths: Vec<String> = list_worktree_captured_paths(
        context,
        cwd,
        indexes,
        mode,
        stages_worktree,
        base_revision,
    )?;
    let record: CapturedRecord = CapturedRecord {
        worktree_id: stamp.worktree_id,
        sequence: stamp.sequence,
        next_sequence_before_base,
        worktree_paths,
    };
    write_captured_record(directory, &record)?;
    return Ok(record);
}

/// Capture-order controls stay out of the release executable.
#[cfg(test)]
#[path = "capture_order_tests.rs"]
mod tests;
