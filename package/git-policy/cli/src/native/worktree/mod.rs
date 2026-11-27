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
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `add_standing_notice.rs`:
//!   the line a `git worktree add` prints while it is still running,
//!   naming the step it is in.
//! - `add_timing.rs`:
//!   the record of where one wrapped `git worktree add` spent its time,
//!   step by step,
//!   appended beside the commit timing record.
//! - `checkout_workers.rs`:
//!   whether the checkout of a new worktree may be given Git's parallel checkout,
//!   and the option that does it.
//! - `idle_io_priority.rs`:
//!   running one stretch of storage work at the lowest I/O priority Linux offers.
//! - `inner_git_words.rs`:
//!   where the words of a Git run the wrapper or the service started on its own go,
//!   now that none carries `--quiet`.
//! - `ladder_step_ceilings.rs`:
//!   which of the ladder's own Git steps has a ceiling,
//!   and which number that ceiling is.
//! - `shell_quote.rs`:
//!   quoting of bytes printed inside a shell command for a person to run.
//! - `worktree_add_option_table.rs`:
//!   the option table of `git worktree add`,
//! - `worktree_add_plan_path_questions.rs`:
//!   small questions about a path or a byte that the add plans ask.
//! - `worktree_attributes.rs`:
//!   git's attribute-file lexer and pattern matcher over raw bytes,
//! - `worktree_audit.rs`:
//!   the facts one audit line of a removed worktree carries.
//! - `worktree_branch_reflog.rs`:
//!   the commits a branch's own reflog says it moved between,
//!   which is what says whether a branch a merge brought in holds work of its own.
//! - `worktree_change_a_rung_cannot_serve.rs`:
//!   one path the target tree of an add changes,
//!   and why a cheap rung cannot serve it.
//! - `worktree_clean_content_attribute_sources.rs`:
//!   the environment and attribute files a content check reads,
//!   and the attributes that convert content.
//! - `worktree_clean_stat_verdicts.rs`:
//!   what a clean check says about one tracked file:
//!   refusals,
//!   recorded stat data and the metadata verdict.
//! - `worktree_clean_untracked_scan.rs`:
//!   the one question the index of a worktree cannot answer:
//!   whether an untracked file is one Git ignores.
//! - `worktree_clone_block_cloning.rs`:
//!   the pool's block-cloning switch and what the clone rung does when it reads off.
//! - `worktree_clone_conversion_git_reads.rs`:
//!   how the conversion fence makes one Git run,
//!   and the two runs that read the blob sizes it decides with.
//! - `worktree_clone_conversion_rules.rs`:
//!   git's conversion rules for one path,
//!   read from its attributes and `core.autocrlf`/`core.eol`,
//!   as the decision whether a clone may carry the file.
//! - `worktree_clone_conversion_size_read_deadline.rs`:
//!   the join's shipped deadline,
//!   and the control that shortens it for one thread.
//! - `worktree_clone_conversion_size_read_free.rs`:
//!   the answered map of a size read an add will not use,
//!   and the thread that frees it,
//!   with the control seams that show where the free happened.
//! - `worktree_clone_domain.rs`:
//!   the clone domain of a reflink:
//!   whether two paths share data blocks,
//!   and why they do not.
//! - `worktree_clone_index.rs`:
//!   the install of the rung's index:
//!   the stat fields the clones recorded,
//!   and none for the paths the conversion fence leaves to Git.
//! - `worktree_clone_partial_index.rs`:
//!   the new worktree's index built from the tree of its `HEAD`,
//!   with the stat fields of the files the clones made.
//! - `worktree_completion_file_inspection.rs`:
//!   the descriptor-bound look at one added path's worktree copy.
//! - `worktree_completion.rs`:
//!   bringing added paths' worktree copies to their landed bytes.
//! - `worktree_convert_arguments.rs`:
//!   the grammar of `git monochromatic-git-policy-cli convert-worktree-to-subvolume`.
//! - `worktree_convert_compare.rs`:
//!   the read-only comparison of the old tree of a conversion with the installed one.
//! - `worktree_convert_filled_stages.rs`:
//!   what a conversion does about a stage container another command is filling below the tree it would swap or delete.
//! - `worktree_convert_journal.rs`:
//!   the journal of a conversion and the retirement of the kernel lock file older releases left.
//! - `worktree_convert_remove.rs`:
//!   the removal of the old tree of a conversion beneath a pinned descriptor.
//! - `worktree_finish_commands.rs`:
//!   the commands a message prints to finish or remove a worktree an interrupted `git worktree add` left.
//! - `worktree_identity.rs`:
//!   repository and worktree identity as reported by real Git.
//! - `worktree_command_facts.rs`:
//!   the plain facts of one worktree command that several folders pass around.
//! - `worktree_add_plan.rs`:
//!   the plan for one `git worktree add` that the cheap-worktree ladder serves:
//!   its destination and insertion point.
//! - `worktree_kept_notices.rs`:
//!   the record of a worktree a removal kept,
//!   which the next command of the repository shows once.
//! - `worktree_operands.rs`:
//!   the operands of one `git worktree` subcommand,
//!   read the way Git's own option parser leaves them.
//! - `worktree_path_failure_message.rs`:
//!   the one message shape of a failed step on a path:
//!   `<path> <what> (<error>)`.
//! - `worktree_add_binding.rs`:
//!   the binding of a `git worktree add` to the directories Git made for it,
//!   its record,
//!   and the check that the binding holds.
//! - `worktree_clean_content.rs`:
//!   the decision,
//!   by content and without a Git process,
//!   whether a tracked file that its index entry's stat data cannot vouch for (a racily clean entry,
//! - `worktree_clean_content_attribute_file_identity.rs`:
//!   which file a name holds,
//!   as its device and inode.
//! - `worktree_clean_content_attribute_bytes.rs`:
//!   the bytes of one attributes file,
//!   up to the room the caller's bound leaves.
//! - `worktree_clean_content_attribute_reading.rs`:
//!   the reading of one attributes file,
//!   at an absolute path or below the root.
//! - `worktree_clean_content_attribute_configuration.rs`:
//!   where Git takes the attributes files that apply whatever the path is.
//! - `worktree_clean_content_attributes.rs`:
//!   which attribute files apply to one path,
//!   and what the decision does with the answer.
//! - `worktree_clean_tree_walk.rs`:
//!   the walks of tracked files against the index's recorded stat data,
//!   by metadata or by content.
//! - `worktree_clean.rs`:
//!   whether a worktree is clean enough to copy from or to remove without asking Git,
//!   proven in process.
//! - `worktree_clone_conversion_decline.rs`:
//!   how many tracked paths a rule converts,
//!   and the share past which the rung declines to write them.
//! - `worktree_clone_partial_split.rs`:
//!   which tracked paths of a source with changes a clone may take,
//!   and which Git writes.
//! - `worktree_clone_paths.rs`:
//!   the clone rung's files in dependency order,
//!   the entry point last:
//!   one indexed path's names and the destination directories a run holds (`worktree_clone_paths`),
//!   one tracked entry and its clone (`worktree_clone_entry`),
//!   one run of entries on this thread (`worktree_clone_entries`),
//!   the slices of the blocking pool (`worktree_clone_slices`),
//!   the index install (`worktree_clone_index`),
//!   and the rung itself,
//!   whose module doc names each of them.
//! - `worktree_convert_inspection.rs`:
//!   the first look at a worktree a conversion might change,
//!   and every reason the conversion must not start.
//! - `worktree_convert_leftover_stage_record.rs`:
//!   the record,
//!   beside a worktree,
//!   of the objects one conversion may leave at its converting name.
//! - `worktree_convert_phases.rs`:
//!   the journaled phases of one worktree conversion.
//! - `worktree_convert_unrecorded_stage.rs`:
//!   the empty stage a conversion created and did not record,
//!   and whether the object at the converting name is it.
//! - `worktree_convert_verdict_and_options.rs`:
//!   what one conversion answers,
//!   and what the command asks of it.
//! - `worktree_convert_abandon.rs`:
//!   undoing a conversion that did not reach its swap.
//! - `worktree_convert.rs`:
//!   converting a linked worktree that is an ordinary directory into a btrfs subvolume at the same path,
//!   resumably.
//! - `worktree_convert_budget.rs`:
//!   how many conversions one run runs at once,
//!   from the bytes one holds and the ceiling the filesystem's work sets.
//! - `worktree_convert_targets.rs`:
//!   what one run covers and how its worktrees are laid into lanes.
//! - `worktree_convert_each_worktree.rs`:
//!   one worktree's conversion as a job of the blocking pool inside the runner's task.
//! - `worktree_convert_report.rs`:
//!   the lines a run says as its conversions finish,
//!   in input order,
//!   and how many of each kind there were.
//! - `worktree_convert_run_awaiting.rs`:
//!   the run on the runtime:
//!   every worktree a task,
//!   each conversion a job of the blocking pool,
//!   and the summary the run ends with.
//! - `worktree_convert_command.rs`:
//!   `git monochromatic-git-policy-cli convert-worktree-to-subvolume`:
//!   the worktrees to cover and the report.
//! - `worktree_process_inside.rs`:
//!   whether any process is working inside a folder,
//!   which keeps a landed worktree.
//! - `worktree_registration_proof.rs`:
//!   whether a directory is one of this repository's linked worktrees,
//!   proved from the pointer and the registration Git wrote there,
//!   which every command that changes state in a recorded worktree asks for first.
//! - `worktree_added_by_the_wrapper.rs`:
//!   the record that says the wrapper made one linked worktree,
//!   which every automatic removal asks for first.
//! - `worktree_landed_registrations.rs`:
//!   the linked worktrees a repository registers,
//!   with the branch each one has checked out.
//! - `worktree_removal_around_begun.rs`:
//!   the record a removal writes before its pass empties a worktree around a volume,
//!   so a command killed in that pass leaves a state the same command finishes.
//! - `worktree_removal_clears_the_pool.rs`:
//!   the removal of the shadow pool a removed worktree's commits were served from,
//!   taken after the removal proved the worktree is gone.
//! - `worktree_removal_held_commits_words.rs`:
//!   the words a person reads about a held commit,
//!   and the paragraph that stops a removal and names the command that finishes it.
//! - `worktree_removal_held_commits_keeping.rs`:
//!   keeping one held commit a record names on a reference of the common Git directory,
//!   at the batch's own kept name or beside a name another commit holds.
//! - `worktree_removal_held_commits_unreadable_record.rs`:
//!   the commit behind a held record this build cannot read,
//!   read from its batch's own reference and kept the same way,
//!   or the refusal that keeps the worktree.
//! - `worktree_removal_held_commits.rs`:
//!   what becomes of the commit a reference lock holds in a linked worktree that Git removes:
//!   the service of every linked worktree and the answer the sweep reads.
//! - `worktree_removal_interrupted_destroy.rs`:
//!   the finish of a removal whose destroy happened and whose registration did not go.
//! - `worktree_removal_leaving_that_outlives_its_record.rs`:
//!   the record of one subvolume a removal renamed aside,
//!   written before the rename and settled by nothing,
//!   and the queueing of its delete that outlives the queue record's own state.
//! - `worktree_removal_leaves_a_subvolume_for_the_service.rs`:
//!   the subvolumes this program leaves for the commit service to delete,
//!   moved out of a tree that is being removed with one rename and a record.
//! - `worktree_removal_clears_the_ahead_pool.rs`:
//!   the snapshots this program made ahead of an add,
//!   taken out of a worktree's administrative directory before that worktree is removed.
//! - `worktree_removal_rescue_removal_state.rs`:
//!   whether the worktree a removal snapshot was taken for is gone,
//!   refused,
//!   or still on its way,
//!   judged from the Git directory,
//!   the processes that took the snapshot and the checkout.
//! - `worktree_removal_snapshot_promotion.rs`:
//!   what a removal snapshot kept,
//!   promoted to the notices every worktree shows.
//! - `worktree_removal_rescue.rs`:
//!   what becomes of the unshown push notices and unfinished pushes of a linked worktree that Git removes.
//! - `worktree_removal_finishes_an_interrupted_destroy.rs`:
//!   the finish of a removal a command killed once its destroy happened,
//!   and the words for what a person still has to do.
//! - `worktree_remove_last_look.rs`:
//!   the question a removal's caller asks again as the removal runs,
//!   immediately before anything is destroyed,
//!   and the printing of the request that carries it.
//! - `worktree_remove_notices.rs`:
//!   the four lines a removal that destroyed a subvolume prints,
//!   and the one place that chooses between them.
//! - `worktree_remove_outcome.rs`:
//!   how one attempt to remove a worktree ended,
//!   and what each ending needs of the caller.
//! - `worktree_remove_registry.rs`:
//!   which registration of the invoking repository a worktree directory belongs to,
//!   and whether Git would pick that same worktree for the name the person typed.
//! - `worktree_move_source.rs`:
//!   the worktree a `git worktree move` renames,
//!   resolved the way Git resolves that operand.
//! - `worktree_removal_holds_the_worktree.rs`:
//!   the directories one removal holds open,
//!   the last look at them before the destroy,
//!   and the lock that keeps a worktree.
//! - `worktree_removal_stops_units.rs`:
//!   the stop of a removed worktree's commit service units,
//!   named from the registration before the removal and stopped after it.
//! - `worktree_removal_takes_its_pending_deletions.rs`:
//!   the subvolumes this program left inside a worktree for its commit service to delete,
//!   moved out of that tree before its removal hands anything to the kernel or to Git.
//! - `worktree_remove.rs`:
//!   the removal of a clean subvolume worktree by one ioctl,
//!   and the plan of the `git worktree remove` it serves.
//! - `worktree_remove_plan.rs`:
//!   the plan of a `git worktree remove` that names one directory and no option.
//! - `worktree_remove_reports.rs`:
//!   the three reports a removal can end with.
//! - `worktree_snapshot_source_changed_paths.rs`:
//!   what Git's status lists as changed or untracked in a worktree,
//!   for the snapshot source.
//! - `worktree_subvolume.rs`:
//!   the empty subvolume made at the destination of a `git worktree add` before Git runs,
//!   and its cleanup.
//! - `worktree_trash_exclude_line.rs`:
//!   the repository's own exclude line that keeps a standing trash out of every `git add --all`.
//! - `worktree_snapshot_source_cheap_refusals.rs`:
//!   the reasons a snapshot rung or a clone rung cannot serve a planned add,
//!   read from the two filesystems before anything is copied.
//! - `worktree_snapshot_source_refusal.rs`:
//!   why a source cannot be copied,
//!   and which kind of check said so.
//! - `worktree_snapshot_source_reads.rs`:
//!   the reads the source's checks make,
//!   each timed as the work it is:
//!   the Git configuration and the reasons it rules a source out,
//!   and the cleanliness of the files.
//! - `worktree_snapshot_source.rs`:
//!   the snapshot source:
//!   whether the creating worktree can be snapshotted for a new worktree,
//!   and what that needs.
//! - `worktree_clone_entry.rs`:
//!   one tracked entry of the frozen index,
//!   made in the destination with the checks that prove the source did not move.
//! - `worktree_clone_entries.rs`:
//!   one run of those entries on this thread:
//!   what it created,
//!   what it recorded for the index,
//!   and why it stopped.
//! - `worktree_clone_partial_staged.rs`:
//!   the live listing of the paths the source staged,
//!   and the refusal of a staged path among the clones.
//! - `worktree_clone_slices.rs`:
//!   those entries cut into slices:
//!   contiguous runs,
//!   one job of the blocking pool each,
//!   and the worker count the host's budget allows.
//! - `worktree_clone.rs`:
//!   the clone rung itself:
//!   tracked files cloned into the new worktree,
//!   the source's index installed with fresh stat fields.
//! - `worktree_ladder_target_conversion_agreement.rs`:
//!   the checks that the new worktree converts files the way the source does:
//!   its conversion settings in the order the configuration gives them,
//!   its `.gitattributes` files against the source's,
//!   and every filter an attribute selects or the configuration extends.
//! - `worktree_ladder_target.rs`:
//!   what both filling rungs read from Git's `--no-checkout` result,
//!   and the one step that applies the tree difference.
//! - `worktree_clone_conversion_fence.rs`:
//!   the fence over a clone rung's bytes:
//!   which paths the clones may carry,
//!   and the one Git run that writes the rest.
//! - `worktree_clone_conversion_fence_measurement_routes.rs`:
//!   the measurement-only shapes of that fence:
//!   option C,
//!   option D,
//!   and the switch that picks one of them.
//! - `worktree_clone_conversion_beside_the_clone.rs`:
//!   the same fence beside the clone:
//!   its reads on a thread of their own while the tracked files are copied,
//!   and the seed the index is installed from once both have answered.
//! - `worktree_clone_conversion_size_read_answer.rs`:
//!   the answer a size read sends and the value one add holds while it runs,
//!   with the join within its deadline and the abandon that ends its Git child.
//! - `worktree_clone_conversion_size_read_seams.rs`:
//!   the seams a control arms to hold a read before its Git run,
//!   to hold it after its child ended,
//!   and to make its thread die before it answers.
//! - `worktree_clone_conversion_size_read_git_read.rs`:
//!   the early read's one `git ls-tree` run,
//!   the variable that keeps a partial clone from fetching,
//!   and the parse of the sizes it prints.
//! - `worktree_clone_conversion_size_read_start.rs`:
//!   the start of that read beside the add's source check,
//!   the thread that carries it,
//!   and the answer it sends once.
//! - `worktree_clone_conversion_size_read.rs`:
//!   the fence's one `git ls-tree` size read:
//!   the task a rung-1 add can start before its source check,
//!   and the join at the fence.
//! - `worktree_clone_partial_tracked.rs`:
//!   the source's index and the tree of its `HEAD`,
//!   the paths Git writes,
//!   and the stat fields the new index records.
//! - `worktree_clone_rung_limits.rs`:
//!   the provisional limits that send an add to Git's own checkout:
//!   a whole-tree size threshold where the source's filesystem has one,
//!   a mean file size edge,
//!   and a changed-file cut,
//!   each with the measurement it rests on.
//! - `worktree_clone_partial.rs`:
//!   the clone rung for a source with uncommitted changes:
//!   the paths the proof settles cloned,
//!   the index built from the tree,
//!   and the rest written by Git.
//! - `worktree_dirty_restore.rs`:
//!   the restore of a snapshot of a main checkout with uncommitted changes,
//!   and its proof.
//! - `worktree_ladder_finish_clones.rs`:
//!   the answers worktree ladder finish clones holds.
//! - `worktree_ladder_target_tree_skip.rs`:
//!   the one read that lets a fast rung step aside when the tree the add names changes a path the fence must have Git
//!   write.
//! - `worktree_landed_ignored_files.rs`:
//!   the ignored entries of a landed worktree that nothing else in the repository holds,
//!   which keep it.
//! - `worktree_removal_only_deletions.rs`:
//!   whether everything Git reports about a worktree is an entry missing from it,
//!   which is the state a removal that was killed in the middle leaves.
//! - `worktree_removal_goes_around_a_foreign_volume.rs`:
//!   the removal that goes around a subvolume or a mount point it did not make,
//!   and the words it says about what it left.
//! - `worktree_removal_second_look.rs`:
//!   the proof that a removal may go,
//!   made again immediately before the destruction.
//! - `worktree_landed_removal_last_look.rs`:
//!   what a removal deletes beside the files Git keeps,
//!   and the sweep's last look that asks it again inside the removal.
//! - `worktree_remove_around.rs`:
//!   the half of a removal that goes around a volume it did not make:
//!   the record a killed pass leaves,
//!   the continuation that record proves,
//!   and the pass.
//! - `worktree_remove_hand_over_records_beside_the_registration.rs`:
//!   the records a removal writes beside a worktree's registration,
//!   which say which worktree a record was written for.
//! - `worktree_remove_hand_over_begun_record.rs`:
//!   the record that says a removal of one worktree began,
//!   so a command killed inside it is finished by the next one.
//! - `worktree_remove_hand_over_service_git_and_pool.rs`:
//!   the Git a service's own removal starts,
//!   and the pool of the worktree it removes.
//! - `worktree_remove_hand_over_kept_worktree_notice.rs`:
//!   the words a person reads for a worktree a removal kept.
//! - `worktree_remove_hand_over_queue_and_wake.rs`:
//!   the record a removal queues for the commit service,
//!   and the wake that makes the service of the repository read it.
//! - `worktree_remove_hand_over_subvolume_delete.rs`:
//!   the kernel's destroy of the subvolume one record names.
//! - `worktree_remove_hand_over.rs`:
//!   the hand-over of a worktree's subvolume to the commit service,
//!   the removal the service makes around a volume that is not its own,
//!   and the names every caller already uses.
//! - `worktree_remove_hand_over_service_removal.rs`:
//!   the service's own removal of a whole worktree with Git,
//!   and the choice of which destroy a record asks for.
//! - `worktree_landed_removal_records.rs`:
//!   what one landed sweep is asked with and answers with:
//!   where it looks,
//!   each worktree it removed,
//!   left,
//!   finished or handed to the commit service,
//!   and everything one sweep did.
//! - `worktree_landed_removal_landed_branches.rs`:
//!   which branches the merge that just ran landed,
//!   and which of them hold a commit of their own.
//! - `worktree_landed_removal_one_candidate.rs`:
//!   one candidate of a landed sweep:
//!   the conditions it must meet,
//!   the fast destroy or the hand over to the commit service,
//!   and what the attempt came to.
//! - `worktree_landed_removal.rs`:
//!   the removal of every wrapper-made worktree whose branch a merge landed,
//!   which the commit service performs:
//!   the candidates this merge landed and the walk over them.
//! - `worktree_landed_sweep_notice.rs`:
//!   the words one landed sweep prints.
//! - `worktree_landed_after_a_merge.rs`:
//!   the commit and the branch one merge stood between,
//!   and the sweep of the worktrees whose branches it landed.
//! - `worktree_removal_leavings_a_command_queues_again.rs`:
//!   the look every command makes at those records:
//!   the delete of a leaving no queue record owes is queued again,
//!   and a place this program may not act on is named for the person.
//! - `worktree_slow_route.rs`:
//!   the plain sentence that says why an add was not made as a snapshot,
//!   and where it goes.
//! - `worktree_snapshot_index_step.rs`:
//!   the index step of the snapshot rung:
//!   prove the copy against the source's index,
//!   settle an entry by content where the stat data cannot vouch for it,
//!   and keep the snapshot's own index when the source changed under the copy.
//! - `worktree_snapshot_ahead.rs`:
//!   the snapshots the wrapper makes ahead of an add,
//!   and the add's claim of one instead of a snapshot of its own.
//! - `worktree_snapshot_ahead_claim.rs`:
//!   what one add does with the snapshots the service made ahead of it:
//!   the proof,
//!   the claim,
//!   the bounded wait and the sweep.
//! - `worktree_snapshot_ahead_job.rs`:
//!   the commit service's own snapshot of a worktree,
//!   made for the add after the one that asked for it.
//! - `worktree_trash_hand_over.rs`:
//!   handing a snapshot's trash to the commit service without waiting for it to be emptied.
//! - `worktree_trash_ignore_rule.rs`:
//!   whether Git ignores the place a snapshot's trash would be made at in one worktree.
//! - `worktree_trash_manifest.rs`:
//!   what a snapshot's leave-out made and moved,
//!   with identities,
//!   as the text a work item carries.
//! - `worktree_trash_dispose_awaiting.rs`:
//!   the service's handler for a trash work item,
//!   awaited by a caller that is itself a task of the runtime.
//! - `worktree_trash_own_manifest.rs`:
//!   the manifest a trash carries inside itself,
//!   written before the swap publishes it,
//!   so a later pass can prove what the trash is without a record.
//! - `worktree_trash_remove.rs`:
//!   removal of a directory tree beneath a held parent handle,
//!   never following a link.
//! - `worktree_trash.rs`:
//!   setting aside what a snapshot of the main checkout leaves out,
//!   and having the commit service empty it.
//! - `worktree_trash_entries.rs`:
//!   the removal of the entries a snapshot's leave-out moved into its trash,
//!   side by side.
//! - `worktree_trash_dispose.rs`:
//!   emptying a snapshot's trash:
//!   what a trash is,
//!   the service's handler,
//!   and the trashes nothing will empty.
//! - `worktree_ladder_finish_trash_record.rs`:
//!   the record of the trash a snapshot's leave-out made,
//!   written before the rung that published the worktree returns.
//! - `worktree_trash_set_aside.rs`:
//!   the trash directory one snapshot's leave-out made and holds open,
//!   the one the stand-in step puts a whole worktree folder into,
//!   the two ways it is made or opened again,
//!   and the moves in and out of it.
//! - `worktree_kept_folders.rs`:
//!   the folder a taken worktree folder is kept in when the name it came from is taken,
//!   and the cleanup that must leave it alone.
//! - `worktree_ladder_run.rs`:
//!   the ladder's choice of rung before real Git runs.
//! - `worktree_ladder.rs`:
//!   the cheap-worktree ladder around one `git worktree add`:
//!   what runs before real Git and after it.
//! - `worktree_snapshot_placeholders.rs`:
//!   the empty folders a snapshot holds where its source holds a nested subvolume it did not carry,
//!   found below the folders that hold worktrees and removed as empty directories.
//! - `worktree_snapshot.rs`:
//!   the snapshot rung's mechanics:
//!   snapshot to a stage name,
//!   make it a worktree,
//!   swap it into place.
//! - `worktree_ladder_finish.rs`:
//!   the ladder's steps after real Git returned:
//!   the snapshot rung and the fall back to Git's checkout.
//! - `worktree_trash_set_aside_entries.rs`:
//!   the entries a snapshot's leave-out sets aside,
//! - `worktree_trash_unaccounted.rs`:
//!   what every later command in a worktree says about the trash left there:
//!   which directory nothing will empty,
//!   which trash it records again,
//!   and the wake that tells the service so.
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

