//! What:
//!  What the commit service does to land a commit:
//!  the serialized step,
//!  the index lock hold,
//!  preparation,
//!  replay and after-commit work.
//! Why:
//!  The landing is the service's core and changes together.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `service_durable_the_background_sync_thread.rs`:
//!   the task that makes a landing's syncs after the landing answered,
//!   and the wait a control makes until its queue is empty.
//! - `service_index_prestage_the_markers_made_ahead.rs`:
//!   the marker this service keeps made ahead:
//!   the marked file,
//!   its slot,
//!   the record of every file made,
//!   and the step that makes and records one.
//! - `the_clause_a_commit_no_reference_proves.rs`:
//!   the clause a sentence about a commit ends with when no reference is proven to name it.
//! - `the_memory_one_round_keeps.rs`:
//!   the bytes one repository's drain holds at its peak,
//!   and the measurement those bytes are rounded up from.
//! - `the_names_of_the_index_lock.rs`:
//!   the path of an index lock,
//!   how often a holder's line is repeated,
//!   what the wait tells the clients,
//!   why a wait gave up,
//!   and the code a storage error stands for.
//! - `service_messages_the_failures_a_person_can_meet.rs`:
//!   every failure the client can meet,
//!   and the facts its text names.
//! - `service_messages_what_a_failure_gives_a_person.rs`:
//!   the commands and words a failure hands a person to act on.
//! - `conclusion_state_copy.rs`:
//!   the copy the service keeps of the conclusion state a landed commit removes.
//! - `earlier_service_look_paths.rs`:
//!   the questions about the file system that the look at an earlier commit service asks:
//!   the bytes that spell a path,
//!   the part of a path the file system knows,
//!   and the identity of each earlier journal that exists.
//! - `earlier_service_unsure_program.rs`:
//!   the description of one running program that the look for an earlier commit service could not rule out,
//!   for the message that names it.
//! - `earlier_service_command_line_read.rs`:
//!   what one process of the process table says about itself:
//!   its command line up to the largest line any program can carry,
//!   and the report of a line that cannot be proved.
//! - `held_commit_record.rs`:
//!   the record of a commit a reference lock keeps off its branch:
//!   what the branch was,
//!   what it goes to,
//!   the locks that held the write off,
//!   and when the commit was held.
//! - `held_push_commit_on_the_remote.rs`:
//!   whether Git's own record of a branch's remote says a held push's commit has reached it.
//! - `housekeeping_of_one_round.rs`:
//!   how long one round may go on starting housekeeping,
//!   and what it leaves for a later round.
//! - `landed_summary.rs`:
//!   the one summary line a landed commit prints,
//!   with the abbreviated name Git would print.
//! - `landing_location.rs`:
//!   the landing location a forwarded index writer coordinates with:
//!   the real index and the registry.
//! - `maintenance_decision.rs`:
//!   automatic maintenance decision:
//!   config record parsing and the enable and detach rules.
//! - `maintenance_run.rs`:
//!   running automatic maintenance after a landing:
//!   the config read,
//!   then `git maintenance run --auto`.
//! - `owed_index_entries.rs`:
//!   how the repair reads one entry of a tree or an index,
//!   and the listings it compares.
//! - `preparation_notice.rs`:
//!   the preparation of a commit that the service lands,
//!   for every command line.
//!   The notices a commit's preparation prints on standard error,
//!   given to the descriptor under a ceiling.
//! - `scan_awaited_under_a_ceiling.rs`:
//!   a scan of the processes run on the blocking pool and awaited under a ceiling.
//! - `earlier_service_process_look_answers.rs`:
//!   what the look at an earlier commit service answers with,
//!   and the words of a service's start it reads.
//! - `earlier_service_process_start_arguments.rs`:
//!   which earlier commit service one command line of a process names.
//! - `earlier_service_process_look.rs`:
//!   the look at the running processes and the sockets that tells whether a commit service of an earlier release may
//!   still run or may start for a worktree.
//! - `service_captures.rs`:
//!   the service's memory of when the bytes of the commits it landed were captured,
//!   per worktree.
//! - `service_claim_watch_note_format.rs`:
//!   the bytes of one note the claim watch keeps:
//!   the format words,
//!   the bound on its length,
//!   and the spelling of a path in hexadecimal.
//! - `service_claim_watch_whose_repository_it_is.rs`:
//!   whether a Git directory is the repository of the instance whose journal a note is written into.
//! - `service_claim_watch_writing_a_note.rs`:
//!   writing a note:
//!   the landing's own write,
//!   and the store's write under the first free name.
//! - `service_commit_args_editor_opening.rs`:
//!   whether a `commit` command line opens an editor.
//! - `service_commit_args_message.rs`:
//!   the commit message Git makes of the `-m` texts when no editor opens.
//! - `service_commit_args_fixed_message.rs`:
//!   the arguments of a commit Git built,
//!   rewritten so that the message the person already typed is used as it is and no editor opens.
//! - `service_commit_args.rs`:
//!   which commit command lines the commit service covers,
//!   and the message they stand for.
//! - `service_commit_saved_message.rs`:
//!   the file a typed message is kept in when the commit it belonged to cannot be made.
//! - `service_durable_syncs.rs`:
//!   the syncs one landing made,
//!   what each was for and how long it took,
//!   and the words its answer gives for them.
//! - `landing_state_carried_by_its_task.rs`:
//!   the state one landing keeps,
//!   carried by the task that runs the landing and not by a thread.
//! - `service_durable.rs`:
//!   where the landing's own syncs wait:
//!   queued for a background thread when the landing publishes,
//!   made before return otherwise.
//! - `service_head.rs`:
//!   whether a worktree's head still names the branch a commit was prepared on.
//! - `service_index_marker.rs`:
//!   the marker the service writes into the index lock and the questions asked of a marker.
//! - `service_index_ownership.rs`:
//!   whether the index lock is still the file the holder placed,
//!   and the record of a lock that could not be settled.
//! - `service_index_refused_lock.rs`:
//!   the foreign index lock that already outlasted a commit's wait,
//!   so the commits queued behind it are not made to wait again.
//! - `service_index_same.rs`:
//!   whether a new index file holds the live index's bytes.
//! - `service_index_hold.rs`:
//!   the service's hold on a placed index lock:
//!   the check that it is still the holder's,
//!   the release and the removal.
//! - `service_index_watch.rs`:
//!   the inotify watch the wait for the index lock blocks on.
//! - `service_land_batch_import_stream_and_overlaps.rs`:
//!   the `fast-import` stream that makes one commit per request of a batch,
//!   the marks it exports,
//!   and the tests of which requests of a batch touch the same paths.
//! - `service_land_branch_move_transaction.rs`:
//!   the two command lines a landing writes for its branch:
//!   the question that reads the commit it names and the transaction that moves it.
//! - `service_land_context.rs`:
//!   what the commit service needs to land a batch.
//! - `service_land_copy.rs`:
//!   what the commit service's landing does once the branch has moved:
//!   index entries and conclusion state.
//! - `service_land_finish_branch_questions.rs`:
//!   the answers service land finish branch questions holds.
//! - `service_land_follow_index_reset_git_questions.rs`:
//!   how one step of the index reset asks Git,
//!   on a thread and on the runtime,
//!   and what answer it stands for.
//! - `service_land_follow_index_reset_paths.rs`:
//!   the paths a reset installs, as the bytes `git reset` reads them:
//!   a pathspec that names one path from the top of the worktree,
//!   a NUL-separated list of them,
//!   and the names that differ between two commits.
//! - `service_land_follow_index_reset.rs`:
//!   the answers service land follow index reset holds.
//! - `service_land_follow.rs`:
//!   the answers service land follow holds.
//! - `service_land_reference_lock.rs`:
//!   what a landing does when its branch write is refused while the branch is still where the batch found it:
//!   the reference locks that stand,
//!   the bounded wait for a live holder,
//!   and the words for the locks nothing proves gone.
//! - `service_land_reflog.rs`:
//!   the reflog message the branch move writes:
//!   the commit's kind in Git's own words and the first line of its message.
//! - `service_land_start.rs`:
//!   the witness of the live index file as the landing found it under the index lock,
//!   recorded in the journal,
//!   and the comparison a replay makes with it.
//! - `owed_index_repair_the_entries_it_reads.rs`:
//!   what the repair reads about one path:
//!   the entries in which the commit's own index differs from the live index,
//!   and the object the worktree holds there.
//! - `owed_index_repair_the_commit_index_writes.rs`:
//!   the writes the repair makes to the commit's own index.
//! - `service_message_the_identity.rs`:
//!   the identities a commit the service lands records.
//! - `service_message_the_text_steps.rs`:
//!   the text steps of a commit's message in the order Git applies them.
//! - `service_message_what_a_commit_says.rs`:
//!   the message and subject of an existing commit,
//!   read for an amend or a fixup.
//! - `service_message.rs`:
//!   the message and identities of a service commit,
//!   made as native `git commit` makes them.
//! - `service_amend.rs`:
//!   what an amend takes from the commit it replaces:
//!   parents,
//!   message and author.
//! - `service_messages_of_a_failure.rs`:
//!   the word each failure of a commit goes by in the timing record,
//!   declared beside the renderings because they are at their line bound.
//! - `service_messages_of_a_manager_that_did_not_answer.rs`:
//!   the words for a manager that answered nothing in the time one exchange is given,
//!   which says nothing about whether it is there.
//! - `service_messages_of_a_platform.rs`:
//!   the sentence a platform without a user service manager leaves the person,
//!   used by every failure that needed one.
//! - `service_messages_of_a_service_that_stopped.rs`:
//!   the words for a commit whose service said nothing,
//!   stopped reading its request,
//!   or stopped answering it.
//! - `service_messages_of_a_start_that_cannot_come.rs`:
//!   the words for a start that cannot come at all,
//!   because the address cannot exist or the manager made the service listen elsewhere.
//! - `service_messages_of_a_start_that_was_asked_and_did_not_come.rs`:
//!   the words for a wait that ended without a greeting,
//!   built from the unit the manager was asked about,
//!   what it answered,
//!   and what this command asked it to do.
//! - `service_messages_of_a_wait_for_the_service.rs`:
//!   what a commit says while it waits for the service,
//!   and that the service is restarted or the commit built again.
//! - `service_messages_of_an_interrupted_commit.rs`:
//!   the words for a commit the person interrupted,
//!   which the service either gave up or left unsettled.
//! - `service_messages.rs`:
//!   what the user is told when the commit service cannot land a commit.
//! - `service_commit_tree_ownership.rs`:
//!   which end of a commit's scratch tree a submitted request leaves it to.
//! - `service_commit_waiting_notices.rs`:
//!   the submission of a prepared commit whose waiting notices are written away from the thread that waits.
//! - `service_messages_of_a_work_that_was_not_handed_over.rs`:
//!   the words for work a hand-off could not hand to the commit service,
//!   in the words of what stopped it.
//! - `service_native.rs`:
//!   the commits real Git builds for the service:
//!   an editor,
//!   a signature,
//!   and what the service does not embed.
//! - `service_object_questions_left_to_git.rs`:
//!   the object questions a declined in-process reader leaves to Git:
//!   a commit's own bytes and whether the objects a request names are gone.
//! - `service_built_checks.rs`:
//!   what a commit Git built adds to the branch,
//!   answered from the object store without starting Git.
//! - `service_prepare_selection_refusals.rs`:
//!   the findings a preparation ends with when the selection the command line makes cannot be committed.
//! - `service_prepare_unmerged_staging.rs`:
//!   the staging start that puts every unmerged file's working copy into the commit's own index.
//! - `service_prepare_own_git_steps.rs`:
//!   the two Git steps a service preparation starts itself,
//!   with the face a task of the runtime asks.
//! - `service_pruned_tree_closures.rs`:
//!   whether a tree, a tree below it or a blob in any of them is gone from the object store,
//!   for every root of a batch,
//!   and what that walk cost.
//! - `service_pruned.rs`:
//!   how a commit headed for the service notices that its prepared content was removed before it landed.
//! - `service_replay_conclusion.rs`:
//!   the refusal of a conclusion whose operation another command ended while the commit was prepared.
//! - `service_replay_line_text.rs`:
//!   the small text steps of the replay of captured changes.
//! - `service_retry.rs`:
//!   the repositories the service looks at again when a claim another instance holds may have run out.
//! - `service_route_what_the_status_answers.rs`:
//!   what the status of a worktree's service answers,
//!   and the words it answers with.
//! - `service_route.rs`:
//!   which way a commit goes:
//!   through the service,
//!   set up first,
//!   or the earlier path.
//! - `held_push_release_wakes_the_service.rs`:
//!   the wake a command gives the service when it releases a held push by itself.
//! - `service_commit_input_notes_and_embedded_commit.rs`:
//!   the small steps of a service commit and the commit the service builds itself.
//! - `service_rules_precompile.rs`:
//!   the service's warm of the compiled forbidden-strings rules,
//!   off the commit's path.
//! - `service_scratch_abandonment.rs`:
//!   which commit scratch trees a later command takes for abandoned,
//!   and which it leaves alone because their owner still runs.
//! - `service_scratch_ref_names_and_values.rs`:
//!   the names of a landing's side references,
//!   how a reference's value is read,
//!   the one write of a side reference,
//!   and the line that says one was left where it is.
//! - `service_scratch_ref_of_a_dropped_batch.rs`:
//!   the face a caller on a thread asks of the removal of a batch's scratch reference,
//!   once the commit it holds is kept.
//! - `held_commit_settle_move.rs`:
//!   the one branch write of a settlement,
//!   the reference a batch's commit is kept on until that write,
//!   and what the batch owes around it.
//! - `service_scratch_ref_of_a_dropped_batch_awaiting.rs`:
//!   the same removals as the face a task of the runtime awaits,
//!   every Git child and every look at an intent a job of the pool.
//! - `held_commit_words.rs`:
//!   what the wrapper says about a commit that a reference lock keeps off its branch.
//! - `held_commit_settle_outcomes.rs`:
//!   what became of one held commit when a settlement looked at it,
//!   and the journal line the service writes about it.
//! - `service_scratch_ref_marker_of_a_batch.rs`:
//!   the record that names the worktree of a batch's scratch reference,
//!   so a restart can remove a reference whose landing was killed before its intent.
//! - `service_scratch_renewal.rs`:
//!   the renewal that keeps a commit's scratch tree from aging out while the command that owns it lives.
//! - `service_prepare_ready_shapes.rs`:
//!   the shapes a service preparation hands back.
//! - `service_prepare_changes_config_and_pathspec_steps.rs`:
//!   the smaller steps of a service preparation.
//! - `service_replay_the_captured_change.rs`:
//!   what a first preparation captured,
//!   kept for the retry,
//!   and the records a replay speaks in.
//! - `service_replay_capture_order.rs`:
//!   the capture order of every shared path of a replay,
//!   from one walk over the history.
//! - `service_replay_the_word_pass.rs`:
//!   the word-level second pass over the files a line merge could not settle.
//! - `service_replay_the_merge_onto_the_branch.rs`:
//!   the tree the captured change makes,
//!   the merge of it onto the branch as it is now,
//!   and the commit that landed first on a path that still conflicts.
//! - `service_replay.rs`:
//!   the retry of a commit the service sent back:
//!   the captured change merged onto the branch,
//!   never prepared again from the worktree.
//! - `claim_time.rs`:
//!   the clocks a claim's lease is judged by.
//! - `held_push_holder_and_verdict.rs`:
//!   which command will decide a held commit's post-commit checks,
//!   and the verdict that command left.
//! - `service_amend_replayed_the_commit_made_again.rs`:
//!   the commit an amend makes on the branch as it is now:
//!   where it is written,
//!   how it is signed,
//!   and the request that lands it.
//! - `service_amend_replayed.rs`:
//!   the commit an amend makes when the branch moved under it:
//!   the amended commit's own author,
//!   committer,
//!   message and signature,
//!   on the replayed tree,
//!   on top of the branch as it is now.
//! - `held_commit_replay.rs`:
//!   the change of a held commit put onto the branch as it is now,
//!   when that branch moved while the lock stood.
//! - `service_commit_line_reissued.rs`:
//!   the command line a commit is built again with once its editor has run and the branch moved.
//! - `service_index_identity.rs`:
//!   who a marker's holder is:
//!   the identity a process writes,
//!   and the answers a reader gives (gone,
//!   running,
//!   unknown).
//! - `service_index_found.rs`:
//!   what an acquisition of the index lock found when it had to wait,
//!   for the service's log.
//! - `service_index_placement.rs`:
//!   how the service's marked file becomes the index lock,
//!   and the cleanup of staging files.
//! - `service_index_staging.rs`:
//!   the staging files of the index lock.
//! - `service_index_holder_look.rs`:
//!   what one look at the held index lock proves,
//!   and the words the person is told about its holder.
//! - `service_index_claim_files.rs`:
//!   what the file at one slot of the claim chain is,
//!   read without following a link or waiting on a pipe.
//! - `service_index_claim.rs`:
//!   the claim chain that keeps two reclaimers apart where the file system gives no `flock`.
//! - `service_index_reclaim.rs`:
//!   the removal of a stale index lock,
//!   by a rename and a second look,
//!   and the pass that reclaims one whose holder is gone.
//! - `service_index_unheld_removal.rs`:
//!   the removal of an index lock that no process holds and no Git lock PID file stands behind,
//!   on a proof written to the service journal.
//! - `service_index_attempts.rs`:
//!   the attempts and the wait of an acquisition of the index lock.
//! - `service_index_prestage.rs`:
//!   the marker of the next batch's index lock,
//!   made and flushed while the service is idle.
//! - `service_index_lock.rs`:
//!   the commit service's hold on Git's index lock across a landing.
//! - `service_index_wait.rs`:
//!   the wait of a wrapped index writer for the commit service's own hold on the index.
//! - `service_hold_retry.rs`:
//!   a forwarded index writer started again when the commit service took the index before its Git did.
//! - `service_land_outcomes.rs`:
//!   what became of one request of a batch,
//!   and the announcement a landing makes when none of its clients is told anything.
//! - `service_land_requests_whose_client_is_gone.rs`:
//!   which requests of a batch a landing still makes,
//!   given which of their clients went away before their commits landed.
//! - `the_answer_of_one_git_step_of_a_side_reference.rs`:
//!   what became of one Git step of a landing's own side reference,
//!   and the sentence a step that a stop or a ceiling ended keeps for the landing that answers.
//! - `the_write_of_a_side_reference.rs`:
//!   the write itself:
//!   made once more only when the wait for it ended without an answer,
//!   never replacing a reference another command wrote,
//!   answered by the reference's own value,
//!   and the words a person reads when Git refused it.
//! - `the_side_references_a_commit_is_kept_on.rs`:
//!   the write named for each reference of the wrapper's own:
//!   the one a commit that did not land is kept on,
//!   the shared one of a batch,
//!   and a named side reference.
//! - `the_commit_a_side_reference_did_not_keep.rs`:
//!   the commit a write did not prove:
//!   the reference it is kept on,
//!   the sentence the person reads,
//!   letting go of the batch's own reference,
//!   and the tip a batch built that its reference did not hold.
//! - `service_land_side_reference_write.rs`:
//!   the face a landing's code calls for its own side references,
//!   re-exporting the four files beside it under the names callers use.
//! - `service_land_git_steps_and_import_stream.rs`:
//!   the Git steps of a landing,
//!   the import stream of a batch and the overlap tests of its requests.
//! - `service_land_hold_a_batch.rs`:
//!   what a landing does when a reference lock keeps its branch where it is,
//!   and what it gives back when a batch will never land.
//! - `service_land_kept_commit.rs`:
//!   the commit of a batch a recovery drops,
//!   kept on `refs/cli-git/kept/<number>`.
//! - `service_land_record_the_batch.rs`:
//!   the record a landing puts in the journal before it moves a branch,
//!   and the reflog row that move writes.
//! - `service_land_take_index_lock.rs`:
//!   takes the index lock for a batch that is ready to land,
//!   notes the wait in the timing and the log,
//!   starts making the markers of the two batches after it,
//!   and cleans up when the lock is refused.
//! - `service_land_without_its_scratch_reference.rs`:
//!   the batch whose own scratch reference the write did not prove:
//!   it lands as real Git lands a commit,
//!   and the person is told once that the extra protection was not in place.
//! - `service_work_branch_questions.rs`:
//!   whether a branch exists and the commit it names,
//!   which the service's push asks before it sends anything.
//! - `service_work_common_directory.rs`:
//!   the common Git directory a linked worktree's Git directory names,
//!   and whether it belongs to a repository with no main worktree.
//! - `service_work_push_environment.rs`:
//!   the environment every Git child of a service push receives,
//!   and where its program search list comes from.
//! - `side_reference_written_here.rs`:
//!   the reference that keeps a batch's commits reachable until its branch moves,
//!   written by the landing itself where that write is provably Git's,
//!   and left to Git everywhere else.
//! - `service_land_settings.rs`:
//!   takes the index lock for a batch that is ready to land.
//!   What became of one request of a batch,
//!   and the announcement a landing makes when nobody is told.
//!   What a landing starts with:
//!   how many times its batch may be built again,
//!   and what it reads once.
//! - `service_land_objects_of_batch.rs`:
//!   the objects a batch lands:
//!   the commit Git built,
//!   or the commits made from the requests' changes.
//! - `unreachable_git_dir_proven_gone.rs`:
//!   what the filesystem proves about a Git directory the service can no longer resolve.
//! - `work_item_kinds_and_their_names.rs`:
//!   the kinds of work a landed object can ask for,
//!   and the names that pick a consumer's handler.
//! - `work_item_ids_and_names.rs`:
//!   the digest a push's id carries for its branch,
//!   and what may follow the kind in an id.
//! - `work_item_state_directories_and_claim_names.rs`:
//!   the state folders of the work queue,
//!   the names of claim files,
//!   and the listing of a folder oldest first.
//! - `work_item_record.rs`:
//!   the durable work-item record of asynchronous work (review,
//!   push,
//!   maintenance) left for a resident consumer:
//!   its fields,
//!   its one JSON line and the reader of that line.
//! - `the_record_of_an_owed_index_update.rs`:
//!   the index update one landed batch still owes:
//!   its shape,
//!   how a batch's intent becomes one,
//!   and the records a Git directory holds.
//! - `owed_index_repair.rs`:
//!   the repair that repeats a landed batch's owed index update for the entries the landing left.
//! - `work_item_claim_taking.rs`:
//!   the steps that take one pending record out of the queue,
//!   and the stamp that keeps a claim's lease readable.
//! - `work_item_settling.rs`:
//!   moving a held item's claim to its state,
//!   starting the claim's lease again,
//!   and the age of the oldest pending item.
//! - `work_item_holding_the_verdicts_and_the_promotion.rs`:
//!   the verdicts written into a held record,
//!   the promotion that makes the record runnable,
//!   and the two ways a record leaves the queue.
//! - `work_item_enqueue.rs`:
//!   writing one producer's record,
//!   or a round of them,
//!   to the queue,
//!   and finding what is already there.
//! - `work_item_holding.rs`:
//!   the steps of a work item that waits for its producer's approval.
//! - `work_item_queued_ids.rs`:
//!   the ids the queue still owes work for,
//!   asked by a producer that writes a record and by a consumer that takes a name away.
//! - `work_item.rs`:
//!   the one name a caller uses for the record,
//!   the steps that take and settle it,
//!   and the files they are declared beside.
//! - `owed_index_update.rs`:
//!   the record of an index update a landed batch owes.
//! - `service_land_finish_after_the_move.rs`:
//!   the conclusion state and the index update that follow a batch's branch move,
//!   recorded so that a restart never replays the index over what another program staged.
//! - `service_land_finish_replay_gate.rs`:
//!   whether a restart may replay an open batch's index update,
//!   and the two words a settlement comes to.
//! - `held_commit_settle.rs`:
//!   moving the branch of a held commit once nothing stands in the way,
//!   and finishing what the landing owes after the move.
//! - `service_claim_renewal.rs`:
//!   the renewal that keeps the claims of a shared work directory from looking abandoned while their holder works.
//! - `service_claim_renewal_running.rs`:
//!   the task of a running claim renewal,
//!   its driver for controls,
//!   and its stop.
//! - `service_land_finish.rs`:
//!   the batches a restart finishes or drops,
//!   and what it reports about each.
//! - `service_land_attempt_values.rs`:
//!   the values one attempt of a landing carries from step to step,
//!   and the checks that a batch may land at all.
//! - `service_land_attempt_reads.rs`:
//!   what one attempt reads before it writes anything:
//!   the commit the branch names,
//!   the parents an amend replaces,
//!   the places that still want their commit,
//!   and whether their content is still there.
//! - `service_land_attempt_batch_built.rs`:
//!   the commits of one attempt's batch,
//!   the commit its branch is about to name,
//!   the row its move writes in the reflog,
//!   and the record the journal holds of it.
//! - `service_land_attempt_index_lock.rs`:
//!   the index lock of one attempt,
//!   the checked-out branch read again under it,
//!   and the places whose client went away while the lock was awaited.
//! - `service_land_attempt_owed_and_witness.rs`:
//!   what one attempt's batch owes,
//!   and the index file the landing saw,
//!   recorded under the lock before the branch moves.
//! - `service_land_attempt_branch_move.rs`:
//!   the one branch write of an attempt:
//!   the scratch reference's removal,
//!   the transaction,
//!   and the words that say what the write is doing.
//! - `service_land_attempt_refused_move.rs`:
//!   what an attempt does when its branch write did not happen:
//!   wait out the lock that refused it,
//!   hold the commit behind that lock,
//!   or build the batch again because the branch moved.
//! - `service_land_attempt_after_the_move.rs`:
//!   the head read again,
//!   the index installed,
//!   and the answer every landed place gets.
//! - `service_land_batch_body.rs`:
//!   the landing itself:
//!   one batch written,
//!   journaled and moved onto a branch.
//! - `service_land.rs`:
//!   the commit service's serialized step.
//! - `earlier_service_journal_adopted.rs`:
//!   the batches the commit service of an earlier release left open,
//!   finished by the service of this release when none runs.
//! - `service_prepare_what_it_reads.rs`:
//!   what a preparation reads once before it stages anything:
//!   the layout,
//!   the selection,
//!   the capture,
//!   the scratch directory,
//!   and the two index files.
//! - `service_prepare_the_commit_index.rs`:
//!   the commit's own index,
//!   from the change a retry captured or from the working copy the command line selects.
//! - `service_prepare_the_ready_shape.rs`:
//!   the shape every ending of a preparation shares,
//!   and the conclusion state a retry is compared against.
//! - `service_prepare_the_policies.rs`:
//!   the policies a preparation runs over the commit's own index.
//! - `service_prepare.rs`:
//!   the answers service prepare holds.
//! - `service_branch_shows.rs`:
//!   what the branch shows once the commit service went silent after a commit was prepared.
//! - `service_commit_looking_for_what_landed.rs`:
//!   the look for the commit a request that was already sent made,
//!   once its answer was lost.
//! - `service_commit_rebuild_ceiling.rs`:
//!   how long a commit whose branch moved under its build is built again,
//!   and the line that says so.
//! - `service_commit_after_a_moved_branch.rs`:
//!   what a commit does when the service answers that the branch moved under it.
//! - `service_commit_requests_and_landing_report.rs`:
//!   the requests of a service commit and what a landed commit leaves for the person.
//! - `service_commit_fitting_request.rs`:
//!   whether the request the embedded route would send fits the one line the service reads.
//! - `service_commit_request_of.rs`:
//!   the request one attempt of a commit sends:
//!   the amended commit landing as a new one,
//!   the commit Git builds,
//!   or the commit the service builds from captured changes.
//! - `service_work_branch_is_gone.rs`:
//!   the notice for a push whose branch the person deleted.
//! - `work_item_recovery.rs`:
//!   the answers work item recovery holds.
//! - `service_work_kinds_and_a_wake.rs`:
//!   the kinds of work the service runs,
//!   how many times an item is tried,
//!   and what a wake message brings.
//! - `service_work_handlers_and_verdicts.rs`:
//!   what a handler came to,
//!   and one method per kind of work with its awaiting twin.
//! - `service_work_acceptable_records_and_claims.rs`:
//!   the work directory,
//!   which records are acceptable work,
//!   and what becomes of the claims this instance holds.
//! - `service_work_given_up_notices.rs`:
//!   the words a person reads when the service gave up on a push or a relink.
//! - `service_work_drained_and_replanning.rs`:
//!   what one drain hands back,
//!   and the retry state it works with.
//! - `service_work_planned_retry.rs`:
//!   the words and the plan of a push that is tried again.
//! - `service_work_unreachable_repository_plans.rs`:
//!   the plans of a repository that is gone or cannot be reached.
//! - `service_work_handlers_replanning_and_acceptance.rs`:
//!   the one name a caller uses for all of the files listed before it here.
//! - `service_work_git_handlers.rs`:
//!   the handlers that run real Git:
//!   the settings a round copies,
//!   the Git one push runs,
//!   and the review a landed commit is given.
//! - `service_work_pushes.rs`:
//!   the push of one branch:
//!   the commit it may send,
//!   the records that hold it back,
//!   and the words each answer comes to.
//! - `service_work_worktree_upkeep.rs`:
//!   the worktree work a landed commit leaves:
//!   trash,
//!   Cargo build directories,
//!   a removal's subvolume,
//!   a snapshot ahead,
//!   and the automatic upkeep.
//! - `service_work_drain_waves_of_one_kind.rs`:
//!   the bodies a wave of one kind asks of its handler,
//!   and the line one answer that is done writes.
//! - `service_work_storage_step_awaited.rs`:
//!   how one storage step of a drain runs as a job of the blocking pool,
//!   and the words a step that got no place is refused with.
//! - `service_work_storage_claims_awaited.rs`:
//!   the storage steps that touch the queue's own records,
//!   from the claim of a directory to the claims another instance let run out.
//! - `service_work_storage_awaited.rs`:
//!   the storage steps of one drain that leave a push's record for the person,
//!   each a job of the blocking pool awaited by a task.
//! - `service_work_drains_a_snapshot_ahead.rs`:
//!   the drain of the snapshots the commit service makes ahead of the next add,
//!   as one step of a round.
//! - `service_work_settling_with_words.rs`:
//!   how one item whose kind answers with words is settled,
//!   with the line the person reads for it.
//! - `service_work_items_side_by_side.rs`:
//!   which items of one batch of that drain run at once and how many of them.
//! - `service_work_settling_a_verdict.rs`:
//!   what a drained push's verdict does to its records,
//!   its retry plan and the person.
//! - `service_work_drain_pushes_of_each_branch.rs`:
//!   the push step of one round:
//!   the pushes grouped by branch,
//!   the branches run beside each other,
//!   and each settled as its own answer comes.
//! - `service_work_drain_deletes_subvolumes.rs`:
//!   the drain's step that deletes the subvolumes a removal handed over.
//! - `service_work_drain_claims_by_kind.rs`:
//!   the claimed items of one batch of the drain,
//!   sorted into one list per kind of work.
//! - `service_work_drain_housekeeping_and_upkeep.rs`:
//!   the housekeeping one round of the drain may start,
//!   the upkeep it runs on the items it holds,
//!   and what it leaves for a later round.
//! - `service_work_drain_look_again.rs`:
//!   when the caller of one drain of a work directory must look at it again.
//! - `service_work_drains_awaiting_the_due_push_retries.rs`:
//!   the pushes a round runs again because their retry is due,
//!   and the programs a record's end gives up.
//! - `service_work_drain_pending_claims.rs`:
//!   drains the pending items of one work directory.
//! - `service_work_drains_awaiting.rs`:
//!   the drains of one directory and of a common directory,
//!   awaited by a task of the runtime.
//! - `service_work.rs`:
//!   the service's consumer of after-commit push and upkeep work.
//! - `service_claim_watch_watched_repositories.rs`:
//!   what the notes of one claim watch say about the repositories they name,
//!   read without keeping anything.
//! - `service_claim_watch_reading_the_store.rs`:
//!   the reading of the claim watch's store:
//!   which notes are kept,
//!   which are removed as certainly of no use,
//!   and what could not be read at all.
//! - `service_claim_watch.rs`:
//!   the Git directories the service looks at by itself when it starts.
//! - `service_claim_watch_note_within_a_ceiling.rs`:
//!   the wait a landing gives the note that names its repository to the next service.
//! - `service_handoff_queued_push_words.rs`:
//!   the words a waiting push record is named with:
//!   the bound a running service looks in and how long the record has waited.
//! - `service_handoff_push_notices.rs`:
//!   what a command says about the pushes this worktree holds and the pushes it queued.
//! - `service_handoff.rs`:
//!   the wrapper's side of after-commit work:
//!   records and the wake.
//! - `service_landed_aftermath.rs`:
//!   what a landed commit leaves the command to do:
//!   its result and the work after it.
//! - `service_commit_after_a_failure.rs`:
//!   what a commit does when the service answered without landing it:
//!   the words for content a prune removed,
//!   and the aftermath of a service that failed or went silent.
//! - `service_commit.rs`:
//!   the commit command's path through the commit service.
//! - `service_round.rs`:
//!   the drain of several repositories of one round,
//!   side by side and without a lock.
//! - `service_work_run_round.rs`:
//!   runs one round of the service's work over several directories.
//! - `service_work_slow_lane.rs`:
//!   the service's work that can take minutes:
//!   a lane of its own,
//!   beside the round that serves the person's pushes.
//! - `push_record_of_a_moved_branch.rs`:
//!   the push record a command that moves a branch leaves,
//!   and the hand-off that wakes the service to run it with that command's programs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/index.ts: one `export * from './file'` per module below.
//! ```

