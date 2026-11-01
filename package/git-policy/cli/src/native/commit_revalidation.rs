//! What:
//!  Revalidation of a replayed tree, outside both landing locks:
//!  a private index for the replayed tree, the shadow `HEAD` at the replay parent, the policies converged again over
//!  that index, and `pre-commit` re-run whenever the tree differs from the one it last approved.
//! Why:
//!  A replay merges changes the policies and the hook never saw together, so the merged tree must pass the same gates
//!  as the prepared one (`src/policy-engine/commit-revalidation.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const revalidation = await revalidateReplay({ context, replay, mergedTree, onto, approvedTree });
//! ```

use super::candidate_object::{ObjectId, parse_object_id};
use super::capture_invocation::InvocationCapture;
use super::commit_convergence::{CommitConvergence, CommitConvergenceInput, converge_commit_policies};
use super::commit_facts::PrivateIndexFacts;
use super::commit_index::{list_changed_index_paths, write_private_tree};
use super::commit_transaction::{CommitInvocation, TransactionStop, failed, settled_passes, staged_at_base};
use super::commit_workspace::CommitWorkspace;
use super::diagnostic_log::debug;
use super::policy_checks::ShippedChecks;
use super::policy_content::{ContentState, LifecycleContent};
use super::policy_engine::{StageRequest, pass_exit_code};
use super::policy_events::PolicyEvent;
use super::policy_pass::{PassResult, run_policy_pass};
use super::private_storage::create_private_directory;
use super::replay_revalidation::{move_shadow_head, read_replayed_index, replayed_index_path};
use super::repository_facts::RepositoryFacts;
use super::transaction_git::GitContext;
use super::transaction_journal::SymbolicHead;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "revalidateReplay";

/// What:
///  The result of revalidating a replayed tree.
/// Why:
///   A blocked tree stops the commit with the policies' events; a revalidated one is rebuilt and landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Revalidation = { kind: 'revalidated'; ... } | { kind: 'blocked'; result };
/// ```
pub enum Revalidation {
    /// The replayed commit may land.
    Revalidated {
        /// The tree to commit, with the policy changes.
        tree: String,
        /// The private index holding that tree.
        index: PathBuf,
        /// The settled pass's events, with the fix summary the corrections earned.
        events: Vec<PolicyEvent>,
        /// The paths the replayed commit changes against its new parent.
        committed_paths: Vec<Vec<u8>>,
    },
    /// A policy blocked the replayed commit.
    Blocked {
        /// The blocking pass's events.
        events: Vec<PolicyEvent>,
        /// The exit status of the block.
        exit_code: i32,
    },
}

/// What:
///  What revalidation reads besides the policies.
/// Why:
///   The policies need the capture and the kept global options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RevalidationContext = { gitPath; cwd; capture; workspace; args; policyOptions; readOnlySelection; repositoryRoot; globalArgs; noVerify };
/// ```
pub struct RevalidationContext<'a> {
    /// The invocation's fixed facts.
    pub invocation: &'a CommitInvocation<'a>,
    /// The invocation capture.
    pub capture: &'a InvocationCapture,
    /// Whether interactive or include selection forbids automatic patches.
    pub read_only_selection: bool,
}

/// What:
///  Revalidate a replayed tree.
/// Why:
///   The policies converge over the replayed index with the replay parent as their baseline, and `pre-commit` sees
///   the tree when the policies changed what it last approved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await revalidateReplay({ context, replay, mergedTree, onto, approvedTree });
/// ```
pub fn revalidate_replay<F: RepositoryFacts>(
    context: &RevalidationContext<'_>,
    workspace: &CommitWorkspace,
    request: &StageRequest,
    checks: &mut ShippedChecks<PrivateIndexFacts<F>>,
    replay: u64,
    merged_tree: &str,
    onto: &str,
) -> Result<Revalidation, TransactionStop> {
    let git: &GitContext = context.invocation.context;
    let cwd: &Path = context.invocation.cwd;
    let index: PathBuf = replayed_index_path(workspace.landing.directory.as_path(), replay);
    let directory: PathBuf = workspace.landing.directory.join(format!("replay-{replay}"));
    ensure_replay_directory(directory.as_path())
        .map_err(|error| { return failed(format!("creating {} failed: {error}", directory.display())); })?;
    read_replayed_index(git, cwd, workspace.object_directory.as_path(), index.as_path(), merged_tree)?;
    move_shadow_head(
        git,
        workspace.landing.shadow.as_path(),
        cwd,
        context.capture.layout.target_ref.as_str(),
        onto,
        is_detached(&context.capture.layout.symbolic_head),
    )?;
    let candidate_paths: Vec<Vec<u8>> =
        list_changed_index_paths(git, cwd, index.as_path(), onto, workspace.object_directory.as_path())?;
    let base: Option<ObjectId> = parse_object_id(onto.as_bytes());
    checks.facts.use_index(index.as_path(), workspace.object_directory.as_path());
    checks.content = ContentState::new();
    checks.candidates = LifecycleContent::Requested(staged_at_base(base, candidate_paths.as_slice()));
    let (pass, changed_passes, changed_paths): (PassResult, u64, Vec<Vec<u8>>) = if context.read_only_selection {
        checks.can_apply_patches = false;
        (run_policy_pass(request, checks), 0, Vec::new())
    } else {
        let outcome: CommitConvergence = converge_commit_policies(CommitConvergenceInput {
            request,
            checks,
            stage: super::private_index_stage::PrivateStage {
                context: git,
                cwd,
                index: index.as_path(),
                object_directory: Some(workspace.object_directory.as_path()),
            },
        });
        let passes: u64 = settled_passes(&outcome.convergence);
        (outcome.pass, passes, outcome.changed_paths)
    };
    let code: i32 = pass_exit_code(pass.events.as_slice(), pass.end);
    if code != 0 {
        return Ok(Revalidation::Blocked {
            events: pass.events,
            exit_code: code,
        });
    }
    let policy_tree: String =
        write_private_tree(git, cwd, index.as_path(), workspace.object_directory.as_path())?;
    let tree: String = policy_tree;
    debug(TAG, format!("revalidated replay {replay} onto {onto}: tree {tree}").as_str());
    return Ok(Revalidation::Revalidated {
        tree,
        index,
        events: super::commit_conclusion::events_with_changed_summary(pass.events, changed_passes, changed_paths.as_slice()),
        committed_paths: candidate_paths,
    });
}

/// What:
///  Make the directory of one replay, accepting one that subsumption already made for its base index.
/// Why:
///   Every other failure leaves the replay without a place for its index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }).catch((error) => { if (error.code !== 'EEXIST') throw error; });
/// ```
pub fn ensure_replay_directory(directory: &Path) -> std::io::Result<()> {
    match create_private_directory(directory) {
        Err(error) if error.kind() != std::io::ErrorKind::AlreadyExists => return Err(error),
        Ok(()) | Err(_) => return Ok(()),
    }
}

/// What:
///  Whether the invocation started on a detached `HEAD`.
/// Why:
///   A detached replay moves the shadow `HEAD` itself; a branch replay moves the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symbolicHead.kind === 'detached'
/// ```
pub fn is_detached(head: &SymbolicHead) -> bool {
    return *head == SymbolicHead::Detached;
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_revalidation_tests.rs"]
mod tests;
