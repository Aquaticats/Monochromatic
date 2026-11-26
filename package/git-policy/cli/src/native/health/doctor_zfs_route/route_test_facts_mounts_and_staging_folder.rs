//! What:
//!  One mount table in the kernel's own grammar,
//!  with the checkout mounted at its path and the clone container unmounted,
//!  and the staging folder under the account's cache with no folder in it.
//! Why:
//!  The mount checks read the mount table's rows,
//!  and the staging folder's checks read the folder and the binds under it,
//!  so both facts are built here at the design's shape,
//!  and a control adds the one row or the one folder it judges.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rows = parseMountTable(healthyMountText()); const staging = { cache: rootDirectory('/home/user/.cache', 0o700), account: 1000, folders: [] };
//! ```

use super::mount_checks::MountCheckFacts;
use super::route_test_facts_names::{
    ACCOUNT,
    CHECKOUT,
    CHECKOUT_DATASET,
};
use super::route_test_facts_paths::root_directory;
use super::staging_folder_checks::StagingFolderFacts;
use crate::fs_mount_options::parse_mount_table;
use std::path::PathBuf;

/// One mountinfo row in the kernel's grammar.
pub(crate) fn mount_row(
    id: u32,
    parent: u32,
    root: &str,
    mount_point: &str,
    filesystem: &str,
    source: &str,
    optional: &str,
) -> String {
    let optional_fields: String = if optional.is_empty() {
        String::new()
    } else {
        format!("{optional} ")
    };
    return format!(
        "{id} {parent} 0:42 {root} {mount_point} rw,relatime {optional_fields}- {filesystem} {source} rw,xattr,posixacl"
    );
}

/// A mount table with the checkout mounted at its path and the container
/// unmounted.
pub(crate) fn healthy_mount_text() -> String {
    return [
        mount_row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root", ""),
        mount_row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET, ""),
        mount_row(41, 1, "/", "/run", "tmpfs", "tmpfs", ""),
        mount_row(
            42,
            1,
            "/",
            "/home/user/.cache",
            "zfs",
            "zpcachyos/nosnap/user-cache",
            "shared:7",
        ),
    ]
    .join("\n");
}

/// The mount facts of the healthy table.
pub(crate) fn healthy_mounts() -> MountCheckFacts {
    return MountCheckFacts {
        checkout_path: PathBuf::from(CHECKOUT),
        checkout_dataset: String::from(CHECKOUT_DATASET),
        rows: parse_mount_table(healthy_mount_text().as_bytes()),
    };
}

/// The cache with no staging folder under it.
pub(crate) fn healthy_staging() -> StagingFolderFacts {
    return StagingFolderFacts {
        cache: root_directory("/home/user/.cache", 0o700),
        account: ACCOUNT,
        folders: Vec::new(),
    };
}
