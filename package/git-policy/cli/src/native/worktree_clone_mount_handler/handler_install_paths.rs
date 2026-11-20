//! What:
//!  The two paths the same build of this program is installed at:
//!  the root-owned executable a service call starts,
//!  and the ZED history event the same program answers as.
//! Why:
//!  ZED runs a history event as an executable in `/etc/zfs/zed.d`,
//!  so the handler's ZED entry is a second install of one build,
//!  never a second program
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!  Code,
//!  executable search paths and the handler's configuration stay root-controlled,
//!  and this module is the one place either path is written,
//!  so a moved path is one edit and the install step of the plan
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 19)
//!  reads them rather than repeating them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const servicePath = '/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets';
//! const zedletPath = '/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets';
//! ```

/// The program's name,
/// which both installed files carry as the name they were built with,
/// and the name the handler's messages use for itself.
pub const PROGRAM_NAME: &str = "mount-git-worktree-clone-datasets";

/// The root-owned executable a service call starts,
/// and the path every other installed file names when it starts the handler.
pub const SERVICE_INSTALL_PATH: &str =
    "/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets";

/// The ZED history event file:
/// the same build,
/// installed where ZED looks for the executable one event starts.
pub const ZEDLET_INSTALL_PATH: &str = "/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets";

/// The handler's configuration,
/// which stays root-controlled beside the executable's install folder.
pub const CONFIGURATION_PATH: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_install_paths_tests.rs"]
mod tests;