/// The plain facts of one worktree command that several folders pass around:
///  what a forwarded run must copy,
///  and the plans of a `git worktree add` and a `git worktree remove`.
pub mod worktree_command_facts;

/// The folder a taken worktree folder is kept in when the name it came from is taken,
/// and the cleanup that must leave it alone.
pub mod worktree_kept_folders;

/// The line a `git worktree add` prints while it is still running,
/// naming the step it is in.
pub mod add_standing_notice;

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

/// The words a person reads about a held commit,
/// and the paragraph that stops a removal and names the command that finishes it.
pub mod worktree_removal_held_commits_words;

/// Keeping one held commit a record names on a reference of the common Git directory,
/// at the batch's own kept name or beside a name another commit holds.
pub mod worktree_removal_held_commits_keeping;

/// The commit behind a held record this build cannot read,
/// read from its batch's own reference and kept the same way,
/// or the refusal that keeps the worktree.
pub mod worktree_removal_held_commits_unreadable_record;

/// What becomes of the commit a reference lock holds in a linked worktree that Git removes:
/// the service of every linked worktree and the answer the sweep reads.
pub mod worktree_removal_held_commits;

/// The option table of `git worktree add`,
/// and the rules that read a long word,
/// a bundled short word and a value.
// Its only user is `worktree_add_plan`, which is `#[cfg(unix)]`, so a non-Unix build would hold this table's
// `pub(crate)` items unused and the Windows lint denies that warning.
#[cfg(unix)]
pub mod worktree_add_option_table;

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

