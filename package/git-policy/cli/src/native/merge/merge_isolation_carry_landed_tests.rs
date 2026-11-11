//! Controls for the record of what Git's own update left:
//! the round trip of the entries a merge judges its paths against,
//! the words a record is made of,
//! and what a record that cannot be read proves.

use super::{LANDED_RECORD_SCHEMA_VERSION, LandedFile, encode_landed_record, parse_landed_record};
use crate::merge_isolation_changes::FileIdentity;

/// One entry a merge wrote,
/// as the command that moved the checkout read it.
fn landed(path: &[u8], identity: Option<FileIdentity>) -> LandedFile {
    return LandedFile { path: path.to_vec(), identity, changed_seconds: 1_760_000_000, changed_nanos: 123_456_789 };
}

/// The identity of one file,
/// with every part a rename or a write changes.
fn an_identity() -> FileIdentity {
    return FileIdentity { device: 20_481, inode: 9_007_199_254_740_993, size: 12, mode: 33_188, modified_seconds: 1_760_000_000, modified_nanos: 5 };
}

/// What:
///  A record comes back with every entry whole,
///  including a path that is not text and an entry nothing stood at.
/// Why:
///   The recovery of a killed merge judges its paths against exactly these entries,
/// so a part lost or rounded in the
///  record is a path compared with something Git never wrote.
///  An inode beyond a double's exact range is kept as text for that reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parseLandedRecord(encodeLandedRecord('id', entries))).toEqual(entries);
/// ```
#[test]
fn a_record_comes_back_with_every_entry_whole() {
    let entries: Vec<LandedFile> = vec![
        landed(b"a.txt", Some(an_identity())),
        landed(b"dir/a name with \xff bytes", None),
    ];
    let text: String = encode_landed_record("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10", entries.as_slice());
    assert!(text.contains(&format!("\"schemaVersion\":{LANDED_RECORD_SCHEMA_VERSION}")) || text.starts_with("{\"schemaVersion\":"), "the record names its version: {text}");
    assert_eq!(parse_landed_record(text.as_bytes()), Some(entries), "every entry comes back whole");
}

/// What:
///  A record that is not one proves nothing,
///  and neither does a field that is not a number.
/// Why:
///   A reading that cannot be read leaves the recovery reading by bytes,
/// which is what a command with no
///  reading has always done;
/// a reading half understood would judge paths against numbers Git never
///  wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parseLandedRecord(Buffer.from('{not a record}'))).toBeUndefined();
/// ```
#[test]
fn a_record_that_is_not_one_proves_nothing() {
    let good: String = encode_landed_record("id", &[landed(b"a.txt", Some(an_identity()))]);
    assert!(parse_landed_record(good.as_bytes()).is_some(), "the control's own record is one");
    for broken in [
        String::from(""),
        String::from("{not a record}"),
        good.replace("\"merge-landed\"", "\"something-else\""),
        good.replace("\"schemaVersion\":1", "\"schemaVersion\":2"),
        good.replace("\"size\":\"12\"", "\"size\":\"twelve\""),
        good.replace("\"inode\":", "\"inode\":\"x\",\"gone\":"),
        good.replace("\"holds\":\"file\"", "\"holds\":\"half\""),
        good.replace("\"pathHex\":\"612e747874\"", "\"pathHex\":\"612e74787\""),
    ] {
        assert!(parse_landed_record(broken.as_bytes()).is_none(), "{broken:?} proves nothing");
    }
}

/// What:
///  A record of no entries is one,
///  and names nothing.
/// Why:
///   A merge whose paths Git wrote none of still moved the checkout,
/// and a reading of nothing must not be mistaken for
///  a reading that could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parseLandedRecord(encodeLandedRecord('id', []))).toEqual([]);
/// ```
#[test]
fn a_record_of_no_entries_names_nothing() {
    let text: String = encode_landed_record("id", &[]);
    assert_eq!(parse_landed_record(text.as_bytes()), Some(Vec::new()), "an empty reading is a reading");
}
