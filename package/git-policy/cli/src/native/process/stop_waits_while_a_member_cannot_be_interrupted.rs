//! What:
//!  When a stop that asked its tree to end may force the end:
//! at the end of its grace when every member can still act on a signal,
//!  and not before the stop's ceiling while a member is in uninterruptible sleep.
//! Why:
//!  A `SIGKILL` at the end of a grace cut Git's cleanup between its two unlinks in the gate of `85e6d2ab99`:
//! the handler removes a lock's
//!  pid file and then the lock,
//! and the lock was left without its pid file,
//! which nothing proves stale (plan,
//!  `doc/planning/git-policy-cli-full-tokio.md`,
//! "The two gate failures of main at `85e6d2ab99`").
//! The owner picked,
//! 2026-11-07,
//!  verbatim:
//! "3x worst case",
//! as the wait before giving up on something that is only slow,
//! so the ceiling is three times the longest
//!  delay measured (`foundation/worst_delay_seen_on_this_host.rs`).
//! Nothing waits for good:
//! past the ceiling the end is forced whatever
//!  the state.
//! Every stop that escalates `SIGTERM` to `SIGKILL` asks this module:
//! `push/bounded_push.rs` `stop_push` (the push,
//! and
//!  `push/bounded_query.rs` through it) and `push/child_watch_clock_marks_and_stopping.rs` `stop_process` (the watched steps).
//!  The look at each member's state is made at the interval the stop's own loop already looks at its tree
//!  (`POLL_PAUSE` of `bounded_push.rs` and `STOP_POLL` of `child_watch_clock_marks_and_stopping.rs`,
//! 10 ms each),
//! so no loop is added.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! while (escalationWaits(elapsedSinceRequest, graceMs, tree) && anyStillRunning(tree)) await sleep(LOOK_MS);
//! forceEnd(tree);
//! ```

use crate::process_tree::{TreeMember, in_uninterruptible_sleep};
use std::time::Duration;

/// The longest a stop waits after its request for a member that cannot be interrupted,
/// before it forces the end:
/// the owner's multiple
///  of the longest delay measured.
pub const STOP_CEILING: Duration = crate::worst_delay_seen_on_this_host::WORST_CASE_WITH_MARGIN;

/// What a control may change on its own thread:
/// which members count as uninterruptible,
/// and the ceiling.
#[cfg(test)]
pub(crate) struct ControlRules {
    /// Whether a member counts as uninterruptible.
    pub(crate) blocked: fn(&TreeMember) -> bool,
    /// The ceiling a stop on this thread uses.
    pub(crate) ceiling: Duration,
}

#[cfg(test)]
thread_local! {
    /// The rules of the control running on this thread;
    /// nothing outside a control.
    static CONTROL_RULES: std::cell::RefCell<Option<ControlRules>> = const { std::cell::RefCell::new(None) };
}

/// Puts `rules` in force on this thread until the returned guard drops.
#[cfg(test)]
pub(crate) fn rules_for_this_thread(rules: ControlRules) -> RulesGuard {
    CONTROL_RULES.with(|slot| { *slot.borrow_mut() = Some(rules); });
    return RulesGuard;
}

/// Ends the control's rules when dropped.
#[cfg(test)]
pub(crate) struct RulesGuard;

#[cfg(test)]
impl Drop for RulesGuard {
    fn drop(&mut self) {
        CONTROL_RULES.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// The ceiling in force:
/// the real one,
/// or a control's.
fn ceiling_in_force() -> Duration {
    #[cfg(test)]
    if let Some(ceiling) = CONTROL_RULES.with(|slot| { return slot.borrow().as_ref().map(|rules| { return rules.ceiling; }); }) {
        return ceiling;
    }
    return STOP_CEILING;
}

/// Whether `member` counts as uninterruptible:
/// the real state,
/// or a control's.
fn member_is_blocked(member: &TreeMember) -> bool {
    #[cfg(test)]
    if let Some(blocked) = CONTROL_RULES.with(|slot| { return slot.borrow().as_ref().map(|rules| { return rules.blocked; }); }) {
        return blocked(member);
    }
    return in_uninterruptible_sleep(member);
}

/// What:
///  Whether a stop that asked `elapsed` ago keeps waiting before it forces the end of `tree`.
/// Why:
///   Inside the grace it always waits,
/// as before.
/// Past the grace it waits only while some member is in uninterruptible sleep and the
///  ceiling has not passed,
/// so a stop with every member able to act on its signal forces the end at the grace as it always did,
/// and
///  one that waits for a member stuck in a call is bounded by the ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// elapsed < grace || (elapsed < ceiling && tree.some(inUninterruptibleSleep))
/// ```
pub fn escalation_waits(elapsed: Duration, grace: Duration, tree: &[TreeMember]) -> bool {
    if elapsed < grace {
        return true;
    }
    return elapsed < ceiling_in_force() && tree.iter().any(member_is_blocked);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "stop_waits_while_a_member_cannot_be_interrupted_tests.rs"]
mod tests;
