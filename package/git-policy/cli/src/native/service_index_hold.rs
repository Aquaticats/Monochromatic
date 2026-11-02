//! What:
//!  The service's hold on one index lock after it is placed:
//! the check that the lock is still the holder's,
//! the release that installs the new index,
//! and the removal of a lock that is not released.
//! Why:
//!  A holder removes its lock by path,
//! so it first asks whether the path still names the file it placed (`service_index_ownership.rs`).
//! A question that cannot be answered is neither answer:
//! the lock stays,
//! the failure is said in the log,
//! and the service ends after its batch so that the restart reclaims the lock.
//! The module is split from `service_index_lock.rs`,
//! which re-exports `IndexLock`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await lock.releaseWith(builtIndex) // rename(built, index); unlink(lock)
//! ```

use super::service_index_ownership::{Ownership, note_unsettled};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How many times a question about the lock that fails is asked before it is given up,
/// and the pause between them.
const SETTLE_LOOKS: u32 = 5;
const SETTLE_PAUSE: Duration = Duration::from_millis(10);

/// The service's hold on one index lock;
/// removed unless released.
#[derive(Debug)]
pub struct IndexLock {
    lock: PathBuf,
    index: PathBuf,
    /// The text this holder placed in the lock.
    marker: String,
    /// The staging file the lock was made from,
    /// kept open:
    /// it names the lock's inode,
    /// which cannot be numbered again while it is open.
    held: std::fs::File,
    /// Asks whether the lock is still the holder's.
    inspect: fn(&Path, &std::fs::File, &str) -> Ownership,
    armed: bool,
}

impl IndexLock {
    /// A hold on `lock`,
    /// which was made from the file `held` and carries `marker`.
    pub(super) fn placed(lock: PathBuf, index: &Path, marker: String, held: std::fs::File, inspect: fn(&Path, &std::fs::File, &str) -> Ownership) -> IndexLock {
        return IndexLock { lock, index: index.to_path_buf(), marker, held, inspect, armed: true };
    }

    /// Whether the lock is still the holder's.
    ///  A question that fails is asked again a few times,
    /// because the failures that are not final end with the pressure
    /// that caused them.
    fn ownership(&self) -> Ownership {
        let mut answer: Ownership = (self.inspect)(&self.lock, &self.held, self.marker.as_str());
        for _ in 1..SETTLE_LOOKS {
            if !matches!(answer, Ownership::Unreadable(_)) {
                break;
            }
            std::thread::sleep(SETTLE_PAUSE);
            answer = (self.inspect)(&self.lock, &self.held, self.marker.as_str());
        }
        return answer;
    }

    /// What:
    ///  Installs `built`,
    /// which sits in the index's own directory,
    /// as the index by one rename,
    /// then gives the lock up.
    /// Why:
    ///   A reader sees the old index or the new one and never a part of either,
    /// as with Git's rename of its lock.
    /// The lock,
    /// still marked,
    /// is removed only after the rename,
    /// so a kill between the two leaves a lock the next instance knows as the service's own.
    /// Copying the new index over the lock first left a lock without the marker for as long as the copy ran,
    /// and a kill then left a lock no restart would take over.
    /// Before it installs anything the holder checks that the path still names the file it placed;
    /// a replacement that check sees keeps the index as it was and leaves the other program's lock alone.
    /// The check is a preflight,
    /// not a guard:
    /// a replacement after it is not seen,
    /// so the lifecycle that lets one service at a time take the lock stays required (`SPEC.md`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.releaseWith(built) // rename(built, index); unlink(lock)
    /// ```
    ///
    /// # Errors
    ///
    /// The storage failure when the new index cannot be put in place;
    /// the lock is then removed and the old index stays.
    /// An error naming the lock when the check sees it replaced:
    /// it and the old index stay.
    /// An error naming the lock when the check cannot be made:
    /// the lock stays,
    /// and the service ends after its batch so that the restart reclaims it.
    pub fn release_with(mut self, built: &Path) -> Result<(), std::io::Error> {
        match self.ownership() {
            Ownership::Owned => {}
            Ownership::Replaced => {
                self.armed = false;
                let message: String = format!(
                    "the index lock {} is no longer the file this service placed: another program removed or replaced it, so the index was not replaced and the lock was left to that program",
                    self.lock.display()
                );
                eprintln!("cli-git-service: {message}");
                return Err(std::io::Error::other(message));
            }
            Ownership::Unreadable(error) => {
                let message: String = format!(
                    "the index lock {} could not be checked ({error}), so the index was not replaced and the lock was left in place for the restart to reclaim",
                    self.lock.display()
                );
                eprintln!("cli-git-service: {message}");
                return Err(std::io::Error::other(message));
            }
        }
        // No index existed and nothing was staged when `built` is absent: there is nothing to install.
        if built.exists() {
            std::fs::rename(built, &self.index)?;
        }
        std::fs::remove_file(&self.lock)?;
        self.armed = false;
        return Ok(());
    }
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        match self.ownership() {
            Ownership::Owned => {
                let _removed = std::fs::remove_file(&self.lock);
            }
            Ownership::Replaced => {}
            Ownership::Unreadable(error) => {
                eprintln!(
                    "cli-git-service: the index lock {} could not be checked ({error}) and stays; the service ends after this batch so that the restart reclaims it",
                    self.lock.display()
                );
                note_unsettled();
            }
        }
    }
}
