//! What:
//!  The wrapper's commit path:
//!  arguments,
//!  preparation,
//!  convergence,
//!  conclusion and timing.
//! Why:
//!  A commit is the central transaction of the wrapper.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `commit_accounting.rs`:
//!   how much of a commit's time its timing record names,
//!   and how much no phase covers.
//! - `commit_args_token_bytes.rs`:
//!   one token of a commit command line,
//!   read as the bytes it is:
//!   an option's name,
//!   a cluster's letters and the value it carries inline.
//! - `commit_args.rs`:
//!   argument transforms of native preparation:
//!   normalisation,
//!   global split,
//!   `-a` removal and private arguments.
//! - `commit_conclusion.rs`:
//!   the conclusion of a settled commit transaction:
//!   preparation,
//!   landing and post-landing completion.
//! - `commit_index_builds.rs`:
//!   how each build of the private commit index was made,
//!   and whether it kept the real index's stat data,
//!   for the timing record.
//! - `commit_index_selection.rs`:
//!   the selection a commit names:
//!   the choices that shape its private index,
//!   its `git add` arguments and the interactive selection.
//! - `commit_no_change.rs`:
//!   the normalization-only transaction for a correction that removed every change.
//! - `commit_pathspec_matching_nothing.rs`:
//!   the pathspec a caller names that Git refuses for matching nothing,
//!   read from the words of the private staging (issue 6283).
//! - `commit_prepare_part_durations.rs`:
//!   how long each stretch of a commit's preparation held no Git child,
//!   by name,
//!   for the timing record.
//! - `commit_build_part_durations.rs`:
//!   how long each stretch of a commit's build phase held no Git child,
//!   by name,
//!   for the timing record.
//! - `commit_reach_trace.rs`:
//!   what reaching the commit service cost,
//!   as the timing record states it.
//! - `commit_worktree_records.rs`:
//!   the worktree completion records of selected corrections and of hook changes.
//! - `conclusion_cleanup.rs`:
//!   native conclusion cleanup reproduced in the owning worktree.
//! - `conclusion_copy.rs`:
//!   the shadow conclusion copies at preparation.
//! - `git_report_answers.rs`:
//!   the answers of one `git rev-parse` report the invocation capture reads.
//! - `capture_invocation_git_questions.rs`:
//!   the Git questions one invocation capture asks,
//!   and the words a question that could not start is reported with.
//! - `capture_invocation_layout_report.rs`:
//!   the repository layout of one invocation:
//!   the one report that answers every pathname,
//!   and the questions asked one pathname at a time when that report cannot be split.
//! - `capture_invocation_head_and_conclusion.rs`:
//!   the symbolic `HEAD` of the owning worktree,
//!   and the conclusion kind an amend or a sequencer marker decides.
//! - `capture_invocation_ref_storage_and_base.rs`:
//!   the ref storage backend,
//!   the commit a ref names at invocation,
//!   and the empty tree an unborn ref starts from.
//! - `capture_invocation.rs`:
//!   the invocation capture:
//!   the facts it records,
//!   and the two steps that record them.
//! - `git_step_failure.rs`:
//!   the last Git step of this process that failed,
//!   and the status it exited with,
//!   for the timing record.
//! - `native_commit_configuration.rs`:
//!   the configuration native `git commit` reads that decides what it writes,
//!   and its effective values.
//! - `post_commit.rs`:
//!   the post-commit gate over a landed commit:
//!   its events,
//!   its blocking and the backup it allows.
//! - `prepare_work_spans.rs`:
//!   what the preparation's own work did,
//!   by name,
//!   for the timing record:
//!   reading and copying the index,
//!   writing its entries,
//!   and each policy.
//! - `commit_index_bytes.rs`:
//!   the one read of the real index that every private copy of a commit is written from.
//! - `commit_form_refusal_from_git.rs`:
//!   the refusal Git prints for such a form,
//!   asked of Git itself.
//! - `commit_index_stat_staleness_staging.rs`:
//!   what a commit stages from the worktree,
//!   as the stat-data question reads it.
//! - `commit_index_stat_staleness_worktree_file.rs`:
//!   one worktree file's stat data as an index entry records it,
//!   and whether an entry's recorded data is still that file's.
//! - `commit_index_stat_staleness_left_to_git.rs`:
//!   the entries and the paths the stat-data question leaves to Git.
//! - `commit_index_stat_staleness_walk.rs`:
//!   the walk of an index's entries,
//!   one slice at a time on several threads,
//!   and the answer it gives.
//! - `commit_index_stat_staleness_reading.rs`:
//!   the one reading of an index file the stat-data question is answered from.
//! - `commit_index_stat_staleness.rs`:
//!   the question whether an index still records the stat data its worktree has,
//!   so the refresh runs only when it would change something.
//! - `private_index_request.rs`:
//!   the private index and object store a commit's preparation selects,
//!   and the Git request that carries them.
//! - `commit_index_copying.rs`:
//!   the copying of the real index into a private index and the check of whether it may seed an explicit-path commit.
//! - `commit_index_stat_refresh.rs`:
//!   the refresh of an index's stat data before the scratch commit,
//!   so its ceiling does not end a commit over a moved worktree.
//! - `commit_staged_additions.rs`:
//!   the finding for a staged deletion that an explicit-path commit cannot record.
//! - `commit_selection_staging_failure.rs`:
//!   why a start of the private staging ended:
//!   Git's own failure,
//!   or the caller's pathspec that matches nothing.
//! - `commit_selection_staging_element_words.rs`:
//!   the words one pathspec element is read as:
//!   an exact file name,
//!   a directory or a pattern,
//!   or a subtraction that applies to every element of the list.
//! - `commit_selection_staging_selection.rs`:
//!   what one selection gives the staging step:
//!   its elements,
//!   the pathspec file itself,
//!   or nothing when this reader cannot take that file.
//! - `commit_selection_staging.rs`:
//!   the worktree paths an explicit-path commit stages,
//!   and the two `git add` starts that stage them.
//! - `commit_index_private_git.rs`:
//!   the one Git start every private-index step makes,
//!   and the empty index a repository with no index file starts from.
//! - `commit_index_listings.rs`:
//!   what the private index says:
//!   the intended tree written from it,
//!   the paths with unmerged stages,
//!   and the paths it changes against the recorded base.
//! - `commit_index_checks.rs`:
//!   the questions about the private index this process answers without a Git start:
//!   whether it holds exactly the base revision's tree,
//!   and which selected paths the real index stages as additions.
//! - `commit_index_staging.rs`:
//!   the staging of one selection into the private commit index:
//!   the copy of the index this preparation read,
//!   the reset to the base an explicit-path commit needs,
//!   and the `git add` that stages the selected paths.
//! - `commit_index_retry.rs`:
//!   the second attempt of an initialization whose branch moved under the commit,
//!   which rebuilds the private index to hold exactly the tree it commits.
//! - `commit_index.rs`:
//!   the private commit index of one transaction:
//!   its files,
//!   and the initialization that reads the real index once and refreshes the private copy.
//! - `commit_named_ignored_files.rs`:
//!   the named files an ignore rule refuses,
//!   decided from the selection before any `git add` runs (issue 6278).
//! - `commit_staged_deletions.rs`:
//!   the commit-only finding for a staged deletion that an explicit-path commit cannot record because the path is
//!   still on disk.
//! - `private_index_stage.rs`:
//!   slice 6:
//!   writes one corrected file into the private commit index as a blob and an index entry.
//! - `commit_convergence.rs`:
//!   slice 6:
//!   the commit path's bounded policy convergence over the private index.
//! - `rule_commit_index.rs`:
//!   pure core of the index-against-`HEAD` check.
//! - `rule_commit_only_message.rs`:
//!   user-facing diagnostics of the commit-only transform.
//! - `rule_commit_region_git_refuses.rs`:
//!   which forms of `git commit` real Git refuses,
//!   read from the options the person wrote.
//! - `rule_commit_sequencer.rs`:
//!   pure core of the merge,
//!   cherry-pick and revert conclusion check.
//! - `commit_facts.rs`:
//!   rule facts from the real repository with candidates read from the private index.
//! - `commit_transaction.rs`:
//!   the commit transaction in the incumbent's order,
//!   up to the settled policies.
//! - `commit_stop_reason.rs`:
//!   why one commit did not land,
//!   as the timing record states it.
//! - `rule_commit_only.rs`:
//!   pure decision of the commit-only transform.
//! - `scratch_commit_step.rs`:
//!   what the waiting notice of the scratch `git commit` saw while the step ran,
//!   for the timing record.
//! - `commit_timing_submit.rs`:
//!   what the client did to hand one request to the service and get its answer,
//!   as the timing record states it.
//! - `commit_timing_place.rs`:
//!   where one timing record goes:
//!   the worktree's Git directory,
//!   the common Git directory that holds the file,
//!   and the service instance it names.
//! - `commit_timing_json_words.rs`:
//!   the small JSON words one timing record is written with:
//!   a duration in whole milliseconds,
//!   the word for a field with no value,
//!   and the landing's own figures as one object.
//! - `commit_timing.rs`:
//!   the record of where one wrapped commit spent its time:
//!   its fields and the clocks that fill them.
//! - `commit_timing_record_line.rs`:
//!   one timing record as the single line of JSON it is written as.
//! - `commit_timing_record_writing.rs`:
//!   the one append that writes a timing record to its file,
//!   best effort.
//! - `commit_command_input.rs`:
//!   what one commit command is handed,
//!   and how it ended.
//! - `commit_command_directory.rs`:
//!   the directory one commit command runs in:
//!   the effective invocation directory after every `-C`,
//!   and the real Git the commit's own starts run.
//! - `commit_command_words.rs`:
//!   the words one commit command says when it stops,
//!   each handed to the thread that writes standard error.
//! - `commit_command_form_refusal.rs`:
//!   the refusal of a `git commit` whose options are a form real Git refuses,
//!   with Git's own status and bytes.
//! - `commit_command.rs`:
//!   the `git commit` command:
//!   the transaction,
//!   the settled events,
//!   the post-commit gate and the backup.
//! - `scratch_step_ceiling_stop.rs`:
//!   what the ceiling of the scratch `git commit` does when it comes:
//!   ends the step it watches,
//!   and answers whether that step was signalled.
//! - `scratch_step_held_by_the_kernel.rs`:
//!   the line a commit prints when the step it stopped is still held by the kernel.
//! - `scratch_step_look_below.rs`:
//!   one look at the step the commit waits for and the programs below it.
//! - `scratch_step_notice_line.rs`:
//!   the line a commit prints while it waits for the `git commit` of its scratch repository.
//! - `scratch_step_programs_below.rs`:
//!   the programs a look of the notice saw below the step,
//!   and the time each was credited with.
//! - `scratch_step_stall_decided_once.rs`:
//!   the one decision two enforcers of a step's ceiling make together:
//!   whether the ceiling stands aside for a step the kernel holds in a disk wait.
//! - `native_commit_waiting_notice.rs`:
//!   the waiting notice of the commit Git builds in the scratch repository.
//! - `scratch_step_stop_words.rs`:
//!   the words a stopped scratch `git commit` prints.
//! - `scratch_step_stop_record.rs`:
//!   the record of one stop of the scratch `git commit`,
//!   taken whole from the enforcer of its ceiling that ended the step.
//! - `commit_preparation.rs`:
//!   native `git commit` in the shadow,
//!   the prepared commit read back and `prepared.json`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commit/index.ts: one `export * from './file'` per module below.
//! ```

