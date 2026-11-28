//! What:
//!  The checks the step makes before it touches a mount:
//!  the point is the checkout path,
//!  the type is `zfs`,
//!  the source is the resolved checkout dataset,
//!  the instance has a namespace,
//!  and the mount table's row for it describes the same mount.
//! Why:
//!  The step changes one mount instance and never a path,
//!  so every fact that says which instance it read is proved before the set,
//!  and a disagreement is a refusal rather than a guess
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "What it verifies before",
//!  steps 3 to 5,
//!  and "What each check owes the threat model",
//!  which keeps these on a trusted machine because a boot whose provider
//!  mounted the wrong thing produces exactly them).
//!  The table's row is found by the reused number the kernel prints and must
//!  describe the same mount,
//!  so a replacement between the descriptor read and the table read sets
//!  nothing on either instance.
//!  The source is compared where the kernel reports one,
//!  and from the table's row otherwise:
//!  `sb_source` arrived in Linux 6.13 and the table has carried the same name
//!  since before it,
//!  so a kernel without the field still proves which dataset the mount serves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = checkTheFacts(facts, checkout, dataset); // undefined when they agree
//! const row = checkTheRow(facts, table, dataset); // throws 'before-table' or 'source'
//! ```

use crate::checkout_propagation::mount_descriptor_read::DescriptorMount;
use crate::subvolume::fs_mount_options::MountFacts;
use std::path::Path;

/// The filesystem type a checkout's mount must carry,
/// which is the dataset's own.
const ZFS_TYPE: &str = "zfs";

/// What:
///  Why a descriptor's mount is not the checkout's own instance.
/// Why:
///  Each cause is the design's own step word,
///  so the record's `step` field and the sentence a person reads name the
///  same check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountRefusal = { word: 'point' | 'type' | 'source' | 'namespace' | 'before-table'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountRefusal {
    /// The step word the record carries.
    pub word: &'static str,
    /// What disagreed,
    /// in the words a person reads.
    pub why: String,
}

/// What:
///  The checks the descriptor's own answer settles,
///  before the table is read.
/// Why:
///  A fact the answer carries needs no table,
///  and a run that refuses here has read nothing else,
///  so the record says the step that refused and leaves the fields it never
///  reached as `none`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = checkTheFacts(facts, checkout, dataset); // undefined when they agree
/// ```
pub fn check_the_facts(
    facts: &DescriptorMount,
    checkout: &Path,
    dataset: &str,
) -> Option<MountRefusal> {
    if facts.mount_point != checkout {
        return Some(MountRefusal {
            word: "point",
            why: format!(
                "the descriptor's mount is at {} and the checkout is {}",
                facts.mount_point.display(),
                checkout.display()
            ),
        });
    }
    if facts.filesystem_type != ZFS_TYPE {
        return Some(MountRefusal {
            word: "type",
            why: format!(
                "the descriptor's mount is a `{}` mount and the checkout is a ZFS dataset's",
                facts.filesystem_type
            ),
        });
    }
    if let Some(source) = facts.source.as_deref() {
        // The kernel reports a source from Linux 6.13 on, and the table's row answers for it on an older one.
        if source != dataset {
            return Some(MountRefusal {
                word: "source",
                why: format!(
                    "the descriptor's mount takes its bytes from `{source}` and the checkout dataset is `{dataset}`"
                ),
            });
        }
    }
    if facts.namespace_id.is_none() {
        return Some(MountRefusal {
            word: "namespace",
            why: String::from(
                "the kernel reports no mount namespace for the descriptor's mount, so it may be an instance a lazy unmount detached",
            ),
        });
    }
    return None;
}

/// What:
///  The table's row for the instance the descriptor read,
///  or the refusal that no row describes it.
/// Why:
///  The reused number finds the row,
///  and the row must then agree with the descriptor about the point,
///  the type and the source,
///  because the kernel hands the reused number out again and a row that
///  disagrees is a replacement between the two reads
///  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
///  "What it verifies before",
///  step 5).
///  The source check falls here on a kernel that reports none,
///  so one dataset name decides the two paths to the same fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = checkTheRow(facts, table, dataset); // throws 'before-table' or 'source'
/// ```
pub fn check_the_row<'table>(
    facts: &DescriptorMount,
    table: &'table [MountFacts],
    dataset: &str,
) -> Result<&'table MountFacts, MountRefusal> {
    let mut found: Option<&MountFacts> = None;
    for row in table {
        if row.mount_id == facts.mount_id_old {
            found = Some(row);
            break;
        }
    }
    let Some(row) = found else {
        return Err(MountRefusal {
            word: "before-table",
            why: format!(
                "this process's mount table holds no row with the number {} the kernel prints for the descriptor's mount",
                facts.mount_id_old
            ),
        });
    };
    if row.mount_point != facts.mount_point || row.filesystem_type != facts.filesystem_type {
        return Err(MountRefusal {
            word: "before-table",
            why: format!(
                "the mount table's row with the number {} names {} on a `{}` mount and the descriptor's mount is {} on a `{}` mount, so the mount was replaced between the two reads",
                facts.mount_id_old,
                row.mount_point.display(),
                row.filesystem_type,
                facts.mount_point.display(),
                facts.filesystem_type
            ),
        });
    }
    if facts.source.is_some() {
        // The descriptor's answer names a source, so a row naming another one describes another mount.
        if row.source != facts.source.clone().unwrap_or_default() {
            return Err(MountRefusal {
                word: "before-table",
                why: format!(
                    "the mount table's row takes its bytes from `{}` and the descriptor's mount from `{}`, so the mount was replaced between the two reads",
                    row.source,
                    facts.source.clone().unwrap_or_default()
                ),
            });
        }
        return Ok(row);
    }
    // A kernel that reports no source leaves the table's row to answer which dataset the mount serves.
    if row.source != dataset {
        return Err(MountRefusal {
            word: "source",
            why: format!(
                "the mount table's row takes its bytes from `{}` and the checkout dataset is `{dataset}`",
                row.source
            ),
        });
    }
    return Ok(row);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "checkout_mount_checks_tests.rs"]
mod tests;
