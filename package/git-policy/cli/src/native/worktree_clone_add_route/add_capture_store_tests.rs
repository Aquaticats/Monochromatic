//! Controls for the capture's store:
//!  one publication that replaces nothing,
//!  a read that refuses a writable record,
//!  the proof marker,
//!  and the clearing of another add's records at one slot.

use super::{capture_name, clear_other_adds_records, marker_is_ours, publish_capture, publish_marker, read_capture, remove_capture};
use crate::test_support::{fixture, remove};
use crate::worktree_clone_add_route::add_capture::{Capture, CapturedDigest, CapturedEntry, CapturedIndex, CapturedKind};
use crate::worktree_clone_binding_record::registration_home::{home_folder, open_or_create_home};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

/// One capture keyed by `nonce`.
fn a_capture(nonce: &RequestNonce) -> Capture {
    return Capture {
        nonce: nonce.clone(),
        repository: b"/repo/.git".to_vec(),
        source: b"/repo".to_vec(),
        checkout_guid: DatasetGuid::parse("7").expect("the guid"),
        index: CapturedIndex { size: 1, mtime: (1, 2), digest: [3; 32] },
        head: String::new(),
        source_head: String::new(),
        entries: Vec::new(),
        digests: Vec::new(),
    };
}

/// What:
///  A capture whose encoded bytes are longer than the prefix the reader takes.
/// Why:
///  The reader learns the record's whole bound from its first bytes,
///  and a capture of a few hundred tracked paths is longer than that prefix,
///  which is the shape a person's checkout has and the empty capture does not.
fn a_capture_longer_than_the_prefix(nonce: &RequestNonce) -> Capture {
    let mut capture: Capture = a_capture(nonce);
    let count: usize = 200;
    for index in 0..count {
        let path: Vec<u8> = format!("a/long/tracked/path/number/{index:03}/of/the/record").into_bytes();
        capture.entries.push(CapturedEntry {
            path: path.clone(),
            kind: CapturedKind::Regular,
            size: 1,
            mtime: (1, 2),
            ctime: (1, 2),
            executable: false,
            link_target: None,
        });
        capture.digests.push(CapturedDigest { path, digest: [7; 32] });
    }
    return capture;
}

/// A common directory with a fresh home.
fn a_home(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    open_or_create_home(common.as_path()).expect("the home opens");
    return (root, home_folder(common.as_path()));
}

#[test]
fn one_publication_stays_and_a_second_replaces_nothing() {
    let (root, home) = a_home("capture-store-publication");
    let nonce: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let capture: Capture = a_capture(&nonce);
    let folder: std::fs::File = open_or_create_home(root.join(".git").as_path()).expect("the home opens");
    publish_capture(&folder, "topic", &capture).expect("the first publication");
    let read: Capture = read_capture(home.as_path(), "topic").expect("the capture reads").expect("it stands");
    assert_eq!(read, capture);
    let second: std::io::Result<()> = publish_capture(&folder, "topic", &capture);
    assert!(second.is_err(), "a second publication replaces nothing");
    remove(root.as_path());
}

#[test]
fn a_capture_longer_than_the_prefix_reads_back() {
    let (root, home) = a_home("capture-store-prefix");
    let nonce: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let capture: Capture = a_capture_longer_than_the_prefix(&nonce);
    let folder: std::fs::File = open_or_create_home(root.join(".git").as_path()).expect("the home opens");
    publish_capture(&folder, "topic", &capture).expect("the publication");
    let bytes: u64 = std::fs::metadata(home.join(capture_name("topic"))).expect("the record's metadata").len();
    assert!(bytes > 8 * 1024, "the control's capture is longer than the prefix the reader takes: {bytes} bytes");
    let read: Capture = read_capture(home.as_path(), "topic").expect("a capture longer than the prefix reads").expect("it stands");
    assert_eq!(read.entries.len(), capture.entries.len(), "every entry of the longer capture reads back");
    remove(root.as_path());
}

#[test]
fn a_record_another_account_or_another_mode_may_write_is_refused() {
    let (root, home) = a_home("capture-store-mode");
    let nonce: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let folder: std::fs::File = open_or_create_home(root.join(".git").as_path()).expect("the home opens");
    publish_capture(&folder, "topic", &a_capture(&nonce)).expect("the publication");
    let path: PathBuf = home.join(capture_name("topic"));
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o666)).expect("the mode is loosened");
    let refusal: String = read_capture(home.as_path(), "topic").expect_err("a writable record is refused");
    assert!(refusal.contains("may be written by this account's group or others"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
    remove(root.as_path());
}

#[test]
fn the_marker_says_whose_proof_it_is() {
    let (root, home) = a_home("capture-store-marker");
    let ours: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let folder: std::fs::File = open_or_create_home(root.join(".git").as_path()).expect("the home opens");
    publish_marker(&folder, "topic", &ours).expect("the marker is published");
    assert!(marker_is_ours(home.as_path(), "topic", &ours).expect("the marker reads"));
    let theirs: RequestNonce = RequestNonce::fresh().expect("another nonce");
    let refusal: String = marker_is_ours(home.as_path(), "topic", &theirs).expect_err("another add's marker is no proof");
    assert!(refusal.contains("this add's nonce is"), "{refusal}");
    remove_capture(home.as_path(), "topic");
    assert!(!marker_is_ours(home.as_path(), "topic", &ours).expect("a removed marker is not ours"));
    remove(root.as_path());
}

#[test]
fn another_adds_records_are_cleared_and_this_adds_are_kept() {
    let (root, home) = a_home("capture-store-clearing");
    let ours: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let theirs: RequestNonce = RequestNonce::fresh().expect("another nonce");
    let folder: std::fs::File = open_or_create_home(root.join(".git").as_path()).expect("the home opens");
    publish_capture(&folder, "topic", &a_capture(&theirs)).expect("another add's capture");
    publish_marker(&folder, "topic", &theirs).expect("another add's marker");
    clear_other_adds_records(home.as_path(), "topic", &ours).expect("the clearing");
    assert_eq!(read_capture(home.as_path(), "topic").expect("no refusal"), None, "another add's capture is gone");
    assert!(!marker_is_ours(home.as_path(), "topic", &theirs).expect("no marker stands"));
    publish_capture(&folder, "topic", &a_capture(&ours)).expect("this add's capture");
    publish_marker(&folder, "topic", &ours).expect("this add's marker");
    clear_other_adds_records(home.as_path(), "topic", &ours).expect("the second clearing");
    assert_eq!(read_capture(home.as_path(), "topic").expect("no refusal").map(|capture| return capture.nonce), Some(ours.clone()), "this add's own record stays");
    let _shown: &Path = home.as_path();
    remove(root.as_path());
}
