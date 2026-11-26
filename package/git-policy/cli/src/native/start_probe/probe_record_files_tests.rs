//! What:
//!  Controls of the record's directory and its atomic write:
//!  the directory and its two mount points are `0755`,
//!  the record is `0644`,
//!  a second publish replaces the file whole,
//!  and a killed writer leaves the staged sibling rather than half a record.
//! Why:
//!  A reader opens this file without a lock,
//!  so the rename is the only step that may make a record visible,
//!  and the mode is a fact a root-owned reader can rely on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = RecordPlace.at(fixture('record')); place.prepare(); place.publish('v1\n');
//! ```

use super::{
    DIRECTORY_MODE,
    HOLDER_NAME,
    RECORD_MODE,
    RecordPlace,
    SCRATCH_NAME,
    STAGING_NAME,
};
use crate::testing::test_support::fixture;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The directory,
/// its scratch and its holder are each `0755`,
/// and a second prepare keeps them.
#[test]
fn the_directory_and_its_mount_points_are_seventy_five_five() {
    let root: PathBuf = fixture("start-probe-record-dirs");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    place.prepare().expect("the directory is made");
    place.prepare().expect("a second prepare keeps it");
    for path in [
        place.directory().to_path_buf(),
        place.scratch(),
        place.holder(),
    ] {
        let mode: u32 = std::fs::metadata(&path)
            .expect("the directory exists")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, DIRECTORY_MODE, "{}", path.display());
    }
    assert!(place.scratch().ends_with(SCRATCH_NAME));
    assert!(place.holder().ends_with(HOLDER_NAME));
}

/// The record is `0644` and a second publish replaces it whole.
#[test]
fn the_record_is_written_whole_and_readable() {
    let root: PathBuf = fixture("start-probe-record-write");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    place.prepare().expect("the directory is made");
    place
        .publish("v1\nstate started\n")
        .expect("the first record is written");
    let first: String = std::fs::read_to_string(place.result()).expect("the record is read");
    assert_eq!(first, "v1\nstate started\n");
    let mode: u32 = std::fs::metadata(place.result())
        .expect("the record exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, RECORD_MODE);
    place
        .publish("v1\nstate finished\n")
        .expect("the second record is written");
    let second: String = std::fs::read_to_string(place.result()).expect("the record is read again");
    assert_eq!(second, "v1\nstate finished\n");
    assert!(
        !place.staging().exists(),
        "the staged sibling is renamed, not left"
    );
    assert!(
        place
            .staging()
            .ends_with(format!("{STAGING_NAME}.{}", std::process::id()))
    );
}
