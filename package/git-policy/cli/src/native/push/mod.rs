//! What:
//!  Pushes:
//!  the automatic push after a commit,
//!  manual push handling,
//!  bounded Git queries,
//!  retries and progress watching.
//! Why:
//!  Push has a long lifecycle with many waits.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `auto_push_branch_key.rs`:
//!   the per-branch file name of the push failure notice:
//!   a reversible,
//!   flat,
//!   case-safe key,
//!   and the hash that shortens a key too long for a file name.
//! - `auto_push_dirty_paths.rs`:
//!   the paths of a dirty worktree and of a fast-forward,
//!   and their overlap.
//! - `auto_push_refspec_judging.rs`:
//!   the reading of a configured push refspec and the judgement of what the refspecs make of one branch.
//! - `bounded_query_input.rs`:
//!   the thread that gives a second-pass command its request,
//!   and the bounds it runs under.
//! - `child_watch_running_task.rs`:
//!   the task a watch runs,
//!   the bound its end is given,
//!   and the refusal a control arms.
//! - `manual_push_cause.rs`:
//!   why a manual push's scope could not be established,
//!   as the cause its hint is chosen by.
//! - `manual_push_porcelain.rs`:
//!   authoritative manual-push updates discovered by a private dry-run push and confirmed with ls-remote.
//! - `push_deadline_and_patience_settings.rs`:
//!   the deadline a push may show nothing for,
//!   its default and the patience a push runs under.
//! - `child_class.rs`:
//!   the classes of Git step a commit starts for itself,
//!   with the bound and the notice grace of each.
//! - `child_watch_step_bounds_and_errors.rs`:
//!   how a watched step may be ended and the errors that say it was.
//! - `child_watch_signalling_only_the_child_that_was_started.rs`:
//!   how a stop reaches the child it started and never another command:
//!   the pin of a step's root taken while that child is this process's own unreaped child,
//!   the signals sent through it and the processes below the root read through it.
//! - `push_deadline_advice.rs`:
//!   the words that say how to change the push deadline,
//!   for the terminal and for the resident commit service.
//! - `one_push_attempt_and_its_failure.rs`:
//!   what one push this invocation ran left behind:
//!   the attempt it ended as,
//!   and the failure that stopped a push or one of its Git queries.
//! - `push_failure_notice_record_text.rs`:
//!   what a record's text is:
//!   its first line,
//!   the field lines that name its branch and its commit,
//!   the name it is kept under,
//!   the words it carries,
//!   and their cleaning.
//! - `push_failure_notice_worktrees.rs`:
//!   which worktree a record belongs to:
//!   the branch each worktree has checked out,
//!   the rule that says a record is the asking worktree's,
//!   and the way a commit is named for the person.
//! - `push_failure_notice_claimed_by_one_command.rs`:
//!   the name a command holds a notice's record under while it shows it,
//!   and who may take one over.
//! - `push_failure_notice_machine_output.rs`:
//!   whether a command's output is read by a program,
//!   so that no failure notice is shown to it.
//! - `push_failure_notice_reports.rs`:
//!   the record of what a push that succeeded did,
//!   and the name it is kept under.
//! - `push_failure_notice_record_parsed.rs`:
//!   a record read back:
//!   the kind its first line names,
//!   the branch,
//!   the commit,
//!   and the words after them.
//! - `push_failure_notice_shown_then_removed.rs`:
//!   one record of a failed push as a command holds it,
//!   with the step that shows it and removes it.
//! - `push_failure_notice_cleared_after_a_push.rs`:
//!   the records a push of a branch makes no longer true,
//!   and the step that removes them once such a push succeeded.
//! - `push_limit_held_for_an_ordering.rs`:
//!   where a control holds a run's limit until the ordering it needs has happened,
//!   so a stop is never decided by elapsed time alone.
//! - `push_minimum_rate.rs`:
//!   the least traffic a push's connection must carry in one window and still count as progress,
//!   the variable that sets it and its measured default.
//! - `push_retry_store_plan_record.rs`:
//!   the record a retry plan is kept as:
//!   its version and the JSON that reads back as the plan.
//! - `push_retry_store_loaded.rs`:
//!   reading every plan a service instance's journal holds,
//!   one plan per branch and whether something could not be read.
//! - `push_retry_pauses.rs`:
//!   the pause before each new try of a push the service stopped or that failed.
//! - `push_retry_waiting_branches.rs`:
//!   the shapes one round of retries is handed:
//!   a branch waiting,
//!   the tries due again,
//!   and the change a worker asks of the plan.
//! - `push_retry_cause_line.rs`:
//!   the first of Git's own lines that gives a reason for a push's failure.
//! - `push_retry_store.rs`:
//!   the retry plan on disk,
//!   in the service instance's journal,
//!   so that a restarted service keeps trying the pushes it was trying.
//! - `push_trigger_log.rs`:
//!   the record of each decision of the push trigger:
//!   handed over,
//!   or declined with its reason.
//! - `auto_push_trigger_watched_branch.rs`:
//!   the branch a command started on,
//!   watched before it runs:
//!   where it pointed and the tip it held.
//! - `auto_push_trigger_ahead_of_upstream.rs`:
//!   the checks that say whether the moved branch grew ahead of its upstream,
//!   with the walk over the commit parents and the decline a check answers with.
//! - `auto_push_trigger.rs`:
//!   the push a merge,
//!   pull,
//!   cherry-pick,
//!   revert or am hands to the service when it leaves the branch ahead of its upstream.
//! - `start_retry.rs`:
//!   starting a child again when the operating system refuses it for lack of processes,
//!   within a bounded wait.
//! - `stop_cause_and_words.rs`:
//!   the causes and reports of a stop,
//!   and the line a stalled push prints,
//!   which names the window that fired and a remedy that works.
//! - `stop_of_a_push_and_sweep_of_its_locks.rs`:
//!   the stop of a running push and the sweep of the lock files it and what it started leave.
//! - `tcp_sockets.rs`:
//!   the connected TCP sockets of the machine with their byte counts,
//!   and the sockets a process holds.
//! - `push_progress_signs.rs`:
//!   the two signs one look at a process of a push finds,
//!   and the identities a holder is remembered by.
//! - `push_progress_meter.rs`:
//!   what one look at a push's tree finds:
//!   the bytes each process moved,
//!   the processor time it used,
//!   its tasks' states,
//!   and the connected sockets' own counts.
//! - `push_held_back_by_a_blocked_commit.rs`:
//!   where the automatic push stops when a post-commit gate kept one of a branch's commits local,
//!   and the decision a verdict makes.
//! - `push_progress.rs`:
//!   the clock that stops a push on a stall,
//!   over what one look at its tree finds.
//! - `push_wait_until_done.rs`:
//!   the wait of a bounded run:
//!   it ends when the run does,
//!   or stops when the run shows no work,
//!   floods its output or the service is stopped,
//!   with the cap it is judged by and the words a wait adds.
//! - `wait_progress.rs`:
//!   the shared clock of every wait notice:
//!   the grace,
//!   the repeat interval and a queued commit's place in line.
//! - `child_watch_clock_marks_and_stopping.rs`:
//!   what a child watch is made of below its public face:
//!   terms,
//!   marks,
//!   shared flags and the stopping of a child.
//! - `child_watch_task_loop_and_notice.rs`:
//!   the loop of a child watch's task and the writing of its notices.
//! - `child_watch.rs`:
//!   a watcher for waits that block on a child process:
//!   notices at the cadence and the child stopped at the bound.
//! - `child_watch_one_shot_run.rs`:
//!   the one-shot run of a watched step:
//!   the command started,
//!   its output read whole,
//!   and the watch that ends it.
//! - `bounded_query_run_of_one_command.rs`:
//!   one read-only Git command run to its end,
//!   or until it has shown no sign of work for its window,
//!   with the answer it printed and how it ended.
//! - `bounded_query.rs`:
//!   read-only Git queries of the auto-push,
//!   each with a deadline and separate output streams.
//! - `child_watch_output_watched_on_the_runtime.rs`:
//!   the watched one-shot run for a caller that is itself a task of the runtime.
//! - `auto_push_single_flight.rs`:
//!   the real Git of a push and its query runner.
//! - `manual_push_probe_arguments.rs`:
//!   the probe's Git arguments and the destination locations it confirms,
//!   beside the discovery so that each file passes the package's length limit.
//! - `manual_push_probe_error.rs`:
//!   why the probe of a manual push could not establish the updates Git would push,
//!   and what Git said when it refused.
//! - `push_server_naming.rs`:
//!   the server a push waits for,
//!   named from the command's own destination.
//! - `auto_push_destination_settings.rs`:
//!   the settings that decide where one branch is pushed,
//!   read from one listing of the configuration.
//! - `bounded_push.rs`:
//!   the auto-push's real `git push` with a deadline and its low-speed abort.
//! - `auto_push_reconcile_merge_commit_answer_ceilings.rs`:
//!   the largest answer one command of the second pass may print,
//!   and the ceiling each command is bounded by.
//! - `auto_push_destination_by_push_default.rs`:
//!   the ref `push.default` sends one branch to,
//!   or the note that says why Git sends it nowhere.
//! - `auto_push_destination_by_refspecs.rs`:
//!   where the configured push refspecs send one branch,
//!   judged from one answer about the local refs.
//! - `auto_push_destination.rs`:
//!   where a plain `git push` of one named branch goes,
//!   from the configuration and the local refs.
//! - `auto_push_reconcile_basics_and_limits.rs`:
//!   what a reconcile of a rejected push shares:
//!   the line prefixes,
//!   the attempt limit,
//!   the stale-rejection test and the small Git queries.
//! - `auto_push_reconcile_branch_move.rs`:
//!   the step that moves the branch to the published merge,
//!   with the window a command's own `git switch` opens in it
//!   and the refusal Git answers with.
//! - `auto_push_reconcile_branch_advance.rs`:
//!   when the local branch may move after the merge was pushed,
//!   and the record that settles a move left half done.
//! - `auto_push_reconcile_plan.rs`:
//!   what one reconcile run integrates,
//!   where it publishes,
//!   how it words itself and how it ended,
//!   so the loop and the entry that reaches it read one plan.
//! - `matching_push_delivered_the_recorded_branch.rs`:
//!   whether a push Git reported as failed published the recorded branch all the same,
//!   which `push.default=matching` makes possible by carrying every branch the remote has in one push.
//! - `push_held_back_by_a_blocked_commit_awaiting.rs`:
//!   where the push of one branch stops when its questions go to a Git awaited on the runtime,
//!   which is the decision of `push_held_back_by_a_blocked_commit.rs` with its waits as tasks of the runtime.
//! - `auto_push_reconcile_merge_commit.rs`:
//!   the merge commit of a reconcile,
//!   built in the object store without a worktree or an index.
//! - `push_retry.rs`:
//!   the service's own retry of a push it stopped or that failed:
//!   the list of branches waiting for a new try,
//!   and what a start rebuilds it from.
//! - `push_retry_changes_applied.rs`:
//!   what the workers of a round asked,
//!   applied to the list the service loop reads.
//! - `push_retry_failure.rs`:
//!   what a failed push's own words mean for that retry:
//!   whether the failure is final,
//!   how many tries the one counted cause gets,
//!   and the words the person reads.
//! - `auto_push_reconcile_awaiting.rs`:
//!   the reconcile loop and its merge push,
//!   awaited by a task of the runtime,
//!   which is the one body of those decisions.
//! - `auto_push_reconcile.rs`:
//!   publishing a landed commit that a moved remote branch rejected:
//!   the merge is made from objects alone and pushed.
//! - `manual_push_update_and_query_limits.rs`:
//!   what a push would update:
//!   the public update of one pushed reference,
//!   the limits on the probe's side-by-side queries,
//!   and the batch query whose answer a range and its published tips are read with.
//! - `manual_push_range_published_tips.rs`:
//!   what a remote has already published:
//!   the commits a prior destination value names,
//!   and the tracking refs Git keeps.
//! - `manual_push_range.rs`:
//!   the range computation for manual-push scans:
//!   which pushed commits a remote does not have.
//! - `manual_push_probe_confirmation.rs`:
//!   one destination's confirmation:
//!   the updates it confirmed,
//!   and everything its one `ls-remote` reported.
//! - `manual_push_probe_destination_blocks.rs`:
//!   one destination's block of Git's own dry-run report,
//!   confirmed:
//!   the local objects a new ref needs,
//!   the updates Git printed for that destination,
//!   and the pieces that confirm every destination side by side.
//! - `manual_push_probe.rs`:
//!   the updates a manual `git push` would send,
//!   learned from Git's own dry-run report and confirmed against the remote.
//! - `manual_push_lifecycle.rs`:
//!   the manual-push candidate scope of one push:
//!   the commits and trees the destination lacks.
//! - `push_stacking.rs`:
//!   an explicit push that lost a race:
//!   run as a child,
//!   merge in what landed on the remote and publish the merge.
//! - `stacked_merge_push.rs`:
//!   the push of a merge that integrated a lost race,
//!   planned and run inside the running wrapper.
//! - `blocked_commit_lines.rs`:
//!   the lines a command prints about a branch the automatic push stops short of.
//! - `blocked_commit_questions.rs`:
//!   the questions about a repository's history that decide where the automatic push stops,
//!   in the faces that ask them.
//! - `auto_push.rs`:
//!   the plan and the words of a post-commit auto-push:
//!   the push argv,
//!   the surfaced output and the failure reporting.
//! - `push_tracking_ref_repair.rs`:
//!   the remote-tracking reference a successful push delivered,
//!   brought up to date when Git could not write it.
//! - `push_failure_notice_words.rs`:
//!   the words a found record becomes:
//!   the text of a failed push,
//!   of a push the service did not make,
//!   and of one lost with its worktree.
//! - `auto_push_awaiting.rs`:
//!   the same automatic push as a task of the runtime,
//!   and the plain-thread face that enters the runtime for it.
//! - `push_without_its_transport.rs`:
//!   the words a service push stops with when the transport settings of the command that asked for it are gone,
//!   with the one command that pushes the branch instead.
//! - `push_failure_notice.rs`:
//!   the record of a failed backup push and the rule for when a wrapper command shows it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // push/index.ts: one `export * from './file'` per module below.
//! ```

