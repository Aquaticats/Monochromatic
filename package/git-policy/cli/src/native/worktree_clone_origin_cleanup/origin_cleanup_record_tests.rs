//! What:
//!  The controls of the record's bytes:
//!  an owed and a blocked record round trip,
//!  a refused errno is kept,
//!  and each state refuses the lines the other needs.
//! Why:
//!  The writer and the reader are two files,
//!  so a round trip through both is the only proof they agree,
//!  and the state's own lines are the record's whole consistency rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseTheRecord(record.encode())).toEqual(record);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::{
    OWN,
    an_owed_record,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_parse::parse_the_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusalCause;
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;

#[test]
fn an_owed_record_round_trips() {
    let record: OriginCleanupRecord = an_owed_record();
    let bytes: Vec<u8> = record.encode().expect("the record encodes");
    let read: OriginCleanupRecord = parse_the_record(bytes.as_slice()).expect("the record parses");
    assert_eq!(read, record);
    assert_eq!(
        read.pool().as_deref(),
        Ok("tank"),
        "the pool is the origin dataset's first component"
    );
    assert_eq!(
        read.full_snapshot_name(),
        format!("tank/mono-checkout@{OWN}")
    );
}

#[test]
fn a_blocked_record_round_trips_with_its_reason_and_counts() {
    let owed: OriginCleanupRecord = an_owed_record();
    let blocked: OriginCleanupRecord = owed.blocked(
        CleanupBlock {
            reason: BlockReason::Dependency,
            errno: 0,
            clones: 1,
            holds: 0,
        },
        "2026-11-25T09:15:01-05:00",
    );
    let bytes: Vec<u8> = blocked.encode().expect("the record encodes");
    let text: String = String::from_utf8(bytes.clone()).expect("the record is text");
    assert!(text.contains("state=blocked"));
    assert!(text.contains("blockedReason=dependency"));
    assert!(text.contains("numClones=1"));
    assert!(text.contains("userRefs=0"));
    let read: OriginCleanupRecord = parse_the_record(bytes.as_slice()).expect("the record parses");
    assert_eq!(read, blocked);
}

#[test]
fn a_refused_records_errno_is_kept() {
    let blocked: OriginCleanupRecord = an_owed_record().blocked(
        CleanupBlock {
            reason: BlockReason::Refused,
            errno: 16,
            clones: 0,
            holds: 0,
        },
        "2026-11-25T09:15:01-05:00",
    );
    let read: OriginCleanupRecord =
        parse_the_record(blocked.encode().expect("the record encodes").as_slice())
            .expect("the record parses");
    assert_eq!(
        read.block.map(|held| return held.errno),
        Some(16),
        "an administrator reads the errno of a permission refusal"
    );
}

#[test]
fn a_blocked_record_without_its_counts_refuses() {
    let owed: OriginCleanupRecord = an_owed_record();
    let mut blocked: OriginCleanupRecord = owed.blocked(
        CleanupBlock {
            reason: BlockReason::Dependency,
            errno: 0,
            clones: 0,
            holds: 0,
        },
        "2026-11-25T09:15:01-05:00",
    );
    blocked.block = None;
    let refusal = blocked
        .encode()
        .expect_err("a blocked record without its lines is not one this writer writes");
    assert_eq!(refusal.cause, CleanupRefusalCause::UnexpectedKey);
}

#[test]
fn a_snapshot_name_outside_the_grammar_refuses() {
    let mut record: OriginCleanupRecord = an_owed_record();
    record.origin_snapshot = String::from("origin");
    let refusal = record
        .encode()
        .expect_err("the record's key must be the add's own name");
    assert_eq!(refusal.cause, CleanupRefusalCause::NotCanonical);
}
