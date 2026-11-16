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
use crate::service_server_start_by_the_wrapper::StartAttempt;
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
/// ensurePrivate(host.socketDirectory); switch (startTheServer(host.server)) { hidden: case greeted, another-command: ok; case did-not-greet: fail }
/// ```
pub fn make_sure_a_server_exists(host: &ServiceHost) -> Result<(), ClientFailure> {
    return make_sure_a_server_exists_within(host, crate::service_client_listen_and_ask_to_retire::START_WAIT);
}

/// What:
///  Starts the service waiting no longer than `wait` for its greeting.
/// Why:
///   The commit's own bound is `START_WAIT`, and a caller with a shorter one (a removal or a relink hand-over, whose bound is
///   `HAND_OVER_GREETING_WAIT`) gives it here, so its bound reaches the start instead of being set after it
///   (the review of `567ae37198`, finding 5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makeSureAServerExistsWithin(host, 2000) // the hand-over's bound reaches the start
/// ```
pub fn make_sure_a_server_exists_within(host: &ServiceHost, wait: std::time::Duration) -> Result<(), ClientFailure> {
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
    return match crate::service_server_start_by_the_wrapper::start_within(&server, wait) {
        StartAttempt::Greeted | StartAttempt::AnotherCommandStartsIt => Ok(()),
        StartAttempt::DidNotGreet(log) => Err(ClientFailure::CouldNotStart(format!(
            "the commit service was started and sent no greeting within {}. Its output is in {}; if it says nothing, remove \
             that file and the address beside it and run the commit again",
            bound_in_words(wait),
            log.display()
        ))),
    };
}

/// What:
///  A bound in the words a person reads:
///  whole seconds, or milliseconds when it is not a whole second.
/// Why:
///   The commit waits ten seconds and a hand-over waits two,
///  and a number of milliseconds in a message about a wait reads as a measurement of something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// boundInWords(2000) // '2 seconds'
/// ```
fn bound_in_words(wait: std::time::Duration) -> String {
    let millis: u128 = wait.as_millis();
    return match millis % 1000 {
        0 => format!("{} seconds", millis / 1000),
        _ => format!("{millis} milliseconds"),
    };
}

/// Controls of the start a commit makes stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_starts_the_server_tests.rs"]
mod tests;
