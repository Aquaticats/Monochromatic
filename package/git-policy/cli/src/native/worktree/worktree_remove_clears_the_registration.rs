//! What:
//!  What a removal does after the subvolume is gone:
//!  clear the snapshots ahead of an add out of the registration,
//!  empty the registration through the descriptors the removal holds,
//!  remove it from the `worktrees` directory it stands in,
//!  and say how that ended.
//! Why:
//!   The destroy is one kernel call and the registration is not part of it,
//!  so a command killed between them leaves files gone and a registration standing;
//!  only the person can finish that,
//!  and every later command shows a notice about it.
//!  Emptying through the held descriptor and removing by name succeeds only for an empty directory,
//!  so a registration another process replaced is never touched.
//!  It is its own file because `worktree_remove.rs` is at the crate's length limit,
//!  and because the around pass and the removal both finish a removal the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function finish(held: Held, registration: Registration, request: RemoveRequest): Promise<RemoveOutcome>;
//! ```

// What:     `use crate::diagnostic_log::{debug, warn};` imports the two log sinks
//           this step writes to.
// Why:      A registration that could not be cleared is reported to the person as a
//           warning, and a destroy that finished is a debug line.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — the package's own logger, called by tag.
// ```
use crate::diagnostic_log::{
    debug,
    warn,
};
// What:     `use crate::fs_beneath::{handle_path, identity_at, remove_at};` imports
//           three calls on a directory this removal holds open.
// Why:      The registration is compared, emptied and removed through the
//           descriptor the removal already holds, never by a path that could name
//           another directory by then.
//
// In TS you'd write (pseudocode):
// ```ts
// import { handlePath, identityAt, removeAt } from '../fs_beneath';
// ```
use crate::fs_beneath::{
    handle_path,
    identity_at,
    remove_at,
};
// What:     `use crate::subvolume_remove_walk::{Boundary, empty_directory, verify_below};`
//           imports the proof that a directory holds no nested subvolume and the
//           emptying that follows it. `Boundary` is the record of one such volume,
//           which this step only names in the proof's answer.
// Why:      The kernel refuses to empty a directory that holds a subvolume, so the
//           emptying runs only after the proof.
//
// In TS you'd write (pseudocode):
// ```ts
// import { emptyDirectory, verifyBelow } from '../subvolume_remove_walk';
// ```
use crate::subvolume_remove_walk::{
    empty_directory,
    verify_below,
};
// What:     `use crate::worktree_removal_clears_the_ahead_pool::clear_the_pools_ahead_of_a_worktree;`
//           imports the clear of the snapshots one add made ahead of another.
// Why:      Those pools stand in the registration this step empties, so they go first,
//           and a pool that cannot go is the reason the registration stays.
//
// In TS you'd write (pseudocode):
// ```ts
// import { clearThePoolsAheadOfAWorktree } from './worktree_removal_clears_the_ahead_pool';
// ```
use crate::worktree_removal_clears_the_ahead_pool::clear_the_pools_ahead_of_a_worktree;
// What:     `use crate::worktree_removal_holds_the_worktree::Held;` imports the
//           directories and identities one removal holds open.
// Why:      The registration is emptied through the descriptor that record holds.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Held } from './worktree_removal_holds_the_worktree';
// ```
use crate::worktree_removal_holds_the_worktree::Held;
// What:     `use crate::worktree_remove_outcome::RemoveOutcome;` imports the endings
//           a removal answers with.
// Why:      This step answers `Removed`, or one of the two endings that carry the
//           words for what still stands in the way.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemoveOutcome } from './worktree_remove_outcome';
// ```
use crate::worktree_remove_outcome::RemoveOutcome;
// What:     `use crate::worktree_remove_registry::Registration;` imports the record of
//           which worktree registration this one is.
// Why:      The step answers for that registration and for no other.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Registration } from './worktree_remove_registry';
// ```
use crate::worktree_remove_registry::Registration;
// What:     `use crate::worktree_remove_reports::incomplete_report;` imports the words
//           for a removal whose subvolume is gone and whose registration is not.
// Why:      That ending is the only one this step words itself, and the words are one
//           subject of their own.
//
// In TS you'd write (pseudocode):
// ```ts
// import { incompleteReport } from './worktree_remove_reports';
// ```
use crate::worktree_remove_reports::incomplete_report;
// What:     `use crate::worktree_remove_request_and_last_look::{RemoveRequest, TAG};`
//           imports what the removal was asked and the tag its log lines carry.
// Why:      The registration is reported against the plan the person named, and every
//           line of this subject writes the same tag.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TAG, type RemoveRequest } from './worktree_remove_request_and_last_look';
// ```
use crate::worktree_remove_request_and_last_look::{
    RemoveRequest,
    TAG,
};
// What:     `use std::fs::File;` imports the owned descriptor type, and
//           `use std::path::Path;` the borrowed path type.
// Why:      The registration is made one descriptor of its own, and the plan's
//           target is only read.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — a file handle is owned, and a path is a plain string.
// ```
use std::fs::File;
use std::path::Path;

