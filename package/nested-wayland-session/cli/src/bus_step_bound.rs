//! Time-bounded D-Bus steps for the synchronous threads of the compositor.
//!
//! zbus 6 has no blocking API.
//!  A synchronous thread drives the async API with `zbus::block_on`,
//!  which runs zbus's own zruntime on the calling thread until the future is done,
//!  and a helper thread keeps a live connection served between calls.
//!  This crate adds no runtime of its own:
//!  its Smithay loop and its worker threads stay synchronous,
//!  and each D-Bus step is one `block_on` call.
//!
//! Nothing here waits forever.
//!  Every step races the zruntime timer of the calling thread,
//!  because zbus's own timeout helper is crate-private
//!  and `Builder::method_timeout` bounds method calls only,
//!  not a connect,
//!  a signal send,
//!  or a stream read.
//!  Every connection also carries `method_timeout`,
//!  so a call made through a proxy ends in `ErrorKind::TimedOut` at the same bound.

/// Imports `Future` from `std`,
///  `Duration` from `std::time`.
use std::{future::Future, time::Duration};

/// Imports `Context`,
///  `Result`,
///  and `bail` from `anyhow`.
use anyhow::{bail, Context, Result};
/// Imports `Builder`,
///  `Connection` from `zbus::connection`.
use zbus::connection::{Builder, Connection};
/// Imports `SharedRuntime` from `zruntime`.
use zruntime::SharedRuntime;

/// Longest one D-Bus step may take:
///  a connect,
///  a signal send,
///  a method call,
///  or a registration.
///
/// What:
///      `pub const BUS_STEP_BOUND: Duration = Duration::from_secs(5);`.
/// Why:
///       Every step of this crate runs against a private bus that this process started a moment earlier,
///  so a healthy step finishes in milliseconds
///  (measured by `bus_steps_finish_far_inside_the_bound` in `src/bus_step_bound_tests.rs`);
///  the bound is the largest value already used for a bus wait in this crate
///  (`READ_BUDGET` in `src/a11y_tree.rs`),
///  which leaves a loaded host room and still ends a stalled peer.
pub const BUS_STEP_BOUND: Duration = Duration::from_secs(5);

/// Awaits `work` and ends it with an error after `bound`.
///
/// What:
///      `pub async fn within<T, E>(bound: Duration, step: &str, work: impl Future<Output = Result<T, E>>)
///           -> Result<T>`.
///  `step` names the action in the error text.
///  The future is dropped at the deadline,
///  so a stalled step leaves no thread behind.
///  It must run inside `zbus::block_on`,
///  because the timer belongs to the runtime that call drives.
/// Why:
///       A peer that stops answering would otherwise hold the step for good.
///
/// # Errors
///
/// Fails with the step's own error,
///  or with a timeout message naming `step` and `bound`.
pub async fn within<T, E>(bound: Duration, step: &str, work: impl Future<Output = Result<T, E>>) -> Result<T>
where
    E: Into<anyhow::Error>,
{
    // What:     `SharedRuntime::current()`. The runtime `block_on` is running on this thread.
    // Why:      Its timer is driven by the same call that drives the connections.
    let runtime = SharedRuntime::current().context("opening the timer of the D-Bus runtime")?;
    return match runtime.timeout(bound, work).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.into().context(format!("{step} failed"))),
        Err(timed_out) => bail!("{step} did not finish within {bound:?}: {timed_out}"),
    };
}

/// Runs `work` to completion on the calling thread and ends it with an error after `bound`.
///
/// What:
///      `pub fn block_on_within<T, E>(bound: Duration, step: &str, work: impl Future<Output = Result<T, E>>)
///           -> Result<T>`.
/// Why:
///       zbus's `block_on` returns only when its future does,
///  and a peer that stops answering would hold the caller for good.
///
/// # Errors
///
/// Fails as `within` does.
pub fn block_on_within<T, E>(bound: Duration, step: &str, work: impl Future<Output = Result<T, E>>) -> Result<T>
where
    E: Into<anyhow::Error>,
{
    return zbus::block_on(within(bound, step, work));
}

/// Runs `work` as one D-Bus step with the standard bound.
///
/// What:
///      `pub fn block_on_bounded<T, E>(step: &str, work: impl Future<Output = Result<T, E>>) -> Result<T>`.
/// Why:
///       Most steps have the one bound `BUS_STEP_BOUND`.
///
/// # Errors
///
/// Fails as `within` does.
pub fn block_on_bounded<T, E>(step: &str, work: impl Future<Output = Result<T, E>>) -> Result<T>
where
    E: Into<anyhow::Error>,
{
    return block_on_within(BUS_STEP_BOUND, step, work);
}

/// Connects the connection `builder` describes,
///  with the method-call bound set and the connect itself bounded.
///
/// What:
///      `pub async fn connect_within_bound(builder: Builder<'_>, step: &str) -> Result<Connection>`.
///  The connection runs on zbus's zruntime of the thread inside `block_on`.
/// Why:
///       A connection built without `method_timeout` waits for a reply with no limit.
///
/// # Errors
///
/// Fails as `within` does.
pub async fn connect_within_bound(builder: Builder<'_>, step: &str) -> Result<Connection> {
    return within(BUS_STEP_BOUND, step, builder.method_timeout(BUS_STEP_BOUND).build()).await;
}

/// Builds the connection `builder` describes on the calling thread,
///  as `connect_within_bound` does.
///
/// What:
///      `pub fn build_bounded(builder: Builder<'_>, step: &str) -> Result<Connection>`.
///  Built inside `block_on`,
///  the connection is served by the calling thread during a call
///  and by zbus's helper thread between calls.
/// Why:
///       The services of this crate are built once on the compositor thread and then emit from it.
///
/// # Errors
///
/// Fails as `within` does.
pub fn build_bounded(builder: Builder<'_>, step: &str) -> Result<Connection> {
    return zbus::block_on(connect_within_bound(builder, step));
}

/// Tests for the bound.
#[cfg(test)]
#[path = "bus_step_bound_tests.rs"]
mod tests;
