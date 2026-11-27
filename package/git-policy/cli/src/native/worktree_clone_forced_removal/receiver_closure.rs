//! What:
//!  The mounts an unmount reaches:
//!  the seed the design takes from the parent mount,
//!  the closure over shared groups and masters,
//!  the members that have a mount at the slot,
//!  the covered ones,
//!  and the namespaces outside the act.
//! Why:
//!  A plain unmount reaches the mounts that receive propagation from the
//!  unmounted mount's parent and never a set chosen by the unmounted mount
//!  itself,
//!  propagation runs to peers and slaves and never up to a master,
//!  and the walk is a closure,
//!  because slaves of slaves are receivers too
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F47,
//!  F55,
//!  and "The preflight").
//!  The preflight's answer decides the act's route rather than refusing:
//!  a kept receiver outside the namespaces the act handles means way 8 is not
//!  used and the act falls to way 7,
//!  which is what T0-13 measured in the pure-slave and peer shapes,
//!  where the host is a receiver and its own copy goes with the holder's.
//!  The design also fixes where way 8 is taken at all:
//!  "exactly where the marker exists and the parent shares with a group of its
//!  own",
//!  so a parent that is private or a pure slave takes way 7.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planTheReceivers(acting, tables, parentMountId, theSlot, handled);
//! plan.wayEight; plan.receivers;
//! ```

/// What:
///  One row of a mount table,
///  with the fields the propagation tags are read from.
/// Why:
///  The closure is computed from the kernel's own tags and mount numbers,
///  and group numbers are global to the machine,
///  so rows of different namespaces compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountRow = { mountId; parentId; mountPoint; root; filesystem; source; shared: number[]; master?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountRow {
    /// The mount's own number,
    /// which names one mount across namespaces.
    pub mount_id: u32,
    /// The number of the mount it sits on.
    pub parent_id: u32,
    /// Where it is mounted in its own namespace.
    pub mount_point: String,
    /// The root of the mounted tree at that mount point.
    pub root: String,
    /// Its file-system type,
    /// which must be `zfs` for the clone's own mount.
    pub filesystem: String,
    /// Its source,
    /// which for a mount of the clone is the dataset's name.
    pub source: String,
    /// Every peer group the mount is in.
    pub shared: Vec<u32>,
    /// The group whose slave this mount is,
    /// when it is one.
    pub master: Option<u32>,
}

/// What:
///  One namespace's table and what its slot reads.
/// Why:
///  The closure is over groups,
///  which are global,
///  but whether a member keeps a receiver is a question about that member's own
///  namespace:
/// only a member with a mount at the slot's dentry receives the
///  unmount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceTable = { namespace; rows: MountRow[]; atTheSlot?: MountRow };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceTable {
    /// The namespace's kernel identity,
    /// such as `mnt:[4026532001]`.
    pub namespace: String,
    /// Its rows.
    pub rows: Vec<MountRow>,
    /// The topmost mount at the slot's dentry in this namespace,
    /// read through the namespace itself.
    pub at_the_slot: Option<MountRow>,
}

/// What:
///  One namespace the unmount reaches and what it holds at the slot.
/// Why:
///  The report names the namespace and the mount it loses,
///  and the covered case is the one where the namespace's own mount sits over
///  its copy and stays,
///  which the design asks the act to proceed on and report (V9).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Receiver = { namespace; mountId; atTheSlot: boolean; covered: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receiver {
    /// The namespace the member is in.
    pub namespace: String,
    /// The member mount's number.
    pub mount_id: u32,
    /// Whether the namespace has a mount at the slot's dentry.
    pub at_the_slot: bool,
    /// Whether that mount is another mount over the copy,
    /// so the copy goes and the cover stays.
    pub covered: bool,
}

/// What:
///  What the preflight decided about the routes and the receivers.
/// Why:
///  Way 8 is taken only where the parent shares with a group of its own and no
///  kept receiver lies outside the namespaces the act handles,
///  and the names outside it are what the report names when way 7 stops the
///  parent propagating (V8).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReceiverSet = { seed: number[]; receivers: Receiver[]; outside: string[]; wayEight: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverSet {
    /// The groups the closure started from.
    pub seed: Vec<u32>,
    /// Every namespace the unmount reaches,
    /// in the order the tables were given.
    pub receivers: Vec<Receiver>,
    /// The kept receivers that are not namespaces the act handles.
    pub outside: Vec<String>,
    /// Whether way 8 may be used.
    pub way_eight: bool,
}

