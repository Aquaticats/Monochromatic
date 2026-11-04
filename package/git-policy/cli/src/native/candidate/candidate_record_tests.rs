//! What:
//!  Controls for raw `-z` changed-path record parsing.
//! Why:
//!  The record list decides which paths are checked at all,
//!  so a misread or
//!      partially read listing must fail instead of silently shrinking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseRawRecords(Buffer.from(':000000 100644 0..0 a..a A\0f\0'))).toEqual([{ path: 'f', ... }]);
//! ```

/// Import the parser under test and the values it produces.
use super::{CandidateChange, CandidateRecord, parse_raw_records};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};

/// Git's all-zero placeholder for "no object on this side".
const ZERO: &str = "0000000000000000000000000000000000000000";

/// A blob name.
const BLOB: &str = "5626abf0f72e58d7a153368ba57db4c673c0e171";

/// Another blob name.
const OTHER: &str = "646377bf16c0f4b10249686e46f203a6cb55bfa5";

/// The validated form of a fixture name.
fn object(text: &str) -> ObjectId {
    return parse_object_id(text.as_bytes()).expect("fixture name");
}

/// Build one raw record with a NUL after the metadata and after the path.
fn raw(metadata: &str, path: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = metadata.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend_from_slice(path);
    bytes.push(0);
    return bytes;
}

/// Parse output that must be refused and return the failure.
fn failure_of(output: &[u8]) -> CandidateError {
    let error: CandidateError = parse_raw_records(output).expect_err("refused listing");
    assert!(
        error
            .message
            .starts_with("cli-git could not list candidate files: changed-path record "),
        "{error}"
    );
    return error;
}

/// No output is no records.
#[test]
fn empty_output_lists_nothing() {
    assert_eq!(parse_raw_records(b"").expect("empty"), Vec::new());
}

/// Added,
///  modified,
///  type-changed and deleted entries carry the right side's mode and object,
///  in Git's order.
#[test]
fn each_status_reads_the_side_it_describes() {
    let mut output: Vec<u8> = raw(&format!(":000000 100644 {ZERO} {BLOB} A"), b"new.txt");
    output.extend(raw(
        &format!(":100644 100755 {BLOB} {OTHER} M"),
        b"dir/changed.sh",
    ));
    output.extend(raw(
        &format!(":100644 120000 {BLOB} {OTHER} T"),
        b"now-a-link",
    ));
    output.extend(raw(&format!(":100755 000000 {BLOB} {ZERO} D"), b"gone"));
    output.extend(raw(&format!(":000000 160000 {ZERO} {OTHER} A"), b"sub"));
    assert_eq!(
        parse_raw_records(output.as_slice()).expect("records"),
        vec![
            CandidateRecord {
                path: b"new.txt".to_vec(),
                mode: CandidateMode::Regular,
                change: CandidateChange::Added,
                object: Some(object(BLOB)),
            },
            CandidateRecord {
                path: b"dir/changed.sh".to_vec(),
                mode: CandidateMode::Executable,
                change: CandidateChange::Modified,
                object: Some(object(OTHER)),
            },
            CandidateRecord {
                path: b"now-a-link".to_vec(),
                mode: CandidateMode::Symlink,
                change: CandidateChange::Modified,
                object: Some(object(OTHER)),
            },
            CandidateRecord {
                path: b"gone".to_vec(),
                mode: CandidateMode::Executable,
                change: CandidateChange::Deleted,
                object: None,
            },
            CandidateRecord {
                path: b"sub".to_vec(),
                mode: CandidateMode::Gitlink,
                change: CandidateChange::Added,
                object: Some(object(OTHER)),
            },
        ]
    );
}

/// Pathname bytes are kept exactly:
///  non-UTF-8 bytes,
///  spaces,
///  tabs,
///  line feeds,
///  colons and quotes.
#[test]
fn pathname_bytes_are_never_decoded() {
    let path: &[u8] = b"d\xff\xfe/ sp\tace\nline:colon\"quote\\";
    let output: Vec<u8> = raw(&format!(":000000 100644 {ZERO} {BLOB} A"), path);
    let records: Vec<CandidateRecord> = parse_raw_records(output.as_slice()).expect("record");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].path, path);
}

/// SHA-256 repositories print 64-digit names,
///  which are carried the same way.
#[test]
fn sha256_names_are_accepted() {
    let zero: String = "0".repeat(64);
    let name: String =
        "0b3131207073aeabcc6ae5b5d57a4875e562bf4ef3b757adb6b75defb2ec9df1".to_owned();
    let output: Vec<u8> = raw(&format!(":000000 100644 {zero} {name} A"), b"q");
    let records: Vec<CandidateRecord> = parse_raw_records(output.as_slice()).expect("record");
    assert_eq!(records[0].object, Some(object(name.as_str())));
}

