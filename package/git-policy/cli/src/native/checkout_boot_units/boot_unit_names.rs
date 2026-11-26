//! What:
//!  Every name and path the boot and shutdown units are installed under,
//!  the owner's two time bounds,
//!  and the mount unit name a checkout path escapes to.
//! Why:
//!   A setup run writes files under fixed names and an `ExecStart` line names a
//!  mode word,
//!  so a name typed in two places drifts silently:
//!  the provider drop-in asks gate `X` to reconcile,
//!  a unit carries `ExecReload` and a reader asks `doctor` about gate `Y`,
//!  and nothing fails until a boot does.
//!  One constant per name keeps the plan,
//! the texts and the runbook one source
//!  (`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
//!  "The units,
//! whole" and "The stop-failure drop-in the unit needs beside it").
//!  The mount unit name is derived the way the design derives it,
//!  by `systemd-escape --suffix=mount`,
//!  because a drop-in on a unit systemd learned from the mount table is read
//!  only under that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gate = '/etc/systemd/system/reconcile-git-worktree-clone-mounts.service';
//! const mount = systemdEscapeMountUnit('/home/user/Monochromatic'); // 'home-user-Monochromatic.mount'
//! ```

use std::path::{
    Path,
    PathBuf,
};

/// The folder every unit of this route is written into.
pub const SYSTEM_UNIT_DIRECTORY: &str = "/etc/systemd/system";

/// The gate's unit name,
/// which its drop-in folder and every consumer's requirement name.
pub const BOOT_GATE_UNIT_NAME: &str = "reconcile-git-worktree-clone-mounts.service";

/// The gate's own file,
/// under the folder every unit of this route is written into.
pub const BOOT_GATE_UNIT_PATH: &str =
    "/etc/systemd/system/reconcile-git-worktree-clone-mounts.service";

/// The unit that runs `zfs mount -a` on this route's machines,
/// and the unit the gate wants and orders after (V1 and V2).
pub const PROVIDER_UNIT_NAME: &str = "zfs-mount.service";

/// The mode word the gate's `ExecStart` runs.
pub const BOOT_RECONCILE_MODE_WORD: &str = "boot-reconcile";

/// The mode word the gate's `ExecStartPre` runs,
/// with the failure prefix.
pub const START_PROBE_MODE_WORD: &str = "start-probe";

/// The mode word the gate's `ExecStopPost` runs,
/// with the failure prefix.
pub const SHUTDOWN_UNMOUNT_MODE_WORD: &str = "shutdown-unmount";

/// The owner's start bound (V8b),
/// which plan step 22 replaces with a measured ceiling.
/// Value:
///  `5min`,
/// as `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`
///  records it.
pub const GATE_START_BOUND: &str = "5min";

/// The owner's stop bound (V8b),
/// which plan step 22 replaces with a measured ceiling.
/// Value:
///  `1min`,
/// from the same answer as the start bound.
pub const GATE_STOP_BOUND: &str = "1min";

/// The result word the stop-failure drop-in sets,
/// which ends an overrunning stop stage in SIGKILL rather than SIGABRT.
pub const STOP_FAILURE_MODE_WORD: &str = "terminate";

/// The stop-failure drop-in's file name.
/// Why:
///  systemd parses drop-ins in file-name order,
/// and Fedora's own
///  `service.d/10-timeout-abort.conf` sets the aborting mode,
/// so this name must
///  sort after it.
pub const STOP_FAILURE_DROP_IN_NAME: &str = "zz-checkout-stop-failure-mode.conf";

/// The provider drop-in's file name.
pub const PROVIDER_DROP_IN_NAME: &str = "reconcile-git-worktree-clone-mounts-after-checkout.conf";

/// The propagation service's name.
pub const PROPAGATION_SERVICE_NAME: &str = "checkout-propagation.service";

/// The propagation drop-in's file name,
/// on the unit that mounts the checkout path.
pub const PROPAGATION_MOUNT_DROP_IN_NAME: &str = "10-upholds-propagation.conf";

/// Where the propagation service asks the manager for the gate's reload.
pub const EXECUTABLE_SYSTEMCTL: &str = "/usr/bin/systemctl";

/// The prefix and suffix one nested dataset's unit name is built between.
pub const NESTED_UNIT_NAME_PREFIX: &str = "git-worktree-nested-";

/// The suffix of a nested dataset's unit name.
pub const NESTED_UNIT_NAME_SUFFIX: &str = "-mount.service";

/// The words every unit line spells before a mode word,
/// which the root-owned file's dispatch reads.
/// Value:
///  `monochromatic-git-policy-cli zfs-root`,
/// from the root-modes record's
///  choice 2.
/// Why a function:
///  the group word is that design's constant,
/// so a rename moves every unit text
///  with it.
pub fn root_words() -> String {
    return crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name();
}

/// The path a unit is installed at.
///
/// Why:
///  every unit of this route lives in the same folder,
/// so only its name is
///  spelled beside the text.
pub fn unit_path(name: &str) -> PathBuf {
    return Path::new(SYSTEM_UNIT_DIRECTORY).join(name);
}

