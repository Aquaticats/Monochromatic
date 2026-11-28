//! What:
//!  The checkout's own mount,
//!  its propagation,
//!  the caller's visibility and every dataset mounted under the checkout
//!  path,
//!  checked against the kernel's mount table.
//! Why:
//!  A mount-table entry alone can describe a mount no path reaches:
//!  the guest arm of VM item 9 attached a nested dataset to the home underlay
//!  and a later checkout mount hid it,
//!  leaving its row and an unreachable tree
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "One-time move and checkout boot protection").
//!  Propagation belongs to a mount instance and not to a dataset,
//!  so it is rechecked on the live row rather than trusted from whoever set it
//!  (`mount_namespaces(7)`,
//!  "Shared subtrees";
//!  the project's "Checkout propagation policy" answer).
//!  The table is read by the repository's own parser,
//!  so a control hands in the text of a table it wrote and this machine's
//!  mounts are never read by a control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const row = theCheckoutRow(facts); // the one zfs row at the checkout path
//! const outcome = checkCheckoutPropagation(row); // pass | finding('propagation') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;
use std::collections::BTreeMap;
use std::path::{
    Path,
    PathBuf,
};

/// The root a mount of the dataset's own root shows,
/// which a bind of a subtree shows deeper.
pub const THE_DATASETS_OWN_ROOT: &str = "/";

/// The optional field the kernel writes for a mount shared with another
/// namespace.
pub const SHARED_FIELD_PREFIX: &str = "shared:";

/// The optional field the kernel writes for a mount that has a master.
pub const MASTER_FIELD_PREFIX: &str = "master:";

/// What:
///  Everything the mount checks read:
///  the checkout path,
///  the dataset that should be mounted there,
///  and the caller's own mount table.
/// Why:
///  The three facts decide which row is the checkout's,
///  and the whole table decides whether a nested row is reachable,
///  so one value carries both and a report cannot mix two tables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountCheckFacts = { checkoutPath: string; checkoutDataset: string; rows: MountFacts[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountCheckFacts {
    /// Where the checkout is mounted.
    pub checkout_path: PathBuf,
    /// The dataset the design puts the checkout on.
    pub checkout_dataset: String,
    /// The rows of the caller's own mount table,
    /// in the order the kernel lists them.
    pub rows: Vec<MountFacts>,
}

/// What:
///  One dataset mounted under the checkout path,
///  which is no managed clone's slot.
/// Why:
///  The design inventories every dataset mounted under the checkout path
///  regardless of ZFS ancestry,
///  because the one it measured was nested under `nosnap` in the ZFS tree and
///  inside the checkout only by path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NestedMount = { dataset: string; mountPoint: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedMount {
    /// The dataset the row names as its source.
    pub dataset: String,
    /// Where the row stands.
    pub mount_point: PathBuf,
}

/// The row that is the checkout's own mount,
/// or nothing when the table holds none or several.
fn the_checkout_row(facts: &MountCheckFacts) -> Result<&MountFacts, Vec<&MountFacts>> {
    let mut held: Vec<&MountFacts> = Vec::new();
    for row in &facts.rows {
        if row.filesystem_type == ZFS_FILE_SYSTEM_TYPE
            && row.source == facts.checkout_dataset
            && row.mount_point == facts.checkout_path
        {
            held.push(row);
        }
    }
    if held.len() == 1 {
        return Ok(held[0]);
    }
    return Err(held);
}

