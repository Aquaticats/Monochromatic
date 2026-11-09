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

/// What:
///  The rule the two fields of a record of lines keep:
/// absent bytes are the text of an earlier release,
/// present bytes are the path,
/// damaged bytes are an error,
/// and neither is nothing.
/// Why:
///   Every line-oriented reader of these records shares it,
/// so a reader that took damage for absence would act on the path the older form spells instead.
#[test]
fn the_two_fields_of_a_record_of_lines_are_read_by_one_rule() {
    let path: PathBuf = path_that_is_not_text();
    let bytes: String = hex_of_path(path.as_path());
    assert_eq!(path_of_the_two_fields(Some(bytes.as_str()), Some("/srv/plain/.git")), Ok(Some(path.clone())), "the bytes win over the plain text");
    assert_eq!(path_of_the_two_fields(None, Some("/srv/old/.git")), Ok(Some(PathBuf::from("/srv/old/.git"))), "an earlier release holds the text");
    for damaged in ["2f6", "2f6g", "2f 6"] {
        assert!(path_of_the_two_fields(Some(damaged), Some("/srv/plain/.git")).is_err(), "{damaged:?} is not a path");
    }
    assert_eq!(path_of_the_two_fields(None, None), Ok(None), "a record with neither field names no path");
}

/// What:
///  The one rule a list of repository paths and a single entry of one keep:
/// the text field is written exactly when the text spells every byte,
/// the bytes field then stands alone,
/// and a reader takes whichever the record holds.
/// Why:
///   A record written by the release before the bytes form keeps the text form byte for byte,
/// so that release still reads it,
/// and a record naming a name that is not text has no text field,
/// so that release refuses it rather than acting on a replacement spelling.
#[test]
fn a_list_and_an_entry_keep_their_bytes_without_losing_the_earlier_release() {
    let text_paths: Vec<Vec<u8>> = vec![b"cache".to_vec(), b"cache/a".to_vec()];
    assert_eq!(list_fragment("intended", text_paths.as_slice()), r#""intended":["cache","cache/a"]"#, "the text form is unchanged for names that are text");
    let mixed: Vec<Vec<u8>> = vec![b"cache".to_vec(), b"cache/a\xff".to_vec()];
    assert_eq!(
        list_fragment("intended", mixed.as_slice()),
        r#""intendedHex":["6361636865","63616368652f61ff"]"#,
        "one name that is not text moves the whole list to its bytes"
    );
    let written: Value = serde_json::from_str(r#"{"intendedHex":["6361636865","63616368652f61ff"]}"#).expect("record");
    assert_eq!(read_path_list(&written, "intended"), Some(mixed.clone()), "the bytes are read back");
    let earlier: Value = serde_json::from_str(r#"{"intended":["cache","cache/a"]}"#).expect("record");
    assert_eq!(read_path_list(&earlier, "intended"), Some(text_paths.clone()), "the text of an earlier release is read as it is");
    let damaged: Value = serde_json::from_str(r#"{"intendedHex":["6361636865","63616368652f61f"]}"#).expect("record");
    assert_eq!(read_path_list(&damaged, "intended"), None, "damaged bytes name no path");
    let neither: Value = serde_json::from_str(r#"{"other":1}"#).expect("record");
    assert_eq!(read_path_list(&neither, "intended"), None, "a record with no such field names no list");

    let entry: Vec<u8> = b"cache/a\xff".to_vec();
    assert_eq!(field_fragment("relativePath", entry.as_slice()), r#""relativePathHex":"63616368652f61ff""#);
    assert_eq!(field_fragment("relativePath", b"cache/a"), r#""relativePath":"cache/a""#);
    let entry_record: Value = serde_json::from_str(r#"{"relativePathHex":"63616368652f61ff"}"#).expect("record");
    assert_eq!(read_path_field(&entry_record, "relativePath"), Some(entry));
    let earlier_entry: Value = serde_json::from_str(r#"{"relativePath":"cache/a"}"#).expect("record");
    assert_eq!(read_path_field(&earlier_entry, "relativePath"), Some(b"cache/a".to_vec()));
    let damaged_entry: Value = serde_json::from_str(r#"{"relativePathHex":"6"}"#).expect("record");
    assert_eq!(read_path_field(&damaged_entry, "relativePath"), None);
}
