//! What:
//!  The wrapper's commit path:
//!  arguments,
//!  preparation,
//!  convergence,
//!  conclusion and timing.
//! Why:
//!  A commit is the central transaction of the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commit/index.ts: one `export * from './file'` per module below.
//! ```

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

/// Rule facts from the real repository with candidates read from the private index.
pub mod commit_facts;

/// The worktree completion records of selected corrections and of hook changes.
pub mod commit_worktree_records;

/// The record of where one wrapped commit spent its time,
/// appended beside the discard audit.
pub mod commit_timing;

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

/// The invocation capture:
///  repository layout,
/// symbolic `HEAD`,
/// conclusion kind,
/// ref backend and base.
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

/// The `commit` argument region as Git reads it:
///  which token takes the next as its value,
///  where the options end,
///  which tokens are a value,
///  and the region with every short cluster's value separated.
pub mod commit_args_reading;

/// The private commit index:
/// copies,
/// selection modes,
/// pathspec files and listings.
pub mod commit_index;

/// The worktree paths an explicit-path commit stages,
/// and the two `git add` starts that stage them.
pub mod commit_selection_staging;

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