/// Every failure the client can meet,
/// and the facts its text names.
pub mod service_messages_the_failures_a_person_can_meet;

/// The commands and words a failure hands a person to act on.
pub mod service_messages_what_a_failure_gives_a_person;

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

/// The durable work-item record of asynchronous work (review,
/// push,
/// maintenance) left for a resident consumer:
/// its fields,
/// its one JSON line and the reader of that line.
pub mod work_item_record;

/// The steps that take one pending record out of the queue,
/// and the stamp that keeps a claim's lease readable.
pub mod work_item_claim_taking;

/// Moving a held item's claim to its state,
/// starting the claim's lease again,
/// and the age of the oldest pending item.
pub mod work_item_settling;

/// The one name a caller uses for the record,
/// the steps that take and settle it,
/// and the files they are declared beside.
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

/// Writing one producer's record,
/// or a round of them,
/// to the queue,
/// and finding what is already there.
pub mod work_item_enqueue;

/// The verdicts written into a held record,
/// the promotion that makes the record runnable,
/// and the two ways a record leaves the queue.
pub mod work_item_holding_the_verdicts_and_the_promotion;

/// The steps of a work item that waits for its producer's approval.
pub mod work_item_holding;

/// The ids the queue still owes work for,
/// asked by a producer that writes a record and by a consumer that takes a name away.
pub mod work_item_queued_ids;

