//! What:
//!  What an explicit mount of one dataset is refused by,
//!  and what attributes it is asked for:
//!  the properties and values upstream's `share_mount_one` reads one by one,
//!  the refusal each of them answers,
//!  and the `fsmount` attributes the dataset's own properties resolve to.
//! Why:
//!  `mount_path_descendant_mode.rs` gathers the proofs and places the mount and
//!  would go over the file-size limit carrying this vocabulary too,
//!  and these decisions are the only ones the mode makes out of a dataset's own
//!  properties,
//!  so a reader can hold them beside upstream's rules rather than among the
//!  proofs about the record and the mount table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const attributes = attributesOf(facts, reader, 'zpcachyos'); // MOUNT_ATTR_NOATIME
//! ```

use super::refusals::MountPathDescendantFailure;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{CloneObjectFacts, PropertyValue, value_of};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

/// The properties whose values decide whether an explicit mount succeeds,
/// which upstream's `share_mount_one` refuses one by one.
pub const CANMOUNT_PROPERTY: &str = "canmount";
/// The property that says the dataset belongs to a non-global zone.
pub const ZONED_PROPERTY: &str = "zoned";
/// The property holding how much of a received stream is redacted.
pub const REDACTED_PROPERTY: &str = "redacted";
/// The property that says a receive is unfinished.
pub const INCONSISTENT_PROPERTY: &str = "inconsistent";
/// The property holding a partially received stream's resume token.
pub const RECEIVE_RESUME_TOKEN_PROPERTY: &str = "receive_resume_token";
/// The property naming the folder a dataset mounts at.
pub const MOUNTPOINT_PROPERTY: &str = "mountpoint";
/// The property that allows one mount above another.
pub const OVERLAY_PROPERTY: &str = "overlay";
/// The property naming the cipher a dataset's data is encrypted with.
pub const ENCRYPTION_PROPERTY: &str = "encryption";
/// The property that says whether an encrypted dataset's key is loaded.
pub const KEY_STATUS_PROPERTY: &str = "keystatus";
/// The property that says a mount of the dataset carries no access-time writes.
pub const ATIME_PROPERTY: &str = "atime";
/// The property that says access times are updated relative to the modification.
pub const RELATIME_PROPERTY: &str = "relatime";
/// The property that says a mount of the dataset carries device nodes.
pub const DEVICES_PROPERTY: &str = "devices";
/// The property that says a mount of the dataset may run its files.
pub const EXEC_PROPERTY: &str = "exec";
/// The property that says a mount of the dataset is read-only.
pub const READONLY_PROPERTY: &str = "readonly";
/// The property that says a mount of the dataset carries set-user-ID execution.
pub const SETUID_PROPERTY: &str = "setuid";
/// The property that has no Linux mount attribute and is read as the no-op it
/// is.
pub const NBMAND_PROPERTY: &str = "nbmand";
/// The value `canmount` holds when nothing may mount the dataset itself.
pub const CANMOUNT_OFF: &str = "off";
/// The value a property holds when it is switched on.
pub const SWITCHED_ON: &str = "on";
/// The value `keystatus` holds when the key is loaded.
pub const KEY_STATUS_AVAILABLE: &str = "available";
/// The value a property holds when it is switched off.
pub const SWITCHED_OFF: &str = "off";
/// The value `mountpoint` holds when nothing may mount the dataset by name.
pub const MOUNTPOINT_LEGACY: &str = "legacy";
/// The value `mountpoint` holds when nobody mounts the dataset.
pub const MOUNTPOINT_NONE: &str = "none";
/// The value `mountpoint` holds when the dataset has no mountpoint at all.
pub const MOUNTPOINT_DASH: &str = "-";