/// Which file a name holds,
/// as its device and inode.
pub mod worktree_clean_content_attribute_file_identity;

/// The bytes of one attributes file,
/// up to the room the caller's bound leaves.
pub mod worktree_clean_content_attribute_bytes;

/// The reading of one attributes file,
/// at an absolute path or below the root.
pub mod worktree_clean_content_attribute_reading;

/// Where Git takes the attributes files that apply whatever the path is.
pub mod worktree_clean_content_attribute_configuration;

/// Which attribute files apply to one path,
/// and what the decision does with the answer.
pub mod worktree_clean_content_attributes;

/// The environment and attribute files a content check reads,
/// and the attributes that convert content.
#[cfg(target_os = "linux")]
pub mod worktree_clean_content_attribute_sources;

/// The operands of one `git worktree` subcommand,
/// read the way Git's own option parser leaves them.
#[cfg(unix)]
pub mod worktree_operands;

/// The plan of a `git worktree remove` that names one directory and no option.
#[cfg(unix)]
pub mod worktree_remove_plan;

/// The worktree a `git worktree move` renames,
/// resolved the way Git resolves that operand.
#[cfg(unix)]
pub mod worktree_move_source;

/// The directories one removal holds open,
/// the last look at them before the destroy,
/// and the lock that keeps a worktree.
pub mod worktree_removal_holds_the_worktree;

