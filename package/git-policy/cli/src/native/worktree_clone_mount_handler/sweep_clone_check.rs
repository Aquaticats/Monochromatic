//! What:
//!  The checks one immediate child of the pinned parent passes before the sweep
//!  attaches it:
//!  the slot its name gives,
//!  its filesystem type,
//!  its GUID,
//!  its origin,
//!  the inherited mountpoint and the two safety properties,
//!  and whether the key under which its data is encrypted is loaded.
//! Why:
//!  A malformed property,
//!  name or origin never authorizes work on another mount,
//!  and the sweep derives the slot from one name under the grammar
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The key check comes before the claim and the walk so an unavailable key is
//!  reported as itself rather than as an identity or permission failure:
//!  item 5 of the design's VM measurements saw a clone made with the pool's key
//!  unloaded,
//!  a detached mount refused with `EACCES` before attachment,
//!  and the attach succeed after a manual key load.
//!  Every refusal carries the word a receipt names the failed step with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verified = verify(facts, parent); // { slot, guid } or a named refusal
//! ```

use super::sweep_dataset_facts::{
    CloneObjectFacts, DEVICES_PROPERTY, DMU_OBJECT_SET_FILESYSTEM, ENCRYPTION_PROPERTY, ENCRYPTION_ROOT_PROPERTY, INDEX_OFF, KEY_STATUS_PROPERTY, KEY_STATUS_UNAVAILABLE, MOUNTPOINT_PROPERTY,
    PropertyValue, SETUID_PROPERTY,
};
use crate::start_probe::probe_source_name::check_dataset_name;
use crate::worktree_clone_mount::SLOT_NAME_MAXIMUM_BYTES;
use crate::worktree_clone_mount::slot_name_refusal;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuidRefusal;

/// The mountpoint value that keeps `zfs mount -a` and the systemd mount
/// generator away from a clone.
pub const LEGACY_MOUNTPOINT: &str = "legacy";

/// The property value that says a dataset's data is not encrypted.
pub const ENCRYPTION_OFF: u64 = 0;

/// The separator between a dataset and the snapshot of it a clone names as its
/// origin.
pub const SNAPSHOT_SEPARATOR: char = '@';

/// The prefix the design gives the snapshot the add route clones a managed
/// clone from.
pub const CLONE_ORIGIN_SNAPSHOT_PREFIX: &str = "git-worktree-clone-origin-";

/// What:
///  One verified clone:
///  the slot its name gives and the GUID a receipt is filed under.
/// Why:
///  The slot is what the walk and the store and the claim all key on,
///  and the GUID is the receipt's only file name,
///  so a caller that holds both cannot act on a name and file a result under
///  another dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VerifiedClone = { slot: string; dataset: DatasetGuid; datasetName: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedClone {
    /// The slot's name below `.claude/worktrees`.
    pub slot: String,
    /// The clone's canonical numeric GUID.
    pub dataset: DatasetGuid,
    /// The clone's full dataset name.
    pub dataset_name: String,
}

