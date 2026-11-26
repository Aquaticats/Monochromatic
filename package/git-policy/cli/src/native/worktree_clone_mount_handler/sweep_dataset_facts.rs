//! What:
//!  What one dataset the sweep reached says about itself:
//!  its name,
//!  its own GUID,
//!  its object-set type,
//!  its origin,
//!  and the property values the sweep's checks read.
//! Why:
//!  The sweep verifies each immediate child of the pinned parent before it attaches anything,
//!  and every fact that check reads comes from the kernel's own reply
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  OpenZFS writes each property as one nested list holding `value` and `source`
//!  (`module/zfs/dsl_prop.c`,
//! OpenZFS `zfs-2.4.4`,
//! as `local_user_property` describes),
//!  so a value that is not that pair is its own answer rather than a guess:
//!  a check that cannot see `mountpoint` must refuse,
//!  because an absent mountpoint and a legacy one are different instructions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = factsOf(name, identity); // { guid, type, origin, mountpoint, setuid, devices, keystatus, ... }
//! ```

use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    NvList,
};

/// What:
///  The one object-set type a clone this route attaches has.
/// Why:
///   A clone is a filesystem and never a volume or a snapshot,
///  and the kernel reports the type in `dmu_objset_stats.dds_type`
///  (`include/sys/fs/zfs.h` at tag `zfs-2.4.4`,
///  `typedef enum dmu_objset_type`,
///  where `DMU_OST_ZFS` is the third member).
///  The value is written out because the binding's layout pins offsets,
///  not the enum's members,
///  and `dmu_objset_type_t` has been reordered between OpenZFS releases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filesystemType = 2; // DMU_OST_ZFS
/// ```
pub const DMU_OBJECT_SET_FILESYSTEM: u32 = 2;

/// The property naming the folder a dataset mounts at,
/// or `legacy` for a dataset nothing mounts automatically.
pub const MOUNTPOINT_PROPERTY: &str = "mountpoint";

/// The property that removes set-user-ID execution from a dataset mount.
pub const SETUID_PROPERTY: &str = "setuid";

/// The property that removes device nodes from a dataset mount.
pub const DEVICES_PROPERTY: &str = "devices";

/// The property naming the cipher a dataset's data is encrypted with,
/// or `off`.
pub const ENCRYPTION_PROPERTY: &str = "encryption";

/// The property naming the dataset whose key the data is encrypted under.
pub const ENCRYPTION_ROOT_PROPERTY: &str = "encryptionroot";

/// The property that says whether that key is loaded.
pub const KEY_STATUS_PROPERTY: &str = "keystatus";

/// What:
///  The `keystatus` value that means the key is not loaded,
///  which is what item 5 of the design's VM measurements saw on a clone made
///  with the pool's key unloaded
///  (`package/zfs/request-to-openzfs-kernel-device/doc/guest-measurements-2026-11-19.md`,
///  where the value was 1 and 2 is available).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unavailable = 1; // ZFS_KEYSTATUS_UNAVAILABLE
/// ```
pub const KEY_STATUS_UNAVAILABLE: u64 = 1;

/// What:
///  The `setuid` and `devices` value that means off,
///  and the one that means on.
/// Why:
///  Both properties are OpenZFS index properties,
///  whose values the reply carries as numbers rather than the words
///  `zfs get` prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const off = 0; const on = 1;
/// ```
pub const INDEX_OFF: u64 = 0;

/// The `setuid` and `devices` value that means on.
pub const INDEX_ON: u64 = 1;

/// What:
///  One property of one dataset,
///  as the kernel's reply holds it.
/// Why:
///  A property that is not there and a property whose reply is not the
///  `value`/`source` pair are different states a check must tell apart,
///  and only one of them is a dataset an administrator configured wrongly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropertyValue = { kind: 'absent' } | { kind: 'text'; text } | { kind: 'number'; number } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyValue {
    /// The reply holds no such property at all.
    Absent,
    /// The property's value is text.
    Text(String),
    /// The property's value is a number.
    Number(u64),
    /// The reply's entry is not the pair OpenZFS writes.
    Unreadable {
        /// What about the entry could not be read.
        why: String,
    },
}

impl PropertyValue {
    /// The property's text,
    /// or nothing for every other state.
    pub fn text(&self) -> Option<&str> {
        return match self {
            PropertyValue::Text(text) => Some(text.as_str()),
            PropertyValue::Absent
            | PropertyValue::Number(_)
            | PropertyValue::Unreadable {
                ..
            } => None,
        };
    }