/// The removal of a clean subvolume worktree by one ioctl,
/// and the plan of the `git worktree remove` it serves.
#[cfg(target_os = "linux")]
pub mod worktree_remove;

/// How one attempt to remove a worktree ended,
/// and what each ending needs of the caller.
pub mod worktree_remove_outcome;

/// The four lines a removal that destroyed a subvolume prints,
/// and the one place that chooses between them.
pub mod worktree_remove_notices;

/// The half of a removal that goes around a volume it did not make:
/// the record a killed pass leaves,
/// the continuation that record proves,
/// and the pass.
pub mod worktree_remove_around;
pub mod worktree_remove_hand_over;
pub mod worktree_remove_hand_over_begun_record;
pub mod worktree_remove_hand_over_kept_worktree_notice;
pub mod worktree_remove_hand_over_queue_and_wake;
pub mod worktree_remove_hand_over_records_beside_the_registration;
pub mod worktree_remove_hand_over_service_git_and_pool;
pub mod worktree_remove_hand_over_service_removal;
pub mod worktree_remove_hand_over_subvolume_delete;

pub mod worktree_remove_registry;

/// The record of a worktree a removal kept,
/// which the next command of the repository shows once.
pub mod worktree_kept_notices;

/// The proof that a removal may go,
/// made again immediately before the destruction.
pub mod worktree_removal_second_look;

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