/// What:
///  Why one child of the pinned parent is not a clone this sweep attaches.
/// Why:
///   Each cause sends a person to a different property,
///  and the receipt's failed step is one of these words,
///  so the words are the refusal's own rather than a sentence a caller parses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneRefusal = { kind: 'not-a-child' | 'slot-name' | 'not-a-filesystem' | 'snapshot' | 'guid' | 'no-origin' | 'origin' | 'mountpoint' | 'setuid' | 'devices' | 'property' | 'key-unavailable'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneRefusal {
    /// The name is not one component below the pinned parent.
    NotAChild {
        /// The dataset's name.
        name: String,
        /// The pinned parent.
        parent: String,
    },
    /// The name below the parent is no name this route mounts.
    SlotName {
        /// The dataset's name.
        name: String,
        /// The component below the parent.
        slot: String,
        /// Why the component is not a slot name.
        reason: String,
    },
    /// The object-set type is not a filesystem.
    NotAFilesystem {
        /// The dataset's name.
        name: String,
        /// The type the kernel reported.
        found: u32,
    },
    /// The kernel called the object a snapshot.
    Snapshot {
        /// The dataset's name.
        name: String,
    },
    /// The GUID is zero,
    /// so no receipt could be filed under it.
    NoGuid {
        /// The dataset's name.
        name: String,
    },
    /// The GUID could not be written as a receipt's file name.
    Guid {
        /// The dataset's name.
        name: String,
        /// Why the GUID is not canonical.
        reason: DatasetGuidRefusal,
    },
    /// The dataset is not a clone of anything.
    NoOrigin {
        /// The dataset's name.
        name: String,
    },
    /// The origin is not a snapshot of the checkout dataset the design names.
    OriginOutsideTheCheckout {
        /// The dataset's name.
        name: String,
        /// The origin the kernel reported.
        origin: String,
        /// The checkout dataset whose snapshot the add route clones.
        checkout_dataset: String,
    },
    /// The origin is a snapshot of the checkout dataset,
    /// but not one the add route made.
    OriginIsNotACloneOrigin {
        /// The dataset's name.
        name: String,
        /// The origin the kernel reported.
        origin: String,
        /// The prefix the add route gives its snapshots.
        prefix: &'static str,
    },
    /// The mountpoint property could not be read.
    Property {
        /// The dataset's name.
        name: String,
        /// What the reply said instead of a value.
        said: String,
        /// The receipt's word for this step.
        step: &'static str,
    },
    /// The mountpoint is not `legacy`,
    /// so `zfs mount -a` or the systemd generator could mount the clone first.
    MountpointIsNotLegacy {
        /// The dataset's name.
        name: String,
        /// The value the kernel reported.
        found: String,
    },
    /// The setuid property is on,
    /// so a mount of the clone would carry set-user-ID execution.
    SetuidIsOn {
        /// The dataset's name.
        name: String,
        /// The value the kernel reported.
        found: String,
    },
    /// The devices property is on,
    /// so a mount of the clone would carry device nodes.
    DevicesIsOn {
        /// The dataset's name.
        name: String,
        /// The value the kernel reported.
        found: String,
    },
    /// The key the clone's data is encrypted under is not loaded.
    KeyUnavailable {
        /// The dataset's name.
        name: String,
        /// The dataset whose key is missing,
        /// as the kernel named it.
        encryption_root: String,
        /// What the reply said about the encryption,
        /// so a person can tell an encrypted clone from one whose reply is
        /// incomplete.
        encryption: String,
    },
}

