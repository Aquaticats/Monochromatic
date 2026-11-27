//! What:
//!  A true `git merge <branch>` completed in isolation when the index holds staged changes that make native Git
//!  refuse it:
//! the merge is computed from objects,
//! committed,
//! and applied with Git's own two-way tree update,
//! which
//!  keeps every entry and every file the merge does not change.
//!  This file holds the entry every merge comes through and the loop that makes a round again when the branch
//!  moved under it;
//! one round is `merge_isolation_round.rs`,
//! what a round writes of its merge is `merge_isolation_round_writes.rs`,
//! and the merges a dead process left are recovered by `merge_isolation_recovery_before_the_merge.rs`.
//! Why:
//!  Several sessions share one checkout.
//!  Native Git refuses every merge that creates a commit while the index differs from `HEAD` at all (measured in
//!  `doc/planning/cli-git-merge-with-staged-index.md`),
//! although the merge may touch none of the staged paths.
//!  A fast-forward is not refused for unrelated paths,
//! because it is a two-way update that keeps entries it does not
//!  write;
//! a merge commit whose first parent is the current commit is exactly such an update.
//!  So the merge tree comes from `git merge-tree --write-tree`,
//! the commit from `git commit-tree`,
//! and the checkout
//!  moves with `git merge --ff-only <commit>`,
//! which refuses when the branch is no longer where the merge was computed,
//! after a check that no changed path is staged
//!  or modified by someone else.
//!  An intent record,
//! written before Git starts,
//! lets recovery finish or discard a merge whose process died.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runIsolatedMerge({ command, identity, environment }); // 'forward' | { code, stdout, stderr }
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// One round of an isolated merge,
/// which this loop makes again when the branch moved under it.
use crate::merge_isolation_round::attempt_round;
/// The ending every early stop of a merge takes,
/// how many rounds one merge may take,
/// and the tag this file's log lines carry.
use crate::merge_isolation_run_types::{
    Bail,
    MAX_ROUNDS,
    TAG,
    refused,
};
/// Owned operating-system text.
pub use crate::merge_isolation_run_types::{
    IsolatedMerge,
    IsolatedMergeInput,
};
/// The notices of a round that is made again.
use std::io::Write;

/// What:
///  Complete or refuse one merge,
/// or hand the command back to native Git.
/// Why:
///   The wrapper acts only on the exact failing case:
/// a supported merge command,
/// a staged index,
/// a true merge.
///  Everything else is native Git's,
/// including its own errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runIsolatedMerge(input: IsolatedMergeInput): Promise<IsolatedMerge>;
/// ```
pub fn run_isolated_merge(input: &IsolatedMergeInput<'_>) -> IsolatedMerge {
    // What the person wrote in the editor, kept across the rounds a moved branch makes again, so their message is
    //  never asked for twice.
    let mut edited: Option<Vec<u8>> = None;
    match attempt(input, &mut edited) {
        Ok(done) | Err(Bail::Done(done)) => return done,
        Err(Bail::Forward(notice)) => {
            return IsolatedMerge::Forward {
                notice,
            };
        }
        // `attempt` makes the rounds itself and never lets one escape; native Git is the safe answer to a bug here.
        Err(Bail::Again(reason)) => {
            debug(
                TAG,
                format!("a round escaped the attempt ({reason}); native Git runs the merge")
                    .as_str(),
            );
            return IsolatedMerge::Forward {
                notice: None,
            };
        }
    }
}

/// What:
///  The whole attempt,
/// with `Bail` standing for every early ending.
/// Why:
///   Early endings are many and mostly mean "forward";
/// the question mark keeps the sequence readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attempt(input): Promise<IsolatedMerge>; // throws Bail
/// ```
fn attempt(
    input: &IsolatedMergeInput<'_>,
    edited: &mut Option<Vec<u8>>,
) -> Result<IsolatedMerge, Bail> {
    for round in 1..=MAX_ROUNDS {
        match attempt_round(input, edited) {
            Err(Bail::Again(reason)) => {
                debug(
                    TAG,
                    format!("round {round} of {MAX_ROUNDS}: {reason}").as_str(),
                );
                let _noted = std::io::stderr().write_all(format!("monochromatic-git-policy-cli: git merge: {reason}; merging again on the current branch.\n").as_bytes());
            }
            finished => return finished,
        }
    }
    return Err(refused(
        2,
        format!(
            "monochromatic-git-policy-cli: git merge: the branch moved, a file the merge carries was saved again, or the index lock was \
             taken in {MAX_ROUNDS} merges in a row, so the wrapper stopped; nothing of yours was changed. Run the merge again.\n"
        ),
    ));
}
