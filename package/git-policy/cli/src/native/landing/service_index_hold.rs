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

use crate::service_index_ownership::{Ownership, note_unsettled};
use crate::service_index_same::same_index;
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
    /// Removes the lock the holder owns.
    remove: fn(&Path) -> std::io::Result<()>,
    armed: bool,
}

impl IndexLock {
    /// A hold on `lock`,
    /// which was made from the file `held` and carries `marker`.
    pub(crate) fn placed(
        lock: PathBuf,
        index: &Path,
        marker: String,
        held: std::fs::File,
        calls: (fn(&Path, &std::fs::File, &str) -> Ownership, fn(&Path) -> std::io::Result<()>),
    ) -> IndexLock {
        return IndexLock { lock, index: index.to_path_buf(), marker, held, inspect: calls.0, remove: calls.1, armed: true };
    }

    /// The live index this lock guards.
    ///  A function that takes a borrow of the lock can only be called by the holder, and works on files beside this index.
    pub fn index(&self) -> &Path {
        return self.index.as_path();
    }

    /// What:
    ///  Removes the lock the holder owns.
    /// Why:
    ///   A lock that is already gone is removed.
    /// Any other failure leaves a lock that names this live process,
    /// which no later acquisition reclaims,
    /// so it is said in the log and counted as unsettled:
    /// the service ends after the batch and the restart reclaims the lock.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { await rm(lock) } catch (e) { if (e.code !== 'ENOENT') { log(e); unsettled++ } }
    /// ```
    fn remove_owned(&self) -> std::io::Result<()> {
        return match (self.remove)(&self.lock) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                crate::service_error!("the index lock {} is this service's and cannot be removed ({error}); it stays, and the service ends after this batch so that the restart reclaims it",
                    self.lock.display()
                );
                note_unsettled();
                Err(error)
            }
            _ => Ok(()),
        };
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
    pub fn release_with(self, built: &Path) -> Result<(), std::io::Error> {
        return self.release_with_record(built, &|| { return Ok(()); });
    }

    /// What:
    ///  `release_with`, calling `record` after the index is installed and before the lock is given up.
    /// Why:
    ///   Once the lock is gone another program may stage into the installed index, so a restart that would replay the
    ///  installation then loses what it staged. The caller records in `record` (durably) that the installation is done, while the
    ///  lock still keeps every other writer out: a death before the record leaves the lock for the restart to reclaim, and the
    ///  replay it does is of an index nobody could have changed; a death after it leaves a record that stops the replay.
    ///  A record that cannot be made keeps the lock in place and ends the service after its batch, as an unanswerable ownership
    ///  question does, so an installed index is never left unlocked and replayable.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.releaseWithRecord(built, record) // rename(built, index); record(); unlink(lock)
    /// ```
    ///
    /// # Errors
    ///
    /// As `release_with`, and the error of `record`, with the lock left in place.
    pub fn release_with_record(self, built: &Path, record: &dyn Fn() -> Result<(), std::io::Error>) -> Result<(), std::io::Error> {
        return self.release_flushed_with_record(built, false, record);
    }

    /// What:
    ///  `release_with_record` for a `built` the caller knows is already on the disk as it is.
    /// Why:
    ///   The in-process update flushes its file before it renames it into place, and nothing writes to it after. Flushing it
    ///  again would cost a flush on the landing's path for no guarantee.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.releaseFlushedWithRecord(built, alreadyFlushed, record)
    /// ```
    ///
    /// # Errors
    ///
    /// As `release_with_record`.
    pub fn release_flushed_with_record(mut self, built: &Path, flushed: bool, record: &dyn Fn() -> Result<(), std::io::Error>) -> Result<(), std::io::Error> {
        match self.ownership() {
            Ownership::Owned => {}
            Ownership::Replaced => {
                self.armed = false;
                let message: String = format!(
                    "the index lock {} is no longer the file this service placed: another program removed or replaced it, so the index was not replaced and the lock was left to that program",
                    self.lock.display()
                );
                crate::service_error!("{message}");
                return Err(std::io::Error::other(message));
            }
            Ownership::Unreadable(error) => {
                let message: String = format!(
                    "the index lock {} could not be checked ({error}), so the index was not replaced and the lock was left in place for the restart to reclaim",
                    self.lock.display()
                );
                crate::service_error!("{message}");
                return Err(std::io::Error::other(message));
            }
        }
        // No index existed and nothing was staged when `built` is absent: there is nothing to install. A copy that holds the live
        //  index's own bytes is not installed either: the live file stays the same file, with the access list it has, and no
        //  flush or rename is spent on it. Any other copy is flushed after its last writer (the in-process update, Git's own
        //  `reset`, or the carried conflict stages, each of which may leave unflushed bytes) and before the rename, so a
        //  completion that is recorded durably never certifies an index whose contents are not on the disk. A flush that fails
        //  installs nothing, as a rename that fails does.
        if built.exists() && !same_index(built, &self.index) {
            if !flushed {
                let synced: std::io::Result<()> = if crate::service_fault::fails("sync-built-index") {
                    Err(std::io::Error::other("the flush of the new index was made to fail by a control"))
                } else {
                    std::fs::File::open(built).and_then(|file| { return crate::service_durable::flush_index(&file, "built-index"); })
                };
                if let Err(error) = synced {
                    crate::service_error!("the new index {} could not be flushed to the disk ({error}), so the index was not replaced", built.display());
                    return Err(error);
                }
            }
            std::fs::rename(built, &self.index)?;
        }
        // From the installation on, the lock is given up only by this function's own removal: a failure or an unwinding panic
        //  in what follows leaves it for the restart, because an installed index must never be left unlocked and replayable.
        self.armed = false;
        let durable: std::io::Result<()> = self.index.parent().map_or(Ok(()), |directory| { return crate::service_durable::flush_directory(directory, "index-dir"); });
        let recorded: std::io::Result<()> = durable.and_then(|()| { return record(); });
        if let Err(error) = recorded {
            crate::service_error!(
                "the index lock {} stays: the installed index could not be made durable and recorded as complete ({error}), and the service ends after this batch so that the restart reclaims the lock",
                self.lock.display()
            );
            note_unsettled();
            return Err(error);
        }
        self.armed = true;
        self.remove_owned()?;
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
                let _removed = self.remove_owned();
            }
            Ownership::Replaced => {}
            Ownership::Unreadable(error) => {
                crate::service_error!("the index lock {} could not be checked ({error}) and stays; the service ends after this batch so that the restart reclaims it",
                    self.lock.display()
                );
                note_unsettled();
            }
        }
    }
}

/// Controls for what the release installs stay out of the release executable.
#[cfg(test)]
#[path = "service_index_hold_tests.rs"]
mod tests;
