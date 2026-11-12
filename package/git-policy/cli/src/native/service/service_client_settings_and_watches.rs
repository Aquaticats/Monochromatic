//! What:
//!  The settings of one client run (how it reaches the service,
//! how long it waits,
//! what it keeps fresh) and the watch it can
//!  keep on the service manager's answer about the unit.
//! Why:
//!  Reaching the service and submitting a request both read these;
//! keeping them in a file that uses neither lets both use it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClientConfig = { activation: Activation; startWait: number; keepFresh?: FreshScratch; managerWatch?: ManagerWatch; ... }
//! ```

use crate::service_messages::Platform;
use crate::service_instance::{service_unit_instance, socket_unit_instance};
use crate::service_provision::ProvisionHost;
use crate::systemd_dbus::{Bus, EXCHANGE_WAIT};
use crate::service_server::Bind;
use std::path::PathBuf;
use std::time::Duration;
/// How long a start takes before the client says it is starting the service.
pub const START_NOTICE_GRACE: Duration = Duration::from_secs(1);

/// How many times a request is sent again after the connection broke before the client says it cannot be sure.
pub const RECONNECTS: usize = 5;

/// The pause between connection tries while the service starts.
pub(crate) const CONNECT_PAUSE: Duration = Duration::from_millis(25);

/// What the client may rely on the operating system to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Activation {
    /// The manager owns the socket and starts the service on demand.
    Managed,
    /// This platform cannot start the service on demand.
    Unsupported(Platform),
}

/// What the client needs to reach the service.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Where the service listens.
    pub address: Bind,
    /// What the operating system can do about a service that is not running.
    pub activation: Activation,
    /// Longest wait for a start when `start_watch` is absent;
    /// with a watch the wait ends only when the manager says the start
    ///  failed,
    /// and this is the wait for an older service to retire.
    pub start_wait: Duration,
    /// How long the service may say nothing before the client treats it as stopped,
    /// counted from each message it sends.
    /// There is no limit
    ///  on how long a request waits while the service keeps sending.
    pub heartbeat_silence: Duration,
    /// What:
    ///  The one budget of this command's whole exchange with the service.
    /// Why:
    ///   A commit's bound is the whole commit and not one of its steps,
    ///  so every step of its exchange asks `Patience` for what is left instead of taking its own bound.
    ///  The value every other command keeps is the silence bound each of its steps had already (`HEARTBEAT_SILENCE`).
    pub patience: Duration,
    /// How to ask the service manager whether the service failed to start;
    /// absent where the client cannot ask.
    pub start_watch: Option<ManagerWatch>,
    /// A scratch directory of this command to keep from being swept as stale while the command waits.
    pub keep_fresh: Option<FreshScratch>,
    /// The longest the client waits for the service's answer;
    /// then it withdraws the request (`wait_ceilings.rs`).
    pub answer_ceiling: Duration,
    /// The longest the client waits for the service to answer the withdrawal.
    pub withdraw_ceiling: Duration,
    /// How long the request may wait before the client says where the commit stands:
    /// the place the service reported,
    /// or that the
    ///  service has not started on it (`wait_ceilings.rs`,
    /// `WAITING_NOTICE_AFTER`).
    pub waiting_notice_after: Duration,
    /// How much longer a client whose own request is inside the service's branch write waits,
    /// after its answer ceiling,
    /// for that
    ///  write to end before it reports that the result is not known (`wait_ceilings.rs`,
    /// `SHORT_EXTRA_WAIT`).
    pub extra_wait: Duration,
    /// The longest a start is waited for when the service manager can be asked.
    pub start_ceiling: Duration,
    /// What:
    ///  The runtime directory whose record of the unit texts the manager was told about is dropped when a service of
    ///  another release will not retire,
    /// or nothing where this client did not provision.
    /// Why:
    ///   A service of another release that will not stop is proof that the unit the manager loaded and the files on disk
    ///  disagree,
    /// whatever a record says;
    /// dropping the record makes the next commit set the units up again,
    /// which is
    ///  the only repair of an interrupted downgrade followed by an interrupted upgrade
    ///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
    /// finding 2 of the round-2 review).
    pub units_record_dir: Option<PathBuf>,
}

/// What:
///  A scratch directory a waiting command keeps fresh,
/// and how often.
/// Why:
///   The scratch sweep removes entries by age,
/// which says nothing about whether the command that made the entry still runs;
/// a waiter that is alive says so by
///  touching its directory,
/// one that died stops and the sweep collects the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FreshScratch = { path: string; every: number };
/// ```
#[derive(Clone, Debug)]
pub struct FreshScratch {
    /// The directory.
    pub path: std::path::PathBuf,
    /// How often its modification time is renewed while the command waits.
    pub every: Duration,
}