/// The clocks a claim's lease is judged by.
pub mod claim_time;

/// The repositories the service looks at again when a claim another instance holds may have run out.
pub mod service_retry;

/// The bytes of one note the claim watch keeps:
/// the format words,
/// the bound on its length,
/// and the spelling of a path in hexadecimal.
pub mod service_claim_watch_note_format;

/// What the notes of one claim watch say about the repositories they name,
/// read without keeping anything.
pub mod service_claim_watch_watched_repositories;

/// Writing a note:
/// the landing's own write,
/// and the store's write under the first free name.
pub mod service_claim_watch_writing_a_note;

/// The reading of the claim watch's store:
/// which notes are kept,
/// which are removed as certainly of no use,
/// and what could not be read at all.
pub mod service_claim_watch_reading_the_store;

/// The Git directories the service looks at by itself when it starts.
pub mod service_claim_watch;

/// The wait a landing gives the note that names its repository to the next service.
pub mod service_claim_watch_note_within_a_ceiling;

/// Whether a Git directory is the repository of the instance whose journal a note is written into.
pub mod service_claim_watch_whose_repository_it_is;

/// What the user is told when the commit service cannot land a commit.
pub mod service_messages;

/// The word each failure of a commit goes by in the timing record,
/// declared beside the renderings because they are at their line bound.
pub mod service_messages_of_a_failure;

