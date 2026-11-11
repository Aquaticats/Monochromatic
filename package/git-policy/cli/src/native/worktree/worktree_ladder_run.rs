//! What:
//!  The ladder around one `git worktree add` on Linux:
//! the choice of rung before real Git runs,
//! and,
//! after it
//!  returned,
//! the snapshot rung's steps with a fall back to Git's own checkout whenever one cannot finish.
//! Why:
//!  `doc/planning/cli-git-btrfs-worktree.md` designs it.
//! With a snapshot source Git runs with `--no-checkout` (branch
//!  validation,
//! administrative files,
//! configuration copy and every message stay Git's),
//! the wrapper snapshots the source
//!  into a stage name beside the destination,
//! makes it the worktree,
//! applies the difference between the source's tree
//!  and the new `HEAD` with `git read-tree -m -u`,
//! and swaps it into place.
//! Any failure before the swap destroys the
//!  stage and completes the worktree the way Git does,
//! with `git reset --hard`;
//! the person only ever sees Git's result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ladder = beginLadder(plan, environment);
//! const git = await runGit(ladder.firstStep());
//! const result = await ladder.finish(git, runGit);
//! ```

use crate::add_timing::{begin_step, end_step, note as note_timing};
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
///  Which rung the ladder chose before Git ran,
/// with what it prepared for it.
/// Why:
///   The snapshot rung holds the claim on its stage name,
/// which must be released on every path out;
/// the rungs that leave
///  Git or the wrapper to fill the destination prepare an empty subvolume there when they can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rung = { kind: 'git'; subvolume?: CreatedSubvolume } | { kind: 'clone'; source: SnapshotSource; subvolume?: CreatedSubvolume; claim: StageClaim } | { kind: 'snapshot'; source: SnapshotSource; claim: StageClaim };
/// ```
pub(crate) enum Rung {
    /// Git's own checkout,
    /// into an empty subvolume the wrapper made when it could.
    Git(Option<CreatedSubvolume>),
    /// Clones of the source's tracked files,
    /// Git running without a checkout first;
    /// the claim records the add durably,
    /// so
    ///  an interrupted one is removed by the next command.
    Clone(Box<SnapshotSource>, Option<CreatedSubvolume>, StageClaim),
    /// A snapshot of the source,
    /// Git running without a checkout first.
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
    /// The destination plan,
    /// absent when the invocation is not a form the ladder serves.
    pub(crate) plan: Option<AddPlan>,
    /// The chosen rung.
    pub(crate) rung: Rung,
    /// Why the add will not be a snapshot,
    /// when it will not and a sentence would help;
    /// told to the person after Git ran.
    pub(crate) slow: Option<SlowRoute>,
}

/// What:
///  What the ladder tells the lifecycle after Git:
/// Git's status as the person should see it,
/// the destination whose
///  ignored state the rung already carried,
/// the object the rung left at that destination,
/// the trash a snapshot left there,
/// and the claim that names this add.
/// Why:
///   A snapshot carries the source's ignored state,
/// so the lifecycle's own copy skips that destination.
/// The trash is only emptied
///  once the repair has run,
/// beside it,
/// so the lifecycle owns the emptying.
/// A rung that swapped its own snapshot into the destination
///  replaced the directory Git made,
/// so the object at the path is the snapshot and not the one the lifecycle reserved,
///  and only the rung can say which object that is.
/// The claim is handed over rather than released:
/// a rung that takes the claim from the caller would otherwise leave the add with no record between the end of its
///  checkout and the copy's own claim,
/// so a command killed in that handover would leave a worktree with no ignored state and nothing naming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LadderResult = { git: GitStatus; carried?: string; placed?: ObjectId; trash?: PendingTrash; claim?: StageClaim };
/// ```
#[derive(Debug)]
pub struct LadderResult {
    /// Git's status for the person.
    pub git: GitStatus,
    /// The destination that received the source's ignored state with the snapshot.
    pub carried: Option<PathBuf>,
    /// The object that destination names now,
    /// when a rung put its own there;
    /// the snapshot that was swapped into place.
    pub placed: Option<crate::fs_beneath::ObjectId>,
    /// The trash that snapshot left in the destination,
    /// when it left one.
    #[cfg(target_os = "linux")]
    pub trash: Option<crate::worktree_trash::PendingTrash>,
    /// The durable record of this add,
    /// for the copy that fills the destination to own until its journal exists.
    pub claim: Option<StageClaim>,
}

