//! What:
//!  What the pool answered,
//! read back and refused:
//! one dataset's identity,
//! one
//!  clone's identity with the properties its reply carried,
//! the sentence for a
//!  clone whose mountpoint moved,
//! and the shapes a reply must not hold.
//! Why:
//!  A write's outcome is unknown the moment the kernel refuses it,
//! so no request
//!  is sent twice and each answer is read back before its identity is recorded
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//! A8 and A9).
//!  `add_zfs.rs` sends the requests and would go over the file-size limit
//!  carrying the reading of their answers,
//!  and `add_zfs_inventories.rs` lists what an origin and a clone hold,
//!  so this file is every answer read back and every refusal of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = readIdentity(device, name); // GUID and transaction group
//! const made = readTheClone(device, name); // identity and properties at once
//! const failure = theClonesMountpointFailure(facts, container); // string | undefined
//! ```

use crate::doctor_zfs_route::zfs_checks::LEGACY_MOUNTPOINT;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    MOUNTPOINT_PROPERTY,
    PropertyValue,
    facts_of as facts_from_the_reply,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::local_user_property::{
    PropertyReading,
    read_local_property,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    NvList,
    ZfsDevice,
    requests,
};

/// What:
///  One dataset or snapshot's identity,
///  the two numbers the record writes.
/// Why:
///  A name alone proves nothing about which object the pool holds,
///  so every read-back gives the GUID and the transaction group together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ObjectIdentity = { guid: DatasetGuid; createtxg: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectIdentity {
    /// The object's GUID.
    pub guid: DatasetGuid,
    /// The transaction group that created it.
    pub createtxg: u64,
}

/// What:
///  What one clone request answered:
///  the identity the record keeps,
///  and the properties the add reads to report a clone it cannot use.
/// Why:
///  The kernel's own reply for the clone carries every property with its value
///  and its source,
///  so the add judges the clone without asking the pool a second question
///  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneMade = { identity: ObjectIdentity; facts: CloneObjectFacts };
/// ```
pub struct CloneMade {
    /// The clone's own identity,
    /// which the record writes.
    pub identity: ObjectIdentity,
    /// The clone's properties,
    /// read from the same reply.
    pub facts: CloneObjectFacts,
}

/// One dataset's identity from the pool's statistics.
pub(super) fn read_identity(
    device: &ZfsDevice,
    name: &str,
) -> Result<ObjectIdentity, String> {
    let identity: DatasetIdentity = device
        .objset_stats(name)
        .map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
    return Ok(ObjectIdentity {
        guid: guid_of(identity.stats.dds_guid, name)?,
        createtxg: identity.stats.dds_creation_txg,
    });
}

/// What:
///  A clone's identity and its properties from the one reply that follows the
///  clone request.
/// Why:
///  The record needs the identity and the add's report needs the properties,
///  and the kernel gave both in this reply,
///  so the clone costs one read rather than two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made: CloneMade = { identity: identityOf(reply), facts: factsOf(name, reply) };
/// ```
pub(super) fn read_the_clone(
    device: &ZfsDevice,
    name: &str,
) -> Result<CloneMade, String> {
    let identity: DatasetIdentity = device
        .objset_stats(name)
        .map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
    let object: ObjectIdentity = ObjectIdentity {
        guid: guid_of(identity.stats.dds_guid, name)?,
        createtxg: identity.stats.dds_creation_txg,
    };
    return Ok(CloneMade {
        identity: object,
        facts: facts_from_the_reply(name, &identity),
    });
}

/// What:
///  The sentence for a clone whose `mountpoint` is not `legacy` inherited from
///  the container,
///  or nothing when it is.
/// Why:
///  The design has the add report such a clone and destroy nothing
///  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`),
///  and a clone whose property moved is not one this route may mount.
///  The value and the source are read from the reply the pool already gave,
///  so a person learns which dataset set the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = theClonesMountpointFailure(facts, container); // string | undefined
/// ```
pub fn the_clones_mountpoint_failure(
    facts: &CloneObjectFacts,
    container: &str,
) -> Option<String> {
    let value: PropertyValue = facts.mountpoint.clone();
    let source: PropertyReading<'_> = read_local_property(
        Some(&facts.properties),
        facts.name.as_str(),
        MOUNTPOINT_PROPERTY,
    );
    let required: String = format!(
        "a clone must read `{MOUNTPOINT_PROPERTY}` `{LEGACY_MOUNTPOINT}` inherited from its container `{container}`"
    );
    return match &source {
        PropertyReading::Inherited {
            from,
        } if from == container && value.text() == Some(LEGACY_MOUNTPOINT) => None,
        PropertyReading::Inherited {
            from,
        } => Some(format!(
            "the clone `{}`: {}, inherited from `{from}`, and {required}",
            facts.name,
            value.describe(MOUNTPOINT_PROPERTY)
        )),
        PropertyReading::Local {
            ..
        } => Some(format!(
            "the clone `{}`: {}, set on the clone itself, and {required}",
            facts.name,
            value.describe(MOUNTPOINT_PROPERTY)
        )),
        PropertyReading::Absent => Some(format!(
            "the clone `{}` answers no `{MOUNTPOINT_PROPERTY}` property, and {required}",
            facts.name
        )),
        PropertyReading::Malformed(refusal) => Some(refusal.message(MOUNTPOINT_PROPERTY)),
    };
}

/// A GUID the kernel reported,
/// refused when it is zero,
/// which no live object answers.
pub(super) fn guid_of(
    value: u64,
    name: &str,
) -> Result<DatasetGuid, String> {
    if value == 0 {
        return Err(format!(
            "the pool reported GUID 0 for {name}, and no live object has that identity"
        ));
    }
    return DatasetGuid::parse(value.to_string().as_str()).map_err(|refusal| {
        return format!("the pool's GUID for {name} is not canonical ({refusal:?})");
    });
}

/// The name one listing reports without its dataset prefix.
pub(super) fn short_name(
    full: &str,
    dataset: &str,
) -> String {
    if let Some((prefix, rest)) = full.split_once('@')
        && prefix == dataset
    {
        return String::from(rest);
    }
    if let Some((prefix, rest)) = full.split_once('#')
        && prefix == dataset
    {
        return String::from(rest);
    }
    return String::from(full);
}

/// A property nested as `{value}` in a bookmark's reply.
pub(super) fn nested_u64(
    entry: &NvList,
    key: &str,
) -> Option<u64> {
    return entry
        .get_list(key)
        .and_then(|value| return value.get_u64("value"));
}

/// Refuse when a write's reply carries a per-name failure,
/// naming each failed name and the kernel's error.
pub(super) fn refuse_any_element(
    reply: Option<&NvList>,
    what: &str,
) -> Result<(), String> {
    let Some(found_reply) = reply else {
        return Ok(());
    };
    let failures: Vec<requests::ElementError> = requests::element_errors(found_reply);
    if failures.is_empty() {
        return Ok(());
    }
    let shown: Vec<String> = failures
        .iter()
        .map(|failure| return failure.to_string())
        .collect();
    return Err(format!(
        "{what} was refused for {}; nothing was retried. Ways forward: free space in the pool or inspect the objects, then run the add again",
        shown.join(", ")
    ));
}
