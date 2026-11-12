//! What:
//!  The evidence a command records before it makes a snapshot:
//! the source subvolume the snapshot is taken from,
//! and the instant before the creation.
//! Why:
//!  An object's identity cannot be read before the object exists,
//! so a kill between the creation of a snapshot and the record of its identity
//!  leaves a record that names no object.
//! A btrfs subvolume root is the shape that creation makes,
//! and it is the shape a snapshot another party took
//!  before this record has too,
//! with every entry below the recorded instant and none of the wrapper's own,
//! so the shape alone let a reaping delete a person's
//!  snapshot and the only copy of a file it held.
//! The filesystem writes facts of its own into every subvolume,
//! which no other party can forge at will,
//! and `BTRFS_IOC_GET_SUBVOL_INFO` returns them (`fs/btrfs/ioctl.c`,
//! `_btrfs_ioctl_get_subvol_info`):
//!  the subvolume it is a snapshot of (`parent_uuid`,
//! written by `create_pending_snapshot` in `fs/btrfs/transaction.c` from the
//!  source's own UUID),
//! and the instant it was created (`otime`,
//! from `current_time()`,
//! which is the coarse realtime clock,
//! `fs/inode.c`).
//! A folder that stood at the name before this record was written is older than that instant,
//! so a person's snapshot keeps itself,
//! and the one this command's creation makes is not.
//!
//! A transaction number is not one of those facts,
//! which is why none is compared here any more.
//! `create_pending_snapshot` does stamp `otransid` with the transaction that ran,
//! but the number a record could hold before the creation is
//!  a subvolume's `generation`,
//! and `btrfs_update_root` sets that to the generation of the tree's own root node
//! (`fs/btrfs/root-tree.c`),
//! so a subvolume's generation only advances with the transactions that write its own tree.
//! It can therefore be older than the filesystem's last commit,
//! and a snapshot a person makes after that commit carries a later creation transaction and was
//!  proved to be the wrapper's:
//! the deletion this record exists to prevent.
//! A committed transaction would have to be forced to tell the two apart,
//! and a forced commit is a flush of the whole filesystem on the add's waiting path
//! (`SPEC.md`,
//! "Local durability follows what the remote already holds").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const origin = originOfTheSource(sourceRoot); // { parent, createdAfter } | undefined
//! proveMadeFromTheRecordedSource(container, origin); // ok, or the reason a person reads
//! ```

use std::path::Path;
use std::time::{Duration, SystemTime};

/// What:
///  The source a snapshot is about to be taken from,
/// and the instant a snapshot that creation makes cannot be older than.
/// Why:
///   The record holds it before the object exists,
/// so a later command compares what the filesystem wrote with what this command knew before it wrote anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotOrigin = { parent: Uint8Array /* 16 bytes of UUID */; createdAfter: Date };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotOrigin {
    /// The UUID of the source subvolume the snapshot is taken from.
    pub parent: [u8; 16],
    /// The instant before the creation,
    /// read from the coarse realtime clock,
    /// which is the clock the filesystem stamps a subvolume's creation with.
    pub created_after: SystemTime,
}

/// What:
///  The instant the coarse realtime clock holds now.
/// Why:
///   A subvolume's recorded creation instant comes from `current_time()` (`fs/inode.c`),
/// which reads `CLOCK_REALTIME_COARSE`;
/// comparing it with a fine reading of the wall clock would compare two clocks whose
///  answers differ by up to one tick,
/// and the wrapper's own snapshot would be kept because it looked older than the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const coarseNow = () => new Date(Math.floor(clock_gettime(CLOCK_REALTIME_COARSE) * 1000));
/// ```
#[cfg(target_os = "linux")]
fn coarse_now() -> SystemTime {
    // SAFETY: `now` is a `timespec` the call fills in, and the clock id is one Linux defines.
    let mut now: libc::timespec = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    let called: libc::c_int = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME_COARSE, &mut now) };
    assert_eq!(called, 0, "the coarse realtime clock is readable: {}", std::io::Error::last_os_error());
    return SystemTime::UNIX_EPOCH + Duration::new(now.tv_sec as u64, now.tv_nsec as u32);
}