    /// The property's number,
    /// or nothing for every other state.
    pub fn number(&self) -> Option<u64> {
        return match self {
            PropertyValue::Number(number) => Some(*number),
            PropertyValue::Absent
            | PropertyValue::Text(_)
            | PropertyValue::Unreadable {
                ..
            } => None,
        };
    }

    /// How a refusal names this state,
    /// so a person learns whether the property is missing or was written by
    /// another OpenZFS release.
    pub fn describe(
        &self,
        property: &str,
    ) -> String {
        return match self {
            PropertyValue::Absent => {
                format!("`{property}` is not in the reply the kernel gave for this dataset")
            }
            PropertyValue::Text(text) => format!("`{property}` is `{text}`"),
            PropertyValue::Number(number) => format!("`{property}` is {number}"),
            PropertyValue::Unreadable {
                why,
            } => format!("`{property}` could not be read ({why})"),
        };
    }
}

/// What:
///  Everything the sweep reads from one child of the pinned parent.
/// Why:
///  The checks and the receipt are filled from one reply,
///  so two checks cannot judge two different states of the same dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneObjectFacts = { name; guid; objectType; isSnapshot; origin; mountpoint; setuid; devices; encryption; encryptionRoot; keyStatus; properties };
/// ```
#[derive(Clone, Debug)]
pub struct CloneObjectFacts {
    /// The dataset's full name,
    /// as the kernel's listing wrote it.
    pub name: String,
    /// The dataset's own GUID.
    pub guid: u64,
    /// The object-set type the kernel reported.
    pub object_type: u32,
    /// Whether the kernel called this a snapshot.
    pub is_snapshot: bool,
    /// The origin snapshot's name when the dataset is a clone.
    pub origin: String,
    /// Where the dataset mounts,
    /// or `legacy`.
    pub mountpoint: PropertyValue,
    /// Whether a mount of it carries set-user-ID execution.
    pub setuid: PropertyValue,
    /// Whether a mount of it carries device nodes.
    pub devices: PropertyValue,
    /// The cipher the data is encrypted with,
    /// or `off`.
    pub encryption: PropertyValue,
    /// The dataset whose key the data is encrypted under.
    pub encryption_root: PropertyValue,
    /// Whether that key is loaded.
    pub key_status: PropertyValue,
    /// The property list itself,
    /// which the intent's own reader takes.
    pub properties: NvList,
}

/// What:
///  One property's value out of a reply,
///  read as the pair OpenZFS writes.
/// Why:
///  Every property the sweep checks arrives in the same shape,
///  so the shape is decided once and a value of an unexpected type is
///  reported rather than silently treated as absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = valueOf(reply.properties, 'mountpoint');
/// ```
pub fn value_of(
    properties: &NvList,
    property: &str,
) -> PropertyValue {
    let Some(entry) = properties.get(property) else {
        return PropertyValue::Absent;
    };
    let Some(pair) = properties.get_list(property) else {
        return PropertyValue::Unreadable {
            why: format!(
                "the value is not the pair of `value` and `source` OpenZFS returns for a property ({entry:?})"
            ),
        };
    };
    if let Some(text) = pair.get_str("value") {
        return PropertyValue::Text(String::from(text));
    }
    if let Some(number) = pair.get_u64("value") {
        return PropertyValue::Number(number);
    }
    return PropertyValue::Unreadable {
        why: String::from("the pair has no `value` this route reads"),
    };
}

/// What:
///  Read one dataset's facts from the reply the kernel gave for it.
/// Why:
///  The name comes from the listing rather than from the fact's own property,
///  because the name the sweep walked to is the name it acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = factsOf(child.name, child.identity);
/// ```
pub fn facts_of(
    name: &str,
    identity: &DatasetIdentity,
) -> CloneObjectFacts {
    let properties: &NvList = &identity.properties;
    return CloneObjectFacts {
        name: String::from(name),
        guid: identity.stats.dds_guid,
        object_type: identity.stats.dds_type,
        is_snapshot: identity.stats.dds_is_snapshot != 0,
        origin: identity.stats.origin(),
        mountpoint: value_of(properties, MOUNTPOINT_PROPERTY),
        setuid: value_of(properties, SETUID_PROPERTY),
        devices: value_of(properties, DEVICES_PROPERTY),
        encryption: value_of(properties, ENCRYPTION_PROPERTY),
        encryption_root: value_of(properties, ENCRYPTION_ROOT_PROPERTY),
        key_status: value_of(properties, KEY_STATUS_PROPERTY),
        properties: properties.clone(),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_dataset_facts_tests.rs"]
mod tests;
