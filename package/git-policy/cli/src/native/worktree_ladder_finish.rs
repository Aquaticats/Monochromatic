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
use super::worktree_clone::{clear_destination, clone_tracked_files};
use super::worktree_ladder_target::{Target, apply_difference, checkout_environment, prepare_target, run_step};
use super::worktree_copy::{GitRunner, GitStatus};
use super::worktree_add_plan::AddPlan;
use super::worktree_copy_stage_claim::StageClaim;
use super::worktree_ladder_run::{Ladder, LadderResult, Rung, discard_stage};
use super::worktree_snapshot::{put_in_place, remove_leftover, take_snapshot};
use super::worktree_snapshot_source::SnapshotSource;
use super::worktree_subvolume::settle_subvolume_destination;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  Everything before the swap and the swap itself; the error is the reason the rung could not finish.
/// Why:
///   The caller destroys the stage on an error and completes the worktree with Git's own checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveSnapshot(args): Promise<string>; // the checkout line; throws a reason
/// ```
fn serve_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext) -> Result<String, String> {
    let target: Target = prepare_target(source, plan, context)?;
    take_snapshot(source, plan, stage, target.admin.as_path(), target.pointer.as_slice())?;
    apply_difference(source, &target, stage, token, run_git)?;
    let mode: u32 = std::fs::metadata(source.root.as_path()).map_err(|error| { return format!("the source's root is unreadable ({error})"); })?.mode() & 0o7777;
    put_in_place(stage, plan.destination.as_path(), mode)?;
    if let Err(reason) = remove_leftover(stage) {
        warn(TAG, reason.as_str());
    }
    return Ok(target.line);
}

/// What:
///  The clone rung: clone the tracked files into the destination, install the index, apply the tree difference.
/// Why:
///   The destination is filled in place, so an error leaves a partial worktree for the caller to clear before Git's own
///  checkout completes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveClones(args): Promise<string>; // the checkout line; throws a reason
/// ```
fn serve_clones(source: &SnapshotSource, plan: &AddPlan, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext) -> Result<String, String> {
    let target: Target = prepare_target(source, plan, context)?;
    clone_tracked_files(source, plan.destination.as_path(), target.admin.as_path())?;
    apply_difference(source, &target, plan.destination.as_path(), token, run_git)?;
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
    let status: GitStatus = match run_step(run_git, token, arguments.as_slice(), environment.as_slice()) {
        Ok(status) => status,
        Err(reason) => {
            warn(TAG, format!("Git's checkout of {} could not start ({reason})", destination.display()).as_str());
            return GitStatus { code: Some(1) };
        }
    };
    if !status.succeeded() {
        return status;
    }
    return status;
}

/// What:
///  Serve the clone rung after real Git returned, or complete the worktree with Git's own checkout.
/// Why:
///   The destination is filled in place, so a failed attempt is cleared first; the ignored state is copied by the
///  lifecycle afterwards, as for Git's checkout, so nothing is reported as carried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishClones(args): Promise<LadderResult>;
/// ```
fn finish_clones(source: &SnapshotSource, plan: &AddPlan, git: GitStatus, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext, out: &mut dyn Write) -> LadderResult {
    if !git.succeeded() {
        return LadderResult { git, carried: None };
    }
    return match serve_clones(source, plan, token, run_git, context) {
        Ok(line) => {
            debug(TAG, format!("clones served {}", plan.destination.display()).as_str());
            // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
            let _written = out.write_all(line.as_bytes());
            LadderResult { git, carried: None }
        }
        Err(reason) => {
            debug(TAG, format!("the clone rung ended: {reason}; Git checks the worktree out").as_str());
            if let Err(clearing) = clear_destination(plan.destination.as_path()) {
                warn(TAG, clearing.as_str());
            }
            LadderResult { git: checkout_like_git(plan, token, run_git), carried: None }
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
        let (source, claim, plan) = match (self.rung, self.plan) {
            (Rung::Snapshot(source, claim), Some(plan)) => (source, claim, plan),
            (Rung::Snapshot(_, claim), None) => {
                release_claim(claim);
                return LadderResult { git, carried: None };
            }
            (Rung::Clone(source, subvolume), Some(plan)) => {
                let result: LadderResult = finish_clones(&source, &plan, git, token, run_git, context, out);
                if let Some(created) = subvolume.as_ref() {
                    settle_subvolume_destination(created);
                }
                return result;
            }
            (Rung::Git(subvolume) | Rung::Clone(_, subvolume), _) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_subvolume_destination(created);
                }
                return LadderResult { git, carried: None };
            }
        };
        let stage: PathBuf = claim.container().to_path_buf();
        if !git.succeeded() {
            release_claim(claim);
            return LadderResult { git, carried: None };
        }
        let result: LadderResult = match serve_snapshot(&source, &plan, stage.as_path(), token, run_git, context) {
            Ok(line) => {
                debug(TAG, format!("the snapshot served {}", plan.destination.display()).as_str());
                // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
                let _written = out.write_all(line.as_bytes());
                LadderResult { git, carried: Some(plan.destination.clone()) }
            }
            Err(reason) => {
                debug(TAG, format!("the snapshot rung ended: {reason}; Git checks the worktree out").as_str());
                discard_stage(stage.as_path());
                LadderResult { git: checkout_like_git(&plan, token, run_git), carried: None }
            }
        };
        release_claim(claim);
        return result;
    }
}

/// Ladder finish controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_finish_tests.rs"]
mod tests;
