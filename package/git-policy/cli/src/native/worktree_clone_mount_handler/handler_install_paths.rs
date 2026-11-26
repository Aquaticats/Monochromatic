//! What:
//!  The paths one root-owned build serves:
//!  the root-owned regular file every root mode and every unit starts,
//!  the ZED history event ZED starts the same file by,
//!  the configuration beside them,
//!  and the checkout record the route reads.
//! Why:
//!  ZED runs a history event as an executable in `/etc/zfs/zed.d`,
//!  and after the owner's answer the release holds one program only
//!  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`),
//!  so the ZED entry is a link to the one root-owned file and no file here is
//!  named for the wrapper or for the modes
//!  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
//!  choice 3).
//!  Code,
//!  executable search paths and the configuration stay root-controlled,
//!  and this module is the one place a path is written,
//!  so a moved path is one edit and the install step of the plan
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 19)
//!  reads them rather than repeating them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rootModePath = '/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets';
//! const zedletPath = '/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets';
//! ```

use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;
use crate::worktree_clone_mount_handler::root_mode_names::ROOT_GROUP_WORD;

/// What:
///  The root-owned regular file every root mode is started as,
///  and the file ZED's entry links to.
/// Why:
///   One file under the ZED event's own name means a stray, mistyped or stale
///  start at this path is a refusal or a mode and never Git as root,
///  which is what the layout is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootModePath = '/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets';
/// ```
pub const ROOT_MODE_INSTALL_PATH: &str =
    "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets";

/// The ZED history event file:
/// the link ZED looks for the executable one event starts.
pub const ZEDLET_INSTALL_PATH: &str =
    "/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets";

/// The handler's configuration,
/// which stays root-controlled beside the root modes' install folder.
pub const CONFIGURATION_PATH: &str =
    "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// What:
///  The one record that says which dataset is the served checkout,
///  beside the slot identities.
/// Why:
///   Every reader of the checkout refuses a record that disagrees with the
///  kernel,
///  and `mount-path-descendant` places a mount only under a checkout this
///  record names,
///  so the path is written once and cited by name everywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkoutRecordPath = '/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json';
/// ```
pub const CHECKOUT_RECORD_PATH: &str =
    "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json";

/// What:
///  The words every root-mode line and every root-mode refusal names:
///  the program's own word and the group word choice 2 fixes.
/// Why:
///   The guards already pin both constants,
///  so a refusal cannot name words the dispatch does not accept,
///  and a rename moves the version line, the usage and every refusal
///  together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootModesName = () => `${COMMAND_WORD.current} zfs-root`;
/// ```
pub fn root_modes_name() -> String {
    return format!("{} {}", COMMAND_WORD.current, ROOT_GROUP_WORD);
}

/// What:
///  The line `--version` prints:
///  the words and the release this build came from.
/// Why:
///   An install, the release smoke and a person check the installed bytes with
///  one command that carries no word a mode would read,
///  and the line names the release so a lockfile's row can be compared with
///  it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const versionLine = () => `${rootModesName()} ${packageVersion}`;
/// ```
pub fn version_line() -> String {
    return format!("{} {}", root_modes_name(), env!("CARGO_PKG_VERSION"));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_install_paths_tests.rs"]
mod tests;
