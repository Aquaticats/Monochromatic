//! What:
//!  A child this process started and handed to the signal relay:
//!  a signal a process sends to the wrapper reaches that child until the token is dropped,
//! and the
//!  caller's own wait owns the child meanwhile.
//! Why:
//!  A bounded wait owns its child (the LFS upload before a typed push waits under the push's stall window),
//! so the
//!  guard's own start-and-wait cannot be the caller there;
//! this token publishes the child for the whole wait and
//!  stops the descriptor pointing at it afterwards,
//! without the relay module growing past the package's length limit
//! (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // signal-relay-relayed-child.ts: RelayedChild, startRelayed, takeChild, Symbol.dispose
//! ```

/// The guard whose handlers the child is published to,
/// and the two doors onto the relay's descriptor.
use crate::signal_relay::{SignalGuard, publish_child, retire_child};
/// Starts that ride out a short shortage of processes.
use crate::start_retry::spawn_retrying_without_blocking;
/// The I/O error a refused start answers with,
/// and the command that is started.
use std::io;
use std::process::Command;

/// What:
///  A child this process started and gave to the relay's handlers:
/// a signal a process sends to the wrapper reaches this child until the token is dropped.
/// Why:
///  The LFS upload before a typed push owns its `git-lfs` child for the length of a bounded wait,
/// and that wait is the child's stall window rather than a fixed ceiling,
/// so the relay's own start-and-wait cannot be the caller.
/// The guard starts the child and hands it over still published,
/// and the token stops the relay pointing at it when the wait is over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RelayedChild { child?: Child; published: boolean; takeChild(): Child; [Symbol.dispose]() { retire(published); } }
/// ```
#[must_use = "the relay points at the child only while this token lives"]
pub struct RelayedChild {
    /// The child itself,
    /// owned here until the waiter takes it;
    /// `Option` stands in for "moved out",
    /// which the siblings `Box` and `Rc` cannot say.
    child: Option<std::process::Child>,
    /// Whether the handler's descriptor points at the child.
    published: bool,
}

/// What:
///  Start a child and publish it to this guard's handlers,
/// handing the caller a token that stops the relay pointing at it when dropped.
/// Why:
///   A caller that owns its own bounded wait still needs the process's signals to reach the child,
/// and only this module may point the handler's descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relayed = await guard.startRelayed(command); const child = relayed.takeChild();
/// ```
#[cfg(unix)]
impl SignalGuard {
    /// What:
    ///  Start `command`,
    /// publish it to this guard's handlers when this guard owns them,
    /// and hand it over.
    /// Why:
    ///   A guard that owns no handlers starts the same way and publishes nothing,
    /// as `run_child_awaiting` does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async startRelayed(command: Command): Promise<RelayedChild>
    /// ```
    pub async fn start_relayed(&self, command: &mut Command) -> io::Result<RelayedChild> {
        let started: std::process::Child = spawn_retrying_without_blocking(command).await?;
        let published: bool = self.relays() && publish_child(started.id());
        // The token holds the child and whether the descriptor points at it.
        return Ok(RelayedChild { child: Some(started), published });
    }
}

/// What:
///  The same start for a system with no signal relay:
/// the child runs and the caller waits,
/// with no handler to point.
/// Why:
///   Keeps the call site identical on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async startRelayed(command: Command): Promise<RelayedChild>
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
impl SignalGuard {
    /// What:
    ///  Start `command` and hand it over,
    /// publishing nothing.
    /// Why:
    ///   There is no relay on this system,
    /// so the wait is the whole job.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async startRelayed(command: Command): Promise<RelayedChild>
    /// ```
    pub async fn start_relayed(&self, command: &mut Command) -> io::Result<RelayedChild> {
        let started: std::process::Child = spawn_retrying_without_blocking(command).await?;
        // Nothing is published: this system has no handler.
        return Ok(RelayedChild { child: Some(started), published: false });
    }
}

/// Methods over a relayed child.
impl RelayedChild {
    /// What:
    ///  Take the child back for the wait that ends it.
    ///  The token keeps pointing the relay at it until it is dropped.
    /// Why:
    ///   The waiter needs the child itself,
    /// and the relay must cover the whole wait,
    /// not the start alone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// takeChild(): Child
    /// ```
    pub fn take_child(&mut self) -> Option<std::process::Child> {
        // `take()` moves the child out and leaves `None`, so the token no longer owns it but still retires the relay.
        return self.child.take();
    }
}

/// Stop the relay pointing at the child when the token goes.
impl Drop for RelayedChild {
    /// What:
    ///  Point the handler's descriptor away from the child,
    /// whose number the waiter has reaped by now.
    /// Why:
    ///   A descriptor left pointing at a reaped number must not receive a signal that a later child's number reaches.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { if (this.published) retire(); }
    /// ```
    fn drop(&mut self) {
        retire_child(self.published);
    }
}
