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
//! In TS you'd write (pseudocode):
//! ```ts
//! // foundation/index.ts: one `export * from './file'` per module below.
//! ```

/// Private,
///  crash-safe file primitives shared by every durable record.
pub mod private_storage;

/// How one scratch entry is removed within its ceiling,
/// and which commit scratch trees a later command takes for abandoned.
pub mod scratch_entry_removal;

/// Where short-lived private files live:
///  the per-user scratch directory and the repository scratch directory,
///  never the system temporary directory.
pub mod scratch_location;

/// The common Git directory a worktree's own Git directory belongs to,
/// read from its `commondir` file.
pub mod common_directory_of_a_git_directory;

/// Compact JSON records read and written with the incumbent's `JSON` semantics.
pub mod json_record;

/// Random identifiers in the layout of the incumbent's `randomUUID`.
pub mod random_id;

/// Text operations with the exact semantics of JavaScript's built-ins.
pub mod js_text;

/// Line-oriented debug and warning diagnostics on standard error.
pub mod diagnostic_log;

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

/// The names this program answers to and keeps on disk,
///  each as the current spelling and the one an earlier release used.
pub mod wrapper_names_old_and_new;

/// The rename that never replaces an existing name,
///  and the test for a filesystem that cannot do it.
pub mod rename_that_never_replaces;

/// Where a kept file or folder is,
///  moved in place from its earlier name on the first need.
pub mod earlier_name_moved_to_current_in_place;

/// Every kept name in a repository's Git directories settled to its current spelling,
///  and the finder for a reader that moves nothing.
pub mod kept_names_in_the_git_directories;

/// Whether any process has a file open for writing,
/// found with a read lease that is taken and given back at once.
pub mod file_open_for_write_by_any_process;

/// The journal of a log fold:
/// a round,
/// a record's text and the journal file held with its lock.
pub mod earlier_log_lines_journal_held_and_recorded;

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