/// Controls for stopping a push on no progress,
/// with real Git and a local remote.
#[cfg(test)]
mod push_patience_tests;

/// Controls for a push that waits for a slow server while connected,
/// on probes that hold sockets and on Git over HTTP.
#[cfg(test)]
mod push_wait_tests;

/// Controls for a push whose server runs a silent check after the pack is sent,
/// over HTTP and over a local path.
#[cfg(test)]
mod push_slow_server_tests;

/// Controls for the least traffic that counts as progress,
/// on real sockets and through the push's own clock.
#[cfg(test)]
mod push_minimum_rate_socket_tests;

/// The push of a merge that integrated a lost race,
/// planned and run inside the running wrapper.
pub mod stacked_merge_push;

/// The remote-tracking reference a successful push delivered,
/// brought up to date when Git could not write it.
pub mod push_tracking_ref_repair;

/// The pause before each new try of a push the service stopped or that failed.
pub mod push_retry_pauses;

/// The shapes one round of retries is handed:
/// a branch waiting,
/// the tries due again,
/// and the change a worker asks of the plan.
pub mod push_retry_waiting_branches;

/// The service's own retry of a push it stopped or that failed:
/// the list of branches waiting for a new try,
/// and what a start rebuilds it from.
pub mod push_retry;

/// What the workers of a round asked,
/// applied to the list the service loop reads.
pub mod push_retry_changes_applied;

