//! The forbidden work and the justified exceptions of the scan in `blocking_code_off_the_runtime_tests.rs`.
//! What:
//!  The rules that keep blocked code off the runtime's workers,
//!  and the list of the places the move has not reached yet.
//! Why:
//!  The runtime's workers only await.
//!  A thread a task starts,
//!  a clock a task sleeps on,
//!  a child started without the retry,
//!  and a storage call
//!  or a blocking wait inside a task all hold the worker that would end the very wait they are in,
//!  which turns a bounded
//!  wait into a stall (`doc/planning/git-policy-cli-full-tokio.md`,
//! "The place a worker was held,
//! measured").

/// Where a rule reads.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Place {
    /// Every code line of the file.
    Everywhere,
    /// The body of a function that builds or names `std::process::Command`,
    /// which is where a start of a child belongs to a command.
    InACommandFunction,
    /// The body of an `async fn`,
    /// outside the closures handed to the blocking pool:
    /// the code the runtime polls.
    OnATask,
}

/// One kind of forbidden work.
pub(super) struct Rule {
    /// The name an `ALLOWED` entry gives.
    pub(super) id: &'static str,
    /// Why the work may not run where the rule reads.
    pub(super) reason: &'static str,
    /// Where the rule reads.
    pub(super) place: Place,
    /// Whether a source line holds the forbidden work;
    /// the second text is the line with the next one joined,
    /// for a call split over two lines.
    pub(super) matches: fn(&str, &str) -> bool,
}

/// Whether `line` holds any of `needles`.
fn holds_any(line: &str, needles: &[&str]) -> bool {
    return needles.iter().any(|needle| {
        return line.contains(needle);
    });
}

/// The calls on a repository or user file that can wait on a disk,
/// a lock,
/// a remote mount or a FUSE mount,
/// and so never belong on a worker.
const STORAGE_CALLS: &[&str] = &[
    "std::fs::read(",
    "std::fs::read_to_string(",
    "std::fs::read_link(",
    "std::fs::read_dir(",
    "std::fs::write(",
    "std::fs::metadata(",
    "std::fs::symlink_metadata(",
    "std::fs::remove_file(",
    "std::fs::remove_dir(",
    "std::fs::remove_dir_all(",
    "std::fs::rename(",
    "std::fs::canonicalize(",
    "std::fs::create_dir(",
    "std::fs::create_dir_all(",
    "std::fs::copy(",
    "std::fs::hard_link(",
    "std::fs::try_exists(",
    "std::fs::OpenOptions",
    "File::open(",
    "File::create(",
    "OpenOptions::new()",
    ".sync_all()",
    ".sync_data(",
];

/// The waits that block the thread they run on until something else moves,
/// which on a worker is the task that would move it.
const BLOCKING_WAITS: &[&str] = &["recv_timeout(", "thread::sleep(", "park_timeout(", "thread::park(", "futex::wait(", "Condvar"];

