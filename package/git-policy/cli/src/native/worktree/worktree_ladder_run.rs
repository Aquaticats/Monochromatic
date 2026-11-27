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
//! The rung this ladder chose,
//! the two steps a refused rung takes there,
//! and the choice itself,
//! are the sibling files `worktree_ladder_run_rung_and_refusal.rs` and `worktree_ladder_run_choose.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ladder = beginLadder(plan, environment);
//! const git = await runGit(ladder.firstStep());
//! const result = await ladder.finish(git, runGit);
//! ```

use crate::add_timing::note as note_timing;
use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::fs_btrfs_ioctl::subvolume_destroy_at;
use crate::transaction_git::GitContext;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy::{
    GitStatus,
    GitStep,
};
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_ladder_run_choose::choose_rung;
pub(crate) use crate::worktree_ladder_run_rung_and_refusal::Rung;
use crate::worktree_slow_route::SlowRoute;
use std::path::{
    Path,
    PathBuf,
};

/// Import the tag every line of this folder's ladder work carries.
use crate::worktree::worktree_ladder_finish_binding::TAG;

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
    /// The fence's size read this add started before its source check,
    /// until the fence takes it or the add's end settles it.
    /// Why:
    ///  It is this add's own value and never a process-wide slot,
    ///  so no other command and no other record can carry its answer or its failure.
    pub(crate) sizes_beside:
        Option<crate::worktree_clone_conversion_size_read::SizeReadBesideTheCheck>,
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
    pub trash: Option<crate::the_names_and_the_manifest_of_the_trash::PendingTrash>,
    /// The durable record of this add,
    /// for the copy that fills the destination to own until its journal exists.
    pub claim: Option<StageClaim>,
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
pub fn begin_ladder_holding(
    plan: Option<&AddPlan>,
    held: &mut Option<StageClaim>,
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Ladder {
    let Some(planned) = plan else {
        return Ladder {
            plan: None,
            rung: Rung::Git(None),
            slow: None,
            sizes_beside: None,
        };
    };
    let began: std::time::Instant = std::time::Instant::now();
    let mut sizes_beside: Option<
        crate::worktree_clone_conversion_size_read::SizeReadBesideTheCheck,
    > = None;
    let (rung, slow) = choose_rung(planned, context, identity, held, &mut sizes_beside);
    let chosen: &str = match &rung {
        Rung::Git(_) => "Git's checkout",
        Rung::Clone(source, ..) if source.dirty.is_some() => "a partial clone",
        Rung::Clone(..) => "clones",
        Rung::Snapshot(source, _claim, _fence) if source.dirty.is_some() => {
            "a snapshot with a restore"
        }
        Rung::Snapshot(..) => "a snapshot",
    };
    debug(
        TAG,
        format!(
            "{}: {chosen} chosen in {} ms",
            planned.destination.display(),
            began.elapsed().as_millis()
        )
        .as_str(),
    );
    note_timing("route", chosen);
    if let Some(route) = slow.as_ref() {
        note_timing(
            "slowRoute",
            format!("{}: {}", route.word(), route.detail()).as_str(),
        );
    }
    return Ladder {
        plan: Some(planned.clone()),
        rung,
        slow,
        sizes_beside,
    };
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
pub fn begin_ladder(
    plan: Option<&AddPlan>,
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Ladder {
    let mut held: Option<StageClaim> = None;
    return begin_ladder_holding(plan, &mut held, context, identity);
}

/// What:
///  The path `path` as the text a person reads.
/// Why:
///   The cleanup names every kept folder,
/// and one named function keeps that naming out of the closure check it would
///  otherwise be reported as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathText = (path) => path.display().toString();
/// ```
fn path_text(path: &Path) -> String {
    return path.display().to_string();
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
pub(crate) fn discard_stage(
    stage: &Path,
    made: Option<crate::fs_beneath::ObjectId>,
) {
    let Some(expected) = made else {
        warn(
            TAG,
            format!(
                "the stage {} was not identified and is left in place",
                stage.display()
            )
            .as_str(),
        );
        return;
    };
    let current = identity_of_stage_name(stage);
    if current.is_none() {
        return;
    }
    if current != Some(expected) {
        warn(
            TAG,
            format!(
                "the stage {} is not the directory this add made and is left in place",
                stage.display()
            )
            .as_str(),
        );
        return;
    }
    let kept: Vec<PathBuf> = crate::worktree_kept_folders::kept_folders(stage);
    if !kept.is_empty() {
        let places: Vec<String> = kept.iter().map(PathBuf::as_path).map(path_text).collect();
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
        warn(
            TAG,
            format!(
                "the stage {} could not be removed ({error}); it stays for the person to remove",
                stage.display()
            )
            .as_str(),
        );
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
            Rung::Git(Some(made)) | Rung::Clone(_, Some(made), _, _, _) => made.identity,
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
            (Rung::Snapshot(..) | Rung::Clone(..), Some(plan)) => {
                GitStep::WithoutCheckout(plan.insert_at)
            }
            _ => GitStep::Person,
        };
    }
}

/// Ladder run controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_run_tests.rs"]
mod tests;
