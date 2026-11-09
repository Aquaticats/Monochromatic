//! What:
//!  Which of the ladder's own Git steps has a ceiling,
//! and which number that ceiling is.
//! Why:
//!  The ladder starts Git itself for `git read-tree`
//! (the tree difference of a clean snapshot and the reset of a
//!  dirty one)
//! and for the fallback `git reset --hard` of a refused rung.
//! Those steps run the checkout's conversion
//!  filters and write into the wrapper's own stage or into the new worktree,
//! and the person's command is not
//!  waiting on a terminal for them,
//! so the owner's "nothing should ever wait forever or indefinitely" applies to them
//! (`service/wait_ceilings.rs` holds each number with its measurement).
//! The person's own command is never one of them:
//! native Git has no ceiling and the wrapper adds none to what
//!  the person typed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ceiling = ceilingOf(arguments); // undefined for the person's command
//! ```
//!
//! The ceilings themselves:
//!  `wait_ceilings.rs`.

use crate::checkout_workers::is_fallback_checkout;
use crate::transaction_git::subcommand_of;
use crate::wait_ceilings::{LADDER_CHECKOUT_CEILING, LADDER_READ_TREE_CEILING};
use std::ffi::OsString;
use std::time::Duration;

/// What:
///  The longest the step with these arguments may run,
/// or nothing when the step has no ceiling.
/// Why:
///   The class decides the number:
/// a `read-tree` writes only the wrapper's own stage and can be run again,
/// while the
///  fallback checkout writes a whole worktree and is given the ceiling of a whole checkout.
/// Every other word
///  list is one this module knows nothing about,
/// and a step nobody classified gets no ceiling from here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// subcommandOf(arguments) === 'read-tree' ? READ_TREE_CEILING : isFallbackCheckout(arguments) ? CHECKOUT_CEILING : undefined
/// ```
pub fn ceiling_of(arguments: &[OsString]) -> Option<Duration> {
    let (_index, name) = subcommand_of(arguments)?;
    let ceiling: Duration = if name == "read-tree" {
        LADDER_READ_TREE_CEILING
    } else if is_fallback_checkout(arguments) {
        LADDER_CHECKOUT_CEILING
    } else {
        return None;
    };
    #[cfg(test)]
    if let Some(shortened) = CEILING_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return Some(shortened);
    }
    return Some(ceiling);
}

#[cfg(test)]
thread_local! {
    /// A shortened ceiling for the steps the current thread starts,
    /// so a control ends a step in a moment instead of at ten or twenty minutes.
    pub(crate) static CEILING_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// Step ceilings stay out of the release executable.
#[cfg(test)]
#[path = "ladder_step_ceilings_tests.rs"]
mod tests;
