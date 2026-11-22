//! What:
//!  Controls of one dataset's facts,
//!  read from a reply this control builds the way OpenZFS writes one.
//! Why:
//!  The checks must tell a missing property from a pair they cannot read and
//!  from the value they expect,
//!  and the listener's own reply shape is the only shape that decides it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(valueOf(reply, 'mountpoint')).toEqual({ kind: 'text', text: 'legacy' });
//! ```

use super::{
    CloneObjectFacts, DEVICES_PROPERTY, DMU_OBJECT_SET_FILESYSTEM, ENCRYPTION_PROPERTY, ENCRYPTION_ROOT_PROPERTY, KEY_STATUS_PROPERTY, KEY_STATUS_UNAVAILABLE,
    MOUNTPOINT_PROPERTY, PropertyValue, SETUID_PROPERTY, facts_of, value_of,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DmuObjsetStats, NvList};

/// The clone the controls read.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// The parent snapshot the clone's origin names.
const ORIGIN: &str = "zpcachyos/nosnap/clones@origin";

/// One property as OpenZFS writes it:
/// a nested list of its value and the dataset the value comes from.
fn pair_number(value: u64, source: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", value);
    pair.add_str("source", source);
    return pair;
}

/// One text property in the same shape.
fn pair_text(value: &str, source: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", source);
    return pair;
}

/// A reply holding the properties a managed clone carries.
fn reply_of_a_clone() -> NvList {
    let mut reply: NvList = NvList::new();
    reply.add_nvlist(MOUNTPOINT_PROPERTY, pair_text("legacy", "zpcachyos/nosnap"));
    reply.add_nvlist(SETUID_PROPERTY, pair_number(0, "zpcachyos/nosnap"));
    reply.add_nvlist(DEVICES_PROPERTY, pair_number(0, "zpcachyos/nosnap"));
    reply.add_nvlist(ENCRYPTION_PROPERTY, pair_number(7, CLONE));
    reply.add_nvlist(ENCRYPTION_ROOT_PROPERTY, pair_text("zpcachyos", CLONE));
    reply.add_nvlist(KEY_STATUS_PROPERTY, pair_number(2, CLONE));
    return reply;
}

/// The kernel's statistics for the clone,
/// whose origin names the parent's snapshot.
fn stats_of_a_clone() -> DmuObjsetStats {
    let mut origin: [u8; 256] = [0; 256];
    origin[..ORIGIN.len()].copy_from_slice(ORIGIN.as_bytes());
    return DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 42,
        dds_guid: 9_000_000_000_000_000_001,
        dds_type: DMU_OBJECT_SET_FILESYSTEM,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: origin,
        dds_flags: 0,
    };
}

/// The identity a child of the pinned parent answers with.
fn identity() -> DatasetIdentity {
    return DatasetIdentity { stats: stats_of_a_clone(), properties: reply_of_a_clone() };
}

/// Every value the checks read comes out of the reply it belongs to.
#[test]
fn a_managed_clones_facts_come_out_of_its_reply() {
    let facts: CloneObjectFacts = facts_of(CLONE, &identity());
    assert_eq!(facts.name, CLONE);
    assert_eq!(facts.guid, 9_000_000_000_000_000_001);
    assert_eq!(facts.object_type, DMU_OBJECT_SET_FILESYSTEM);
    assert!(!facts.is_snapshot);
    assert_eq!(facts.origin, ORIGIN);
    assert_eq!(facts.mountpoint, PropertyValue::Text(String::from("legacy")));
    assert_eq!(facts.setuid, PropertyValue::Number(0));
    assert_eq!(facts.devices, PropertyValue::Number(0));
    assert_eq!(facts.encryption, PropertyValue::Number(7));
    assert_eq!(facts.encryption_root, PropertyValue::Text(String::from("zpcachyos")));
    assert_eq!(facts.key_status, PropertyValue::Number(2));
    assert_ne!(facts.key_status.number(), Some(KEY_STATUS_UNAVAILABLE));
}

/// A property the reply does not hold is absent rather than a value.
#[test]
fn a_property_the_reply_does_not_hold_is_absent() {
    let empty: NvList = NvList::new();
    assert_eq!(value_of(&empty, MOUNTPOINT_PROPERTY), PropertyValue::Absent);
    assert!(PropertyValue::Absent.text().is_none());
    assert!(PropertyValue::Absent.number().is_none());
    let said: String = PropertyValue::Absent.describe(MOUNTPOINT_PROPERTY);
    assert!(said.contains(MOUNTPOINT_PROPERTY), "the description names the property: {said}");
    assert!(said.contains("not in the reply"), "the description says it is not there: {said}");
}

/// A value that is not the `value`/`source` pair is unreadable rather than absent,
/// because only one of the two is a dataset an administrator wrote wrongly.
#[test]
fn a_value_outside_the_pair_is_unreadable() {
    let mut bare: NvList = NvList::new();
    bare.add_str(MOUNTPOINT_PROPERTY, "legacy");
    assert!(matches!(value_of(&bare, MOUNTPOINT_PROPERTY), PropertyValue::Unreadable { .. }));
    let mut pairless: NvList = NvList::new();
    pairless.add_nvlist(MOUNTPOINT_PROPERTY, NvList::new());
    assert!(matches!(value_of(&pairless, MOUNTPOINT_PROPERTY), PropertyValue::Unreadable { .. }));
    let mut source_only: NvList = NvList::new();
    let mut pair: NvList = NvList::new();
    pair.add_str("source", "zpcachyos");
    source_only.add_nvlist(MOUNTPOINT_PROPERTY, pair);
    let unreadable: PropertyValue = value_of(&source_only, MOUNTPOINT_PROPERTY);
    assert!(matches!(unreadable, PropertyValue::Unreadable { .. }), "{unreadable:?}");
    assert!(unreadable.describe(MOUNTPOINT_PROPERTY).contains("could not be read"), "a person learns it was unreadable: {}", unreadable.describe(MOUNTPOINT_PROPERTY));
}

/// A dataset that is not a clone reports no origin,
/// which is how a plain child of the parent is told from a clone of it.
#[test]
fn a_dataset_that_is_not_a_clone_reports_no_origin() {
    let mut plain: DmuObjsetStats = stats_of_a_clone();
    plain.dds_origin = [0; 256];
    let facts: CloneObjectFacts = facts_of(CLONE, &DatasetIdentity { stats: plain, properties: reply_of_a_clone() });
    assert!(facts.origin.is_empty(), "a dataset with no origin reports none: {:?}", facts.origin);
}
