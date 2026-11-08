//! What:
//!  The service's acceptor of connections:
//! one task that accepts and starts a task per connection.
//! Why:
//!  A client that waits on the service costs a task,
//! not a thread (`doc/planning/git-policy-cli-full-tokio.md`,
//! step T10),
//! so the process's limited threads are never spent on waiting clients and no connection is refused because the system would
//! not start a thread for it.
//! The runtime that runs the tasks is the process's lazy runtime;
//! a system that gives it no worker thread ends the service with a plain line and status 4,
//! because a service that goes on without its acceptor answers no one,
//! while an ended service is started again by the manager.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! server.on('connection', (socket) => handleConnection(socket, shared));
//! ```

use crate::lazy_runtime::spawn_on_runtime;
use crate::service_server_connection_handling::{ConnectionShared, handle_connection};
use std::os::unix::net::UnixListener;
use std::time::Duration;

/// How long the acceptor waits after an accept that failed (no descriptor left,
/// a connection aborted) before it asks again:
/// the failure lasts while the pressure does,
/// and waiting keeps the acceptor from spinning on it.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(100);

/// What:
///  Starts the acceptor as a task of the process's runtime,
/// or ends the service with a plain line and status 4 when the runtime has no worker to run it.
/// Why:
///  The acceptor and the landing loop are the two things a service cannot do without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startOrExit('the acceptor of connections', () => acceptConnections(listener, shared));
/// ```
pub(crate) fn start_acceptor_or_exit(listener: UnixListener, shared: ConnectionShared) {
    if spawn_on_runtime(accept_connections(listener, shared)).is_none() {
        crate::service_error!("cannot start the acceptor of connections: the runtime that runs it has no worker thread (the system may refuse threads; check `ulimit -u` and the control group's `pids.max`)");
        std::process::exit(4);
    }
}

/// Accepts connections for as long as the process runs,
/// each served by a task of its own.
async fn accept_connections(inherited: UnixListener, shared: ConnectionShared) {
    if let Err(error) = inherited.set_nonblocking(true) {
        crate::service_error!("cannot make the listening socket non-blocking for the acceptor: {error}");
        std::process::exit(4);
    }
    let listener: tokio::net::UnixListener = match tokio::net::UnixListener::from_std(inherited) {
        Ok(registered) => registered,
        Err(error) => {
            crate::service_error!("cannot hand the listening socket to the runtime: {error}");
            std::process::exit(4);
        }
    };
    loop {
        match listener.accept().await {
            Ok((connection, _address)) => {
                tokio::spawn(handle_connection(connection, shared.clone()));
            }
            Err(error) => {
                crate::service_error!("a connection could not be accepted, so the acceptor waits a moment before it asks again: {error}");
                tokio::time::sleep(ACCEPT_BACKOFF).await;
            }
        }
    }
}
