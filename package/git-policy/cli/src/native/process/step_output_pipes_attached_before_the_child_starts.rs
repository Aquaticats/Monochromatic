//! What:
//!  The read ends of a child's standard output and standard error,
//!  made as pipes,
//!  registered with the runtime's reactor,
//!  and handed to a command as its two streams.
//! Why:
//!  The read of a step's output needs the child's writes to arrive through pipes this process made,
//!  registered with the reactor before the child starts,
//!  so a refusal leaves no child behind.
//!  `rust/max-lines` splits the read of a step's output by role,
//!  and making the pipes is the role that runs before the start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pipes = await attachPipes(command);
//! releaseWriteEnds(command);
//! ```

use crate::lazy_runtime::run_on_runtime;
use std::os::fd::OwnedFd;
use std::process::{
    Command,
    Stdio,
};
use tokio::net::unix::pipe::Receiver;

/// The read ends of a child's standard output and standard error,
/// registered with the runtime's reactor.
pub struct StepPipes {
    /// What the child writes to its standard output.
    pub(super) standard_output: Receiver,
    /// What the child writes to its standard error.
    pub(super) standard_error: Receiver,
}

/// What:
///  Makes the two pipes,
/// registers their read ends with the runtime (building it on the first need) and hands their write ends to
///  `command` as its standard output and standard error.
/// Why:
///  Nothing here starts a child,
/// so a failure is plain and leaves nothing running.
/// A call from inside the runtime is refused,
///  because the later wait enters the runtime and would never end from within it.
/// After the start the caller must call
///  `release_write_ends`,
/// or the readers never see the end of the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipes = await attachPipes(command); // throws before any child exists
/// ```
pub fn attach_pipes(command: &mut Command) -> std::io::Result<StepPipes> {
    return match run_on_runtime(attach_pipes_awaiting(command)) {
        Ok(opened) => opened,
        Err(refusal) => Err(std::io::Error::other(refusal.to_string())),
    };
}

/// What:
///  `attach_pipes` as the future a task of the runtime awaits.
/// Why:
///   The read ends must be registered with the reactor of the runtime the wait will run on,
/// and a caller that is already a task has one;
///  entering the runtime from inside it would never end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipes = await attachPipes(command);
/// ```
pub async fn attach_pipes_awaiting(command: &mut Command) -> std::io::Result<StepPipes> {
    let (output_reader, output_writer) = std::io::pipe()?;
    let (error_reader, error_writer) = std::io::pipe()?;
    let pipes: StepPipes = StepPipes {
        standard_output: Receiver::from_owned_fd(OwnedFd::from(output_reader))?,
        standard_error: Receiver::from_owned_fd(OwnedFd::from(error_reader))?,
    };
    command
        .stdout(Stdio::from(output_writer))
        .stderr(Stdio::from(error_writer));
    return Ok(pipes);
}

/// What:
///  Drops the write ends that `attach_pipes` gave `command`,
/// by replacing its standard output and standard error.
/// Why:
///  The command holds them until it is changed or dropped;
/// while this process holds a write end,
/// the readers wait for an end of
///  output that never comes.
/// Call it after the start,
/// whether or not the start worked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// releaseWriteEnds(command);
/// ```
pub fn release_write_ends(command: &mut Command) {
    command.stdout(Stdio::null()).stderr(Stdio::null());
}