/// The finish of a removal a command killed once its destroy happened,
/// and the words for what a person still has to do.
pub mod worktree_removal_finishes_an_interrupted_destroy;

/// The subvolumes this program leaves for the commit service to delete,
/// moved out of a tree that is being removed with one rename and a record.
pub mod worktree_removal_leaves_a_subvolume_for_the_service;

/// The record of one subvolume a removal renamed aside,
/// written before the rename and settled by nothing,
/// and the queueing of its delete that outlives the queue record's own state.
pub mod worktree_removal_leaving_that_outlives_its_record;

/// The look every command makes at those records:
/// the delete of a leaving no queue record owes is queued again,
/// and a place this program may not act on is named for the person.
pub mod worktree_removal_leavings_a_command_queues_again;

/// The stop of a removed worktree's commit service units,
/// named from the registration before the removal and stopped after it.
pub mod worktree_removal_stops_units;

/// The removal of the shadow pool a removed worktree's commits were served from,
/// taken after the removal proved the worktree is gone.
pub mod worktree_removal_clears_the_pool;

/// The snapshots this program made ahead of an add,
/// taken out of a worktree's administrative directory before that worktree is removed.
pub mod worktree_removal_clears_the_ahead_pool;

/// The removal that goes around a subvolume or a mount point it did not make,
/// and the words it says about what it left.
pub mod worktree_removal_goes_around_a_foreign_volume;

