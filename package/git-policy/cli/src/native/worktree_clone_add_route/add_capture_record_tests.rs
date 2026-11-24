//! Controls for the capture's durable grammar:
//!  its bytes,
//!  its reading,
//!  and every shape a reader refuses.

use super::{CAPTURE_FIXED_KEYS, CAPTURE_HEADER, capture_bound, encode_capture, parse_capture};
use crate::worktree_clone_add_route::add_capture::{CapturedEntry, CapturedIndex, CapturedKind, Capture};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// One capture with every kind of entry.
fn a_capture() -> Capture {
    return Capture {
        nonce: RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f").expect("the nonce"),
        repository: b"/home/user/Monochromatic/.git".to_vec(),
        source: b"/home/user/Monochromatic".to_vec(),
        checkout_guid: DatasetGuid::parse("4815162342109876543").expect("the guid"),
        index: CapturedIndex { size: 1024, mtime: (1763750400, 123456789), digest: [7; 32] },
        head: String::from("1111111111111111111111111111111111111111"),
        source_head: String::from("1111111111111111111111111111111111111111"),
        entries: vec![
            CapturedEntry { path: b"plain.txt".to_vec(), kind: CapturedKind::Regular, size: 6, mtime: (1763750400, 1), ctime: (1763750400, 2), executable: false, link_target: None },
            CapturedEntry { path: b"run.sh".to_vec(), kind: CapturedKind::Regular, size: 10, mtime: (1763750400, 3), ctime: (1763750400, 4), executable: true, link_target: None },
            CapturedEntry { path: b"link".to_vec(), kind: CapturedKind::Link, size: 9, mtime: (1763750400, 5), ctime: (1763750400, 6), executable: false, link_target: Some(b"plain.txt".to_vec()) },
            CapturedEntry { path: b"gone.txt".to_vec(), kind: CapturedKind::Absent, size: 0, mtime: (0, 0), ctime: (0, 0), executable: false, link_target: None },
        ],
    };
}

#[test]
fn the_bytes_and_the_reading_agree() {
    let capture: Capture = a_capture();
    let bytes: Vec<u8> = encode_capture(&capture);
    let text: String = String::from_utf8(bytes).expect("the record is text");
    assert!(text.starts_with(CAPTURE_HEADER));
    for key in CAPTURE_FIXED_KEYS {
        assert!(text.contains(format!("{key}=").as_str()), "the fixed key {key} is written");
    }
    assert_eq!(parse_capture(text.as_str()).expect("the record reads back"), capture);
    assert_eq!(text.lines().filter(|line| return line.starts_with("entry=")).count(), 4);
}

#[test]
fn a_reader_refuses_every_shape_it_did_not_write() {
    let text: String = String::from_utf8(encode_capture(&a_capture())).expect("the record is text");
    let cases: [(&str, String, &str); 6] = [
        ("a foreign header", text.replace(CAPTURE_HEADER, "# another record"), "this grammar's"),
        ("a missing key", text.replace("indexSize=1024\n", ""), "where `indexSize=` belongs"),
        ("a count one short", text.replace("entries=4", "entries=3"), "holds a line after"),
        ("a count one long", text.replace("entries=4", "entries=5"), "holds fewer entry lines"),
        ("an unknown kind", text.replace("entry=676f6e652e747874 absent", "entry=676f6e652e747874 folder"), "is not `regular`, `link` or `absent`"),
        ("a bad digest", text.replace("indexDigest=070707", "indexDigest=zzzzzz"), "is not lowercase hexadecimal"),
    ];
    for (what, refused, wanted) in cases {
        let refusal: String = parse_capture(refused.as_str()).expect_err(what);
        assert!(refusal.contains(wanted), "{what}: {refusal}");
    }
}

#[test]
fn a_link_without_a_target_and_a_regular_file_with_one_refuse() {
    let text: String = String::from_utf8(encode_capture(&a_capture())).expect("the record is text");
    let no_target: String = text.replace("entry=6c696e6b link 9 1763750400.5 1763750400.6 0 706c61696e2e747874", "entry=6c696e6b link 9 1763750400.5 1763750400.6 0 -");
    let refusal: String = parse_capture(no_target.as_str()).expect_err("a link names no target");
    assert!(refusal.contains("names no target"), "{refusal}");
    let regular_with_one: String = text.replace("entry=706c61696e2e747874 regular 6 1763750400.1 1763750400.2 0 -", "entry=706c61696e2e747874 regular 6 1763750400.1 1763750400.2 0 aa");
    let with_a_target: String = parse_capture(regular_with_one.as_str()).expect_err("a file names no target");
    assert!(with_a_target.contains("names a link target"), "{with_a_target}");
}

#[test]
fn the_bound_grows_with_the_entry_count_and_the_fixed_lines_fit() {
    assert_eq!(capture_bound(0), crate::worktree_clone_add_route::add_capture_record::CAPTURE_FIXED_BYTES);
    assert!(capture_bound(10) > capture_bound(9));
    let bytes: Vec<u8> = encode_capture(&a_capture());
    assert!((bytes.len() as u64) <= capture_bound(4), "the fixture's record fits its own bound");
    assert!(capture_bound(0) >= bytes.len() as u64, "the fixed part alone is already a generous bound");
}
