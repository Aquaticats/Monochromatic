//! What:
//!  Reading `Cargo.lock` before a `cargo clean` and putting it back as it was afterwards,
//! and keeping the bytes a clean displaced where they cannot be attributed to the clean.
//! Why:
//!  A clean may rewrite the lock file,
//! and a new worktree must stay as checked out;
//! the two steps need nothing from either clean,
//! so they live in a file of their own that both import:
//! `package_repair_cargo.rs`,
//! which cleans what the worktree already has,
//! and `package_repair_prewarm_job.rs`,
//! which cleans the copy the commit service fills a build directory from after the add returns.
//! The add's clean runs while the worktree it fills was made by the same command and nobody has it yet,
//! which `SPEC.md` records as the reason its put-back stands
//! ("A committed `Cargo.lock` that disagrees with its manifest"),
//! so it puts the checked-out lock back over whatever the clean left (`restore_lock`).
//! The filling runs after the add has returned and the person may be working in the worktree,
//! so it accepts the checked-out bytes back only over bytes its own resolution wrote
//! (`restore_only_the_cleans_own`),
//! and the clean it runs is the one given `--locked` (`package_repair_prewarm_job.rs`).
//!   The clean rewrites the lock because the committed one no longer matches its manifest,
//! and the worktree it runs in may be one another command is already working in,
//! so the bytes standing there when it ends can be Cargo's own rewrite
//!  or a command or person's newer write,
//! and nothing at that moment can tell the two apart.
//! A comparison of the file with the snapshot cannot say who wrote it,
//!  so the bytes that stand there are kept whole under the repository's Git directory
//!  before the checked-out ones are put back
//! (the owner's rule "never discard user data";
//! a review of `ee545033ad`,
//! finding 3,
//!  found the restore writing over a newer edit without a copy of it).
//! A lock the clean left unchanged is not written at all,
//!  and a lock whose displaced bytes cannot be kept is left exactly as it was found,
//! because putting the checked-out bytes back would then be the only place those bytes existed.
//! The reading that decided all this is a moment of its own,
//! and a person may write the lock after it:
//! a write made over the file by an earlier version was in neither the copy nor the file afterwards.
//! So the checked-out bytes are staged in the displaced directory and exchanged with the lock in one step
//! (`renameat2(RENAME_EXCHANGE)`,
//! `rename_that_never_replaces.rs`),
//! which hands back the file that stood there at that instant:
//! the file the reading named goes,
//! and any other is put back where the person wrote it while the checked-out bytes stay where the line names them
//! (`replace_the_clean_s_own`).
//! A lock the clean created is taken away by a move and put back if it was written since,
//! and one the clean removed is written with `create_new`,
//! so no step here replaces a file nobody kept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = readLock(root); await clean(); restoreLock({ root, before, keptIn: gitDir });
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_cargo_lock_names.rs`:
//!   the names a displaced lock's bytes are kept under.
//! - `package_repair_cargo_lock_keeping_the_bytes.rs`:
//!   the copy that survives the clean,
//!   and the staging of the checked-out bytes beside it.
//! - `package_repair_cargo_lock_never_replacing.rs`:
//!   putting the checked-out lock back without replacing a file nobody kept.

/// Import the debug log of the repair.
use crate::diagnostic_log::debug;
/// Import the copy that survives the clean.
pub(super) use crate::package_repair_cargo_lock_keeping_the_bytes::keep_the_bytes;
/// Import the names a displaced lock's bytes are kept under,
/// which this file's own reading and putting back name.
pub use crate::package_repair_cargo_lock_names::{
    DISPLACED_DIRECTORY,
    LOCK_FILE,
};
/// Import the steps that put the checked-out bytes back,
/// and the answer each of them gives.
pub(super) use crate::package_repair_cargo_lock_never_replacing::{
    Restored,
    replace_the_clean_s_own,
};
/// Import the borrowed and owned filesystem paths a lock is read from and written to.
use std::path::{
    Path,
    PathBuf,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairCargoLock";
/// What:
///  The bytes of the workspace's `Cargo.lock`,
/// or none when it has none.
/// Why:
///   `cargo clean --package` resolves the workspace,
/// and Cargo rewrites a lock that no longer matches its manifest;
/// the
///  new worktree must stay as checked out,
/// so the file is put back afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = readLock(root); // Uint8Array | undefined
/// ```
pub fn read_lock(root: &Path) -> Option<Vec<u8>> {
    return std::fs::read(root.join(LOCK_FILE)).ok();
}

/// What:
///  Put `Cargo.lock` back as `before` read it,
/// keeping the bytes that stood there when the clean ended,
/// and answer the line the person reads when anything was kept.
/// Why:
///   A write is made only when the content changed,
/// so an untouched lock keeps its file time.
///   The bytes that changed it are kept before they are displaced,
/// because the change may be Cargo's rewrite of a lock that no longer matches its manifest
///  or another command's newer write,
/// and the repair may not decide which by guessing:
/// a copy under the worktree's Git directory holds either,
/// and the checked-out lock comes back so the worktree is not left dirty by a clean
///  (`DISPLACED_DIRECTORY`).
/// A lock whose bytes cannot be kept is left exactly as it was found,
/// and the line says so,
/// because the checked-out bytes are in the branch and the bytes standing there are not.
/// A lock that cannot be put back is logged,
///  and left for the person:
/// it is the worktree's own file and the checkout shows the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = restoreLock({ root, before, keptIn }); // string | undefined
/// ```
pub fn restore_lock(
    root: &Path,
    before: Option<&[u8]>,
    kept_in: Option<&Path>,
) -> Option<String> {
    return put_the_checkout_back(root, before, kept_in, None);
}

