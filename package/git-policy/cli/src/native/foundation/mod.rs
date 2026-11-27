//! What:
//!  Small shared building blocks:
//!  private files,
//!  scratch locations,
//!  JSON,
//!  time,
//!  random ids,
//!  paths and bounded parallelism.
//! Why:
//!  Subsystems depend on these and they depend on no subsystem.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `a_panic_that_ends_one_item.rs`:
//!   one item's work awaited with a panic in it kept as that item's answer,
//!   so one item's failure ends that item and not the task that ran the wave.
//! - `blocking_job.rs`:
//!   a blocking call on tokio's blocking pool,
//!   with "no thread to start" as an error value instead of a panic.
//! - `call_catching_panics.rs`:
//!   a call turned into a value when it panics,
//!   for the places that guard a function and its state.
//! - `call_catching_panics_awaiting.rs`:
//!   a future awaited with the panic of any one of its polls as an error value,
//!   for the places that guard work that awaits.
//! - `collected_elements_looked_at.rs`:
//!   the collected elements a pass over unbounded input has looked at,
//!   counted for the controls of the passes that must stay linear.
//! - `file_only_root_may_write.rs`:
//!   reading a file only the account this program runs as may write.
//!   Linux only,
//!   like the handler whose configuration it reads:
//!   the flags and the owner check are the Linux kernel's own.
//! - `file_open_for_write_by_any_process.rs`:
//!   whether any process has a file open for writing,
//!   found with a read lease that is taken and given back at once.
//! - `handler_configuration_keys.rs`:
//!   the keys the root-owned handler configuration holds,
//!   which the wrapper and the root modes both read.
//! - `hexadecimal_of_bytes.rs`:
//!   bytes as lowercase hexadecimal digits,
//!   the one encoder every record,
//!   claim name and audit line spells bytes with.
//! - `iso_time.rs`:
//!   UTC timestamps in the exact form of JavaScript's `toISOString`.
//! - `diagnostic_log.rs`:
//!   line-oriented debug and warning diagnostics on standard error.
//! - `earlier_push_programs_record.rs`:
//!   the record an unreleased branch wrote of a shell's programs,
//!   and the staging files of its writes,
//!   removed by the first command that meets them.
//! - `json_record.rs`:
//!   compact JSON records read and written with the incumbent's `JSON` semantics.
//! - `js_text.rs`:
//!   text operations with the exact semantics of JavaScript's built-ins.
//! - `jsonc_member_text.rs`:
//!   one text value inside one object of a JSONC document.
//! - `lexical_path.rs`:
//!   lexical path normalization.
//! - `line_sink.rs`:
//!   where one line of progress goes,
//!   as a trait a struct can implement.
//! - `log_generations_kept_to_a_bound.rs`:
//!   the numbered generations a log's rotations leave,
//!   the bound on their bytes and the pass that removes the oldest.
//! - `name_held_while_one_thread_publishes_it.rs`:
//!   the in-memory claim on a file name that one thread of this process is publishing by the checked rename.
//! - `notice_stream_that_does_not_hold_its_writer.rs`:
//!   a stream of a command's notices written by a thread of its own,
//!   so a reader that stopped draining never holds the command.
//! - `parallel_budget.rs`:
//!   how many workers independent items may use together,
//!   from the CPUs allowed,
//!   the memory free and a measured ceiling.
//! - `path_bytes_in_records.rs`:
//!   a path kept as text in a record,
//!   with its bytes in hexadecimal beside the plain text.
//! - `plain_file.rs`:
//!   reading a file only when it is a plain one:
//!   never waiting for a writer,
//!   never following a link,
//!   never past a limit.
//! - `common_directory_of_a_git_directory.rs`:
//!   the common Git directory a worktree's own Git directory belongs to,
//!   read from its `commondir` file.
//! - `publishing_entries_in_one_open_folder.rs`:
//!   entries created,
//!   filled and published by name below one open folder.
//! - `random_id.rs`:
//!   random identifiers in the layout of the incumbent's `randomUUID`.
//! - `ref_name.rs`:
//!   a reference name as the bytes Git printed,
//!   with the spelling each reader of it takes.
//! - `runtime_limits.rs`:
//!   what is left of the limits a process that starts threads lives under:
//!   address space,
//!   `RLIMIT_NPROC` and `pids.max`.
//! - `runtime_plan.rs`:
//!   how many workers and blocking threads a runtime gets from the budget and the limits.
//! - `scoped_start.rs`:
//!   a scoped thread,
//!   or the work done in turn where the system cannot start one.
//! - `lines_written_by_a_thread.rs`:
//!   lines a caller hands over and a thread of its own writes:
//!   the bounded queue,
//!   its thread,
//!   and the wait the end of a process makes for them.
//! - `shared_notice_stream.rs`:
//!   one stream of a command's notices,
//!   written whole line by whole line by two threads through one small lock.
//! - `side_by_side_on_the_current_task.rs`:
//!   futures the caller lends,
//!   run beside each other on the task that awaits them,
//!   for the items a drain knows only when it runs and whose work is awaited rather than run as a job.
//! - `side_by_side_threads.rs`:
//!   jobs on scoped threads beside each other or beside the calling thread,
//!   each a function and the state it reads.
//! - `parallel_runner.rs`:
//!   independent items on a bounded number of threads,
//!   reported in input order with no lock.
//! - `standard_error_descriptor.rs`:
//!   the process's own standard error descriptor,
//!   written without the lock the standard library's shared handle takes.
//! - `diagnostic_lines_off_the_calling_thread.rs`:
//!   the diagnostic lines themselves,
//!   written by a thread of their own,
//!   so a reader that stopped draining never holds the command that said them.
//! - `standard_error_off_the_calling_thread.rs`:
//!   this process's standard error as a stream whose every whole line is handed to the thread that writes it,
//!   so the thread that says a line never waits for the reader.
//! - `start_directory.rs`:
//!   the directory a command was started in,
//!   read from the process once where the command begins and handed to the code that needs it.
//! - `test_only_switches.rs`:
//!   the one read of every test-only switch,
//!   which a build the release ships holds not at all.
//! - `session_runtime_directory_and_bus_address.rs`:
//!   the one place this crate reads this session's runtime directory and its session-bus address,
//!   which a build made for tests takes from what a control handed in and never from the environment
//!   the test process inherited.
//! - `what_a_built_runtime_is.rs`:
//!   the runtime a build made,
//!   how its build came out,
//!   the time that build took,
//!   and why a future could not be run on it.
//! - `the_build_of_the_process_runtime.rs`:
//!   the process's multi-thread runtime built on a helper thread under a deadline,
//!   and the runtime a build that missed that deadline keeps.
//! - `lazy_runtime.rs`:
//!   the process's one tokio runtime:
//!   built on the first need,
//!   sized from the budget and the limits,
//!   bounded by a deadline.
//! - `runtime_probe.rs`:
//!   the test-only entry that builds the runtime and runs a blocking job,
//!   for the controls that start the executable under limits.
//! - `the_claim_a_job_takes_before_running_it.rs`:
//!   a claim that decides,
//!   once,
//!   whether the work of a job runs or its reporter may say it never did.
//! - `the_place_of_one_blocking_job.rs`:
//!   the place one blocking job holds of the process's bound,
//!   shared between the frame that waits for it and the job itself.
//! - `blocking_jobs_within_the_budget.rs`:
//!   the most blocking jobs this process runs at once,
//!   one bound for the whole process,
//!   taken by each job and given back when it ends.
//! - `what_a_failed_run_says_to_a_person.rs`:
//!   why a run of independent items on the runtime did not finish every item,
//!   in the words a person reads for each reason.
//! - `parallel_runner_on_the_runtime.rs`:
//!   independent items as blocking jobs of the process's runtime,
//!   under the process's bound on blocking jobs,
//!   reported in the same input order.
//! - `parallel_runner_on_the_runtime_awaiting.rs`:
//!   independent items as tasks of the process's runtime,
//!   for work that awaits a Git child rather than blocking on it,
//!   reported in the same input order.
//! - `side_by_side_on_the_runtime.rs`:
//!   jobs of the blocking pool beside each other or beside the calling task's own work,
//!   each a function and the state it owns,
//!   under the process's bound on blocking jobs.
//! - `time_spent_outside_the_phases.rs`:
//!   the two costs a command pays outside the phases of its timing record,
//!   and the writer that measures what it prints.
//! - `unix_socket_connect_within_a_deadline.rs`:
//!   a connection to a unix socket that is made within one deadline,
//!   so a socket whose program stopped costs its caller a bounded wait.
//! - `work_awaited_within_a_ceiling.rs`:
//!   a blocking job on a thread of its own,
//!   awaited no longer than a ceiling,
//!   because a file system call can wait in the kernel with no end.
//! - `worst_delay_seen_on_this_host.rs`:
//!   the longest delay measured on this host and the owner's multiple of it,
//!   which the held-name wait and the wait before a forced stop use.
//! - `a_record_or_folder_the_account_alone_may_read_and_write.rs`:
//!   the permission bits that keep a file or a folder to the account that made it,
//!   the folders the wrapper creates with them,
//!   and the removal of a folder tree it retires.
//! - `a_record_made_durable_with_a_sync.rs`:
//!   making a record that was already written survive a power loss:
//!   the flush of its file,
//!   the sync of the folder whose entry names it,
//!   and the switch a control makes both no-ops with.
//! - `a_record_read_with_the_reason_it_was_refused.rs`:
//!   reading a record as a regular file,
//!   never through a final link and never past the caller's limit,
//!   and the reason a refusal gives.
//! - `private_storage.rs`:
//!   writing a record whole:
//!   the staging file beside it,
//!   the exclusive publish,
//!   the rename that never replaces a name,
//!   and the names the other three files' callers reach.
//! - `private_storage_replacing_a_record.rs`:
//!   publishing a record over the one the same name already holds,
//!   in one rename,
//!   with or without making it durable.
//! - `earlier_log_lines_journal_held_and_recorded.rs`:
//!   the journal of a log fold:
//!   a round,
//!   a record's text and the journal file held with its lock.
//! - `entries_of_one_open_folder.rs`:
//!   one folder held open,
//!   and the entries below it read and listed by name.
//! - `log_lines_appended_without_a_lock.rs`:
//!   one line or entry appended to a log with one write,
//!   and the rotation that takes a full file off its name.
//! - `rename_that_never_replaces.rs`:
//!   the rename that never replaces an existing name,
//!   and the test for a filesystem that cannot do it.
//! - `scratch_entry_removal.rs`:
//!   how one scratch entry is removed within its ceiling,
//!   and which commit scratch trees a later command takes for abandoned.
//! - `wrapper_names_old_and_new.rs`:
//!   the names this program answers to and keeps on disk,
//!   each as the current spelling and the one an earlier release used.
//! - `the_lines_of_a_log_compared_and_appended.rs`:
//!   the bytes of a log as a fold of it uses them:
//!   a round's bytes and the older copies they may already be in,
//!   the one write that adds them,
//!   and how a fold of one log ended.
//! - `earlier_log_lines_added_to_current_log.rs`:
//!   the lines of a log an earlier release wrote added to the log under the current name,
//!   restartable after a kill.
//! - `earlier_folder_entries_moved_into_current_folder.rs`:
//!   the entries of a folder an earlier release wrote moved into the folder under the current name.
//! - `earlier_name_moved_to_current_in_place.rs`:
//!   where a kept file or folder is,
//!   moved in place from its earlier name on the first need.
//! - `what_a_fold_is_given_and_what_it_answers.rs`:
//!   the kinds of name a fold works on,
//!   what it reads of the process around it,
//!   and how the fold of one name ended.
//! - `what_a_fold_says_to_a_person.rs`:
//!   the two lines a fold says,
//!   one when it waits for an earlier program to stop and one when it could not finish,
//!   each said once in a process.
//! - `earlier_names_folded_into_current.rs`:
//!   every kept name that exists under both spellings folded into the current one when no earlier program writes it.
//! - `kept_names_in_the_git_directories.rs`:
//!   every kept name in a repository's Git directories settled to its current spelling,
//!   and the finder for a reader that moves nothing.
//! - `a_private_scratch_folder_and_the_stale_entries_a_sweep_removes.rs`:
//!   what any scratch folder of this wrapper must be,
//!   how old an entry must be before a preparation removes it,
//!   and the sweep that removes the stale entries of one folder.
//! - `scratch_location.rs`:
//!   where short-lived private files live:
//!   the per-user scratch directory and the repository scratch directory,
//!   never the system temporary directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // foundation/index.ts: one `export * from './file'` per module below.
//! ```

