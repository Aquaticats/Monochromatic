//! What:
//!  One landing attempt inside the landing critical section:
//!  the real `index.lock`,
//!  the branch-switch and head-moved checks,
//!  object migration,
//!  the post-index,
//!  `landing-<n>.json`,
//!  the compare-and-swap with the nonce reflog message,
//!  `ref-updated.json`,
//!  the index install,
//!  the `index-installed` marker and conclusion cleanup.
//! Why:
//!  A landing is serial under the landing lock and the real `index.lock`, and it journals every state
//!  so recovery can finish or discard a landing that died anywhere inside
//!  (`src/policy-engine/commit-landing.ts`).
//!  The caller takes the landing lock, checks the starvation reservation and replays a moved target;
//!  the caller's post-landing step disposes of the transaction directory.
//!  This module never runs hooks, the editor, signing, network operations or the hook lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await landTransaction({ gitPath, cwd, capture, workspace, mode, payload, committedPaths, addedPaths, selectedWorktreePaths, indexLockTimeoutMs, attempt });
//! ```

/// Recording the capture of a landed commit.
use super::capture_records::record_landed_capture_or_warn;
/// The invocation facts, and the baseline revision the post-index reads.
use super::capture_invocation::{InvocationCapture, base_revision, resolve_ref_commit, resolve_symbolic_head};
/// Where the worktree's `HEAD` file and branch files live, for the fresh read under the landing lock.
use super::ref_files::RefFilesLayout;
/// Native conclusion cleanup after the index is installed.
use super::conclusion_cleanup::reproduce_conclusion_cleanup;
/// The index copy that keeps the timestamps Git reads.
use super::commit_index::copy_index_with_timestamps;
/// Debug and warning text to the diagnostic log.
use super::diagnostic_log::{debug, warn};
/// Failure of waiting for or creating the real index lock.
use super::index_writer_coordination::CoordinationFailure;
/// The post-index computation and its inputs.
use super::landing_index::{PostIndexInput, compute_landing_post_index};
/// Object migration into a kept pack.
use super::landing_objects::migrate_shadow_objects;
/// Target comparison and the compare-and-swap.
use super::landing_support::{CompareAndSwap, compare_and_swap_target, same_target};
/// The keep message and its removal.
use super::pack_keep::{remove_pack_keep, transaction_keep_message};
/// The held landing lock, which the caller must prove it holds.
use super::owner_lock::OwnerLock;
/// Journal files are written durably.
use super::private_storage::{sync_directory, write_private_file};
/// Recovery failures that the shared steps report.
use super::recovery_error::RecoveryError;
/// File identities of the landing artifacts.
use super::recovery_files::file_identity_of;
/// The real index lock, taken for the attempt.
use super::real_index_lock::{RealIndexLock, acquire_landing_index_lock};
/// Running real Git.
use super::transaction_git::{GitContext, GitFailure};
/// Journal records, names and filenames.
use super::transaction_journal::{
    AddedPath, Base, Conclusion, FileIdentity, INDEX_INSTALLED_FILENAME, LandingOperation, LandingRecord,
    REF_UPDATED_FILENAME, SymbolicHead, TransactionMode, landing_record_filename, post_index_filename,
    pre_landing_index_filename,
};
/// Record encoders.
use super::transaction_journal_encode::{encode_landing, encode_ref_updated};
/// The test-only phase markers.
use super::transaction_phase::{Phase, PhaseMarkerError, reach_transaction_phase};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "landTransaction";

