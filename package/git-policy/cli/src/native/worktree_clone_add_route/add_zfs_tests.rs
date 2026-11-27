//! Controls for the pool calls that need no pool:
//!  a write's per-name failure is reported with its name and error,
//!  a GUID the pool did not report is refused,
//!  the clone request sends no property,
//!  and the clone's `mountpoint` is judged from fixture facts for every state
//!  the kernel's reply can hold.

use super::{
    ObjectIdentity,
    guid_of,
    refuse_any_element,
    the_clones_mountpoint_failure,
};
use crate::source_guard::{
    HoldingLine,
    MayHold,
    anchors_the_scope_no_longer_holds,
    lines_holding_one_of,
    lines_no_entry_excuses,
    release_sources_under,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    DMU_OBJECT_SET_FILESYSTEM,
    facts_of,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};

/// The container the fixture clones live under.
const CONTAINER: &str = "tank/clones";

/// The clone the fixture reads.
const CLONE: &str = "tank/clones/topic";

#[test]
fn a_per_name_failure_is_named_with_its_error() {
    let mut reply: NvList = NvList::new();
    reply.add_i32("tank/clones/topic", 28);
    let refusal: String = refuse_any_element(Some(&reply), "the clone")
        .expect_err("a failed element refuses the write");
    assert!(refusal.contains("tank/clones/topic"), "{refusal}");
    assert!(refusal.contains("No space left on device"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
    assert_eq!(
        refuse_any_element(Some(&NvList::new()), "the clone"),
        Ok(()),
        "an empty reply is a success"
    );
    assert_eq!(
        refuse_any_element(None, "the clone"),
        Ok(()),
        "no reply is a success"
    );
}

#[test]
fn a_guid_of_zero_is_refused() {
    let refusal: String =
        guid_of(0, "tank/clones/topic").expect_err("no live object answers GUID 0");
    assert!(refusal.contains("GUID 0"), "{refusal}");
    assert!(guid_of(7, "tank/clones/topic").is_ok());
    let identity: ObjectIdentity = ObjectIdentity {
        guid: guid_of(7, "tank/clones/topic").expect("the guid"),
        createtxg: 9,
    };
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
fn mountpoint_pair(
    value: &str,
    source: &str,
) -> NvList {
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
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(),
        properties,
    };
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
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER)
        .expect("a value the design does not serve is reported");
    assert!(reported.contains(CLONE), "{reported}");
    assert!(
        reported.contains("`/srv/elsewhere`"),
        "the value is named: {reported}"
    );
    assert!(
        reported.contains(CONTAINER),
        "the source is named: {reported}"
    );
    assert!(
        reported.contains("must read `mountpoint` `legacy` inherited from its container"),
        "the required value is named: {reported}"
    );
}

/// A clone whose value came from another dataset is reported even when the
/// value is `legacy`, because the source is what the design requires.
#[test]
fn a_clone_inheriting_the_right_value_from_another_dataset_is_reported() {
    let facts: CloneObjectFacts = facts_with(Some(mountpoint_pair("legacy", "tank/other")));
    let reported: String = the_clones_mountpoint_failure(&facts, CONTAINER)
        .expect("a value inherited from anywhere but the container is reported");
    assert!(
        reported.contains("tank/other"),
        "the other source is named: {reported}"
    );
    assert!(
        reported.contains("must read `mountpoint` `legacy` inherited from its container"),
        "{reported}"
    );
}

/// A clone that carries its own value is reported, because a local value is
/// one the add cannot prove the container gave it.
#[test]
fn a_clone_setting_its_own_value_is_reported() {
    let facts: CloneObjectFacts = facts_with(Some(mountpoint_pair("legacy", CLONE)));
    let reported: String =
        the_clones_mountpoint_failure(&facts, CONTAINER).expect("a local value is reported");
    assert!(reported.contains("set on the clone itself"), "{reported}");
}

/// A clone whose reply holds no `mountpoint` at all is reported as absent
/// rather than read as the value the design wants.
#[test]
fn a_clone_answering_no_mountpoint_is_reported() {
    let facts: CloneObjectFacts = facts_with(None);
    let reported: String =
        the_clones_mountpoint_failure(&facts, CONTAINER).expect("an absent property is reported");
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
    let reported: String =
        the_clones_mountpoint_failure(&facts, CONTAINER).expect("a malformed entry is reported");
    assert!(reported.contains("has no `source` key"), "{reported}");
}

/// What:
///  `THE_ADD_ZFS_MODULE` names the release sources of the pool route's calls:
///  the route,
///  the answers the pool gives while it is made,
///  and the pool's own inventories.
/// Why:
///  The ban below is about the pool route,
///  and the alpha.48 split moved nine items out of `add_zfs.rs` into `add_zfs_pool_answers.rs` and
///  `add_zfs_inventories.rs`,
///  so the scope is stated as a prefix rather than as the one file it used to be
///  (issue 6599).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_ADD_ZFS_MODULE = ['worktree_clone_add_route/add_zfs'];
/// ```
const THE_ADD_ZFS_MODULE: &[&str] = &["worktree_clone_add_route/add_zfs"];

/// What:
///  `THE_CLONE_REQUEST` is the one call that makes the clone.
/// Why:
///  The request is made once and carries no property list,
///  and the count is what says the request is still the one call it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CLONE_REQUEST = 'self.device.clone(\'';
/// ```
const THE_CLONE_REQUEST: &str = "self.device.clone(";

/// What:
///  `THE_LIST_IT_CARRIES` is the arguments the one clone request is made with.
/// Why:
///  `target, origin, None` is what says the request passes no property list,
///  which is the delegation the design forbids this account.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_LIST_IT_CARRIES = 'target, origin, None';
/// ```
const THE_LIST_IT_CARRIES: &str = "target, origin, None";

/// What:
///  `THE_PROPERTY_THAT_IS_NOT_SENT` is the one property the request must not build.
/// Why:
///  A `mountpoint` property would make the pool choose where the clone goes,
///  which is the delegation the account does not make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_PROPERTY_THAT_IS_NOT_SENT = ['add_str("mountpoint"'];
/// ```
const THE_PROPERTY_THAT_IS_NOT_SENT: &[&str] = &["add_str(\"mountpoint\""];

/// What:
///  `MAY_HOLD` is empty:
///  no file of the pool route builds a property list.
/// Why:
///  The ban reaches every file the route was split into,
///  so a file added here fails until a reader decides whether that property answers,
///  and that decision is what an entry in this list would record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAY_HOLD: MayHold[] = [];
/// ```
const MAY_HOLD: &[MayHold] = &[];

/// What:
///  `SCOPE_ANCHORS` is the text that proves the scope still reaches the request this ban was written for.
/// Why:
///  A prefix that reached nothing would pass every tree,
///  and the clone call is what the property's absence is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCOPE_ANCHORS = [['self.device.clone(', 'no file of the pool route makes the clone any more']];
/// ```
const SCOPE_ANCHORS: &[(&str, &str)] = &[(
    THE_CLONE_REQUEST,
    "no release file of the pool route makes the clone any more, so this ban reaches nothing",
)];

/// The clone request carries no property, which is the delegation the design
/// forbids this account,
/// in every file the split made of the route.
#[test]
fn the_clone_request_sends_no_property() {
    let sources: Vec<(String, String)> =
        release_sources_under(THE_ADD_ZFS_MODULE).expect("the crate's release sources");
    let abandoned: Vec<String> =
        anchors_the_scope_no_longer_holds(sources.as_slice(), SCOPE_ANCHORS);
    assert!(
        abandoned.is_empty(),
        "the scope reached no file holding what this ban is about:\n{}",
        abandoned.join("\n")
    );
    let holding = lines_holding_one_of(sources.as_slice(), THE_PROPERTY_THAT_IS_NOT_SENT);
    let unexplained: Vec<String> = lines_no_entry_excuses(holding.as_slice(), MAY_HOLD);
    assert!(
        unexplained.is_empty(),
        "no `mountpoint` property is written:\n{}",
        unexplained.join("\n")
    );
    let requests: Vec<HoldingLine> = lines_holding_one_of(sources.as_slice(), &[THE_CLONE_REQUEST]);
    assert_eq!(
        requests.len(),
        1,
        "the clone request is made once, and this scope holds {} of them: {}",
        requests.len(),
        requests
            .iter()
            .map(|held| return format!("{}:{}", held.file, held.number))
            .collect::<Vec<String>>()
            .join(", ")
    );
    let request: &str = &requests[0].line[requests[0]
        .line
        .find(THE_CLONE_REQUEST)
        .expect("the line holds the call it was found by")..];
    assert!(
        request.starts_with(format!("{THE_CLONE_REQUEST}{THE_LIST_IT_CARRIES}").as_str()),
        "the request passes no property list: {request}"
    );
}

/// The scan flags a property list a split carried into a receiver,
/// and reads the one request as no finding.
#[test]
fn the_scan_flags_a_property_in_a_receiver_file() {
    let planted: Vec<(String, String)> = vec![(
        String::from("worktree_clone_add_route/add_zfs_planted.rs"),
        String::from(
            "fn planted(list: &mut NvList) { list.add_str(\"mountpoint\", \"/tank/clones\"); }\n",
        ),
    )];
    let holding = lines_holding_one_of(planted.as_slice(), THE_PROPERTY_THAT_IS_NOT_SENT);
    assert!(
        !lines_no_entry_excuses(holding.as_slice(), MAY_HOLD).is_empty(),
        "a property list in a file the route's split made is a finding"
    );
    let request: Vec<(String, String)> = vec![(
        String::from("worktree_clone_add_route/add_zfs.rs"),
        String::from("fn made() { let _clone = self.device.clone(target, origin, None); }\n"),
    )];
    assert!(
        lines_holding_one_of(request.as_slice(), THE_PROPERTY_THAT_IS_NOT_SENT).is_empty(),
        "the request the route makes is no property list"
    );
}