/// The permission bits that keep a file or a folder to the account that made it,
/// the folders the wrapper creates with them,
/// and the removal of a folder tree it retires.
pub mod a_record_or_folder_the_account_alone_may_read_and_write;

/// Making a record that was already written survive a power loss:
/// the flush of its file,
/// the sync of the folder whose entry names it,
/// and the switch a control makes both no-ops with.
pub mod a_record_made_durable_with_a_sync;

/// Reading a record as a regular file,
/// never through a final link and never past the caller's limit,
/// and the reason a refusal gives.
pub mod a_record_read_with_the_reason_it_was_refused;

/// Writing a record whole:
/// the staging file beside it,
/// the exclusive publish,
/// the rename that never replaces a name,
/// and the names the other three files' callers reach.
pub mod private_storage;

/// Publishing a record over the one the same name already holds,
/// in one rename,
/// with or without making it durable.
pub mod private_storage_replacing_a_record;

/// One folder held open,
/// and the entries below it read and listed by name.
pub mod entries_of_one_open_folder;

/// Entries created,
/// filled and published by name below one open folder.
pub mod publishing_entries_in_one_open_folder;

/// How one scratch entry is removed within its ceiling,
/// and which commit scratch trees a later command takes for abandoned.
pub mod scratch_entry_removal;

