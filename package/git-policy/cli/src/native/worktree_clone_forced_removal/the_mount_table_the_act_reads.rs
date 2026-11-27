//! What:
//!  The kernel's mount table as the act reads it inside one holder's namespace:
//!  the parse into the closure's own rows,
//!  the row one mount id names,
//!  the topmost mount at one path,
//!  and the read of `/proc/self/mountinfo` that feeds them.
//! Why:
//!  Every stage re-reads this table before and after what it changes,
//!  so the read and the two lookups are one job and must judge the table one
//!  way:
//!  the preflight and the act both ask which mount sits on one path,
//!  and the answer is the kernel's own row rather than a name resolved after
//!  the proof
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Way 8,
//!  in place by the handle").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rows = rowsOf(readTheTable());      // inside the holder's namespace
//! const copy = rowOf(rows, copyMountId);    // the mount the act changed
//! const topmost = topmostAt(rows, slot);    // the mount an unmount there meets
//! ```

use crate::fs_mount_options::{
    MountFacts,
    parse_mount_table,
};
use crate::worktree_clone_forced_removal::receiver_closure::MountRow;
use std::path::Path;

/// Where the kernel lists the mounts of the namespace a child entered.
const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";

/// What:
///  One row of the kernel's table as the closure's own shape,
///  so the receivers are computed from what the act read.
/// Why:
///  The preflight and the act read one table and must judge it one way,
///  and the closure already takes rows with the tags the kernel prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = rowsOf(mountinfoText);
/// ```
pub fn rows_of(table: &str) -> Vec<MountRow> {
    let parsed: Vec<MountFacts> = parse_mount_table(table.as_bytes());
    let mut rows: Vec<MountRow> = Vec::with_capacity(parsed.len());
    for facts in parsed {
        let mut groups: Vec<u32> = Vec::new();
        let mut master: Option<u32> = None;
        for field in &facts.optional_fields {
            if let Some(number) = field
                .strip_prefix("shared:")
                .and_then(|text| return text.parse::<u32>().ok())
            {
                groups.push(number);
            }
            if let Some(number) = field
                .strip_prefix("master:")
                .and_then(|text| return text.parse::<u32>().ok())
            {
                master = Some(number);
            }
        }
        rows.push(MountRow {
            mount_id: u32::try_from(facts.mount_id).unwrap_or(u32::MAX),
            parent_id: u32::try_from(facts.parent_id).unwrap_or(u32::MAX),
            mount_point: facts.mount_point.to_string_lossy().into_owned(),
            root: facts.root.to_string_lossy().into_owned(),
            filesystem: facts.filesystem_type,
            source: facts.source,
            shared: groups,
            master,
        });
    }
    return rows;
}

/// What:
///  The row the kernel lists for one mount id.
/// Why:
///  Every stage's read-after compares the kernel's own number,
///  and a row found by number is the mount the act changed.
pub fn row_of(
    rows: &[MountRow],
    mount_id: u64,
) -> Option<MountRow> {
    return rows
        .iter()
        .find(|row| return u64::from(row.mount_id) == mount_id)
        .cloned();
}

/// What:
///  The topmost mount at one path,
///  which is the last row the kernel prints for it.
/// Why:
///  A mount stacked over another is printed after it,
///  and the unmount by the handle acts on the topmost one,
///  so the act reads the same row the kernel will.
pub fn topmost_at(
    rows: &[MountRow],
    at: &Path,
) -> Option<MountRow> {
    let wanted: String = at.to_string_lossy().into_owned();
    let mut found: Option<MountRow> = None;
    for row in rows {
        if row.mount_point == wanted {
            found = Some(row.clone());
        }
    }
    return found;
}

/// What:
///  Read `/proc/self/mountinfo` inside the namespace the child is in.
/// Why:
///  The table is the kernel's own and this child's, not the handler's,
///  which is what makes the read-after a fact about the namespace it changed.
pub(super) fn read_the_table() -> Result<String, std::io::Error> {
    return crate::plain_file::read_plain_text_at(
        Path::new(MOUNTINFO_PATH),
        crate::plain_file::MOUNT_TABLE_LIMIT,
    );
}
