//! What:
//!  The wake a command gives the commit service when it releases a held push by itself:
//!  the record became runnable,
//!  and the service that makes it has to look.
//! Why:
//!  A held record is promoted only by the command that landed the commit,
//! and that command woke the service itself.
//! A
//!  command that settles a record from the verdict a dead command left is not the command that landed the
//!  commit,
//! so nothing else would tell the service the push is now runnable,
//! and the branch's backup would wait for the
//!  next commit,
//! which may never come
//! (`doc/handover/questions-for-the-owner.md`,
//! "A commit whose command died before its checks ran now
//!  stays local").
//! The wake is best effort:
//! a machine with no service to reach leaves the record in `pending`,
//! where the next service to
//!  look makes it
//! (`service_claim_watch.rs`),
//! and the line the command printed says exactly that.
//! The seam is injected so a control never reaches a service or the session bus,
//! as the push line's own proof is
//!  (`held_push_commit_on_the_remote.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! wakeTheService(gitDir); // route from the environment, provision, wake; a failure is not the command's
//! ```

use crate::service_client::{Activation, ClientConfig, manager_watch_for, wake};
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::provision;
use crate::service_provision_host_and_manager::units_record_dir_of;
use crate::service_route::{Route, route_from_environment};
use crate::transaction_git::GitContext;
use crate::wait_ceilings::{ANSWER_CEILING, HAND_OVER_GREETING_WAIT, SHORT_EXTRA_WAIT, WITHDRAW_CEILING, waiting_notice_after};
use std::path::Path;

/// What:
///  How a command tells the commit service to look at a repository now.
/// Why:
///   The step reaches a socket,
/// a service manager and the session bus,
/// and a control of the held line must do none of
///  those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WakeTheService { wake(gitDir: string): void }
/// ```
pub trait WakeTheService {
    /// Says that a repository has a runnable record now.
    fn wake(&self, git_dir: &Path);
}

/// What:
///  The wake through this process's environment,
/// as a commit's hand-off makes it.
/// Why:
///   A released push is handed over exactly as a landed commit's push is,
/// so both find the service the same way,
/// and the
///  command that printed the queued line has done its part even when the wake fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AskTheService implements WakeTheService { wake(gitDir) { route(gitDir)?.provision(); config().wake(gitDir) } }
/// ```
pub struct AskTheService<'a> {
    /// The Git this command runs,
    /// which the route and the unit texts are about.
    pub context: &'a GitContext,
}

impl WakeTheService for AskTheService<'_> {
    fn wake(&self, git_dir: &Path) {
        let Some(Route { address, provision: host }) = route_from_environment(self.context.real_git.as_path(), git_dir) else {
            return;
        };
        if let Some(setup) = host.as_ref()
            && provision(setup).is_err()
        {
            return;
        }
        let config: ClientConfig = ClientConfig {
            address,
            activation: Activation::Managed,
            // The held push's record is durable, so the wake waits the hand-over's own greeting bound.
            start_wait: HAND_OVER_GREETING_WAIT,
            heartbeat_silence: HEARTBEAT_SILENCE,
            patience: HEARTBEAT_SILENCE,
            start_watch: host.as_ref().map(manager_watch_for),
            keep_fresh: None,
            answer_ceiling: ANSWER_CEILING,
            withdraw_ceiling: WITHDRAW_CEILING,
            waiting_notice_after: waiting_notice_after(),
            extra_wait: SHORT_EXTRA_WAIT,
            start_ceiling: HAND_OVER_GREETING_WAIT,
            units_record_dir: host.as_ref().map(units_record_dir_of),
        };
        // The wake's own lines are not the command's report, and a service that cannot be reached is not its failure.
        let mut silent: std::io::Sink = std::io::sink();
        let _woken: Result<(), crate::service_messages::ClientFailure> = wake(&config, git_dir, None, &mut silent);
    }
}

/// What:
///  A caller that asks nothing of any service:
/// a control,
/// and a command with no repository to wake one for.
/// Why:
///   The line the person reads is about the record's own state,
/// and it must be readable where no service exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingWoken = { wake() {} };
/// ```
pub struct NothingWoken;

impl WakeTheService for NothingWoken {
    fn wake(&self, _git_dir: &Path) {}
}

/// Controls of the wake stay out of the release executable.
#[cfg(test)]
#[path = "held_push_release_wakes_the_service_tests.rs"]
mod tests;
