//! What:
//!  The question a commit asks its service before it builds anything:
//!  has a finished shadow been drawn ahead of me,
//!  and where is it.
//! Why:
//!  The owner chose the service to rename a pool member into place ahead of a commit
//!  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
//!  "The design the owner's decision made"),
//!  and what a commit pays for that is one question on a connection of its own.
//!  The whole exchange is bounded by `DRAWN_ANSWER_DEADLINE`,
//!  which the record's measured answers make a hundred and fifty times the largest one seen,
//!  and every way it can end is the same for the commit:
//!  a socket nobody listens on,
//!  a service that does not answer,
//!  a stock that is empty,
//!  and an answer that names a tree this invocation cannot use
//!  all leave the commit to take the pool's member or make its own creation,
//!  which is what it did before the pool existed.
//!  Nothing here says anything on the terminal:
//!  a commit that fell back is slower for a reason the person cannot act on,
//!  and the record of its build names the stretches that were slow.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = askForADrawnShadow(address, template, scratch); // the path, or undefined
//! ```

/// One line of the protocol in each direction.
use crate::service_protocol::{ClientMessage, ServiceMessage, encode_client};
/// Where the service listens.
use crate::service_server_socket_binding::Bind;
/// The facts a creation is made from.
use crate::shadow_create::ShadowTemplate;
/// Whether an answer names a tree of this worktree's drawn folder.
use crate::shadow_drawn::is_a_drawn_tree;
/// What a connection to a socket that takes no connection costs.
use crate::unix_socket_connect_within_a_deadline::connect_within;
/// A reader of the answers,
/// and the failure of a socket that closed.
use crate::service_client_listen_and_ask_to_retire::read_message;
/// The bytes of one line.
use std::io::Write;
/// The connection and the reader over it.
use std::io::BufReader;
use std::os::unix::net::UnixStream;
/// The socket,
/// the deadline and the path an answer names.
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The span one ask has,
/// from the connection to the last byte of the answer.
/// Measured:
///  a stand-in service answered the question this replaces in 0.043 ms at the median,
///  0.069 ms at the ninetieth percentile,
///  0.135 ms at the ninety-ninth and 1.684 ms at the largest of 3,000 round trips
///  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
///  "A. The service hands a drawn tree out"),
///  so this is a hundred and fifty times the largest answer seen:
///  it covers a stalled listing or a busy service and stays far inside the owner's five seconds,
///  and a commit that waits past it takes today's path rather than waiting for an answer that is not coming.
pub const DRAWN_ANSWER_DEADLINE: Duration = Duration::from_millis(250);

/// What:
///  Asks the service for the finished shadow it drew ahead of this commit,
///  and answers where it is,
///  or nothing when the service holds none or did not answer.
/// Why:
///  The ask is an offer and never a step the commit needs,
///  so every ending other than a tree this invocation can use is silence:
///  a socket nothing listens on is answered without a wait,
///  the exchange is bounded as a whole,
///  and an answer naming a path outside this worktree's drawn folder is refused here
///  even though the service is this wrapper's own,
///  because a commit removes the tree it was handed and must never remove one the wrapper did not make.
///  A socket with no name in the filesystem is not asked:
///  this release serves one with a name under the runtime directory,
///  and a socket that is not checked before it is connected to would be started by the connection itself,
///  which would put a service start on a commit's build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = askForADrawnShadow(address, template, scratch);
/// ```
pub fn ask_for_a_drawn_shadow(address: &Bind, template: &ShadowTemplate, scratch: &Path) -> Option<PathBuf> {
    let socket: &Path = match address {
        Bind::Path(path) => path.as_path(),
        // A connection to an abstract socket cannot be checked first, so nothing is asked of one.
        Bind::Abstract(_) | Bind::Inherited => return None,
    };
    // A socket file that is not there is no service, and asking would start one: the commit keeps today's path.
    if !socket.exists() {
        return None;
    }
    let deadline: Instant = Instant::now() + DRAWN_ANSWER_DEADLINE;
    let line: String = encode_client(&ClientMessage::Shadow { git_dir: template.git_dir.clone(), scratch: scratch.to_path_buf() });
    let mut reader: BufReader<UnixStream> = BufReader::new(connected_within(socket, deadline)?);
    let _greeting: ServiceMessage = answered_within(&mut reader, deadline)?;
    let _set: std::io::Result<()> = reader.get_mut().set_write_timeout(Some(left_of(deadline)?));
    if reader.get_mut().write_all(format!("{line}\n").as_bytes()).is_err() {
        return None;
    }
    let answer: ServiceMessage = answered_within(&mut reader, deadline)?;
    let ServiceMessage::Drawn { tree } = answer else { return None };
    if !is_a_drawn_tree(tree.as_path(), template.common_dir.as_path(), template.git_dir.as_path()) {
        crate::diagnostic_log::debug("shadowDrawn", format!("refused the answer {}: it is not a drawn shadow of this worktree", tree.display()).as_str());
        return None;
    }
    return Some(tree);
}

/// What:
///  A connection to the service's socket made within the deadline.
/// Why:
///  A socket whose listener is stopped takes no connection and refuses nothing,
///  and the ask is bounded from its first byte, not from its first answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stream = connectedWithin(socket, deadline); // undefined when the deadline passed
/// ```
fn connected_within(socket: &Path, deadline: Instant) -> Option<UnixStream> {
    return connect_within(socket, left_of(deadline)?).ok();
}

/// What:
///  One message of the service read within the deadline.
/// Why:
///  A service that greets and then says nothing is the same to a commit as no service,
///  and its read timeout is what ends the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = answeredWithin(reader, deadline); // undefined when nothing came in time
/// ```
fn answered_within(reader: &mut BufReader<UnixStream>, deadline: Instant) -> Option<ServiceMessage> {
    let left: Duration = left_of(deadline)?;
    let _set: std::io::Result<()> = reader.get_ref().set_read_timeout(Some(left));
    return read_message(reader);
}

/// What:
///  What is left of the ask's deadline,
///  and nothing when it has passed.
/// Why:
///  Every step shares the one span,
///  so a slow connection cannot be followed by a slow read that doubles it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = (deadline) => deadline - Date.now() || undefined;
/// ```
fn left_of(deadline: Instant) -> Option<Duration> {
    let left: Duration = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return None;
    }
    return Some(left);
}

/// The ask is controlled in the test module.
#[cfg(test)]
#[path = "service_client_shadow_hand_out_tests.rs"]
mod tests;
