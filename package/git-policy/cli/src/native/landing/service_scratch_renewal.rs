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
//! The first renewal is at once and never after a whole span:
//! a tree is somebody's from the moment it is held,
//! and a tree that arrived already old (a drawn tree carries the age of the pool member it was drawn from,
//! a stocked one the age of the moment it was drawn) would otherwise be at the sweep's age for a whole span
//! before its holder marked it once.
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
///  The renewal of one tree a command holds,
///  which stops when this value goes,
///  and the claim on that tree a clock step cannot move.
/// Why:
///   The value lives as long as the tree does,
///  so a command that ends,
///  by answer,
///  by a stop or by a panic,
///  stops renewing,
///  and one that is killed stops with the process.
///  The claim is the half of that a sweep can read without arithmetics on the wall clock:
///  a modification time cannot tell a live tree from an abandoned one across a forward clock step,
///  and the claim can, because the kernel holds it and no clock moves it
///  (`tree_held_by_a_live_process.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renewal = startScratchRenewal(path); // stops on disposal, and holds the tree meanwhile
/// ```
#[derive(Debug)]
pub struct ScratchRenewal {
    /// The claim this command keeps on the tree while it lives,
    /// which is what a sweep reads before it takes the tree for abandoned.
    ///  Nothing reads it here: holding it is the whole of the claim,
    ///  and it ends with this value or with the process.
    _held: Option<crate::tree_held_by_a_live_process::HeldTree>,
    /// The message that ends the waiting thread,
    /// which no one needs to receive:
    /// the thread ends when the sender goes,
    /// and the process ends with it.
    stop: Sender<()>,
}

impl ScratchRenewal {
    /// What:
    ///  Starts renewing `path` every `every`,
    ///  the first renewal at once.
    /// Why:
    ///   A caller that cannot start a thread keeps the tree without a renewal,
    ///  which is what the wrapper did for its whole life until now;
    ///  the tree then ages out as an unrenewed one does.
    ///  The first beat is made before the waiting and not after it,
    ///  because a tree that arrived already old is a tree a sweep may take while its holder waits out the first span:
    ///  the mark is the whole of what says the tree is somebody's.
    ///  The claim is taken before the thread starts,
    ///  so a tree is known as held from the moment this value exists and not from the first beat:
    ///  a sweep that runs inside that span reads the claim and leaves the tree,
    ///  and a volume that gives no `flock` leaves the tree to the mark alone, as before.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ScratchRenewal.start(path, REFRESH_EVERY);
    /// ```
    pub fn start(path: &Path, every: Duration) -> Self {
        let held: Option<crate::tree_held_by_a_live_process::HeldTree> = crate::tree_held_by_a_live_process::hold(path);
        let (stop, waited): (Sender<()>, Receiver<()>) = channel();
        let renewed: PathBuf = path.to_path_buf();
        let started = std::thread::Builder::new().name(String::from("scratch-renewal")).spawn(move || {
            loop {
                renew(renewed.as_path());
                match waited.recv_timeout(every) {
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    // The value that started this renewal went, or the channel did: either way this command is done with the tree.
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });
        if let Err(error) = started {
            crate::diagnostic_log::debug(
                "monochromatic-git-policy-cli",
                format!("the scratch tree {} is not renewed while this command lives: {error}", path.display()).as_str(),
            );
        }
        return Self { _held: held, stop };
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
