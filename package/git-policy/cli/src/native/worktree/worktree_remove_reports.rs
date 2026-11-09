//! What:
//!  The three reports a removal can end with:
//! the command that forgets a registration whose directory is gone,
//! the report of a refused
//!  removal,
//! and the report of one that destroyed the files and could not remove the registration.
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
//! const incomplete = incompleteReport({ target, registration, commonDir, reason });
//! ```

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
/// and both ways to clear it.
/// Why:
///   The removal deletes the registration,
/// and the journal that names that copy lives in it,
/// so the copy must be
///  finished or discarded first,
/// and the person is given each command and told nothing was removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leftoverStageReport(target: string, converting: string): string;
/// ```
pub(crate) fn leftover_stage_report(target: &Path, converting: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: the worktree {target} has {converting} beside it, the copy a conversion of it left when it stopped. \n\
         Removing the worktree now would leave that copy behind with nothing that names it, so nothing was removed. \n\
         Finish the conversion with `git monochromatic-git-policy-cli convert-worktree-to-subvolume {target}`, or discard the copy with \n\
         `git monochromatic-git-policy-cli convert-worktree-to-subvolume --abandon {target}`, and then run the removal again.",
        target = target.display(),
        converting = converting.display()
    );
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