/// Which forms of `git commit` real Git refuses,
///  read from the options the person wrote.
pub mod rule_commit_region_git_refuses;

/// The refusal Git prints for such a form,
///  asked of Git itself.
pub mod commit_form_refusal_from_git;

/// Pure core of the index-against-`HEAD` check.
pub mod rule_commit_index;

/// Pure decision of the commit-only transform.
pub mod rule_commit_only;

/// User-facing diagnostics of the commit-only transform.
pub mod rule_commit_only_message;

/// Pure core of the merge,
///  cherry-pick and revert conclusion check.
pub mod rule_commit_sequencer;

/// Slice 6:
/// writes one corrected file into the private commit index as a blob and an index entry.
pub mod private_index_stage;

/// Slice 6:
/// the commit path's bounded policy convergence over the private index.
pub mod commit_convergence;

/// The commit transaction in the incumbent's order,
/// up to the settled policies.
pub mod commit_transaction;

/// The conclusion of a settled commit transaction:
/// preparation,
/// landing and post-landing completion.
pub mod commit_conclusion;

/// The normalization-only transaction for a correction that removed every change.
pub mod commit_no_change;

/// The finding for a staged deletion that an explicit-path commit cannot record.
pub mod commit_staged_additions;

pub mod commit_staged_deletions;

