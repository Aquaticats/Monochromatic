//! Time-bounded D-Bus steps for the synchronous threads of the player, Linux only.
//!
//! zbus 6 has no blocking API.
//! A synchronous thread drives the async API with `zbus::block_on`, which runs zbus's own zruntime on the calling
//! thread until the future is done, and a helper thread keeps a live connection answering between calls.
//! The player adds no runtime of its own for this: each D-Bus step is one `block_on` call on the thread that needs it.
//!
//! Nothing here waits without a bound.
//! `Builder::method_timeout` bounds method calls only, and zbus's own timeout helper is crate-private, so every
//! step (a connect, a signal send, a call, a registration) races the zruntime timer of the calling thread, and every
//! connection also carries `method_timeout`, so a call ends in `ErrorKind::TimedOut` at the same bound.
//! The one wait without a bound is a listener thread's read of its signal stream, which waits for senders and ends
//! when the connection closes.

use std::future::Future;
use std::io::{Error as IoError, ErrorKind};
use std::time::Duration;

use zbus::Connection;
use zbus::connection::Builder;
use zruntime::SharedRuntime;

use crate::analysis_notification::METHOD_TIMEOUT;

/// What: `DBUS_STEP_BOUND` is the longest one D-Bus step may take: a connect, a signal send, a call or a registration.
/// Why: it is `METHOD_TIMEOUT`, the bound the notifiers already put on every call that waits for a reply. On a private
/// bus the whole of `a_connect_and_a_call_on_a_private_bus_finish_far_inside_the_bound` (start the daemon, connect,
/// one `Notify` call) took 0.043 s under nextest on a host at a load average of 12 to 49, and the test fails when
/// the connect or the call alone takes a tenth of the bound; the sibling crate `nested-wayland-session` measured a
/// connect at 1.2 to 1.4 ms and a call at 0.19 to 0.26 ms over six runs. Three seconds ends a stalled peer and
/// leaves a loaded host room.
pub(crate) const DBUS_STEP_BOUND: Duration = METHOD_TIMEOUT;

/// What: `within` awaits `work` and ends it with a `TimedOut` error after `bound`.
/// Why: the future is dropped at the deadline, so a stalled step leaves no thread behind. It must run inside
/// `zbus::block_on`, because the timer belongs to the runtime that call drives.
///
/// # Errors
/// The step's own error, or `zbus::Error::InputOutput` of kind `TimedOut` after `bound`.
pub(crate) async fn within<T>(bound: Duration, work: impl Future<Output = zbus::Result<T>>) -> zbus::Result<T> {
    let runtime = SharedRuntime::current()?;
    return match runtime.timeout(bound, work).await {
        Ok(outcome) => outcome,
        Err(timed_out) => Err(zbus::Error::from(IoError::new(
            ErrorKind::TimedOut,
            format!("the D-Bus step did not finish within {bound:?}: {timed_out}"),
        ))),
    };
}

/// What: `step` runs `work` to completion on the calling thread, ending it with a `TimedOut` error after
/// `DBUS_STEP_BOUND`.
/// Why: the notifiers, the launcher and the probes are synchronous threads, and `zbus::block_on` returns only when its
/// future does.
///
/// # Errors
/// As `within`.
pub(crate) fn step<T>(work: impl Future<Output = zbus::Result<T>>) -> zbus::Result<T> {
    return zbus::block_on(within(DBUS_STEP_BOUND, work));
}

/// What: `connect` builds the connection `builder` describes, with `timeout` as both its method-call bound and the
/// bound on the connect itself.
/// Why: a connection built without `method_timeout` waits for a reply with no limit; built inside `block_on`, it is
/// served by the calling thread during a call and by zbus's helper thread between calls.
///
/// # Errors
/// As `within`.
pub(crate) fn connect(builder: Builder<'_>, timeout: Duration) -> zbus::Result<Connection> {
    return zbus::block_on(within(timeout, builder.method_timeout(timeout).build()));
}

#[cfg(all(test, target_os = "linux"))]
#[path = "dbus_step_bound_tests.rs"]
mod tests;
