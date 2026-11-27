//! What:
//!  The kernel's own destroy of the subvolume one record names,
//!  with the words for what became of it.
//! Why:
//!  A subvolume destroy is one kernel call,
//!  and the record names the subvolume by the id that stands there,
//!  because a path names whatever is at it when the call runs.
//!  A path that is gone is nothing to delete:
//!  either the rename never happened and the removal destroyed the subvolume in place,
//!  or a service already deleted it,
//!  so the record is done.
//!  A path that holds another subvolume than the record's id is not this record's to remove,
//!  so a replacement is left as it is and named in the failure.
//!  A mount that does not allow an unprivileged destroy is a failure the person must hear,
//!  because the files stay;
//!  anything else may pass and is tried again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! destroyTheSubvolumeOf(item): Verdict
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{
    open_directory_at,
    open_root,
};
use crate::fs_btrfs_ioctl::{
    snapshot_destroy_by_id,
    snapshot_destroy_in,
    subvolume_id,
};
use crate::service_work::Verdict;
use crate::work_item::WorkItem;
use std::fs::File;
use std::path::PathBuf;

/// Import the tag every log line of this subject carries.
use crate::worktree::worktree_remove_hand_over_begun_record::TAG;

/// What:
///  The subvolume id an item's object name holds,
/// or nothing when the name is not one.
/// Why:
///   The id names one subvolume for good,
/// while a name names whatever is there when the call runs,
/// so the destroy must
///  name the id the record holds and never the name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = (oid) => /^[0-9a-fA-F]{40}$/.test(oid) ? BigInt('0x' + oid) : undefined;
/// ```
pub(crate) fn recorded_subvolume(oid: &str) -> Option<u64> {
    if oid.len() != 40 || !oid.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    return u64::from_str_radix(oid, 16).ok();
}

/// Whether an errno from the destroy by id means this kernel lacks the call,
/// so that the destroy by name below the held parent is the way left.
fn by_id_unavailable(errno: i32) -> bool {
    return errno == libc::ENOTTY || errno == libc::EOPNOTSUPP || errno == libc::EINVAL;
}

/// What:
///  The destroy of the subvolume one record names,
///  once the record is known to name a subvolume and not a whole worktree.
/// Why:
///  The destroy names the subvolume by the id the record holds when the kernel allows it,
///  so a replaced ancestor cannot redirect it,
///  and by name below the opened parent otherwise,
///  once the id that stands there is proven to be the record's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// destroyTheSubvolumeOf(item): Verdict
/// ```
pub(crate) fn destroy_the_subvolume_of(item: &WorkItem) -> Verdict {
    let path: PathBuf = PathBuf::from(item.parent.as_str());
    if !path.is_absolute() {
        return Verdict::Failed(format!(
            "the record names no absolute path to delete: {}",
            item.parent
        ));
    }
    let Some(parent_path) = path.parent() else {
        return Verdict::Failed(format!(
            "the record names no directory to delete below: {}",
            item.parent
        ));
    };
    let Some(name) = path.file_name() else {
        return Verdict::Failed(format!(
            "the record names no directory to delete: {}",
            item.parent
        ));
    };
    let Some(recorded): Option<u64> = recorded_subvolume(item.oid.as_str()) else {
        return Verdict::Failed(format!(
            "the record of {} names no subvolume id ({:?}), so nothing proves which subvolume stands there and it was left as it is",
            path.display(),
            item.oid
        ));
    };
    let parent: File = match open_root(parent_path) {
        Ok(opened) => opened,
        // The directory that held it is gone, so nothing stands at the name below it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Verdict::Done,
        Err(error) => {
            return Verdict::Retry(format!(
                "the directory {} cannot be opened ({error})",
                parent_path.display()
            ));
        }
    };
    // The name still has to lead to the directory the record names, or the delete would act on whatever took its place.
    let standing: File = match open_directory_at(&parent, name) {
        Ok(opened) => opened,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.kind() == std::io::ErrorKind::NotADirectory =>
        {
            debug(
                TAG,
                format!("{} is gone already; nothing to delete", path.display()).as_str(),
            );
            return Verdict::Done;
        }
        Err(error) => {
            return Verdict::Retry(format!("{} cannot be opened ({error})", path.display()));
        }
    };
    match subvolume_id(&standing, path.as_path()) {
        Ok(id) if id == recorded => {}
        Ok(id) => {
            return Verdict::Failed(format!(
                "{} holds the subvolume {id}, and the record names the subvolume {recorded}, so it was left as it is; remove it by hand if it is yours",
                path.display()
            ));
        }
        Err(error) if error.errno == libc::ENOENT => return Verdict::Done,
        Err(error) if by_id_unavailable(error.errno) => {
            return Verdict::Failed(format!(
                "{} cannot be identified as the subvolume {recorded} on this kernel ({error}), so it was left as it is",
                path.display()
            ));
        }
        Err(error) => {
            return Verdict::Retry(format!(
                "the subvolume of {} cannot be identified ({error})",
                path.display()
            ));
        }
    }
    match snapshot_destroy_by_id(&parent, recorded, path.as_path()) {
        Ok(()) => return Verdict::Done,
        Err(error) if error.errno == libc::EPERM => {
            return Verdict::Failed(format!(
                "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand, and remount it with user_subvol_rm_allowed to let the wrapper do it",
                path.display()
            ));
        }
        Err(error) if error.errno == libc::ENOENT => return Verdict::Done,
        Err(error) if by_id_unavailable(error.errno) => {
            debug(
                TAG,
                format!(
                    "the destroy of {} by id is not available ({error}); destroying it by name",
                    path.display()
                )
                .as_str(),
            );
        }
        Err(error) => {
            return Verdict::Retry(format!(
                "the subvolume of {} cannot be destroyed ({error})",
                path.display()
            ));
        }
    }
    return match snapshot_destroy_in(&parent, name, parent_path) {
        Ok(()) => Verdict::Done,
        Err(error) if error.errno == libc::ENOENT => Verdict::Done,
        Err(error) if error.errno == libc::EPERM => Verdict::Failed(format!(
            "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand",
            path.display()
        )),
        Err(error) => Verdict::Retry(format!(
            "the subvolume of {} cannot be destroyed ({error})",
            path.display()
        )),
    };
}
