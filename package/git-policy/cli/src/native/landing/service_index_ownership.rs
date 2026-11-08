//! What:
//!  Whether the file at the path of an index lock is still the file this holder placed there,
//! and the record that a lock could not be settled.
//! Why:
//!  A holder releases or removes its lock by path.
//! The staging file's descriptor is kept open for as long as the hold lasts,
//! so the inode it names cannot be freed and numbered again,
//! and asking whether the path still names it needs a `stat` and a `pread`,
//! never a new descriptor.
//! A check that opened the lock by path failed with `EMFILE` exactly when the service was out of descriptors,
//! and a failed check was read as "another program's lock",
//! which left a lock naming the live service behind.
//! A question that cannot be answered is neither answer:
//! the lock is kept and the service ends after its batch,
//! so that the restart reclaims it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ownership = await inspectOwnership(lockPath, heldHandle, marker) // 'owned' | 'replaced' | Error
//! ```

use crate::landing_state_carried_by_its_task::with_the_landing_state;
use std::os::unix::fs::{FileExt, MetadataExt};
use std::path::Path;

/// What the lock at a path is to the holder that placed it.
#[derive(Debug)]
pub enum Ownership {
    /// The path names the file the holder placed,
    /// holding its marker.
    Owned,
    /// The path names nothing,
    /// another file,
    /// or the same file with other content:
    /// the lock is not the holder's to release or remove.
    Replaced,
    /// The question could not be answered.
    Unreadable(std::io::Error),
}

/// What:
///  Whether the lock at `lock` is still the file `held` names,
/// holding exactly `marker`.
/// Why:
///   The same inode with the same content is the holder's own lock;
/// a different inode,
/// or none,
/// is another taker's,
/// and the inode cannot be a reused number while `held` pins it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (await stat(lock)).ino === (await held.stat()).ino && (await held.read()) === marker
/// ```
pub fn inspect_ownership(lock: &Path, held: &std::fs::File, marker: &str) -> Ownership {
    let ours = match held.metadata() {
        Ok(metadata) => metadata,
        Err(error) => return Ownership::Unreadable(error),
    };
    let on_path = match std::fs::metadata(lock) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ownership::Replaced,
        Err(error) => return Ownership::Unreadable(error),
    };
    if on_path.dev() != ours.dev() || on_path.ino() != ours.ino() {
        return Ownership::Replaced;
    }
    // One byte more than the marker, so a longer text is told from it.
    let mut text: Vec<u8> = vec![0; marker.len() + 1];
    let mut filled: usize = 0;
    while filled < text.len() {
        match held.read_at(&mut text[filled..], filled as u64) {
            Ok(0) => break,
            Ok(count) => filled += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Ownership::Unreadable(error),
        }
    }
    return if &text[..filled] == marker.as_bytes() { Ownership::Owned } else { Ownership::Replaced };
}

/// Records that a lock this landing held could not be settled,
/// and stays where it is.
pub fn note_unsettled() {
    with_the_landing_state(|state| state.unsettled_locks.set(state.unsettled_locks.get() + 1));
}

/// How many locks the holders of this landing could not settle.
///  The service ends after the batch that leaves one,
/// and its restart reclaims the lock,
/// whose marker then names a process that is gone.
pub fn unsettled_locks() -> usize {
    return with_the_landing_state(|state| state.unsettled_locks.get());
}
