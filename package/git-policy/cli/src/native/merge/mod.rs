//! What:
//!  Merges:
//!  the word-level second pass,
//!  isolated merges and pull handling.
//! Why:
//!  Merge code shares fixtures and a journal,
//!  so it is kept together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // merge/index.ts: one `export * from './file'` per module below.
//! ```

/// The three-way merge by line and by word that resolves a conflict Git's line merge reports.
pub mod word_merge;

/// The conflicted files a merge left,
/// which of them the word pass may re-merge,
/// and the lines that list each merged part.
pub mod merge_second_pass;

/// The words that list a merged part and the journal entry for a merged file.
pub mod merge_second_pass_report;

/// The conflicted files of a merge tree and the reading of `git merge-tree` output into conflicts.
pub mod merge_second_pass_conflicts;

/// How the word-level second pass starts Git.
pub mod merge_second_pass_git_runner;

/// The word-level second pass for merges computed from objects,
/// and the attribute tree those merges read.
pub mod merge_second_pass_tree;

/// The word-level second pass after a forwarded merge,
/// rebase,
/// cherry-pick or revert that Git stopped with a conflict.
pub mod merge_second_pass_worktree;

/// What the word-level second pass will not replace:
/// a path another command moved while it computed its merge.
pub mod merge_second_pass_worktree_unchanged;

/// The sentences the word-level second pass ends a stopped operation with.
pub mod merge_second_pass_worktree_words;

/// The finish of a stopped operation,
/// like a clean run,
/// after the second pass resolved every conflicted file.
pub mod merge_second_pass_finish;

/// Which options make the finish of a stopped operation stop where Git would stop.
pub mod merge_second_pass_finish_stops;

/// The `git merge` command lines the wrapper completes in isolation,
/// and those it leaves to native Git.
pub mod merge_options;

/// The settling of an isolated merge whose branch moved while it was prepared or applied.
pub mod merge_isolation_moved;

/// The durable intent record of an isolated merge.
pub mod merge_isolation_journal;

/// The raw differences of an isolated merge and the worktree state of each changed path.
pub mod merge_isolation_changes;

/// Recovery of an isolated merge whose process died.
pub mod merge_isolation_recovery;

/// The steps shared by the recovery of an interrupted merge and the settling of a moved branch.
pub mod merge_isolation_recovery_steps;

/// The commit message of an isolated merge.
pub mod merge_isolation_message;

/// A pull that merges,
/// read as its fetch and its merge of `FETCH_HEAD`.
pub mod pull_isolation;

/// The refusal findings of an isolated merge.
pub mod merge_isolation_report;

/// A true merge completed in isolation while the index holds staged changes.
pub mod merge_isolation;

/// The writing half of an isolated merge:
/// the merge tree,
/// the commit and the two-way update.
pub mod merge_isolation_apply;

/// The questions an isolated merge asks Git and the terminal.
pub mod merge_isolation_git_reads;

/// The values an isolated merge passes between its steps.
pub mod merge_isolation_run_types;
