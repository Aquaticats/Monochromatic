//! What:
//!  The change of a held batch carried onto the branch as it is now,
//!  and the intent that names the commit the branch will take with it.
//! Why:
//!   The branch of a held commit may have moved while the lock stood,
//!  and no client is left to prepare the commit again,
//!  so the change of the batch is carried onto the branch as it is
//!  (`held_commit_replay.rs`),
//!  which is the rule a commit that lands on a branch another session moved follows
//!  (`doc/decision/cli-git-concurrent-commits.md`).
//!  The replay writes the commits and nothing else;
//!  the branch is moved by the caller,
//!  from the value it read,
//!  so two settlements cannot both move it,
//!  and the rewritten intent is on disk before the move.
//!  It sits in a file of its own because the settlement was over the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const replayed = carry(git, held, intent, current); // { intent, commit } | settled
//! ```

use crate::held_commit_record::Held;
use crate::held_commit_replay::{
    Replay,
    onto,
};
use crate::held_commit_settle_outcomes::Settled;
use crate::service_journal::{
    Intent,
    write_intent,
};
use crate::transaction_git::GitContext;

/// What:
///  Carries the batch onto the branch as it is now,
///  and answers the intent that names the commit the branch will take with it.
/// Why:
///   The replay writes the commits and nothing else;
///  the branch is moved by the caller,
///  from the value it read,
///  so two settlements cannot both move it.
///  The rewritten intent is on disk before the move,
///  so a kill between the two leaves an intent that names the commit this release puts on the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const replayed = await carryOnto(held, current); if (replayed.kind !== 'replayed') return replayed; await writeIntent({ ...intent, new: last, commits })
/// ```
pub(crate) fn carry(
    git: &GitContext,
    held: &Held,
    intent: &Intent,
    current: &str,
) -> Result<(Intent, String), Settled> {
    if crate::service_protocol::is_zero_object(intent.old.as_str()) {
        return Err(Settled::Cannot {
            commit: held.new.clone(),
            because: String::from(
                "the branch was created while the lock stood, and the commit was made for a branch that did not exist",
            ),
        });
    }
    return match onto(git, held, current, held.common_dir.as_path()) {
        Replay::Replayed {
            commits,
        } => {
            let Some(last) = commits.last().cloned() else {
                return Err(Settled::Cannot {
                    commit: held.new.clone(),
                    because: String::from("the batch names no commit"),
                });
            };
            let mut written: Intent = intent.clone();
            written.new = last.clone();
            written.commits = commits;
            match write_intent(held.journal.as_path(), &written) {
                Ok(()) => Ok((written, last)),
                // The commits exist and no reference holds them: the side reference still names the commit the batch made,
                //  so the line that says what is kept names that one and says what became of the others.
                Err(error) => Err(Settled::Cannot {
                    commit: held.new.clone(),
                    because: format!(
                        "the record of the batch cannot be written ({error}), so the commit this command made from it onto the branch as it moved is kept by no reference and the next command makes it again"
                    ),
                }),
            }
        }
        Replay::Conflict {
            paths,
        } => Err(Settled::Conflicts {
            commit: held.new.clone(),
            paths,
        }),
        Replay::Cannot {
            because,
        } => Err(Settled::Cannot {
            commit: held.new.clone(),
            because,
        }),
    };
}