/// What:
///  What one landing installs.
/// Why:
///   A prepared commit advances the target;
///  a normalization installs only the settled index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingPayload = { operation: 'commit'; newOid; landedTreeOid; expectedOld; landedIndexPath; exactPrivateIndex; guardedPaths } | { operation: 'normalize-only'; landedTreeOid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LandingPayload {
    /// A prepared commit.
    Commit {
        /// Prepared commit.
        new_oid: String,
        /// Prepared or replayed tree.
        landed_tree_oid: String,
        /// Target value the commit's parent names:
        ///  the preparation base,
        ///  or the replay parent.
        expected_old: Base,
        /// Private index the commit's tree came from.
        landed_index: PathBuf,
        /// Whether `landed_index` is the exact index native preparation committed,
        ///  false after a replay.
        exact_private_index: bool,
        /// Paths a commit hook changed,
        ///  reconciled only while their real index entry is unchanged since invocation.
        guarded_paths: Vec<Vec<u8>>,
    },
    /// A normalization whose settled tree equals the base.
    NormalizeOnly {
        /// Base tree the index settles to.
        landed_tree_oid: String,
    },
}

/// What:
///  The result of one landing attempt that ran to a decision.
/// Why:
///   The caller replays a moved target, reports a switched branch, and completes a landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingOutcome = { kind: 'landed'; oid } | { kind: 'head-moved'; current; replayable } | { kind: 'branch-switched'; current };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LandingOutcome {
    /// The target advanced,
    ///  or a normalization installed its index.
    Landed {
        /// Landed commit;
        ///  the base for a normalization,
        ///  empty when that base is unborn.
        oid: String,
    },
    /// The target moved since preparation;
    ///  nothing landed.
    HeadMoved {
        /// Target value that won.
        current: Base,
        /// Whether replay can land this commit on the new target;
        ///  amends,
        ///  conclusions,
        ///  normalizations and a deleted target never replay.
        replayable: bool,
    },
    /// `HEAD` names another branch,
    ///  or is detached,
    ///  unlike at invocation;
    ///  nothing landed.
    BranchSwitched {
        /// Current symbolic `HEAD` target.
        current: SymbolicHead,
    },
}

/// What:
///  Why a landing attempt stopped without a decision.
/// Why:
///   Each variant names the step that failed,
///  and the workspace's `preserved` flag says whether recovery now owns the transaction directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingFailure = IndexLockUnprovenOwnerError | SubprocessError | Error;
/// ```
#[derive(Debug)]
pub enum LandingError {
    /// The real index lock could not be taken.
    Lock(CoordinationFailure),
    /// A Git step failed.
    Git(GitFailure),
    /// A shared recovery step failed:
    ///  pack keep removal,
    ///  index install or conclusion cleanup.
    Recovery(RecoveryError),
    /// A journal record or marker could not be written.
    Journal(String),
    /// The test-only phase marker variable is malformed or its marker could not be written.
    Phase(PhaseMarkerError),
    /// The landing decided but could not release the real index lock.
    Release(String),
    /// The branch already advanced, and a step that completes the landing failed afterwards:
    ///  the commit exists and must not be committed again.
    LandedThenFailed {
        /// The commit that landed.
        oid: String,
        /// The reference it landed on.
        target: String,
        /// The step that failed afterwards.
        cause: Box<LandingError>,
    },
}

/// The wording of the failure.
impl std::fmt::Display for LandingError {
    /// Writes the wording of the failing step.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Lock(failure) => write!(formatter, "{failure:?}"),
            Self::Git(failure) => write!(formatter, "{failure}"),
            Self::Recovery(failure) => write!(formatter, "{}", failure.0),
            Self::Journal(text) | Self::Release(text) => formatter.write_str(text.as_str()),
            Self::Phase(failure) => write!(formatter, "{failure}"),
            Self::LandedThenFailed { oid, target, cause } => write!(
                formatter,
                "the commit {oid} landed on {target}, but completing its records failed: {cause}. \
The commit is done, so do not retry it; check it with git log and git status."
            ),
        };
    }
}