/// What:
///  Whether the checkout's dataset is mounted at the checkout path in the
///  caller's own table,
///  and shows the dataset's own root.
/// Why:
///  The route attaches nothing unless the caller can see the checkout,
///  and a bind mount of a subtree under the checkout's name is a different
///  tree than the dataset the route reads,
///  so the row's root decides as well as its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCheckoutMount(facts); // pass | finding('checkout-mount'|'root') | unknown
/// ```
pub fn check_checkout_mount(facts: &MountCheckFacts) -> CheckOutcome {
    let expected: String = facts.checkout_path.display().to_string();
    return match the_checkout_row(facts) {
        Ok(row) => {
            if row.root != Path::new(THE_DATASETS_OWN_ROOT) {
                return finding(
                    "root",
                    format!(
                        "the row at {expected} shows {} of `{}` rather than the dataset's own root, so it is a bind of a subtree",
                        row.root.display(),
                        facts.checkout_dataset
                    ),
                );
            }
            pass(format!(
                "`{}` is mounted at {expected} (mount id {}) and this caller's table holds it",
                facts.checkout_dataset, row.mount_id
            ))
        }
        Err(held) if held.is_empty() => finding(
            "checkout-mount",
            format!(
                "no zfs row in this caller's table mounts `{}` at {expected}",
                facts.checkout_dataset
            ),
        ),
        Err(held) => {
            let ids: Vec<String> = held
                .iter()
                .map(|row| return row.mount_id.to_string())
                .collect();
            finding(
                "checkout-mount",
                format!(
                    "{} rows mount `{}` at {expected} (mount ids {}) and the route needs exactly one",
                    held.len(),
                    facts.checkout_dataset,
                    ids.join(", ")
                ),
            )
        }
    };
}

/// What:
///  The optional fields that say the mount is shared with another namespace.
/// Why:
///  The kernel prints `shared:N` for a mount a peer receives,
///  and `master:N` for a slave,
///  so either one means the mount is not private;
///  a private mount carries neither and needs no field to say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sharing = row.optionalFields.filter((field) => field.startsWith('shared:') || field.startsWith('master:'));
/// ```
fn sharing_fields(row: &MountFacts) -> Vec<&str> {
    let mut found: Vec<&str> = Vec::new();
    for field in &row.optional_fields {
        if field.starts_with(SHARED_FIELD_PREFIX) || field.starts_with(MASTER_FIELD_PREFIX) {
            found.push(field.as_str());
        }
    }
    return found;
}

/// What:
///  Whether the checkout's mount is private,
///  which the design's boot unit and the staging route both require.
/// Why:
///  A shared checkout delivers a mount made on it to another namespace,
///  which is what the guest measured as the shared arm's preserved delivery
///  and its cost,
///  and propagation belongs to the mount instance,
///  so the live row decides.
///  The finding prints the one command that changes it and never runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCheckoutPropagation(facts); // pass | finding('propagation') | unknown
/// ```
pub fn check_checkout_propagation(facts: &MountCheckFacts) -> CheckOutcome {
    let Ok(row) = the_checkout_row(facts) else {
        return unknown(String::from(
            "the checkout's own row is not in this caller's table exactly once, so its propagation is unproved",
        ));
    };
    let sharing: Vec<&str> = sharing_fields(row);
    if sharing.is_empty() {
        return pass(format!(
            "the checkout's mount (id {}) is private",
            row.mount_id
        ));
    }
    let path: String = row.mount_point.display().to_string();
    return finding(
        "propagation",
        format!(
            "the checkout's mount is not private ({}), and the route needs a mount no other namespace receives; set it back with `mount --make-rprivate {path}` after the mount is made",
            sharing.join(", ")
        ),
    );
}

/// What:
///  Every zfs row mounted strictly under the checkout path.
/// Why:
///  The inventory is by mount path rather than by dataset ancestry,
///  so a descendant of another ZFS parent (the measured `vm-builder-output`)
///  is found as well as a child of the checkout dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nested = rows.filter((row) => row.filesystem === 'zfs' && row.mountPoint.startsWith(checkoutPath) && row.mountPoint !== checkoutPath);
/// ```
pub fn nested_mounts(facts: &MountCheckFacts) -> Vec<NestedMount> {
    let mut nested: Vec<NestedMount> = Vec::new();
    for row in &facts.rows {
        if row.filesystem_type != ZFS_FILE_SYSTEM_TYPE {
            continue;
        }
        if !row.mount_point.starts_with(facts.checkout_path.as_path())
            || row.mount_point == facts.checkout_path
        {
            continue;
        }
        nested.push(NestedMount {
            dataset: row.source.clone(),
            mount_point: row.mount_point.clone(),
        });
    }
    return nested;
}

