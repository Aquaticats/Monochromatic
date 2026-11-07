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
use crate::service_instance::service_unit_instance;
use crate::service_provision::ProvisionHost;
use crate::systemd_dbus::{Bus, EXCHANGE_WAIT};
use crate::service_server::Bind;
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
    /// How long the service may send nothing,
    /// heartbeats included,
    /// before the client treats it as stopped.
    /// There is no limit
    ///  on how long a request waits while the service keeps sending.
    pub heartbeat_silence: Duration,
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
    /// How much longer a client whose own request is inside the service's branch write waits,
    /// after its answer ceiling,
    /// for that
    ///  write to end before it reports that the result is not known (`wait_ceilings.rs`,
    /// `SHORT_EXTRA_WAIT`).
    pub extra_wait: Duration,
    /// The longest a start is waited for when the service manager can be asked.
    pub start_ceiling: Duration,
}

/// What:
///  A scratch directory a waiting command keeps fresh,
/// and how often.
/// Why:
///   The scratch sweep removes entries by age,
/// and a wait with no bound can outlast any age;
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
/// type ManagerWatch = { bus: string; unit: string; every: number; state: (bus, unit) => string | undefined; start: (bus, unit) => boolean };
/// ```
#[derive(Clone, Debug)]
pub struct ManagerWatch {
    /// The session bus of the account.
    pub bus: std::path::PathBuf,
    /// The service unit of the worktree.
    pub unit: String,
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
    return ManagerWatch { bus: host.bus.clone(), unit: service_unit_instance(host.instance.as_str()), every: MANAGER_ASK_EVERY, state: unit_state, start: start_unit };
}
