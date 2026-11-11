//! What:
//!  The guard of a system with no signal relay:
//! it starts the child and waits for it,
//! and publishes nothing,
//!  because there is no handler to point.
//! Why:
//!  Callers keep one API on every platform (`signal_relay::SignalGuard`),
//! and Windows ends a console child differently and is
//!  not covered by these controls;
//! the platform's half stands in its own file so the relay's file passes the package's length limit
//!  (`rust/max-lines`).
//! UNTESTED ON WINDOWS:
//! no gate compiles or runs this module (the crate does not build for Windows;
//! README,
//! Requirements).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // signal-relay-without-relay.ts: class SignalGuard { static install(); runChild(command); }
//! ```

/// The I/O error type the install and the wait answer with.
use std::io;
/// How the child ended,
/// with the exit-code mapping the wrapper's exit uses.
use crate::forwarding::ChildOutcome;
/// The command that is started,
/// and the status it ends with.
use std::process::{Command, ExitStatus};

/// What:
///  Non-Unix guard:
/// no signal relay exists,
/// so the child runs and the wrapper waits.
/// Why:
///   Callers keep one API;
/// Windows ends a console child differently and is not covered by these controls.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub struct SignalGuard;

/// Methods over the non-Unix guard.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
impl SignalGuard {
    /// What:
    ///  Return a guard that relays nothing.
    /// Why:
    ///   Keeps the call site identical on every platform.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static install(): SignalGuard
    /// ```
    pub fn install() -> io::Result<Self> {
        return Ok(Self);
    }

    /// What:
    ///  Start `command` and wait for it.
    /// Why:
    ///   Without relaying,
    /// the wait is the whole job.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async runChild(command): Promise<ChildOutcome>
    /// ```
    pub fn run_child(&self, command: &mut Command) -> io::Result<ChildOutcome> {
        let mut child: std::process::Child = crate::start_retry::RetryStart::spawn_retrying(command)?;
        let status: ExitStatus = child.wait()?;
        return Ok(crate::forwarding::outcome_of(status));
    }
}