/// What any scratch folder of this wrapper must be:
/// the check that it is this account's private directory,
/// how old an entry must be before a preparation removes it,
/// and the sweep that removes the stale entries of one folder.
pub mod a_private_scratch_folder_and_the_stale_entries_a_sweep_removes;

/// Where short-lived private files live:
///  the per-user scratch directory and the repository scratch directory,
///  never the system temporary directory.
pub mod scratch_location;

/// The directory a command was started in,
///  read from the process once where the command begins
///  and handed to the code that needs it.
pub mod start_directory;

/// The common Git directory a worktree's own Git directory belongs to,
/// read from its `commondir` file.
pub mod common_directory_of_a_git_directory;

/// Compact JSON records read and written with the incumbent's `JSON` semantics.
pub mod json_record;

/// One text value inside one object of a JSONC document.
pub mod jsonc_member_text;

/// The keys the root-owned handler configuration holds,
/// which the wrapper and the root modes both read.
pub mod handler_configuration_keys;

/// Reading a file only the account this program runs as may write.
///
/// Linux only,
/// like the handler whose configuration it reads:
/// the flags and the owner check are the
/// Linux kernel's own.
#[cfg(target_os = "linux")]
pub mod file_only_root_may_write;

/// Random identifiers in the layout of the incumbent's `randomUUID`.
pub mod random_id;

