//! What:
//!  How deep one finish may start the next:
//!  the finishes now running in this process,
//!  the budget the outermost one set,
//!  and the todo lines that budget is read from.
//! Why:
//!  A rebase or a sequence of picks ends one conflicting step and runs the rest,
//!  and that step can stop again,
//!  so a finish nests inside a finish.
//!  The steps still to do bound them:
//!  a rebase's remaining todo list plus the finish that is running,
//!  a pick sequence's todo list (one for a single pick),
//!  and one for a merge,
//!  which has no further step.
//!  The two figures belong to the whole process,
//!  so they are atomics rather than a lock,
//!  and a run that used its budget stops instead of nesting for ever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const budget = finishBudget(operation, gitDir);
//! const allowed = nestingAllows(runningFinishes, budget);
//! ```

use crate::merge_second_pass_finish_stops::Operation;
use std::path::Path;
use std::sync::atomic::AtomicUsize;

/// The finishes now running in this process,
/// outermost first.
pub(super) static NESTED_FINISHES: AtomicUsize = AtomicUsize::new(0);

/// How many finishes the outermost one of this process may nest,
/// set when it is planned (`finish_budget`).
pub(super) static FINISH_BUDGET: AtomicUsize = AtomicUsize::new(0);

/// The commands of a todo file:
/// its lines that are neither blank nor comments.
///  The file is read as a plain one and to a bound
///  (`crate::plain_file::SMALL_FILE_LIMIT`,
/// one line per commit,
/// so a list past it is more than 13,000 commits),
/// because a named pipe at a todo file holds the finish for ever otherwise.
fn todo_commands(path: &Path) -> usize {
    let Ok(text) = crate::plain_file::read_plain_text_at(path, crate::plain_file::SMALL_FILE_LIMIT)
    else {
        return 0;
    };
    let mut commands: usize = 0;
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            commands += 1;
        }
    }
    return commands;
}

/// What:
///  How many finishes the outermost one may nest,
/// from the input of the operation.
/// Why:
///   Each finish ends one conflicting step and runs the rest,
/// so the steps still to do bound the finishes:
/// a rebase's remaining
///  todo list plus the finish that is running,
/// a cherry-pick or revert sequence's todo list (one for a single pick),
/// and one for
///  a merge,
/// which has no further step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishBudget(operation: Operation, gitDir: string): number;
/// ```
pub(super) fn finish_budget(
    operation: Operation,
    git_dir: &Path,
) -> usize {
    return match operation {
        Operation::Rebase => {
            todo_commands(git_dir.join("rebase-merge/git-rebase-todo").as_path()) + 1
        }
        Operation::Pick => todo_commands(git_dir.join("sequencer/todo").as_path()).max(1),
        Operation::Merge => 1,
    };
}

/// Whether one more finish may be nested inside `running` others,
/// under a budget of `budget` finishes.
pub(super) fn nesting_allows(
    running: usize,
    budget: usize,
) -> bool {
    return running < budget;
}
