//! What:
//!  Worktree commands:
//!  add,
//!  remove,
//!  clean,
//!  convert,
//!  trash,
//!  snapshots and the placement ladder.
//! Why:
//!  Worktree lifecycle code shares plans and journals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // worktree/index.ts: one `export * from './file'` per module below.
//! ```

/// Whether the checkout of a new worktree may be given Git's parallel checkout,
/// and the option that does it.
pub mod checkout_workers;

/// Repository and worktree identity as reported by real Git.
pub mod worktree_identity;

/// The folder a taken worktree folder is kept in when the name it came from is taken,
/// and the cleanup that must leave it alone.
pub mod worktree_kept_folders;

/// The record of where one wrapped `git worktree add` spent its time,
/// step by step,
/// appended beside the commit timing record.
pub mod add_timing;

/// Bringing added paths' worktree copies to their landed bytes.
pub mod worktree_completion;

/// The descriptor-bound look at one added path's worktree copy.
pub mod worktree_completion_file_inspection;

/// What becomes of the unshown push notices and unfinished pushes of a linked worktree that Git removes.
pub mod worktree_removal_rescue;

/// What becomes of the commit a reference lock holds in a linked worktree that Git removes.
pub mod worktree_removal_held_commits;

/// The plan for one `git worktree add` that the cheap-worktree ladder serves:
/// its destination and insertion point.
#[cfg(unix)]
pub mod worktree_add_plan;

/// Small questions about a path or a byte that the add plans ask.
#[cfg(unix)]
pub mod worktree_add_plan_path_questions;

/// The empty subvolume made at the destination of a `git worktree add` before Git runs,
/// and its cleanup.
#[cfg(target_os = "linux")]
pub mod worktree_subvolume;

/// Whether a worktree is clean enough to copy from or to remove without asking Git,
/// proven in process.
#[cfg(target_os = "linux")]
pub mod worktree_clean;

/// What a clean check says about one tracked file:
/// refusals,
/// recorded stat data and the metadata verdict.
#[cfg(target_os = "linux")]
pub mod worktree_clean_stat_verdicts;

/// The one question the index of a worktree cannot answer:
/// whether an untracked file is one Git ignores.
#[cfg(target_os = "linux")]
pub mod worktree_clean_untracked_scan;

/// The walks of tracked files against the index's recorded stat data,
/// by metadata or by content.
#[cfg(target_os = "linux")]
pub mod worktree_clean_tree_walk;

pub mod worktree_attributes;

pub mod worktree_clean_content;

/// The attribute files a content decision reads,
/// and what it charges the caller for them.
pub mod worktree_clean_content_attributes;

/// The environment and attribute files a content check reads,
/// and the attributes that convert content.
#[cfg(target_os = "linux")]
pub mod worktree_clean_content_attribute_sources;

/// The plan of a `git worktree remove` that names one directory and no option.
#[cfg(unix)]
pub mod worktree_remove_plan;

/// The removal of a clean subvolume worktree by one ioctl,
/// and the plan of the `git worktree remove` it serves.
#[cfg(target_os = "linux")]
pub mod worktree_remove;
pub mod worktree_remove_hand_over;

pub mod worktree_remove_registry;

/// The three reports a removal can end with.
pub mod worktree_remove_reports;

/// The record,
/// beside a worktree,
/// of the objects one conversion may leave at its converting name.
pub mod worktree_convert_leftover_stage_record;

/// What a removal snapshot kept,
/// promoted to the notices every worktree shows.
pub mod worktree_removal_snapshot_promotion;

/// The finish of a removal whose destroy happened and whose registration did not go.
pub mod worktree_removal_interrupted_destroy;

/// The stop of a removed worktree's commit service units,
/// named from the registration before the removal and stopped after it.
pub mod worktree_removal_stops_units;

/// The removal of the shadow pool a removed worktree's commits were served from,
/// taken after the removal proved the worktree is gone.
pub mod worktree_removal_clears_the_pool;

