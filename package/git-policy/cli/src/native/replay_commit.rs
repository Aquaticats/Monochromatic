//! What:
//!  One replay of a prepared commit onto a moved target, as the landing loop runs it outside both locks:
//!       shared paths, capture order, subsumption, the merge, and either the replayed commit or the conflict.
//! Why:
//!  This is the replay result the integrator consumes.
//!       Revalidation after a clean merge (policies, `pre-commit`) and the events are the integrator's;
//!       this function returns the data for `commit-replayed`, `replay-headers-dropped`, and
//!       `concurrent-commit/replay-conflict`, and the replayed tree the revalidation reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commit-replay.ts + commit-capture-order-replay.ts + commit-replay-subsumption.ts, composed as the landing step does
//! ```

/// Shared paths, capture order, subsumption, the merge, and the rebuild.
use super::replay_capture_history::{CaptureHistoryError, decide_shared_paths};
use super::replay_merge::{ReplayMerge, ReplayMergeError, ReplayedCommit, first_commit_touching, merge_replay_tree, replay_merge_base, write_replayed_commit};
use super::replay_object::{RawCommit, parse_raw_commit};
use super::replay_shadow::run_shadow_checked;
use super::replay_shared_paths::{SharedPathError, list_shared_paths};
use super::replay_subsumption::{SubsumptionBase, SubsumptionError, subsume_landed_changes};
use super::transaction_git::{GitContext, GitFailure};
/// Capture-order decisions keyed by path.
use std::collections::BTreeMap;
/// Filesystem paths.
use std::path::Path;
use super::replay_capture_order::CaptureOrderDecision;

/// What:
///  One replay's inputs: the Git context and directories, the transaction, the commit IDs, and the signing options.
/// Why:
///   A struct keeps the call site short; every field is named once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayRequest = { gitPath; shadowPath; objectDirectory; cwd; gitDir; directory; replay; base; emptyTree; current; prepared; signingKey; globalArgs };
/// ```
pub struct ReplayRequest<'a> {
    /// Real Git executable and the variables every child receives.
    pub context: &'a GitContext,
    /// Shadow repository, as `--git-dir`.
    pub shadow: &'a Path,
    /// Shadow object store, whose alternates reach the real objects.
    pub object_directory: &'a Path,
    /// Worktree root, where Git reports conflicted paths from.
    pub worktree_root: &'a Path,
    /// Git directory of the owning worktree, holding the capture store.
    pub git_dir: &'a Path,
    /// This transaction's directory, holding `captured.json`.
    pub transaction_directory: &'a Path,
    /// Replay number, naming the private files.
    pub replay: u64,
    /// Preparation base commit; `None` for an unborn branch.
    pub base: Option<&'a str>,
    /// Empty tree of the repository's object format, for an unborn base.
    pub empty_tree: &'a str,
    /// Target that won.
    pub current: &'a str,
    /// Prepared commit.
    pub prepared: &'a str,
    /// Key the invocation passed to `-S`; `None` signs with the configured key.
    pub signing_key: Option<&'a str>,
    /// The caller's kept global options, which may configure signing.
    pub global_arguments: &'a [String],
}

/// What:
///  The replay result of a clean merge:
///       the replayed commit and the data for `commit-replayed` and `replay-headers-dropped`.
/// Why:
///   The integrator runs revalidation on `tree`, then emits the events and lands `oid`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Replayed = { oid; tree; onto; preparedOid; fromBase; droppedHeaders; subsumedPaths; keptLandedPaths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replayed {
    /// Replayed commit in the shadow store.
    pub oid: String,
    /// Merged tree the replayed commit holds.
    pub tree: String,
    /// The target the replay parent is.
    pub onto: String,
    /// The prepared commit that was replayed.
    pub prepared_oid: String,
    /// The preparation base, or `None` for an unborn branch.
    pub from_base: Option<String>,
    /// Custom headers a signed rebuild dropped.
    pub dropped_headers: Vec<String>,
    /// Paths whose prepared entries replay as they are.
    pub subsumed_paths: Vec<String>,
    /// Paths whose landed entries stay.
    pub kept_landed_paths: Vec<String>,
}