/// Bytes as lowercase hexadecimal digits,
/// the one encoder every record,
/// claim name and audit line spells bytes with.
pub mod hexadecimal_of_bytes;

/// Text operations with the exact semantics of JavaScript's built-ins.
pub mod js_text;

/// Line-oriented debug and warning diagnostics on standard error.
pub mod diagnostic_log;

/// Lines a caller hands over and a thread of its own writes:
/// the bounded queue,
/// its thread,
/// and the wait the end of a process makes for them.
pub mod lines_written_by_a_thread;

/// The diagnostic lines themselves,
/// written by a thread of their own,
/// so a reader that stopped draining never holds the command that said them.
pub mod diagnostic_lines_off_the_calling_thread;

/// The record an unreleased branch wrote of a shell's programs,
/// and the staging files of its writes,
/// removed by the first command that meets them.
pub mod earlier_push_programs_record;

/// The two costs a command pays outside the phases of its timing record,
/// and the writer that measures what it prints.
pub mod time_spent_outside_the_phases;

/// UTC timestamps in the exact form of JavaScript's `toISOString`.
pub mod iso_time;

/// Lexical path normalization.
pub mod lexical_path;

/// Reading a file only when it is a plain one:
/// never waiting for a writer,
/// never following a link,
/// never past a limit.
pub mod plain_file;