/// What a failed push's own words mean for that retry:
/// whether the failure is final,
/// how many tries the one counted cause gets,
/// and the words the person reads.
pub mod push_retry_failure;

/// The two signs one look at a process of a push finds,
/// and the identities a holder is remembered by.
pub mod push_progress_signs;

/// The server a push waits for,
/// named from the command's own destination.
pub mod push_server_naming;

/// The first of Git's own lines that gives a reason for a push's failure.
pub mod push_retry_cause_line;

/// The record a retry plan is kept as:
/// its version and the JSON that reads back as the plan.
pub mod push_retry_store_plan_record;

/// Reading every plan a service instance's journal holds,
/// one plan per branch and whether something could not be read.
pub mod push_retry_store_loaded;

/// The retry plan on disk,
/// in the service instance's journal,
/// so that a restarted service keeps trying the pushes it was trying.
pub mod push_retry_store;

/// The per-branch file name of the push failure notice:
/// a reversible,
/// flat,
/// case-safe key.
pub mod auto_push_branch_key;

/// The shared clock of every wait notice:
/// the grace,
/// the repeat interval and a queued commit's place in line.
pub mod wait_progress;

/// A watcher for waits that block on a child process:
/// notices at the cadence and the child stopped at the bound.
pub mod child_watch;

