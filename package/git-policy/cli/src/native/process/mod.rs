//! What:
//!  Processes:
//!  identities,
//!  trees,
//!  sockets,
//!  signals and quiet-time ledgers.
//! Why:
//!  The facts about processes are shared by locks and pushes.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `descriptor_record.rs`:
//!   one descriptor's own record,
//!   opened only when the text of its link says it can be the file a scan looks for,
//!   and never past a bound.
//! - `open_file_of_a_descriptor.rs`:
//!   the device and the inode of the file an open descriptor refers to,
//!   read from the kernel's own copy of the descriptor.
//! - `output_capture.rs`:
//!   output of a child captured into anonymous files,
//!   so that the child outlives the command that started it.
//! - `pid_namespace.rs`:
//!   the PID namespace an owner record names,
//!   and the rule for owners of another one.
//! - `process_identity.rs`:
//!   process-birth identities in the incumbent's exact spelling.
//! - `process_start.rs`:
//!   process start times for lock owners,
//!   read from `/proc`,
//!   `ps` or the Windows process table.
//! - `process_tree.rs`:
//!   the processes below one process,
//!   read before it is stopped and signalled only while they are still the ones found.
//! - `member_signalled_through_its_pidfd.rs`:
//!   signalling one member of a tree through a descriptor opened while the number still named it,
//!   and what each signal came to.
//! - `child_number_told_at_start.rs`:
//!   the number of the child a caller waits for,
//!   published the moment the child starts.
//! - `process_use_targets_and_answers.rs`:
//!   what a scan of this host's processes is asked for,
//!   and what it answers with.
//! - `descriptor_of_a_process_that_may_hold_the_file.rs`:
//!   whether one descriptor of one process is the file a scan looks for.
//! - `process_use.rs`:
//!   the processes of this host that use a directory or a file.
//! - `quiet_ledger.rs`:
//!   the longest quiet gap of every Git step and tool this process ran,
//!   kept until a timing record takes it.
//! - `signal_relay_descriptor.rs`:
//!   the descriptor the relay sends through:
//!   made once,
//!   pointed at the child being waited for,
//!   and the pidfd that names that child.
//! - `signal_relay_without_relay.rs`:
//!   the guard of a system that has no signal relay,
//!   which keeps the relay's name at every call site.
//! - `signal_report.rs`:
//!   what the kernel's own report of a signal says:
//!   whether a process sent it,
//!   and which one,
//!   which the relay and the wait both read.
//! - `stop_waits_while_a_member_cannot_be_interrupted.rs`:
//!   when a stop that asked its tree to end may force the end:
//!   at the end of its grace,
//!   and not before the ceiling while a member is in uninterruptible sleep.
//! - `stop_of_the_service_noticed_by_its_children.rs`:
//!   how a stop of the commit service reaches the Git processes it waits for:
//!   the handler records it,
//!   the shutdown tells the waiters,
//!   and each waiter ends the child it owns.
//! - `stop_word.rs`:
//!   a stop's signal and the process the kernel named as its sender,
//!   kept as one word so a delivery or a kill cannot pair one stop's signal with another's sender.
//! - `signal_relay_relayed_stop.rs`:
//!   the stop signal this process relayed to a child,
//!   and the process the kernel named as its sender,
//!   kept for the command that waits after it.
//! - `stuck_children_reaped_when_the_kernel_lets_go.rs`:
//!   the children this command stopped waiting for,
//!   gathered by a thread of their own,
//!   and kept when the system refuses that thread,
//!   whether the starter still holds a `Child` the standard library made or only the number a raw fork left.
//! - `child_waited_for_within_a_ceiling.rs`:
//!   a child waited for by the thread that owns it,
//!   with a ceiling,
//!   so a query that never answers is ended by its owner.
//! - `child_awaited_through_a_pidfd.rs`:
//!   a child started with the standard library and awaited on the runtime through a pidfd,
//!   owned through every failure after its start.
//! - `captured_child_run_to_its_end.rs`:
//!   a command run to its end like `Command::output`,
//!   awaited on the runtime through a pidfd,
//!   with its output read from anonymous files.
//! - `child_awaited_until_it_exits_or_the_service_is_stopped.rs`:
//!   an owned child awaited until it exits,
//!   or ended and reported when the service's stop is told first.
//! - `open_file_of_a_descriptor_in_a_child.rs`:
//!   the same attributes of the file of another process's descriptor,
//!   read in a child that the caller stops waiting for at a ceiling,
//!   for a mount whose file system this process cannot ask in its own name.
//! - `step_output_pipes_attached_before_the_child_starts.rs`:
//!   the read ends of a child's standard output and standard error,
//!   made as pipes and registered with the runtime before the child starts.
//! - `step_output_held_open_by_a_program_the_step_started.rs`:
//!   what a person is told when a program the step started still holds one of its streams open after it ended,
//!   and the error that carries those words.
//! - `step_output_ceiling_carried_by_the_wait.rs`:
//!   the ceiling a wait for a step's output carries,
//!   and which of the two enforcers ends the step at it.
//! - `step_output_read_whole_through_pipes.rs`:
//!   the output of a child the caller started,
//!   read whole through pipes on the runtime while the child's exit is awaited through its pidfd.
//! - `thread_quiet.rs`:
//!   whether a thread of this process is at work,
//!   and the clock that says when it has been quiet for a window.
//! - `wait_after_the_stop_saying_the_kernel_holds_it.rs`:
//!   the wait for a step a ceiling stopped and the kernel has not released:
//!   the reap of that step,
//!   and the line that says it for as long as the wait lasts.
//! - `waiting_for_a_step_the_kernel_holds.rs`:
//!   the wait for a step the kernel holds in a call a signal cannot end:
//!   its bound,
//!   the one line that says why it lasts,
//!   and its three ends.
//! - `child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`:
//!   an owned child awaited until it exits,
//!   or ended and reaped here when its step's ceiling comes,
//!   because a number is safe only while the child is this process's own unreaped child.
//! - `signal_relay_child_published_to_the_handler.rs`:
//!   the child this process publishes to the relay's handler:
//!   the slots a signal caught before that child exists waits in,
//!   and the publishing the handler reaches it through.
//! - `signal_relay.rs`:
//!   catching SIGINT,
//!   SIGTERM,
//!   SIGHUP and SIGQUIT while a Git child runs,
//!   and relaying only process-sent signals.
//! - `signal_relay_relayed_child.rs`:
//!   a child started by this process and published to the relay,
//!   so a caller's own bounded wait still relays the process's signals.
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