/// The words for a start that cannot come at all,
/// because the address cannot exist or the manager made the service listen elsewhere.
pub mod service_messages_of_a_start_that_cannot_come;

/// The words for a manager that answered nothing in the time one exchange is given,
/// which says nothing about whether it is there.
pub mod service_messages_of_a_manager_that_did_not_answer;

/// The words for a wait that ended without a greeting,
/// built from the unit the manager was asked about,
/// what it answered,
/// and what this command asked it to do.
pub mod service_messages_of_a_start_that_was_asked_and_did_not_come;

/// The words for work a hand-off could not hand to the commit service,
/// in the words of what stopped it.
pub mod service_messages_of_a_work_that_was_not_handed_over;

/// The sentence a platform without a user service manager leaves the person,
/// used by every failure that needed one.
pub mod service_messages_of_a_platform;

/// The words for a commit the person interrupted,
/// which the service either gave up or left unsettled.
pub mod service_messages_of_an_interrupted_commit;

/// What a commit says while it waits for the service,
/// and that the service is restarted or the commit built again.
pub mod service_messages_of_a_wait_for_the_service;

/// The words for a commit whose service said nothing,
/// stopped reading its request,
/// or stopped answering it.
pub mod service_messages_of_a_service_that_stopped;

/// The one branch write of a settlement,
/// and what the batch owes around it.
pub mod held_commit_settle_move;

