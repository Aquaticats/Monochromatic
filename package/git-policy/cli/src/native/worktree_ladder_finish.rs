//! What:
//!  What the ladder does after real Git returned: the snapshot rung's steps (read the new `HEAD`, snapshot, apply the
//!  tree difference, swap into place, print Git's checkout line), and the fall back to Git's own `reset --hard` when a
//!  step cannot finish.
//! Why:
//!  Every step before the swap leaves the destination exactly as Git's `--no-checkout` run made it, so a failure there
//!  destroys the stage and completes the worktree the way Git would have. The line that Git's own checkout would have
//!  printed is printed here, so the person cannot tell the rungs apart except by the time they take. No hook runs on
//!  any rung: every real-Git start of the wrapper switches hooks off (`hooks_off.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await ladder.finish({ git, token, runGit, context, out });
//! ```

use super::diagnostic_log::{debug, warn};
use super::transaction_git::GitContext;
use super::worktree_add_plan::AddPlan;
use super::worktree_add_rollback::{Bound, Removal, bind_registration, remove_bound};
use super::worktree_clone::{Cloned, clone_tracked_files};
use super::worktree_copy::{GitRunner, GitStatus};
use super::worktree_copy_stage_claim::StageClaim;
use super::worktree_ladder_run::{Ladder, LadderResult, Rung, discard_stage};
use super::worktree_ladder_target::{Target, apply_difference, checkout_environment, prepare_target, run_step};
use super::worktree_snapshot::{PlaceError, put_in_place, settle_displaced, take_snapshot};
use super::worktree_snapshot_source::SnapshotSource;
use super::worktree_subvolume::settle_subvolume_destination;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// Why a rung ended before it finished, and whether the object at the stage name is still the snapshot this process made.
struct Ended {
    /// The reason, in words.
    reason: String,
    /// Whether the stage name holds this process's snapshot, so that destroying it destroys nothing else.
    stage_is_ours: bool,
}

impl From<String> for Ended {
    fn from(reason: String) -> Ended {
        return Ended { reason, stage_is_ours: true };
    }
}

impl From<PlaceError> for Ended {
    fn from(error: PlaceError) -> Ended {
        return Ended { reason: error.reason, stage_is_ours: error.stage_is_ours };
    }
}

/// What:
///  Everything before the swap and the swap itself; the error is the reason the rung could not finish.
/// Why:
///   The caller destroys the stage on an error that left the stage name holding the snapshot, and completes the worktree
///  with Git's own checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveSnapshot(args): Promise<string>; // the checkout line; throws a reason
/// ```
fn serve_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, bound: &Bound, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext) -> Result<String, Ended> {
    let target: Target = prepare_target(source, plan, context, bound)?;
    take_snapshot(source, plan, stage, &bound.admin, target.pointer.as_slice())?;
    apply_difference(source, &target, stage, token, run_git)?;
    let mode: u32 = std::fs::metadata(source.root.as_path()).map_err(|error| { return format!("the source's root is unreadable ({error})"); })?.mode() & 0o7777;
    put_in_place(stage, plan.destination.as_path(), mode, bound.binding.destination_id)?;
    for note in settle_displaced(stage, plan.destination.as_path(), bound.binding.destination_id, target.pointer.as_slice()) {
        warn(TAG, format!("cli-git: {note}").as_str());
    }
    return Ok(target.line);
}

/// What:
///  The clone rung: clone the tracked files into the destination, install the index, apply the tree difference.
/// Why:
///   The destination is filled in place; an error removes the entries the run created (and nothing else) before Git's own
///  checkout completes the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveClones(args): Promise<string>; // the checkout line; throws a reason
/// ```
fn serve_clones(source: &SnapshotSource, plan: &AddPlan, bound: &Bound, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext) -> Result<String, String> {
    let target: Target = prepare_target(source, plan, context, bound)?;
    let cloned: Cloned = clone_tracked_files(source, &bound.destination, &bound.admin)?;
    if let Err(reason) = apply_difference(source, &target, plan.destination.as_path(), token, run_git) {
        // Only the entries the clones created go, while they are still the objects that were made; Git's own checkout
        // completes the rest.
        for kept in cloned.remove() {
            warn(TAG, format!("cli-git: {kept}").as_str());
        }
        return Err(reason);
    }
    return Ok(target.line);
}

