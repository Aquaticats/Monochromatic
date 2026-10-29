//! Run one control request:
//!  immediately,
//!  or as a timed sequence that answers when it ends.

/// Standard library items.
use std::sync::mpsc::SyncSender;

/// Sibling modules of this crate.
use crate::{control::execute, protocol::{Command, Response}, state::Compositor, timeline, timeline_plan::plan};

/// Execute a command and answer on its reply channel.
///
/// What:
///      `pub fn run(state: &mut Compositor, command: Command, reply: SyncSender<Response>)`.
///  A command the planner turns into steps starts a timeline that answers at its end;
///  an invalid plan answers `err` at once;
///  every other command runs now.
/// Why:
///       A key held for a second must not stop the event loop that serves the client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(state: Compositor, command: Command, reply: (r: Response) => void): void { ... }
/// ```
pub fn run(state: &mut Compositor, command: Command, reply: SyncSender<Response>) {
    match plan(&command) {
        Some(Ok(steps)) => timeline::start(state, steps, reply),
        Some(Err(message)) => {
            let _ = reply.send(Response::Err(message));
        }
        None => {
            let response = execute(state, command);
            let _ = reply.send(response);
        }
    }
}
