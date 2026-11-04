//! Controls for the blob names the content decision compares with.

use super::*;

/// A blob is named by its framed bytes: the empty blob and a short one have the names Git gives them.
#[test]
fn a_blob_is_named_as_git_names_it() {
    let hex = |name: Vec<u8>| -> String { return name.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect(); };
    assert_eq!(hex(blob_name(b"", ObjectFormat::Sha1)), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    assert_eq!(hex(blob_name(b"hello\n", ObjectFormat::Sha1)), "ce013625030ba8dba906f756967f9e9ca394464a");
    assert_eq!(blob_name(b"hello\n", ObjectFormat::Sha256).len(), 32);
}

/// `GIT_ATTR_NOSYSTEM` is read as a Git boolean: unset and empty are false, a boolean word is itself, anything else is unknown.
#[test]
fn the_system_switch_is_read_as_git_reads_a_boolean() {
    let with = |value: &str| -> Vec<(OsString, OsString)> { return vec![(OsString::from("GIT_ATTR_NOSYSTEM"), OsString::from(value))]; };
    assert_eq!(system_attributes_from(with("1").as_slice()), SystemAttributes::Disabled);
    assert_eq!(system_attributes_from(with("true").as_slice()), SystemAttributes::Disabled);
    assert_eq!(system_attributes_from(with("maybe").as_slice()), SystemAttributes::Unknown);
    // Without a forward target the executable cannot be asked, so a switch that is off leaves the location unknown.
    assert_eq!(system_attributes_from(with("no").as_slice()), SystemAttributes::Unknown);
    assert_eq!(system_attributes_from(with("").as_slice()), SystemAttributes::Unknown);
    assert_eq!(system_attributes_from(&[]), SystemAttributes::Unknown);
}
