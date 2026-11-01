//! Spawning and lifecycle of the single hosted client process.
//!
//! The fixture forks exactly one client,
//!  pointed at the nested socket via
//! `WAYLAND_DISPLAY`,
//!  and stops the event loop (propagating the exit code) when that
//! client exits.
//!  Child exit is detected by a periodic calloop timer that polls
//! `try_wait`,
//!  which keeps everything on the event loop's single thread.

/// What:
///      `use std::time::Duration;`.
///  `Duration` is the exit-poll interval.
///  The child
///           `Command` itself is built by the `systemd` module,
///  so it is not named here.
/// Why:
///       Needed to schedule the exit poll.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Duration ~ a ms count.
/// ```
use std::{
    ffi::OsStr,
    process::Command,
    time::{Duration, Instant},
};

/// What:
///      Grouped `use` of the calloop timer types and loop handle.
/// Why:
///       `register_exit_poll` inserts a `Timer` source through the `LoopHandle`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Timer, TimeoutAction, LoopHandle } from "smithay/calloop";
/// ```
use smithay::reexports::calloop::{
    timer::{TimeoutAction, Timer},
    LoopHandle,
};

/// What:
///      `use anyhow::{Context, Result};`.
///  Error helpers.
/// Why:
///       `spawn_child` returns `Result` and annotates spawn failures.
use anyhow::{Context, Result};

/// What:
///      `use tracing::{info, warn};`.
///  Structured log macros.
/// Why:
///       Report spawn and exit events.
use tracing::{info, warn};

/// What:
///      `use crate::{state::Compositor, systemd::Isolation};`.
///  Our state type and the
///           CPU-isolation settings.
/// Why:
///       `spawn_child` reads/writes state and builds the child command per the isolation
///           settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Compositor } from "./state"; import { Isolation } from "./systemd";
/// ```
use crate::{state::Compositor, systemd::Isolation};

/// How often to poll the hosted child for exit.
///
/// What:
///      `const POLL_INTERVAL: Duration = Duration::from_millis(200);`.
///  A fixed
///           200-millisecond interval.
/// Why:
///       Frequent enough to shut down promptly after the app exits,
///  rare enough to
///           cost nothing measurable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POLL_INTERVAL_MS = 200;
/// ```
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Grace period between compositor close request and forced child termination.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// Applies child-only Wayland and optional private session-bus environment.
fn configure_child_environment(
    command: &mut Command,
    socket_name: &OsStr,
    session_bus_address: Option<&str>,
    gpu_variables: &[(&str, Option<&str>)],
) {
    // Keep the child on the compositor's device: variables Mesa reads in the client (headless GPU choice only).
    for (name, value) in gpu_variables {
        match value {
            Some(text) => command.env(name, text),
            None => command.env_remove(name),
        };
    }
    command.env("WAYLAND_DISPLAY", socket_name);
    command.env_remove("WAYLAND_SOCKET");
    // Do not let a clipboard library or child GUI fall back to the host X11 display.
    command.env_remove("DISPLAY");
    command.env_remove("XAUTHORITY");
    if let Some(address) = session_bus_address {
        command.env("DBUS_SESSION_BUS_ADDRESS", address);
        command.env_remove("DBUS_STARTER_ADDRESS");
        command.env_remove("DBUS_STARTER_BUS_TYPE");
    }
}

/// Spawn the hosted client,
///  pointed at the nested Wayland socket.
///
/// What:
///      `pub fn spawn_child(...) -> Result<()>` receives compositor state,
///           command,
///           isolation,
///           and optional private session-bus address.
/// Why:
///       Launch the one client on nested Wayland and isolated appearance portal
///           rather than host compositor or host Settings portal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spawnChild(state, command): void { ... }
/// ```
///
/// @example
/// ```ts
/// spawnChild(state, ["music-player", "fixtures"]);
/// ```
pub fn spawn_child(
    state: &mut Compositor,
    command: &[String],
    isolation: &Isolation,
    session_bus_address: Option<&str>,
) -> Result<()> {
    // What:     `let program = &command[0];`. Borrow the first token (the executable).
    //           `parse_args` guarantees `command` is non-empty, so index 0 is safe.
    // Why:      Names the program to launch and to report in messages.
    let program = &command[0];

    // What:     `let mut cmd = crate::systemd::build_child_command(program, &command[1..],
    //           isolation);`. Build the spawn command: either a `systemd-run --scope`
    //           wrapper (when isolation is enabled and systemd is available) or the app
    //           directly. `program` (a `&String`) coerces to the `&str` the helper takes.
    // Why:      Centralise the isolate-or-degrade decision in one place; `mut` because we
    //           set the Wayland environment on it next.
    let mut cmd = crate::systemd::build_child_command(program, &command[1..], isolation);

    // Point child at nested Wayland and optional private appearance portal only.
    let gpu_variables = state
        .backend
        .child_gpu_environment()
        .map(crate::gpu_choice::child_variables)
        .unwrap_or_default();
    configure_child_environment(
        &mut cmd,
        state.socket_name.as_os_str(),
        session_bus_address,
        &gpu_variables,
    );

    // What:     `let child = cmd.spawn().with_context(|| format!("failed to spawn {program}"))?;`.
    //           `spawn()` starts the process, returning `io::Result<Child>`;
    //           `.with_context(closure)` lazily attaches a message on error; `?` unwraps
    //           or returns the error.
    // Why:      Actually launch the client, failing cleanly if the binary is missing.
    let child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn hosted client: {program}"))?;

    // What:     `info!(...)`. Log the spawn with the socket the child will use.
    // Why:      Make the wiring visible for debugging.
    info!("spawned hosted client {program} on {:?}", state.socket_name);

    // What:     `state.child = Some(child);`. Store the process handle. `Some(child)`
    //           wraps it in the present variant of `Option`.
    // Why:      The exit-poll timer needs the handle to `try_wait` on.
    state.child = Some(child);

    // What:     `return Ok(());`. Success with unit value.
    // Why:      Signal the spawn succeeded.
    return Ok(());
}

