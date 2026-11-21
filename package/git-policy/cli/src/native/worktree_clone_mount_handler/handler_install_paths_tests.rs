//! What:
//!  Controls of the two installed paths and the handler's own name.
//! Why:
//!  The design fixes both paths and the name the ZED event must carry,
//!  and a moved character in either is an install that ZED never starts,
//!  so the strings are held here rather than trusted to a reader
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(SERVICE_INSTALL_PATH).toBe('/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets');
//! ```

use super::{CONFIGURATION_PATH, PROGRAM_NAME, SERVICE_INSTALL_PATH, ZEDLET_INSTALL_PATH};

/// The service path is the one the design names,
/// and its file name is the program's own name.
#[test]
fn the_service_path_is_the_designed_one() {
    assert_eq!(SERVICE_INSTALL_PATH, "/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets");
    assert_eq!(SERVICE_INSTALL_PATH.rsplit('/').next(), Some(PROGRAM_NAME));
}

/// The ZED entry is the history event the design names,
/// and ZED only runs a file whose name starts with `history_event-`.
#[test]
fn the_zedlet_path_is_the_designed_history_event() {
    assert_eq!(ZEDLET_INSTALL_PATH, "/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets");
    let name: &str = ZEDLET_INSTALL_PATH.rsplit('/').next().unwrap_or_default();
    assert!(name.starts_with("history_event-"), "ZED runs only files named history_event-*: {name}");
    assert!(name.contains(PROGRAM_NAME.replace('-', "_").as_str()), "the event names the program it starts: {name}");
}

/// The configuration stays in the root-controlled folder the design names.
#[test]
fn the_configuration_path_is_the_designed_one() {
    assert_eq!(CONFIGURATION_PATH, "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc");
}