impl CloneRefusal {
    /// The word a receipt and a sweep line name this refusal's step with.
    pub fn step(&self) -> &'static str {
        return match self {
            CloneRefusal::NotAChild { .. } | CloneRefusal::SlotName { .. } => "clone-name",
            CloneRefusal::NotAFilesystem { .. } | CloneRefusal::Snapshot { .. } => "clone-type",
            CloneRefusal::NoGuid { .. } | CloneRefusal::Guid { .. } => "clone-guid",
            CloneRefusal::NoOrigin { .. } | CloneRefusal::OriginOutsideTheCheckout { .. } | CloneRefusal::OriginIsNotACloneOrigin { .. } => "clone-origin",
            CloneRefusal::Property { step, .. } => step,
            CloneRefusal::MountpointIsNotLegacy { .. } => "mountpoint",
            CloneRefusal::SetuidIsOn { .. } => "setuid",
            CloneRefusal::DevicesIsOn { .. } => "devices",
            CloneRefusal::KeyUnavailable { .. } => "key-unavailable",
        };
    }

    /// The refusal in words,
    /// naming the dataset,
    /// the property and every way forward.
    pub fn message(&self) -> String {
        let cause: String = match self {
            CloneRefusal::NotAChild { name, parent } => format!(
                "the dataset `{name}` is not one immediate child of the pinned clone parent `{parent}`, so no slot is derived from its name. Nothing was mounted, unmounted or recorded on its behalf"
            ),
            CloneRefusal::SlotName { name, slot, reason } => format!(
                "the dataset `{name}` sits below the pinned parent as `{slot}`, which is no slot name ({reason}). Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::NotAFilesystem { name, found } => format!(
                "the dataset `{name}` reports object-set type {found}, and a clone this route attaches is a filesystem (type {DMU_OBJECT_SET_FILESYSTEM}). Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::Snapshot { name } => format!("`{name}` is a snapshot, not a clone dataset. Nothing was mounted or recorded on its behalf"),
            CloneRefusal::NoGuid { name } => format!(
                "the dataset `{name}` reports GUID 0, which cannot name a receipt file. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::Guid { name, reason } => format!(
                "the dataset `{name}` reports a GUID this route cannot file a receipt under: {}. Nothing was mounted or recorded on its behalf",
                reason.message()
            ),
            CloneRefusal::NoOrigin { name } => format!(
                "the dataset `{name}` is not a clone: the kernel reported no origin, so nothing ties it to a snapshot of the checkout dataset. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::OriginOutsideTheCheckout { name, origin, checkout_dataset } => format!(
                "the clone `{name}` was made from `{origin}`, which is not a snapshot of the checkout dataset `{checkout_dataset}`, so this sweep does not attach it. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::OriginIsNotACloneOrigin { name, origin, prefix } => format!(
                "the clone `{name}` was made from `{origin}`, whose snapshot name does not begin with `{prefix}`, and an origin the add route did not make is not one this sweep attaches. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::Property { name, said, .. } => format!(
                "the dataset `{name}` could not be judged: {said}. A property this route cannot read is not treated as absent, because an absent mountpoint and a legacy one are different instructions. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::MountpointIsNotLegacy { name, found } => format!(
                "the clone `{name}` reports `{MOUNTPOINT_PROPERTY}={found}`, and a clone this route attaches inherits `mountpoint={LEGACY_MOUNTPOINT}` so that neither `zfs mount -a` nor the systemd mount generator mounts it first. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::SetuidIsOn { name, found } => format!(
                "the clone `{name}` reports `{SETUID_PROPERTY}={found}`, and this route requires `{SETUID_PROPERTY}={INDEX_OFF}` on the datasets it attaches. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::DevicesIsOn { name, found } => format!(
                "the clone `{name}` reports `{DEVICES_PROPERTY}={found}`, and this route requires `{DEVICES_PROPERTY}={INDEX_OFF}` on the datasets it attaches. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::KeyUnavailable { name, encryption_root, encryption } => format!(
                "the key the clone `{name}` is encrypted under is not loaded: `{KEY_STATUS_PROPERTY}` is {KEY_STATUS_UNAVAILABLE} for the encryption root `{encryption_root}` ({encryption}). This is a load-key question and neither a changed folder identity nor a permission failure. Nothing was mounted or recorded on its behalf"
            ),
        };
        return format!(
            "{cause}. Ways forward: `zfs get -H -o value,source {MOUNTPOINT_PROPERTY},{SETUID_PROPERTY},{DEVICES_PROPERTY} {name}` shows the three properties, `zfs get -H -o value encryptionroot,keystatus <clone>` shows the encryption, and a key that is not loaded is loaded with `zfs load-key <encryptionroot>` before the next sweep or the boot unit's next start.",
            name = self.name()
        );
    }

    /// The dataset this refusal is about.
    pub fn name(&self) -> &str {
        return match self {
            CloneRefusal::NotAChild { name, .. }
            | CloneRefusal::SlotName { name, .. }
            | CloneRefusal::NotAFilesystem { name, .. }
            | CloneRefusal::Snapshot { name }
            | CloneRefusal::NoGuid { name }
            | CloneRefusal::Guid { name, .. }
            | CloneRefusal::NoOrigin { name }
            | CloneRefusal::OriginOutsideTheCheckout { name, .. }
            | CloneRefusal::OriginIsNotACloneOrigin { name, .. }
            | CloneRefusal::Property { name, .. }
            | CloneRefusal::MountpointIsNotLegacy { name, .. }
            | CloneRefusal::SetuidIsOn { name, .. }
            | CloneRefusal::DevicesIsOn { name, .. }
            | CloneRefusal::KeyUnavailable { name, .. } => name.as_str(),
        };
    }
}