/// The path a drop-in of one unit is installed at.
///
/// Why:
///  a drop-in is read only from `<unit>.d/<name>`,
/// so the folder and the name
///  travel together.
pub fn drop_in_path(
    unit: &str,
    name: &str,
) -> PathBuf {
    return Path::new(SYSTEM_UNIT_DIRECTORY)
        .join(format!("{unit}.d"))
        .join(name);
}

/// The propagation service's own path.
pub fn propagation_service_path() -> PathBuf {
    return unit_path(PROPAGATION_SERVICE_NAME);
}

/// One nested dataset's unit name.
///
/// Why:
///  the name carries the dataset's own short word and the suffix a person reads
///  as a mount unit,
/// and one dataset gets one unit.
pub fn nested_unit_name(slug: &str) -> String {
    return format!("{NESTED_UNIT_NAME_PREFIX}{slug}{NESTED_UNIT_NAME_SUFFIX}");
}

/// What:
///  The unit name a checkout path escapes to,
///  as `systemd-escape --suffix=mount --path <path>` prints it.
/// Why:
///  a drop-in on the mount unit is read only under the name systemd gives that
///  unit,
/// and this host's mount units are learned from the mount table,
///  so the spelled name is the only handle:
///  `home-user-Monochromatic.mount` for this route's checkout,
///  and `srv-zfs\x2dstep\x2d19\x2dfixture-home.mount` in a guest whose path
///  carries dashes.
///  Every byte outside `[A-Za-z0-9:_.]` is escaped as `\xNN`,
/// a `/` becomes
///  `-`,
/// leading and trailing slashes are dropped,
/// and the root path alone
///  escapes to `-`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = systemdEscapeMountUnit('/home/user/Mono chrome'); // 'home-user-Mono\\x20chrome.mount'
/// ```
pub fn mount_unit_name(path: &Path) -> String {
    let bytes: &[u8] = std::os::unix::ffi::OsStrExt::as_bytes(path.as_os_str());
    let mut start: usize = 0;
    let mut end: usize = bytes.len();
    while start < end && bytes[start] == b'/' {
        start += 1;
    }
    while end > start && bytes[end - 1] == b'/' {
        end -= 1;
    }
    let mut escaped: String = String::with_capacity(end - start);
    let mut wrote_component: bool = false;
    let mut index: usize = start;
    while index < end {
        while index < end && bytes[index] == b'/' {
            index += 1;
        }
        if index >= end {
            break;
        }
        if wrote_component {
            escaped.push('-');
        }
        while index < end && bytes[index] != b'/' {
            push_one_escaped(&mut escaped, bytes[index]);
            index += 1;
        }
        wrote_component = true;
    }
    if escaped.is_empty() {
        escaped.push('-');
    }
    return format!("{escaped}.mount");
}

/// Write one byte the way `systemd-escape` writes it:
/// itself when it is one of the characters a unit name keeps,
/// and `\xNN` otherwise.
///
/// Why:
///  one byte rule serves every path component,
/// so a second spelling of it cannot
///  disagree with the first.
fn push_one_escaped(
    escaped: &mut String,
    byte: u8,
) {
    match byte {
        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b':' | b'_' | b'.' => {
            escaped.push(char::from(byte))
        }
        other => escaped.push_str(format!("\\x{other:02x}").as_str()),
    }
}

/// What:
///  Whether a word is safe where a unit line's argument stands:
///  a mode word,
///  a unit name,
///  a dataset name or a path with no whitespace,
///  no quote and no backslash.
/// Why:
///  systemd splits a command line on whitespace and honours quotes and a
///  trailing backslash,
/// so a value carrying one would change what the line runs
///  rather than what it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isOneWord(dataset)) throw new TypeError(dataset);
/// ```
pub fn is_one_word(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let forbidden: [char; 6] = [' ', '\t', '\n', '"', '\'', '\\'];
    return !text
        .chars()
        .any(|letter| return forbidden.contains(&letter));
}

/// What:
///  Whether a word is a unit name a dependency line can carry:
///  one name with no whitespace,
///  which may hold the backslashes `systemd-escape` writes.
/// Why:
///  a dependency line is a space-separated list and not a command line,
///  so a backslash is part of the name there,
///  and the escaped key-loading unit's name holds one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isOneUnitName(unit)) throw new TypeError(unit);
/// ```
pub fn is_one_unit_name(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    return !text
        .chars()
        .any(|letter| return letter == ' ' || letter == '\t' || letter == '\n' || letter == '\r');
}

/// A path as text a unit line can carry,
/// or nothing when its bytes are not UTF-8 or it carries a forbidden character.
///
/// Why:
///  a unit file is text,
/// and a path the kernel holds can carry bytes text
///  cannot,
/// so the caller refuses such a path rather than writing a line that
///  names another one.
pub fn path_as_one_word(path: &Path) -> Option<&str> {
    let text: &str = path.to_str()?;
    if !is_one_word(text) {
        return None;
    }
    return Some(text);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_unit_names_tests.rs"]
mod tests;
