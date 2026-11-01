//! Controls for the `last-pushed` record.
//! The TypeScript corpus is `src/auto-push-record.unit.test.ts`;
//! the encoded line below is the incumbent's `JSON.stringify` output for the same record.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// A valid failed-attempt record, as the corpus's `FAILED_RECORD`.
fn failed_record() -> LastPushedRecord {
    return LastPushedRecord {
        tip: "a".repeat(40),
        outcome: LastPushedOutcome::Failed,
        owner_token: "token-1".to_owned(),
        owner_pid: 4242,
        exit_code: 1,
        output: "error: failed to push some refs\n".to_owned(),
    };
}

/// The serialized failed record with `JSON.stringify` key order and escaping,
/// a record whose text holds quotes, backslashes, a control character, non-ASCII and U+2028.
const INCUMBENT_LINE: &str = "{\"schemaVersion\":1,\"tip\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"outcome\":\"failed\",\"ownerToken\":\"tok\\\"en\\\\x\",\"ownerPid\":4242,\"exitCode\":1,\"output\":\"error: ü \u{2028} \\u0001\\n\"}\n";

/// A complete record parses to its fields.
#[test]
fn accepts_a_complete_record() {
    let text = encode_last_pushed_record(&failed_record());
    assert_eq!(parse_last_pushed_record(&text), Some(failed_record()));
}

/// Every missing or mistyped field is rejected; so is `null`.
#[test]
fn rejects_every_missing_or_mistyped_field() {
    let malformed = [
        r#"{"schemaVersion":2,"tip":"a","outcome":"failed","ownerToken":"t","ownerPid":1,"exitCode":1,"output":""}"#,
        r#"{"schemaVersion":1,"tip":"","outcome":"failed","ownerToken":"t","ownerPid":1,"exitCode":1,"output":""}"#,
        r#"{"schemaVersion":1,"tip":"a","outcome":"maybe","ownerToken":"t","ownerPid":1,"exitCode":1,"output":""}"#,
        r#"{"schemaVersion":1,"tip":"a","outcome":"failed","ownerToken":"","ownerPid":1,"exitCode":1,"output":""}"#,
        r#"{"schemaVersion":1,"tip":"a","outcome":"failed","ownerToken":"t","ownerPid":1.5,"exitCode":1,"output":""}"#,
        r#"{"schemaVersion":1,"tip":"a","outcome":"failed","ownerToken":"t","ownerPid":1,"exitCode":"1","output":""}"#,
        r#"{"schemaVersion":1,"tip":"a","outcome":"failed","ownerToken":"t","ownerPid":1,"exitCode":1,"output":null}"#,
        r#"{"schemaVersion":1,"outcome":"failed","ownerToken":"t","ownerPid":1,"exitCode":1,"output":""}"#,
        "null",
        "[]",
    ];
    for text in malformed {
        assert_eq!(parse_last_pushed_record(text), None, "{text}");
    }
}

/// The encoder writes the incumbent's bytes exactly.
#[test]
fn encoding_matches_the_incumbent_bytes() {
    let record = LastPushedRecord {
        tip: "a".repeat(40),
        outcome: LastPushedOutcome::Failed,
        owner_token: "tok\"en\\x".to_owned(),
        owner_pid: 4242,
        exit_code: 1,
        output: "error: ü \u{2028} \u{1}\n".to_owned(),
    };
    assert_eq!(encode_last_pushed_record(&record), INCUMBENT_LINE);
    assert_eq!(parse_last_pushed_record(INCUMBENT_LINE), Some(record));
}

/// A record the incumbent wrote with an integral `ownerPid` written as `1.0` is still valid.
#[test]
fn integral_numbers_written_with_a_fraction_are_accepted() {
    let text = r#"{"schemaVersion":1.0,"tip":"a","outcome":"pushed","ownerToken":"t","ownerPid":1.0,"exitCode":0,"output":""}"#;
    assert!(parse_last_pushed_record(text).is_some());
}