/// What:
///  Complete the worktree the way Git does after a refused rung: `git reset --hard --no-recurse-submodules`.
/// Why:
///   Real Git ran with `--no-checkout`, so the destination holds only its `.git` file; this is Git's own checkout step.
///  Git's `post-checkout` hook is not run: every real-Git start of the wrapper switches hooks off (`hooks_off.rs`), so
///  the Git run of a forwarded `git worktree add` never ran it either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkoutLikeGit(args): Promise<GitStatus>;
/// ```
fn checkout_like_git(plan: &AddPlan, token: Option<&str>, run_git: &mut GitRunner) -> GitStatus {
    let destination: &Path = plan.destination.as_path();
    let mut arguments: Vec<OsString> = ["-C"].iter().map(OsString::from).chain([destination.as_os_str().to_os_string()]).chain(["reset", "--hard", "--no-recurse-submodules"].iter().map(OsString::from)).collect();
    if plan.quiet {
        arguments.push(OsString::from("--quiet"));
    }
    let environment: Vec<(OsString, OsString)> = checkout_environment(destination.join(".git").as_path(), destination);
    return match run_step(run_git, token, arguments.as_slice(), environment.as_slice()) {
        Ok(status) => status,
        Err(reason) => {
            warn(TAG, format!("Git's checkout of {} could not start ({reason})", destination.display()).as_str());
            GitStatus { code: Some(1) }
        }
    };
}

/// What the ladder did to the add's records at the end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Records {
    /// Release the claim: nothing is pending.
    Release,
    /// Keep the claim and the record: something pending could not be settled, and the next command says so.
    Keep,
}

/// What:
///  Make the end of a successful add durable: the completion marker, or, when it cannot be written, the withdrawal of the
///  record that lets recovery delete the worktree. When neither can be done the add is reported as failed and removed.
/// Why:
///   A success that is printed while recovery may still delete the worktree is a lie with a delay. The marker or the
///  withdrawal must hold before the person is told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await confirmComplete(claim, bound); // throws a reason
/// ```
fn confirm_complete(claim: &StageClaim, bound: &Bound) -> Result<(), String> {
    return confirm_with(
        &mut || { return claim.mark_complete().map_err(|error| { return error.0; }); },
        &mut || { return claim.withdraw_authority().map_err(|error| { return error.0; }); },
        &mut || {
            return match remove_bound(&bound.binding) {
                Removal::Removed => String::from("the worktree was removed"),
                Removal::Kept { reason, .. } => format!("the worktree stays ({reason})"),
            };
        },
    );
}

/// The decision of `confirm_complete` with its three actions as calls: write the marker, withdraw the record, remove the
///  add's objects.
fn confirm_with(mark: &mut dyn FnMut() -> Result<(), String>, withdraw: &mut dyn FnMut() -> Result<(), String>, remove: &mut dyn FnMut() -> String) -> Result<(), String> {
    let marker: String = match mark() {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };
    return match withdraw() {
        Ok(()) => {
            warn(TAG, format!("the completion of the add could not be recorded ({marker}); the record that lets recovery act on it was removed").as_str());
            Ok(())
        }
        Err(withdrawal) => {
            // Neither record can be written or removed; the add is reported as failed and its own objects are removed.
            let outcome: String = remove();
            Err(format!("the completion of the add could not be recorded ({marker}) and the record could not be removed ({withdrawal}); {outcome}"))
        }
    };
}