/// What:
///  Log why Git,
/// and not a snapshot or clones,
/// checks the worktree out.
/// Why:
///   One log line carries both refusals,
/// so a person reading the log sees why the cheap rungs were not used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// debug(`${plan.destination}: Git checks the worktree out (${reason}; snapshot: ${snapshotRefusal}; clone: ${cloneRefusal})`)
/// ```
fn log_git_checkout(plan: &AddPlan, reason: &str, snapshot_refusal: &Option<String>, clone_refusal: &Option<String>) {
    debug(TAG, format!("{}: Git checks the worktree out ({reason}; snapshot: {snapshot_refusal:?}; clone: {clone_refusal:?})", plan.destination.display()).as_str());
}

/// What:
///  The identity of what the stage's name leads to now,
/// or nothing when its parent cannot be opened or the name cannot be looked up.
/// Why:
///   Discarding a stage checks that the name still holds the object this process made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityAt(openRoot(parent(stage)), basename(stage)) ?? undefined
/// ```
fn identity_of_stage_name(stage: &Path) -> Option<crate::fs_beneath::ObjectId> {
    let parent: &Path = stage.parent()?;
    let held: std::fs::File = crate::fs_beneath::open_root(parent).ok()?;
    return crate::fs_beneath::identity_at(&held, stage.file_name()?).ok();
}

/// What:
///  Choose the rung for one invocation and prepare it:
/// a snapshot source and the claim on its stage,
/// a clone source,
/// or an
///  empty subvolume at the destination for Git to fill;
/// with the sentence that says why it is not the snapshot,
/// when it is not.
/// Why:
///   Every check that can be made before Git runs is made now,
/// cheapest first,
/// so a refused rung costs no extra Git run and
///  each refusal leaves its reason in the log and,
/// in words,
/// with the person once Git has succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseRung(plan: AddPlan, context: GitContext, identity: WorktreeIdentity): { rung: Rung; slow?: SlowRoute };
/// ```
fn choose_rung(plan: &AddPlan, context: &GitContext, identity: &WorktreeIdentity, held: &mut Option<StageClaim>) -> (Rung, Option<SlowRoute>) {
    let refusals_entered: bool = begin_step("refusals");
    let snapshot_refusal: Option<String> = snapshot_refusal(identity, plan);
    end_step(refusals_entered);
    let refusals_again_entered: bool = begin_step("refusals");
    let clone_refusal: Option<String> = clone_refusal(identity, plan);
    end_step(refusals_again_entered);
    // The cheap refusal comes first in the sentence: it is the structural one, which no commit or wait removes.
    let cheap: Option<SlowRoute> = match snapshot_refusal.as_deref() {
        Some(reason) => explain_cheap_refusal(identity, reason),
        None => None,
    };
    if snapshot_refusal.is_some() && clone_refusal.is_some() {
        log_git_checkout(plan, "neither a snapshot nor clones apply", &snapshot_refusal, &clone_refusal);
        return (Rung::Git(prepare_subvolume_destination(plan)), cheap);
    }
    let check_source_entered: bool = begin_step("checkSource");
    let check_source_outcome = source_or_dirty(context, identity);
    end_step(check_source_entered);
    let source: SnapshotSource = match check_source_outcome {
        Ok(found) => found,
        Err(refusal) => {
            log_git_checkout(plan, refusal.reason.as_str(), &snapshot_refusal, &clone_refusal);
            // The explanation is made only when no cheap one was found, as `or_else` did.
            let slow: Option<SlowRoute> = match cheap {
                Some(structural) => Some(structural),
                None => explain_refusal(context, identity, &refusal),
            };
            return (Rung::Git(prepare_subvolume_destination(plan)), slow);
        }
    };
    let snapshot_blocked: Option<String> = match snapshot_refusal.clone() {
        Some(reason) => Some(reason),
        None => snapshot_blocker(&source),
    };
    let mut slow: Option<SlowRoute> = match cheap {
        Some(structural) => Some(structural),
        None => snapshot_blocked.as_deref().map(explain_unsupported),
    };
    if snapshot_blocked.is_none() {
        // A claim the caller recorded before Git names this add's stage already, so the rung reuses it.
        let claim_stage_entered: bool = begin_step("claimStage");
        let claim_stage_outcome = match held.take() {
            Some(recorded) => Ok(recorded),
            None => claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()),
        };
        end_step(claim_stage_entered);
        match claim_stage_outcome {
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
        let claim_stage_again_entered: bool = begin_step("claimStage");
        let claim_stage_again_outcome = match held.take() {
            Some(recorded) => Ok(recorded),
            None => claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()),
        };
        end_step(claim_stage_again_entered);
        match claim_stage_again_outcome {
            Ok(claim) => return (Rung::Clone(Box::new(source), prepare_subvolume_destination(plan), claim), slow),
            Err(error) => debug(TAG, format!("the add cannot be recorded ({}); no clones", error.0).as_str()),
        }
    }
    log_git_checkout(plan, "no rung applies", &snapshot_refusal, &clone_refusal);
    return (Rung::Git(prepare_subvolume_destination(plan)), slow);
}

