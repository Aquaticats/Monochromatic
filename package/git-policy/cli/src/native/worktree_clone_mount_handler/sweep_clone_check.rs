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
    CloneObjectFacts,
    DEVICES_PROPERTY,
    DMU_OBJECT_SET_FILESYSTEM,
    ENCRYPTION_PROPERTY,
    ENCRYPTION_ROOT_PROPERTY,
    INDEX_OFF,
    KEY_STATUS_PROPERTY,
    KEY_STATUS_UNAVAILABLE,
    MOUNTPOINT_PROPERTY,
    PropertyValue,
    SETUID_PROPERTY,
};
use crate::start_probe::probe_source_name::check_dataset_name;
use crate::worktree_clone_mount::SLOT_NAME_MAXIMUM_BYTES;
use crate::worktree_clone_mount::slot_name_refusal;
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// The values the design gives a clone's own properties and its origin
/// snapshot,
/// which a caller of these checks names by this file.
pub use super::sweep_clone_property_values::{
    CLONE_ORIGIN_SNAPSHOT_PREFIX,
    ENCRYPTION_OFF,
    LEGACY_MOUNTPOINT,
    SNAPSHOT_SEPARATOR,
};

/// Why one immediate child of the pinned parent is not a clone this sweep
/// attaches,
/// and the words each cause answers.
pub use super::sweep_clone_refusals::CloneRefusal;

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
pub fn slot_of(
    name: &str,
    parent: &str,
) -> Result<String, CloneRefusal> {
    let prefix: String = format!("{parent}/");
    let Some(slot) = name.strip_prefix(prefix.as_str()) else {
        return Err(CloneRefusal::NotAChild {
            name: String::from(name),
            parent: String::from(parent),
        });
    };
    if slot.contains('/') || slot.is_empty() {
        return Err(CloneRefusal::NotAChild {
            name: String::from(name),
            parent: String::from(parent),
        });
    }
    if let Some(reason) = clone_slot_refusal(slot) {
        return Err(CloneRefusal::SlotName {
            name: String::from(name),
            slot: String::from(slot),
            reason,
        });
    }
    if slot.len() as u64 > SLOT_NAME_MAXIMUM_BYTES {
        return Err(CloneRefusal::SlotName {
            name: String::from(name),
            slot: String::from(slot),
            reason: format!(
                "it is longer than the {SLOT_NAME_MAXIMUM_BYTES} bytes a slot record's name holds"
            ),
        });
    }
    if let Err(why) = check_dataset_name(name) {
        return Err(CloneRefusal::SlotName {
            name: String::from(name),
            slot: String::from(slot),
            reason: format!("the dataset name itself is refused: {}", why.message()),
        });
    }
    return Ok(String::from(slot));
}

/// The text one property must be,
/// or the refusal naming the state the reply holds.
fn require_text(
    name: &str,
    property: &str,
    value: &PropertyValue,
    step: &'static str,
) -> Result<String, CloneRefusal> {
    return match value {
        PropertyValue::Text(text) => Ok(text.clone()),
        other => Err(CloneRefusal::Property {
            name: String::from(name),
            said: other.describe(property),
            step,
        }),
    };
}

