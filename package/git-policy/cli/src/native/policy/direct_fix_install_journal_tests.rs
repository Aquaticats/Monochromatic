//! What:
//!  Controls for the record a fix writes before it touches a file:
//!  every field survives the text it is
//!       written as,
//!  a pathname of any bytes survives it,
//!  and a malformed record is refused rather than
//!       half-read.
//! Why:
//!  A later command reads that record to decide whether to put a file back;
//!  a field that decodes
//!       wrongly would make it act on the wrong file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(decodeJournal(encodeJournal(record))).toEqual(record);
//! ```
#![cfg(unix)]

/// Import the record under test.
use super::JournalRecord;
/// One extended attribute the record carries.
use crate::direct_fix_install_attributes::Attribute;
/// `OsStringExt` builds OS text from raw bytes.
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

/// A record of a pathname whose bytes are not UTF-8,
///  with every field filled.
fn record() -> JournalRecord {
    return JournalRecord {
        destination: PathBuf::from(std::ffi::OsString::from_vec(
            b"/r\xff/\xfe lot/..\\name".to_vec(),
        )),
        mode: 0o6755,
        uid: 4_294_967_295,
        gid: 1,
        owner: Some(String::from("1234~1234~pid%3a%5b4026531836%5d")),
        replacement: String::from("0f".repeat(32)),
        attributes: vec![
            Attribute {
                name: b"system.posix_acl_access".to_vec(),
                value: vec![2, 0, 0, 0],
            },
            Attribute {
                name: b"user.draft".to_vec(),
                value: b"only-copy-of-draft".to_vec(),
            },
        ],
    };
}

/// Every field survives the text,
///  including a pathname of raw bytes.
#[test]
fn a_record_survives_its_text() {
    let written: JournalRecord = record();
    let text: String = written.encode();
    assert!(text.starts_with("destination "), "{text}");
    assert!(text.contains("\nmode 6755\n"), "{text}");
    assert!(
        text.contains("\nattribute 757365722e6472616674 6f6e6c792d636f70792d6f662d6472616674\n"),
        "{text}"
    );
    assert_eq!(JournalRecord::decode(text.as_str()), Some(written.clone()));
    // A record whose writer the platform could not name decodes as one that names nobody.
    let mut unnamed: JournalRecord = written;
    unnamed.owner = None;
    assert_eq!(
        JournalRecord::decode(unnamed.encode().as_str()),
        Some(unnamed)
    );
}

/// A malformed record is refused,
///  never half-read.
#[test]
fn a_malformed_record_is_refused() {
    assert_eq!(JournalRecord::decode(""), None);
    assert_eq!(JournalRecord::decode("destination 6a\n"), None);
    assert_eq!(
        JournalRecord::decode("destination 6a\nmode 644\nuid 1\ngid 1\nowner x\n"),
        None
    );
    // An odd number of hexadecimal digits names no pathname.
    assert_eq!(
        JournalRecord::decode(
            "destination 6\nmode 644\nuid 1\ngid 1\nowner x\nreplacement 0f\n"
        ),
        None
    );
}
