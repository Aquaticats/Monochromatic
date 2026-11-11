//! What:
//!  What the commit service does to land a commit:
//!  the serialized step,
//!  the index lock hold,
//!  preparation,
//!  replay and after-commit work.
//! Why:
//!  The landing is the service's core and changes together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/index.ts: one `export * from './file'` per module below.
//! ```

/// The landing location a forwarded index writer coordinates with:
///  the real index and the registry.
pub mod landing_location;

/// Automatic maintenance decision:
/// config record parsing and the enable and detach rules.
pub mod maintenance_decision;

/// Running automatic maintenance after a landing:
/// the config read,
/// then `git maintenance run --auto`.
pub mod maintenance_run;

/// The durable work-item record for asynchronous work (review,
/// push,
/// maintenance) left for a resident consumer.
pub mod work_item;

/// The digest a push's id carries for its branch,
/// and what may follow the kind in an id.
pub mod work_item_ids_and_names;

/// The kinds of work a landed object can ask for,
/// and the names that pick a consumer's handler.
pub mod work_item_kinds_and_their_names;

/// The state folders of the work queue,
/// the names of claim files,
/// and the listing of a folder oldest first.
pub mod work_item_state_directories_and_claim_names;

pub mod work_item_recovery;

/// The steps of a work item that waits for its producer's approval.
pub mod work_item_holding;

/// The clocks a claim's lease is judged by.
pub mod claim_time;

/// The repositories the service looks at again when a claim another instance holds may have run out.
pub mod service_retry;

/// The Git directories the service looks at by itself when it starts.
pub mod service_claim_watch;

/// What the user is told when the commit service cannot land a commit.
pub mod service_messages;

/// The words for a start that cannot come at all,
/// because the address cannot exist or the manager made the service listen elsewhere.
pub mod service_messages_of_a_start_that_cannot_come;

/// The commit service's serialized step.
pub mod service_land;

/// The batches the commit service of an earlier release left open,
/// finished by the service of this release when none runs.
pub mod earlier_service_journal_adopted;

/// A scan of the processes run on the blocking pool and awaited under a ceiling.
pub mod scan_awaited_under_a_ceiling;

/// What one process of the process table says about itself:
/// its command line up to the largest line any program can carry,
/// and the report of a line that cannot be proved.
pub mod earlier_service_command_line_read;

/// The look at the running processes that tells whether a commit service of an earlier release may still run for a worktree.
pub mod earlier_service_process_look;

/// The description of one running program that the look for an earlier commit service could not rule out,
/// for the message that names it.
pub mod earlier_service_unsure_program;

/// What the commit service needs to land a batch.
pub mod service_land_context;

/// Takes the index lock for a batch that is ready to land.
/// What became of one request of a batch,
/// and the announcement a landing makes when nobody is told.
/// What a landing starts with:
/// how many times its batch may be built again,
/// and what it reads once.
pub mod service_land_settings;

/// The landing itself:
/// one batch written,
/// journaled and moved onto a branch.
pub mod service_land_batch_body;

/// The reflog message the branch move writes:
/// the commit's kind in Git's own words and the first line of its message.
pub mod service_land_reflog;

pub mod service_land_outcomes;

pub mod service_land_take_index_lock;

/// The Git steps of a landing,
/// the import stream of a batch and the overlap tests of its requests.
pub mod service_land_git_steps_and_import_stream;

/// The objects a batch lands:
/// the commit Git built,
/// or the commits made from the requests' changes.
pub mod service_land_objects_of_batch;

/// What the commit service's landing does once the branch has moved:
/// index entries and conclusion state.
pub mod service_land_copy;

pub mod service_land_follow;

/// What a landing does when its branch write is refused while the branch is still where the batch found it:
/// the reference locks that stand,
/// the bounded wait for a live holder,
/// and the words for the locks nothing proves gone.
pub mod service_land_reference_lock;

/// The copy the service keeps of the conclusion state a landed commit removes.
pub mod conclusion_state_copy;

/// The record of an index update a landed batch owes.
pub mod owed_index_update;

/// The repair that repeats a landed batch's owed index update for the entries the landing left.
pub mod owed_index_repair;

/// How the repair reads one entry of a tree or an index,
/// and the listings it compares.
pub mod owed_index_entries;

pub mod service_land_start;

/// The commit service's hold on Git's index lock across a landing.
pub mod service_index_lock;

/// The attempts and the wait of an acquisition of the index lock.
pub mod service_index_attempts;

/// The service's warm of the compiled forbidden-strings rules,
/// off the commit's path.
pub mod service_rules_precompile;

/// The foreign index lock that already outlasted a commit's wait,
/// so the commits queued behind it are not made to wait again.
pub mod service_index_refused_lock;

/// The state one landing keeps,
/// carried by the task that runs the landing and not by a thread.
pub mod landing_state_carried_by_its_task;

/// The marker the service writes into the index lock and the questions asked of a marker.
pub mod service_index_marker;