/// What:
///  The replay result of an overlapping merge:
///       the data for `concurrent-commit/replay-conflict`.
/// Why:
///   The named prepared commit stays in the object store so the user can cherry-pick it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayConflict = { winningOid; paths; preparedOid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayConflict {
    /// First landed commit after the base that touches a conflicting path.
    pub winning_oid: String,
    /// Conflicting paths in Git path byte order, as the exact bytes Git reported.
    pub paths: Vec<Vec<u8>>,
    /// The prepared commit the replay could not land.
    pub prepared_oid: String,
}

/// What:
///  The outcome of one replay.
/// Why:
///   The landing loop branches on it: a clean replay is revalidated and landed, a conflict fails without landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayOutcome = { kind: 'replayed'; ... } | { kind: 'conflict'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayOutcome {
    /// The change applies cleanly; the replayed commit exists in the shadow store.
    Replayed(Replayed),
    /// The change conflicts with the target.
    Conflict(ReplayConflict),
}

/// What:
///  A replay failure:
///       one of its steps failed, and the replay is `transaction-failed` rather than a conflict.
/// Why:
///   Each step reports its own failure type, so the integrator sees which step broke.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(message);
/// ```
#[derive(Debug)]
pub enum ReplayError {
    /// Listing the shared paths failed.
    Shared(SharedPathError),
    /// Capture order could not read its records or history.
    Capture(CaptureHistoryError),
    /// Subsumption failed.
    Subsumption(SubsumptionError),
    /// The merge or the rebuild failed.
    Merge(ReplayMergeError),
    /// Reading the prepared commit failed.
    Git(GitFailure),
    /// The prepared commit's raw object could not be parsed.
    Object(String),
}

/// What:
///  Reads the prepared commit's raw object through the shadow.
/// Why:
///   The rebuild needs its tree header, identities, encoding, message, and signature state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRawCommit(await runShadowGit({ args: ['cat-file', 'commit', prepared] }).stdout);
/// ```
fn prepared_commit(request: &ReplayRequest<'_>) -> Result<RawCommit, ReplayError> {
    let output = run_shadow_checked(
        request.context,
        request.shadow,
        request.worktree_root,
        &["cat-file", "commit", request.prepared],
        None,
        None,
        &[],
    )
    .map_err(ReplayError::Git)?;
    return parse_raw_commit(&output.stdout).map_err(|error| { return ReplayError::Object(error.0); });
}

/// What:
///  The result of the merge half of a replay: the merged tree and what subsumption decided, or the conflict.
/// Why:
///   The incumbent revalidates the merged tree before it rebuilds the commit, so the two halves are separate steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeStep = { kind: 'merged'; treeOid; subsumedPaths; keptLandedPaths } | { kind: 'conflict'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MergeStep {
    /// The change applies cleanly.
    Merged {
        /// The merged tree.
        tree: String,
        /// Paths whose prepared entries replay as they are.
        subsumed_paths: Vec<String>,
        /// Paths whose landed entries stay.
        kept_landed_paths: Vec<String>,
    },
    /// The change conflicts with the target.
    Conflict(ReplayConflict),
}