/// An absent record reads as absent.
#[test]
fn an_absent_record_reads_as_absent() {
    let root = fixture("record-absent");
    let read = read_last_pushed_record(&root.join("missing.json")).expect("reads");
    assert_eq!(read, LastPushedRead::Absent);
    remove(&root);
}

/// A truncated record reads as absent instead of blocking every later push.
#[test]
fn an_unreadable_record_reads_as_absent() {
    let root = fixture("record-truncated");
    let path = root.join("record.json");
    std::fs::write(&path, "{\"schemaVersion\":1,").expect("write");
    assert_eq!(
        read_last_pushed_record(&path).expect("reads"),
        LastPushedRead::Absent
    );
    remove(&root);
}

/// A record path that is a directory is an I/O failure, not an absence.
#[test]
fn a_read_failure_other_than_absence_is_an_error() {
    let root = fixture("record-directory");
    let path = root.join("record.json");
    std::fs::create_dir(&path).expect("mkdir");
    assert!(read_last_pushed_record(&path).is_err());
    remove(&root);
}

/// A publication replaces the previous record, leaves no candidate, and is mode 0600.
#[test]
fn a_write_replaces_the_record_by_rename_in_mode_0600() {
    let root = fixture("record-write");
    let path = root.join("record.json");
    let mut other = failed_record();
    other.tip = "b".repeat(40);
    write_last_pushed_record(&path, &other).expect("first write");
    write_last_pushed_record(&path, &failed_record()).expect("second write");
    assert_eq!(
        read_last_pushed_record(&path).expect("reads"),
        LastPushedRead::Record(failed_record())
    );
    let names: Vec<String> = std::fs::read_dir(&root)
        .expect("list")
        .map(|entry| {
            return entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
        })
        .collect();
    assert_eq!(names, vec!["record.json".to_owned()]);
    let mode = std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    remove(&root);
}

/// A failed publication removes its candidate and returns the error.
#[test]
fn a_failed_publication_removes_its_candidate_and_reports() {
    let root = fixture("record-write-failure");
    let path = root.join("record.json");
    std::fs::create_dir(&path).expect("mkdir");
    std::fs::write(path.join("occupant"), "").expect("occupant");
    assert!(write_last_pushed_record(&path, &failed_record()).is_err());
    let names: Vec<String> = std::fs::read_dir(&root)
        .expect("list")
        .map(|entry| {
            return entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
        })
        .collect();
    assert_eq!(names, vec!["record.json".to_owned()]);
    remove(&root);
}

/// A record that an earlier release wrote with a `publishedTip` field still parses, and the field is dropped on the next
/// write, so a downgrade or an upgrade never turns a record into "no attempt".
#[test]
fn a_record_with_a_field_this_release_does_not_know_still_parses() {
    let text = r#"{"schemaVersion":1,"tip":"a","outcome":"failed","ownerToken":"t","ownerPid":1,"exitCode":1,"output":"x","publishedTip":"b"}"#;
    let record = parse_last_pushed_record(text).expect("a record with an unknown field parses");
    assert_eq!(record.tip, "a");
    assert!(!encode_last_pushed_record(&record).contains("publishedTip"));
}

/// Each write names its candidate with a fresh suffix, so a candidate another writer left behind never blocks it.
#[test]
fn a_write_is_not_blocked_by_candidates_with_a_fixed_or_empty_suffix() {
    let root = fixture("record-write-unique-candidate");
    let path = root.join("record.json");
    let empty_suffix = root.join("record.json..pending");
    let fixed_suffix = root.join("record.json.xyzzy.pending");
    std::fs::write(&empty_suffix, "other writer").expect("empty-suffix occupant");
    std::fs::write(&fixed_suffix, "other writer").expect("fixed-suffix occupant");
    write_last_pushed_record(&path, &failed_record()).expect("write");
    assert_eq!(
        read_last_pushed_record(&path).expect("reads"),
        LastPushedRead::Record(failed_record())
    );
    assert_eq!(
        std::fs::read_to_string(&empty_suffix).expect("occupant stays"),
        "other writer"
    );
    assert_eq!(
        std::fs::read_to_string(&fixed_suffix).expect("occupant stays"),
        "other writer"
    );
    remove(&root);
}
