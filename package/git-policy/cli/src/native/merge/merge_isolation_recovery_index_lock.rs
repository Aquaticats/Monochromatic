//! What:
//!  Whether the recovery of an interrupted merge may write the index now:
//!  the wait a merge that has part of itself
//!  in the checkout makes,
//!  the one look a merge that has nothing there makes,
//!  and the clearing of a lock a process that no
//!  longer runs left.
//! Why:
//!  `git update-index` takes `index.lock` and dies when another program holds it,
//! so the decision that reads the lock
//!  and the decision that writes are the same decision;
//!  it lives in a file of its own so the steps that write the index stay
//!  under the package's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mayWrite = (gitDir, installedBefore, stderr) => installedBefore ? mayInstall(wait(gitDir)) : !lockStands(gitDir);
//! ```

use crate::index_holder_wait::{
    CLEARS_A_DEAD_OWNERS_LOCK, IndexLook, IndexWait, RECOVERY_CONSEQUENCE, look_at_the_index_lock_once, wait_for_index_release_while_finishing_a_merge,
};
use crate::index_lock_evidence::dead_owner_diagnostic;
use crate::index_lock_holders::scan_platform_holders_bounded;
use crate::service_index_placement::SYSTEM;
use crate::service_index_unheld_removal::{Unheld, remove_if_the_owner_is_gone};
use std::io::Write;
use std::path::Path;

/// What:
///  Whether the recovery may write the index of `git_dir` now,
///  clearing a lock a process that no longer runs left.
/// Why:
///   A merge that finished part of its own checkout must finish it before anything reads the checkout,
/// so it waits
///  for the index lock,
/// bounded and naming it.
/// A merge with nothing of itself in the checkout has nothing to wait for:
/// it is
///  kept on its record and its side ref,
/// and the next command finishes it once the lock is free.
/// Waiting cost every
///  command,
/// a `git rev-parse HEAD` included,
/// the whole ceiling again for as long as the lock stood
///  (finding 3 of the merge-and-pull audit of 2026-11-09).
///  A lock a gone process left is released by nobody,
/// so it is cleared on the commit service's proof of disuse,
/// exactly
///  as the merge's own path clears it,
/// and the merge goes on rather than every command exiting 2 until the person
///  deletes the file (finding 4 of the same audit).
/// A lock that proof does not cover stays where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayWrite = (gitDir, installedBefore, stderr) => installedBefore ? mayInstall(gitDir, wait(gitDir, stderr)) : lockOfANothingInstalledMergeStands(gitDir, stderr) === false;
/// ```
pub fn may_write<W: Write + ?Sized>(git_dir: &Path, installed_before: bool, out: &mut W) -> bool {
    let index: std::path::PathBuf = git_dir.join("index");
    return match installed_before {
        true => may_install(git_dir, wait_for_index_release_while_finishing_a_merge(index.as_path(), out)),
        false => !the_lock_of_a_merge_that_installed_nothing_stands_in_the_way(git_dir, index.as_path(), out),
    };
}

/// What:
///  Whether the merge may be written after this wait for the index lock.
/// Why:
///   A wait that ended with the lock still held by a program that lives means the index cannot be written,
/// and trying
///  would fail with Git's own sentence in place of the record;
/// leaving the record and writing nothing is safe,
/// because the
///  recovery is idempotent.
/// A wait that ended on a lock a gone process left is answered by clearing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayInstall = (gitDir, waited) => waited.kind === 'not-held' || waited.kind === 'released' || (waited.kind === 'owner-gone' && clearTheLockAGoneOwnerLeft(gitDir));
/// ```
pub fn may_install(git_dir: &Path, waited: IndexWait) -> bool {
    return match waited {
        IndexWait::NotHeld | IndexWait::Released(_) => true,
        IndexWait::GaveUp(_) | IndexWait::HeldByAnotherCommand(_) => false,
        IndexWait::OwnerGone(_) => clear_the_lock_a_gone_owner_left(git_dir),
    };
}

/// What:
///  Whether the index lock stands in the way of a merge that has nothing of itself in the checkout,
///  clearing it when a
///  process that no longer runs left it.
/// Why:
///  Nothing of such a merge is installed,
/// so a lock cannot be in its way,
/// and a lock a gone process left is
///  cleared on the commit service's proof of disuse.
/// The line that says so is the wait's own,
/// so a person who removes such a
///  lock by hand learns nothing they did not already read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = (gitDir, index, stderr) => match (lookAtTheIndexLockOnce(index)) {
///   case 'free': return false;
///   case 'left-by-a-gone-process': stderr.write(deadOwnerDiagnostic(...)); return !clearTheLockAGoneOwnerLeft(gitDir);
///   case 'held': return true;
/// };
/// ```
fn the_lock_of_a_merge_that_installed_nothing_stands_in_the_way<W: Write + ?Sized>(git_dir: &Path, index: &Path, out: &mut W) -> bool {
    return match look_at_the_index_lock_once(index) {
        IndexLook::Free => false,
        IndexLook::LeftByAGoneProcess(pid) => {
            let _noted = out.write_all(
                dead_owner_diagnostic(
                    crate::service_index_lock::lock_path_of(index).as_path(),
                    pid,
                    RECOVERY_CONSEQUENCE,
                    CLEARS_A_DEAD_OWNERS_LOCK,
                )
                .as_bytes(),
            );
            !clear_the_lock_a_gone_owner_left(git_dir)
        }
        IndexLook::Held => true,
    };
}

/// What:
///  Whether the index lock a process that no longer runs left is gone now.
/// Why:
///   The merge's own path and the recovery of an interrupted merge clear such a lock on one proof,
/// so neither can
///  drift from the other's conditions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleared = (gitDir) => removeIfTheOwnerIsGone(gitDir) === 'removed';
/// ```
fn clear_the_lock_a_gone_owner_left(git_dir: &Path) -> bool {
    return matches!(
        remove_if_the_owner_is_gone(&SYSTEM, git_dir.join("index").as_path(), &scan_platform_holders_bounded),
        Unheld::Removed(_)
    );
}