/// A path kept as text in a record,
/// with its bytes in hexadecimal beside the plain text.
pub mod path_bytes_in_records;

/// A reference name as the bytes Git printed,
/// with the spelling each reader of it takes.
pub mod ref_name;

/// How many workers independent items may use together,
/// from the CPUs allowed,
/// the memory free and a measured ceiling.
pub mod parallel_budget;

/// Where one line of progress goes,
/// as a trait a struct can implement.
pub mod line_sink;

/// One stream of a command's notices,
/// written whole line by whole line by two threads through one small lock.
pub mod shared_notice_stream;

/// Independent items on a bounded number of threads,
/// reported in input order with no lock.
pub mod parallel_runner;

/// Why a run of independent items on the runtime did not finish every item,
/// in the words a person reads for each reason.
pub mod what_a_failed_run_says_to_a_person;

/// Independent items as blocking jobs of the process's runtime,
/// under the process's bound on blocking jobs,
/// reported in the same input order.
pub mod parallel_runner_on_the_runtime;

/// Independent items as tasks of the process's runtime,
/// for work that awaits a Git child rather than blocking on it,
/// reported in the same input order.
pub mod parallel_runner_on_the_runtime_awaiting;

/// A claim that decides,
/// once,
/// whether the work of a job runs or its reporter may say it never did.
pub mod the_claim_a_job_takes_before_running_it;

/// The place one blocking job holds of the process's bound,
/// shared between the frame that waits for it and the job itself.
pub mod the_place_of_one_blocking_job;

/// A call turned into a value when it panics,
/// for the places that guard a function and its state.
pub mod call_catching_panics;

/// A future awaited with the panic of any one of its polls as an error value,
/// for the places that guard work that awaits.
pub mod call_catching_panics_awaiting;

/// A scoped thread,
/// or the work done in turn where the system cannot start one.
pub mod scoped_start;

/// Jobs on scoped threads beside each other or beside the calling thread,
/// each a function and the state it reads.
pub mod side_by_side_threads;

/// Jobs of the blocking pool beside each other or beside the calling task's own work,
/// each a function and the state it owns,
/// under the process's bound on blocking jobs.
pub mod side_by_side_on_the_runtime;

/// Futures the caller lends,
/// run beside each other on the task that awaits them,
/// for the items a drain knows only when it runs and whose work is awaited rather than run as a job.
pub mod side_by_side_on_the_current_task;

/// One item's work awaited with a panic in it kept as that item's answer,
/// so one item's failure ends that item and not the task that ran the wave.
pub mod a_panic_that_ends_one_item;

/// The names this program answers to and keeps on disk,
///  each as the current spelling and the one an earlier release used.
pub mod wrapper_names_old_and_new;

/// The rename that never replaces an existing name,
///  and the test for a filesystem that cannot do it.
pub mod rename_that_never_replaces;

/// Where a kept file or folder is,
///  moved in place from its earlier name on the first need.
pub mod earlier_name_moved_to_current_in_place;

/// What a fold is given and what it answers:
/// the kinds of name it folds,
/// the surroundings it reads,
/// how one name's fold ended,
/// and one Git directory's report.
pub mod what_a_fold_is_given_and_what_it_answers;

/// The two lines a fold says to a person,
/// one when it waits for an earlier program to stop and one when it could not finish.
pub mod what_a_fold_says_to_a_person;

/// Every kept name in a repository's Git directories settled to its current spelling,
///  and the finder for a reader that moves nothing.
pub mod kept_names_in_the_git_directories;