/// What:
///  Put `Cargo.lock` back as `before` read it only when the bytes standing there are the ones the clean's own
/// resolution of the workspace wrote,
/// and leave whatever else stands there exactly as it is found.
/// Why:
///   This face is the commit service's,
/// and it runs after the add returned,
/// when the person may be working in the worktree:
/// a write standing there when the clean ends may be theirs,
/// and replacing it would be the repair discarding what a person wrote.
/// `cargo clean` rewrites a lock that no longer matches its manifest,
/// so its caller asks Cargo for that resolution before the clean runs and gives the clean `--locked`,
/// which keeps the clean itself from writing the lock at all;
/// the bytes that resolution wrote are Cargo's,
/// so a file equal to them is the clean's own and the checked-out bytes may go back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = restoreOnlyTheCleansOwn({ root, before, keptIn, theCleansOwn }); // string | undefined
/// ```
pub fn restore_only_the_cleans_own(
    root: &Path,
    before: Option<&[u8]>,
    kept_in: Option<&Path>,
    the_cleans_own: &[u8],
) -> Option<String> {
    return put_the_checkout_back(root, before, kept_in, Some(the_cleans_own));
}

/// What:
///  `restore_lock`'s work,
/// with the write it may displace judged first when the caller knows which bytes its own clean wrote.
/// Why:
///   Both faces keep the bytes they displace and put the checked-out lock back;
/// only the service can tell a clean's own write from a person's,
///  and that answer is read before anything is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putTheCheckoutBack({ root, before, keptIn, onlyWhen }): string | undefined;
/// ```
fn put_the_checkout_back(
    root: &Path,
    before: Option<&[u8]>,
    kept_in: Option<&Path>,
    only_when: Option<&[u8]>,
) -> Option<String> {
    let path: PathBuf = root.join(LOCK_FILE);
    let after: Option<Vec<u8>> = std::fs::read(&path).ok();
    if after.as_deref() == before {
        return None;
    }
    // A chained condition keeps the two readings of one write together: whose bytes stand there, and whether they are ours.
    if let Some(the_cleans_own) = only_when
        && after.as_deref() != Some(the_cleans_own)
    {
        return Some(format!(
            "the {LOCK_FILE} of {} was written while the clean ran and does not hold what that clean's own resolution wrote, so it is left exactly as \
             it is found; `git diff -- {LOCK_FILE}` in that worktree shows it",
            path.display()
        ));
    }
    let kept: Option<PathBuf> = match (kept_in, after.as_deref()) {
        // Nothing stands there, so nothing is displaced and nothing has to be kept.
        (_no_place, None) => None,
        (None, Some(_bytes)) => {
            return Some(format!(
                "the {LOCK_FILE} of {} changed while the clean ran, and no Git directory was named to keep those bytes in, so the file is left \
                 exactly as it is found; `git diff -- {LOCK_FILE}` in that worktree shows what changed, and the branch holds the bytes it was \
                 checked out with",
                path.display()
            ));
        }
        (Some(directory), Some(bytes)) => match keep_the_bytes(directory, path.as_path(), bytes) {
            Ok(copy) => Some(copy),
            Err(reason) => {
                return Some(format!(
                    "the {LOCK_FILE} of {} changed while the clean ran, and the bytes that changed it could not be kept ({reason}), so the file \
                     is left exactly as it is found; `git diff -- {LOCK_FILE}` in that worktree shows what changed, and the branch holds the \
                     bytes it was checked out with",
                    path.display()
                ));
            }
        },
    };
    let restored: Restored =
        replace_the_clean_s_own(path.as_path(), before, after.as_deref(), kept_in);
    return match (restored, kept) {
        (Restored::TheirWriteStays(Some(kept_lock)), _) => Some(format!(
            "the {LOCK_FILE} of {} was written again while the checked-out lock was being put back, so the file is left exactly as it is found; \
             the checked-out lock is kept at {}, and `git diff -- {LOCK_FILE}` in that worktree shows the difference",
            root.display(),
            kept_lock.display()
        )),
        (Restored::TheirWriteStays(None), _) => Some(format!(
            "the {LOCK_FILE} of {} was written again while the checked-out lock was being put back, so the file is left exactly as it is found; the \
             branch holds the bytes it was checked out with, and `git diff -- {LOCK_FILE}` in that worktree shows the difference",
            root.display()
        )),
        (Restored::LeftAsFound(reason), _) => {
            // The step names what it could not do and what became of every byte, so this line adds only where to look.
            debug(
                TAG,
                format!(
                    "{} was not put back as checked out ({reason})",
                    path.display()
                )
                .as_str(),
            );
            Some(format!(
                "the {LOCK_FILE} of {} was not put back as checked out ({reason}); `git diff -- {LOCK_FILE}` in that worktree shows the file as it stands",
                root.display()
            ))
        }
        (Restored::TheCheckoutIsBack, Some(copy)) => Some(format!(
            "the {LOCK_FILE} of {} was not the checked-out file when the clean ended, so the bytes that stood there are kept at {}, and the file \
             is back as checked out",
            root.display(),
            copy.display()
        )),
        (Restored::TheCheckoutIsBack, None) => None,
    };
}

// The names this module's controls reach through `super::`,
// re-exported so a control sees the whole module.
#[cfg(test)]
pub(super) use crate::package_repair_cargo_lock_names::{
    STAGING_SUFFIX,
    copies_directory,
};
#[cfg(test)]
pub(super) use crate::package_repair_cargo_lock_never_replacing::{
    BETWEEN_THE_READING_AND_THE_RESTORE,
    THE_NEXT_EXCHANGE_FINDS_ANOTHER_FILESYSTEM,
};

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_lock_tests.rs"]
mod tests;
