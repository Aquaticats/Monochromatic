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

/// The files a forwarded second pass merged,
/// staged as resolved and written to the worktree.
pub mod merge_second_pass_worktree_install;

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

/// The intent records a recovery walks,
/// and what a command is told about each.
pub mod merge_isolation_recovery_records;

/// The person's own work a merge carries,
/// across the two moves a recovery can make.
pub mod merge_isolation_recovery_carry;

/// The steps shared by the recovery of an interrupted merge and the settling of a moved branch.
pub mod merge_isolation_recovery_steps;

/// The write of a merge into the index and the worktree,
/// and what it does when the checkout changed under it.
pub mod merge_isolation_recovery_install;

/// Whether the recovery of an interrupted merge may write the index now,
/// and the clearing of a lock a killed Git left.
pub mod merge_isolation_recovery_index_lock;

/// Finishing a merge from its record:
/// the install and the branch move,
/// with the checks it needs.
pub mod merge_isolation_recovery_complete;

/// What the recovery of a merge reads to decide whose checkout this is and whether it began.
pub mod merge_isolation_recovery_reads;

/// The records and side refs a settled merge leaves behind,
/// and the refusal that carries whether it began.
pub mod merge_isolation_recovery_cleanup;

/// The line a recovery writes when it changes the branch or keeps a saved merge.
pub mod merge_isolation_recovery_notice;

/// The index read the way a merge's recovery needs it.
pub mod merge_isolation_index_reading;

/// The take of a worktree file a merge must replace or delete,
/// and the private names it leaves behind.
pub mod merge_isolation_take_asides;

/// The commit message of an isolated merge.
pub mod merge_isolation_message;

/// A pull that merges,
/// read as its fetch and its merge of `FETCH_HEAD`.
pub mod pull_isolation;

/// The refusal findings of an isolated merge.
pub mod merge_isolation_report;

/// A true merge completed in isolation while the index holds staged changes.
pub mod merge_isolation;

/// The person's own work in the paths a merge changes:
/// what the worktree and the index held,
/// as the record
///  and a ref of its own keep it.
pub mod merge_isolation_carry_parts;

/// Reading the person's work and keeping it outside the checkout.
pub mod merge_isolation_carry;

/// Writing it back:
/// the clear before the merge,
/// the carry of each layer after it,
/// and the restore of a merge that did not land.
pub mod merge_isolation_carry_apply;

/// What a carry reads from the repository:
/// the file form and the stored bytes of one object,
/// one path's index entry,
/// and the raw changes between two commits.
pub mod merge_isolation_carry_reads;

/// What a carry writes:
/// the index records of a batch,
/// one file through a private name and a rename,
/// and the bounded wait for `index.lock`.
pub mod merge_isolation_carry_writes;

/// What one captured path's writes are,
/// once the merge's change is known.
pub mod merge_isolation_carry_plans;

/// What one layer of one path ends as once the person's work is merged with the merge.
pub mod merge_isolation_carry_layers;

/// The lines a carried merge prints about the paths it kept,
/// merged or left alone.
pub mod merge_isolation_carry_notice;

/// The writing half of an isolated merge:
/// the commit and the two-way update.
pub mod merge_isolation_apply;

/// The words an isolated merge prints when it could not be finished now.
pub mod merge_isolation_apply_findings;

/// The merge tree of an isolated merge,
/// computed from objects,
/// and the word-level second pass over its conflicts.
pub mod merge_isolation_merge_tree;

/// The questions an isolated merge asks Git and the terminal.
pub mod merge_isolation_git_reads;

/// The values an isolated merge passes between its steps.
pub mod merge_isolation_run_types;