/// The task a watch runs,
/// the bound its end is given,
/// and the refusal a control arms.
pub mod child_watch_running_task;

/// The classes of Git step a commit starts for itself,
/// with the bound and the notice grace of each.
pub mod child_class;

/// One read-only Git command run to its end,
/// or until it has shown no sign of work for its window,
/// with the answer it printed and how it ended.
pub mod bounded_query_run_of_one_command;

/// Read-only Git queries of the auto-push,
/// each with a deadline and separate output streams.
pub mod bounded_query;

/// The thread that gives a second-pass command its request,
/// and the bounds it runs under.
pub mod bounded_query_input;

/// The auto-push's real `git push` with a deadline and its low-speed abort.
pub mod bounded_push;

/// The wait of a bounded run:
/// it ends when the run does,
/// or stops when the run shows no work,
/// floods its output or the service is stopped,
/// with the cap it is judged by and the words a wait adds.
pub mod push_wait_until_done;

/// The causes and reports of a stop,
/// and the line a stalled push prints,
/// which names the window that fired and a remedy that works.
pub mod stop_cause_and_words;

/// The stop of a running push and the sweep of the lock files it and what it started leave.
pub mod stop_of_a_push_and_sweep_of_its_locks;

/// Where a control holds a run's limit until the ordering it needs has happened,
/// so a stop is never decided by elapsed time alone.
pub mod push_limit_held_for_an_ordering;