/// What:
///  Refuse an explicit mount of a dataset upstream refuses,
///  naming the property and its value.
/// Why:
///  An explicit `zfs mount` fails loudly for each of these and returns 1,
///  and a unit that reported success while mounting nothing is the failure this
///  refusal replaces;
///  the receive-token case also names the resume a person can run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseUnmountable('pool/dataset', facts); // throws dataset-not-mountable
/// ```
///
/// # Errors
/// `DatasetNotMountable` for each property that refuses a mount.
pub(super) fn refuse_unmountable(dataset: &str, facts: &CloneObjectFacts) -> Result<(), MountPathDescendantFailure> {
    if value_of(&facts.properties, CANMOUNT_PROPERTY).text() == Some(CANMOUNT_OFF) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(CANMOUNT_PROPERTY),
            value: String::from(CANMOUNT_OFF),
            because: String::from("an explicit mount of a `canmount=off` dataset refuses rather than mounting it"),
        });
    }
    if value_of(&facts.properties, ZONED_PROPERTY).text() == Some(SWITCHED_ON) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(ZONED_PROPERTY),
            value: String::from(SWITCHED_ON),
            because: String::from("a dataset exported to a local zone is mounted by that zone and an explicit mount in the global zone refuses"),
        });
    }
    let redacted: PropertyValue = value_of(&facts.properties, REDACTED_PROPERTY);
    if redacted.number().is_some_and(|count| return count != 0) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(REDACTED_PROPERTY),
            value: redacted.describe(REDACTED_PROPERTY),
            because: String::from("a redacted dataset is not mounted by an explicit mount"),
        });
    }
    let inconsistent: bool = value_of(&facts.properties, INCONSISTENT_PROPERTY).number() == Some(1);
    let token: PropertyValue = value_of(&facts.properties, RECEIVE_RESUME_TOKEN_PROPERTY);
    if inconsistent && token.text().is_some_and(|text| return !text.is_empty()) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(INCONSISTENT_PROPERTY),
            value: String::from("1"),
            because: String::from("a partially received dataset is refused by an explicit mount, and its receive can be resumed with `zfs send -t <the token>`"),
        });
    }
    let encryption: PropertyValue = value_of(&facts.properties, ENCRYPTION_PROPERTY);
    let keystatus: PropertyValue = value_of(&facts.properties, KEY_STATUS_PROPERTY);
    if encryption.text().is_some_and(|cipher| return cipher != SWITCHED_OFF) && keystatus.text() != Some(KEY_STATUS_AVAILABLE) {
        return Err(MountPathDescendantFailure::EncryptionKeyNotLoaded {
            dataset: String::from(dataset),
            keystatus: keystatus.text().map_or_else(|| return keystatus.describe(KEY_STATUS_PROPERTY), String::from),
        });
    }
    return Ok(());
}

/// What:
///  The `fsmount` attributes the dataset's own properties resolve to,
///  in upstream's order.
/// Why:
///  Upstream derives one option from each pair and skips the `relatime` pair
///  when `atime` is off,
///  and a pool imported read-only forces every mount read-only;
///  `nbmand` has no Linux attribute and is read as the no-op it is,
///  which the mode states rather than passing an option nothing reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributes = attributesOf(facts, reader, 'zpcachyos'); // MOUNT_ATTR_NOATIME
/// ```
///
/// # Errors
/// The pool's own `readonly` could not be read.
pub(super) fn attributes_of(facts: &CloneObjectFacts, datasets: &dyn DatasetReader, pool: &str) -> Result<u64, MountPathDescendantFailure> {
    let mut attributes: u64 = 0;
    let atime_on: bool = value_of(&facts.properties, ATIME_PROPERTY).text() == Some(SWITCHED_ON);
    let relatime_on: bool = value_of(&facts.properties, RELATIME_PROPERTY).text() != Some(SWITCHED_OFF);
    if !atime_on {
        attributes |= libc::MOUNT_ATTR_NOATIME;
    } else if !relatime_on {
        attributes |= libc::MOUNT_ATTR_STRICTATIME;
    }
    if value_of(&facts.properties, DEVICES_PROPERTY).text() == Some(SWITCHED_OFF) {
        attributes |= libc::MOUNT_ATTR_NODEV;
    }
    if value_of(&facts.properties, EXEC_PROPERTY).text() == Some(SWITCHED_OFF) {
        attributes |= libc::MOUNT_ATTR_NOEXEC;
    }
    if value_of(&facts.properties, SETUID_PROPERTY).text() == Some(SWITCHED_OFF) {
        attributes |= libc::MOUNT_ATTR_NOSUID;
    }
    let identity: DatasetIdentity = datasets.identity(pool).map_err(|failure| return MountPathDescendantFailure::DatasetRead { dataset: String::from(pool), failure })?;
    let pool_readonly: bool = value_of(&identity.properties, READONLY_PROPERTY).text() == Some(SWITCHED_ON);
    if pool_readonly || value_of(&facts.properties, READONLY_PROPERTY).text() == Some(SWITCHED_ON) {
        attributes |= libc::MOUNT_ATTR_RDONLY;
    }
    return Ok(attributes);
}
