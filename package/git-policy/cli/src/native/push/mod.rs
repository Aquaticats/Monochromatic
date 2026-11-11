//! What:
//!  Pushes:
//!  the automatic push after a commit,
//!  manual push handling,
//!  bounded Git queries,
//!  retries and progress watching.
//! Why:
//!  Push has a long lifecycle with many waits.
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

/// The push of a merge that integrated a lost race,
/// planned and run inside the running wrapper.
pub mod stacked_merge_push;

/// The service's own retry of a push it stopped or that failed:
/// the pauses,
/// the waiting branches,
/// the final failure.
pub mod push_retry;

/// The server a push waits for,
/// named from the command's own destination.
pub mod push_server_naming;

/// The first of Git's own lines that gives a reason for a push's failure.
pub mod push_retry_cause_line;

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

/// Read-only Git queries of the auto-push,
/// each with a deadline and separate output streams.
pub mod bounded_query;

/// The thread that gives a second-pass command its request,
/// and the bounds it runs under.
pub mod bounded_query_input;

/// The auto-push's real `git push` with a deadline and its low-speed abort.
pub mod bounded_push;

/// The causes and reports of a stop,
/// and the line a stalled push prints,
/// which names the window that fired and a remedy that works.
pub mod stop_cause_and_words;

/// The stop of a running push and the sweep of the lock files it and what it started leave.
pub mod stop_of_a_push_and_sweep_of_its_locks;

/// Where a control holds a run's limit until the ordering it needs has happened,
/// so a stop is never decided by elapsed time alone.
pub mod push_limit_held_for_an_ordering;

/// The bytes a push and its processes move,
/// sampled,
/// and the clock that stops it on a stall.
pub mod push_progress;

/// The connected TCP sockets of the machine with their byte counts,
/// and the sockets a process holds.
pub mod tcp_sockets;

/// The real Git of a push,
/// its query runner and the failure types of a push.
pub mod auto_push_single_flight;

/// Where a plain `git push` of one named branch goes,
/// from the configuration and the local refs.
pub mod auto_push_destination;

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

/// The merge commit of a reconcile,
/// built in the object store without a worktree or an index.
pub mod auto_push_reconcile_merge_commit;

/// The move of the local branch after the merge was pushed,
/// and the checks that guard it.
pub mod auto_push_reconcile_branch_advance;

/// The reading of a configured push refspec and the judgement of what the refspecs make of one branch.
pub mod auto_push_refspec_judging;

/// The deadline a push may show nothing for,
/// its default and the patience a push runs under.
pub mod push_deadline_and_patience_settings;

/// The public update of one pushed reference and the limits on the probe's side-by-side queries.
pub mod manual_push_update_and_query_limits;

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

/// Where the automatic push stops when a post-commit gate kept one of a branch's commits local,
/// and the decision a verdict makes.
pub mod push_held_back_by_a_blocked_commit;

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

/// The updates a manual `git push` would send,
/// learned from Git's own dry-run report and confirmed against the remote.
pub mod manual_push_probe;

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

/// One record of a failed push as a command holds it,
/// with the step that shows it and removes it.
pub mod push_failure_notice_shown_then_removed;

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
