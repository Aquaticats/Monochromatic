//! What:
//!  The evidence a command records before it makes a snapshot:
//! the source subvolume the snapshot is taken from,
//! and the transaction the filesystem had committed after the last durable write before it.
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
//! The filesystem writes two facts of its own into every subvolume,
//! which no other party can forge at will:
//!  the subvolume it is a snapshot of,
//! and the transaction that created it.
//! A snapshot made after the recording transaction therefore names this source as its parent and carries a later
//!  creation transaction,
//! and one made before it does not,
//! so the reaping can keep removing the stage a kill leaves while it can never match a snapshot that stood there
//!  first.
//! The recording transaction is the last commit the journal's own directory took part in,
//! which is the binding record the
//!  add writes immediately before this rung,
//! so nothing is committed here that the ladder would not commit anyway.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const origin = originOfTheSource(sourceRoot, commonDir); // { parent, after } | undefined
//! proveCreatedAfterTheRecord(container, origin); // ok, or the reason a person reads
//! ```

use std::path::Path;

/// What:
///  The source a snapshot is about to be taken from,
/// and the transaction after which it must have been created to be this command's.
/// Why:
///   The record holds it before the object exists,
/// so a later command compares what the filesystem wrote with what this command knew before it wrote anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotOrigin = { parent: Uint8Array /* 16 bytes of UUID */; after: bigint };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotOrigin {
    /// The UUID of the source subvolume the snapshot is taken from.
    pub parent: [u8; 16],
    /// The latest committed transaction the filesystem held when this was read,
    /// which every snapshot made before it has a creation transaction at or below.
    pub after: u64,
}

/// What:
///  What the filesystem says about the source and the journal's directory,
/// or nothing where it cannot be read.
/// Why:
///   A source that is no btrfs subvolume cannot be snapshotted,
/// and a journal directory that cannot be read answers no transaction;
/// the record then names no origin,
/// and a folder a kill leaves is
///  reported instead of removed,
/// which is the safe direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function originOfTheSource(sourceRoot: string, commonDir: string): SnapshotOrigin | undefined { try { return { parent: subvolumeInfo(sourceRoot).uuid, after: subvolumeInfo(journalRoot(commonDir)).generation }; } catch { return undefined; } }
/// ```
#[cfg(target_os = "linux")]
pub fn origin_of_the_source(source_root: &Path, common_dir: &Path) -> Option<SnapshotOrigin> {
    let source: crate::fs_btrfs_ioctl::SubvolumeInfo = crate::fs_btrfs_ioctl::subvolume_info(source_root).ok()?;
    let journal: crate::fs_btrfs_ioctl::SubvolumeInfo = crate::fs_btrfs_ioctl::subvolume_info(crate::worktree_copy_journal::journal_root(common_dir).as_path()).ok()?;
    return Some(SnapshotOrigin { parent: source.uuid, after: journal.generation });
}

/// `origin_of_the_source` where btrfs ioctls are not built.
#[cfg(not(target_os = "linux"))]
pub fn origin_of_the_source(_source_root: &Path, _common_dir: &Path) -> Option<SnapshotOrigin> {
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
/// recorded transaction,
/// or the reason a person reads when it is not.
/// Why:
///   This is the proof for the record a command writes before its snapshot exists,
/// which names no object:
/// the folder must be a subvolume root,
/// taken from the recorded source,
/// and created in a later transaction than the one the record holds.
/// A manual directory answers with the subvolume that holds it,
/// which is not a snapshot of that source;
/// another party's snapshot answers with its own creation transaction,
/// which is at or
///  below the recorded one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provedCreatedAfterTheRecord(container: string, origin: SnapshotOrigin): void { if (!isSubvolumeRoot(container)) throw '...'; const info = subvolumeInfo(container); if (info.parentUuid !== origin.parent) throw '...'; if (info.transidAtCreation <= origin.after) throw '...'; }
/// ```
#[cfg(target_os = "linux")]
pub fn proved_created_after_the_record(container: &Path, origin: &SnapshotOrigin) -> Result<(), String> {
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
    if made.transid_at_creation <= origin.after {
        return Err(format!(
            "the snapshot there now was made in transaction {} or before, and the record of the object its creation made names transaction {} as the one it followed, so this is not the snapshot that creation made",
            made.transid_at_creation, origin.after
        ));
    }
    return Ok(());
}

/// `proved_created_after_the_record` where btrfs ioctls are not built.
#[cfg(not(target_os = "linux"))]
pub fn proved_created_after_the_record(_container: &Path, _origin: &SnapshotOrigin) -> Result<(), String> {
    return Err(String::from("this build cannot read when the folder was made, so nothing in it was removed"));
}

/// Snapshot origin controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_snapshot_origin_tests.rs"]
mod tests;
