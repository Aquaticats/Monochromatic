//! Validates descriptor-keeper process identity before lifecycle signals.

/// The bounded waits and the signals of a stop.
#[path = "keeper_process_bounded_signals.rs"]
mod bounded_signals;
/// The identity a lifecycle signal rests on.
#[path = "keeper_process_identity.rs"]
mod process_identity;
/// Stopping one command and the group it left.
#[path = "keeper_process_stopping.rs"]
mod stopping_a_process;

/// Reports whether exact command remains live and rejects PID reuse or command mismatch.
pub use process_identity::command_process_is_live;
/// Reports whether exact holder remains live and rejects PID reuse or command mismatch.
pub use process_identity::keeper_is_live;
/// Reads Linux process start-time field after safely skipping parenthesized command name.
pub use process_identity::process_start_time;
/// What ending the group of a watcher whose leader is gone did.
pub use stopping_a_process::LeftoverGroup;
/// Ends the process group a watcher led when the watcher itself is gone.
pub use stopping_a_process::end_leftover_group;
/// Stops exact command after typed validation and bounded signal escalation.
pub use stopping_a_process::stop_command_process;
/// Sends SIGTERM,
///  escalates exact surviving holder to SIGKILL,
///  and confirms disappearance.
pub use stopping_a_process::stop_process;