/// What:
///  What the filesystem says about the source,
/// or nothing where it cannot be read.
/// Why:
///   A source that is no btrfs subvolume cannot be snapshotted,
/// and a source whose record cannot be read answers no source;
/// the record then names no origin,
/// and a folder a kill leaves is
///  reported instead of removed,
/// which is the safe direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function originOfTheSource(sourceRoot: string): SnapshotOrigin | undefined { try { return { parent: subvolumeInfo(sourceRoot).uuid, createdAfter: coarseNow() }; } catch { return undefined; } }
/// ```
#[cfg(target_os = "linux")]
pub fn origin_of_the_source(source_root: &Path) -> Option<SnapshotOrigin> {
    let source: crate::fs_btrfs_ioctl::SubvolumeInfo = crate::fs_btrfs_ioctl::subvolume_info(source_root).ok()?;
    // The instant is read last, so it is as close to the creation below as the record can make it: nothing a person
    // made before this call can carry it.
    return Some(SnapshotOrigin { parent: source.uuid, created_after: coarse_now() });
}

/// `origin_of_the_source` where btrfs ioctls are not built.
#[cfg(not(target_os = "linux"))]
pub fn origin_of_the_source(_source_root: &Path) -> Option<SnapshotOrigin> {
    return None;
}

/// What:
///  The reason a person reads when the folder's own record cannot be read.
/// Why:
///  A look that failed is not an answer of absence:
/// the folder is kept and the sentence names what could not be read,
/// and one named function keeps the words where a reader looks for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordsForAnUnreadableSnapshot = (error) => `nothing shows when the snapshot there was made (${error})`;
/// ```
fn words_for_an_unreadable_snapshot(error: crate::fs_primitive_error::PrimitiveError) -> String {
    return format!("nothing shows when the snapshot there was made ({error})");
}

/// What:
///  Whether the snapshot standing at the container's name is one the filesystem says this source made after the
/// recorded instant,
/// or the reason a person reads when it is not.
/// Why:
///   This is the proof for the record a command writes before its snapshot exists,
/// which names no object:
/// the folder must be a subvolume root,
/// taken from the recorded source,
/// and created no earlier than the instant the record holds.
/// A manual directory answers with the subvolume that holds it,
/// which is not a snapshot of that source;
/// another party's snapshot answers with its own source and its own creation instant,
/// and one that stood there before the record was written is older than it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function proveMadeFromTheRecordedSource(container: string, origin: SnapshotOrigin): void { if (!isSubvolumeRoot(container)) throw '...'; const info = subvolumeInfo(container); if (info.parentUuid !== origin.parent) throw '...'; if (info.created < origin.createdAfter) throw '...'; }
/// ```
#[cfg(target_os = "linux")]
pub fn prove_made_from_the_recorded_source(container: &Path, origin: &SnapshotOrigin) -> Result<(), String> {
    if !crate::fs_kind::is_subvolume_root(container).unwrap_or(false) {
        return Err(String::from(
            "the folder there now is not a btrfs subvolume root, and the record of the object its creation made names no object",
        ));
    }
    let made: crate::fs_btrfs_ioctl::SubvolumeInfo = crate::fs_btrfs_ioctl::subvolume_info(container).map_err(words_for_an_unreadable_snapshot)?;
    if made.parent_uuid != origin.parent {
        return Err(String::from(
            "the snapshot there now was not taken from the source this command copied, and the record of the object its creation made names no object",
        ));
    }
    if made.created < origin.created_after {
        return Err(String::from(
            "the snapshot there now came into being before this command wrote the record of its creation, so it is not the snapshot that creation made",
        ));
    }
    return Ok(());
}

/// `prove_made_from_the_recorded_source` where btrfs ioctls are not built.
#[cfg(not(target_os = "linux"))]
pub fn prove_made_from_the_recorded_source(_container: &Path, _origin: &SnapshotOrigin) -> Result<(), String> {
    return Err(String::from("this build cannot read when the folder was made, so nothing in it was removed"));
}

/// Snapshot origin controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_snapshot_origin_tests.rs"]
mod tests;

/// The measurement of what the origin costs an add's path stays out of the release executable too.
#[cfg(test)]
#[path = "worktree_copy_stage_snapshot_origin_measurement_tests.rs"]
mod measurement_tests;