/// What:
///  How the client asks the service manager whether the service unit has failed.
/// Why:
///   A service that is starting sends nothing,
/// so the wait for its first greeting has no heartbeat to listen for.
/// The manager
///  knows whether the start failed,
/// which is the one thing that ends that wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerWatch = { bus: string; unit: string; socketUnit: string; every: number; state: (bus, unit) => string | undefined; start: (bus, unit) => boolean; reset: (bus, unit) => boolean; restart: (bus, unit) => boolean };
/// ```
#[derive(Clone, Debug)]
pub struct ManagerWatch {
    /// The session bus of the account.
    pub bus: std::path::PathBuf,
    /// The service unit of the worktree.
    pub unit: String,
    /// The socket unit of the worktree,
    /// which the service unit requires:
    /// a start of the service starts it too,
    /// so a failed state is forgotten for both together.
    pub socket_unit: String,
    /// How often the manager is asked while the greeting has not come.
    pub every: Duration,
    /// The unit's `ActiveState` now (`active`,
    /// `activating`,
    /// `inactive`,
    /// `failed`,
    /// ...);
    /// nothing when the manager cannot be
    ///  asked,
    /// which leaves the wait to the start wait.
    pub state: fn(&std::path::Path, &str) -> Option<String>,
    /// Asks the manager to start the unit;
    /// whether the manager took the request.
    pub start: fn(&std::path::Path, &str) -> bool,
    /// Asks the manager to forget that the unit failed,
    /// so a start is not refused as repeated too quickly;
    /// whether the manager took the request.
    pub reset: fn(&std::path::Path, &str) -> bool,
    /// Asks the manager to stop the unit and start it again,
    /// which is the one repair of a unit it reports `active` whose program stopped inside its own process;
    /// whether the manager took the request.
    pub restart: fn(&std::path::Path, &str) -> bool,
    /// Asks the manager to read the unit files again,
    /// which is what makes it forget a unit whose executable is gone;
    /// whether the manager took the request.
    pub reload: fn(&std::path::Path) -> bool,
}

/// How often the service manager is asked whether the service unit failed,
/// while a start is waited for.
pub const MANAGER_ASK_EVERY: Duration = START_NOTICE_GRACE;

/// The manager's `ActiveState` of `unit`:
/// nothing when the bus or the unit cannot be asked about.
pub(crate) fn unit_state(bus: &std::path::Path, unit: &str) -> Option<String> {
    let mut connection: Bus = Bus::connect(bus, EXCHANGE_WAIT).ok()?;
    return connection.active_state(unit).ok()?;
}

/// Asks the manager to start `unit`;
/// whether the request was taken.
pub(crate) fn start_unit(bus: &std::path::Path, unit: &str) -> bool {
    let Ok(mut connection) = Bus::connect(bus, EXCHANGE_WAIT) else { return false };
    return connection.start_unit(unit).is_ok();
}

/// Asks the manager to read the units again;
/// whether the request was taken.
pub(crate) fn reload_units(bus: &std::path::Path) -> bool {
    let Ok(mut connection) = Bus::connect(bus, EXCHANGE_WAIT) else { return false };
    return connection.reload().is_ok();
}

/// Asks the manager to forget that `unit` failed;
/// whether the request was taken.
/// A unit that failed too often is refused as "repeated too quickly" until this
/// is asked.
pub(crate) fn reset_failed_unit(bus: &std::path::Path, unit: &str) -> bool {
    let Ok(mut connection) = Bus::connect(bus, EXCHANGE_WAIT) else { return false };
    return connection.reset_failed(unit).is_ok();
}

/// Asks the manager to stop `unit` and start it again;
/// whether the request was taken.
/// A unit the manager reports `active` whose program stopped inside its own process is repaired by no start.
pub(crate) fn restart_unit(bus: &std::path::Path, unit: &str) -> bool {
    let Ok(mut connection) = Bus::connect(bus, EXCHANGE_WAIT) else { return false };
    return connection.restart_unit(unit).is_ok();
}

/// What:
///  The watch that asks the account's service manager about the worktree's service unit.
/// Why:
///   The route knows the session bus and the worktree's instance,
/// which is all the unit's name needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = managerWatchFor(host)
/// ```
pub fn manager_watch_for(host: &ProvisionHost) -> ManagerWatch {
    return ManagerWatch {
        bus: host.bus.clone(),
        unit: service_unit_instance(host.instance.as_str()),
        socket_unit: socket_unit_instance(host.instance.as_str()),
        every: MANAGER_ASK_EVERY,
        state: unit_state,
        start: start_unit,
        reset: reset_failed_unit,
        restart: restart_unit,
        reload: reload_units,
    };
}
