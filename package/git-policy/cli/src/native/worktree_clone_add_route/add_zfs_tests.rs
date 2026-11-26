//! Controls for the pool calls that need no pool:
//!  a write's per-name failure is reported with its name and error,
//!  a GUID the pool did not report is refused,
//!  the clone request sends no property,
//!  and the clone's `mountpoint` is judged from fixture facts for every state
//!  the kernel's reply can hold.

use super::{ObjectIdentity, guid_of, refuse_any_element, the_clones_mountpoint_failure};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{CloneObjectFacts, DMU_OBJECT_SET_FILESYSTEM, facts_of};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DmuObjsetStats, NvList};

/// The container the fixture clones live under.
const CONTAINER: &str = "tank/clones";

/// The clone the fixture reads.
const CLONE: &str = "tank/clones/topic";

#[test]
fn a_per_name_failure_is_named_with_its_error() {
    let mut reply: NvList = NvList::new();
    reply.add_i32("tank/clones/topic", 28);
    let refusal: String = refuse_any_element(Some(&reply), "the clone").expect_err("a failed element refuses the write");
    assert!(refusal.contains("tank/clones/topic"), "{refusal}");
    assert!(refusal.contains("No space left on device"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
    assert_eq!(refuse_any_element(Some(&NvList::new()), "the clone"), Ok(()), "an empty reply is a success");
    assert_eq!(refuse_any_element(None, "the clone"), Ok(()), "no reply is a success");
}

#[test]
fn a_guid_of_zero_is_refused() {
    let refusal: String = guid_of(0, "tank/clones/topic").expect_err("no live object answers GUID 0");
    assert!(refusal.contains("GUID 0"), "{refusal}");
    assert!(guid_of(7, "tank/clones/topic").is_ok());
    let identity: ObjectIdentity = ObjectIdentity { guid: guid_of(7, "tank/clones/topic").expect("the guid"), createtxg: 9 };
    assert_eq!(identity.guid.file_name(), "7");
    assert_eq!(identity.createtxg, 9);
}

/// What:
///  A `mountpoint` property as OpenZFS writes one:
///  a nested list holding the value and the dataset the value comes from.
/// Why:
///  The clone's judgement reads the source as well as the value,
///  and this control has to build the same shape the kernel answers with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pair: { value: string; source: string } = { value, source };
/// ```
fn mountpoint_pair(value: &str, source: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", source);
    return pair;
}

/// What:
///  The statistics one reply carries,
///  which name a filesystem and its GUID.
/// Why:
///  A clone's facts are read from the same reply as its identity,
///  so the fixture needs both halves to reach the judgement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stats = { guid: 9_000_000_000_000_000_001, type: 2, isSnapshot: 0, origin: '' };
/// ```
fn stats() -> DmuObjsetStats {
    return DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 42,
        dds_guid: 9_000_000_000_000_000_001,
        dds_type: DMU_OBJECT_SET_FILESYSTEM,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: [0; 256],
        dds_flags: 0,
    };
}

/// What:
///  The facts of a clone whose reply holds exactly one property entry.
/// Why:
///  Every arm of the judgement is one `mountpoint` reply,
///  so the fixture differs only in the entry it is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = factsOf(CLONE, { stats: stats(), properties: reply });
/// ```
fn facts_with(entry: Option<NvList>) -> CloneObjectFacts {
    let mut properties: NvList = NvList::new();
    if let Some(found) = entry {
        properties.add_nvlist("mountpoint", found);
    }
    let identity: DatasetIdentity = DatasetIdentity { stats: stats(), properties };
    return facts_of(CLONE, &identity);
}

/// A clone that reads `legacy` inherited from the container is the state the
/// design serves, so the add reports nothing.
#[test]
fn a_clone_inheriting_legacy_from_the_container_is_not_reported() {
    let facts: CloneObjectFacts = facts_with(Some(mountpoint_pair("legacy", CONTAINER)));
    assert_eq!(the_clones_mountpoint_failure(&facts, CONTAINER), None);
}

/// A clone whose inherited value is not `legacy` is reported with its value,
/// its source and the value the design requires.
#[test]
fn a_clone_inheriting_another_value_is_reported_with_its_value_and_source() {
    let facts: CloneObjectFacts = facts_with(Some(mountpoint_pair("/srv/elsewhere", CONTAINER)));
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER).expect("a value the design does not serve is reported");
    assert!(reported.contains(CLONE), "{reported}");
    assert!(reported.contains("`/srv/elsewhere`"), "the value is named: {reported}");
    assert!(reported.contains(CONTAINER), "the source is named: {reported}");
    assert!(reported.contains("must read `mountpoint` `legacy` inherited from its container"), "the required value is named: {reported}");
}

/// A clone whose value came from another dataset is reported even when the
/// value is `legacy`, because the source is what the design requires.
#[test]
fn a_clone_inheriting_the_right_value_from_another_dataset_is_reported() {
    let facts: CloneObjectFacts = facts_with(Some(mountpoint_pair("legacy", "tank/other")));
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER).expect("a value inherited from anywhere but the container is reported");
    assert!(reported.contains("tank/other"), "the other source is named: {reported}");
    assert!(reported.contains("must read `mountpoint` `legacy` inherited from its container"), "{reported}");
}

/// A clone that carries its own value is reported, because a local value is
/// one the add cannot prove the container gave it.
#[test]
fn a_clone_setting_its_own_value_is_reported() {
    let facts: CloneObjectFacts = facts_with(Some(mountpoint_pair("legacy", CLONE)));
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER).expect("a local value is reported");
    assert!(reported.contains("set on the clone itself"), "{reported}");
}

/// A clone whose reply holds no `mountpoint` at all is reported as absent
/// rather than read as the value the design wants.
#[test]
fn a_clone_answering_no_mountpoint_is_reported() {
    let facts: CloneObjectFacts = facts_with(None);
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER).expect("an absent property is reported");
    assert!(reported.contains("answers no `mountpoint`"), "{reported}");
}

/// A reply whose entry is not the `value`/`source` pair is reported as
/// unreadable, naming why this route cannot tell a local value from an
/// inherited one.
#[test]
fn a_clone_whose_mountpoint_is_not_the_pair_is_reported() {
    let mut broken: NvList = NvList::new();
    broken.add_str("value", "legacy");
    let facts: CloneObjectFacts = facts_with(Some(broken));
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER).expect("a malformed entry is reported");
    assert!(reported.contains("has no `source` key"), "{reported}");
}

/// The clone request carries no property, which is the delegation the design
/// forbids this account.
#[test]
fn the_clone_request_sends_no_property() {
    let source: &str = include_str!("add_zfs.rs");
    let code: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let joined: String = code.join("\n");
    assert_eq!(joined.matches("self.device.clone(").count(), 1, "the clone request is made once: {joined}");
    let request: &str = joined.split("self.device.clone(").nth(1).expect("the request is there");
    assert!(request.starts_with("target, origin, None"), "the request passes no property list: {request}");
    assert!(!joined.contains("add_str(\"mountpoint\""), "no `mountpoint` property is written: {joined}");
}
