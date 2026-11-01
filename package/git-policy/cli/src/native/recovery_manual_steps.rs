//! What:
//!  The sentence that tells a person how to finish a retained transaction by hand.
//! Why:
//!  Recovery fails closed on a transaction it cannot complete and keeps its directory;
//!  every guarded command then stops with exit status 2 until that directory is removed.
//!  The steps are the same for every such failure, so every message says them the same way:
//!  check that nothing is running, remove the leftovers, and bring the index to the branch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const steps = finishByHandSteps({ directory, preparing });
//! ```

/// The invocation capture record, which names the real index and the shadow repository.
use super::transaction_journal::PreparingRecord;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The steps that finish a retained transaction by hand.
///  The invocation capture record, when the directory still has a readable one, names the leftovers exactly.
/// Why:
///   `git reset` makes the index match `HEAD` and leaves the worktree files alone, so it is also what a landed
///  commit with an unreconciled index needs; it unstages whatever else was staged, which the sentence says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const steps = finishByHandSteps({ directory, preparing });
/// ```
pub fn finish_by_hand_steps(directory: &Path, preparing: Option<&PreparingRecord>) -> String {
    let leftovers: String = match preparing {
        Some(record) => format!(
            "{}.lock (if it exists), {} (if it exists) and {}",
            record.real_index_path,
            record.shadow_path,
            directory.display()
        ),
        None => format!(
            "{} and any index.lock file in the repository's Git directory (or the linked worktree's Git directory) \
             that no Git command holds",
            directory.display()
        ),
    };
    return format!(
        "To finish by hand, first check that no Git command is running in this repository. \
         Then remove {leftovers}, run `git status`, and if the index does not match HEAD, run `git reset`, \
         which makes the index match HEAD: changes staged for other paths become unstaged, \
         and the worktree files stay as they are. \
         Every guarded command stops until {} is removed.",
        directory.display()
    );
}

/// Step text controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_manual_steps_tests.rs"]
mod tests;