/// What:
///  The transaction state a landing reads and marks.
/// Why:
///   `CommitTransactionWorkspace`:
///  the landing needs the directory and its private indexes,
///  and tells the caller whether recovery now owns the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTransactionWorkspace = { transactionId; directory; commitIndexPath; capturedIndexPath; shadowPath; preserveForRecovery(); finishTransaction() };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandingWorkspace {
    /// Transaction ID,
    ///  also the reflog nonce.
    pub transaction_id: String,
    /// Durable transaction directory.
    pub directory: PathBuf,
    /// Shadow repository holding the prepared commit.
    pub shadow: PathBuf,
    /// Exact real index snapshot taken at invocation.
    pub captured_index: PathBuf,
    /// Private commit index.
    pub commit_index: PathBuf,
    /// Whether a landing record hands the directory to recovery,
    ///  so the caller's disposal must leave it in place.
    ///  Set before `landing-<n>.json` is written and cleared again when the compare-and-swap is lost.
    pub preserved: bool,
}

/// What:
///  The facts of one attempt beside the workspace.
/// Why:
///   They stay fixed across attempts of one commit, except the payload and the attempt number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { capture, mode, payload, committedPaths, addedPaths, selectedWorktreePaths, indexLockTimeoutMs, attempt }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LandingRequest<'request> {
    /// Invocation capture naming the target and the real index.
    pub capture: &'request InvocationCapture,
    /// Commit selection mode.
    pub mode: TransactionMode,
    /// The commit or normalization to land.
    pub payload: &'request LandingPayload,
    /// Paths an explicit-path commit carries.
    pub committed_paths: &'request [Vec<u8>],
    /// Policy-added path records.
    pub added_paths: &'request [AddedPath],
    /// Selected newline correction records.
    pub selected_worktree_paths: &'request [AddedPath],
    /// Backoff budget for a foreign `index.lock`.
    pub index_lock_timeout_ms: u64,
    /// Landing attempt number,
    ///  from 1.
    pub attempt: i64,
}

/// What:
///  Write one journal record durably:
///  exclusively,
///  then the directory entry.
/// Why:
///   `writeJournalRecord`:
///  a state file is never rewritten,
///  so a crash leaves the newest complete state readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournalRecord({ directory, filename, record });
/// ```
fn write_journal(directory: &Path, filename: &str, bytes: &[u8]) -> Result<(), LandingError> {
    let path: PathBuf = directory.join(filename);
    if let Err(error) = write_private_file(path.as_path(), bytes) {
        return Err(LandingError::Journal(format!("writing {} failed: {error}", path.display())));
    }
    if let Err(error) = sync_directory(directory) {
        return Err(LandingError::Journal(format!("syncing {} failed: {error}", directory.display())));
    }
    return Ok(());
}

/// What:
///  Reach one test-only phase marker.
/// Why:
///   A malformed marker variable fails the landing at every phase it reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase });
/// ```
fn reach(phase: Phase) -> Result<(), LandingError> {
    return reach_transaction_phase(phase, None).map_err(LandingError::Phase);
}

/// What:
///  The device and inode of a landing artifact.
/// Why:
///   `fileIdentity`:
///  recovery proves an artifact is the exact file written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fileIdentity(path)
/// ```
fn artifact_identity(path: &Path) -> Result<FileIdentity, LandingError> {
    return match file_identity_of(path) {
        Ok((device, inode, _)) => Ok(FileIdentity { device, inode }),
        Err(error) => Err(LandingError::Journal(format!("reading the identity of {} failed: {error}", path.display()))),
    };
}

/// What:
///  Whether a target value names a commit.
/// Why:
///   Only a target that still names a commit can be replayed onto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// current.kind === 'commit'
/// ```
fn names_commit(target: &Base) -> bool {
    return matches!(target, Base::Commit(_));
}

/// What:
///  The real index lock while a landing holds it.
/// Why:
///   The lock moves out when the install consumes it,
///  and a lock that recovery needs for the install stays in place when the attempt fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using indexLock = await acquireRealIndexLock({ realIndexPath, transactionDirectory, attempt, timeoutMs });
/// ```
struct Held {
    /// The lock,
    ///  until the install consumes it.
    lock: Option<RealIndexLock>,
    /// Whether the landing reached the point after which only the install is left,
    ///  so recovery installs through this lock if the attempt dies or fails.
    for_recovery: bool,
}

