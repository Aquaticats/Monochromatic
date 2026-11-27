//! What:
//!  Merges:
//!  the word-level second pass,
//!  isolated merges and pull handling.
//! Why:
//!  Merge code shares fixtures and a journal,
//!  so it is kept together.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `merge_isolation_carry_parts.rs`:
//!   the person's own work in the paths a merge changes:
//!   what the worktree and the index held,
//!   as the record and a ref of its own keep it.
//! - `merge_isolation_carry_place_write.rs`:
//!   the entry such a write makes:
//!   the mode Git's own checkout gives a file,
//!   a symbolic link,
//!   and the permission bits a person set on their own entry.
//! - `merge_isolation_changes.rs`:
//!   the raw differences of an isolated merge and the worktree state of each changed path.
//! - `merge_isolation_a_special_file_the_merge_changes.rs`:
//!   a named pipe,
//!   a socket or a device standing where the merge writes or deletes a path,
//!   which no layer of the carry can hold and which no command may open,
//!   and the sentence that stops the merge before anything is written.
//! - `merge_isolation_carry_kept_path.rs`:
//!   one path a carry kept whole:
//!   what stands at it of the person's work,
//!   what the merge made of it,
//!   and the line that says both and names the command that reaches the merge's version.
//! - `merge_isolation_carry_notice.rs`:
//!   the lines a carried merge prints about the paths it kept,
//!   merged or left alone.
//! - `merge_isolation_editor_words.rs`:
//!   the words of the editor command,
//!   which no shell makes.
//! - `merge_isolation_index_reading.rs`:
//!   the index read the way a merge's recovery needs it.
//! - `merge_isolation_journal.rs`:
//!   the durable intent record of an isolated merge.
//! - `merge_isolation_carry.rs`:
//!   reading the person's work and keeping it outside the checkout.
//! - `merge_isolation_carry_landed.rs`:
//!   what Git's own update left at each path of a merge,
//!   which tells its own file from a save that accepts it.
//! - `merge_isolation_carry_reads.rs`:
//!   what a carry reads from the repository:
//!   the file form and the stored bytes of one object,
//!   one path's index entry,
//!   and the raw changes between two commits.
//! - `merge_isolation_a_directory_the_merge_replaces.rs`:
//!   a directory standing where the merge puts a file,
//!   which no layer of the carry can hold,
//!   and the sentence that stops the merge before anything is written.
//! - `merge_isolation_ignored_paths.rs`:
//!   the ignored untracked files of the person's that stand at the paths a merge writes,
//!   which `git status` never lists.
//! - `merge_isolation_message.rs`:
//!   the commit message of an isolated merge.
//! - `merge_isolation_moved.rs`:
//!   the settling of an isolated merge whose branch moved while it was prepared or applied.
//! - `merge_isolation_recovery.rs`:
//!   recovery of an isolated merge whose process died.
//! - `merge_isolation_recovery_carry.rs`:
//!   the person's own work a merge carries,
//!   across the two moves a recovery can make.
//! - `merge_isolation_recovery_cleanup.rs`:
//!   the records and side refs a settled merge leaves behind,
//!   and the refusal that carries whether it began.
//! - `merge_isolation_recovery_complete.rs`:
//!   finishing a merge from its record:
//!   the install and the branch move,
//!   with the checks it needs.
//! - `merge_isolation_recovery_index_lock.rs`:
//!   whether the recovery of an interrupted merge may write the index now,
//!   and the clearing of a lock a killed Git left.
//! - `merge_isolation_recovery_install.rs`:
//!   the write of a merge into the index and the worktree,
//!   and what it does when the checkout changed under it.
//! - `merge_isolation_recovery_notice.rs`:
//!   the line a recovery writes when it changes the branch or keeps a saved merge.
//! - `merge_isolation_recovery_reads.rs`:
//!   what the recovery of a merge reads to decide whose checkout this is and whether it began.
//! - `merge_isolation_recovery_steps.rs`:
//!   the steps shared by the recovery of an interrupted merge and the settling of a moved branch.
//! - `merge_isolation_report.rs`:
//!   the refusal findings of an isolated merge.
//! - `merge_isolation_take_aside_names.rs`:
//!   the private name a take gives a file it moves aside,
//!   and the merge record that name is tied to.
//! - `merge_isolation_carry_place.rs`:
//!   putting one planned worktree write in place,
//!   or leaving a path alone because what stands there is not the entry that was judged.
//! - `merge_isolation_kept_copies.rs`:
//!   the copies of the person's own bytes a merge keeps beside a path,
//!   and the record of each.
//! - `merge_isolation_carry_writes.rs`:
//!   what a carry writes:
//!   the index records of a batch,
//!   one file through a private name and a rename,
//!   and the bounded wait for `index.lock`.
//! - `merge_isolation_kept_copies_words.rs`:
//!   the lines a kept copy earns,
//!   and the reason it stays when the file at its path holds its bytes.
//! - `merge_isolation_recovery_records.rs`:
//!   the intent records a recovery walks,
//!   and what a command is told about each.
//! - `merge_isolation_take_asides_proof.rs`:
//!   the proof that no program can write into a file whose last name is about to go.
//! - `merge_isolation_kept_copies_decisions.rs`:
//!   the one decision every command makes about a copy a merge kept.
//! - `merge_isolation_take_asides.rs`:
//!   the take of a worktree file a merge must replace or delete,
//!   and the private names it leaves behind.
//! - `merge_isolation_worktree_entry.rs`:
//!   what stands at one worktree path,
//!   as a merge reads it.
//! - `merge_isolation_marks_the_status_hides.rs`:
//!   the paths a merge changes whose edit Git's status does not show,
//!   because their index entry carries the `assume-unchanged` or `skip-worktree` mark.
//! - `merge_options.rs`:
//!   the `git merge` command lines the wrapper completes in isolation,
//!   and those it leaves to native Git.
//! - `merge_isolation_run_types.rs`:
//!   the values an isolated merge passes between its steps.
//! - `merge_isolation_git_reads.rs`:
//!   the questions an isolated merge asks Git and the terminal.
//! - `merge_isolation_dirty_paths.rs`:
//!   the paths a merge that writes or deletes them would discard.
//! - `merge_isolation_editor.rs`:
//!   the editor a merge opens for its message when the checkout holds the person's own work:
//!   the file it reads,
//!   the program Git resolves,
//!   and the message that comes back.
//! - `merge_second_pass_conflicts.rs`:
//!   the conflicted files of a merge tree and the reading of `git merge-tree` output into conflicts.
//! - `merge_second_pass_finish_stops.rs`:
//!   which options make the finish of a stopped operation stop where Git would stop.
//! - `merge_second_pass_finish.rs`:
//!   the finish of a stopped operation,
//!   like a clean run,
//!   after the second pass resolved every conflicted file.
//! - `merge_second_pass_git_runner.rs`:
//!   how the word-level second pass starts Git.
//! - `merge_second_pass_line_regions.rs`:
//!   the line merge Git makes for a conflicted file,
//!   cut into the text it settled and the regions it left.
//! - `pull_untracked_paths_in_the_way_kept_copies.rs`:
//!   one such file or link copied beside the Git directory,
//!   and the numbered folder a pull's copies go into.
//! - `pull_untracked_paths_in_the_way.rs`:
//!   the untracked files a pull's branch move would overwrite,
//!   kept before the failure is worded,
//!   and the words that name every copy.
//! - `pull_isolation.rs`:
//!   a pull that merges,
//!   read as its fetch and its merge of `FETCH_HEAD`.
//! - `word_merge.rs`:
//!   the three-way merge by line and by word that resolves a conflict Git's line merge reports.
//! - `merge_second_pass_verdicts.rs`:
//!   the limits a file is merged under,
//!   the reasons a conflicted path is left,
//!   and what became of each path.
//! - `merge_second_pass_objects.rs`:
//!   the blob of a merged file,
//!   the index record that stages it as resolved,
//!   and the tree a merge from objects is carried by.
//! - `merge_second_pass.rs`:
//!   the conflicted files a merge left,
//!   which of them the word pass may re-merge,
//!   and the lines that list each merged part.
//! - `merge_isolation_carry_apply.rs`:
//!   writing it back:
//!   the clear before the merge,
//!   the carry of each layer after it,
//!   and the restore of a merge that did not land.
//! - `merge_isolation_apply_findings.rs`:
//!   the words an isolated merge prints when it could not be finished now.
//! - `merge_isolation_apply_moved.rs`:
//!   what a landed merge does when its branch moved while its own carry ran:
//!   the person's work is merged with the branch as it is now.
//! - `merge_isolation_apply_save.rs`:
//!   the merge a command saves for a later one:
//!   its commit on a side ref,
//!   the record that names it,
//!   and the line the person reads.
//! - `merge_isolation_apply_stops.rs`:
//!   what a step of an isolated merge that stopped means for the command's ending,
//!   and what a clear that met newer bytes does about them.
//! - `merge_isolation_carry_layers.rs`:
//!   what one layer of one path ends as once the person's work is merged with the merge.
//! - `merge_isolation_carry_plans.rs`:
//!   what one captured path's writes are,
//!   once the merge's change is known.
//! - `merge_second_pass_git_lines.rs`:
//!   the line merge Git makes for one candidate file,
//!   and the word pass over the regions it leaves.
//! - `merge_second_pass_report.rs`:
//!   the words that list a merged part and the journal entry for a merged file.
//! - `merge_second_pass_tree_attributes.rs`:
//!   the tree a merge computed from objects reads its `merge` attribute from,
//!   and the attribute source that merge names.
//! - `merge_second_pass_tree.rs`:
//!   the word-level second pass for merges computed from objects,
//!   and the attribute tree those merges read.
//! - `merge_isolation_merge_tree.rs`:
//!   the merge tree of an isolated merge,
//!   computed from objects,
//!   and the word-level second pass over its conflicts.
//! - `merge_isolation_apply.rs`:
//!   the writing half of an isolated merge:
//!   the commit and the two-way update.
//! - `merge_isolation.rs`:
//!   a true merge completed in isolation while the index holds staged changes.
//! - `merge_second_pass_worktree_watch.rs`:
//!   the paths a stopped operation had already left unmerged before it ran,
//!   and the conflicts the index holds for the paths it created.
//! - `merge_second_pass_worktree_install.rs`:
//!   the files a forwarded second pass merged,
//!   staged as resolved and written to the worktree.
//! - `merge_second_pass_worktree_unchanged.rs`:
//!   what the word-level second pass will not replace:
//!   a path another command moved while it computed its merge.
//! - `merge_second_pass_worktree_words.rs`:
//!   the sentences the word-level second pass ends a stopped operation with.
//! - `merge_second_pass_worktree_remerge.rs`:
//!   the awaited re-merge of the conflicts a stopped operation created,
//!   and the install of every file no other command moved.
//! - `merge_second_pass_worktree.rs`:
//!   the word-level second pass after a forwarded merge,
//!   rebase,
//!   cherry-pick or revert that Git stopped with a conflict.
//! - `word_merge_invented_text.rs`:
//!   the text a word merge of one region may not write,
//!   which keeps that region in conflict.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // merge/index.ts: one `export * from './file'` per module below.
//! ```

