//! What:
//!  Controls for the handle's record and its plain reads:
//!  the line round-trips,
//!  a longer handle,
//!  a bad digit,
//!  a wrong version and an extra field are each
//!  refused,
//!  and a folder's mount id and facts come from the kernel's own records.
//! Why:
//!  The two calls that take and reopen a handle need a file system that answers
//!  them and are shown in the guest arm,
//!  and the record's grammar is what a handler reads before it makes either
//!  call,
//!  so it is shown here without a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the handle record round-trips', () => { expect(decode(encode(h))).toEqual(h); });
//! ```

use super::{
    HANDLE_RECORD_VERSION,
    HandleRefusal,
    MAXIMUM_HANDLE_BYTES,
    RecordedHandle,
    folder_facts,
    mount_id_of,
};
use crate::testing::test_support::fixture;

#[test]
fn the_handle_record_round_trips() {
    let recorded: RecordedHandle = RecordedHandle {
        handle_type: 1,
        handle: vec![0x00, 0x7f, 0xff, 0x10],
    };
    let line: String = recorded.encode();
    assert_eq!(line, "v1:1:007fff10");
    assert_eq!(
        RecordedHandle::decode(line.as_str()).expect("a line this build wrote is one it reads"),
        recorded
    );
}

#[test]
fn an_empty_handle_round_trips_because_the_kernel_may_answer_one() {
    let recorded: RecordedHandle = RecordedHandle {
        handle_type: 0,
        handle: Vec::new(),
    };
    assert_eq!(
        RecordedHandle::decode(recorded.encode().as_str())
            .expect("an empty handle is written and read"),
        recorded
    );
}

#[test]
fn a_handle_longer_than_the_kernel_s_is_refused_before_any_call_is_made() {
    let too_long: String = "00".repeat(MAXIMUM_HANDLE_BYTES + 1);
    let refused: HandleRefusal =
        RecordedHandle::decode(format!("{HANDLE_RECORD_VERSION}:1:{too_long}").as_str())
            .expect_err("a handle longer than the kernel's is refused");
    assert_eq!(refused.calling, "record");
    assert!(
        refused.said.contains("longer than the kernel"),
        "the refusal says what is wrong, and it read {}",
        refused.said
    );
    let at_the_bound: String = "ab".repeat(MAXIMUM_HANDLE_BYTES);
    assert!(
        RecordedHandle::decode(format!("{HANDLE_RECORD_VERSION}:1:{at_the_bound}").as_str())
            .is_ok(),
        "a handle of the kernel's own bound is read"
    );
}

#[test]
fn each_malformed_line_is_refused_by_what_it_lacks() {
    for (line, wanted) in [
        ("", "empty"),
        ("v1", "no handle type"),
        ("v1:1", "no handle bytes"),
        ("v2:1:00", "version"),
        ("v1:one:00", "type is not a decimal"),
        ("v1:1:0", "pair into bytes"),
        ("v1:1:00zz", "not hexadecimal"),
        ("v1:1:00:extra", "more than three fields"),
    ] {
        let refused: HandleRefusal =
            RecordedHandle::decode(line).expect_err("a malformed line is refused");
        assert!(
            refused.said.contains(wanted),
            "the refusal of `{line}` says `{wanted}`, and it read {}",
            refused.said
        );
    }
}

#[test]
fn a_folder_s_mount_id_and_facts_come_from_the_kernel_s_own_records() {
    let folder: std::path::PathBuf = fixture("recorded_handle_mount_id");
    let opened: std::fs::File =
        std::fs::File::open(folder.as_path()).expect("the fixture folder is opened");
    let mount_id: u64 = mount_id_of(&opened).expect("a descriptor's fdinfo names its mount");
    assert!(
        mount_id > 0,
        "the kernel's mount numbers start above zero, and it read {mount_id}"
    );
    let (device, inode): (u64, u64) =
        folder_facts(&opened).expect("the folder's own facts are read from its descriptor");
    use std::os::unix::fs::MetadataExt;
    let named: std::fs::Metadata =
        std::fs::metadata(folder.as_path()).expect("the folder's metadata");
    assert_eq!(
        (device, inode),
        (named.dev(), named.ino()),
        "the facts are the descriptor's own, not a path resolved again"
    );
}

#[test]
fn a_descriptor_that_is_not_a_folder_still_names_its_mount() {
    let file: std::path::PathBuf = fixture("recorded_handle_plain_file").join("a-file.txt");
    std::fs::write(file.as_path(), b"a file\n").expect("the fixture file");
    let opened: std::fs::File =
        std::fs::File::open(file.as_path()).expect("the fixture file is opened");
    assert!(mount_id_of(&opened).expect("a regular file's fdinfo names its mount") > 0);
}