/// What:
///  Runs one landing attempt.
/// Why:
///   `landTransaction`:
///  the caller already holds the landing lock (`_landing_lock` is the witness) and has dealt with the starvation reservation.
///  The real `index.lock` is released on every exit except an install,
///  and except a failure after the target advanced (or, for a normalization, just before the install),
///  because recovery completes the install through the same lock and fails closed without it.
///  A target that moved ends the attempt with `HeadMoved`;
///  when `replayable`,
///  the caller replays and revalidates outside both locks and lands again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landTransaction({ gitPath, cwd, capture, workspace, mode, payload, committedPaths, addedPaths, selectedWorktreePaths, indexLockTimeoutMs, attempt });
/// ```
pub fn land_transaction(
    context: &GitContext,
    cwd: &Path,
    _landing_lock: &OwnerLock,
    workspace: &mut LandingWorkspace,
    request: &LandingRequest<'_>,
) -> Result<LandingOutcome, LandingError> {
    debug(TAG, format!("landing attempt {} of transaction {}", request.attempt, workspace.transaction_id).as_str());
    let acquired: RealIndexLock = acquire_landing_index_lock(
        request.capture.layout.real_index_path.as_path(),
        workspace.directory.as_path(),
        request.attempt,
        request.index_lock_timeout_ms,
        "landed nothing",
    )
    .map_err(LandingError::Lock)?;
    let mut held: Held = Held { lock: Some(acquired), for_recovery: false };
    let result: Result<LandingOutcome, LandingError> = land_locked(context, cwd, workspace, request, &mut held);
    let Some(lock) = held.lock.take() else {
        return result;
    };
    if held.for_recovery {
        debug(TAG, "the target advanced before the install failed; the index lock is left for recovery");
        return result;
    }
    match (lock.release_after_landing(), result) {
        (Ok(()), outcome) => return outcome,
        (Err(release), Ok(_)) => return Err(LandingError::Release(release)),
        (Err(release), Err(error)) => {
            warn(TAG, format!("the index lock could not be released after a failed landing: {release}").as_str());
            return Err(error);
        }
    }
}

