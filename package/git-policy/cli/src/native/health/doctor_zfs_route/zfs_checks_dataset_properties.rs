//! What:
//!  One dataset's properties against the state the route attaches it in:
//!  the checkout at its path,
//!  the container unmounted and unmountable automatically,
//!  and each clone attached under the container's properties.
//! Why:
//!  A clone inherits `mountpoint` from its own parent,
//!  so a container that inherits `legacy` from an ancestor hands each clone
//!  the wrong source,
//!  a clone that lost `mountpoint=legacy` could be mounted by the system's own
//!  generator,
//!  and one that gained `setuid` or `devices` widens what its mount exposes,
//!  so every failure names the dataset and the one property to fix.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const failure = propertyFailure(role, facts); // { field, detail } | undefined
//! const outcome = checkDataset(facts); // pass | finding(field) | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// The facts and the role one dataset is judged by.
use super::zfs_checks_dataset_roles::{
    DatasetFacts,
    DatasetRole,
};
/// The values the design fixes for a dataset of this route.
use super::zfs_checks_limits::{
    CANMOUNT_OFF,
    CANMOUNT_PROPERTY,
    FILESYSTEM_OBJECT_SET,
    FLAG_OFF,
    LEGACY_MOUNTPOINT,
};
/// What a read of one dataset answered.
use super::zfs_checks_reading::Reading;
/// The kernel's facts for one dataset,
/// and the property values the checks compare against.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    MOUNTPOINT_PROPERTY,
    PropertyValue,
    value_of,
};
/// Where a property's value comes from,
/// which tells a container that set `legacy` on itself from one that inherits
/// it.
use crate::worktree_clone_mount_protocol::local_user_property::{
    PropertyReading,
    read_local_property,
};
/// What:
///  One dataset's properties against the state the route attaches it in,
///  as a finding or nothing.
/// Why:
///  The checkout, the container and each clone hold the same facts in a
///  different shape,
///  so one comparison answers all three and a new property is one arm rather
///  than three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = property_failure(role, facts); // { field, detail } | undefined
/// ```
fn property_failure(
    role: &DatasetRole,
    facts: &CloneObjectFacts,
) -> Option<(String, String)> {
    if facts.is_snapshot || facts.object_type != FILESYSTEM_OBJECT_SET {
        return Some((
            String::from("object-type"),
            format!(
                "`{}` is not a filesystem (type {}; snapshot: {}), and this route acts on filesystems only",
                facts.name, facts.object_type, facts.is_snapshot
            ),
        ));
    }
    match role {
        DatasetRole::Checkout {
            mount_point,
        } => {
            let expected: String = mount_point.display().to_string();
            let found: String = facts.mountpoint.describe("mountpoint");
            if facts.mountpoint.text() != Some(expected.as_str()) {
                return Some((
                    String::from("mountpoint"),
                    format!(
                        "`{}`: {found}, and the checkout is mounted at {expected}",
                        facts.name
                    ),
                ));
            }
            return None;
        }
        DatasetRole::Container => return container_failure(facts),
        DatasetRole::Clone => return clone_failure(facts),
    }
}