/// What one look at a push's tree finds:
/// the bytes each process moved,
/// the processor time it used,
/// its tasks' states,
/// and the connected sockets' own counts.
pub mod push_progress_meter;

/// The clock that stops a push on a stall,
/// over what one look at its tree finds.
pub mod push_progress;

/// The least traffic a push's connection must carry in one window and still count as progress,
/// the variable that sets it and its measured default.
pub mod push_minimum_rate;

/// The connected TCP sockets of the machine with their byte counts,
/// and the sockets a process holds.
pub mod tcp_sockets;

/// The real Git of a push and its query runner.
pub mod auto_push_single_flight;

/// What one push this invocation ran left behind:
/// the attempt it ended as,
/// and the failure that stopped a push or one of its Git queries.
pub mod one_push_attempt_and_its_failure;

/// The settings that decide where one branch is pushed,
/// read from one listing of the configuration.
pub mod auto_push_destination_settings;

/// The ref `push.default` sends one branch to,
/// or the note that says why Git sends it nowhere.
pub mod auto_push_destination_by_push_default;

/// Where the configured push refspecs send one branch,
/// judged from one answer about the local refs.
pub mod auto_push_destination_by_refspecs;

/// Where a plain `git push` of one named branch goes,
/// from the configuration and the local refs.
pub mod auto_push_destination;