/// What:
///  The landing steps that run while this process holds the real index lock.
/// Why:
///   The lock moves out of `held` only when the install consumes it,
///  so the caller releases whatever is still held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the body of landTransaction after `await using indexLock`
/// ```
fn land_locked(
    context: &GitContext,
    cwd: &Path,
    workspace: &mut LandingWorkspace,
    request: &LandingRequest<'_>,
    held: &mut Held,
) -> Result<LandingOutcome, LandingError> {
    reach(Phase::LandingLocked)?;
    let capture: &InvocationCapture = request.capture;
    // A fresh read of the worktree's `HEAD` under the landing lock, never the recorded one.
    let ref_files: RefFilesLayout<'_> = RefFilesLayout {
        git_dir: capture.layout.git_dir.as_path(),
        common_dir: capture.layout.common_dir.as_path(),
        ref_format: capture.layout.ref_format,
        object_format: capture.layout.object_format.as_str(),
    };
    let symbolic_head: SymbolicHead = resolve_symbolic_head(context, cwd, &ref_files).map_err(LandingError::Git)?;
    if symbolic_head != capture.layout.symbolic_head {
        debug(TAG, "HEAD names another target than at invocation");
        return Ok(LandingOutcome::BranchSwitched { current: symbolic_head });
    }
    let expected_old: Base = match request.payload {
        LandingPayload::Commit { expected_old, .. } => expected_old.clone(),
        LandingPayload::NormalizeOnly { .. } => capture.base.clone(),
    };
    let current: Base = resolve_ref_commit(context, cwd, capture.layout.target_ref.as_str()).map_err(LandingError::Git)?;
    if !same_target(&current, &expected_old) {
        debug(TAG, format!("{} moved since preparation", capture.layout.target_ref).as_str());
        let replayable: bool = matches!(request.payload, LandingPayload::Commit { .. })
            && capture.layout.conclusion == Conclusion::None
            && names_commit(&current);
        return Ok(LandingOutcome::HeadMoved { current, replayable });
    }
    let pack_name: Option<String> = match request.payload {
        LandingPayload::Commit { new_oid, .. } => Some(
            migrate_shadow_objects(
                context,
                cwd,
                workspace.shadow.as_path(),
                new_oid.as_str(),
                &expected_old,
                Some(transaction_keep_message(workspace.transaction_id.as_str()).as_str()),
            )
            .map_err(LandingError::Git)?,
        ),
        LandingPayload::NormalizeOnly { .. } => None,
    };
    let post: PathBuf = match record_landing(context, cwd, workspace, request, held, &expected_old, pack_name.as_ref()) {
        Ok(post) => post,
        Err(error) => {
            discard_unrecorded_keep(capture.layout.object_directory.as_path(), workspace, pack_name.as_deref());
            return Err(error);
        }
    };
    if let LandingPayload::Commit { new_oid, .. } = request.payload {
        let pack: &str = pack_name.as_deref().unwrap_or("");
        if let Some(lost) = advance_target(context, cwd, workspace, request, held, new_oid.as_str(), &expected_old, pack)? {
            return Ok(lost);
        }
    }
    if let Err(error) = complete_landing(context, cwd, workspace, request, held, post.as_path()) {
        // Once the branch advanced, a later failure must say so, or a consumer would commit the same work again.
        return Err(match request.payload {
            LandingPayload::Commit { new_oid, .. } => LandingError::LandedThenFailed {
                oid: new_oid.clone(),
                target: capture.layout.target_ref.clone(),
                cause: Box::new(error),
            },
            LandingPayload::NormalizeOnly { .. } => error,
        });
    }
    debug(TAG, format!("landed attempt {}", request.attempt).as_str());
    let oid: String = match request.payload {
        LandingPayload::Commit { new_oid, .. } => new_oid.clone(),
        LandingPayload::NormalizeOnly { .. } => match &capture.base {
            Base::Commit(oid) => oid.clone(),
            Base::Unborn => String::new(),
        },
    };
    return Ok(LandingOutcome::Landed { oid });
}

/// What:
///  Install the post-index and write the records that complete a landing whose record is durable.
/// Why:
///   Everything here follows the branch advance, so the caller words a failure of any step as one that happened
///  after the commit landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the tail of landTransaction after the ref update
/// ```
fn complete_landing(
    context: &GitContext,
    cwd: &Path,
    workspace: &LandingWorkspace,
    request: &LandingRequest<'_>,
    held: &mut Held,
    post: &Path,
) -> Result<(), LandingError> {
    // A normalization has only the install left once its record is durable.
    held.for_recovery = true;
    let Some(installing) = held.lock.take() else {
        return Err(LandingError::Journal(String::from("the real index lock is not held")));
    };
    installing.install(post).map_err(LandingError::Recovery)?;
    write_journal(workspace.directory.as_path(), INDEX_INSTALLED_FILENAME, b"")?;
    reach(Phase::IndexInstalled)?;
    if matches!(request.payload, LandingPayload::Commit { .. }) {
        reproduce_conclusion_cleanup(
            context,
            cwd,
            request.capture.layout.git_dir.as_path(),
            workspace.shadow.as_path(),
            workspace.directory.as_path(),
            request.capture.layout.ref_format,
        )
        .map_err(LandingError::Recovery)?;
    }
    return Ok(());
}

