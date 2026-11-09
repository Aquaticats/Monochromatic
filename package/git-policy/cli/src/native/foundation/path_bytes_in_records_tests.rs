//! Controls for a path kept as text in a record.

use super::*;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

/// A record of `path` under `key`,
/// as `write_field` makes one.
fn record_of(key: &str, path: &Path) -> Map<String, Value> {
    let mut fields: Map<String, Value> = Map::new();
    write_field(&mut fields, key, path);
    return fields;
}

/// A path whose name holds a byte that is not UTF-8,
/// so no text spells it.
fn path_that_is_not_text() -> PathBuf {
    return PathBuf::from("/srv").join(OsStr::from_bytes(b"reposi\xfftory"));
}

/// A path that is text keeps every byte through the hexadecimal field,
/// and the plain field spells it exactly too,
/// so a reader of either form finds the same directory.
#[test]
fn a_path_that_is_text_round_trips_through_both_fields() {
    let path: PathBuf = PathBuf::from("/srv/a repository/.git");
    let fields: Map<String, Value> = record_of("gitDir", path.as_path());
    let value: Value = Value::Object(fields);
    assert_eq!(value.get("gitDir").and_then(Value::as_str), Some("/srv/a repository/.git"), "a plain reader sees the path");
    assert_eq!(read_field(&value, "gitDir"), Ok(path));
}

/// A path whose bytes are not text keeps every byte:
/// the hexadecimal field is the whole path,
/// and the reader that prefers it finds the directory the writer named.
#[test]
fn a_path_that_is_not_utf8_round_trips() {
    let path: PathBuf = path_that_is_not_text();
    let fields: Map<String, Value> = record_of("gitDir", path.as_path());
    assert_eq!(fields.get("gitDirHex").and_then(Value::as_str), Some("2f7372762f7265706f7369ff746f7279"), "the bytes, in hexadecimal");
    assert_eq!(read_field(&Value::Object(fields), "gitDir"), Ok(path));
}

/// A record written before the hexadecimal field existed holds the path as plain text,
/// and that is still what it is read as.
#[test]
fn a_record_of_plain_text_is_still_read() {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("gitDir"), Value::from("/srv/old/.git"));
    assert_eq!(read_field(&Value::Object(fields), "gitDir"), Ok(PathBuf::from("/srv/old/.git")));
}

/// A field that says it holds bytes but holds something else is refused,
/// never taken as plain text:
/// a record whose path is damaged names no directory.
#[test]
fn a_hexadecimal_field_that_is_not_bytes_is_refused() {
    for damaged in ["2f6", "2f6g", "2f 6"] {
        let mut fields: Map<String, Value> = Map::new();
        fields.insert(String::from("gitDir"), Value::from("/srv/plain/.git"));
        fields.insert(String::from("gitDirHex"), Value::from(damaged));
        assert!(read_field(&Value::Object(fields), "gitDir").is_err(), "{damaged:?} is not a path");
    }
    let empty: Map<String, Value> = Map::new();
    assert_eq!(read_field(&Value::Object(empty), "gitDir"), Err(String::from("the record has no gitDir field")));
}