/// The `git commit` command:
/// the transaction,
/// the settled events,
/// the post-commit gate and the backup.
pub mod commit_command;

/// What one commit command is handed,
/// and how it ended.
pub mod commit_command_input;

/// The directory one commit command runs in:
/// the effective invocation directory after every `-C`,
/// and the real Git the commit's own starts run.
pub mod commit_command_directory;

/// The words one commit command says when it stops,
/// each handed to the thread that writes standard error.
pub mod commit_command_words;

/// The refusal of a `git commit` whose options are a form real Git refuses,
/// with Git's own status and bytes.
pub mod commit_command_form_refusal;

/// Rule facts from the real repository with candidates read from the private index.
pub mod commit_facts;

/// The worktree completion records of selected corrections and of hook changes.
pub mod commit_worktree_records;

/// The record of where one wrapped commit spent its time:
/// its fields and the clocks that fill them.
pub mod commit_timing;

/// What the client did to hand one request to the service and get its answer,
/// as the timing record states it.
pub mod commit_timing_submit;

/// Where one timing record goes:
/// the worktree's Git directory,
/// the common Git directory that holds the file,
/// and the service instance it names.
pub mod commit_timing_place;

/// The small JSON words one timing record is written with:
/// a duration in whole milliseconds,
/// the word for a field with no value,
/// and the landing's own figures as one object.
pub mod commit_timing_json_words;

