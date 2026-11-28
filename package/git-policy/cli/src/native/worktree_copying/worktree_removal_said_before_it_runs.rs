//! What:
//!  The line one bound slot's removal says before it runs,
//!  and the removal that runs after it.
//! Why:
//!  A whole clone's removal destroys a dataset and every snapshot of it,
//!  which takes as long as the pool takes,
//!  and a person waiting at a terminal cannot tell a removal that runs from
//!  one that hung unless the line that says a removal runs here is out before
//!  the work starts.
//!  A kill in the middle leaves the plan it wrote and, with this line out, a
//!  terminal that says a removal was running.
//!  The line is said first and the removal runs second,
//!  in this order and in one call,
//!  so no caller can run the one without the other and no caller can put the
//!  line after the work.
//!  A terminal that refuses the line is not a reason to keep the removal from
//!  running:
//!  the line is the wrapper's own notice and the removal is what the person
//!  asked for,
//!  so a refused write is dropped and the removal runs all the same
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = sayThenRunTheRemoval({ said, saidTo: stderr, theRemoval: () => remove() });
//! ```

use crate::worktree_clone_removal_proof_and_plan::removal_command_path::TheRemovalAnswer;
use std::io::Write;

/// What:
///  One line a person reads before a removal runs,
///  where it goes,
///  and the removal it announces.
/// Why:
///  The three travel together so the order is this file's rather than each
///  caller's,
///  and a control hands in a stream and a removal of its own to read the order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SaidBeforeItRuns = { said: string; saidTo: Writer; theRemoval: () => Answer };
/// ```
pub struct TheRemovalSaidBeforeItRuns<'a> {
    /// The line or lines the person reads: what runs, and the slot's own report.
    pub said: &'a str,
    /// Where the line goes,
    /// which is this process's standard error in the release.
    pub said_to: &'a mut dyn Write,
    /// The removal the line announces.
    pub the_removal: &'a dyn Fn() -> TheRemovalAnswer,
}

/// What:
///  Say the line, then run the removal,
///  and answer what the removal answered.
/// Why:
///  The removal's own answer is what the command ends with,
///  and the line is said before it starts rather than after it ends,
///  so this is the one place the two are ordered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// saidTo.write(said); saidTo.flush(); return theRemoval();
/// ```
pub fn say_then_run_the_removal(said: TheRemovalSaidBeforeItRuns<'_>) -> TheRemovalAnswer {
    // A closed terminal is not a reason to keep a removal the person asked for from running, so neither the write
    //  nor the flush can fail this call.
    let _written = said.said_to.write_all(said.said.as_bytes());
    let _flushed = said.said_to.flush();
    return (said.the_removal)();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_said_before_it_runs_tests.rs"]
mod tests;