/// The record a removal writes before its pass empties a worktree around a volume,
/// so a command killed in that pass leaves a state the same command finishes.
pub mod worktree_removal_around_begun;

/// Whether everything Git reports about a worktree is an entry missing from it,
/// which is the state a removal that was killed in the middle leaves.
pub mod worktree_removal_only_deletions;

/// The subvolumes this program left inside a worktree for its commit service to delete,
/// moved out of that tree before its removal hands anything to the kernel or to Git.
pub mod worktree_removal_takes_its_pending_deletions;

/// The one message shape of a failed step on a path:
/// `<path> <what> (<error>)`.
pub mod worktree_path_failure_message;

/// Fixtures shared by the controls of the cheap-worktree ladder.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod worktree_ladder_test_support;

/// The clone domain of a reflink:
/// whether two paths share data blocks,
/// and why they do not.
#[cfg(target_os = "linux")]
pub mod worktree_clone_domain;

/// What Git's status lists as changed or untracked in a worktree,
/// for the snapshot source.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source_changed_paths;

/// The cheap reasons a snapshot rung or a clone rung cannot serve a planned add.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source_cheap_refusals;

/// Why a source cannot be copied,
/// and which kind of check said so.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source_refusal;

/// The reads the source's checks make,
/// each timed as the work it is.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source_reads;

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