/// The record a landing puts in the journal before it moves a branch,
/// and the reflog row that move writes.
pub mod service_land_record_the_batch;

/// Which requests of a batch a landing still makes,
/// given which of their clients went away before their commits landed.
pub mod service_land_requests_whose_client_is_gone;

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

/// The questions about the file system that the look at an earlier commit service asks:
/// the bytes that spell a path,
/// the part of a path the file system knows,
/// and the identity of each earlier journal that exists.
pub mod earlier_service_look_paths;

/// What the look at an earlier commit service answers with,
/// and the words of a service's start it reads.
pub mod earlier_service_process_look_answers;

/// Which earlier commit service one command line of a process names.
pub mod earlier_service_process_start_arguments;

/// The look at the running processes and the sockets that tells whether a commit service of an earlier release may
/// still run or may start for a worktree.
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

/// The values one attempt of a landing carries from step to step,
/// and the checks that a batch may land at all.
pub mod service_land_attempt_values;

/// What one attempt of a landing reads before it writes anything.
pub mod service_land_attempt_reads;

/// The commits of one attempt's batch and the record the journal holds of it.
pub mod service_land_attempt_batch_built;

/// The index lock of one attempt,
/// and the two questions asked with it held.
pub mod service_land_attempt_index_lock;

/// What one attempt's batch owes,
/// and the index file the landing saw.
pub mod service_land_attempt_owed_and_witness;