/// What:
///  Why a nested row is not reachable by path,
///  or that the table cannot say.
/// Why:
///  A row whose ancestor chain reaches the checkout's mount is reachable
///  through the checkout unless another mount hides its own path,
///  and a chain that reaches the table's root without the checkout hangs from
///  the tree the checkout replaced.
///  A chain naming a parent the table does not hold proves neither,
///  so it is unknown rather than a finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = reachability(row, parentById, checkoutId);
/// ```
fn reachability(
    row: &MountFacts,
    parents: &BTreeMap<u64, u64>,
    checkout_id: u64,
) -> Reachability {
    let mut current: u64 = row.mount_id;
    let mut steps: usize = 0;
    while steps <= parents.len() {
        if current == checkout_id {
            return Reachability::Reachable;
        }
        let Some(parent) = parents.get(&current) else {
            // The chain left this namespace's tree, so no path in the table
            // reaches the checkout from the row.
            return Reachability::Orphaned;
        };
        current = *parent;
        steps += 1;
    }
    return Reachability::Incomplete {
        mount_id: current,
    };
}

/// What one nested row's ancestor chain proved.
enum Reachability {
    /// The chain reaches the checkout's own mount.
    Reachable,
    /// The chain reaches the table's root without the checkout's mount.
    Orphaned,
    /// The chain names a mount the table does not hold,
    /// or loops.
    Incomplete {
        /// The mount the walk stopped at.
        mount_id: u64,
    },
}

/// What:
///  Whether every dataset mounted under the checkout path is reachable
///  through the checkout's own mount,
///  and no two rows share a path there.
/// Why:
///  A mount-table entry alone can describe a hidden mount,
///  which is what the guest measured when a later checkout attach hid the
///  output dataset while its row survived,
///  so a person must be told the path and the dataset rather than a count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkNestedReachability(facts); // pass | finding('path'|'reachability') | unknown
/// ```
pub fn check_nested_reachability(facts: &MountCheckFacts) -> CheckOutcome {
    let Ok(checkout) = the_checkout_row(facts) else {
        return unknown(String::from(
            "the checkout's own row is not in this caller's table exactly once, so no nested mount can be judged",
        ));
    };
    let mut parents: BTreeMap<u64, u64> = BTreeMap::new();
    let mut paths: BTreeMap<&Path, Vec<u64>> = BTreeMap::new();
    for row in &facts.rows {
        parents.insert(row.mount_id, row.parent_id);
        if row.mount_point.starts_with(facts.checkout_path.as_path()) {
            paths
                .entry(row.mount_point.as_path())
                .or_default()
                .push(row.mount_id);
        }
    }
    for (path, ids) in &paths {
        if ids.len() > 1 {
            return finding(
                "path",
                format!(
                    "{} rows are mounted at {} (mount ids {}), so the earlier one is hidden by the later and no path reaches it",
                    ids.len(),
                    path.display(),
                    ids.iter()
                        .map(u64::to_string)
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            );
        }
    }
    let nested: Vec<&MountFacts> = facts
        .rows
        .iter()
        .filter(|row| {
            return row.filesystem_type == ZFS_FILE_SYSTEM_TYPE
                && row.mount_point.starts_with(facts.checkout_path.as_path())
                && row.mount_point != facts.checkout_path;
        })
        .collect();
    let mut incomplete: Vec<String> = Vec::new();
    for row in &nested {
        match reachability(row, &parents, checkout.mount_id) {
            Reachability::Reachable => {}
            Reachability::Orphaned => {
                return finding(
                    "reachability",
                    format!(
                        "`{}` at {} hangs from the tree the checkout's mount replaced, so no path from the checkout reaches it (mount id {})",
                        row.source,
                        row.mount_point.display(),
                        row.mount_id
                    ),
                );
            }
            Reachability::Incomplete {
                mount_id,
            } => incomplete.push(format!(
                "{} (mount id {mount_id})",
                row.mount_point.display()
            )),
        }
    }
    if !incomplete.is_empty() {
        return unknown(format!(
            "the ancestor chain of {} loops inside the table, so its reachability is unproved",
            incomplete.join(", ")
        ));
    }
    return pass(format!(
        "{} dataset(s) mounted under the checkout path are reachable through its mount",
        nested.len()
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_checks_tests.rs"]
mod tests;
