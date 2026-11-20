//! What:
//!  The pool's block-cloning switch as the kernel reports it,
//!  and what the clone rung does when it reads off:
//!  nothing,
//!  so the add goes to Git's checkout instead of paying the source proof for a clone that would be refused.
//! Why:
//!  A ZFS pool with `zfs_bclone_enabled` at 0 refuses every `FICLONE`,
//!  so the rung's source proof (68 to 109 ms on this repository's tree) and its first created file
//!  would buy nothing and the checkout would follow anyway;
//!  the switch is read once per add,
//! before the proof,
//!  so such an add is declined on the cheap question rather than after the expensive one.
//!  A switch that cannot be read is not a refusal:
//!  a host without the ZFS module has no such file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const off = readSwitch('/sys/module/zfs/parameters/zfs_bclone_enabled') === false;
//! if (kindOf(target) === 'zfs' && off) decline('its pool has block cloning off');
//! ```

use crate::fs_kind::{FilesystemKind, filesystem_kind};
use std::path::Path;

/// What:
///  The kernel file that says whether ZFS block cloning is on:
///  `1` when it is and `0` when it is not.
/// Why:
///  It is the pool's switch for every dataset of the host,
///  so one read answers for the destinations this add could clone into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ZFS_BLOCK_CLONING_SWITCH = '/sys/module/zfs/parameters/zfs_bclone_enabled';
/// ```
pub(crate) const ZFS_BLOCK_CLONING_SWITCH: &str = "/sys/module/zfs/parameters/zfs_bclone_enabled";

/// What:
///  What the switch file at `path` says:
///  `Some(true)` for block cloning on,
///  `Some(false)` for off,
///  and nothing when the file is absent,
/// unreadable or holds neither value.
/// Why:
///  Nothing means "try the rung":
///  a host that cannot answer is not a host that refuses clones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function switchAt(path: string): boolean | undefined;
/// ```
pub(crate) fn switch_at(path: &Path) -> Option<bool> {
    let text: String = match std::fs::read_to_string(path) {
        Ok(found) => found,
        Err(_unreadable) => return None,
    };
    return match text.trim() {
        "1" => Some(true),
        "0" => Some(false),
        _other => None,
    };
}

/// What:
///  Why a destination whose pool has block cloning off cannot take the clones,
///  or nothing when the switch is on or unknown,
/// or the destination is not on ZFS.
/// Why:
///  Only a ZFS dataset is asked:
///  the switch belongs to the pool,
///  so a btrfs or overlay destination on a host that also runs ZFS is decided by its own filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = (kind, on, target) => kind === 'zfs' && on === false ? `${target} would receive the clones, and its pool has block cloning off` : undefined;
/// ```
pub(crate) fn block_cloning_refusal((kind, switch): (FilesystemKind, Option<bool>), target: &Path) -> Option<String> {
    if kind != FilesystemKind::Zfs || switch != Some(false) {
        return None;
    }
    return Some(format!("{} would receive the clones, and its pool has ZFS block cloning off (`{}` reads 0), so every clone into it would be refused", target.display(), ZFS_BLOCK_CLONING_SWITCH));
}

/// What:
///  Why the clone rung cannot serve an add into `target` on this host,
///  read from the destination's own filesystem and the switch file at `switch`.
/// Why:
///  The ladder asks this once per add,
/// before the source proof,
///  and a control asks it of a fixture file that stands in for the kernel's switch,
///  so the decision is proven on a host whose own pool has block cloning on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusalAt = (target, switch) => refusal(kindOf(target), readSwitch(switch), target);
/// ```
pub(crate) fn block_cloning_refusal_at((target, switch): (&Path, &Path)) -> Option<String> {
    let kind: FilesystemKind = filesystem_kind(target).unwrap_or(FilesystemKind::Other);
    return block_cloning_refusal((kind, switch_at(switch)), target);
}

/// Block-cloning controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_block_cloning_tests.rs"]
mod tests;
