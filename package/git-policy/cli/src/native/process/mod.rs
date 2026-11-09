//! What:
//!  Processes:
//!  identities,
//!  trees,
//!  sockets,
//!  signals and quiet-time ledgers.
//! Why:
//!  The facts about processes are shared by locks and pushes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // process/index.ts: one `export * from './file'` per module below.
//! ```

/// The longest quiet gap of every Git step and tool this process ran,
/// kept until a timing record takes it.
pub mod quiet_ledger;

/// Whether a thread of this process is at work,
/// and the clock that says when it has been quiet for a window.
pub mod thread_quiet;

/// Output of a child captured into anonymous files,
/// so that the child outlives the command that started it.
pub mod output_capture;

/// The PID namespace an owner record names,
/// and the rule for owners of another one.
pub mod pid_namespace;

/// Process-birth identities in the incumbent's exact spelling.
pub mod process_identity;

/// Process start times for lock owners,
///  read from `/proc`,
///  `ps` or the Windows process table.
pub mod process_start;

/// Catching SIGINT,
/// SIGTERM,
/// SIGHUP and SIGQUIT while a Git child runs,
/// and relaying only process-sent signals.
pub mod signal_relay;

/// The descriptor the relay sends through:
/// made once,
/// pointed at the child being waited for,
/// and the pidfd that names that child.
pub mod signal_relay_descriptor;

/// The processes of this host that use a directory or a file.
pub mod process_use;

/// The device and the inode of the file an open descriptor refers to,
/// read from the kernel's own copy of the descriptor.
pub mod open_file_of_a_descriptor;

/// The processes below one process,
/// read before it is stopped and signalled only while they are still the ones found.
pub mod process_tree;

/// Signalling one member of a tree through a descriptor opened while the number still named it,
/// and what each signal came to.
pub mod member_signalled_through_its_pidfd;

/// The number of the child a caller waits for,
/// published the moment the child starts.
pub mod child_number_told_at_start;

/// A child started with the standard library and awaited on the runtime through a pidfd,
/// owned through every failure after its start.
pub mod child_awaited_through_a_pidfd;

/// The children this command stopped waiting for,
/// gathered by a thread of their own,
/// and kept when the system refuses that thread.
pub mod stuck_children_reaped_when_the_kernel_lets_go;

/// A child waited for by the thread that owns it,
/// with a ceiling,
/// so a query that never answers is ended by its owner.
pub mod child_waited_for_within_a_ceiling;

/// A command run to its end like `Command::output`,
/// awaited on the runtime through a pidfd,
/// with its output read from anonymous files.
pub mod captured_child_run_to_its_end;

/// The output of a child the caller started,
/// read whole through pipes on the runtime while the child's exit is awaited through its pidfd.
pub mod step_output_read_whole_through_pipes;

/// When a stop that asked its tree to end may force the end:
/// at the end of its grace,
/// and not before the ceiling while a member is in uninterruptible sleep.
pub mod stop_waits_while_a_member_cannot_be_interrupted;

/// How a stop of the commit service reaches the Git processes it waits for:
/// the handler records it,
/// the shutdown tells the waiters,
/// and each waiter ends the child it owns.
pub mod stop_of_the_service_noticed_by_its_children;

/// An owned child awaited until it exits,
/// or ended and reported when the service's stop is told first.
pub mod child_awaited_until_it_exits_or_the_service_is_stopped;

/// An owned child awaited until it exits,
/// or ended and reaped here when its step's ceiling comes,
/// because a number is safe only while the child is this process's own unreaped child.
pub mod child_awaited_until_it_exits_the_stop_or_the_ceiling;