/// What:
///  Computes the post-index against the real index as it is now.
/// Why:
///   A normalization lands the commit index as an exact private index with no guarded paths,
///  as the incumbent's defaults do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await computeLandingPostIndex({ gitPath, cwd, mode, preLandingIndexPath, capturedIndexPath, postIndexPath, landedTreeOid, baseRevision, committedPaths, landedIndexPath });
/// ```
fn compute_post_index(
    context: &GitContext,
    cwd: &Path,
    workspace: &LandingWorkspace,
    request: &LandingRequest<'_>,
    landed_tree: &str,
    pre_landing: &Path,
    post: &Path,
) -> Result<(), LandingError> {
    let base: String = base_revision(request.capture);
    let (landed_index, exact_private_index, guarded_paths): (&Path, bool, &[Vec<u8>]) = match request.payload {
        LandingPayload::Commit { landed_index, exact_private_index, guarded_paths, .. } => {
            (landed_index.as_path(), *exact_private_index, guarded_paths.as_slice())
        }
        LandingPayload::NormalizeOnly { .. } => (workspace.commit_index.as_path(), true, &[]),
    };
    let input: PostIndexInput<'_> = PostIndexInput {
        mode: request.mode,
        pre_landing_index: pre_landing,
        captured_index: workspace.captured_index.as_path(),
        landed_index,
        post_index: post,
        landed_tree,
        base_revision: base.as_str(),
        committed_paths: request.committed_paths,
        guarded_paths,
        exact_private_index,
    };
    return compute_landing_post_index(context, cwd, &input).map_err(LandingError::Git);
}

/// What:
///  The steps between the objects' migration and the landing record: the pre-landing index copy,
///  the post-index, and the record itself.
///  Returns the post-index path.
/// Why:
///   Until the record is written nothing but this process knows the pack's keep,
///  so the caller removes the keep when any of these steps fails;
///  after `preserved` is set, recovery owns the directory and removes the keep by its message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const post = await recordLanding({ gitPath, cwd, workspace, request, indexLock, expectedOld, packName });
/// ```
fn record_landing(
    context: &GitContext,
    cwd: &Path,
    workspace: &mut LandingWorkspace,
    request: &LandingRequest<'_>,
    held: &Held,
    expected_old: &Base,
    pack_name: Option<&String>,
) -> Result<PathBuf, LandingError> {
    reach(Phase::ObjectsMigrated)?;
    let capture: &InvocationCapture = request.capture;
    let landed_tree: &str = match request.payload {
        LandingPayload::Commit { landed_tree_oid, .. } | LandingPayload::NormalizeOnly { landed_tree_oid } => landed_tree_oid.as_str(),
    };
    let pre_landing: PathBuf = workspace.directory.join(pre_landing_index_filename(request.attempt));
    let post: PathBuf = workspace.directory.join(post_index_filename(request.attempt));
    if let Err(error) = copy_index_with_timestamps(capture.layout.real_index_path.as_path(), pre_landing.as_path()) {
        return Err(LandingError::Journal(format!("copying the real index to {} failed: {error}", pre_landing.display())));
    }
    compute_post_index(context, cwd, workspace, request, landed_tree, pre_landing.as_path(), post.as_path())?;
    // The record below hands the directory to recovery, so the mark comes first, as the incumbent's does.
    workspace.preserved = true;
    let lock_identity = match held.lock.as_ref() {
        Some(lock) => lock.identity().clone(),
        None => return Err(LandingError::Journal(String::from("the real index lock is not held"))),
    };
    let record: LandingRecord = LandingRecord {
        attempt: request.attempt,
        operation: match request.payload {
            LandingPayload::Commit { .. } => LandingOperation::Commit,
            LandingPayload::NormalizeOnly { .. } => LandingOperation::NormalizeOnly,
        },
        expected_old: expected_old.clone(),
        new_oid: match request.payload {
            LandingPayload::Commit { new_oid, .. } => Some(new_oid.clone()),
            LandingPayload::NormalizeOnly { .. } => None,
        },
        landed_tree_oid: landed_tree.to_string(),
        pre_landing_index: artifact_identity(pre_landing.as_path())?,
        post_index: artifact_identity(post.as_path())?,
        lock: lock_identity,
        pack_name: pack_name.cloned(),
        added_paths: request.added_paths.to_vec(),
        selected_worktree_paths: request.selected_worktree_paths.to_vec(),
    };
    write_journal(
        workspace.directory.as_path(),
        landing_record_filename(request.attempt).as_str(),
        encode_landing(&record).as_bytes(),
    )?;
    return Ok(post);
}