/// What:
///  Why one `/`-separated component of a dataset name is no slot name.
/// Why:
///  Two rules apply and a person needs both:
///  the store's and the walk's own slot rule,
///  and this route's narrower one,
///  which accepts lowercase ASCII letters and digits with interior hyphens
///  because that is the grammar the design names for the slot it derives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = cloneSlotRefusal('Two Words'); // 'it holds a character...'
/// ```
fn clone_slot_refusal(slot: &str) -> Option<String> {
    if let Some(reason) = slot_name_refusal(slot) {
        return Some(reason);
    }
    let bytes: &[u8] = slot.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        let interior_hyphen: bool = *byte == b'-' && index > 0 && index + 1 < bytes.len();
        let letter_or_digit: bool = byte.is_ascii_lowercase() || byte.is_ascii_digit();
        if !(letter_or_digit || interior_hyphen) {
            return Some(format!(
                "it holds `{}`, and a slot this sweep derives holds lowercase ASCII letters, digits and interior hyphens",
                char::from(*byte)
            ));
        }
    }
    return None;
}

/// What:
///  The slot one child's name gives,
///  or the refusal of a name that gives none.
/// Why:
///  The slot is derived from one plain component below the pinned parent and
///  from nothing else,
///  so a name with a second component,
///  or one the grammar refuses,
///  never selects another folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = slot_of('pool/clones/one', 'pool/clones');
/// ```
///
/// # Errors
/// The name is not one child of the parent,
/// its component is no slot name,
/// or the name and the slot exceed the kernel's and the store's bounds.
pub fn slot_of(name: &str, parent: &str) -> Result<String, CloneRefusal> {
    let prefix: String = format!("{parent}/");
    let Some(slot) = name.strip_prefix(prefix.as_str()) else {
        return Err(CloneRefusal::NotAChild { name: String::from(name), parent: String::from(parent) });
    };
    if slot.contains('/') || slot.is_empty() {
        return Err(CloneRefusal::NotAChild { name: String::from(name), parent: String::from(parent) });
    }
    if let Some(reason) = clone_slot_refusal(slot) {
        return Err(CloneRefusal::SlotName { name: String::from(name), slot: String::from(slot), reason });
    }
    if slot.len() as u64 > SLOT_NAME_MAXIMUM_BYTES {
        return Err(CloneRefusal::SlotName {
            name: String::from(name),
            slot: String::from(slot),
            reason: format!("it is longer than the {SLOT_NAME_MAXIMUM_BYTES} bytes a slot record's name holds"),
        });
    }
    if let Err(why) = check_dataset_name(name) {
        return Err(CloneRefusal::SlotName { name: String::from(name), slot: String::from(slot), reason: format!("the dataset name itself is refused: {}", why.message()) });
    }
    return Ok(String::from(slot));
}

/// The text one property must be,
/// or the refusal naming the state the reply holds.
fn require_text(name: &str, property: &str, value: &PropertyValue, step: &'static str) -> Result<String, CloneRefusal> {
    return match value {
        PropertyValue::Text(text) => Ok(text.clone()),
        other => Err(CloneRefusal::Property { name: String::from(name), said: other.describe(property), step }),
    };
}

/// The number one property must be,
/// or the refusal naming the state the reply holds.
fn require_number(name: &str, property: &str, value: &PropertyValue, step: &'static str) -> Result<u64, CloneRefusal> {
    return match value {
        PropertyValue::Number(number) => Ok(*number),
        other => Err(CloneRefusal::Property { name: String::from(name), said: other.describe(property), step }),
    };
}

