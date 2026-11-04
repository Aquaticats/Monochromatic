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

use crate::add_timing::{note as note_timing, step as timed};
use crate::diagnostic_log::{debug, warn};
use crate::fs_btrfs_ioctl::subvolume_destroy_at;
use crate::transaction_git::GitContext;
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_subvolume::{CreatedSubvolume, prepare_subvolume_destination};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage_for_add};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_slow_route::{SlowRoute, explain_cheap_refusal, explain_refusal, explain_unsupported};
use crate::worktree_snapshot_source::{SnapshotSource, clone_refusal, snapshot_blocker, snapshot_refusal, source_or_dirty};
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
pub(crate) enum Rung {
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
/// type Ladder = { plan?: AddPlan; rung: Rung; slow?: SlowRoute };
/// ```
pub struct Ladder {
    /// The destination plan, absent when the invocation is not a form the ladder serves.
    pub(crate) plan: Option<AddPlan>,
    /// The chosen rung.
    pub(crate) rung: Rung,
    /// Why the add will not be a snapshot, when it will not and a sentence would help; told to the person after Git ran.
    pub(crate) slow: Option<SlowRoute>,
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
///  empty subvolume at the destination for Git to fill; with the sentence that says why it is not the snapshot, when it is not.
/// Why:
///   Every check that can be made before Git runs is made now, cheapest first, so a refused rung costs no extra Git run and
///  each refusal leaves its reason in the log and, in words, with the person once Git has succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseRung(plan: AddPlan, context: GitContext, identity: WorktreeIdentity): { rung: Rung; slow?: SlowRoute };
/// ```
fn choose_rung(plan: &AddPlan, context: &GitContext, identity: &WorktreeIdentity) -> (Rung, Option<SlowRoute>) {
    let snapshot_refusal: Option<String> = timed("refusals", || { return snapshot_refusal(identity, plan); });
    let clone_refusal: Option<String> = timed("refusals", || { return clone_refusal(identity, plan); });
    let reasons = |reason: &str| { debug(TAG, format!("{}: Git checks the worktree out ({reason}; snapshot: {snapshot_refusal:?}; clone: {clone_refusal:?})", plan.destination.display()).as_str()); };
    // The cheap refusal comes first in the sentence: it is the structural one, which no commit or wait removes.
    let cheap: Option<SlowRoute> = snapshot_refusal.as_deref().and_then(|reason: &str| { return explain_cheap_refusal(identity, reason); });
    if snapshot_refusal.is_some() && clone_refusal.is_some() {
        reasons("neither a snapshot nor clones apply");
        return (Rung::Git(prepare_subvolume_destination(plan)), cheap);
    }
    let source: SnapshotSource = match timed("checkSource", || { return source_or_dirty(context, identity); }) {
        Ok(found) => found,
        Err(refusal) => {
            reasons(refusal.reason.as_str());
            let slow: Option<SlowRoute> = cheap.or_else(|| { return explain_refusal(context, identity, &refusal); });
            return (Rung::Git(prepare_subvolume_destination(plan)), slow);
        }
    };
    let snapshot_blocked: Option<String> = snapshot_refusal.clone().or_else(|| { return snapshot_blocker(&source); });
    let mut slow: Option<SlowRoute> = cheap.or_else(|| { return snapshot_blocked.as_deref().map(explain_unsupported); });
    if snapshot_blocked.is_none() {
        match timed("claimStage", || { return claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()); }) {
            Ok(claim) => return (Rung::Snapshot(Box::new(source), claim), None),
            Err(error) => {
                debug(TAG, format!("the stage cannot be claimed ({}); no snapshot", error.0).as_str());
                slow = Some(explain_unsupported(format!("the add could not be recorded durably ({})", error.0).as_str()));
            }
        }
    } else {
        debug(TAG, format!("no snapshot for {} ({:?})", plan.destination.display(), snapshot_blocked).as_str());
    }
    // Clones copy the tracked files of a source the checks proved clean; a source with changes is only ever snapshotted.
    if clone_refusal.is_none() && source.dirty.is_none() {
        // The record comes first: without it a killed add could not be removed, so the rung is not tried.
        match timed("claimStage", || { return claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()); }) {
            Ok(claim) => return (Rung::Clone(Box::new(source), prepare_subvolume_destination(plan), claim), slow),
            Err(error) => debug(TAG, format!("the add cannot be recorded ({}); no clones", error.0).as_str()),
        }
    }
    reasons("no rung applies");
    return (Rung::Git(prepare_subvolume_destination(plan)), slow);
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
    let Some(planned) = plan else { return Ladder { plan: None, rung: Rung::Git(None), slow: None } };
    let began: std::time::Instant = std::time::Instant::now();
    let (rung, slow) = choose_rung(planned, context, identity);
    let chosen: &str = match &rung {
        Rung::Git(_) => "Git's checkout",
        Rung::Clone(..) => "clones",
        Rung::Snapshot(source, _claim) if source.dirty.is_some() => "a snapshot with a restore",
        Rung::Snapshot(..) => "a snapshot",
    };
    debug(TAG, format!("{}: {chosen} chosen in {} ms", planned.destination.display(), began.elapsed().as_millis()).as_str());
    note_timing("route", chosen);
    if let Some(route) = slow.as_ref() {
        note_timing("slowRoute", format!("{}: {}", route.word(), route.detail()).as_str());
    }
    return Ladder { plan: Some(planned.clone()), rung, slow };
}

/// What:
///  Destroy the stage this process made, a subvolume or, where the mount does not allow the destroy, a tree, but only if the
///  name still holds the object that was made.
/// Why:
///   A name can be taken by another process; nothing is removed that is not the object this process made, and a stage that
///  cannot be proven is left and reported. The stage name is random and private, so the window between the check and the
///  destroy has no other party in practice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await discardStage(stage, made);
/// ```
pub(crate) fn discard_stage(stage: &Path, made: Option<crate::fs_beneath::ObjectId>) {
    let Some(expected) = made else {
        warn(TAG, format!("the stage {} was not identified and is left in place", stage.display()).as_str());
        return;
    };
    let current = stage.parent().and_then(|parent: &Path| { return crate::fs_beneath::open_root(parent).ok(); }).and_then(|held: std::fs::File| { return crate::fs_beneath::identity_at(&held, stage.file_name()?).ok(); });
    if current.is_none() {
        return;
    }
    if current != Some(expected) {
        warn(TAG, format!("the stage {} is not the directory this add made and is left in place", stage.display()).as_str());
        return;
    }
    if subvolume_destroy_at(stage).is_ok() {
        return;
    }
    if let Err(error) = crate::subvolume_remove::remove_tree(stage) {
        warn(TAG, format!("the stage {} could not be removed ({error}); it stays for the person to remove", stage.display()).as_str());
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