/// The three-way merge by line and by word that resolves a conflict Git's line merge reports.
pub mod word_merge;

/// The text a word merge of one region may not write,
/// which keeps that region in conflict.
pub mod word_merge_invented_text;

/// The limits a file is merged under,
/// the reasons a conflicted path is left,
/// and what became of each path.
pub mod merge_second_pass_verdicts;

/// The blob of a merged file,
/// the index record that stages it as resolved,
/// and the tree a merge from objects is carried by.
pub mod merge_second_pass_objects;

/// The conflicted files a merge left,
/// which of them the word pass may re-merge,
/// and the lines that list each merged part.
pub mod merge_second_pass;

/// The line merge Git makes for a conflicted file,
/// cut into the text it settled and the regions it
/// left.
pub mod merge_second_pass_line_regions;

/// The line merge Git makes for one candidate file,
/// and the word pass over the regions it leaves.
pub mod merge_second_pass_git_lines;

/// The words that list a merged part and the journal entry for a merged file.
pub mod merge_second_pass_report;

/// The conflicted files of a merge tree and the reading of `git merge-tree` output into conflicts.
pub mod merge_second_pass_conflicts;

/// How the word-level second pass starts Git.
pub mod merge_second_pass_git_runner;

/// The tree a merge computed from objects reads its `merge` attribute from,
/// and the attribute source that merge names.
pub mod merge_second_pass_tree_attributes;

