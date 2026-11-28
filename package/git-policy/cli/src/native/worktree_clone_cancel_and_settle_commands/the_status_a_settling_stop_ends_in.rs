//! What:
//!  The exit status a settling command ends in,
//!  for each way it stops and for a run that settled the records.
//! Why:
//!  The command's own type states why it stopped,
//!  and nothing in its folder states what status a caller reads,
//!  because the round that wires it is the round that decides the stream and
//!  the status.
//!  The cancel command's two statuses are the repository's rule for a management
//!  command
//!  (`worktree_clone_removal_cancel::cancel_refusal`):
//!  a caller's mistake exits `2` and a state that prevents the command exits `1`,
//!  so the settling command answers its own mistakes the same way and no person
//!  has to learn two rules for two words of one grammar.
//!  `SPEC.md` states the same rule for the namespace:
//!  "Unknown management flags and extra positionals are usage errors with exit
//!  `2`".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = new SettlingStop('usage'); // exitStatusOf(stop) === 2
//! const other = new SettlingStop('the-clone-still-stands'); // 1
//! ```

use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_removal_cancel::cancel_refusal::{
    CALLERS_MISTAKE,
    THE_OPERATION_FAILED,
};

/// What:
///  The status one stop of the settling command ends in.
/// Why:
///  The words and both values a person types are the caller's,
///  so a mistake among them is the same `2` the cancel command answers,
///  and every state the command refuses is a condition of the machine and
///  records,
///  which is `1`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exitStatusOf(stop); // 2 for a usage or a confirmation mistake, 1 otherwise
/// ```
pub(crate) fn exit_status_of(stop: &SettlingStop) -> i32 {
    return match stop {
        SettlingStop::Usage {
            ..
        }
        | SettlingStop::PlaceholderLeft {
            ..
        }
        | SettlingStop::TheSlotIsNamedTwice {
            ..
        } => CALLERS_MISTAKE,
        _ => THE_OPERATION_FAILED,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_status_a_settling_stop_ends_in_tests.rs"]
mod tests;