/// The one branch write of an attempt.
pub mod service_land_attempt_branch_move;

/// What an attempt does when its branch write did not happen.
pub mod service_land_attempt_refused_move;

/// The head read again after the move,
/// the index installed,
/// and the answer.
pub mod service_land_attempt_after_the_move;

/// The landing itself:
/// one batch written,
/// journaled and moved onto a branch.
pub mod service_land_batch_body;

/// The reflog message the branch move writes:
/// the commit's kind in Git's own words and the first line of its message.
pub mod service_land_reflog;

pub mod service_land_outcomes;

pub mod service_land_take_index_lock;

/// The two command lines a landing writes for its branch:
/// the question that reads the commit it names and the transaction that moves it.
pub mod service_land_branch_move_transaction;

/// The Git steps of a landing,
/// the import stream of a batch and the overlap tests of its requests.
pub mod service_land_git_steps_and_import_stream;

/// What became of one Git step of a landing's own side reference,
/// and the sentence a step that a stop or a ceiling ended keeps.
pub mod the_answer_of_one_git_step_of_a_side_reference;

/// The write of one of a landing's own side references,
/// and the words a person reads when Git refused it.
pub mod the_write_of_a_side_reference;

/// The write named for each reference of the wrapper's own.
pub mod the_side_references_a_commit_is_kept_on;

