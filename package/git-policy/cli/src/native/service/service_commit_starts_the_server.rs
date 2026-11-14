//! What:
//!  The first thing a commit does about the service when it needs one:
//! makes sure the worktree's service exists,
//! starting it when nothing answers,
//! and says why when it does not come up.
//! Why:
//!   A commit that never reaches the service (a declined one,
//! one a policy blocked,
//! one abandoned in the editor) must not start one,
//! so this runs at the first submission and nowhere else,
//! and its time is the timing record's `provisionMs`.
//! Until this release the wrapper asked the service manager to start the unit and then
//! waited for a greeting;
//! the wrapper starts the server itself now,
//! so a machine with no service manager is a machine where a commit still lands.
//! A start that another command of the same worktree is making is not a failure:
//! this returns as provisioned and the submit's own connect wait takes over,
//! because the other command is making the same service exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! provisioned = await ensureAServer(host, provisioned, timing);
//! ```

use crate::service_messages::{ClientFailure, render};
use crate::service_host_of_a_worktree::ServiceHost;
use crate::service_server_start_by_the_wrapper::{StartAttempt, start};
use crate::service_socket_directory_only_this_account_may_open::ensure_private;

/// What:
///  Makes sure the service exists,
/// once per command,
/// and gives whether it does.
/// Why:
///   A command that already started it asks nothing more,
/// and a worktree with no route to any service has nothing to start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (named === undefined || provisioned) return provisioned; const began = now(); const made = await ensure(named); timing.provision += now() - began;
/// ```
pub(crate) fn ensure_a_server(named: Option<&ServiceHost>, provisioned: bool, timing: &mut std::time::Duration) -> Result<bool, crate::commit_transaction::TransactionStop> {
    let Some(host) = named else {
        return Ok(provisioned);
    };
    if provisioned {
        return Ok(true);
    }
    let began: std::time::Instant = std::time::Instant::now();
    let made: Result<(), ClientFailure> = make_sure_a_server_exists(host);
    *timing += began.elapsed();
    return match made {
        Ok(()) => Ok(true),
        Err(failure) => Err(crate::commit_transaction::TransactionStop::Message(render(&failure))),
    };
}

/// What:
///  Starts the service when nothing answers,
/// and reports a start that does not greet.
/// It is the one way a command makes a server exist,
/// so the commit path and every hand-over path start one the same way.
/// Why:
///   The socket sits in a folder only this account may enter,
/// which must exist before a server can listen in it and before the service's
/// log can be opened beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ensurePrivate(host.socketDirectory); switch (startTheServer(host.server)) { hidden: case greeted, another-command, service-is-preparing: ok; case did-not-greet: fail }
/// ```
pub fn make_sure_a_server_exists(host: &ServiceHost) -> Result<(), ClientFailure> {
    let directory: std::path::PathBuf = host.socket_directory();
    if let Err(error) = ensure_private(directory.as_path()) {
        return Err(ClientFailure::CouldNotStart(format!(
            "{} cannot be created ({}), and the commit service's address and its output live there, so set XDG_RUNTIME_DIR to a \
             runtime directory this account can write or make that one writable",
            directory.display(),
            error
        )));
    }
    // What: The real Git this service runs is recorded beside the sockets before the service is started.
    // Why:  The unit file used to be that record; with no unit, a wrapper that finds no real Git on `PATH` has nothing else to
    //       fall back to. It is written here, on the one path that starts a server, so every command that starts one leaves it.
    if let Err(error) = crate::real_git_record::remember(host.runtime_dir.as_path(), host.git.as_path()) {
        crate::service_warn!("the real Git this commit service runs cannot be recorded beside its address ({error}), so a command that finds no Git on PATH has no fallback");
    }
    let server = host.server();
    return match start(&server) {
        // What: A service that is alive and still preparing is not a failure: the submit's own connect wait takes over.
        // Why:  A service recovers the batches an earlier service left open before it greets, and that recovery may outlast this start's
        //       wait; the wait that has the ceiling for it is the connect wait, which says a service was preparing when it ends
        //       (the fourth review of issue 6293, finding 1).
        StartAttempt::Greeted | StartAttempt::AnotherCommandStartsIt | StartAttempt::ServiceIsPreparing { .. } => Ok(()),
        StartAttempt::DidNotGreet(log) => Err(ClientFailure::CouldNotStart(format!(
            "the commit service was started and sent no greeting within {} seconds. Its output is in {}; if it says nothing, remove \
             that file and the address beside it and run the commit again",
            crate::service_client_listen_and_ask_to_retire::START_WAIT.as_secs(),
            log.display()
        ))),
    };
}

/// Controls of the start a commit makes stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_starts_the_server_tests.rs"]
mod tests;
