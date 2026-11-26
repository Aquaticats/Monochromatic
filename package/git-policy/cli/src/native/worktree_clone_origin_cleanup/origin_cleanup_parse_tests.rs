//! What:
//!  The controls of the record's parse:
//!  a round trip,
//!  and one control for each malformed form the grammar refuses.
//! Why:
//!  A record that does not read is the only evidence of what a cleanup owed,
//!  so the reader refuses rather than repairs,
//!  and a control that only read a good record would never show that.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => parse(bytesWithARepeatedKey)).toThrow();
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::{
    an_owed_record,
    the_bytes,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_parse::parse_the_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusalCause;

#[test]
fn a_repeated_key_refuses() {
    let mut text: String = the_bytes();
    text.push_str("state=owed\n");
    let refusal = parse_the_record(text.as_bytes()).expect_err("one key, one line");
    assert_eq!(refusal.cause, CleanupRefusalCause::RepeatedKey);
}

#[test]
fn a_key_the_grammar_does_not_hold_refuses() {
    let mut text: String = the_bytes();
    text.push_str("freeText=anything\n");
    let refusal = parse_the_record(text.as_bytes()).expect_err("no free-text field exists");
    assert_eq!(refusal.cause, CleanupRefusalCause::UnknownKey);
}

#[test]
fn a_key_out_of_order_refuses() {
    let mut lines: Vec<String> = the_bytes().lines().map(String::from).collect();
    let at: usize = lines
        .iter()
        .position(|line| return line.starts_with("startedAt="))
        .expect("the record carries it");
    let later: usize = lines
        .iter()
        .position(|line| return line.starts_with("originDatasetGuid="))
        .expect("the record carries it");
    lines.swap(at, later);
    let mut text: String = lines.join("\n");
    text.push('\n');
    let refusal = parse_the_record(text.as_bytes()).expect_err("the order is the grammar's");
    assert_eq!(refusal.cause, CleanupRefusalCause::OutOfOrder);
}

#[test]
fn a_missing_key_refuses() {
    let text: String = the_bytes().replace("originSnapshotGuid=", "originSnapshotGuidExtra=");
    let refusal = parse_the_record(text.as_bytes()).expect_err("the identity lines are required");
    assert!(matches!(
        refusal.cause,
        CleanupRefusalCause::MissingKey | CleanupRefusalCause::UnknownKey
    ));
}

#[test]
fn a_record_that_stops_before_its_last_line_refuses() {
    let text: String = the_bytes();
    let cut: &str = &text[..text.rfind('\n').expect("the record has lines")];
    let refusal = parse_the_record(cut.as_bytes())
        .expect_err("a record cut short is not one this reader reads");
    assert_eq!(refusal.cause, CleanupRefusalCause::EndsEarly);
}

#[test]
fn a_non_canonical_guid_refuses() {
    let text: String =
        the_bytes().replace("originSnapshotCreatetxg=15", "originSnapshotCreatetxg=015");
    let refusal =
        parse_the_record(text.as_bytes()).expect_err("a leading zero is not canonical decimal");
    assert_eq!(refusal.cause, CleanupRefusalCause::NotCanonical);
}

#[test]
fn an_owed_record_with_a_block_reason_refuses() {
    let mut text: String = the_bytes();
    text = text.replace("state=owed", "state=blocked");
    let refusal =
        parse_the_record(text.as_bytes()).expect_err("a blocked record needs every state line");
    assert!(matches!(
        refusal.cause,
        CleanupRefusalCause::MissingKey | CleanupRefusalCause::UnexpectedKey
    ));
}

#[test]
fn a_record_over_its_bound_refuses_before_any_line() {
    let mut text: String = the_bytes();
    text.push_str(&"x".repeat(9_000));
    let refusal = parse_the_record(text.as_bytes()).expect_err("the bound is read before a line");
    assert_eq!(refusal.cause, CleanupRefusalCause::Oversized);
}

#[test]
fn a_record_without_its_clone_guid_reads_as_the_add_that_made_no_clone() {
    let mut without: OriginCleanupRecord = an_owed_record();
    without.clone_guid = None;
    let bytes: Vec<u8> = without.encode().expect("a record without a clone encodes");
    let read: OriginCleanupRecord = parse_the_record(bytes.as_slice()).expect("and reads back");
    assert_eq!(read.clone_guid, None);
}
