//! What:
//!  Controls of the mark's three states:
//!  the property's value decides and its source never does,
//!  an absent or unreadable property is never off,
//!  and a hold count the reply lacks does not become zero.
//! Why:
//!  A reader that required a `source` would refuse every origin, marked or not,
//!  and a reader that fell through to "not marked" would fail open on the one state the check
//!  exists to catch
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`, F4 and veto V3).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(markOfReply(reply(1))).toEqual({ kind: 'marked', clones: 1, holds: { kind: 'counted', count: 0 } });
//! ```

use super::super::fixtures::{
    Answer,
    CLONE,
    Pool,
    pair_number,
    pair_text,
};
use super::super::origin_mark::{
    DEFER_DESTROY_PROPERTY,
    MARK_OFF,
    MARK_ON,
    OriginMark,
    USER_HOLDS_PROPERTY,
    UserHolds,
    mark_of_reply,
    origin_mark,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};

/// The statistics a marked snapshot's reply carries,
/// with this many clones of it.
fn stats(clones: u64) -> DmuObjsetStats {
    return super::super::fixtures::stats(11, true, 7, clones, "");
}

/// One reply holding the mark's entry as a control wrote it,
/// and the hold count when the control asks for one.
fn reply(
    mark: NvList,
    holds: Option<NvList>,
) -> NvList {
    let mut properties: NvList = NvList::new();
    properties.add_nvlist(DEFER_DESTROY_PROPERTY, mark);
    if let Some(entry) = holds {
        properties.add_nvlist(USER_HOLDS_PROPERTY, entry);
    }
    return properties;
}

#[test]
fn a_marked_snapshot_reads_on_with_its_clone_and_hold_counts() {
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(pair_number(MARK_ON, None), Some(pair_number(0, None))),
    };
    assert_eq!(
        mark_of_reply(&identity),
        OriginMark::Marked {
            clones: 1,
            holds: UserHolds::Counted(0)
        }
    );
}

#[test]
fn the_mark_is_read_from_the_value_and_never_from_the_source() {
    let with_source: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(
            pair_number(MARK_ON, Some("zpcachyos/nosnap/checkout")),
            Some(pair_number(0, Some("checkout"))),
        ),
    };
    let without_source: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(pair_number(MARK_ON, None), Some(pair_number(0, None))),
    };
    assert_eq!(
        mark_of_reply(&with_source),
        mark_of_reply(&without_source),
        "a source changes no answer"
    );
    assert_eq!(
        mark_of_reply(&with_source),
        OriginMark::Marked {
            clones: 1,
            holds: UserHolds::Counted(0)
        }
    );
    let off: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(
            pair_number(MARK_OFF, Some("local")),
            Some(pair_number(0, None)),
        ),
    };
    assert_eq!(mark_of_reply(&off), OriginMark::Unmarked);
}

#[test]
fn a_text_value_is_unreadable_rather_than_marked() {
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(pair_text("on"), Some(pair_number(0, None))),
    };
    let mark: OriginMark = mark_of_reply(&identity);
    assert!(!mark.is_marked(), "a text value is no mark at all");
    match mark {
        OriginMark::Unreadable {
            said,
        } => assert!(
            said.contains(DEFER_DESTROY_PROPERTY),
            "the state names the property: {said}"
        ),
        other => panic!("a text value must be unreadable, not {other:?}"),
    }
}

#[test]
fn an_absent_property_is_unreadable_rather_than_off() {
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: NvList::new(),
    };
    match mark_of_reply(&identity) {
        OriginMark::Unreadable {
            said,
        } => assert!(
            said.contains("is not in the reply"),
            "the state says the property is absent: {said}"
        ),
        other => panic!("an absent property must be unreadable, not {other:?}"),
    }
}

#[test]
fn a_value_beyond_the_two_the_kernel_writes_is_unreadable() {
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(pair_number(2, None), Some(pair_number(0, None))),
    };
    match mark_of_reply(&identity) {
        OriginMark::Unreadable {
            said,
        } => assert!(
            said.contains("reads 2"),
            "the state names the value it found: {said}"
        ),
        other => panic!("a value of 2 must be unreadable, not {other:?}"),
    }
}

#[test]
fn an_entry_that_is_not_the_pair_openzfs_writes_is_unreadable() {
    // The entry itself is a number rather than the nested list a property is, which is the shape F4 rules out.
    let mut properties: NvList = NvList::new();
    properties.add_u64(DEFER_DESTROY_PROPERTY, MARK_ON);
    properties.add_nvlist(USER_HOLDS_PROPERTY, pair_number(0, None));
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties,
    };
    match mark_of_reply(&identity) {
        OriginMark::Unreadable {
            said,
        } => assert!(
            said.contains("not the pair"),
            "the state names the shape it found: {said}"
        ),
        other => panic!("a bare number must be unreadable, not {other:?}"),
    }
}

#[test]
fn a_reply_without_a_hold_count_keeps_the_mark_and_says_the_count_was_not_read() {
    let identity: DatasetIdentity = DatasetIdentity {
        stats: stats(1),
        properties: reply(pair_number(MARK_ON, None), None),
    };
    match mark_of_reply(&identity) {
        OriginMark::Marked {
            clones,
            holds,
        } => {
            assert_eq!(clones, 1);
            match holds {
                UserHolds::Unreadable {
                    said,
                } => assert!(
                    said.contains(USER_HOLDS_PROPERTY),
                    "the state names the property: {said}"
                ),
                other => panic!("a missing hold count must stay unreadable, not {other:?}"),
            }
        }
        other => panic!("the mark itself was readable, so it stays marked: {other:?}"),
    }
}

#[test]
fn a_failed_read_of_the_object_is_unreadable_with_the_kernels_words() {
    let pool: Pool = Pool::new(vec![(
        CLONE,
        Answer::Failed(
            String::from("the device answered EACCES"),
            Some(libc::EACCES),
        ),
    )]);
    assert_eq!(
        origin_mark(&pool, CLONE),
        OriginMark::Unreadable {
            said: String::from("the device answered EACCES")
        }
    );
    assert_eq!(
        pool.reads(),
        vec![String::from(CLONE)],
        "one read, of the name asked for"
    );
}

#[test]
fn a_read_of_an_unmarked_object_reads_it_through_the_same_seam() {
    let pool: Pool = super::super::fixtures::unmarked();
    assert_eq!(
        origin_mark(&pool, super::super::fixtures::ORIGIN_SNAPSHOT),
        OriginMark::Unmarked
    );
}

#[test]
fn the_mark_words_its_value_for_a_message() {
    let marked: OriginMark = OriginMark::Marked {
        clones: 2,
        holds: UserHolds::Counted(1),
    };
    let unmarked: OriginMark = OriginMark::Unmarked;
    let unreadable: OriginMark = OriginMark::Unreadable {
        said: String::from("nothing"),
    };
    assert!(
        marked.described().contains(&format!("reads {MARK_ON}")),
        "a marked mark names its value: {}",
        marked.described()
    );
    assert!(
        unmarked.described().contains(&format!("reads {MARK_OFF}")),
        "an unmarked mark names its value: {}",
        unmarked.described()
    );
    assert!(
        unreadable.described().contains("could not be read"),
        "an unreadable mark says so: {}",
        unreadable.described()
    );
    assert!(marked.is_marked() && !unmarked.is_marked() && !unreadable.is_marked());
}