/// The word-level second pass for merges computed from objects,
/// and the attribute tree those merges read.
pub mod merge_second_pass_tree;

/// The word-level second pass after a forwarded merge,
/// rebase,
/// cherry-pick or revert that Git stopped with a conflict.
pub mod merge_second_pass_worktree;

/// The paths a stopped operation had already left unmerged before it ran,
/// and the conflicts the index holds for the paths it created.
pub mod merge_second_pass_worktree_watch;

/// The awaited re-merge of the conflicts a stopped operation created,
/// and the install of every file no other command moved.
pub mod merge_second_pass_worktree_remerge;

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

/// The paths a merge that writes or deletes them would discard.
pub mod merge_isolation_dirty_paths;

/// The private name a take gives a file it moves aside,
/// and the merge record that name is tied to.
pub mod merge_isolation_take_aside_names;

/// The take of a worktree file a merge must replace or delete,
/// and the private names it leaves behind.
pub mod merge_isolation_take_asides;

/// The proof that no program can write into a file whose last name is about to go.
pub mod merge_isolation_take_asides_proof;

/// The copies of the person's own bytes a merge keeps beside a path,
/// and the record of each.
pub mod merge_isolation_kept_copies;

/// The one decision every command makes about a copy a merge kept.
pub mod merge_isolation_kept_copies_decisions;