/// The marker of the next batch's index lock,
/// made and flushed while the service is idle.
pub mod service_index_prestage;

/// What an acquisition of the index lock found when it had to wait,
/// for the service's log.
pub mod service_index_found;

/// Who a marker's holder is:
/// the identity a process writes,
/// and the answers a reader gives (gone,
/// running,
/// unknown).
pub mod service_index_identity;

/// The service's hold on a placed index lock:
/// the check that it is still the holder's,
/// the release and the removal.
pub mod service_index_hold;

/// Whether the index lock is still the file the holder placed,
/// and the record of a lock that could not be settled.
pub mod service_index_ownership;

/// The staging files of the index lock.
pub mod service_index_staging;

/// How the service's marked file becomes the index lock,
/// and the cleanup of staging files.
pub mod service_index_placement;

/// The inotify watch the wait for the index lock blocks on.
pub mod service_index_watch;

/// The preparation of a commit that the service lands,
/// for every command line.
/// The notices a commit's preparation prints on standard error,
/// given to the descriptor under a ceiling.
pub mod preparation_notice;

pub mod service_prepare;

/// The findings a preparation ends with when the selection the command line makes cannot be committed.
pub mod service_prepare_selection_refusals;

/// The staging start that puts every unmerged file's working copy into the commit's own index.
pub mod service_prepare_unmerged_staging;

/// The renewal that keeps a commit's scratch tree from aging out while the command that owns it lives.
pub mod service_scratch_renewal;

/// Which commit scratch trees a later command takes for abandoned,
/// and which it leaves alone because their owner still runs.
pub mod service_scratch_abandonment;

/// The staging step of a preparation,
/// for a control that asks it directly.
#[cfg(test)]
pub mod service_prepare_exclude_or_stop_for_test;

/// The file a typed message is kept in when the commit it belonged to cannot be made.
pub mod service_commit_saved_message;

/// The shapes a service preparation hands back.
pub mod service_prepare_ready_shapes;

/// The smaller steps of a service preparation.
pub mod service_prepare_changes_config_and_pathspec_steps;

/// Which commit command lines the commit service covers,
/// and the message they stand for.
pub mod service_commit_args;

/// Whether a `commit` command line opens an editor.
pub mod service_commit_args_editor_opening;

/// The command line a commit is built again with once its editor has run and the branch moved.
pub mod service_commit_line_reissued;

/// The one summary line a landed commit prints,
/// with the abbreviated name Git would print.
pub mod landed_summary;

/// The wait of a wrapped index writer for the commit service's own hold on the index.
pub mod service_index_wait;

/// A forwarded index writer started again when the commit service took the index before its Git did.
pub mod service_hold_retry;

/// The commit command's path through the commit service.
pub mod service_commit;

/// What a commit does when the service answers that the branch moved under it.
pub mod service_commit_after_a_moved_branch;

/// Which end of a commit's scratch tree a submitted request leaves it to.
pub mod service_commit_tree_ownership;

/// What the branch shows once the commit service went silent after a commit was prepared.
pub mod service_branch_shows;

/// The look for the commit a request that was already sent made,
/// once its answer was lost.
pub mod service_commit_looking_for_what_landed;

/// What a landed commit leaves the command to do:
/// its result and the work after it.
pub mod service_landed_aftermath;

/// The small steps of a service commit and the commit the service builds itself.
pub mod service_commit_input_notes_and_embedded_commit;

/// Whether the request the embedded route would send fits the one line the service reads.
pub mod service_commit_fitting_request;

/// The requests of a service commit and what a landed commit leaves for the person.
pub mod service_commit_requests_and_landing_report;

/// The request one attempt of a commit sends:
/// the amended commit landing as a new one,
/// the commit Git builds,
/// or the commit the service builds from captured changes.
pub mod service_commit_request_of;

/// The service's memory of when the bytes of the commits it landed were captured,
/// per worktree.
pub mod service_captures;

/// The retry of a commit the service sent back:
/// the captured change merged onto the branch,
/// never prepared again from the worktree.
pub mod service_replay;

/// The capture order of every shared path of a replay,
/// from one walk over the history.
pub mod service_replay_capture_order;

/// The small text steps of the replay of captured changes.
pub mod service_replay_line_text;

/// Which way a commit goes:
/// through the service,
/// set up first,
/// or the earlier path.
pub mod service_route;

/// The message and identities of a service commit,
/// made as native `git commit` makes them.
pub mod service_message;

/// What an amend takes from the commit it replaces:
/// parents,
/// message and author.
pub mod service_amend;

/// The refusal of a conclusion whose operation another command ended while the commit was prepared.
pub mod service_replay_conclusion;

/// The commit an amend makes when the branch moved under it:
/// the amended commit's own author,
/// committer,
/// message and signature,
/// on the replayed tree,
/// on top of the branch as it is now.
pub mod service_amend_replayed;

/// The commits real Git builds for the service:
/// an editor,
/// a signature,
/// and what the service does not embed.
pub mod service_native;