/// One line or entry appended to a log with one write,
/// and the rotation that takes a full file off its name.
pub mod log_lines_appended_without_a_lock;

/// The numbered generations a log's rotations leave,
/// the bound on their bytes and the pass that removes the oldest.
pub mod log_generations_kept_to_a_bound;

/// Whether any process has a file open for writing,
/// found with a read lease that is taken and given back at once.
pub mod file_open_for_write_by_any_process;

/// The journal of a log fold:
/// a round,
/// a record's text and the journal file held with its lock.
pub mod earlier_log_lines_journal_held_and_recorded;

/// The bytes of a log as a fold of it uses them:
/// a round's bytes and the older copies they may already be in,
/// the one write that adds them,
/// and how a fold of one log ended.
pub mod the_lines_of_a_log_compared_and_appended;

/// The lines of a log an earlier release wrote added to the log under the current name,
/// restartable after a kill.
pub mod earlier_log_lines_added_to_current_log;

/// The entries of a folder an earlier release wrote moved into the folder under the current name.
pub mod earlier_folder_entries_moved_into_current_folder;

/// Every kept name that exists under both spellings folded into the current one when no earlier program writes it.
pub mod earlier_names_folded_into_current;

/// What is left of the limits a process that starts threads lives under:
/// address space,
/// `RLIMIT_NPROC` and `pids.max`.
pub mod runtime_limits;

/// How many workers and blocking threads a runtime gets from the budget and the limits.
pub mod runtime_plan;

/// What a build made of a runtime:
/// the runtime itself,
/// how its build came out,
/// the time that build took,
/// and why a future could not be run on it.
pub mod what_a_built_runtime_is;

/// The process's multi-thread runtime built on a helper thread under a deadline,
/// and the runtime a build that missed that deadline keeps.
pub mod the_build_of_the_process_runtime;

/// The process's one tokio runtime:
/// built on the first need,
/// sized from the budget and the limits,
/// bounded by a deadline.
pub mod lazy_runtime;

/// A blocking call on tokio's blocking pool,
/// with "no thread to start" as an error value instead of a panic.
pub mod blocking_job;

/// The most blocking jobs this process runs at once,
/// one bound for the whole process,
/// taken by each job and given back when it ends.
pub mod blocking_jobs_within_the_budget;

/// The test-only entry that builds the runtime and runs a blocking job,
/// for the controls that start the executable under limits.
pub mod runtime_probe;

/// The one read of every test-only switch,
/// which a build the release ships holds not at all.
pub mod test_only_switches;

/// The one place this crate reads this session's runtime directory and its session-bus address,
/// which a build made for tests takes from what a control handed in and never from the environment
/// the test process inherited.
pub mod session_runtime_directory_and_bus_address;

/// The in-memory claim on a file name that one thread of this process is publishing by the checked rename.
pub mod name_held_while_one_thread_publishes_it;

/// The longest delay measured on this host and the owner's multiple of it,
/// which the held-name wait and the wait before a forced stop use.
pub mod worst_delay_seen_on_this_host;

/// The collected elements a pass over unbounded input has looked at,
/// counted for the controls of the passes that must stay linear.
pub mod collected_elements_looked_at;

/// A blocking job on a thread of its own,
/// awaited no longer than a ceiling,
/// because a file system call can wait in the kernel with no end.
pub mod work_awaited_within_a_ceiling;

/// The process's own standard error descriptor,
/// written without the lock the standard library's shared handle takes.
pub mod standard_error_descriptor;

/// A stream of a command's notices written by a thread of its own,
/// so a reader that stopped draining never holds the command.
pub mod notice_stream_that_does_not_hold_its_writer;
/// This process's standard error as a stream whose every whole line is handed to the thread that
/// writes it,
/// so the thread that says a line never waits for the reader.
pub mod standard_error_off_the_calling_thread;

/// A connection to a unix socket that is made within one deadline,
///  so a socket whose program stopped costs its caller a bounded wait.
pub mod unix_socket_connect_within_a_deadline;