/// What:
///  The parent mount of the copy in the acting namespace,
///  which decides where the unmount goes.
/// Why:
///  A plain unmount's receivers come from the parent's group and not from the
///  copy's,
///  so the copy's own tags decide nothing about the reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = acting.rows.find(row => row.mountId === copy.parentId);
/// ```
pub fn parent_of<'a>(
    acting: &'a NamespaceTable,
    copy: &MountRow,
) -> Option<&'a MountRow> {
    return acting
        .rows
        .iter()
        .find(|row| return row.mount_id == copy.parent_id);
}

/// What:
///  Plan the routes and the receivers for one holder namespace's copy.
/// Why:
///  The seed is the parent's own `shared:` group,
///  and a parent that has none is the case the design sends to way 7 without
///  seeding anything:
///  "the marker route is taken unchanged exactly where the marker exists and the
///  parent shares with a group of its own",
///  because a seed derived from a `master:` twice removed over-read the set in
///  the arm that tried it (T0-5a-literal).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seed = parent.shared; if (!seed.length) return { wayEight: false, ... };
/// let set = new Set(seed); do { for (row of everyRow) if (row.shared.some(inSet) || set.has(row.master)) { member; row.shared.forEach(add); } } while (changed);
/// ```
pub fn plan_the_receivers(
    acting: &NamespaceTable,
    copy: &MountRow,
    tables: &[NamespaceTable],
    dataset: &str,
    handled: &[String],
) -> ReceiverSet {
    let Some(parent) = parent_of(acting, copy) else {
        return ReceiverSet {
            seed: Vec::new(),
            receivers: Vec::new(),
            outside: Vec::new(),
            way_eight: false,
        };
    };
    let seed: Vec<u32> = parent.shared.clone();
    if seed.is_empty() {
        // A parent that shares with no group of its own takes way 7, and its receivers are not seeded from a
        // `master:` the design found over-read.
        return ReceiverSet {
            seed,
            receivers: Vec::new(),
            outside: Vec::new(),
            way_eight: false,
        };
    }
    let mut groups: Vec<u32> = seed.clone();
    let mut members: Vec<(String, u32)> = Vec::new();
    let mut grew: bool = true;
    while grew {
        grew = false;
        for table in tables {
            for row in &table.rows {
                let joined: bool = row.shared.iter().any(|group| return groups.contains(group))
                    || row
                        .master
                        .is_some_and(|master| return groups.contains(&master));
                if !joined {
                    continue;
                }
                if !members.iter().any(|(namespace, mount)| {
                    return *namespace == table.namespace && *mount == row.mount_id;
                }) {
                    members.push((table.namespace.clone(), row.mount_id));
                }
                for group in &row.shared {
                    if !groups.contains(group) {
                        groups.push(*group);
                        grew = true;
                    }
                }
            }
        }
    }
    let mut receivers: Vec<Receiver> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    for table in tables {
        if !members
            .iter()
            .any(|(namespace, _mount)| return *namespace == table.namespace)
        {
            continue;
        }
        let Some(slot) = table.at_the_slot.as_ref() else {
            // A member with no mount at the slot keeps nothing and receives nothing.
            continue;
        };
        let covered: bool = slot.filesystem != "zfs" || slot.source != dataset;
        // The report names the mount an unmount at that dentry meets, which is the namespace's own topmost mount
        // there, and not the first member row the closure happened to find.
        let mount_id: u32 = slot.mount_id;
        receivers.push(Receiver {
            namespace: table.namespace.clone(),
            mount_id,
            at_the_slot: true,
            covered,
        });
        if !handled.contains(&table.namespace) {
            outside.push(table.namespace.clone());
        }
    }
    let way_eight: bool = outside.is_empty();
    return ReceiverSet {
        seed,
        receivers,
        outside,
        way_eight,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "receiver_closure_tests.rs"]
mod tests;