/// One timing record as the single line of JSON it is written as.
pub mod commit_timing_record_line;

/// The one append that writes a timing record to its file,
/// best effort.
pub mod commit_timing_record_writing;

/// How much of a commit's time its timing record names,
/// and how much no phase covers.
pub mod commit_accounting;

/// What reaching the commit service cost,
/// as the timing record states it.
pub mod commit_reach_trace;

/// Why one commit did not land,
/// as the timing record states it.
pub mod commit_stop_reason;

/// The last Git step of this process that failed,
/// and the status it exited with,
/// for the timing record.
pub mod git_step_failure;

/// How long each stretch of a commit's preparation held no Git child,
/// by name,
/// for the timing record.
pub mod commit_prepare_part_durations;

/// How long each stretch of a commit's build phase held no Git child,
/// by name,
/// for the timing record.
pub mod commit_build_part_durations;

/// Native conclusion cleanup reproduced in the owning worktree.
pub mod conclusion_cleanup;

/// The Git questions one invocation capture asks,
/// and the words a question that could not start is reported with.
pub mod capture_invocation_git_questions;

/// The repository layout of one invocation:
/// the one report that answers every pathname,
/// and the questions asked one pathname at a time when that report cannot be split.
pub mod capture_invocation_layout_report;

/// The symbolic `HEAD` of the owning worktree,
/// and the conclusion kind an amend or a sequencer marker decides.
pub mod capture_invocation_head_and_conclusion;

/// The ref storage backend,
/// the commit a ref names at invocation,
/// and the empty tree an unborn ref starts from.
pub mod capture_invocation_ref_storage_and_base;

/// The invocation capture:
///  the facts it records,
///  and the two steps that record them.
pub mod capture_invocation;

/// The answers of one `git rev-parse` report the invocation capture reads.
pub mod git_report_answers;

/// The private index and object store a commit's preparation selects,
/// and the Git request that carries them.
pub mod private_index_request;

/// The shadow conclusion copies at preparation.
pub mod conclusion_copy;

/// Argument transforms of native preparation:
/// normalisation,
/// global split,
/// `-a` removal and private arguments.
pub mod commit_args;

/// One token of a commit command line,
/// read as the bytes it is:
/// an option's name,
/// a cluster's letters and the value it carries inline.
pub mod commit_args_token_bytes;

/// The private commit index of one transaction:
/// its files,
/// and the initialization that reads the real index once and refreshes the private copy.
pub mod commit_index;

/// The one Git start every private-index step makes,
/// and the empty index a repository with no index file starts from.
pub mod commit_index_private_git;

/// What the private index says:
/// the intended tree written from it,
/// the paths with unmerged stages,
/// and the paths it changes against the recorded base.
pub mod commit_index_listings;

/// The questions about the private index this process answers without a Git start:
/// whether it holds exactly the base revision's tree,
/// and which selected paths the real index stages as additions.
pub mod commit_index_checks;

/// The staging of one selection into the private commit index:
/// the copy of the index this preparation read,
/// the reset to the base an explicit-path commit needs,
/// and the `git add` that stages the selected paths.
pub mod commit_index_staging;