/// A conflicted index entry is its own failure,
///  naming the remedy.
#[test]
fn unmerged_entry_is_refused() {
    let mut output: Vec<u8> = raw(&format!(":000000 100644 {ZERO} {BLOB} A"), b"ok");
    output.extend(raw(
        &format!(":100644 000000 {BLOB} {ZERO} U"),
        b"private-name",
    ));
    let error: CandidateError = failure_of(output.as_slice());
    assert_eq!(error.failure, CandidateFailure::UnmergedPath);
    // The record is named by position, never by pathname.
    assert!(error.message.contains("record 1 is an unmerged"), "{error}");
    assert!(!error.message.contains("private-name"), "{error}");
}

/// Statuses a listing without rename detection never prints are refused,
///  including near misses of accepted ones.
#[test]
fn other_statuses_are_refused() {
    for status in ["R100", "C75", "X", "B", "", "AM", "a", "m", "t", "d", "DD"] {
        let output: Vec<u8> = raw(&format!(":100644 100644 {BLOB} {OTHER} {status}"), b"p");
        let error: CandidateError = failure_of(output.as_slice());
        // An empty status is still the fifth field: the space before it produces an empty piece.
        assert_eq!(
            error.failure,
            CandidateFailure::UnsupportedStatus,
            "{status:?}"
        );
    }
}

/// Directory,
///  absent and unknown modes are refused on the side the status reads.
#[test]
fn unsupported_modes_are_refused() {
    for metadata in [
        format!(":000000 040000 {ZERO} {BLOB} A"),
        format!(":100644 000000 {BLOB} {OTHER} M"),
        format!(":100644 100664 {BLOB} {OTHER} M"),
        // A deletion reads the old mode, so an absent old side is refused even with a valid new one.
        format!(":000000 100644 {BLOB} {ZERO} D"),
        format!(":040000 100644 {BLOB} {ZERO} D"),
    ] {
        let output: Vec<u8> = raw(metadata.as_str(), b"p");
        assert_eq!(
            failure_of(output.as_slice()).failure,
            CandidateFailure::UnsupportedMode,
            "{metadata}"
        );
    }
    // The same deletion with a valid old mode and an unusable new one is accepted.
    let accepted: Vec<u8> = raw(&format!(":100644 000000 {BLOB} {ZERO} D"), b"p");
    assert_eq!(
        parse_raw_records(accepted.as_slice())
            .expect("deleted")
            .len(),
        1
    );
}

/// Records with the wrong shape are refused:
///  prefix,
///  field count,
///  object names,
///  and an empty pathname.
#[test]
fn malformed_records_are_refused() {
    for (metadata, path) in [
        (format!("000000 100644 {ZERO} {BLOB} A"), b"p".as_slice()),
        (format!(":000000 100644 {ZERO} {BLOB}"), b"p"),
        (format!(":000000 100644 {ZERO} {BLOB} A extra"), b"p"),
        (format!(":000000 100644 {ZERO} {BLOB} A"), b""),
        (format!(":000000 100644 {ZERO} {} A", &BLOB[..39]), b"p"),
        (format!(":000000 100644 {} {BLOB} A", &ZERO[..39]), b"p"),
        (
            format!(":000000 100644 {ZERO} {} A", BLOB.to_uppercase()),
            b"p",
        ),
        (format!(":100644 000000 {} {ZERO} D", &BLOB[..39]), b"p"),
        (format!(":100644 000000 {BLOB} {} D", &ZERO[..39]), b"p"),
        (String::from(":"), b"p"),
        (String::new(), b"p"),
    ] {
        let output: Vec<u8> = raw(metadata.as_str(), path);
        assert_eq!(
            failure_of(output.as_slice()).failure,
            CandidateFailure::ListingMalformed,
            "{metadata:?} {path:?}"
        );
    }
}

/// Output cut before its final NUL,
///  or holding a record without a pathname,
///  is refused whole.
#[test]
fn cut_short_listings_are_refused() {
    let complete: Vec<u8> = raw(&format!(":000000 100644 {ZERO} {BLOB} A"), b"first");
    // Every proper prefix that drops the final NUL is refused.
    let without_terminator: &[u8] = &complete[..complete.len() - 1];
    let error: CandidateError = failure_of(without_terminator);
    assert_eq!(error.failure, CandidateFailure::ListingMalformed);
    assert!(
        error.message.contains("does not end with a NUL byte"),
        "{error}"
    );
    // Two complete records and a third metadata token without its pathname.
    let mut odd: Vec<u8> = complete.clone();
    odd.extend(raw(&format!(":000000 100644 {ZERO} {OTHER} A"), b"second"));
    odd.extend_from_slice(format!(":000000 100644 {ZERO} {BLOB} A").as_bytes());
    odd.push(0);
    let odd_error: CandidateError = failure_of(odd.as_slice());
    assert_eq!(odd_error.failure, CandidateFailure::ListingMalformed);
    assert!(
        odd_error.message.contains("record 2 has no pathname token"),
        "{odd_error}"
    );
    // A lone NUL is one empty token, which is a record without a pathname.
    assert_eq!(failure_of(&[0]).failure, CandidateFailure::ListingMalformed);
    // A failure in a later record names that record's position.
    let mut second_bad: Vec<u8> = complete.clone();
    second_bad.extend(raw("not-a-record", b"p"));
    assert!(
        failure_of(second_bad.as_slice())
            .message
            .contains("record 1 does not start with a colon")
    );
}