/// What:
///  Lists the shared paths, decides capture order, settles subsumed paths and merges, naming the first landed commit
///  that touched a conflicting path when the merge conflicts.
/// Why:
///   Every step runs outside both landing locks in the shadow store; the merged tree is revalidated before the
///  commit is rebuilt on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subsumption = await subsumeLandedChanges(...); const merge = await mergeReplayTree(...);
/// ```
pub fn merge_replay(request: &ReplayRequest<'_>) -> Result<MergeStep, ReplayError> {
    let merge_base: String = replay_merge_base(request.context, request.shadow, request.worktree_root, request.base, request.empty_tree)
        .map_err(ReplayError::Merge)?;
    let shared = list_shared_paths(request.context, request.shadow, request.worktree_root, &merge_base, request.current, request.prepared)
        .map_err(ReplayError::Shared)?;
    let names: Vec<String> = shared.iter().map(|entry| { return entry.path.clone(); }).collect();
    let order: BTreeMap<String, CaptureOrderDecision> = decide_shared_paths(
        request.context,
        request.shadow,
        request.worktree_root,
        request.git_dir,
        request.transaction_directory,
        request.base,
        request.current,
        &names,
    )
    .map_err(ReplayError::Capture)?;
    let subsumption: SubsumptionBase = subsume_landed_changes(
        request.context,
        request.shadow,
        request.object_directory,
        request.worktree_root,
        request.transaction_directory,
        request.replay,
        &merge_base,
        &shared,
        &order,
    )
    .map_err(ReplayError::Subsumption)?;
    let merged: ReplayMerge = merge_replay_tree(
        request.context,
        request.shadow,
        request.worktree_root,
        &subsumption.merge_base,
        request.current,
        request.prepared,
    )
    .map_err(ReplayError::Merge)?;
    match merged {
        ReplayMerge::Conflict { paths } => {
            let winning_oid: String = first_commit_touching(request.context, request.worktree_root, request.base, request.current, &paths)
                .map_err(ReplayError::Merge)?;
            return Ok(MergeStep::Conflict(ReplayConflict {
                winning_oid,
                paths,
                prepared_oid: String::from(request.prepared),
            }));
        }
        ReplayMerge::Merged { tree_oid } => {
            return Ok(MergeStep::Merged {
                tree: tree_oid,
                subsumed_paths: subsumption.subsumed_paths,
                kept_landed_paths: subsumption.kept_landed_paths,
            });
        }
    }
}

/// What:
///  Rebuilds the prepared commit on a tree, with the target that won as its parent.
/// Why:
///   The tree is the merged tree, or the tree revalidation settled; a signed commit is re-signed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeReplayedCommit({ gitPath, shadowPath, directory, attempt, commit: prepared.raw, treeOid, parentOid: onto, globalArgs, signingKey });
/// ```
pub fn rebuild_replay(request: &ReplayRequest<'_>, tree: &str) -> Result<ReplayedCommit, ReplayError> {
    let commit: RawCommit = prepared_commit(request)?;
    return write_replayed_commit(
        request.context,
        request.shadow,
        request.worktree_root,
        request.transaction_directory,
        request.replay,
        &commit,
        tree,
        request.current,
        request.signing_key,
        request.global_arguments,
    )
    .map_err(ReplayError::Merge);
}

/// What:
///  Replays the prepared commit onto `current`:
///       lists the shared paths, decides capture order, settles subsumed paths, merges, and rebuilds on a clean merge;
///       on a conflict, names the first landed commit that touched a conflicting path.
/// Why:
///   Every step runs outside both landing locks in the shadow store, so the landing loop can take the locks only for the
///       revalidated commit it then lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // replay in commit-replay.ts; order decisions and subsumption composed as the landing step does
/// ```
pub fn replay_commit(request: &ReplayRequest<'_>) -> Result<ReplayOutcome, ReplayError> {
    let (tree, subsumed_paths, kept_landed_paths): (String, Vec<String>, Vec<String>) = match merge_replay(request)? {
        MergeStep::Conflict(conflict) => return Ok(ReplayOutcome::Conflict(conflict)),
        MergeStep::Merged {
            tree,
            subsumed_paths,
            kept_landed_paths,
        } => (tree, subsumed_paths, kept_landed_paths),
    };
    let replayed: ReplayedCommit = rebuild_replay(request, tree.as_str())?;
    return Ok(ReplayOutcome::Replayed(Replayed {
        oid: replayed.oid,
        tree,
        onto: String::from(request.current),
        prepared_oid: String::from(request.prepared),
        from_base: request.base.map(String::from),
        dropped_headers: replayed.dropped_headers,
        subsumed_paths,
        kept_landed_paths,
    }));
}

#[cfg(test)]
#[path = "replay_commit_tests.rs"]
mod tests;
