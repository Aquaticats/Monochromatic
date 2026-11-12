//! What:
//!  The reports a removal can end with:
//! the command that forgets a registration whose directory is gone,
//! the report of a refused
//!  removal,
//! the report of one that destroyed the files and could not remove the registration,
//! the report of one
//!  refused because the snapshots this program made ahead of an add cannot be taken,
//! and the reports of one refused because
//!  the registration holds a volume it cannot empty.
//! Why:
//!  Each report is text a person acts on,
//! so each names the paths and every command that finishes the job,
//! and they live
//!  together instead of inside the removal's steps.
//! The refusal and the incomplete ending are different outcomes:
//! one changed nothing,
//! the other destroyed
//!  files and left a registration,
//! and neither may be reported as the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prune = pruneCommand(commonDir);
//! const refused = leftoverStageReport(target, converting);
//! const blocked = snapshotsAheadReport({ target, admin, reason });
//! const incomplete = incompleteReport({ target, registration, commonDir, reason });
//! const held = registrationVolumeReport({ target, admin, found });
//! const unreadable = unwalkableRegistrationReport({ target, admin, reason });
//! ```

use crate::subvolume_remove_walk::Boundary;
use crate::worktree_remove_registry::Registration;
use std::path::Path;

/// The command that forgets a registration whose directory is gone,
/// aimed at the repository of `common_dir`.
pub(crate) fn prune_command(common_dir: &Path) -> String {
    let main_checkout: Option<&str> = match common_dir.to_str() {
        Some(text) => text.strip_suffix("/.git"),
        None => None,
    };
    return match main_checkout {
        Some(main) => format!("git -C {main} worktree prune"),
        None => format!("git --git-dir={} worktree prune", common_dir.display()),
    };
}

/// What:
///  The report of a removal refused because a conversion's copy lies beside the worktree:
/// what is there,
/// why removing the worktree now
///  would orphan it,
/// and the commands that clear it,
/// which depend on how far the conversion got.
/// Why:
///   The removal deletes the registration,
/// and the journal that names that copy lives in it,
/// so the copy must be
///  finished or discarded first.
/// `--abandon` refuses every conversion at or past its exchange ("past the swap:
/// run the command again"),
/// so naming it for a post-swap copy sends the person to a command that cannot work;
/// the
///  remedy that does work there is the conversion itself,
/// which resumes,
/// rewrites the index,
/// removes the copy and drops
///  the journal.
/// A worktree with no journal beside it has no conversion to finish and only the copy a removal left:
/// that is the
///  one `--abandon` discards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leftoverStageReport(target: string, converting: string, stage: StageState): string;
/// ```
pub(crate) fn leftover_stage_report(target: &Path, converting: &Path, stage: StageState) -> String {
    let remedy: String = match stage {
        StageState::PastTheSwap => format!(
            "Finish the conversion with `git monochromatic-git-policy-cli convert-worktree-to-subvolume {target}`, which completes it in place (the index, then the copy, then the journal), and then run the removal again. `--abandon` cannot help here: it refuses every conversion at or past its exchange.",
            target = target.display()
        ),
        StageState::BeforeTheSwap => format!(
            "Finish the conversion with `git monochromatic-git-policy-cli convert-worktree-to-subvolume {target}`, or discard what it copied with `git monochromatic-git-policy-cli convert-worktree-to-subvolume --abandon {target}`, and then run the removal again.",
            target = target.display()
        ),
        StageState::NoJournal => format!(
            "Discard the copy with `git monochromatic-git-policy-cli convert-worktree-to-subvolume --abandon {target}`, and then run the removal again.",
            target = target.display()
        ),
    };
    return format!(
        "monochromatic-git-policy-cli: the worktree {target} has {converting} beside it, the copy a conversion of it left when it stopped. \n\
         Removing the worktree now would leave that copy behind with nothing that names it, so nothing was removed. \n\
         {remedy}",
        target = target.display(),
        converting = converting.display()
    );
}