/// What:
///  Choose the rung for one invocation and prepare it;
/// see `choose_rung`.
/// Why:
///   No plan means a form the ladder does not serve,
/// which Git runs as the person wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginLadder(plan: AddPlan | undefined, context: GitContext, identity: WorktreeIdentity): Ladder;
/// ```
/// What:
///  Choose the rung for one invocation and prepare it;
/// see `choose_rung`.
/// Why:
///   No plan means a form the ladder does not serve,
/// which Git runs as the person wrote it,
/// and a claim the caller recorded is left to the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginLadder(plan: AddPlan | undefined, held: StageClaim | undefined, context: GitContext, identity: WorktreeIdentity): Ladder;
/// ```
pub fn begin_ladder_holding(plan: Option<&AddPlan>, held: &mut Option<StageClaim>, context: &GitContext, identity: &WorktreeIdentity) -> Ladder {
    let Some(planned) = plan else { return Ladder { plan: None, rung: Rung::Git(None), slow: None } };
    let began: std::time::Instant = std::time::Instant::now();
    let (rung, slow) = choose_rung(planned, context, identity, held);
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
///  Choose the rung for one invocation and record it through a claim this call makes;
/// see `begin_ladder_holding`.
/// Why:
///   A caller that recorded no stage name before Git hands the ladder nothing to reuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginLadder(plan: AddPlan | undefined, context: GitContext, identity: WorktreeIdentity): Ladder;
/// ```
pub fn begin_ladder(plan: Option<&AddPlan>, context: &GitContext, identity: &WorktreeIdentity) -> Ladder {
    let mut held: Option<StageClaim> = None;
    return begin_ladder_holding(plan, &mut held, context, identity);
}

/// What:
///  Destroy the stage this process made,
/// a subvolume or,
/// where the mount does not allow the destroy,
/// a tree,
/// but only if the
///  name still holds the object that was made.
/// Why:
///   A name can be taken by another process;
/// nothing is removed that is not the object this process made,
/// and a stage that
///  cannot be proven is left and reported.
/// The stage name is random and private,
/// so the window between the check and the
///  destroy has no other party in practice.
/// A stage that holds a folder the stand-in sweep kept is left as well,
/// with every kept path named:
/// those
///  bytes are what the add decided never to delete (`never discard user data`),
/// and destroying the stage would
///  take them.
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
    let current = identity_of_stage_name(stage);
    if current.is_none() {
        return;
    }
    if current != Some(expected) {
        warn(TAG, format!("the stage {} is not the directory this add made and is left in place", stage.display()).as_str());
        return;
    }
    let kept: Vec<PathBuf> = crate::worktree_snapshot_placeholders::kept_folders(stage);
    if !kept.is_empty() {
        let places: Vec<String> = kept.iter().map(|path: &PathBuf| { return path.display().to_string(); }).collect();
        warn(
            TAG,
            format!(
                "monochromatic-git-policy-cli: the stage {} is left in place because it holds what this add kept: {}; copy what you need from there and remove the folder yourself",
                stage.display(),
                places.join(", ")
            )
            .as_str(),
        );
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
    ///  The identity of the destination this ladder made before real Git,
    /// when it made one.
    /// Why:
    ///   The rungs that prepare an empty destination settle it themselves,
    /// and the lifecycle must not reserve a second
    ///  one on top of it;
    /// the identity is what it needs to tell the two apart.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ladder.destinationIdentity(); // the subvolume the rung made, if any
    /// ```
    pub fn destination_identity(&self) -> Option<crate::fs_beneath::ObjectId> {
        return match &self.rung {
            Rung::Git(Some(made)) | Rung::Clone(_, Some(made), _) => made.identity,
            _ => None,
        };
    }

    /// What:
    ///  The first Git run the lifecycle starts:
    /// the person's arguments,
    /// with `--no-checkout` for the snapshot rung.
    /// Why:
    ///   Git then validates,
    /// registers and configures the worktree,
    /// and leaves the checkout to the rung.
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