/// The commit a landing's own side reference did not prove,
/// and the tip a batch built that it did not hold.
pub mod the_commit_a_side_reference_did_not_keep;

/// The face a landing's code calls for its own side references,
/// re-exporting the four files beside it under the names callers use.
pub mod service_land_side_reference_write;

/// The batch whose own scratch reference the write did not prove:
/// it lands as real Git lands a commit,
/// and the person is told once that the extra protection was not in place.
pub mod service_land_without_its_scratch_reference;

/// The objects a batch lands:
/// the commit Git built,
/// or the commits made from the requests' changes.
pub mod service_land_objects_of_batch;

/// The reference that keeps a batch's commits reachable until its branch moves,
/// written by the landing itself where that write is provably Git's,
/// and left to Git everywhere else.
pub mod side_reference_written_here;

/// What the commit service's landing does once the branch has moved:
/// index entries and conclusion state.
pub mod service_land_copy;

pub mod service_land_follow;

/// How one step of the index reset asks Git,
/// on a thread and on the runtime,
/// and what answer it stands for.
pub mod service_land_follow_index_reset_git_questions;

/// The paths a reset installs, as the bytes `git reset` reads them:
/// a pathspec that names one path from the top of the worktree,
/// a NUL-separated list of them,
/// and the names that differ between two commits.
pub mod service_land_follow_index_reset_paths;

/// What a landing does when its branch write is refused while the branch is still where the batch found it:
/// the reference locks that stand,
/// the bounded wait for a live holder,
/// and the words for the locks nothing proves gone.
pub mod service_land_reference_lock;

/// The copy the service keeps of the conclusion state a landed commit removes.
pub mod conclusion_state_copy;

/// The index update one landed batch still owes:
/// its shape,
/// and how a batch's intent becomes one.
pub mod the_record_of_an_owed_index_update;

/// The record of an index update a landed batch owes.
pub mod owed_index_update;

/// What the repair reads about one path:
/// the entries in which the commit's own index differs from the live index,
/// and the object the worktree holds there.
pub mod owed_index_repair_the_entries_it_reads;

/// The writes the repair makes to the commit's own index.
pub mod owed_index_repair_the_commit_index_writes;

/// The repair that repeats a landed batch's owed index update for the entries the landing left.
pub mod owed_index_repair;

/// How the repair reads one entry of a tree or an index,
/// and the listings it compares.
pub mod owed_index_entries;

pub mod service_land_start;

/// What one look at the held index lock proves,
/// and the words the person is told about its holder.
pub mod service_index_holder_look;

/// The clause a sentence about a commit ends with when no reference is proven to name it.
pub mod the_clause_a_commit_no_reference_proves;

/// The words and the names the index lock uses:
/// the path of a lock,
/// how often a holder's line is repeated,
/// what the wait tells the clients,
/// why a wait gave up,
/// and the code a storage error stands for.
/// The bytes one repository's drain holds at its peak.
pub mod the_memory_one_round_keeps;

pub mod the_names_of_the_index_lock;

/// The attempts and the wait of an acquisition of the index lock.
pub mod service_index_attempts;

/// The commit service's hold on Git's index lock across a landing.
pub mod service_index_lock;

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
/// The marker this service keeps made ahead:
/// its slot,
/// the record of every file made,
/// and the step that makes and records one.
pub mod service_index_prestage_the_markers_made_ahead;

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

/// What a preparation reads once before it stages anything:
/// the layout,
/// the selection,
/// the capture,
/// the scratch directory,
/// and the two index files.
pub mod service_prepare_what_it_reads;

/// The commit's own index,
/// from the change a retry captured or from the working copy the command line selects.
pub mod service_prepare_the_commit_index;

/// The shape every ending of a preparation shares,
/// and the conclusion state a retry is compared against.
pub mod service_prepare_the_ready_shape;

/// The policies a preparation runs over the commit's own index.
pub mod service_prepare_the_policies;

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

/// The two Git steps a service preparation starts itself,
/// with the face a task of the runtime asks.
pub mod service_prepare_own_git_steps;

/// The smaller steps of a service preparation.
pub mod service_prepare_changes_config_and_pathspec_steps;

/// The commit message Git makes of the `-m` texts when no editor opens.
pub mod service_commit_args_message;

/// The arguments of a commit Git built,
/// rewritten so that the message the person already typed is used as it is and no editor opens.
pub mod service_commit_args_fixed_message;

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

/// What a first preparation captured,
/// kept for the retry,
/// and the records a replay speaks in.
pub mod service_replay_the_captured_change;

/// The capture order of every shared path of a replay,
/// from one walk over the history.
pub mod service_replay_capture_order;

/// The word-level second pass over the files a line merge could not settle.
pub mod service_replay_the_word_pass;

/// The tree the captured change makes,
/// the merge of it onto the branch as it is now,
/// and the commit that landed first on a path that still conflicts.
pub mod service_replay_the_merge_onto_the_branch;

/// The retry of a commit the service sent back:
/// the captured change merged onto the branch,
/// never prepared again from the worktree.
pub mod service_replay;

/// The small text steps of the replay of captured changes.
pub mod service_replay_line_text;

/// Which way a commit goes:
/// through the service,
/// set up first,
/// or the earlier path.
/// What the status of a worktree's service answers,
/// and the words it answers with.
pub mod service_route_what_the_status_answers;

pub mod service_route;

/// The message and identities of a service commit,
/// made as native `git commit` makes them.
/// The identities a commit the service lands records.
pub mod service_message_the_identity;

/// The message and subject of an existing commit,
/// read for an amend or a fixup.
pub mod service_message_what_a_commit_says;

/// The text steps of a commit's message in the order Git applies them.
pub mod service_message_the_text_steps;

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
/// The commit an amend makes on the branch as it is now,
/// its fields,
/// and the request that lands it.
pub mod service_amend_replayed_the_commit_made_again;

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

/// The handlers that run real Git:
/// the settings a round copies,
/// the Git one push runs,
/// and the review a landed commit is given.
pub mod service_work_git_handlers;

/// The push of one branch:
/// the commit it may send,
/// the records that hold it back,
/// and the words each answer comes to.
pub mod service_work_pushes;

