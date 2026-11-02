//! What:
//!  The ladder around one `git worktree add` on Linux: the choice of rung before real Git runs, and, after it
//!  returned, the snapshot rung's steps with a fall back to Git's own checkout whenever one cannot finish.
//! Why:
//!  `doc/planning/cli-git-btrfs-worktree.md` designs it. With a snapshot source Git runs with `--no-checkout` (branch
//!  validation, administrative files, configuration copy and every message stay Git's), the wrapper snapshots the source
//!  into a stage name beside the destination, makes it the worktree, applies the difference between the source's tree
//!  and the new `HEAD` with `git read-tree -m -u`, and swaps it into place. Any failure before the swap destroys the
//!  stage and completes the worktree the way Git does, with `git reset --hard`; the person only ever sees Git's result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ladder = beginLadder(plan, environment);
//! const git = await runGit(ladder.firstStep());
//! const result = await ladder.finish(git, runGit);
//! ```

use super::diagnostic_log::{debug, warn};
use super::fs_btrfs_ioctl::subvolume_destroy_at;
use super::transaction_git::GitContext;
use super::worktree_copy::{GitStatus, GitStep};
use super::worktree_subvolume::{CreatedSubvolume, prepare_subvolume_destination};
use super::worktree_add_plan::AddPlan;
use super::worktree_copy_stage_claim::{StageClaim, claim_stage_for_add};
use super::worktree_identity::WorktreeIdentity;
use super::worktree_snapshot_source::{SnapshotSource, clone_refusal, common_source, snapshot_blocker, snapshot_refusal};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  Which rung the ladder chose before Git ran, with what it prepared for it.
/// Why:
///   The snapshot rung holds the claim on its stage name, which must be released on every path out; the rungs that leave
///  Git or the wrapper to fill the destination prepare an empty subvolume there when they can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rung = { kind: 'git'; subvolume?: CreatedSubvolume } | { kind: 'clone'; source: SnapshotSource; subvolume?: CreatedSubvolume; claim: StageClaim } | { kind: 'snapshot'; source: SnapshotSource; claim: StageClaim };
/// ```
pub(super) enum Rung {
    /// Git's own checkout, into an empty subvolume the wrapper made when it could.
    Git(Option<CreatedSubvolume>),
    /// Clones of the source's tracked files, Git running without a checkout first; the claim records the add durably, so
    ///  an interrupted one is removed by the next command.
    Clone(Box<SnapshotSource>, Option<CreatedSubvolume>, StageClaim),
    /// A snapshot of the source, Git running without a checkout first.
    Snapshot(Box<SnapshotSource>, StageClaim),
}

/// What:
///  The ladder of one invocation.
/// Why:
///   It carries the plan and the chosen rung from before Git to after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ladder = { plan?: AddPlan; rung: Rung };
/// ```
pub struct Ladder {
    /// The destination plan, absent when the invocation is not a form the ladder serves.
    pub(super) plan: Option<AddPlan>,
    /// The chosen rung.
    pub(super) rung: Rung,
}

/// What:
///  What the ladder tells the lifecycle after Git: Git's status as the person should see it, and the destination whose
///  ignored state the rung already carried.
/// Why:
///   A snapshot carries the source's ignored state, so the lifecycle's own copy skips that destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LadderResult = { git: GitStatus; carried?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LadderResult {
    /// Git's status for the person.
    pub git: GitStatus,
    /// The destination that received the source's ignored state with the snapshot.
    pub carried: Option<PathBuf>,
}

/// What:
///  Choose the rung for one invocation and prepare it: a snapshot source and the claim on its stage, a clone source, or an
///  empty subvolume at the destination for Git to fill.
/// Why:
///   Every check that can be made before Git runs is made now, cheapest first, so a refused rung costs no extra Git run and
///  each refusal leaves its reason in the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseRung(plan: AddPlan, context: GitContext, identity: WorktreeIdentity): Rung;
/// ```
fn choose_rung(plan: &AddPlan, context: &GitContext, identity: &WorktreeIdentity) -> Rung {
    let snapshot_refusal: Option<String> = snapshot_refusal(identity, plan);
    let clone_refusal: Option<String> = clone_refusal(identity, plan);
    let reasons = |reason: &str| { debug(TAG, format!("{}: Git checks the worktree out ({reason}; snapshot: {snapshot_refusal:?}; clone: {clone_refusal:?})", plan.destination.display()).as_str()); };
    if snapshot_refusal.is_some() && clone_refusal.is_some() {
        reasons("neither a snapshot nor clones apply");
        return Rung::Git(prepare_subvolume_destination(plan));
    }
    let source: SnapshotSource = match common_source(context, identity) {
        Ok(found) => found,
        Err(reason) => {
            reasons(reason.as_str());
            return Rung::Git(prepare_subvolume_destination(plan));
        }
    };
    let snapshot_blocked: Option<String> = snapshot_refusal.clone().or_else(|| { return snapshot_blocker(&source); });
    if snapshot_blocked.is_none() {
        match claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()) {
            Ok(claim) => return Rung::Snapshot(Box::new(source), claim),
            Err(error) => debug(TAG, format!("the stage cannot be claimed ({}); no snapshot", error.0).as_str()),
        }
    } else {
        debug(TAG, format!("no snapshot for {} ({:?})", plan.destination.display(), snapshot_blocked).as_str());
    }
    if clone_refusal.is_none() {
        // The record comes first: without it a killed add could not be removed, so the rung is not tried.
        match claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()) {
            Ok(claim) => return Rung::Clone(Box::new(source), prepare_subvolume_destination(plan), claim),
            Err(error) => debug(TAG, format!("the add cannot be recorded ({}); no clones", error.0).as_str()),
        }
    }
    reasons("no rung applies");
    return Rung::Git(prepare_subvolume_destination(plan));
}

/// What:
///  Choose the rung for one invocation and prepare it; see `choose_rung`.
/// Why:
///   No plan means a form the ladder does not serve, which Git runs as the person wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginLadder(plan: AddPlan | undefined, context: GitContext, identity: WorktreeIdentity): Ladder;
/// ```
pub fn begin_ladder(plan: Option<&AddPlan>, context: &GitContext, identity: &WorktreeIdentity) -> Ladder {
    let Some(planned) = plan else { return Ladder { plan: None, rung: Rung::Git(None) } };
    return Ladder { plan: Some(planned.clone()), rung: choose_rung(planned, context, identity) };
}

/// Destroy a stage subvolume, or unlink it when the mount does not allow the destroy.
pub(super) fn discard_stage(stage: &Path) {
    if std::fs::symlink_metadata(stage).is_err() {
        return;
    }
    if subvolume_destroy_at(stage).is_ok() {
        return;
    }
    if let Err(error) = super::subvolume_remove::remove_tree(stage) {
        warn(TAG, format!("the stage {} could not be removed ({error}); the next command of the repository removes it", stage.display()).as_str());
    }
}

impl Ladder {
    /// What:
    ///  The first Git run the lifecycle starts: the person's arguments, with `--no-checkout` for the snapshot rung.
    /// Why:
    ///   Git then validates, registers and configures the worktree, and leaves the checkout to the rung.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ladder.firstStep(); // 'person' | { withoutCheckout: insertAt }
    /// ```
    pub fn first_step(&self) -> GitStep<'static> {
        return match (&self.rung, &self.plan) {
            (Rung::Snapshot(..) | Rung::Clone(..), Some(plan)) => GitStep::WithoutCheckout(plan.insert_at),
            _ => GitStep::Person,
        };
    }
}

/// Ladder run controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_run_tests.rs"]
mod tests;