/// The branch a command started on,
/// watched before it runs:
/// where it pointed and the tip it held.
#[cfg(unix)]
pub mod auto_push_trigger_watched_branch;

/// The checks that say whether the moved branch grew ahead of its upstream,
/// with the walk over the commit parents and the decline a check answers with.
#[cfg(unix)]
pub mod auto_push_trigger_ahead_of_upstream;

/// The push a merge,
/// pull,
/// cherry-pick,
/// revert or am hands to the service when it leaves the branch ahead of its upstream.
#[cfg(unix)]
pub mod auto_push_trigger;

/// The record of each decision of the push trigger:
/// handed over,
/// or declined with its reason.
pub mod push_trigger_log;

/// What one reconcile run integrates,
/// where it publishes,
/// how it words itself and how it ended,
/// so the loop and the entry that reaches it read one plan.
pub mod auto_push_reconcile_plan;

/// Publishing a landed commit that a moved remote branch rejected:
/// the merge is made from objects alone and pushed.
pub mod auto_push_reconcile;

/// The reconcile loop and its merge push,
/// awaited by a task of the runtime,
/// which is the one body of those decisions.
pub mod auto_push_reconcile_awaiting;

/// What a reconcile of a rejected push shares:
/// the line prefixes,
/// the attempt limit,
/// the stale-rejection test and the small Git queries.
pub mod auto_push_reconcile_basics_and_limits;

/// The largest answer one command of the second pass may print,
/// and the ceiling each command is bounded by.
pub mod auto_push_reconcile_merge_commit_answer_ceilings;

/// The merge commit of a reconcile,
/// built in the object store without a worktree or an index.
pub mod auto_push_reconcile_merge_commit;

/// The step that moves the branch to the published merge,
/// with the window a command's own `git switch` opens in it
/// and the refusal Git answers with.
pub mod auto_push_reconcile_branch_move;

/// When the local branch may move after the merge was pushed,
/// and the record that settles a move left half done.
pub mod auto_push_reconcile_branch_advance;

/// The reading of a configured push refspec and the judgement of what the refspecs make of one branch.
pub mod auto_push_refspec_judging;

/// The deadline a push may show nothing for,
/// its default and the patience a push runs under.
pub mod push_deadline_and_patience_settings;

/// The public update of one pushed reference and the limits on the probe's side-by-side queries.
pub mod manual_push_update_and_query_limits;

/// How a stop reaches the child it started and never another command:
/// the pin of a step's root taken while that child is this process's own unreaped child,
/// the signals sent through it and the processes below the root read through it.
pub mod child_watch_signalling_only_the_child_that_was_started;

/// What a child watch is made of below its public face:
/// terms,
/// marks,
/// shared flags and the stopping of a child.
pub mod child_watch_clock_marks_and_stopping;

/// How a watched step may be ended and the errors that say it was.
pub mod child_watch_step_bounds_and_errors;

/// The loop of a child watch's task and the writing of its notices.
pub mod child_watch_task_loop_and_notice;

/// The watched one-shot run for a caller that is itself a task of the runtime.
pub mod child_watch_output_watched_on_the_runtime;

/// The one-shot run of a watched step:
/// the command started,
/// its output read whole,
/// and the watch that ends it.
pub mod child_watch_one_shot_run;

/// The paths of a dirty worktree and of a fast-forward,
/// and their overlap.
pub mod auto_push_dirty_paths;

/// Post-commit auto-push of a landed commit:
/// the push argv,
/// the surfaced output and the failure reporting.
pub mod auto_push;

/// The same automatic push as a task of the runtime.
pub mod auto_push_awaiting;

/// Whether a push Git reported as failed published the recorded branch all the same,
/// which `push.default=matching` makes possible by carrying every branch the remote has in one push.
pub mod matching_push_delivered_the_recorded_branch;

/// Where the automatic push stops when a post-commit gate kept one of a branch's commits local,
/// and the decision a verdict makes.
pub mod push_held_back_by_a_blocked_commit;