/// The worktree work a landed commit leaves:
/// trash,
/// Cargo build directories,
/// a removal's subvolume,
/// a snapshot ahead,
/// and the automatic upkeep.
pub mod service_work_worktree_upkeep;

/// The notice for a push whose branch the person deleted.
pub mod service_work_branch_is_gone;

/// Whether a branch exists and the commit it names,
/// which the service's push asks before it sends anything.
pub mod service_work_branch_questions;

/// The common Git directory a linked worktree's Git directory names,
/// and whether it belongs to a repository with no main worktree.
pub mod service_work_common_directory;

/// The kinds of work the service runs,
/// how many times an item is tried,
/// and what a wake message brings.
pub mod service_work_kinds_and_a_wake;

/// What a handler came to,
/// and what runs each kind of work.
pub mod service_work_handlers_and_verdicts;

/// The work directory,
/// which records are acceptable work,
/// and what becomes of the claims this instance holds.
pub mod service_work_acceptable_records_and_claims;

/// The words a person reads when the service gave up on a push or a relink.
pub mod service_work_given_up_notices;

/// What one drain hands back,
/// and the retry state it works with.
pub mod service_work_drained_and_replanning;

/// The words and the plan of a push that is tried again.
pub mod service_work_planned_retry;

/// The plans of a repository that is gone or cannot be reached.
pub mod service_work_unreachable_repository_plans;

/// The one name a caller uses for the files listed before it here:
/// the kinds,
/// handlers,
/// acceptance rules,
/// settling and replanning of the service's work.
pub mod service_work_handlers_replanning_and_acceptance;

/// The bodies a wave of one kind asks of its handler,
/// and the line one answer that is done writes.
pub mod service_work_drain_waves_of_one_kind;

/// The push step of one round:
/// the pushes grouped by branch,
/// the branches run beside each other,
/// and each settled as its own answer comes.
pub mod service_work_drain_pushes_of_each_branch;

/// The claimed items of one batch of the drain,
/// sorted into one list per kind of work.
pub mod service_work_drain_claims_by_kind;

/// The housekeeping one round of the drain may start,
/// the upkeep it runs on the items it holds,
/// and what it leaves for a later round.
pub mod service_work_drain_housekeeping_and_upkeep;

/// When the caller of one drain of a work directory must look at it again.
pub mod service_work_drain_look_again;

/// Drains the pending items of one work directory.
pub mod service_work_drain_pending_claims;

/// How long one round may go on starting housekeeping,
/// and what it leaves for a later round.
pub mod housekeeping_of_one_round;

/// Which items of one batch of that drain run at once and how many of them.
pub mod service_work_items_side_by_side;

/// The drain's step that deletes the subvolumes a removal handed over.
pub mod service_work_drain_deletes_subvolumes;

/// What a drained push's verdict does to its records,
/// its retry plan and the person.
pub mod service_work_settling_a_verdict;

/// The environment every Git child of a service push receives,
/// and where its program search list comes from.
pub mod service_work_push_environment;

/// How one item whose kind answers with words is settled,
/// with the line the person reads for it.
pub mod service_work_settling_with_words;

/// The drain of the snapshots the commit service makes ahead of the next add,
/// as one step of a round.
pub mod service_work_drains_a_snapshot_ahead;

/// How one storage step of a drain runs as a job of the blocking pool,
/// and the words a step that got no place is refused with.
pub mod service_work_storage_step_awaited;

/// The storage steps that touch the queue's own records,
/// from the claim of a directory to the claims another instance let run out.
pub mod service_work_storage_claims_awaited;

/// The storage steps of one drain that leave a push's record for the person,
/// each a job of the blocking pool awaited by a task.
pub mod service_work_storage_awaited;

/// The drains of one directory and of a common directory,
/// awaited by a task of the runtime.
/// The pushes a round runs again because their retry is due,
/// and the programs a record's end gives up.
pub mod service_work_drains_awaiting_the_due_push_retries;

pub mod service_work_drains_awaiting;

/// The service's work that can take minutes:
/// a lane of its own,
/// beside the round that serves the person's pushes.
pub mod service_work_slow_lane;

/// Runs one round of the service's work over several directories.
pub mod service_work_run_round;

/// The drain of several repositories of one round,
/// side by side and without a lock.
pub mod service_round;

/// The push record a command that moves a branch leaves,
/// and the hand-off that wakes the service to run it with that command's programs.
pub mod push_record_of_a_moved_branch;

/// What a command says about the pushes this worktree holds and the pushes it queued.
pub mod service_handoff_push_notices;

/// The wrapper's side of after-commit work:
/// records and the wake.
pub mod service_handoff;

/// The words a waiting push record is named with:
/// the bound a running service looks in and how long the record has waited.
pub mod service_handoff_queued_push_words;

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

/// Whether a tree, a tree below it or a blob in any of them is gone from the object store,
/// for every root of a batch,
/// and what that walk cost.
pub mod service_pruned_tree_closures;

/// How a commit headed for the service notices that its prepared content was removed before it landed.
pub mod service_pruned;

/// The object questions a declined in-process reader leaves to Git:
/// a commit's own bytes and whether the objects a request names are gone.
pub mod service_object_questions_left_to_git;

/// Where the landing's own syncs wait:
/// queued for a background thread when the landing publishes,
/// made before return otherwise.
/// The task that makes a landing's syncs after the landing answered,
/// and the wait a control makes until its queue is empty.
pub mod service_durable_the_background_sync_thread;

pub mod service_durable;

/// The syncs one landing made,
/// what each was for and how long it took,
/// and the words its answer gives for them.
pub mod service_durable_syncs;

/// The conclusion state and the index update that follow a batch's branch move,
/// recorded so that a restart never replays the index over what another program staged.
pub mod service_land_finish_after_the_move;

/// Whether a restart may replay an open batch's index update,
/// and the two words a settlement comes to.
pub mod service_land_finish_replay_gate;

/// The batches a restart finishes or drops,
/// and what it reports about each.
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

/// What the file at one slot of the claim chain is,
/// read without following a link or waiting on a pipe.
pub mod service_index_claim_files;

/// The claim chain that keeps two reclaimers apart where the file system gives no `flock`.
pub mod service_index_claim;

/// Whether a new index file holds the live index's bytes.
pub mod service_index_same;

/// The names of a landing's side references,
/// how a reference's value is read,
/// the one write of a side reference,
/// and the line that says one was left where it is.
pub mod service_scratch_ref_names_and_values;

/// The face a caller on a thread asks of the removal of a batch's scratch reference,
/// once the commit it holds is kept.
pub mod service_scratch_ref_of_a_dropped_batch;

/// The same removals as the face a task of the runtime awaits,
/// every Git child and every look at an intent a job of the pool.
pub mod service_scratch_ref_of_a_dropped_batch_awaiting;

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
