//! What:
//!  The words of a stop:
//! what ended,
//! after how long,
//! why this long,
//! what became of the person's work,
//! what may still run,
//! and what to run.
//! Why:
//!   A commit that stops with a bare exit status leaves the person guessing whether anything was committed;
//! the step runs in a scratch repository,
//! so nothing was,
//! and the message is on the command line.
//!  The bound the words name is the one that really ended the step,
//!  and what the step was waited for is said only when it really was:
//!  a stop that named a bound it did not run to,
//!  or a wait that never happened,
//!  tells the person the wrapper failed them early at a bound they were promised
//!  (finding 2 of the 2026-11-12 review of the stand-aside).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stopWords({ waited, boundThatEndedIt, signs, belowUnlisted, belowUnended })
//! ```

/// Time spans,
/// for the two figures the words name.
use std::time::Duration;
/// Why a step's ceiling stood aside for it,
/// which the words say when it did.
use crate::scratch_step_stall_decided_once::WhyTheStepWasWaitedFor;

/// What:
///  The bound that ended a stopped step,
///  and what the step was waited for before that bound came.
/// Why:
///   One of two bounds ends a step,
///  and each is a different promise to the person:
///  the usual ceiling of a commit whose message came with the command,
///  or the longer bound this release waits a step for while it cannot end it at that ceiling.
///  The reason the longer bound came is part of the bound,
///  so the words never name the longer one and forget to say why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheBoundThatEndedACommit = { theUsualCeiling: number } | { theStallBound: { bound: number; why: WhyTheStepWasWaitedFor } };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheBoundThatEndedACommit {
    /// The usual ceiling for a commit whose message came with the command.
    TheUsualCeiling(Duration),
    /// The longer bound a step is waited for while its ceiling stands aside.
    TheStallBound {
        /// The longest the step was allowed to last.
        bound: Duration,
        /// Why the ceiling stood aside.
        why: WhyTheStepWasWaitedFor,
    },
}

impl TheBoundThatEndedACommit {
    /// What:
    ///  How long the step was allowed to last,
    ///  which the record and the diagnostic line name.
    /// Why:
    ///   A stop answers the bound it ran under and never the one it passed,
    ///  so every reader of a stop asks this one question of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bound.theBound(); // nanos
    /// ```
    pub fn the_bound(&self) -> Duration {
        return match self {
            Self::TheUsualCeiling(ceiling) => *ceiling,
            Self::TheStallBound { bound, why: _why } => *bound,
        };
    }
}

/// What:
///  What a stop's words are built from,
///  and what each figure is.
/// Why:
///   One stop has these facts and no others,
///  and every one of them changes what the person is told to do next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScratchStepStop = { waited: number; boundThatEndedIt: TheBoundThatEndedACommit; signs: boolean; belowUnlisted: boolean; belowUnended: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScratchStepStop {
    /// How long the step had lasted when the bound that ended it came.
    pub waited: Duration,
    /// The bound that really ended it,
    /// and what the step was waited for before that.
    pub bound_that_ended_it: TheBoundThatEndedACommit,
    /// Whether the commit asks for a signature,
    /// which makes the signing program the first thing to check.
    pub signs: bool,
    /// Whether the programs below the step could not be listed when it was ended.
    pub below_unlisted: bool,
    /// How many programs below the step were not ended,
    /// so each of them may still be running.
    pub below_unended: u32,
}

/// What:
///  The words of a stop:
/// what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// what may still run,
/// and what to run.
/// Why:
///   A commit that stops with a bare exit status leaves the person guessing whether anything was committed;
/// the step runs in a scratch repository,
/// so nothing was,
/// and the message is on the command line.
///  A program the ceiling could not list or could not end may still be running,
///  and the person is the one who can end it,
///  so those two facts are said plainly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopWords({ waited, boundThatEndedIt, signs, belowUnlisted, belowUnended })
/// ```
pub fn stop_words(stop: ScratchStepStop) -> String {
    let ScratchStepStop { waited, bound_that_ended_it, signs, below_unlisted, below_unended } = stop;
    // What: The bound that ended the step, named as the promise it was.
    // Why:   A step the kernel held in a disk wait was already given the longer bound before it was ended,
    //        and a person who is told only the usual ceiling reads the stop as one that ended a step working on its own.
    //        A step nothing could be learned about is the same promise with a different reason,
    //        and saying the kernel held it would be a fact no look found.
    let (the_bound, waited_for): (String, &str) = match &bound_that_ended_it {
        TheBoundThatEndedACommit::TheUsualCeiling(ceiling) => (
            format!(
                "the bound this release holds for a commit whose message came with the command ({} seconds: three times the slowest `git commit` step that ended by itself in this \
                 repository's commit timing log, or the one-minute floor, whichever is later)",
                ceiling.as_secs()
            ),
            "",
        ),
        TheBoundThatEndedACommit::TheStallBound { bound, why } => (
            format!("the longer bound this release waits a step for when it may be held in a disk wait ({} seconds)", bound.as_secs()),
            match why {
                WhyTheStepWasWaitedFor::TheKernelHeldIt(_) => " The kernel held it in a disk wait, so it was waited for to that bound before it was stopped.",
                WhyTheStepWasWaitedFor::ItsStateCouldNotBeRead => {
                    " Its state could not be read, so nothing could tell whether the disk was holding it, and it was waited for to that bound before it was stopped."
                }
            },
        ),
    };
    let signing: &str = if signs {
        " The commit signs the commit because `commit.gpgsign` or `-S` asks for it, so the signing program is the first thing to check."
    } else {
        ""
    };
    // A step ended while the programs below it could not be listed: one of them may still run, and the person is told so.
    let unlisted: &str = if below_unlisted {
        " The programs it started could not be listed when it was ended, so one of them may still be running; check with `ps`."
    } else {
        ""
    };
    // A signal the kernel refused leaves the program it named running: the walk listed it, and nothing ended it.
    let unended: String = match below_unended {
        0 => String::new(),
        1 => String::from(" One program it started could not be ended, so it may still be running; check with `ps`."),
        count => format!(" {count} programs it started could not be ended, so they may still be running; check with `ps`."),
    };
    return format!(
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after {} seconds, at {the_bound}; nothing was committed, \
         and your index and files are as they were.{waited_for}{signing}{unlisted}{unended} Run the same `git commit` command again.\n",
        waited.as_secs()
    );
}

/// The words and their controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_stop_words_tests.rs"]
mod tests;
