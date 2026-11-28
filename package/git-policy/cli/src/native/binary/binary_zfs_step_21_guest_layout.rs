//! What:
//!  One throwaway guest's layout for plan step 21's arms:
//!  the guest variables the driver sets,
//!  the design's checkout dataset with the delegation the route's preflight
//!  reads,
//!  the clone container beside it,
//!  and the readings that tell an arm which mount and which dataset it faces.
//! Why:
//!  Every arm here prepares dependencies in a worktree the ZFS clone route made,
//!  and that route serves an add only on the step 19 design's layout,
//!  so the layout is the design's datasets rather than a folder on the guest's
//!  root
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19's layout).
//!  The checkout dataset is made here rather than migrated,
//!  because the owner's migration has not run,
//!  so every fact these arms prove is the guest's semantics on a file-backed
//!  pool and never this host's storage.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { checkout, container } = makeTheLayout(sbin, pool, root);
//! ```

#![cfg(target_os = "linux")]

use super::zfs_sweep_guest::{
    zfs,
    zfs_ignored,
    zfs_ok,
};
use std::path::Path;
use std::process::{
    Command,
    Output,
};
use std::time::Duration;

/// The variable the guest driver sets,
/// so a host run skips instead of touching a pool.
pub(super) const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_GUEST";

/// The pool the driver made.
pub(super) const POOL_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_POOL";

/// The OpenZFS userspace the arm's own reads use.
pub(super) const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_SBIN";

/// The checkout root the arm mounts the design's dataset at.
pub(super) const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_ROOT";

/// The checkout dataset's leaf name the step 19 design fixes.
pub(super) const CHECKOUT_LEAF: &str = "monochromatic-git-checkout";

/// The container's leaf name the design fixes beside it.
pub(super) const CONTAINER_LEAF: &str = "monochromatic-git-worktree-clone-datasets";

/// The wrapper program the driver built.
pub(super) const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// The real Git the fixture commits with,
/// never the wrapper's link.
pub(super) const REAL_GIT: &str = "/usr/bin/git";

/// How often a poller looks at a dataset,
/// a record or a mount.
pub(super) const POLL: Duration = Duration::from_millis(20);

/// How long one add may take before the arm gives up on it.
pub(super) const ADD_WAIT: Duration = Duration::from_secs(180);

/// How many interleaved rounds each measured arm runs.
/// Why:
///  An odd count gives each arm a median,
///  and alternating which arm goes first keeps neither warmed by the other at
///  the end.
pub(super) const ROUNDS: usize = 3;

/// The desired-state property the add writes its mount request under.
pub(super) const DESIRED_STATE: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The account this arm runs as,
/// whose delegation the route's preflight reads.
const ACCOUNT: &str = "0";

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
pub(super) fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP binary_zfs_step_21: {GUEST_VARIABLE} is unset, so these arms run only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "these arms run only in the guest {name}");
    return true;
}

/// The design's dataset layout for the guest,
/// with the delegation the route's preflight reads:
/// the checkout dataset mounted at the configured path and private,
/// and the clone container beside it as a legacy dataset.
pub(super) fn make_the_layout(
    sbin: &str,
    pool: &str,
    checkout: &Path,
) -> (String, String) {
    let base: String = format!("{pool}/step21");
    let checkout_dataset: String = format!("{base}/{CHECKOUT_LEAF}");
    let container: String = format!("{base}/{CONTAINER_LEAF}");
    zfs_ignored(sbin, &["destroy", "-r", "-f", base.as_str()]);
    // The parent is made without a mountpoint, because `zfs create -p -o` applies its options to every parent it makes and a
    // second mount at the same path would hide the checkout.
    zfs_ok(
        sbin,
        &["create", "-p", "-o", "mountpoint=none", base.as_str()],
    );
    let mountpoint: String = format!("mountpoint={}", checkout.display());
    zfs_ok(
        sbin,
        &[
            "create",
            "-o",
            mountpoint.as_str(),
            checkout_dataset.as_str(),
        ],
    );
    std::fs::create_dir_all(checkout).expect("the checkout's mount point");
    // The design requires a not-shared immediate parent, which its boot unit sets; this arm sets it for the checkout alone.
    let _private: Output = Command::new("/usr/bin/mount")
        .args(["--make-private", checkout.to_str().expect("a text path")])
        .output()
        .expect("the private mount starts");
    zfs_ok(
        sbin,
        &[
            "create",
            "-o",
            "mountpoint=legacy",
            "-o",
            "canmount=off",
            "-o",
            "setuid=off",
            "-o",
            "devices=off",
            container.as_str(),
        ],
    );
    // The permissions the design scopes, handed to the account this arm runs as, which is root: the route's preflight reads
    // the caller's own delegation and refuses without them.
    for arguments in [
        vec![
            "allow",
            "-l",
            "-u",
            ACCOUNT,
            "snapshot",
            checkout_dataset.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            ACCOUNT,
            "clone,mount,destroy",
            checkout_dataset.as_str(),
        ],
        vec![
            "allow",
            "-l",
            "-u",
            ACCOUNT,
            "create,mount",
            container.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            ACCOUNT,
            "mount,destroy,userprop",
            container.as_str(),
        ],
    ] {
        zfs_ok(sbin, arguments.as_slice());
    }
    return (checkout_dataset, container);
}

/// Whether a dataset answers `zfs list`.
pub(super) fn dataset_stands(
    sbin: &str,
    dataset: &str,
) -> bool {
    return zfs(sbin, "zfs", &["list", "-H", "-o", "name", dataset], None).0;
}

/// Whether one clone's durable intent asks for its mount,
/// so the arm mounts only once the route's own state says a mount belongs there.
pub(super) fn the_intent_says_mount(
    sbin: &str,
    clone: &str,
) -> bool {
    let (_ok, said, _error): (bool, String, String) = zfs(
        sbin,
        "zfs",
        &["get", "-H", "-o", "value", DESIRED_STATE, clone],
        None,
    );
    return said.trim().ends_with(":mount");
}

/// Whether a mount stands at one folder of this process's table.
pub(super) fn mounted_at(folder: &Path) -> bool {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo")
        .expect("the arm reads this process's mount table");
    let wanted: String = folder.display().to_string();
    return table
        .lines()
        .any(|line| return line.split(' ').nth(4) == Some(wanted.as_str()));
}

/// The device a folder lives on,
/// which is the number that decides whether a rename can exchange two files.
pub(super) fn device_of(folder: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt as _;
    return std::fs::metadata(folder)
        .expect("the folder's own metadata")
        .dev();
}
