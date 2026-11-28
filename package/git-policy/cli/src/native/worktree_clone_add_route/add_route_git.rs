//! What:
//!  The Git runner the ZFS clone route's own restore steps start real Git
//!  through,
//! with the variables the command carries and the variables the step
//!  adds.
//! Why:
//!  The restore is the btrfs rung's own function
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore" step 6),
//! and it runs its steps through the ladder's runner so
//!  no policy of the wrapper that guards a person's `read-tree` applies to them.
//!  A step is a real Git child started by path with the wrapper's own hooks-off
//!  prefix,
//! and its status is kept as Git answered it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const runner = { run: (step) => runRealGit(realGit, step.arguments, [...overlay, ...step.environment]) };
//! ```

use crate::forwarding::{
    ChildOutcome,
    run_real_git_naming,
};
use crate::hooks_off::StartOwner;
use crate::invocation::forwarding::ChildStarted;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
    GitStep,
};
use crate::worktree_copy_error::WorktreeCopyError;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  One add's own Git runs:
///  the real Git by path and the variables every child of this command carries.
/// Why:
///  The restore's steps name their own environment on top of the add's overlay,
///  and nothing else about the run changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RouteGit = { realGit: string; overlay: [string, string][] };
/// ```
pub struct RouteGit<'a> {
    /// The real Git this command resolved.
    pub real_git: &'a Path,
    /// The variables every Git child of this command receives.
    pub overlay: &'a [(OsString, OsString)],
}

impl GitRunner for RouteGit<'_> {
    fn run(
        &mut self,
        step: &GitStep<'_>,
        child_started: &mut dyn ChildStarted,
    ) -> Result<GitStatus, WorktreeCopyError> {
        let (arguments, environment): (&[OsString], &[(OsString, OsString)]) = match step {
            GitStep::Other {
                arguments,
                environment,
            } => (arguments, environment),
            _other => {
                return Err(WorktreeCopyError(String::from(
                    "the clone route's restore runs only its own Git commands",
                )));
            }
        };
        let mut variables: Vec<(OsString, OsString)> = self.overlay.to_vec();
        variables.extend(environment.iter().cloned());
        let ran: ChildOutcome = run_real_git_naming(
            self.real_git,
            arguments,
            variables.as_slice(),
            child_started,
            StartOwner::Wrapper,
        )
        .map_err(|error| return WorktreeCopyError(format!("Git could not be started ({error})")))?;
        return Ok(match ran {
            ChildOutcome::Exited(code) => GitStatus {
                code: Some(code),
            },
            ChildOutcome::Signaled(_signal) => GitStatus {
                code: None,
            },
        });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_route_git_tests.rs"]
mod tests;
