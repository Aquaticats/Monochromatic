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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stopWords({ waited, ceiling, signs, belowUnlisted, belowUnended })
//! ```

/// Time spans,
/// for the two figures the words name.
use std::time::Duration;

/// What:
///  What a stop's words are built from,
///  and what each figure is.
/// Why:
///   One stop has these facts and no others,
///  and every one of them changes what the person is told to do next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScratchStepStop = { waited: number; ceiling: number; signs: boolean; belowUnlisted: boolean; belowUnended: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScratchStepStop {
    /// How long the step had lasted when the ceiling ended it.
    pub waited: Duration,
    /// The ceiling it ran under,
    /// named in the words as the most a commit of this kind may take.
    pub ceiling: Duration,
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
/// and the person is the one who can end it,
/// so those two facts are said plainly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopWords({ waited, ceiling, signs, belowUnlisted, belowUnended })
/// ```
pub fn stop_words(stop: ScratchStepStop) -> String {
    let ScratchStepStop { waited, ceiling, signs, below_unlisted, below_unended } = stop;
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
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after {} seconds, the most a commit whose \
         message came with the command may take ({} seconds, three times the slowest `git commit` step this repository's commit timing log holds); nothing was \
         committed, and your index and files are as they were.{signing}{unlisted}{unended} Run the same `git commit` command again.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}

/// The words and their controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_stop_words_tests.rs"]
mod tests;
