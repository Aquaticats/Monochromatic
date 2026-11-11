//! What:
//!  The branch's remote-tracking reference, brought up to date with what a successful push delivered,
//!  when Git could not write it.
//! Why:
//!  Git delivers a push and then updates the remote-tracking reference of the pushed branch.
//! When another Git holds that reference's lock,
//! it delivers the commits,
//! prints `cannot lock ref '...'`,
//! and still exits `0`,
//! which leaves the reference naming an older commit
//!  (`auto_push.rs`,
//! `REFERENCE_LOCK_REFUSAL`).
//! A repeat push cannot repair it:
//! there is nothing left to deliver,
//! so Git changes nothing.
//!  The reference then names an older commit until something else moves it,
//! a later push that delivers, or a fetch,
//! and every answer
//!  this repository gives from that reference is wrong meanwhile:
//! a held push record is not settled by a commit the remote already has
//!  (`held_push_commit_on_the_remote.rs`),
//! so every command says a published commit is held,
//! and a post-commit verdict the remote already holds keeps holding
//!  the branch back (`push_held_back_by_a_blocked_commit.rs`).
//!  The wrapper knows what its push delivered,
//! so it writes the reference itself,
//! with Git's own reflog words for such an update
//!  ("update by push"),
//! and with the retries the push itself gets when the lock still stands.
//! Nothing a person asked for waits on this:
//! it runs after the push answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repaired = bringUpToDate(git, 'refs/remotes/origin/main', delivered); // true when the reference names the commit
//! ```

use crate::auto_push::{PUSH_LOCK_PAUSE, PUSH_LOCK_RETRIES};
use crate::auto_push_reconcile_basics_and_limits::is_ancestor;
use crate::auto_push_single_flight::{PushGit, run_query};
use crate::diagnostic_log::debug;
use std::ffi::OsString;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// The reflog message Git writes for an update a push made.
pub const UPDATE_BY_PUSH: &str = "update by push";

/// What:
///  Whether `upstream` names a commit that holds `delivered` among its ancestors,
///  or nothing when Git could not say.
/// Why:
///   A reference that already holds the delivered commit is what the push would have written,
/// so it is left alone;
/// a question Git cannot answer is not a reason to leave a stale reference stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alreadyThere = isAncestor(git, delivered, upstream) === true;
/// ```
fn already_there(git: &PushGit, upstream: &str, delivered: &str) -> bool {
    return matches!(is_ancestor(git, delivered, upstream), Ok(Some(true)));
}

/// What:
///  One `git update-ref` of the tracking reference,
///  which is the write Git refused.
/// Why:
///   The reference is written by Git itself,
/// with Git's own message,
/// so the reflog reads like every other push's row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wrote = run(git, ['update-ref', '-m', 'update by push', upstream, delivered]).code === 0;
/// ```
fn write_the_reference(git: &PushGit, upstream: &str, delivered: &str) -> bool {
    let arguments: Vec<OsString> = ["update-ref", "-m", UPDATE_BY_PUSH, upstream, delivered].iter().map(OsString::from).collect();
    return matches!(run_query(git, arguments.as_slice()), Ok(output) if output.code == Some(0));
}

/// What:
///  Makes `upstream` name `delivered` when it does not already,
///  and answers whether the reference now holds the commit.
/// Why:
///   The reference a push updates is what every later question about the branch's remote is answered from,
/// so it must name what the
///  remote holds as soon as the push says it delivered,
/// and not wait for a later push or a fetch.
/// A write can meet the same held lock the push met,
///  so it is tried as often and as patiently as the push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (alreadyThere(...)) return true; for (let try = 0; try <= RETRIES; try += 1) { if (write(...)) return true; sleep(PAUSE * try) } return false;
/// ```
pub fn bring_up_to_date(git: &PushGit, upstream: &str, delivered: &str) -> bool {
    if upstream.is_empty() || delivered.is_empty() {
        return false;
    }
    if already_there(git, upstream, delivered) {
        return true;
    }
    for attempt in 0..=PUSH_LOCK_RETRIES {
        if write_the_reference(git, upstream, delivered) {
            return true;
        }
        // Nothing is waited for on the first failure only when the retries are used up: a lock another Git holds ends.
        if attempt < PUSH_LOCK_RETRIES {
            std::thread::sleep(PUSH_LOCK_PAUSE * (attempt + 1));
        }
    }
    debug(TAG, format!("the tracking reference {upstream} still does not name {delivered}: the push delivered it, and a later push or fetch writes the reference").as_str());
    return false;
}

/// Controls for the repair stay out of the release executable.
#[cfg(test)]
#[path = "push_tracking_ref_repair_tests.rs"]
mod tests;
