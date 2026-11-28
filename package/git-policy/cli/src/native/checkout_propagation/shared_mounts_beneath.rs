//! What:
//!  The mounts a run found beneath the checkout whose set carries `MS_SHARED`:
//!  the exact count,
//!  and the mount point of the first four,
//!  written in the kernel's own spelling.
//! Why:
//!  The project decided the repair changes the checkout's own mount and no
//!  mount beneath it,
//!  and reports a shared mount it finds beneath
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "How far the setting reaches").
//!  On this host the case is real rather than hypothetical:
//!  `zfs mount -a` mounts the vm-builder output dataset under the checkout
//!  before any handler run,
//!  and a mount made under a shared checkout comes up shared.
//!  The rows are selected by where a mount sits rather than by its path,
//!  because a mount hidden under the checkout keeps the number of the mount
//!  that was there before it.
//!  Each path is written the way `/proc/self/mountinfo` spells it,
//!  so a space,
//!  a tab,
//!  a newline or a backslash in a path cannot break the record's line
//!  (rule `SYB`;
//!  `fs/proc_namespace.c`'s mangle).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const beneath = sharedMountsBeneath(table, facts.mountIdOld); // { count, paths, capped }
//! ```

use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::checkout_propagation::propagation_words::SHARED_MOUNTS_NAMED;
use crate::subvolume::fs_mount_options::MountFacts;
use std::os::unix::ffi::OsStrExt;

/// The byte a path's escaped spelling starts with.
const ESCAPE: u8 = b'\\';

/// The digits one escape writes,
/// which is three.
const ESCAPE_DIGITS: usize = 3;

/// What:
///  What one run found beneath the checkout.
/// Why:
///  The count is exact and the paths are capped,
///  so a reader that needs the fifth mount reads the table,
///  and a run that refused before it read the table says so rather than
///  reporting a count of none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedMountsBeneath = { kind: 'unread' } | { kind: 'counted'; count: number; paths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedMountsBeneath {
    /// The run refused before it read the table,
    /// so nothing is known.
    Unread,
    /// The table was read:
    /// this many mounts beneath the checkout carry `MS_SHARED`,
    /// and the first `SHARED_MOUNTS_NAMED` are named.
    Counted {
        /// How many shared mounts stand beneath the checkout.
        count: usize,
        /// The mount points,
        /// in the table's order,
        /// in the kernel's spelling.
        paths: Vec<String>,
    },
}

impl SharedMountsBeneath {
    /// The word the record's `beneath` field carries:
    /// the count,
    /// or `unread`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = (beneath: SharedMountsBeneath): string => beneath.kind === 'unread' ? 'unread' : String(beneath.count);
    /// ```
    pub fn word(&self) -> String {
        return match self {
            SharedMountsBeneath::Unread => String::from("unread"),
            SharedMountsBeneath::Counted {
                count,
                paths: _paths,
            } => count.to_string(),
        };
    }

    /// The paths the record names,
    /// which are none at all when the table was never read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const paths = (beneath: SharedMountsBeneath): string[] => beneath.kind === 'counted' ? beneath.paths : [];
    /// ```
    pub fn paths(&self) -> &[String] {
        return match self {
            SharedMountsBeneath::Unread => &[],
            SharedMountsBeneath::Counted {
                count: _count,
                paths,
            } => paths.as_slice(),
        };
    }
}

/// What:
///  The mounts strictly below one mount point whose set carries `MS_SHARED`.
/// Why:
///  A mount's place beneath the checkout is decided by its own row's parent
///  chain,
///  so a mount another mount covers is still found,
///  and the walk answers in the table's own order,
///  which is the order the kernel lists mounts in and the order a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const beneath = sharedMountsBeneath(table, checkoutRowId); // { count, paths: first four }
/// ```
pub fn shared_mounts_beneath(
    table: &[MountFacts],
    checkout_mount_id: u64,
) -> SharedMountsBeneath {
    let mut count: usize = 0;
    let mut paths: Vec<String> = Vec::new();
    for row in table {
        if !sits_below(table, row, checkout_mount_id) {
            continue;
        }
        if !MountPropagation::of_a_table_row(row).carries_shared() {
            continue;
        }
        count += 1;
        if paths.len() < SHARED_MOUNTS_NAMED {
            paths.push(spell_a_path(row.mount_point.as_os_str().as_bytes()));
        }
    }
    return SharedMountsBeneath::Counted {
        count,
        paths,
    };
}

/// What:
///  Whether one row's parent chain reaches the checkout's own row.
/// Why:
///  The chain is the kernel's own answer about where a mount sits,
///  and a row whose parent is not in the table ends the walk rather than
///  being guessed at,
///  so a cut or hidden table cannot make a mount look like the checkout's
///  child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sitsBelow = (table, row, id) => { let parent = row.parentId; while (parent !== 0 && parent !== row.mountId) { if (parent === id) return true; const next = table.find(r => r.mountId === parent); if (!next) return false; parent = next.parentId; } return false; };
/// ```
fn sits_below(
    table: &[MountFacts],
    row: &MountFacts,
    checkout_mount_id: u64,
) -> bool {
    let mut parent: u64 = row.parent_id;
    // The chain is bounded by the table's own length, so a table whose rows name each other in a loop ends.
    for _step in 0..table.len() {
        if parent == checkout_mount_id {
            return true;
        }
        if parent == 0 || parent == row.mount_id {
            return false;
        }
        let mut next: Option<u64> = None;
        for candidate in table {
            if candidate.mount_id == parent {
                next = Some(candidate.parent_id);
                break;
            }
        }
        let Some(above) = next else {
            return false;
        };
        parent = above;
    }
    return false;
}

/// What:
///  One path in the spelling `/proc/self/mountinfo` uses:
///  a space,
///  a tab,
///  a newline and a backslash each written as a backslash and three octal
///  digits.
/// Why:
///  The record is one line per path,
///  so a path that holds a newline would otherwise add a line of its own,
///  and a byte that is not text becomes the replacement character,
///  which is the same limit every path this crate reads has
///  (`fs/proc_namespace.c`,
///  whose mangle escapes exactly those four bytes).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spelled = (bytes: Uint8Array): string => [...bytes].map(b => [0x20, 0x09, 0x0a, 0x5c].includes(b) ? `\\${b.toString(8).padStart(3, '0')}` : String.fromCharCode(b)).join('');
/// ```
fn spell_a_path(bytes: &[u8]) -> String {
    let mut spelled: Vec<u8> = Vec::with_capacity(bytes.len());
    for byte in bytes {
        if *byte == b' ' || *byte == b'\t' || *byte == b'\n' || *byte == ESCAPE {
            spelled.push(ESCAPE);
            for place in (0..ESCAPE_DIGITS).rev() {
                let digit: u8 = (byte >> (3 * place)) & 0b111;
                spelled.push(b'0' + digit);
            }
        } else {
            spelled.push(*byte);
        }
    }
    return String::from_utf8_lossy(spelled.as_slice()).into_owned();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "shared_mounts_beneath_tests.rs"]
mod tests;