/// What:
///  How far the conversion of one worktree got,
/// as far as its journal says.
/// Why:
///   The remedy the report names depends on it:
/// before the exchange the stage is the copy `--abandon` may discard,
/// and at or
///  past the exchange the stage is the original tree the conversion itself removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageState = 'past-the-swap' | 'before-the-swap' | 'no-journal';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StageState {
    /// A journal records the exchange or a later phase.
    PastTheSwap,
    /// A journal records an earlier phase.
    BeforeTheSwap,
    /// No journal is there:
    /// the copy is one a removal left behind.
    NoJournal,
}

/// What:
///  The stage state of one worktree,
/// from the journal in its registration.
/// Why:
///   A conversion killed right after its exchange has its journal still at `Synced`,
/// because that phase's checkpoint
///  comes before the phase is recorded,
/// so the journal alone would name the remedy for a copy that is no longer
///  there.
/// The journal and the two names together say where the original tree is,
/// which is what the resume itself asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = stageState({ admin, target, converting });
/// ```
pub(crate) fn stage_state(admin: &Path, target: &Path, converting: &Path) -> StageState {
    let journal = match crate::worktree_convert_journal::Journal::load(admin) {
        Ok(Some(found)) => found,
        // A journal that cannot be read is not a conversion this report can finish; the copy is treated as one a removal
        // left, which is the state `--abandon` reaches.
        Ok(None) | Err(_) => return StageState::NoJournal,
    };
    if journal.phase >= crate::worktree_convert_journal::Phase::Exchanged {
        return StageState::PastTheSwap;
    }
    if journal.phase == crate::worktree_convert_journal::Phase::Synced
        && crate::worktree_convert_phases::old_tree_is_at(&journal, target, converting) == crate::worktree_convert_phases::Where::AtConverting
    {
        return StageState::PastTheSwap;
    }
    return StageState::BeforeTheSwap;
}

/// What:
///  The report of a removal that destroyed the files but left the registration:
/// what is gone,
/// what is left and where,
/// why,
///  and each way to finish.
/// Why:
///   Git would now answer that the path is not a working tree,
/// so the person needs the leftover's name and the command
///  that removes it,
/// and must not be told that the removal succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function incompleteReport(args: { target: string; admin: string; commonDir: string; reason: string }): string;
/// ```
pub(crate) fn incomplete_report(target: &Path, registration: &Registration, common_dir: &Path, reason: &str) -> String {
    let prune: String = prune_command(common_dir);
    return format!(
        "monochromatic-git-policy-cli: the worktree {target} was removed (its subvolume was destroyed), but its registration {admin} was not: {reason}.\n\
         Git still lists the worktree as missing. To finish, run `{prune}`; it forgets every registration whose directory is \
         missing, and `{prune} --dry-run --verbose` lists them first.\n\
         If that fails the same way, make {admin} and {worktrees} writable by you (`chmod --recursive u+rwx {admin}`) and \
         remove {admin} yourself (`rm --recursive {admin}`).\n\
         Running `git worktree remove` for {target} again cannot finish this: the directory is already gone.",
        target = target.display(),
        admin = registration.admin.display(),
        worktrees = registration.worktrees.display(),
        reason = reason,
        prune = prune
    );
}

