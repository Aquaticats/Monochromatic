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
use super::worktree_copy_stage_claim::{StageClaim, claim_stage};
use super::worktree_identity::WorktreeIdentity;
use super::worktree_snapshot_source::{SnapshotSource, choose_snapshot_source};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  Which rung the ladder chose before Git ran, with what it prepared for it.
/// Why:
///   The snapshot rung holds the claim on its stage name, which must be released on every path out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rung = { kind: 'git'; subvolume?: CreatedSubvolume } | { kind: 'snapshot'; source: SnapshotSource; claim: StageClaim };
/// ```
pub(super) enum Rung {
    /// Git's own checkout, into an empty subvolume the wrapper made when it could.
    Git(Option<CreatedSubvolume>),
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
///  Choose the rung for one invocation and prepare it: a snapshot source and the claim on its stage, or an empty
///  subvolume at the destination for Git to fill.
/// Why:
///   Every check that can be made before Git runs is made now, so a refused rung costs no extra Git run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginLadder(plan: AddPlan | undefined, context: GitContext, identity: WorktreeIdentity): Ladder;
/// ```
pub fn begin_ladder(plan: Option<&AddPlan>, context: &GitContext, identity: &WorktreeIdentity) -> Ladder {
    let Some(planned) = plan else { return Ladder { plan: None, rung: Rung::Git(None) } };
    let common_dir: Option<&Path> = match identity {
        WorktreeIdentity::MainWorktree { common_dir, .. } | WorktreeIdentity::LinkedWorktree { common_dir, .. } | WorktreeIdentity::BareRepository { common_dir, .. } => Some(common_dir.as_path()),
        WorktreeIdentity::OutsideWorktree => None,
    };
    let chosen: Result<Rung, String> = choose_snapshot_source(context, identity, planned).and_then(|source: SnapshotSource| {
        let common: &Path = common_dir.ok_or_else(|| { return String::from("no common directory"); })?;
        let claim: StageClaim = claim_stage(common, planned.destination.as_path()).map_err(|error| { return format!("the stage cannot be claimed ({})", error.0); })?;
        return Ok(Rung::Snapshot(Box::new(source), claim));
    });
    let rung: Rung = match chosen {
        Ok(snapshot) => snapshot,
        Err(reason) => {
            debug(TAG, format!("no snapshot for {} ({reason}); Git checks the worktree out", planned.destination.display()).as_str());
            Rung::Git(prepare_subvolume_destination(planned))
        }
    };
    return Ladder { plan: Some(planned.clone()), rung };
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
            (Rung::Snapshot(..), Some(plan)) => GitStep::WithoutCheckout(plan.insert_at),
            _ => GitStep::Person,
        };
    }
}

/// Ladder run controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_run_tests.rs"]
mod tests;