/// The forbidden work.
pub(super) const RULES: &[Rule] = &[
    Rule {
        id: "thread-start",
        reason: "starts a thread; work the runtime can carry is a task, and work it cannot carry is a job of the blocking pool",
        place: Place::Everywhere,
        matches: |line, _joined| {
            return holds_any(line, &["thread::spawn(", "thread::Builder", "thread::scope(", "spawn_scoped("]);
        },
    },
    Rule {
        id: "blocking-sleep",
        reason: "pauses on a clock that holds the thread it runs on; a wait the runtime can end is `tokio::time::sleep`",
        place: Place::Everywhere,
        matches: |line, _joined| {
            return holds_any(line, &["thread::sleep(", "park_timeout(", "std::thread::park("]);
        },
    },
    Rule {
        id: "unretried-start",
        reason: "starts a child without the retried start, so a start the system refuses for lack of processes fails \
                 a commit the next attempt would have landed",
        place: Place::InACommandFunction,
        matches: |_line, joined| {
            if joined.contains(".spawn_retrying(")
                || joined.contains(".output_retrying(")
                || joined.contains(".status_retrying(")
                || joined.contains("spawn_retrying_without_blocking(")
            {
                return false;
            }
            return holds_any(joined, &[".spawn()", ".output()", ".status()", ".wait_with_output()"]);
        },
    },
    Rule {
        id: "unretried-built-start",
        reason: "starts the command a builder made (`git_command`) without the retried start, so a start the system \
                 refuses for lack of processes fails what the next attempt would have landed",
        place: Place::Everywhere,
        matches: |_line, joined| {
            if !joined.contains("git_command(") || joined.contains("_retrying(") {
                return false;
            }
            return holds_any(joined, &[".spawn()", ".output()", ".status()", ".wait_with_output()"]);
        },
    },
    Rule {
        id: "storage-read-on-a-task",
        reason: "reads or changes a file on a worker, where a slow disk, a lock or a mount holds the tasks that would \
                 end the wait; a job of the blocking pool carries it instead",
        place: Place::OnATask,
        matches: |line, _joined| {
            return holds_any(line, STORAGE_CALLS);
        },
    },
    Rule {
        id: "blocking-wait-on-a-task",
        reason: "waits on a thread's clock or channel on a worker, where the task that would end the wait cannot run; \
                 the runtime's timer or an awaited channel ends it instead",
        place: Place::OnATask,
        matches: |line, joined| {
            if holds_any(line, BLOCKING_WAITS) {
                return true;
            }
            if !line.contains(".wait()") {
                return false;
            }
            // The project's own child wait is a future a task awaits, and a `select!` arm is awaited too, so only a
            // call that is neither is a wait that holds the worker.
            return !(joined.contains(".await") || joined.contains("=>"));
        },
    },
    Rule {
        id: "plain-tokio-fs",
        reason: "hands the call to the blocking pool through a path that panics where no thread can start; \
                 `run_blocking_job` answers an error there",
        place: Place::Everywhere,
        matches: |line, _joined| {
            return line.contains("tokio::fs");
        },
    },
    Rule {
        id: "block-in-place",
        reason: "panics on a current-thread runtime, which is the runtime the system selects where it refuses threads",
        place: Place::Everywhere,
        matches: |line, _joined| {
            return line.contains("block_in_place");
        },
    },
];

/// One justified exception:
/// the file's relative path,
/// the rule,
/// and why it stays.
pub(super) struct Allowed {
    /// The file's path relative to `src/native`,
    /// `/`-separated (`start_retry.rs`,
    /// or `push/start_retry.rs` for a nested one);
    /// a file name alone
    ///  excuses nothing in a subdirectory.
    pub(super) file: &'static str,
    /// The rule that file is excused from.
    pub(super) rule: &'static str,
    /// Why the work is right where it is,
    /// and what removes the entry.
    pub(super) why: &'static str,
}