/// The clone rung's files in dependency order,
/// the entry point last:
/// one indexed path's names and the destination directories a run holds
/// (`worktree_clone_paths`),
/// one tracked entry and its clone (`worktree_clone_entry`),
/// one run of entries on this thread (`worktree_clone_entries`),
/// the slices of the blocking pool (`worktree_clone_slices`),
/// the index install (`worktree_clone_index`),
/// and the rung itself,
/// whose module doc names each of them.
#[cfg(target_os = "linux")]
pub mod worktree_clone_paths;

/// One tracked entry of the frozen index,
/// made in the destination with the checks that prove the source did not move.
#[cfg(target_os = "linux")]
pub mod worktree_clone_entry;

/// One run of those entries on this thread:
/// what it created,
/// what it recorded for the index,
///  and why it stopped.
#[cfg(target_os = "linux")]
pub mod worktree_clone_entries;

/// Those entries cut into slices:
/// contiguous runs,
/// one job of the blocking pool each,
///  and the worker count the host's budget allows.
#[cfg(target_os = "linux")]
pub mod worktree_clone_slices;

/// The install of the rung's index:
/// the stat fields the clones recorded,
/// and none for the paths the conversion fence leaves to Git.
#[cfg(target_os = "linux")]
pub mod worktree_clone_index;

/// The clone rung itself:
/// tracked files cloned into the new worktree,
/// the source's index installed with fresh stat fields.
#[cfg(target_os = "linux")]
pub mod worktree_clone;

/// The clone rung for a source with uncommitted changes:
/// the paths the proof settles cloned,
/// the index built from the tree,
/// and the rest written by Git.
#[cfg(target_os = "linux")]
pub mod worktree_clone_partial;

/// Which tracked paths of a source with changes a clone may take,
/// and which Git writes.
#[cfg(target_os = "linux")]
pub mod worktree_clone_partial_split;

/// The new worktree's index built from the tree of its `HEAD`,
/// with the stat fields of the files the clones made.
#[cfg(target_os = "linux")]
pub mod worktree_clone_partial_index;

