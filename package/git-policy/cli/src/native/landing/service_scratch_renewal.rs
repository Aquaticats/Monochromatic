//! What:
//!  The renewal that keeps a commit's scratch tree from aging out while the command that owns it is alive.
//! Why:
//!  A sweep removes an entry of a scratch directory by its modification time,
//! which says nothing about whether the command that made it still runs,
//! and a commit can hold its tree for as long as a person leaves an editor open in it:
//! the step that runs real Git in the scratch repository is unwatched and unbounded
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "Timing record names every step";
//! open for the owner).
//! Renewing the tree's time while the command lives ties its age to that command's liveness,
//! with no lock and no record of the process,
//! and the renewal stops the moment the value that started it goes,
//! however the command ends.
//! `SCRATCH_REFRESH_EVERY` is a twenty-fourth of the stale age,
//! so a sweep that runs at any moment finds a live tree at most an hour old.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! using renewal = startScratchRenewal(tree, REFRESH_EVERY);
//! export const renew = (path: string) => { const s = statSync(path); utimesSync(path, s.atime, new Date()); };
//! ```

/// The span between two renewals.
use crate::scratch_location::SCRATCH_REFRESH_EVERY;
/// The call that sets an entry's modification time to now.
use crate::scratch_location::renew;
/// A path.
use std::path::{Path, PathBuf};
/// A span of time and the channel that stops the renewal.
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// What:
///  The renewal of one scratch tree,
///  which stops when this value goes.
/// Why:
///   The value lives as long as the commit's scratch directory does,
///  so a command that ends,
///  by answer,
///  by a stop or by a panic,
///  stops renewing,
///  and one that is killed stops with the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renewal = startScratchRenewal(path); // stops on disposal
/// ```
#[derive(Debug)]
pub struct ScratchRenewal {
    /// The message that ends the waiting thread,
    /// which no one needs to receive:
    /// the thread ends when the sender goes,
    /// and the process ends with it.
    stop: Sender<()>,
}

impl ScratchRenewal {
    /// What:
    ///  Starts renewing `path` every `every`.
    /// Why:
    ///   A caller that cannot start a thread keeps the tree without a renewal,
    ///  which is what the wrapper did for its whole life until now;
    ///  the tree then ages out as an unrenewed one does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ScratchRenewal.start(path, REFRESH_EVERY);
    /// ```
    pub fn start(path: &Path, every: Duration) -> Self {
        let (stop, waited): (Sender<()>, Receiver<()>) = channel();
        let renewed: PathBuf = path.to_path_buf();
        let started = std::thread::Builder::new().name(String::from("scratch-renewal")).spawn(move || {
            while let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = waited.recv_timeout(every) {
                renew(renewed.as_path());
            }
        });
        if let Err(error) = started {
            crate::diagnostic_log::debug(
                "monochromatic-git-policy-cli",
                format!("the scratch tree {} is not renewed while this command lives: {error}", path.display()).as_str(),
            );
        }
        return Self { stop };
    }

    /// What:
    ///  The pace a commit's scratch tree is renewed at,
    ///  which is the one the sweep's rules are written against.
    /// Why:
    ///   One number for the command's whole life,
    ///  so a tree is never more than an hour old while a commit holds it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const every = SCRATCH_REFRESH_EVERY;
    /// ```
    pub fn of_a_commit(path: &Path) -> Self {
        return Self::start(path, SCRATCH_REFRESH_EVERY);
    }
}

impl Drop for ScratchRenewal {
    /// Ends the waiting thread at its next beat,
    /// without waiting for it:
    /// a thread asleep for an hour must not hold a command that is ending.
    fn drop(&mut self) {
        // The sender is the signal; a send nobody waits for is not an error.
        let _sent: std::result::Result<(), std::sync::mpsc::SendError<()>> = self.stop.send(());
    }
}

/// The renewal and its pace are controlled in the test module.
#[cfg(test)]
#[path = "service_scratch_renewal_tests.rs"]
mod tests;