/// The renewal that keeps the claims of a shared work directory from looking abandoned while their holder works.
pub mod service_claim_renewal;

/// The task of a running claim renewal,
/// its driver for controls,
/// and its stop.
pub mod service_claim_renewal_running;

/// The service's consumer of after-commit push and upkeep work.
pub mod service_work;

/// The notice for a push whose branch the person deleted.
pub mod service_work_branch_is_gone;

/// The kinds,
/// handlers,
/// acceptance rules,
/// settling and replanning of the service's work.
pub mod service_work_handlers_replanning_and_acceptance;

/// Drains the pending items of one work directory.
pub mod service_work_drain_pending_claims;

/// The drain's step that deletes the subvolumes a removal handed over.
pub mod service_work_drain_deletes_subvolumes;

/// What a drained push's verdict does to its records,
/// its retry plan and the person.
pub mod service_work_settling_a_verdict;

/// How one item whose kind answers with words is settled,
/// with the line the person reads for it.
pub mod service_work_settling_with_words;

/// The drain of the snapshots the commit service makes ahead of the next add,
/// as one step of a round.
pub mod service_work_drains_a_snapshot_ahead;

/// The storage steps of one drain of work,
/// each a job of the blocking pool awaited by a task.
pub mod service_work_storage_awaited;

/// The drains of one directory and of a common directory,
/// awaited by a task of the runtime.
pub mod service_work_drains_awaiting;

/// Runs one round of the service's work over several directories.
pub mod service_work_run_round;

/// The drain of several repositories of one round,
/// side by side and without a lock.
pub mod service_round;

/// The wrapper's side of after-commit work:
/// records and the wake.
pub mod service_handoff;

/// Whether Git's own record of a branch's remote says a held push's commit has reached it.
pub mod held_push_commit_on_the_remote;

/// Which command will decide a held commit's post-commit checks,
/// and the verdict that command left.
pub mod held_push_holder_and_verdict;

/// The wake a command gives the service when it releases a held push by itself.
pub mod held_push_release_wakes_the_service;

/// What the filesystem proves about a Git directory the service can no longer resolve.
pub mod unreachable_git_dir_proven_gone;

/// Whether a worktree's head still names the branch a commit was prepared on.
pub mod service_head;

/// What a commit Git built adds to the branch,
/// answered from the object store without starting Git.
pub mod service_built_checks;

/// How a commit headed for the service notices that its prepared content was removed before it landed.
pub mod service_pruned;

/// Where the landing's own syncs wait:
/// queued for a background thread when the landing publishes,
/// made before return otherwise.
pub mod service_durable;

/// The syncs one landing made,
/// what each was for and how long it took,
/// and the words its answer gives for them.
pub mod service_durable_syncs;

pub mod service_land_finish;

/// The removal of a stale index lock,
/// by a rename and a second look.
pub mod service_index_reclaim;

/// How long a commit whose branch moved under its build is built again,
/// and the line that says so.
pub mod service_commit_rebuild_ceiling;

/// The submission of a prepared commit whose waiting notices are written away from the thread that waits.
pub mod service_commit_waiting_notices;

/// The removal of an index lock that no process holds and no Git lock PID file stands behind,
/// on a proof written to the service journal.
pub mod service_index_unheld_removal;

/// The claim chain that keeps two reclaimers apart where the file system gives no `flock`.
pub mod service_index_claim;

/// Whether a new index file holds the live index's bytes.
pub mod service_index_same;

/// The two side references of a landing,
/// and the scratch reference a restart removes once a batch it dropped has its commit kept.
pub mod service_scratch_ref_of_a_dropped_batch;

/// The record that names the worktree of a batch's scratch reference,
/// so a restart can remove a reference whose landing was killed before its intent.
pub mod service_scratch_ref_marker_of_a_batch;

/// The record of a commit a reference lock keeps off its branch:
/// what the branch was,
/// what it goes to,
/// the locks that held the write off,
/// and when the commit was held.
pub mod held_commit_record;

/// The change of a held commit put onto the branch as it is now,
/// when that branch moved while the lock stood.
pub mod held_commit_replay;

/// Moving the branch of a held commit once nothing stands in the way,
/// and finishing what the landing owes after the move.
pub mod held_commit_settle;

/// What became of one held commit when a settlement looked at it,
/// and the journal line the service writes about it.
pub mod held_commit_settle_outcomes;

/// What the wrapper says about a commit that a reference lock keeps off its branch.
pub mod held_commit_words;

/// What a commit does when the service answered without landing it:
/// the words for content a prune removed,
/// and the aftermath of a service that failed or went silent.
pub mod service_commit_after_a_failure;

/// What a landing does when a reference lock keeps its branch where it is,
/// and what it gives back when a batch will never land.
pub mod service_land_hold_a_batch;

/// The commit of a batch a recovery drops,
/// kept on `refs/cli-git/kept/<number>`.
pub mod service_land_kept_commit;