/// What:
///  Removes the pack keep of a landing that failed before its record handed the directory to recovery.
/// Why:
///   The original failure is what the caller reports,
///  so a keep that cannot be removed is a warning that names it; recovery's keep sweep by message still finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!workspace.preserved && packName) await removePackKeep({ objectDirectory, packName }).catch(warn);
/// ```
fn discard_unrecorded_keep(objects: &Path, workspace: &LandingWorkspace, pack_name: Option<&str>) {
    if workspace.preserved {
        return;
    }
    let Some(pack) = pack_name else {
        return;
    };
    if let Err(error) = remove_pack_keep(objects, pack) {
        warn(TAG, format!("the pack keep of a landing that failed before its record could not be removed: {error}").as_str());
    }
}

/// What:
///  Advances the target by compare-and-swap and journals the result.
///  Returns the outcome when the swap was lost.
/// Why:
///   A lost swap removes the pack keep and releases the directory from recovery,
///  then reports who won;
///  a won swap writes `ref-updated.json`,
///  removes the keep,
///  and records the landed capture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const swap = await compareAndSwapTarget({ gitPath, cwd, capture, transactionId, newOid, expectedOld });
/// ```
fn advance_target(
    context: &GitContext,
    cwd: &Path,
    workspace: &mut LandingWorkspace,
    request: &LandingRequest<'_>,
    held: &mut Held,
    new_oid: &str,
    expected_old: &Base,
    pack_name: &str,
) -> Result<Option<LandingOutcome>, LandingError> {
    let capture: &InvocationCapture = request.capture;
    let swap: CompareAndSwap<'_> = CompareAndSwap {
        target_ref: capture.layout.target_ref.as_str(),
        detached: capture.layout.symbolic_head == SymbolicHead::Detached,
        nonce: workspace.transaction_id.as_str(),
        new_oid,
        expected_old,
        hash_length: capture.layout.empty_tree_oid.len(),
    };
    let output = compare_and_swap_target(context, cwd, &swap).map_err(LandingError::Git)?;
    let objects: &Path = capture.layout.object_directory.as_path();
    if !output.succeeded() {
        debug(TAG, format!("compare-and-swap lost: {}", String::from_utf8_lossy(&output.stderr).trim()).as_str());
        remove_pack_keep(objects, pack_name).map_err(LandingError::Recovery)?;
        workspace.preserved = false;
        let winner: Base = resolve_ref_commit(context, cwd, capture.layout.target_ref.as_str()).map_err(LandingError::Git)?;
        let replayable: bool = capture.layout.conclusion == Conclusion::None && names_commit(&winner);
        return Ok(Some(LandingOutcome::HeadMoved { current: winner, replayable }));
    }
    // From here recovery finds the landed commit by its nonce and installs the index through this lock.
    held.for_recovery = true;
    write_journal(workspace.directory.as_path(), REF_UPDATED_FILENAME, encode_ref_updated(new_oid).as_bytes())?;
    reach(Phase::RefUpdated)?;
    remove_pack_keep(objects, pack_name).map_err(LandingError::Recovery)?;
    record_landed_capture_or_warn(
        capture.layout.git_dir.as_path(),
        workspace.directory.as_path(),
        workspace.transaction_id.as_str(),
        new_oid,
    );
    return Ok(None);
}

/// Landing attempt controls stay out of the release executable.
#[cfg(test)]
#[path = "landing_attempt_tests.rs"]
mod tests;
