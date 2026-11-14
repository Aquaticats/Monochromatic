//! What:
//!  The settings of one client run: how it reaches the service,
//! how long it waits,
//! what it keeps fresh,
//! and what it
//!  starts the server with when nothing answers.
//! Why:
//!  Reaching the service and submitting a request both read these;
//! keeping them in a file that uses neither lets both use it.
//! The settings are the whole of what one command decides about the service,
//! so a control can build them
//!  without an environment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClientConfig = { activation: Activation; startWait: number; keepFresh?: FreshScratch; starter?: ServerToStart; ... }
//! ```

use crate::service_messages::Platform;
use crate::service_server::Bind;
use crate::service_server_start_by_the_wrapper::ServerToStart;
use std::time::Duration;
/// How long a start takes before the client says it is starting the service.
pub const START_NOTICE_GRACE: Duration = Duration::from_secs(1);

/// How many times a request is sent again after the connection broke before the client says it cannot be sure.
pub const RECONNECTS: usize = 5;

/// The pause between connection tries while the service starts.
pub(crate) const CONNECT_PAUSE: Duration = Duration::from_millis(25);

/// Who makes the service exist when it is not running.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Activation {
    /// The wrapper starts the server itself,
    /// at the address it looks the service up at.
    StartedByTheWrapper,
    /// This platform cannot start the service at all,
    /// which is every platform but Linux today.
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
    /// What starting the server needs;
    /// absent where this command may not start one (it was handed an address by another program, or the platform cannot start
    /// a process detached).
    pub starter: Option<ServerToStart>,
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
    /// What:
    ///  The longest a command waits for the service to be there,
    ///  starts included,
    ///  before it names the state it found.
    /// Why:
    ///  It is one start's bound (`START_WAIT`), and not the minute a service manager was given:
    /// with the wrapper starting the server itself there is no manager whose start rate, failed state or unit text
    /// could hold a start longer, and the longest measured cold start is 1,041 ms.
    ///  A second command that waits on the first one's start waits the same bound, which ends at the same moment.
    pub start_ceiling: Duration,
    /// What:
    ///  The longest a command waits for a service that is there and has not greeted yet,
    ///  which is a service that is alive and still returning to work.
    /// Why:
    ///  `RECOVERING_CEILING`: the longest a live service takes to greet, over the recoveries and the runtime build it may
    /// be inside, and past it the command ends saying so rather than saying nothing answered.
    pub recovering_ceiling: Duration,
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
///  The settings of a command that may start the server itself.
/// Why:
///   The route and the waits are the same for every command that hands work over,
/// and only the host differs,
/// so the settings are built here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const config = { address, activation: StartedByTheWrapper, starter: host?.server(), startWait, heartbeatSilence, ... };
/// ```
pub fn client_config(address: Bind, host: Option<&crate::service_host_of_a_worktree::ServiceHost>) -> ClientConfig {
    return ClientConfig {
        address,
        activation: Activation::StartedByTheWrapper,
        start_wait: crate::service_client_listen_and_ask_to_retire::START_WAIT,
        heartbeat_silence: crate::service_protocol::HEARTBEAT_SILENCE,
        starter: host.map(crate::service_host_of_a_worktree::ServiceHost::server),
        keep_fresh: None,
        answer_ceiling: crate::wait_ceilings::ANSWER_CEILING,
        withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING,
        extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT,
        start_ceiling: crate::service_client_listen_and_ask_to_retire::START_WAIT,
        recovering_ceiling: crate::wait_ceilings::recovering_ceiling(),
        waiting_notice_after: crate::wait_ceilings::waiting_notice_after(),
    };
}