/// What:
///  After the destroy,
/// remove the registration through the descriptors that were held,
/// and report how that ended.
/// Why:
///   The registration is emptied through its held descriptor and removed from the held `worktrees` directory by name,
/// which
///  succeeds only for an empty directory;
/// a registration that another process replaced is therefore never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finish(held: Held, registration: Registration, request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
/// The outcome of a removal whose subvolume is gone but whose registration could not be cleared:
/// the report is also logged.
fn incomplete(
    registration: &Registration,
    request: &RemoveRequest<'_>,
    reason: String,
) -> RemoveOutcome {
    let report: String = incomplete_report(
        request.plan.target.as_path(),
        registration,
        request.common_dir,
        reason.as_str(),
    );
    warn(TAG, report.as_str());
    return RemoveOutcome::Incomplete(report);
}

pub(crate) fn finish(
    held: &Held,
    registration: &Registration,
    request: &RemoveRequest<'_>,
) -> RemoveOutcome {
    let target: &Path = request.plan.target.as_path();
    match identity_at(&held.worktrees, registration.id.as_os_str()) {
        Ok(found) if found == held.admin_identity => {}
        Ok(_other) => {
            return incomplete(
                registration,
                request,
                String::from(
                    "another registration took its place while the worktree was being removed, and it was left untouched",
                ),
            );
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            debug(
                TAG,
                format!(
                    "the registration {} is already gone",
                    registration.admin.display()
                )
                .as_str(),
            );
            return RemoveOutcome::Removed;
        }
        Err(error) => {
            return incomplete(
                registration,
                request,
                format!("it cannot be looked up ({error})"),
            );
        }
    }
    let admin: File = match held.admin.try_clone() {
        Ok(admin) => admin,
        Err(error) => {
            return incomplete(
                registration,
                request,
                format!("its directory cannot be used ({error})"),
            );
        }
    };
    // The snapshots ahead of an add are this program's own and sit in the registration, so they go before the walk that
    // refuses every nested subvolume. A pool a fill made in the moments since the removal cleared it is the one case this
    // second clear answers, and a member that cannot go is the reason the registration stays.
    if let Err(reason) =
        clear_the_pools_ahead_of_a_worktree(request.common_dir, handle_path(&held.admin).as_path())
    {
        return incomplete(
            registration,
            request,
            format!("its content cannot be removed ({reason})"),
        );
    }
    let emptied = match verify_below(&admin, registration.admin.as_path()) {
        Ok(()) => empty_directory(&admin, registration.admin.as_path()),
        Err(error) => Err(error),
    };
    if let Err(error) = emptied {
        return incomplete(
            registration,
            request,
            format!("its content cannot be removed ({error})"),
        );
    }
    if let Err(error) = remove_at(&held.worktrees, registration.id.as_os_str(), true) {
        return incomplete(
            registration,
            request,
            format!(
                "the directory cannot be removed from {} ({error})",
                registration.worktrees.display()
            ),
        );
    }
    // Git removes the `worktrees` directory when the last registration goes; a directory that still holds others stays.
    let _kept = std::fs::remove_dir(registration.worktrees.as_path());
    debug(
        TAG,
        format!(
            "destroyed the subvolume {} and its administrative directory",
            target.display()
        )
        .as_str(),
    );
    return RemoveOutcome::Removed;
}
