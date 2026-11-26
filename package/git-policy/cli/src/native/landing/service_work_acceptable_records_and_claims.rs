//! What:
//!  Which records of the work directory are acceptable work,
//!  and what becomes of the claims this instance holds:
//!  the work directory of a repository,
//!  the fields an item must carry before anything runs it,
//!  settling every claim to a state,
//!  and keeping only the claims a renewal still holds.
//! Why:
//!  Anyone who can write the work directory can write an item,
//!  so the fields that become a directory,
//!  a reference or a program argument are checked before they are used,
//!  and an item whose claim another instance took over must not run twice;
//!  these rules bound the drain and hold no state of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ok = isAcceptable(item, gitDir);
//! settleAll(workDir, held, claimed, 'failed');
//! const mine = stillHeld(held, workDir, claimed);
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::service_claim_renewal::HeldClaims;
use crate::work_item::{
    Claimed,
    WORK_DIRECTORY,
    WorkItem,
};
use std::path::{
    Path,
    PathBuf,
};

/// The work directory of a repository.
pub fn work_directory(git_dir: &Path) -> PathBuf {
    return path_for_reading(git_dir, WORK_DIRECTORY);
}

/// What:
///  Whether an item is one this service should act on for this repository.
/// Why:
///   Anyone who can write the work directory can write an item,
/// so the fields that become a directory,
/// a reference or a
///  program argument are checked before they are used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAcceptable(item, gitDir)
/// ```
pub fn is_acceptable(
    item: &WorkItem,
    git_dir: &Path,
) -> bool {
    return item.repository.is_absolute()
        && item.repository.is_dir()
        && item.git_dir.as_path() == git_dir
        && item.branch_ref.starts_with("refs/")
        && !item.branch_ref.contains(['\0', '\n', ' '])
        && item.oid.len() >= 40
        && item.oid.as_bytes().iter().all(u8::is_ascii_hexdigit);
}

/// Moves the claims to `state` and stops renewing them;
/// a claim that another instance took over is said,
/// and left to it.
pub(crate) fn settle_all(
    work_dir: &Path,
    held: &HeldClaims,
    claimed: &[Claimed],
    state: &str,
) {
    for item in claimed {
        if let Err(reason) = held.settle(work_dir, item.id.as_str(), state) {
            crate::service_warn!("{reason}");
        }
    }
}

/// What:
///  The claims of `claimed` that this instance still holds,
/// before their work starts.
/// Why:
///   A claim whose renewal found it gone was taken over:
/// another instance runs that item now,
/// so running it here as well would
///  do it twice.
/// The claim is forgotten and the loss said,
/// and nothing else is done for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimed.filter((c) => held.isHeld(c.id))
/// ```
pub(crate) fn still_held(
    held: &HeldClaims,
    work_dir: &Path,
    claimed: Vec<Claimed>,
) -> Vec<Claimed> {
    let mut mine: Vec<Claimed> = Vec::new();
    let mut lost: Vec<Claimed> = Vec::new();
    for item in claimed {
        if held.is_held_on_disk(work_dir, item.id.as_str()) {
            mine.push(item);
        } else {
            lost.push(item);
        }
    }
    for item in lost {
        crate::service_warn!(
            "the claim on {} ran out before this instance could run it, so another instance took the work over and it is not run here",
            item.id
        );
        held.release(item.id.as_str());
    }
    return mine;
}
