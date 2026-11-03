//! The unit tests of the ownership evidence for a file a child may have written.

use std::fs;
use std::time::Duration;

use crate::file_owner::{Claim, claim_output, observe, timed};
use crate::test_scratch::scratch;

/// Longer than the margin the evidence allows before a window starts and the clock tick of a file system, so a later write is later.
const AFTER_THE_WINDOW: Duration = Duration::from_millis(60);

#[test]
fn a_file_the_child_rewrote_inside_its_window_is_the_childs_output() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    fs::write(&path, "before").expect("lock");

    let (_, window) = timed(|| { fs::write(&path, "child output").expect("the child writes"); });

    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Ours("child output".to_string()));
}

#[test]
fn a_file_equal_to_its_snapshot_is_unchanged_and_a_missing_one_is_absent() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    fs::write(&path, "before").expect("lock");
    let (_, window) = timed(|| {});

    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Unchanged);
    fs::remove_file(&path).expect("remove");
    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Absent);
}

/// Review finding (2b 1, 2c 4): another writer publishes after the child returned and before the output is recorded.
/// The file differs from its snapshot, but its change time is later than the window, so it is not the child's output.
#[test]
fn a_file_published_after_the_child_returned_is_another_writers() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    fs::write(&path, "before").expect("lock");

    let (_, window) = timed(|| { fs::write(&path, "child output").expect("the child writes"); });
    std::thread::sleep(AFTER_THE_WINDOW);
    fs::write(&path, "published by another writer").expect("another writer");

    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Foreign);
}

/// The same for a file renamed into place after the child returned: a rename moves the change time even when the source is old.
#[test]
fn a_file_renamed_into_place_after_the_child_returned_is_another_writers() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    let prepared = root.path().join("prepared");
    fs::write(&path, "before").expect("lock");
    fs::write(&prepared, "an editor's save").expect("the editor prepares its file earlier");

    let (_, window) = timed(|| { fs::write(&path, "child output").expect("the child writes"); });
    std::thread::sleep(AFTER_THE_WINDOW);
    fs::rename(&prepared, &path).expect("the editor publishes by rename");

    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Foreign);
}

/// Review finding (2b 5): an unreadable file is an error, never an empty one.
#[test]
fn a_file_that_is_not_utf8_text_is_an_error_not_an_empty_file() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    fs::write(&path, [0xff, 0xfe, 0x00]).expect("binary");
    let (_, window) = timed(|| {});

    assert!(observe(&path).is_err());
    assert!(claim_output(&path, "", window).is_err());
}