/// What:
///  Complete the worktree with Git's own checkout after a refused rung, and remove the add's objects again when that fails.
/// Why:
///   Git's `worktree add` removes the worktree it registered when its checkout fails; the wrapper took the checkout
///  over, so it does the same, but only for the objects the add was bound to. A removal that cannot finish keeps the
///  record, and the person is told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [status, records] = await completeLikeGit({ plan, claim, bound, token, runGit });
/// ```
fn complete_like_git(plan: &AddPlan, claim: &StageClaim, bound: &Bound, token: Option<&str>, run_git: &mut GitRunner) -> (GitStatus, Records) {
    let status: GitStatus = checkout_like_git(plan, token, run_git);
    if status.succeeded() {
        return match confirm_complete(claim, bound) {
            Ok(()) => (status, Records::Release),
            Err(reason) => {
                warn(TAG, format!("cli-git: {reason}").as_str());
                (GitStatus { code: Some(1) }, Records::Keep)
            }
        };
    }
    return match remove_bound(&bound.binding) {
        Removal::Removed => {
            debug(TAG, format!("removed {} after its checkout failed", plan.destination.display()).as_str());
            (status, Records::Release)
        }
        Removal::Kept { reason, replaced } => {
            warn(TAG, format!("cli-git: {} was not removed after its checkout failed ({reason}); the next command reports it", plan.destination.display()).as_str());
            (status, if replaced { Records::Release } else { Records::Keep })
        }
    };
}

/// What:
///  Bind the add to the directories Git made, and write that binding to the add's record.
/// Why:
///   Nothing below may delete anything the add is not bound to. A binding that cannot be proven ends the ladder for this
///  add, leaving what Git made as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = await bind(claim, source, plan); // throws a reason
/// ```
fn bind(claim: &StageClaim, source: &SnapshotSource, plan: &AddPlan) -> Result<Bound, String> {
    let bound: Bound = bind_registration(source.common_dir.as_path(), plan.destination.as_path())?;
    if let Err(error) = claim.record_binding(&bound.binding) {
        warn(TAG, format!("the binding of the add could not be recorded ({}); recovery will not remove it", error.0).as_str());
    }
    return Ok(bound);
}

/// What:
///  Serve the clone rung after real Git returned, or complete the worktree with Git's own checkout.
/// Why:
///   A failed attempt has removed what it created; the ignored state is copied by the lifecycle afterwards, as for Git's
///  checkout, so nothing is reported as carried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishClones(args): Promise<[LadderResult, Records]>;
/// ```
fn finish_clones(source: &SnapshotSource, plan: &AddPlan, git: GitStatus, claim: &StageClaim, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext, out: &mut dyn Write) -> (LadderResult, Records) {
    if !git.succeeded() {
        return (LadderResult { git, carried: None }, Records::Release);
    }
    let bound: Bound = match bind(claim, source, plan) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("cli-git: {} was left as Git made it, without a checkout: {reason}", plan.destination.display()).as_str());
            return (LadderResult { git: GitStatus { code: Some(1) }, carried: None }, Records::Release);
        }
    };
    return match serve_clones(source, plan, &bound, token, run_git, context) {
        Ok(line) => {
            debug(TAG, format!("clones served {}", plan.destination.display()).as_str());
            if let Err(reason) = confirm_complete(claim, &bound) {
                warn(TAG, format!("cli-git: {reason}").as_str());
                return (LadderResult { git: GitStatus { code: Some(1) }, carried: None }, Records::Keep);
            }
            // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
            let _written = out.write_all(line.as_bytes());
            (LadderResult { git, carried: None }, Records::Release)
        }
        Err(reason) => {
            debug(TAG, format!("the clone rung ended: {reason}; Git checks the worktree out").as_str());
            let (status, records): (GitStatus, Records) = complete_like_git(plan, claim, &bound, token, run_git);
            (LadderResult { git: status, carried: None }, records)
        }
    };
}