/// Requests graceful xdg-toplevel close and schedules force-stop fallback.
pub fn request_hosted_client_shutdown(state: &mut Compositor) {
    let toplevels = state
        .space
        .elements()
        .filter_map(|window| return window.toplevel().cloned())
        .collect::<Vec<_>>();
    state.shutdown_deadline = Some(Instant::now() + SHUTDOWN_GRACE);
    if toplevels.is_empty() {
        warn!("no hosted toplevel available for graceful close; scheduling forced shutdown");
        return;
    }
    for toplevel in toplevels {
        toplevel.send_close();
    }
    info!("requested graceful close from hosted client");
}

/// Returns whether pending graceful shutdown passed its fallback deadline.
fn shutdown_expired(deadline: Option<Instant>, now: Instant) -> bool {
    return deadline.is_some_and(|value| return now >= value);
}

/// Register the periodic child-exit poll on the event loop.
///
/// What:
///      `pub fn register_exit_poll(loop_handle: &LoopHandle<Compositor>)`.
///  Borrows
///           the loop handle to insert a repeating timer source.
/// Why:
///       Turns "the child exited" into a loop-stopping event without a second thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registerExitPoll(loopHandle) { ... }
/// ```
pub fn register_exit_poll(loop_handle: &LoopHandle<Compositor>) {
    // What:     `loop_handle.insert_source(Timer::from_duration(POLL_INTERVAL), |_,
    //           _, state| { ... }).expect(...)`. Registers a timer that first fires after
    //           `POLL_INTERVAL`. The callback receives the fire instant (`_`), metadata
    //           (`_`), and `&mut Compositor` (`state`).
    // Why:      Poll the child on a fixed cadence.
    loop_handle
        .insert_source(Timer::from_duration(POLL_INTERVAL), |_, _, state: &mut Compositor| {
            // What:     `poll_child(state);`. Check whether the child exited and, if so,
            //           stop the loop.
            // Why:      The actual exit check.
            poll_child(state);

            // What:     `TimeoutAction::ToDuration(POLL_INTERVAL)`. Tell calloop to
            //           reschedule the timer for another `POLL_INTERVAL` from now (tail
            //           expression of the closure).
            // Why:      Keep polling until the loop stops.
            return TimeoutAction::ToDuration(POLL_INTERVAL)
        })
        .expect("failed to register the child-exit poll timer");
}

/// Check whether the hosted child has exited;
///  if so,
///  record its code and stop the loop.
///
/// What:
///      `fn poll_child(state: &mut Compositor)`.
///  Private helper called by the timer.
/// Why:
///       Centralise the try-wait / record-code / stop-loop sequence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pollChild(state) { ... }
/// ```
fn poll_child(state: &mut Compositor) {
    let force_shutdown = shutdown_expired(state.shutdown_deadline, Instant::now());

    // What:     `let Some(child) = state.child.as_mut() else { return; };`. `as_mut()`
    //           borrows the `Option<Child>` as `Option<&mut Child>`; the `let ... else`
    //           binds the child mutably when present or returns when it is already gone.
    // Why:      Nothing to poll once the child has been cleared.
    let Some(child) = state.child.as_mut() else {
        return;
    };

    // What:     `match child.try_wait() { ... }`. `try_wait()` returns
    //           `io::Result<Option<ExitStatus>>`: `Ok(Some(status))` if exited,
    //           `Ok(None)` if still running, `Err(_)` on a wait error. Pattern-match all
    //           three.
    // Why:      React to each possible state.
    match child.try_wait() {
        Ok(Some(status)) => {
            // What:     `let code = status.code().unwrap_or(0);`. `code()` returns
            //           `Option<i32>` (None when the process was killed by a signal);
            //           `.unwrap_or(0)` substitutes 0 in that case.
            // Why:      A single integer exit code to propagate.
            let code = status.code().unwrap_or(0);

            // What:     `info!(...)`. Log the exit code.
            // Why:      Make shutdown visible.
            info!("hosted client exited with code {code}");

            // What:     `state.child_exit_code = Some(code);`. Record it for `main`.
            // Why:      `run` returns this as the program's exit code.
            state.child_exit_code = Some(code);
            state.shutdown_deadline = None;

            // What:     `state.child = None;`. Clear the handle so we stop polling.
            // Why:      The child is reaped; nothing left to wait on.
            state.child = None;

            // What:     `state.loop_signal.stop();`. Ask the event loop to end.
            // Why:      The fixture's job is done when the app exits.
            state.loop_signal.stop();
        }
        Ok(None) => {
            if force_shutdown {
                warn!("hosted client ignored close request; forcing shutdown");
                if let Err(error) = child.kill() {
                    warn!(%error, "failed to force-stop hosted client");
                }
                state.shutdown_deadline = None;
            }
        }
        Err(err) => {
            // What:     `warn!(...)`. Log the wait error.
            // Why:      Surface the unexpected failure.
            warn!("failed to wait on hosted client: {err}");

            // What:     `state.child = None;`. Stop polling a child we cannot wait on.
            // Why:      Avoid looping on a permanent error.
            state.child = None;
            state.shutdown_deadline = None;

            // What:     `state.loop_signal.stop();`. End the loop; without a hostable
            //           child there is nothing to do.
            // Why:      Fail closed rather than spin.
            state.loop_signal.stop();
        }
    }
}

/// Verifies hosted-child environment isolation.
#[cfg(test)]
#[path = "child_tests.rs"]
mod tests;
