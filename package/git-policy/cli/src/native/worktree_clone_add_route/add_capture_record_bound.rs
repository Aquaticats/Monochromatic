//! What:
//!  The capture's fixed words and the widths its bound rests on:
//!  the header line,
//!  the keys the fixed part writes,
//!  and the widest form of each value.
//! Why:
//!  The bound is the index's own entry count times one line's widest form plus
//!  the fixed lines,
//!  and no separate total is shipped,
//!  exactly as the design
//!  says
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "What 'the captured source' is").
//!  `add_capture_record.rs` writes and reads the record and would go over the
//!  file-size limit carrying these,
//!  and `add_capture_record_fields.rs` refuses one field at a time from the
//!  widths named here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bound = captureBound(entries); // the fixed lines plus every widest line
//! ```

use super::add_capture::MAXIMUM_PATH_BYTES;

/// The first line of a capture,
/// naming the grammar's version.
/// Version 2 carries the C3 and C4 digests a dirty source needs;
/// a version 1
/// record holds none,
/// so it is refused rather than read as one that does.
pub const CAPTURE_HEADER: &str =
    "# monochromatic-git-policy-cli zfs worktree add capture, version 2";

/// The keys the fixed part writes,
/// in the order it writes them.
pub const CAPTURE_FIXED_KEYS: &[&str] = &[
    "nonce",
    "repository",
    "source",
    "checkoutGuid",
    "indexSize",
    "indexMtime",
    "indexDigest",
    "head",
    "sourceHead",
    "entries",
    "digests",
];

/// The widest decimal form of a 64-bit unsigned number.
const WIDEST_DECIMAL: usize = 20;

/// The widest nanosecond count,
/// nine digits.
pub(super) const WIDEST_NANOS: usize = 10;

/// The widest commit name,
/// 64 hexadecimal digits for a SHA-256 repository.
pub(super) const WIDEST_COMMIT: usize = 64;

/// One captured entry's widest line:
/// hex path,
/// six fields,
/// two times,
/// the executable bit and a link's hex target.
pub const CAPTURE_ENTRY_LINE_MAXIMUM_BYTES: u64 = (2 * MAXIMUM_PATH_BYTES) as u64
    + 1
    + 7
    + 1
    + WIDEST_DECIMAL as u64
    + 1
    + (WIDEST_DECIMAL + 1 + WIDEST_NANOS) as u64 * 2
    + 1
    + 1
    + 1
    + (2 * MAXIMUM_PATH_BYTES) as u64
    + 1;

/// One digest's widest line:
/// the hex path,
/// the SHA-256 and their space.
pub const CAPTURE_DIGEST_LINE_MAXIMUM_BYTES: u64 = (2 * MAXIMUM_PATH_BYTES) as u64 + 1 + 64 + 1;

/// The capture's fixed lines' widest total.
pub const CAPTURE_FIXED_BYTES: u64 = CAPTURE_HEADER.len() as u64
    + 1
    + ("nonce=".len() + 32 + 1) as u64
    + ("repository=".len() + 2 * MAXIMUM_PATH_BYTES + 1) as u64
    + ("source=".len() + 2 * MAXIMUM_PATH_BYTES + 1) as u64
    + ("checkoutGuid=".len() + WIDEST_DECIMAL + 1) as u64
    + ("indexSize=".len() + WIDEST_DECIMAL + 1) as u64
    + ("indexMtime=".len() + WIDEST_DECIMAL + 1 + WIDEST_NANOS + 1) as u64
    + ("indexDigest=".len() + 64 + 1) as u64
    + ("head=".len() + WIDEST_COMMIT + 1) as u64
    + ("sourceHead=".len() + WIDEST_COMMIT + 1) as u64
    + ("entries=".len() + WIDEST_DECIMAL + 1) as u64
    + ("digests=".len() + WIDEST_DECIMAL + 1) as u64;

/// The largest capture one entry count and its digests can name.
pub fn capture_bound(entries: usize) -> u64 {
    return CAPTURE_FIXED_BYTES
        + entries as u64 * CAPTURE_ENTRY_LINE_MAXIMUM_BYTES
        + entries as u64 * CAPTURE_DIGEST_LINE_MAXIMUM_BYTES;
}