/// The number one property must be,
/// or the refusal naming the state the reply holds.
fn require_number(
    name: &str,
    property: &str,
    value: &PropertyValue,
    step: &'static str,
) -> Result<u64, CloneRefusal> {
    return match value {
        PropertyValue::Number(number) => Ok(*number),
        other => Err(CloneRefusal::Property {
            name: String::from(name),
            said: other.describe(property),
            step,
        }),
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
pub fn verify(
    facts: &CloneObjectFacts,
    parent: &str,
    checkout_dataset: &str,
) -> Result<VerifiedClone, CloneRefusal> {
    let slot: String = slot_of(facts.name.as_str(), parent)?;
    if facts.object_type != DMU_OBJECT_SET_FILESYSTEM {
        return Err(CloneRefusal::NotAFilesystem {
            name: facts.name.clone(),
            found: facts.object_type,
        });
    }
    if facts.is_snapshot {
        return Err(CloneRefusal::Snapshot {
            name: facts.name.clone(),
        });
    }
    if facts.guid == 0 {
        return Err(CloneRefusal::NoGuid {
            name: facts.name.clone(),
        });
    }
    let dataset: DatasetGuid =
        DatasetGuid::parse(facts.guid.to_string().as_str()).map_err(|reason| {
            return CloneRefusal::Guid {
                name: facts.name.clone(),
                reason,
            };
        })?;
    let origin_prefix: String = format!("{checkout_dataset}{SNAPSHOT_SEPARATOR}");
    if facts.origin.is_empty() {
        return Err(CloneRefusal::NoOrigin {
            name: facts.name.clone(),
        });
    }
    if !facts.origin.starts_with(origin_prefix.as_str()) {
        return Err(CloneRefusal::OriginOutsideTheCheckout {
            name: facts.name.clone(),
            origin: facts.origin.clone(),
            checkout_dataset: String::from(checkout_dataset),
        });
    }
    let snapshot_name: &str = &facts.origin[origin_prefix.len()..];
    if !snapshot_name.starts_with(CLONE_ORIGIN_SNAPSHOT_PREFIX) {
        return Err(CloneRefusal::OriginIsNotACloneOrigin {
            name: facts.name.clone(),
            origin: facts.origin.clone(),
            prefix: CLONE_ORIGIN_SNAPSHOT_PREFIX,
        });
    }
    let mountpoint: String = require_text(
        facts.name.as_str(),
        MOUNTPOINT_PROPERTY,
        &facts.mountpoint,
        "mountpoint",
    )?;
    if mountpoint != LEGACY_MOUNTPOINT {
        return Err(CloneRefusal::MountpointIsNotLegacy {
            name: facts.name.clone(),
            found: mountpoint,
        });
    }
    let setuid: u64 = require_number(
        facts.name.as_str(),
        SETUID_PROPERTY,
        &facts.setuid,
        "setuid",
    )?;
    if setuid != INDEX_OFF {
        return Err(CloneRefusal::SetuidIsOn {
            name: facts.name.clone(),
            found: setuid.to_string(),
        });
    }
    let devices: u64 = require_number(
        facts.name.as_str(),
        DEVICES_PROPERTY,
        &facts.devices,
        "devices",
    )?;
    if devices != INDEX_OFF {
        return Err(CloneRefusal::DevicesIsOn {
            name: facts.name.clone(),
            found: devices.to_string(),
        });
    }
    // An unencrypted dataset's reply carries no `encryption` property at all, which is the same instruction as `off`:
    // a reply that holds no such property says the data is not encrypted, and only a value this route cannot read is
    // refused. `zfs get encryption <plain dataset>` prints `off` for the same reply.
    let encryption: u64 = match &facts.encryption {
        PropertyValue::Absent => ENCRYPTION_OFF,
        other => require_number(
            facts.name.as_str(),
            ENCRYPTION_PROPERTY,
            other,
            "encryption",
        )?,
    };
    if encryption != ENCRYPTION_OFF {
        let key_status: u64 = require_number(
            facts.name.as_str(),
            KEY_STATUS_PROPERTY,
            &facts.key_status,
            "key-unavailable",
        )?;
        if key_status == KEY_STATUS_UNAVAILABLE {
            let root: String = require_text(
                facts.name.as_str(),
                ENCRYPTION_ROOT_PROPERTY,
                &facts.encryption_root,
                "key-unavailable",
            )?;
            return Err(CloneRefusal::KeyUnavailable {
                name: facts.name.clone(),
                encryption_root: root,
                encryption: format!("encryption={encryption}"),
            });
        }
    }
    return Ok(VerifiedClone {
        slot,
        dataset,
        dataset_name: facts.name.clone(),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_clone_check_tests.rs"]
mod tests;