/// The source's index and the tree of its `HEAD`,
/// the paths Git writes,
/// and the stat fields the new index records.
#[cfg(target_os = "linux")]
pub mod worktree_clone_partial_tracked;

/// The live listing of the paths the source staged,
/// and the refusal of a staged path among the clones.
#[cfg(target_os = "linux")]
pub mod worktree_clone_partial_staged;

/// The provisional limits that send an add to Git's own checkout:
/// a whole-tree size threshold where the source's filesystem has one,
/// a mean file size edge,
/// and a changed-file cut,
/// each with the measurement it rests on.
#[cfg(target_os = "linux")]
pub mod worktree_clone_rung_limits;

/// Git's conversion rules for one path,
/// read from its attributes and `core.autocrlf`/`core.eol`,
/// as the decision whether a clone may carry the file.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_rules;

/// How many tracked paths a rule converts,
/// and the share past which the rung declines to write them.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_decline;

/// One path the target tree of an add changes,
/// and why a cheap rung cannot serve it.
#[cfg(target_os = "linux")]
pub mod worktree_change_a_rung_cannot_serve;

/// The fence over a clone rung's bytes:
/// which paths the clones may carry,
/// and the one Git run that writes the rest.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_fence;

/// The measurement-only shapes of that fence:
/// option C,
/// option D,
/// and the switch that picks one of them.
#[cfg(all(target_os = "linux", feature = "measurement-routes"))]
pub mod worktree_clone_conversion_fence_measurement_routes;

/// The same fence beside the clone:
/// its reads on a thread of their own while the tracked files are copied,
/// and the seed the index is installed from once both have answered.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_beside_the_clone;

/// How the conversion fence makes one Git run,
/// and the two runs that read the blob sizes it decides with.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_git_reads;

/// The fence's one `git ls-tree` size read:
/// the task a rung-1 add can start before its source check,
/// and the join at the fence.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_size_read;

/// The answer a size read sends and the value one add holds while it runs,
/// with the join within its deadline and the abandon that ends its Git child.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_size_read_answer;

/// The seams a control arms to hold a read before its Git run,
/// to hold it after its child ended,
/// and to make its thread die before it answers.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_size_read_seams;

/// The early read's one `git ls-tree` run,
/// the variable that keeps a partial clone from fetching,
/// and the parse of the sizes it prints.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_size_read_git_read;

/// The start of that read beside the add's source check,
/// the thread that carries it,
/// and the answer it sends once.
#[cfg(target_os = "linux")]
pub mod worktree_clone_conversion_size_read_start;

/// The pool's block-cloning switch and what the clone rung does when it reads off.
#[cfg(target_os = "linux")]
pub mod worktree_clone_block_cloning;

/// The checks that the new worktree converts files the way the source does:
/// its conversion settings in the order the configuration gives them,
/// its `.gitattributes` files against the source's,
/// and every filter an attribute selects or the configuration extends.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_target_conversion_agreement;

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

/// The one read that lets a fast rung step aside when the tree the add names changes a path the fence must have Git write.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_target_tree_skip;

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

/// What one conversion answers,
/// and what the command asks of it.
#[cfg(target_os = "linux")]
pub mod worktree_convert_verdict_and_options;

/// Undoing a conversion that did not reach its swap.
#[cfg(target_os = "linux")]
pub mod worktree_convert_abandon;

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

/// How many conversions one run of `convert-worktree-to-subvolume` runs at once.
#[cfg(target_os = "linux")]
pub mod worktree_convert_budget;

/// What one run of `convert-worktree-to-subvolume` covers,
/// and how its worktrees are laid into lanes.
#[cfg(target_os = "linux")]
pub mod worktree_convert_targets;

/// One worktree's conversion as a job of the blocking pool inside the runner's task.
#[cfg(target_os = "linux")]
pub mod worktree_convert_each_worktree;

/// The lines a run of `convert-worktree-to-subvolume` says as its conversions finish,
/// in input order.
#[cfg(target_os = "linux")]
pub mod worktree_convert_report;

/// The run of `convert-worktree-to-subvolume` on the runtime,
/// with what it needs besides the line sink.
#[cfg(target_os = "linux")]
pub mod worktree_convert_run_awaiting;

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

/// The commits a branch's own reflog says it moved between,
/// which is what says whether a branch a merge brought in holds work of its own.
pub mod worktree_branch_reflog;

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

/// What one landed sweep is asked with and answers with,
/// and everything one sweep did.
pub mod worktree_landed_removal_records;

/// Which branches the merge that just ran landed,
/// and which of them hold a commit of their own.
pub mod worktree_landed_removal_landed_branches;

/// One candidate of a landed sweep:
/// the conditions it must meet,
/// the fast destroy or the hand over to the commit service,
/// and what the attempt came to.
pub mod worktree_landed_removal_one_candidate;

/// The removal of every wrapper-made worktree whose branch a merge landed,
/// which the commit service performs:
/// the candidates this merge landed and the walk over them.
pub mod worktree_landed_removal;

/// The words one landed sweep prints.
pub mod worktree_landed_sweep_notice;

/// Whether any process is working inside a folder,
/// which keeps a landed worktree.
pub mod worktree_process_inside;
