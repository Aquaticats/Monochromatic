//! What:
//!  What one service run is configured with:
//!  how long it waits with nothing to do,
//!  the release it announces,
//!  the worktree it serves,
//!  the waits its controls shorten,
//!  the Git directory of its own worktree,
//!  how long it reads its journal for,
//!  and what it does when that worktree is gone.
//! Why:
//!   The loop reads one value of this shape for every decision it makes about its own life,
//!   so an ordinary run is one call (`ServeSettings::new`) and a control changes one field.
//!  The idle exit and the worktree check it starts with are the numbers in
//!  `service_server_batch_and_idle_limits.rs`,
//!  and the loop that obeys them is `service_server.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settings = new ServeSettings(IDLE_EXIT); settings.stuck_after = 5000;
//! ```
use crate::service_protocol::RELEASE;
use crate::service_server_batch_and_idle_limits::OWN_WORKTREE_CHECK;
use crate::wait_ceilings::{
    NEVER_ENDED_STEP_SLOW_AFTER,
    START_CEILING,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// How a service run behaves apart from where it listens and what it lands.
/// No equality:
/// the stop of a service's own units is a function pointer,
/// whose addresses are not comparable.
#[derive(Clone, Debug)]
pub struct ServeSettings {
    /// How long the service waits with nothing to do before it exits.
    pub idle: Duration,
    /// The release the service announces to every client.
    /// It is this build's release (`RELEASE`) except in a test that
    ///  needs a service of another release.
    pub release: String,
    /// The worktree this instance serves,
    /// as `service_instance::instance_of` names it;
    /// every message about another worktree
    ///  is refused.
    /// Nothing in a development run that serves any worktree.
    pub instance: Option<String>,
    /// How long a write the service never ends runs before the clients behind it are told,
    /// and may be withdrawn while the
    ///  service is inside it (`wait_ceilings.rs`,
    /// `NEVER_ENDED_STEP_SLOW_AFTER`);
    /// a control shortens it.
    pub stuck_after: Duration,
    /// How long a batch's first request must have waited before it is told it is next,
    /// which is how a person sees the service
    ///  take the commit after a wait (`wait_ceilings.rs`,
    /// `WAITING_NOTICE_AFTER`);
    /// a control shortens it.
    pub place_after: Duration,
    /// How long a landing waits for the note that names its repository to the service that starts next
    /// (`wait_ceilings.rs`,
    /// `LANDING_WATCH_NOTE_CEILING`);
    /// a control shortens it.
    pub note_ceiling: Duration,
    /// The Git directory of the worktree whose units are this service's,
    /// as its note or its journal named it when the service started,
    /// and nothing when it was not there or could not be told.
    /// The serving loop asks about it every `own_worktree_check`.
    pub own_git_dir: Option<PathBuf>,
    /// How often the serving loop asks whether `own_git_dir` is still there.
    pub own_worktree_check: Duration,
    /// How long the plan of the pushes waiting for a new try and the repositories an earlier service was woken for
    /// are read for before the service starts without them
    /// (`service_journal_restored_at_start.rs`);
    /// a control shortens it.
    pub journal_restore_ceiling: Duration,
    /// What the service does when that directory is gone:
    /// the shipped one stops this service's own socket through the user manager
    ///  (`service_units_of_removed_worktrees::stop_own_socket_when_the_worktree_is_gone`),
    /// and a control passes its own,
    /// so the loop is checked without a manager.
    pub own_units_stop: fn(&str, &Path) -> bool,
}

impl ServeSettings {
    /// The settings of an ordinary run:
    /// the given idle exit and this build's release.
    pub fn new(idle: Duration) -> Self {
        return Self {
            idle,
            release: String::from(RELEASE),
            instance: None,
            stuck_after: NEVER_ENDED_STEP_SLOW_AFTER,
            place_after: crate::wait_ceilings::WAITING_NOTICE_AFTER,
            note_ceiling: crate::wait_ceilings::LANDING_WATCH_NOTE_CEILING,
            own_git_dir: None,
            own_worktree_check: OWN_WORKTREE_CHECK,
            journal_restore_ceiling: START_CEILING,
            own_units_stop:
                crate::service_units_of_removed_worktrees::stop_own_socket_when_the_worktree_is_gone,
        };
    }
}