/// What:
///  The report of a removal refused because the Git directory of the worktree holds a subvolume or a mount point this
///  program did not make.
/// Why:
///   The registration is emptied after the destroy and that emptying refuses a nested subvolume,
/// so a volume below it
///  that this program does not own is the one state in which the destroy would take the worktree's files and leave the
///  registration behind,
/// with a notice on every later command until a person acted.
/// The proof is made before anything irreversible,
/// so the worktree and every file of it are still there,
/// and
///  the report names the volume and each way forward,
/// exactly as the refusal for a volume below the worktree's own
///  root does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registrationVolumeReport(target: string, admin: string, found: Boundary): string;
/// ```
pub(crate) fn registration_volume_report(target: &Path, admin: &Path, found: &Boundary) -> String {
    return format!(
        "monochromatic-git-policy-cli: the Git directory {admin} of the worktree {target} holds {inner}, which is {what}, \
         and a registration that holds one cannot be emptied, so the removal of {target} could never finish. Nothing was \
         removed, and nothing of {inner} was deleted.\n\
         Move {inner} out of {admin}, or delete it yourself (`btrfs subvolume delete {inner}`, or `umount {inner}` for a \
         mount point), and then run this removal again; to keep it, leave it there and remove the worktree by hand, one \
         file at a time.",
        target = target.display(),
        admin = admin.display(),
        inner = found.path.display(),
        what = found.what
    );
}

/// What:
///  The report of a removal refused because the registration of the worktree cannot be read at all.
/// Why:
///   Nothing proves the registration can be emptied,
/// and the destroy cannot be undone,
/// so the removal stops while
///  the worktree is whole and names what could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unwalkableRegistrationReport(target: string, admin: string, reason: string): string;
/// ```
pub(crate) fn unwalkable_registration_report(target: &Path, admin: &Path, reason: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the Git directory {admin} of the worktree {target} cannot be read, so nothing proves \
         the registration could be emptied after the destroy: {reason}. Nothing was removed, and nothing of {target} changed.\n\
         Make {admin} readable by you (`chmod --recursive u+rwx {admin}`) and run this removal again.",
        target = target.display(),
        admin = admin.display(),
        reason = reason
    );
}

/// What:
///  The report of a removal refused because a subvolume this program left for its commit service to delete cannot be
///  moved out of the worktree.
/// Why:
///   The leftover is this program's own and holds nothing a person still owns:
/// the removal that renamed it
///  proved the worktree clean and unlocked,
/// and the record of its delete outlives that command.
/// So the person is told what stands in the way of taking it,
/// and that nothing of the worktree was removed,
///  rather than being sent to delete it by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pendingDeletionReport(target: string, reason: string): string;
/// ```
pub(crate) fn pending_deletion_report(target: &Path, reason: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the worktree {target} holds a subvolume this program already renamed for its \
         commit service to delete, and it cannot be moved out of the tree: {reason}.\n\
         Nothing was removed, and nothing of {target} changed. The subvolume holds no work of yours: the removal that \
         renamed it proved the worktree it came from clean and unlocked, and its delete is recorded. Fix what the reason \
         names and run this removal again.",
        target = target.display(),
        reason = reason
    );
}

/// What:
///  The report of a removal refused because the snapshots the wrapper made ahead of an add cannot be taken.
/// Why:
///   A worktree that was the source of one `git worktree add` keeps those snapshots in its administrative
///  directory,
/// the removal destroys the worktree's subvolume before it empties that directory,
/// and the emptying
///  refuses a nested subvolume,
/// so a pool that cannot go first has to stop the removal while nothing has happened yet
///  (a Sonnet finding,
/// F1).
///  The pool is the wrapper's own,
/// so the person is told which folder holds it and the one command that takes each member,
///  and that nothing of the worktree was removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotsAheadReport(target: string, admin: string, reason: string): string;
/// ```
pub(crate) fn snapshots_ahead_report(target: &Path, admin: &Path, reason: &str) -> String {
    let pool: std::path::PathBuf = crate::worktree_snapshot_ahead::pool_folder(admin);
    return format!(
        "monochromatic-git-policy-cli: the worktree {target} holds the snapshots this program made ahead of an add, in \
         {pool}, and they cannot be removed: {reason}.\n\
         Removing the worktree now would leave them behind with no registration that names them, so nothing was removed \
         and nothing of the worktree changed.\n\
         Fix what the reason names and run the removal again; to take the snapshots yourself, delete every entry of {pool} \
         (`btrfs subvolume delete <entry>`), and then run the removal again.",
        target = target.display(),
        pool = pool.display(),
        reason = reason
    );
}
