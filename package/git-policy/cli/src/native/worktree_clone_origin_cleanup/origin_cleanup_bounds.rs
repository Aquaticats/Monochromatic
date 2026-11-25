//! What:
//!  The byte bounds the origin cleanup record's reader enforces and its writer proves:
//!  the record's 9,217 bytes,
//!  and the widest value each field may carry.
//! Why:
//!  The design computes the bound field by field and applies no rounding
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The bound"),
//!  so a writer refuses to publish a record its own reader would refuse,
//!  which is the rule the receipts and the plan already hold to.
//!  Each width rests on a named limit:
//!  255 bytes for a ZFS dataset name,
//!  58 for the snapshot name,
//!  twenty digits for a `uint64`,
//!  four for an errno,
//!  and 4,095 bytes for a slot path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const MAXIMUM = 9_217; const widths = new Map([['originDataset', 255], ...]);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_words::{
    ORIGIN_CLEANUP_HEADER, ORIGIN_CLEANUP_NAME_BYTES,
};

/// The most bytes one record may take,
/// which is its header and every field at its widest.
///
/// Why:
///  The design's own sum,
///  kept as the reader's one cap:
///  the header at 57 bytes,
///  the state's seven lines at 192,
///  and the identity's eight lines at 8,968.
pub const ORIGIN_CLEANUP_MAXIMUM_BYTES: u64 = 9_217;

/// The most digits a canonical `uint64` takes.
pub const NUMBER_MAXIMUM_DIGITS: usize = 20;

/// The most digits an errno takes.
pub const ERRNO_MAXIMUM_DIGITS: usize = 4;

/// The longest one ZFS dataset name the kernel accepts,
/// less its terminator.
pub const ZFS_NAME_MAXIMUM_BYTES: usize = 255;

/// The longest slot path a record carries,
/// which is the kernel's own path bound.
pub const SLOT_PATH_MAXIMUM_BYTES: usize = 4_095;

/// The longest snapshot name this record's grammar admits,
/// which is the prefix and the add's nonce.
pub const ORIGIN_SNAPSHOT_MAXIMUM_BYTES: usize = ORIGIN_CLEANUP_NAME_BYTES;

/// The longest instant a record carries,
/// which is an ISO 8601 second with a six-byte offset.
pub const INSTANT_MAXIMUM_BYTES: usize = 26;

/// The header line and its line feed as they are written.
pub fn header_bytes() -> usize {
    return ORIGIN_CLEANUP_HEADER.len() + 1;
}

/// The widest value each key may carry,
/// in the order the record writes them.
///
/// Why:
///  A writer refuses a value wider than its key's bound,
///  so one rule proves a record cannot carry a field its reader would refuse.
pub const FIELD_WIDEST_BYTES: [(&str, usize); 15] = [
    ("state", 7),
    ("startedAt", INSTANT_MAXIMUM_BYTES),
    ("attemptedAt", INSTANT_MAXIMUM_BYTES),
    ("blockedReason", 10),
    ("blockedErrno", ERRNO_MAXIMUM_DIGITS),
    ("numClones", NUMBER_MAXIMUM_DIGITS),
    ("userRefs", NUMBER_MAXIMUM_DIGITS),
    ("originDataset", ZFS_NAME_MAXIMUM_BYTES),
    ("originDatasetGuid", NUMBER_MAXIMUM_DIGITS),
    ("originSnapshot", ORIGIN_SNAPSHOT_MAXIMUM_BYTES),
    ("originSnapshotGuid", NUMBER_MAXIMUM_DIGITS),
    ("originSnapshotCreatetxg", NUMBER_MAXIMUM_DIGITS),
    ("cloneDataset", ZFS_NAME_MAXIMUM_BYTES),
    ("cloneGuid", NUMBER_MAXIMUM_DIGITS),
    ("slotPath", 2 * SLOT_PATH_MAXIMUM_BYTES),
];

/// The widest one key's value may be,
/// or nothing when the grammar has no such key.
pub fn field_maximum_bytes(key: &str) -> Option<usize> {
    for (named, widest) in FIELD_WIDEST_BYTES {
        if named == key {
            return Some(widest);
        }
    }
    return None;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_bounds_tests.rs"]
mod tests;
