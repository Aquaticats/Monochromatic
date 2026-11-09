//! What:
//!  The words of a stop:
//! what ended,
//! after how long,
//! why this long,
//! what became of the person's work,
//! and what to run.
//! Why:
//!   A commit that stops with a bare exit status leaves the person guessing whether anything was committed;
//! the step runs in a scratch repository,
//! so nothing was,
//! and the message is on the command line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stopWords({ waited, ceiling, signs })
//! ```

/// Time spans,
/// for the two figures the words name.
use std::time::Duration;

/// What:
///  The words of a stop:
/// what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// and what to run.
/// Why:
///   A commit that stops with a bare exit status leaves the person guessing whether anything was committed;
/// the step runs in a scratch repository,
/// so nothing was,
/// and the message is on the command line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopWords({ waited, ceiling, signs, belowUnlisted })
/// ```
pub fn stop_words(waited: Duration, ceiling: Duration, signs: bool, below_unlisted: bool) -> String {
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
    return format!(
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after {} seconds, the most a commit whose \
         message came with the command may take ({} seconds, three times the slowest `git commit` step this repository's commit timing log holds); nothing was \
         committed, and your index and files are as they were.{signing}{unlisted} Run the same `git commit` command again.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}

/// The words and their controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_stop_words_tests.rs"]
mod tests;
