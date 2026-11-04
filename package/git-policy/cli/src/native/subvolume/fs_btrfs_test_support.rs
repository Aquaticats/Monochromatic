//! What:
//!  The fixture root that the controls of the btrfs primitives and of the subvolume migration need, and the loud
//!  report where there is none.
//! Why:
//!  The real btrfs ioctls need a btrfs filesystem under the fixture root. The container gate runs on an overlay, where
//!  they cannot run. A control that silently passed there would claim coverage it does not have, so a control that
//!  needs btrfs fails with the reason unless the run states that the absence is known (the variable below, which the
//!  gate image sets); with the variable it prints one line per control that did not run, naming the control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const root = await btrfsFixture(name); // throws unless the run acknowledged that btrfs is absent
//! ```

use crate::fs_kind::{FilesystemKind, filesystem_kind};
use crate::test_support::fixture;
use std::path::PathBuf;

/// The environment variable that acknowledges "this run has no btrfs fixture root". The container gate image sets it.
pub(crate) const ABSENT_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED";

/// What:
///  A fresh fixture directory on btrfs, or `None` after reporting that the control did not run.
///  `Option<PathBuf>` is the owned path or nothing.
/// Why:
///   `None` is only returned when the run acknowledged that btrfs is absent; otherwise the call panics with the
///  filesystem it found and the two ways forward (point `CLI_GIT_TEST_ROOT` at a btrfs directory, or acknowledge).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function btrfsFixture(name: string): string | undefined;
/// ```
pub(crate) fn btrfs_fixture(name: &str) -> Option<PathBuf> {
    let root: PathBuf = fixture(name);
    let kind: Result<FilesystemKind, _> = filesystem_kind(root.as_path());
    if kind == Ok(FilesystemKind::Btrfs) {
        return Some(root);
    }
    let acknowledged: bool = std::env::var_os(ABSENT_ACKNOWLEDGED_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
    assert!(
        acknowledged,
        "the control {name} needs a btrfs fixture root, but {} is on {kind:?}; set CLI_GIT_TEST_ROOT to a directory on \
         btrfs, or set {ABSENT_ACKNOWLEDGED_VARIABLE}=1 to state that this run has none",
        root.display()
    );
    eprintln!("BTRFS CONTROL NOT RUN: {name} (fixture root {} is on {kind:?}; the host task native:test:host covers it)", root.display());
    return None;
}

/// What:
///  The same, plus the information whether the mount allows an unprivileged subvolume removal.
/// Why:
///   Controls that destroy a subvolume with the ioctl have two correct outcomes, by the mount option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removalAllowed = (await readMountFacts(root))?.hasOption('user_subvol_rm_allowed');
/// ```
pub(crate) fn removal_allowed(root: &std::path::Path) -> bool {
    return crate::fs_mount_options::read_mount_facts(root)
        .ok()
        .flatten()
        .is_some_and(|facts| { return facts.has_option("user_subvol_rm_allowed"); });
}
