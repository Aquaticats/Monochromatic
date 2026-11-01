//! What: Controls for the tree-record parser.
//! Why: A record misread would compare a manifest with the wrong `HEAD` bytes, or admit a
//!      file a direct fix did not select although `HEAD` holds something else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseTreeRecords(Buffer.from('100644 blob <oid>\ta\0'))).toEqual([{ mode: 'regular', ... }]);
//! ```

/// Import the parser under test and its record.
use super::{TreeRecord, parse_tree_records};
/// The failure causes the parser reports.
use crate::candidate_error::{CandidateError, CandidateFailure};
/// The modes and object names records carry.
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};

/// Forty copies of one hexadecimal digit, as text.
fn hex(digit: char) -> String {
    return std::iter::repeat_n(digit, 40).collect();
}

/// The object named by forty copies of one hexadecimal digit.
fn object(digit: char) -> ObjectId {
    return parse_object_id(hex(digit).as_bytes()).expect("forty hexadecimal digits");
}

/// The failure a parse returned, which the control requires.
fn failure(result: Result<Vec<TreeRecord>, CandidateError>) -> CandidateError {
    match result {
        Ok(records) => panic!("expected a failure, parsed {records:?}"),
        Err(error) => return error,
    }
}

/// Every mode, its type, tabs and spaces inside a pathname, and listing order are kept.
#[test]
fn parses_every_file_mode_in_listing_order() {
    let listing: String = format!(
        "100644 blob {}\ta b\x00100755 blob {}\tbin/run\x00120000 blob {}\tlink\x00160000 commit {}\tsub\tmodule\x00",
        hex('a'),
        hex('b'),
        hex('c'),
        hex('d')
    );
    assert_eq!(
        parse_tree_records(listing.as_bytes()).expect("listing"),
        vec![
            TreeRecord {
                mode: CandidateMode::Regular,
                object: object('a'),
                path: b"a b".to_vec(),
            },
            TreeRecord {
                mode: CandidateMode::Executable,
                object: object('b'),
                path: b"bin/run".to_vec(),
            },
            TreeRecord {
                mode: CandidateMode::Symlink,
                object: object('c'),
                path: b"link".to_vec(),
            },
            TreeRecord {
                mode: CandidateMode::Gitlink,
                object: object('d'),
                path: b"sub\tmodule".to_vec(),
            },
        ]
    );
}

/// An empty listing is an empty tree, not a failure.
#[test]
fn an_empty_listing_has_no_records() {
    assert_eq!(parse_tree_records(b"").expect("empty"), Vec::new());
}

/// Each malformed shape is refused with its own cause, naming the record's position only.
#[test]
fn refuses_every_malformed_record() {
    let blob: String = hex('a');
    let cases: [(String, CandidateFailure, &str); 9] = [
        (
            format!("100644 blob {blob}\ta"),
            CandidateFailure::ListingMalformed,
            "record 0 is cut short",
        ),
        (
            format!("100644 blob {blob} a\x00"),
            CandidateFailure::ListingMalformed,
            "record 0 has no tab",
        ),
        (
            format!("100644 blob {blob}\t\x00"),
            CandidateFailure::ListingMalformed,
            "record 0 does not have three metadata fields",
        ),
        (
            format!("100644 {blob}\ta\x00"),
            CandidateFailure::ListingMalformed,
            "record 0 does not have three metadata fields",
        ),
        (
            format!("100644 blob {blob} x\ta\x00"),
            CandidateFailure::ListingMalformed,
            "record 0 does not have three metadata fields",
        ),
        (
            format!("100664 blob {blob}\ta\x00"),
            CandidateFailure::UnsupportedMode,
            "record 0 has a file mode other than",
        ),
        (
            format!("100644 blob {blob}\ta\x00160000 blob {blob}\tb\x00"),
            CandidateFailure::ListingMalformed,
            "record 1 has an object type that does not match its mode",
        ),
        (
            format!("100644 commit {blob}\ta\x00"),
            CandidateFailure::ListingMalformed,
            "record 0 has an object type that does not match its mode",
        ),
        (
            String::from("100644 blob abc\ta\x00"),
            CandidateFailure::ListingMalformed,
            "record 0 does not carry a complete object name",
        ),
    ];
    for (listing, cause, detail) in cases {
        let error: CandidateError = failure(parse_tree_records(listing.as_bytes()));
        assert_eq!(error.failure, cause, "{listing:?}");
        assert!(
            error.message.contains(detail),
            "{listing:?}: {}",
            error.message
        );
        assert!(
            error.message.contains("git ls-tree -r HEAD"),
            "{}",
            error.message
        );
    }
}
