//! What:
//!  Where a plain `git push` of one named branch would send it,
//! decided from the configuration and the local refs,
//!  or why Git would send it nowhere.
//! Why:
//!  The service pushes the branch a record names,
//! which may not be the branch checked out,
//! so it must name both sides of
//!  the push itself;
//! naming them from a partial reading of Git's rules pushed to places a plain `git push` refuses
//!  or never goes (a `matching` push recreating a branch the remote deleted,
//! one of several upstream branches chosen at
//!  random,
//! a destination that another configured refspec had already taken).
//!  Everything here follows `setup_default_push_refspecs` in Git's `builtin/push.c` and `match_push_refs` in `remote.c`;
//!  a setting Git refuses or a form this module does not interpret becomes a skip with a note that names the setting,
//!  never a push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const named: NamedPush = await namedPush({ git, branchRef }); // push | remote-must-have | skip
//! ```

/// Import the decision `push.default` makes,
/// the skip a mirrored remote and a refused setting answer with,
/// and the destination those settings name.
use super::auto_push_destination_by_push_default::{
    destination_of,
    mirror_skip,
    named_by_mode,
    skip,
};
/// Import the push the configured refspecs name,
/// from either face of a read.
use super::auto_push_destination_by_refspecs::{
    named_by_refspecs,
    named_by_refspecs_awaiting,
};
/// Publish where a plain push of one branch goes,
/// and what the service does for that branch,
/// under the names a caller already reads them by.
pub use super::auto_push_destination_settings::{
    Destination,
    NamedPush,
};
/// Import the reading of the settings,
/// from either face of a read.
use super::auto_push_destination_settings::{
    read_push_settings,
    read_push_settings_awaiting,
};
/// Import the judgement of the configured refspecs,
/// which the controls read through this module,
/// and the owned OS string their environment is written in.
#[cfg(test)]
use crate::auto_push_refspec_judging::{
    Judged,
    judge,
};
/// Import the probe's Git runner and the failure a query answers with.
use crate::auto_push_single_flight::{
    PushGit,
    SingleFlightError,
};
#[cfg(test)]
use std::ffi::OsString;

///  The remote and the branch ref that the plain `git push` of this branch updates,
/// when the settings name one.
/// Why:
///   The reconcile of a rejected push merges the tip of the branch the rejected push targeted,
/// which with a push remote,
///  a triangular setup or `push.default=current` is not the tracked branch.
///   The remote `.`,
/// a `remote.<name>.push` refspec,
/// `matching`,
/// `nothing`,
/// several upstream branches and a target
///  outside `refs/heads` name none,
/// because the rejection then came from a destination this module cannot name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readDestination({ git, branchRef }): Promise<Destination | undefined>;
/// ```
pub fn read_destination(
    git: &PushGit,
    branch_ref: &str,
) -> Result<Option<Destination>, SingleFlightError> {
    let Some(settings) = read_push_settings(git, branch_ref)? else {
        return Ok(None);
    };
    return Ok(destination_of(branch_ref, &settings));
}

///  The same destination,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile asks where the rejected push went as a task,
/// and the synchronous read enters the runtime and is refused from there.
/// The reading is the synchronous face's,
/// so the two cannot decide differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readDestinationOnTheRuntime({ git, branchRef });
/// ```
pub async fn read_destination_awaiting(
    git: &PushGit,
    branch_ref: &str,
) -> Result<Option<Destination>, SingleFlightError> {
    let Some(settings) = read_push_settings_awaiting(git, branch_ref).await? else {
        return Ok(None);
    };
    return Ok(destination_of(branch_ref, &settings));
}

///  What the service does for a named branch whose upstream resolves:
/// the remote and refs it pushes to,
/// the one remote
///  question that decides,
/// or the note that explains why it pushes nothing.
/// Why:
///   The plain `git push` pushes the checked-out branch,
/// so a push of a branch that is not checked out names both sides;
///  the destination follows Git's rules for the branch as if it were checked out,
/// and a setting under which Git
///  refuses or pushes nothing for it is a skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namedPush({ git, branchRef }): Promise<NamedPush>;
/// ```
pub fn named_push(
    git: &PushGit,
    branch_ref: &str,
) -> Result<NamedPush, SingleFlightError> {
    let Some(settings) = read_push_settings(git, branch_ref)? else {
        let branch = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
        return Ok(skip(
            branch,
            "it has no remote to push to (its remote is `.`, a local branch, or none is set). Push it yourself with `git push`.",
        ));
    };
    if settings.mirror {
        return Ok(mirror_skip(&settings));
    }
    if !settings.refspecs.is_empty() {
        return named_by_refspecs(git, branch_ref, &settings);
    }
    return Ok(named_by_mode(branch_ref, &settings));
}

///  The same push,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push names both sides of its push as a task,
/// and the synchronous read enters the runtime and is refused from there.
/// Every reading and every note is the synchronous face's,
/// so the two cannot push to different places.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await namedPushOnTheRuntime({ git, branchRef });
/// ```
pub async fn named_push_awaiting(
    git: &PushGit,
    branch_ref: &str,
) -> Result<NamedPush, SingleFlightError> {
    let Some(settings) = read_push_settings_awaiting(git, branch_ref).await? else {
        let branch = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
        return Ok(skip(
            branch,
            "it has no remote to push to (its remote is `.`, a local branch, or none is set). Push it yourself with `git push`.",
        ));
    };
    if settings.mirror {
        return Ok(mirror_skip(&settings));
    }
    if !settings.refspecs.is_empty() {
        return named_by_refspecs_awaiting(git, branch_ref, &settings).await;
    }
    return Ok(named_by_mode(branch_ref, &settings));
}

/// The destination logic is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_destination_tests.rs"]
mod tests;
