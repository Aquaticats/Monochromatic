//! What:
//!  The opening lines one bound slot's removal says,
//!  and the order of saying one before the removal runs.
//! Why:
//!  A whole clone's removal destroys a dataset and every snapshot of it,
//!  which takes as long as the pool takes,
//!  and a person waiting at a terminal cannot tell a removal that runs from
//!  one that hung unless the line that says a removal runs here is out before
//!  the work starts.
//!  A kill in the middle leaves the plan it wrote and,
//!  with this line out,
//!  a terminal that says a removal was running.
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
//! const said = theOpeningLine({ target, resuming, forces, arm: 'live' });
//! const answer = sayThenRunTheRemoval({ said, saidTo: stderr, theRemoval: () => remove() });
//! ```

use crate::worktree_clone_binding_record::THE_REMOVALS_WORDS;
use crate::worktree_clone_removal_proof_and_plan::removal_command_path::TheRemovalAnswer;
use std::io::Write;
use std::path::Path;

/// Which of the two opening lines a removal says.
/// Why:
///  A live removal says that it runs,
///  and a reading that answers for the pool instead of removing anything says
///  that it is refused and what the lines after it hold,
///  so the arm decides which sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheOpeningLinesArm = 'live' | 'refused';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TheOpeningLinesArm {
    /// The removal this machine's own device runs.
    Live,
    /// The reading a control answers for the pool,
    /// which removes nothing.
    Refused,
}

/// What:
///  What one removal's opening line is made of:
///  the slot the command named,
///  the sentence about a plan a run resumes,
///  the force count,
///  and which of the two lines is said.
/// Why:
///  The slot's state and the plan are what the line has to name to be worth
///  reading,
///  and the force count is what tells a person which spellings this removal
///  answers,
///  so they travel together and no caller can print a line about another slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRemovalsOpeningLine = { target: string; plan?: string; forces: number; arm: 'live' | 'refused' };
/// ```
pub(crate) struct TheRemovalsOpeningLine<'a> {
    /// The slot's path,
    /// as the command named it.
    pub target: &'a Path,
    /// The destruction plan that stands for this clone,
    /// and nothing when no plan stands.
    pub plan: Option<&'a Path>,
    /// How many forces the command carried.
    pub forces: usize,
    /// Which of the two lines is said.
    pub arm: TheOpeningLinesArm,
}

/// What:
///  The line a removal says before it runs,
///  for the arm the caller names.
/// Why:
///  The two sentences differ in what they promise a person:
///  one says the removal runs here and that Git does not,
///  and the other says the pool is the caller's own answer,
///  so nothing was removed,
///  and that the lines after it say what a removal would prove and lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = theOpeningLine({ target, resuming, forces, arm });
/// ```
pub(crate) fn the_opening_line(line: &TheRemovalsOpeningLine<'_>) -> String {
    // A plan that stands is the record of what an earlier run destroyed, so the line says that this run resumes it
    //  rather than writing a second plan beside it.
    let resuming: String = match line.plan {
        Some(plan) => format!(
            " A removal's destruction plan at {} stands, so this run resumes it: it reads the plan, proves every name the plan records again, sends no request for a name the plan records as destroyed, carries the loss forward and ends the plan with the removal's own records.",
            plan.display()
        ),
        None => String::new(),
    };
    return match line.arm {
        TheOpeningLinesArm::Live => format!(
            "the removal of `{}` runs here: a whole managed clone is bound to this slot, and {}.{} Git does not run. This removal ends no holder and forces nothing beyond what the command's own force count passes: `{}` force(s) were counted in the words, where two pass every refusal the removal has.\n",
            line.target.display(),
            THE_REMOVALS_WORDS,
            resuming,
            line.forces
        ),
        TheOpeningLinesArm::Refused => format!(
            "the removal of `{}` is refused: a whole managed clone is bound to this slot, and {}.{} The pool here is the caller's own answer and not this machine's device, which the whole removal needs, so nothing was removed and Git did not run; the lines after this one say what a removal would prove and lose.\n",
            line.target.display(),
            THE_REMOVALS_WORDS,
            resuming
        ),
    };
}

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
pub(crate) struct TheRemovalSaidBeforeItRuns<'a> {
    /// The line or lines the person reads:
    /// what runs,
    /// and the slot's own report.
    pub said: &'a str,
    /// Where the line goes,
    /// which is this process's standard error in the release.
    pub said_to: &'a mut dyn Write,
    /// The removal the line announces.
    pub the_removal: &'a dyn Fn() -> TheRemovalAnswer,
}

/// What:
///  Say the line,
///  then run the removal,
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
pub(crate) fn say_then_run_the_removal(said: TheRemovalSaidBeforeItRuns<'_>) -> TheRemovalAnswer {
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