/// What:
///  The container's properties:
///  `legacy` set on the container itself,
///  unmountable automatically,
///  and no set-user-ID or device nodes in a mount of it.
/// Why:
///  A clone inherits `mountpoint` from its own parent,
///  and OpenZFS names the nearest dataset that set the value locally as the source,
///  so a container that inherits `legacy` from an ancestor hands each clone
///  `inherited from <the ancestor>` rather than `inherited from <the container>`,
///  which the read after the clone request refuses
///  (`worktree_clone_add_route::add_zfs::the_clones_mountpoint_failure`,
///  `doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
///  The route's two checks are therefore the same judgement of the same container,
///  and this one stands before any write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = readLocalProperty(facts.properties, facts.name, 'mountpoint');
/// if (source.kind !== 'local') return { field: 'mountpoint', detail: `... must set 'legacy' itself` };
/// ```
fn container_failure(facts: &CloneObjectFacts) -> Option<(String, String)> {
    if facts.mountpoint.text() != Some(LEGACY_MOUNTPOINT) {
        return Some((
            String::from("mountpoint"),
            format!(
                "`{}`: {}, and the clone container must be `{LEGACY_MOUNTPOINT}`",
                facts.name,
                facts.mountpoint.describe("mountpoint")
            ),
        ));
    }
    let source: PropertyReading<'_> = read_local_property(
        Some(&facts.properties),
        facts.name.as_str(),
        MOUNTPOINT_PROPERTY,
    );
    match &source {
        PropertyReading::Local {
            ..
        } => {}
        PropertyReading::Inherited {
            from,
        } => {
            return Some((
                String::from("mountpoint"),
                format!(
                    "`{}`: {}, inherited from `{from}`, and the clone container must set `{LEGACY_MOUNTPOINT}` on itself, because each clone inherits the value from its own parent and not from an ancestor above it",
                    facts.name,
                    facts.mountpoint.describe(MOUNTPOINT_PROPERTY)
                ),
            ));
        }
        PropertyReading::Absent => {
            return Some((
                String::from("mountpoint"),
                format!(
                    "`{}`: the reply carries no source for `{MOUNTPOINT_PROPERTY}`, so nothing proves the container set `{LEGACY_MOUNTPOINT}` on itself",
                    facts.name
                ),
            ));
        }
        PropertyReading::Malformed(refusal) => {
            return Some((
                String::from("mountpoint"),
                refusal.message(MOUNTPOINT_PROPERTY),
            ));
        }
    }
    let canmount: PropertyValue = value_of(&facts.properties, CANMOUNT_PROPERTY);
    if canmount.number() != Some(CANMOUNT_OFF) {
        return Some((
            String::from(CANMOUNT_PROPERTY),
            format!(
                "`{}`: {}, and the container must be off so no clone mounts automatically",
                facts.name,
                canmount.describe(CANMOUNT_PROPERTY)
            ),
        ));
    }
    return flag_failure(facts);
}

/// A clone's properties: legacy inherited from the container, and no
/// set-user-ID or device nodes in its mount.
pub(super) fn clone_failure(facts: &CloneObjectFacts) -> Option<(String, String)> {
    if facts.mountpoint.text() != Some(LEGACY_MOUNTPOINT) {
        return Some((
            String::from("mountpoint"),
            format!(
                "`{}`: {}, and a clone must inherit `{LEGACY_MOUNTPOINT}`",
                facts.name,
                facts.mountpoint.describe("mountpoint")
            ),
        ));
    }
    return flag_failure(facts);
}

/// The two flags a mount of any dataset in this route must be without.
fn flag_failure(facts: &CloneObjectFacts) -> Option<(String, String)> {
    for (property, found) in [("setuid", &facts.setuid), ("devices", &facts.devices)] {
        if found.number() != Some(FLAG_OFF) {
            return Some((
                String::from(property),
                format!(
                    "`{}`: {}, and `{property}` must be off for a mount of this dataset",
                    facts.name,
                    found.describe(property)
                ),
            ));
        }
    }
    return None;
}

/// What:
///  One dataset against the properties its role requires.
/// Why:
///  The checkout check names the checkout path,
///  the container check names the container,
///  and a failure names the property,
///  so a person reading one line learns both the dataset and the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDataset(facts); // pass | finding(field) | unknown
/// ```
pub fn check_dataset(facts: &DatasetFacts) -> CheckOutcome {
    return match &facts.reading {
        Reading::Unreadable {
            why,
        } => unknown(format!(
            "`{}` could not be read ({why}), so its properties are unproved",
            facts.name
        )),
        Reading::Missing {
            why,
        } => finding(
            "dataset",
            format!("`{}` is not in its pool ({why})", facts.name),
        ),
        Reading::Read(read) => {
            let failure: Option<(String, String)> = property_failure(&facts.role, read);
            return match failure {
                Some((field, detail)) => finding(field, detail),
                None => pass(format!(
                    "`{}` carries the properties its role requires",
                    facts.name
                )),
            };
        }
    };
}