/// What:
///  Whether one child of the pinned parent is a clone this sweep attaches,
///  and which slot and receipt file it gives.
/// Why:
///  The order is the design's:
///  the name,
///  the type,
///  the GUID,
///  the origin,
///  the mountpoint and the two safety properties,
///  and the key.
///  The origin is a snapshot of the checkout dataset,
///  beside the container,
///  because the add route snapshots the checkout and clones that snapshot
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Dataset and path layout",
///  "Requests,
/// results and ordering",
///  item 3);
///  a snapshot of the container itself is nothing that route makes.
///  The key is last because it is the only one whose answer is about the data rather than about
///  the dataset's own shape,
///  and a report says it apart from every identity or permission refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verified = verify(facts, 'pool/clones', 'pool/checkout');
/// ```
///
/// # Errors
/// Every cause `CloneRefusal` names,
/// in the design's order.
pub fn verify(facts: &CloneObjectFacts, parent: &str, checkout_dataset: &str) -> Result<VerifiedClone, CloneRefusal> {
    let slot: String = slot_of(facts.name.as_str(), parent)?;
    if facts.object_type != DMU_OBJECT_SET_FILESYSTEM {
        return Err(CloneRefusal::NotAFilesystem { name: facts.name.clone(), found: facts.object_type });
    }
    if facts.is_snapshot {
        return Err(CloneRefusal::Snapshot { name: facts.name.clone() });
    }
    if facts.guid == 0 {
        return Err(CloneRefusal::NoGuid { name: facts.name.clone() });
    }
    let dataset: DatasetGuid = DatasetGuid::parse(facts.guid.to_string().as_str()).map_err(|reason| return CloneRefusal::Guid { name: facts.name.clone(), reason })?;
    let origin_prefix: String = format!("{checkout_dataset}{SNAPSHOT_SEPARATOR}");
    if facts.origin.is_empty() {
        return Err(CloneRefusal::NoOrigin { name: facts.name.clone() });
    }
    if !facts.origin.starts_with(origin_prefix.as_str()) {
        return Err(CloneRefusal::OriginOutsideTheCheckout { name: facts.name.clone(), origin: facts.origin.clone(), checkout_dataset: String::from(checkout_dataset) });
    }
    let snapshot_name: &str = &facts.origin[origin_prefix.len()..];
    if !snapshot_name.starts_with(CLONE_ORIGIN_SNAPSHOT_PREFIX) {
        return Err(CloneRefusal::OriginIsNotACloneOrigin { name: facts.name.clone(), origin: facts.origin.clone(), prefix: CLONE_ORIGIN_SNAPSHOT_PREFIX });
    }
    let mountpoint: String = require_text(facts.name.as_str(), MOUNTPOINT_PROPERTY, &facts.mountpoint, "mountpoint")?;
    if mountpoint != LEGACY_MOUNTPOINT {
        return Err(CloneRefusal::MountpointIsNotLegacy { name: facts.name.clone(), found: mountpoint });
    }
    let setuid: u64 = require_number(facts.name.as_str(), SETUID_PROPERTY, &facts.setuid, "setuid")?;
    if setuid != INDEX_OFF {
        return Err(CloneRefusal::SetuidIsOn { name: facts.name.clone(), found: setuid.to_string() });
    }
    let devices: u64 = require_number(facts.name.as_str(), DEVICES_PROPERTY, &facts.devices, "devices")?;
    if devices != INDEX_OFF {
        return Err(CloneRefusal::DevicesIsOn { name: facts.name.clone(), found: devices.to_string() });
    }
    // An unencrypted dataset's reply carries no `encryption` property at all, which is the same instruction as `off`:
    // a reply that holds no such property says the data is not encrypted, and only a value this route cannot read is
    // refused. `zfs get encryption <plain dataset>` prints `off` for the same reply.
    let encryption: u64 = match &facts.encryption {
        PropertyValue::Absent => ENCRYPTION_OFF,
        other => require_number(facts.name.as_str(), ENCRYPTION_PROPERTY, other, "encryption")?,
    };
    if encryption != ENCRYPTION_OFF {
        let key_status: u64 = require_number(facts.name.as_str(), KEY_STATUS_PROPERTY, &facts.key_status, "key-unavailable")?;
        if key_status == KEY_STATUS_UNAVAILABLE {
            let root: String = require_text(facts.name.as_str(), ENCRYPTION_ROOT_PROPERTY, &facts.encryption_root, "key-unavailable")?;
            return Err(CloneRefusal::KeyUnavailable { name: facts.name.clone(), encryption_root: root, encryption: format!("encryption={encryption}") });
        }
    }
    return Ok(VerifiedClone { slot, dataset, dataset_name: facts.name.clone() });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_clone_check_tests.rs"]
mod tests;