/// The child this process publishes to the relay's handler:
/// the slots a signal caught before that child exists waits in,
/// and the publishing the handler reaches it through.
pub mod signal_relay_child_published_to_the_handler;

/// The stop signal this process relayed to a child,
/// and the process the kernel named as its sender,
/// kept for the command that waits after it.
pub mod signal_relay_relayed_stop;

/// A stop's signal and the process the kernel named as its sender,
/// kept as one word so a delivery or a kill cannot pair one stop's signal with another's sender.
pub mod stop_word;

/// What the kernel's own report of a signal says:
/// whether a process sent it,
/// and which one,
/// which the relay and the wait both read.
pub mod signal_report;

/// A child started by this process and published to the relay,
/// so a caller's own bounded wait still relays the process's signals.
pub mod signal_relay_relayed_child;

/// The guard of a system that has no signal relay,
/// which keeps the relay's name at every call site.
#[cfg(not(unix))]
pub mod signal_relay_without_relay;

/// The descriptor the relay sends through:
/// made once,
/// pointed at the child being waited for,
/// and the pidfd that names that child.
pub mod signal_relay_descriptor;

/// One descriptor's own record,
/// opened only when the text of its link says it can be the file a scan looks for,
/// and never past a bound.
pub mod descriptor_record;

/// The processes of this host that use a directory or a file.
pub mod process_use;

/// What a scan of this host's processes is asked for,
/// and what it answers with.
pub mod process_use_targets_and_answers;

/// Whether one descriptor of one process is the file a scan looks for.
pub mod descriptor_of_a_process_that_may_hold_the_file;

/// The device and the inode of the file an open descriptor refers to,
/// read from the kernel's own copy of the descriptor.
pub mod open_file_of_a_descriptor;

/// The same attributes of the file of another process's descriptor,
/// read in a child that the caller stops waiting for at a ceiling,
/// for a mount whose file system this process cannot ask in its own name.
pub mod open_file_of_a_descriptor_in_a_child;

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
/// and kept when the system refuses that thread,
/// whether the starter still holds a `Child` the standard library made
/// or only the number a raw fork left.
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

/// The read ends of a child's standard output and standard error,
/// made as pipes and registered with the runtime before the child starts.
pub mod step_output_pipes_attached_before_the_child_starts;

/// What a person is told when a program the step started still holds one of its streams open after it ended,
/// and the error that carries those words.
pub mod step_output_held_open_by_a_program_the_step_started;

/// The ceiling a wait for a step's output carries,
/// and which of the two enforcers ends the step at it.
pub mod step_output_ceiling_carried_by_the_wait;

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

/// The wait for a step the kernel holds in a call a signal cannot end:
///  its bound,
///  the one line that says why it lasts,
///  and its three ends.
pub mod waiting_for_a_step_the_kernel_holds;

/// An owned child awaited until it exits,
/// or ended and reaped here when its step's ceiling comes,
/// because a number is safe only while the child is this process's own unreaped child.
pub mod child_awaited_until_it_exits_the_stop_or_the_ceiling;

/// The wait for a step a ceiling stopped and the kernel has not released:
/// the reap of that step,
/// and the line that says it for as long as the wait lasts.
pub mod wait_after_the_stop_saying_the_kernel_holds_it;
