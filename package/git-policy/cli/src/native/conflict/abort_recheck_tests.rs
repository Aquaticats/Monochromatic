//! What:
//!  The second measurement's own promise:
//!  one invocation measures once,
//!  and a command that is no abort or skip owes no measurement at all.
//! Why:
//!  The plan waits in a process-wide slot between the policy pass and the start of Git,
//!  and a slot that were taken twice would ask the repository twice
//!  and refuse on the second answer a command the first let through.
//!  A form outside this carve-out must not leave a measurement behind for a later start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! planTheSecondLook({ context, command: 'stash', identity, seen: [] });
//! expect(recheckBeforeTheCommand()).toBeUndefined(); // and the slot is empty now
//! ```
use super::{AbortRecheckPlan, plan_the_second_look, recheck_before_the_command};
use crate::rule_linked_worktree::GuardedCommand;
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
use std::path::PathBuf;

/// The context a plan carries: no Git is started for a form outside this carve-out.
fn context() -> GitContext {
    return GitContext {
        real_git: PathBuf::from("/usr/bin/git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// One plan for a form this carve-out does not judge.
fn plan() -> AbortRecheckPlan {
    return AbortRecheckPlan {
        context: context(),
        command: GuardedCommand::Stash,
        identity: WorktreeIdentity::OutsideWorktree,
        seen: Vec::new(),
    };
}

/// A form outside this carve-out owes no measurement,
///  and the plan is taken out of the slot anyway.
#[test]
fn a_form_outside_the_carve_out_owes_no_measurement_and_is_taken_once() {
    plan_the_second_look(plan());
    assert_eq!(recheck_before_the_command(), None, "a stash is not this carve-out's business");
    assert_eq!(recheck_before_the_command(), None, "the plan was taken by the first call");
}

/// A process that never planned a measurement measures nothing.
#[test]
fn nothing_is_measured_when_nothing_was_planned() {
    assert_eq!(recheck_before_the_command(), None);
}
