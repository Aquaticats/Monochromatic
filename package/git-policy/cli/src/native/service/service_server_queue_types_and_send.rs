//! What:
//!  The shapes the commit service's connection threads and landing thread pass between them (a queued request,
//! the key that
//!  groups requests,
//! a job),
//! the grouping of queued requests,
//! and the one-message send to a client.
//! Why:
//!  The connection handler and the group lander both take part in the queue and answer clients;
//! keeping these in a file that
//!  uses neither lets each of them use it without using the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // queue.ts: type Queued, GroupKey, Job; function placeInGroups(); function send(stream, message)
//! ```

use crate::service_timing::Arrival;
use crate::service_reply::Reply;
use crate::service_protocol::{
    CommitRequest, ServiceMessage, encode_service,
};
/// A request waiting to be landed:
/// what the client sent,
/// the connection to answer it on,
/// and how it arrived.
pub(crate) type Queued = (Box<CommitRequest>, Reply, Arrival);

/// What a landing group shares:
/// the common Git directory,
/// the queue reference,
/// and whether its request lands alone.
pub(crate) type GroupKey = (std::path::PathBuf, String, bool);

/// What:
///  Puts `member` into the latest group of its common Git directory and queue reference,
/// or into a group of its own when
///  it lands alone,
/// when that group's request does,
/// or when no group of that branch exists yet.
/// Why:
///   Requests of one branch that may share a batch land together and in arrival order;
/// an amend is a group of its own and
///  what arrives after it starts another group,
/// so the order on one branch is the order of arrival,
/// and a request never
///  lands in a batch of another branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const joined = groups.findLast((group) => group.common === key.common && group.reference === key.reference)
/// if (joined && !key.alone && !joined.alone) { joined.members.push(member) } else { groups.push({ key, members: [member] }) }
/// ```
pub(crate) fn place_in_groups<T>(groups: &mut Vec<(GroupKey, Vec<T>)>, key: GroupKey, member: T) {
    // The newest group of the same repository and reference, if any.
    let mut joins: Option<&mut (GroupKey, Vec<T>)> = None;
    for entry in groups.iter_mut().rev() {
        if entry.0.0 == key.0 && entry.0.1 == key.1 {
            joins = Some(entry);
            break;
        }
    }
    match joins {
        Some((existing, members)) if !key.2 && !existing.2 => members.push(member),
        _ => groups.push((key, vec![member])),
    }
}

pub(crate) enum Job {
    Commit(Box<CommitRequest>, Reply, Arrival),
    Retire,
    /// The client stopped waiting for the commit of this identity (its ceiling passed):
    /// answered with the landed commit,
    /// or
    ///  withdrawn so that it never lands.
    Withdraw(String, Reply),
    /// After-commit work was announced;
    /// it only restarts the idle clock.
    Activity,
}

/// Writes `message` and a newline to `stream`;
/// a client that went away has nothing to be told.
pub(crate) async fn send<Stream: tokio::io::AsyncWrite + Unpin>(stream: &mut Stream, message: &ServiceMessage) {
    use tokio::io::AsyncWriteExt;
    let mut line: String = encode_service(message);
    line.push('\n');
    let _written = stream.write_all(line.as_bytes()).await;
}

/// What:
///  The index file a queued request names.
/// Why:
///  The landing loop reads it from the first request of a group to place the next batch's marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexOfQueued = ([request]: Queued) => request.index;
/// ```
pub(crate) fn index_of_queued(queued: &Queued) -> std::path::PathBuf {
    return queued.0.index.clone();
}

/// What:
///  The identity of a queued request.
/// Why:
///  The landing loop lists the identities of a group to admit them in the landing record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const idOfQueued = ([request]: Queued) => request.id;
/// ```
pub(crate) fn id_of_queued(queued: &Queued) -> String {
    return queued.0.id.clone();
}
