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
use super::commit_preparation::{NATIVE_UNSET_VARIABLES, install_guard_when_free};
use super::commit_transaction::{CommitInvocation, TransactionStop, failed, settled_passes, staged_at_base};
use super::commit_workspace::CommitWorkspace;
use super::commit_worktree_records::{HookChanges, describe_hook_changes};
use super::diagnostic_log::debug;
use super::forwarding::{exit_code, git_command};
use super::signal_relay::SignalGuard;
use super::hook_dispatch_plan::SILENCED_HOOK_EVENTS;
use super::policy_checks::ShippedChecks;
use super::policy_content::{ContentState, LifecycleContent};
use super::policy_engine::{StageRequest, pass_exit_code};
use super::policy_events::PolicyEvent;
use super::policy_pass::{PassResult, run_policy_pass};
use super::private_storage::create_private_directory;
use super::replay_revalidation::{move_shadow_head, pre_commit_rerun_required, read_replayed_index, replayed_index_path};
use super::repository_facts::RepositoryFacts;
use super::transaction_git::GitContext;
use super::transaction_journal::SymbolicHead;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "revalidateReplay";

/// The exit status of a commit whose `pre-commit` hook rejected it, as native `git commit` exits.
const HOOK_REJECTION_EXIT: i32 = 1;

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
        /// The tree to commit, with policy and hook changes.
        tree: String,
        /// The private index holding that tree.
        index: PathBuf,
        /// The settled pass's events, with the fix summary the corrections earned.
        events: Vec<PolicyEvent>,
        /// The paths the replayed commit changes against its new parent.
        committed_paths: Vec<Vec<u8>>,
        /// What the re-run `pre-commit` staged.
        hook_changes: HookChanges,
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
///   The hook re-run needs the kept global options and the shim; the policies need the capture.
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
    /// The caller's kept global options.
    pub global_args: &'a [String],
    /// Whether the invocation bypassed `pre-commit` with `--no-verify`.
    pub no_verify: bool,
    /// Whether the dispatch plan wrote hook entries; without them no hook can run, so the re-run starts nothing.
    pub named_hooks_possible: bool,
}

/// What:
///  Re-run `pre-commit` against a replayed private index through the dispatcher shim.
/// Why:
///   It runs exactly as native preparation ran it, so the shim takes the hook lock and restores the caller's
///  configuration; `prepare-commit-msg` and `commit-msg` do not re-run because the message is fixed.
///  Without hook entries nothing runs, and nothing is started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rerunPreCommit({ gitPath, worktreeRoot, globalArgs, shadowPath, hooksDirectory, indexPath });
/// ```
fn rerun_pre_commit(
    context: &GitContext,
    worktree_root: &Path,
    global_args: &[String],
    shadow: &Path,
    hooks_directory: &Path,
    named_hooks_possible: bool,
    index: &Path,
) -> Result<(), TransactionStop> {
    // A configuration with no hook defined by name has no dispatcher entry, so `git hook run` would find nothing.
    if !named_hooks_possible {
        debug(TAG, "the configuration defines no hook by name; the re-run is skipped");
        return Ok(());
    }
    let mut arguments: Vec<OsString> = global_args.iter().map(OsString::from).collect();
    arguments.push(OsString::from(format!("--git-dir={}", shadow.display())));
    arguments.push(OsString::from(format!("--work-tree={}", worktree_root.display())));
    arguments.push(OsString::from("-c"));
    arguments.push(OsString::from(format!("core.hooksPath={}", hooks_directory.display())));
    for event in SILENCED_HOOK_EVENTS {
        arguments.push(OsString::from("-c"));
        arguments.push(OsString::from(format!("hook.{event}.enabled=false")));
    }
    for word in ["hook", "run", "--ignore-missing", "pre-commit"] {
        arguments.push(OsString::from(word));
    }
    let mut command = git_command(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice());
    command
        .current_dir(worktree_root)
        .env("GIT_INDEX_FILE", index)
        // Native Git gives pre-commit `GIT_EDITOR=:` when no editor runs, and a replay never opens one.
        .env("GIT_EDITOR", ":");
    for name in NATIVE_UNSET_VARIABLES {
        command.env_remove(name);
    }
    let guard: SignalGuard = install_guard_when_free()
        .map_err(|error| { return failed(format!("installing the signal guard failed: {error}")); })?;
    let outcome = guard
        .run_child(&mut command)
        .map_err(|error| { return TransactionStop::Message(format!("git hook run could not start: {error}")); })?;
    debug(TAG, format!("pre-commit re-run ended {outcome:?}").as_str());
    if exit_code(outcome) != 0 {
        return Err(TransactionStop::NativeExit(HOOK_REJECTION_EXIT));
    }
    return Ok(());
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
    approved_tree: &str,
) -> Result<Revalidation, TransactionStop> {
    let git: &GitContext = context.invocation.context;
    let cwd: &Path = context.invocation.cwd;
    let root: &Path = context.capture.layout.repository_root.as_path();
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
    let rerun: bool = pre_commit_rerun_required(context.no_verify, policy_tree.as_str(), approved_tree);
    if rerun {
        rerun_pre_commit(
            git,
            root,
            context.global_args,
            workspace.landing.shadow.as_path(),
            workspace.hooks_directory.as_path(),
            context.named_hooks_possible,
            index.as_path(),
        )?;
    }
    let tree: String = if rerun {
        write_private_tree(git, cwd, index.as_path(), workspace.object_directory.as_path())?
    } else {
        policy_tree.clone()
    };
    let hook_changes: HookChanges = if rerun {
        describe_hook_changes(git, cwd, workspace.object_directory.as_path(), policy_tree.as_str(), tree.as_str())?
    } else {
        HookChanges::default()
    };
    debug(
        TAG,
        format!("revalidated replay {replay} onto {onto}: tree {tree}{}", if rerun { " after pre-commit" } else { "" }).as_str(),
    );
    return Ok(Revalidation::Revalidated {
        tree,
        index,
        events: super::commit_conclusion::events_with_changed_summary(pass.events, changed_passes, changed_paths.as_slice()),
        committed_paths: candidate_paths,
        hook_changes,
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