/// Where the push of one branch stops when its questions go to a Git awaited on the runtime,
/// which is the decision of the synchronous face with its waits as tasks of the runtime.
pub mod push_held_back_by_a_blocked_commit_awaiting;

/// The lines a command prints about a branch the automatic push stops short of.
pub mod blocked_commit_lines;

/// The questions about a repository's history that decide where the automatic push stops,
/// in the faces that ask them.
pub mod blocked_commit_questions;

/// Authoritative manual-push updates discovered by a private dry-run push and confirmed with ls-remote.
pub mod manual_push_porcelain;

/// Why a manual push's scope could not be established,
/// as the cause its hint is chosen by.
pub mod manual_push_cause;

/// Why the probe of a manual push could not establish the updates Git would push,
/// and what Git said when it refused.
pub mod manual_push_probe_error;

/// One destination's confirmation:
/// the updates it confirmed,
/// and everything its one `ls-remote` reported.
pub mod manual_push_probe_confirmation;

/// One destination's block of Git's own dry-run report,
/// confirmed:
/// the local objects a new ref needs,
/// the updates Git printed for that destination,
/// and the pieces that confirm every destination side by side.
pub mod manual_push_probe_destination_blocks;

/// The updates a manual `git push` would send,
/// learned from Git's own dry-run report and confirmed against the remote.
pub mod manual_push_probe;

/// The probe's Git arguments and the destination locations it confirms,
/// beside the discovery so that each file passes the package's length limit.
pub mod manual_push_probe_arguments;

/// Published-set and range computation for manual-push scans:
/// which pushed commits a remote does not have.
pub mod manual_push_range;

/// What a remote has already published:
/// the commits a prior destination value names,
/// and the tracking refs Git keeps.
pub mod manual_push_range_published_tips;

/// An explicit push that lost a race:
/// run as a child,
/// merge in what landed on the remote and publish the merge.
pub mod push_stacking;

/// Starting a child again when the operating system refuses it for lack of processes,
/// within a bounded wait.
pub mod start_retry;

/// The manual-push candidate scope of one push:
/// the commits and trees the destination lacks.
pub mod manual_push_lifecycle;

/// Whether a command's output is read by a program,
/// so that no failure notice is shown to it.
pub mod push_failure_notice_machine_output;

/// The record of a failed backup push and the rule for when a wrapper command shows it.
pub mod push_failure_notice;

/// A record read back:
/// the kind its first line names,
/// the branch,
/// the commit,
/// and the words after them.
pub mod push_failure_notice_record_parsed;

/// One record of a failed push as a command holds it,
/// with the step that shows it and removes it.
pub mod push_failure_notice_shown_then_removed;

/// What a record's text is:
/// its first line,
/// the field lines that name its branch and its commit,
/// the name it is kept under,
/// the words it carries,
/// and their cleaning.
pub mod push_failure_notice_record_text;

/// Which worktree a record belongs to:
/// the branch each worktree has checked out,
/// the rule that says a record is the asking worktree's,
/// and the way a commit is named for the person.
pub mod push_failure_notice_worktrees;

/// The records a push of a branch makes no longer true,
/// and the step that removes them once such a push succeeded.
pub mod push_failure_notice_cleared_after_a_push;

/// The name a command holds a notice's record under while it shows it,
/// and who may take one over.
pub mod push_failure_notice_claimed_by_one_command;

/// The words a found record becomes:
/// the text of a failed push,
/// of a push the service did not make,
/// and of one
///  lost with its worktree.
pub mod push_failure_notice_words;

/// The record of what a push that succeeded did,
/// and the name it is kept under.
pub mod push_failure_notice_reports;

/// The words that say how to change the push deadline,
/// for the terminal and for the resident commit service.
pub mod push_deadline_advice;

/// The words a service push stops with when the transport settings of the command that asked for it are gone,
/// with the one command that pushes the branch instead.
pub mod push_without_its_transport;