/// The lines a kept copy earns,
/// and the reason it stays when the file at its path holds its bytes.
pub mod merge_isolation_kept_copies_words;

/// The commit message of an isolated merge.
pub mod merge_isolation_message;

/// A pull that merges,
/// read as its fetch and its merge of `FETCH_HEAD`.
pub mod pull_isolation;

/// The untracked files a pull's branch move would overwrite,
/// kept before the failure is worded,
/// and the words that name every copy.
pub mod pull_untracked_paths_in_the_way;

/// One such file or link copied beside the Git directory,
/// and the numbered folder a pull's copies go into.
pub mod pull_untracked_paths_in_the_way_kept_copies;

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
/// What stands at one worktree path, as a merge reads it.
pub mod merge_isolation_worktree_entry;

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

/// What Git's own update left at each path of a merge,
/// which tells its own file from a save that accepts it.
pub mod merge_isolation_carry_landed;

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

/// One path a carry kept whole:
/// what stands at it of the person's work,
/// what the merge made of it,
/// and the line that says both and names the command that reaches the merge's version.
pub mod merge_isolation_carry_kept_path;

/// A directory standing where the merge puts a file,
/// which no layer of the carry can hold,
/// and the sentence that stops the merge before anything is written.
pub mod merge_isolation_a_directory_the_merge_replaces;

/// A named pipe,
/// a socket or a device standing where the merge writes or deletes a path,
/// which no layer of the carry
///  can hold and which no command may open,
/// and the sentence that stops the merge before anything is written.
pub mod merge_isolation_a_special_file_the_merge_changes;

/// The paths a merge changes whose edit Git's status does not show,
/// because their index entry carries the
///  `assume-unchanged` or `skip-worktree` mark.
pub mod merge_isolation_marks_the_status_hides;

/// Putting one planned worktree write in place,
/// or leaving a path alone because what stands there is not the entry that was judged.
pub mod merge_isolation_carry_place;

/// The entry such a write makes:
/// the mode Git's own checkout gives a file,
/// a symbolic link,
/// and the permission bits a person set on their own entry.
pub mod merge_isolation_carry_place_write;

/// The writing half of an isolated merge:
/// the commit and the two-way update.
pub mod merge_isolation_apply;

/// The merge a command saves for a later one:
/// its commit on a side ref,
/// the record that names it,
/// and the line the person reads.
pub mod merge_isolation_apply_save;

/// What a step of an isolated merge that stopped means for the command's ending,
/// and what a clear that met newer bytes does about them.
pub mod merge_isolation_apply_stops;

/// What a landed merge does when its branch moved while its own carry ran:
/// the person's work is merged with the branch as it is now.
pub mod merge_isolation_apply_moved;

/// The words an isolated merge prints when it could not be finished now.
pub mod merge_isolation_apply_findings;

/// The merge tree of an isolated merge,
/// computed from objects,
/// and the word-level second pass over its conflicts.
pub mod merge_isolation_merge_tree;

/// The questions an isolated merge asks Git and the terminal.
pub mod merge_isolation_git_reads;

/// The ignored untracked files of the person's that stand at the paths a merge writes,
/// which `git status` never lists.
pub mod merge_isolation_ignored_paths;

/// The editor a merge opens for its message when the checkout holds the person's own work:
/// the file it reads,
/// the program Git resolves,
/// and the message that comes back.
pub mod merge_isolation_editor;

/// The words of the editor command,
/// which no shell makes.
pub mod merge_isolation_editor_words;

/// The values an isolated merge passes between its steps.
pub mod merge_isolation_run_types;