/// The exceptions.
/// Remove an entry when its code goes;
/// the test fails on an entry that matches nothing.
pub(super) const ALLOWED: &[Allowed] = &[
    // The threads of a step the move has not converted yet.
    Allowed {
        file: "commit/native_commit_waiting_notice.rs",
        rule: "thread-start",
        why: "the waiting notice writes from a thread of its own until the commit ends; the notice's own task replaces it when the commit path is a task",
    },
    Allowed {
        file: "foundation/lazy_runtime.rs",
        rule: "thread-start",
        why: "the bounded build of the multi-thread runtime on a helper thread, which the plan's open choice 9 requires so a refused build cannot hang the process",
    },
    Allowed {
        file: "foundation/log_generations_kept_to_a_bound.rs",
        rule: "thread-start",
        why: "one thread trims the log generations; a job of the blocking pool replaces it",
    },
    Allowed {
        file: "foundation/notice_stream_that_does_not_hold_its_writer.rs",
        rule: "thread-start",
        why: "the writer of the notice stream, which a reader must never hold",
    },
    Allowed {
        file: "foundation/scoped_start.rs",
        rule: "thread-start",
        why: "the helper that starts a thread and runs the work in turn where no thread can start; this is the one thread the runtime step replaces last",
    },
    Allowed {
        file: "service/service_provision_slow_manager_notice.rs",
        rule: "thread-start",
        why: "the line about a service manager that has not answered is said by a thread of its own, because the D-Bus call that waits for it blocks the thread provisioning runs on and nothing else can count those seconds; a timeout around awaited manager calls replaces it when the provisioning is a task",
    },
    Allowed {
        file: "foundation/scratch_entry_removal.rs",
        rule: "thread-start",
        why: "the removal of a scratch entry on a thread it starts, awaited under a ceiling",
    },
    Allowed {
        file: "foundation/side_by_side_threads.rs",
        rule: "thread-start",
        why: "the scoped runner of the blocking rounds; the runtime's runner replaces it with the last converted call site",
    },
    Allowed {
        file: "foundation/work_awaited_within_a_ceiling.rs",
        rule: "thread-start",
        why: "a call that can wait in the kernel with no end, on a thread awaited under a ceiling",
    },
    Allowed {
        file: "landing/service_scratch_renewal.rs",
        rule: "thread-start",
        why: "the renewal of a scratch claim on its own thread while the landing proceeds",
    },
    Allowed {
        file: "markdown/markdown_linter_guard.rs",
        rule: "thread-start",
        why: "the guarded Markdown linter; its thread is also what the no-progress watcher reads",
    },
    Allowed {
        file: "worktree/add_standing_notice.rs",
        rule: "thread-start",
        why: "the line a long `git worktree add` prints while it runs, on a thread of its own so a stream nobody drains never holds the add; the add's own task replaces it when the add path is a task",
    },
    Allowed {
        file: "worktree/add_standing_notice.rs",
        rule: "blocking-sleep",
        why: "the cadence of that line, a pause of a thread that holds nothing else; `tokio::time::sleep` replaces it when the add path is a task",
    },
    Allowed {
        file: "process/open_file_of_a_descriptor_in_a_child.rs",
        rule: "thread-start",
        why: "the reaper of a child left running, which outlives the command that started it",
    },
    Allowed {
        file: "process/stuck_children_reaped_when_the_kernel_lets_go.rs",
        rule: "thread-start",
        why: "the reaper of a stopped child's whole group, which outlives the command",
    },
    Allowed {
        file: "process/thread_quiet.rs",
        rule: "thread-start",
        why: "the sampler of a thread's own signs of work, whose /proc reads are what the quiet window is judged on",
    },
    Allowed {
        file: "push/auto_push_reconcile.rs",
        rule: "thread-start",
        why: "the merge push of a command outside the runtime runs beside the one loop on a thread of its own, because it starts Git synchronously; an awaited planner for the stacked push removes the thread, the channel and its messages",
    },
    Allowed {
        file: "push/child_watch_one_shot_run.rs",
        rule: "thread-start",
        why: "a step left running is watched by a thread of its own",
    },
    Allowed {
        file: "push/manual_push_range.rs",
        rule: "thread-start",
        why: "the thread that feeds a push range on standard input",
    },
    Allowed {
        file: "repository_link_check/repository_link_check_walk.rs",
        rule: "thread-start",
        why: "the walk of the link check on its own thread, awaited under a ceiling",
    },
    Allowed {
        file: "standards_review/standards_review_run.rs",
        rule: "thread-start",
        why: "the review's contained thread, the one place the review's process-wide state is held",
    },
    Allowed {
        file: "lock/index_lock_holders.rs",
        rule: "unretried-start",
        why: "the macOS `lsof` scan and the Windows Restart Manager scan only; the owner's \"Ignore Win and mac until I say so.\" keeps them unconverted",
    },
    Allowed {
        file: "process/process_start.rs",
        rule: "unretried-start",
        why: "the macOS `ps` start and the Windows `powershell.exe` start only; the owner's \"Ignore Win and mac until I say so.\" keeps them unconverted",
    },
    // The pauses of a synchronous face, each waiting for the step that owns its conversion.
    Allowed {
        file: "foundation/name_held_while_one_thread_publishes_it.rs",
        rule: "blocking-sleep",
        why: "the macOS and Windows fallback of the name wait; on Linux the same wait is a futex",
    },
    Allowed {
        file: "invocation/forwarding.rs",
        rule: "blocking-sleep",
        why: "the forwarded command's bounded look at its child; that path builds no runtime and adds nothing to a forwarded Git call",
    },
    Allowed {
        file: "landing/service_commit_looking_for_what_landed.rs",
        rule: "blocking-sleep",
        why: "the landing loop's pause between looks; the loop becomes a task with the landing's own step",
    },
    Allowed {
        file: "landing/service_index_attempts.rs",
        rule: "blocking-sleep",
        why: "the lock wait of the landing, which the inotify watch replaces",
    },
    Allowed {
        file: "landing/service_index_hold.rs",
        rule: "blocking-sleep",
        why: "the settle pause of the index hold, one of the poll constants this step removes with its last caller",
    },
    Allowed {
        file: "landing/service_index_wait.rs",
        rule: "blocking-sleep",
        why: "the landing's index wait; a timer of the runtime replaces it with the loop",
    },
    Allowed {
        file: "landing/service_land_follow.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at the landed index",
    },
    Allowed {
        file: "landing/service_land_reference_lock.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at the reference lock",
    },
    Allowed {
        file: "lock/index_holder_wait.rs",
        rule: "blocking-sleep",
        why: "the wait for a holder of index.lock, which the inotify watch replaces",
    },
    Allowed {
        file: "process/child_awaited_through_a_pidfd.rs",
        rule: "blocking-sleep",
        why: "the pause of the pidfd look loop, for the caller that has no runtime",
    },
    Allowed {
        file: "process/child_waited_for_within_a_ceiling.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at a child on the synchronous face",
    },
    Allowed {
        file: "process/thread_quiet.rs",
        rule: "blocking-sleep",
        why: "the sampler's own pause between its /proc reads",
    },
    Allowed {
        file: "push/auto_push.rs",
        rule: "blocking-sleep",
        why: "the pause between attempts at the push lock",
    },
    Allowed {
        file: "push/bounded_query_input.rs",
        rule: "blocking-sleep",
        why: "the pause between feeds of a query's standard input",
    },
    Allowed {
        file: "push/start_retry.rs",
        rule: "blocking-sleep",
        why: "the one pause the plan keeps: the start retry of the forwarded command, which builds no runtime",
    },
    Allowed {
        file: "service/service_is_gone_after_a_stop.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at a stopped service's end, on the synchronous sweep and removal commands, which build no runtime",
    },
    Allowed {
        file: "service/service_client.rs",
        rule: "blocking-sleep",
        why: "the client's pause between connects, on the synchronous commit command",
    },
    Allowed {
        file: "service/service_client_reach_service.rs",
        rule: "blocking-sleep",
        why: "the client's pause before it asks the unit manager for the service",
    },
    Allowed {
        file: "service/service_provision_earlier_service_retirement.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at an earlier service's retirement",
    },
    Allowed {
        file: "transaction/transaction_phase.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at a phase's release, on the synchronous face",
    },
    Allowed {
        file: "worktree/worktree_snapshot_ahead_claim.rs",
        rule: "blocking-sleep",
        why: "the pause between looks at the copy the commit service is making, on the add's own synchronous face; the ceiling ends the wait and no worker of the runtime is held by it",
    },
    // One call on a known small file inside a task, which the plan allows once and the merge's own step batches.
    Allowed {
        file: "merge/merge_isolation_carry_place.rs",
        rule: "storage-read-on-a-task",
        why: "one metadata look, one rename and one removal of a known entry; the plan's rule batches such calls on the blocking pool, and the merge's carry step owns that conversion",
    },
    Allowed {
        file: "merge/merge_isolation_changes.rs",
        rule: "storage-read-on-a-task",
        why: "one metadata look and one link read of a known path; the merge's own step owns batching them on the blocking pool",
    },
    Allowed {
        file: "merge/merge_isolation_kept_copies_decisions.rs",
        rule: "storage-read-on-a-task",
        why: "a metadata look at the copy and one at the path it belongs to, once per kept copy; the merge's own step owns batching them on the blocking pool",
    },
    Allowed {
        file: "merge/merge_isolation_recovery_records.rs",
        rule: "storage-read-on-a-task",
        why: "one read of a known merge record, once per recovery; the merge's own step owns batching it on the blocking pool",
    },
    Allowed {
        file: "merge/merge_isolation_recovery_steps.rs",
        rule: "storage-read-on-a-task",
        why: "one metadata look at a known worktree path; the merge's own step owns batching it on the blocking pool",
    },
    Allowed {
        file: "merge/merge_isolation_take_asides.rs",
        rule: "storage-read-on-a-task",
        why: "one metadata look, one link read and one removal of a known entry; the merge's own step owns batching them on the blocking pool",
    },
    Allowed {
        file: "merge/merge_second_pass_tree.rs",
        rule: "storage-read-on-a-task",
        why: "one removal of a known private index file; the merge's own step owns batching it on the blocking pool",
    },
    Allowed {
        file: "merge/merge_second_pass_worktree.rs",
        rule: "storage-read-on-a-task",
        why: "one read of the worktree's HEAD, once per second pass; the merge's own step owns batching it on the blocking pool",
    },
];