/// What:
///  Serve the snapshot rung after real Git returned, or complete the worktree with Git's own checkout.
/// Why:
///   An error that leaves the stage name holding the snapshot destroys the stage and falls to Git's checkout; an error that
///  leaves somebody else's directory there destroys nothing and ends the add as failed, with the names in the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishSnapshot(args): Promise<[LadderResult, Records]>;
/// ```
fn finish_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, claim: &StageClaim, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext, out: &mut dyn Write) -> (LadderResult, Records) {
    let bound: Bound = match bind(claim, source, plan) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("cli-git: {} was left as Git made it, without a checkout: {reason}", plan.destination.display()).as_str());
            return (LadderResult { git: GitStatus { code: Some(1) }, carried: None }, Records::Release);
        }
    };
    return match serve_snapshot(source, plan, stage, &bound, token, run_git, context) {
        Ok(line) => {
            debug(TAG, format!("the snapshot served {}", plan.destination.display()).as_str());
            if let Err(reason) = confirm_complete(claim, &bound) {
                warn(TAG, format!("cli-git: {reason}").as_str());
                return (LadderResult { git: GitStatus { code: Some(1) }, carried: None }, Records::Keep);
            }
            // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
            let _written = out.write_all(line.as_bytes());
            (LadderResult { git: GitStatus { code: Some(0) }, carried: Some(plan.destination.clone()) }, Records::Release)
        }
        Err(ended) if ended.stage_is_ours => {
            debug(TAG, format!("the snapshot rung ended: {}; Git checks the worktree out", ended.reason).as_str());
            discard_stage(stage);
            let (status, records): (GitStatus, Records) = complete_like_git(plan, claim, &bound, token, run_git);
            (LadderResult { git: status, carried: None }, records)
        }
        Err(ended) => {
            warn(TAG, format!("cli-git: the add of {} ended in a state the wrapper cannot untangle: {}; {} is not touched", plan.destination.display(), ended.reason, stage.display()).as_str());
            (LadderResult { git: GitStatus { code: Some(1) }, carried: None }, Records::Release)
        }
    };
}

/// Release the claim on a stage name, saying so when that fails.
fn release_claim(claim: StageClaim) {
    if let Err(error) = claim.release() {
        warn(TAG, format!("the stage claim could not be released ({})", error.0).as_str());
    }
}

impl Ladder {
    /// What:
    ///  Settle the ladder after real Git returned `git`: serve the snapshot rung or fall back to Git's checkout, or settle
    ///  the empty subvolume of the Git rung.
    /// Why:
    ///   The result carries the status the person should see and the destination whose ignored state a snapshot carried.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const result = await ladder.finish({ git, token, runGit, context, out });
    /// ```
    pub fn finish(self, git: GitStatus, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext, out: &mut dyn Write) -> LadderResult {
        match (self.rung, self.plan) {
            (Rung::Snapshot(source, claim), Some(plan)) => {
                if !git.succeeded() {
                    release_claim(claim);
                    return LadderResult { git, carried: None };
                }
                let stage: PathBuf = claim.container().to_path_buf();
                let (result, records): (LadderResult, Records) = finish_snapshot(&source, &plan, stage.as_path(), &claim, token, run_git, context, out);
                if records == Records::Release {
                    release_claim(claim);
                }
                return result;
            }
            (Rung::Snapshot(_, claim), None) => {
                release_claim(claim);
                return LadderResult { git, carried: None };
            }
            (Rung::Clone(source, subvolume, claim), Some(plan)) => {
                let (result, records): (LadderResult, Records) = finish_clones(&source, &plan, git, &claim, token, run_git, context, out);
                if let Some(created) = subvolume.as_ref() {
                    settle_subvolume_destination(created);
                }
                if records == Records::Release {
                    release_claim(claim);
                }
                return result;
            }
            (Rung::Clone(_, subvolume, claim), None) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_subvolume_destination(created);
                }
                release_claim(claim);
                return LadderResult { git, carried: None };
            }
            (Rung::Git(subvolume), _) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_subvolume_destination(created);
                }
                return LadderResult { git, carried: None };
            }
        }
    }
}

/// Ladder finish controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_finish_tests.rs"]
mod tests;