/// The one message shape of a failed step on a path:
/// `<path> <what> (<error>)`.
pub mod worktree_path_failure_message;

/// Fixtures shared by the controls of the cheap-worktree ladder.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod worktree_ladder_test_support;

/// What Git's status lists as changed or untracked in a worktree,
/// for the snapshot source.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source_changed_paths;

/// The snapshot source:
/// whether the creating worktree can be snapshotted for a new worktree,
/// and what that needs.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source;

/// The snapshot rung's mechanics:
/// snapshot to a stage name,
/// make it a worktree,
/// swap it into place.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot;

/// The index step of the snapshot rung:
/// prove the copy against the source's index,
/// settle an entry by content where the stat data cannot vouch for it,
/// and keep the
///  snapshot's own index when the source changed under the copy.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_index_step;

/// The snapshots the wrapper makes ahead of an add,
/// and the add's claim of one instead of a snapshot of its own.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_ahead;

/// What one add does with the snapshots the service made ahead of it:
/// the proof,
/// the claim,
/// the bounded wait and the sweep.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_ahead_claim;

/// The commit service's own snapshot of a worktree,
/// made for the add after the one that asked for it.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_ahead_job;

/// The empty folders a snapshot holds where its source holds a nested subvolume it did not carry,
/// found below the folders that
///  hold worktrees and removed as empty directories.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_placeholders;

/// The binding of a `git worktree add` to the directories Git made for it,
/// its record,
/// and the check that the binding holds.
#[cfg(target_os = "linux")]
pub mod worktree_add_binding;

/// The clone rung:
/// tracked files cloned into the new worktree,
/// the source's index installed with fresh stat fields.
#[cfg(target_os = "linux")]
pub mod worktree_clone;

/// What both filling rungs read from Git's `--no-checkout` result,
/// and the one step that applies the tree difference.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_target;

/// Which of the ladder's own Git steps has a ceiling,
/// and which number that ceiling is.
#[cfg(target_os = "linux")]
pub mod ladder_step_ceilings;

/// The ladder's choice of rung before real Git runs.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_run;

/// Where the words of a Git run the wrapper or the service started on its own go,
/// now that none carries `--quiet`.
pub mod inner_git_words;

/// The plain sentence that says why an add was not made as a snapshot,
/// and where it goes.
#[cfg(target_os = "linux")]
pub mod worktree_slow_route;

/// The ladder's steps after real Git returned:
/// the snapshot rung and the fall back to Git's checkout.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_finish;

/// The record of the trash a snapshot's leave-out made,
/// written before the rung that published the worktree returns.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_finish_trash_record;

/// The restore of a snapshot of a main checkout with uncommitted changes,
/// and its proof.
#[cfg(target_os = "linux")]
pub mod worktree_dirty_restore;

/// The facts one audit line of a removed worktree carries.
pub mod worktree_audit;

/// The cheap-worktree ladder around one `git worktree add`:
/// what runs before real Git and after it.
#[cfg(unix)]
pub mod worktree_ladder;

/// Quoting of bytes printed inside a shell command for a person to run.
#[cfg(target_os = "linux")]
pub mod shell_quote;

/// The commands a message prints to finish or remove a worktree an interrupted `git worktree add` left.
#[cfg(target_os = "linux")]
pub mod worktree_finish_commands;

/// Converting a linked worktree that is an ordinary directory into a btrfs subvolume at the same path,
/// resumably.
#[cfg(target_os = "linux")]
pub mod worktree_convert;

/// The first look at a worktree a conversion might change,
/// and every reason the conversion must not start.
#[cfg(target_os = "linux")]
pub mod worktree_convert_inspection;

/// The journaled phases of one worktree conversion.
#[cfg(target_os = "linux")]
pub mod worktree_convert_phases;

/// What a conversion does about a stage container another command is filling below the tree it would swap or delete.
#[cfg(target_os = "linux")]
pub mod worktree_convert_filled_stages;