/// The second attempt of an initialization whose branch moved under the commit,
/// which rebuilds the private index to hold exactly the tree it commits.
pub mod commit_index_retry;

/// The worktree paths an explicit-path commit stages,
/// and the two `git add` starts that stage them.
pub mod commit_selection_staging;

/// Why a start of the private staging ended:
/// Git's own failure,
/// or the caller's pathspec that matches nothing.
pub mod commit_selection_staging_failure;

/// The words one pathspec element is read as:
/// an exact file name,
/// a directory or a pattern,
/// or a subtraction that applies to every element of the list.
pub mod commit_selection_staging_element_words;

/// What one selection gives the staging step:
/// its elements,
/// the pathspec file itself,
/// or nothing when this reader cannot take that file.
pub mod commit_selection_staging_selection;

/// The named files an ignore rule refuses,
/// decided from the selection before any `git add` runs (issue 6278).
pub mod commit_named_ignored_files;

/// The pathspec a caller names that Git refuses for matching nothing,
/// read from the words of the private staging (issue 6283).
pub mod commit_pathspec_matching_nothing;

/// The one read of the real index that every private copy of a commit is written from.
pub mod commit_index_bytes;

/// The copying of the real index into a private index and the check of whether it may seed an explicit-path commit.
pub mod commit_index_copying;

/// How each build of the private commit index was made,
/// and whether it kept the real index's stat data,
/// for the timing record.
pub mod commit_index_builds;

/// What the waiting notice of the scratch `git commit` saw while the step ran,
/// for the timing record.
pub mod scratch_commit_step;

/// What the preparation's own work did,
/// by name,
/// for the timing record:
/// reading and copying the index,
/// writing its entries,
/// and each policy.
pub mod prepare_work_spans;

/// The refresh of an index's stat data before the scratch commit,
/// so its ceiling does not end a commit over a moved worktree.
pub mod commit_index_stat_refresh;

/// Whether an index still records the stat data its worktree has,
/// so the refresh runs only when it would change something.
pub mod commit_index_stat_staleness;

/// What a commit stages from the worktree,
/// as the stat-data question reads it.
pub mod commit_index_stat_staleness_staging;

/// One worktree file's stat data as an index entry records it,
/// and whether an entry's recorded data is still that file's.
pub mod commit_index_stat_staleness_worktree_file;

/// The entries and the paths the stat-data question leaves to Git.
pub mod commit_index_stat_staleness_left_to_git;

/// The walk of an index's entries,
/// one slice at a time on several threads,
/// and the answer it gives.
pub mod commit_index_stat_staleness_walk;

/// The one reading of an index file the stat-data question is answered from.
pub mod commit_index_stat_staleness_reading;

/// The selection a commit names:
/// the choices that shape its private index,
/// its `git add` arguments and the interactive selection.
pub mod commit_index_selection;

/// Native `git commit` in the shadow,
/// the prepared commit read back and `prepared.json`.
pub mod commit_preparation;

/// The configuration native `git commit` reads that decides what it writes,
/// and its effective values.
pub mod native_commit_configuration;

/// What the ceiling of the scratch `git commit` does when it comes:
/// ends the step it watches,
/// and answers whether that step was signalled.
pub mod scratch_step_ceiling_stop;

/// The words a stopped scratch `git commit` prints.
pub mod scratch_step_stop_words;

/// The record of one stop of the scratch `git commit`,
/// taken whole from the enforcer of its ceiling that ended the step.
pub mod scratch_step_stop_record;

/// The one decision two enforcers of a step's ceiling make together:
/// whether the ceiling stands aside for a step the kernel holds in a disk wait.
pub mod scratch_step_stall_decided_once;

/// The line a commit prints while it waits for the `git commit` of its scratch repository.
pub mod scratch_step_notice_line;

/// The line a commit prints when the step it stopped is still held by the kernel.
pub mod scratch_step_held_by_the_kernel;

/// The programs a look of the notice saw below the step,
/// and the time each was credited with.
pub mod scratch_step_programs_below;

/// One look at the step the commit waits for and the programs below it.
pub mod scratch_step_look_below;

/// The waiting notice of the commit Git builds in the scratch repository.
pub mod native_commit_waiting_notice;

/// The post-commit gate over a landed commit:
/// its events,
/// its blocking and the backup it allows.
pub mod post_commit;