/// The empty stage a conversion created and did not record,
/// and whether the object at the converting name is it.
#[cfg(target_os = "linux")]
pub mod worktree_convert_unrecorded_stage;

/// The read-only comparison of the old tree of a conversion with the installed one.
#[cfg(target_os = "linux")]
pub mod worktree_convert_compare;

/// The journal of a conversion and the retirement of the kernel lock file older releases left.
#[cfg(target_os = "linux")]
pub mod worktree_convert_journal;

/// The removal of the old tree of a conversion beneath a pinned descriptor.
#[cfg(target_os = "linux")]
pub mod worktree_convert_remove;

/// The grammar of `git monochromatic-git-policy-cli convert-worktree-to-subvolume`.
pub mod worktree_convert_arguments;

/// `git monochromatic-git-policy-cli convert-worktree-to-subvolume`:
/// the worktrees to cover and the report.
#[cfg(target_os = "linux")]
pub mod worktree_convert_command;

/// Setting aside what a snapshot of the main checkout leaves out,
/// and having the commit service empty it.
#[cfg(target_os = "linux")]
pub mod worktree_trash;

/// The repository's own exclude line that keeps a standing trash out of every `git add --all`.
#[cfg(target_os = "linux")]
pub mod worktree_trash_exclude_line;

/// Handing a snapshot's trash to the commit service without waiting for it to be emptied.
#[cfg(target_os = "linux")]
pub mod worktree_trash_hand_over;

/// Running one stretch of storage work at the lowest I/O priority Linux offers.
#[cfg(target_os = "linux")]
pub mod idle_io_priority;

/// Emptying a snapshot's trash:
/// what a trash is,
/// the service's handler,
/// and the trashes nothing will empty.
#[cfg(target_os = "linux")]
pub mod worktree_trash_dispose;

/// The service's handler for a trash work item,
/// awaited by a caller that is itself a task of the runtime.
pub mod worktree_trash_dispose_awaiting;

/// Whether Git ignores the place a snapshot's trash would be made at in one worktree.
#[cfg(target_os = "linux")]
pub mod worktree_trash_ignore_rule;

/// The removal of the entries a snapshot's leave-out moved into its trash,
/// side by side.
#[cfg(target_os = "linux")]
pub mod worktree_trash_entries;

/// What a snapshot's leave-out made and moved,
/// with identities,
/// as the text a work item carries.
#[cfg(target_os = "linux")]
pub mod worktree_trash_manifest;

/// The manifest a trash carries inside itself,
/// written before the swap publishes it,
/// so a later pass can prove what the trash is without a record.
#[cfg(target_os = "linux")]
pub mod worktree_trash_own_manifest;

/// What every later command in a worktree says about the trash left there:
///  which directory nothing will empty,
///  which trash it records again,
///  and the wake that tells the service so.
pub mod worktree_trash_unaccounted;

/// Removal of a directory tree beneath a held parent handle,
/// never following a link.
#[cfg(target_os = "linux")]
pub mod worktree_trash_remove;

/// The record that says the wrapper made one linked worktree,
/// which every automatic removal asks for first.
pub mod worktree_added_by_the_wrapper;

/// Whether a directory is one of this repository's linked worktrees,
/// proved from the pointer and the registration Git wrote there,
/// which every command that changes state in a recorded worktree asks for first.
pub mod worktree_registration_proof;

/// The commit and the branch one merge stood between,
/// and the sweep of the worktrees whose branches it landed.
pub mod worktree_landed_after_a_merge;

/// The linked worktrees a repository registers,
/// with the branch each one has checked out.
pub mod worktree_landed_registrations;

/// The ignored entries of a landed worktree that nothing else in the repository holds,
/// which keep it.
pub mod worktree_landed_ignored_files;

/// The removal of every wrapper-made worktree whose branch a merge landed.
pub mod worktree_landed_removal;

/// The words one landed sweep prints.
pub mod worktree_landed_sweep_notice;

/// Whether any process is working inside a folder,
/// which keeps a landed worktree.
pub mod worktree_process_inside;
