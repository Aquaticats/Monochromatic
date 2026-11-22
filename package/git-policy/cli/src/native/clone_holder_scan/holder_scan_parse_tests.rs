//! What:
//!  Controls of reading a result back:
//!  one result round-trips through its own text,
//!  a reordered line,
//!  a cut-short line,
//!  a trailing line,
//!  a count larger than the lines it introduces,
//!  another record's kind,
//!  another version and a nonce that is none are each refused.
//! Why:
//!  A later step reads this file across a privilege boundary,
//!  so nothing in it can be defaulted:
//!  a file cut short must not read as a clear result,
//!  and a reordered one must not read as another's findings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseHolderScan(scan.text())).toEqual(scan);
//! ```

use super::super::held_mounts::HeldMount;
use super::super::holder_scan::HolderScan;
use super::super::holder_scan_refusals::HolderScanRefusal;
use super::super::incomplete_reads::IncompleteRead;
use super::super::namespace_holder::NamespaceHolder;
use super::parse_holder_scan;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::PathBuf;

/// The time every fixture result carries.
const TIME: &str = "2026-11-22T16:00:00-05:00";

/// One result with one holder,
/// one managed attach set aside and one incomplete read.
fn one_scan() -> HolderScan {
    return HolderScan {
        dataset: DatasetGuid::parse("42").expect("the fixture GUID is canonical"),
        source: String::from("zpcachyos/nosnap/clones/one"),
        nonce: RequestNonce::parse("00112233445566778899aabbccddeeff").expect("the fixture nonce is one"),
        own_namespace: Some(String::from("mnt:[7]")),
        namespaces_scanned: 3,
        own_mounts: vec![HeldMount { mount_id: 41, root: PathBuf::from("/"), mount_point: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one") }],
        holders: vec![NamespaceHolder {
            namespace: String::from("mnt:[9]"),
            task: PathBuf::from("/proc/21"),
            mounts: vec![
                HeldMount { mount_id: 42, root: PathBuf::from("/"), mount_point: PathBuf::from("/srv/holder") },
                HeldMount { mount_id: 43, root: PathBuf::from("/sub"), mount_point: PathBuf::from("/srv/two words") },
            ],
        }],
        excluded: vec![NamespaceHolder { namespace: String::from("mnt:[7]"), task: PathBuf::from("/proc/11"), mounts: vec![HeldMount { mount_id: 44, root: PathBuf::from("/"), mount_point: PathBuf::from("/staging/one") }] }],
        incomplete: vec![IncompleteRead::NoLiveMember { namespace: String::from("mnt:[5]"), tasks_tried: 2, reason: String::from("No such file or directory (os error 2)") }],
        time: String::from(TIME),
    };
}

/// One result written and read back is the same result,
/// which is what lets a later step compare what it was told with what it asked.
#[test]
fn a_result_round_trips_through_its_text() {
    let scan: HolderScan = one_scan();
    let read: HolderScan = parse_holder_scan(scan.text().as_str()).expect("the text this route wrote reads back");
    assert_eq!(read, scan);
}

/// A line out of the order this route writes is refused.
#[test]
fn a_reordered_line_is_refused() {
    let text: String = one_scan().text();
    let swapped: String = text.replace("kind clone-holder-scan\ndataset 42", "dataset 42\nkind clone-holder-scan");
    match parse_holder_scan(swapped.as_str()) {
        Err(HolderScanRefusal::Key { expected, .. }) => assert_eq!(expected, "kind"),
        other => panic!("{other:?}"),
    }
}

/// A result cut short is refused rather than read as a result without
/// holders.
#[test]
fn a_cut_short_result_is_refused() {
    let text: String = one_scan().text();
    let cut: String = text.lines().filter(|line| return !line.starts_with("holder ")).collect::<Vec<&str>>().join("\n");
    match parse_holder_scan(cut.as_str()) {
        Err(HolderScanRefusal::Key { expected, .. }) => assert_eq!(expected, "holder"),
        other => panic!("{other:?}"),
    }
}

/// A line after the result's last field is refused.
#[test]
fn a_trailing_line_is_refused() {
    let text: String = format!("{}one more line\n", one_scan().text());
    assert!(matches!(parse_holder_scan(text.as_str()), Err(HolderScanRefusal::TrailingLine { .. })));
}

/// A count larger than the lines it introduces is refused before any line is
/// read under it.
#[test]
fn a_count_past_the_lines_is_refused() {
    let text: String = one_scan().text().replace("holder-namespaces 1", "holder-namespaces 9");
    match parse_holder_scan(text.as_str()) {
        Err(HolderScanRefusal::Count { key, .. }) => assert_eq!(key, "holder-namespaces"),
        other => panic!("{other:?}"),
    }
}

/// Another record's kind,
/// another version and a nonce that is none are each refused.
#[test]
fn another_kind_version_or_nonce_is_refused() {
    let text: String = one_scan().text();
    let kind: String = text.replace("kind clone-holder-scan", "kind clone-mount-result");
    assert!(matches!(parse_holder_scan(kind.as_str()), Err(HolderScanRefusal::Kind { .. })));
    let version: String = text.replacen("v1", "v2", 1);
    assert!(matches!(parse_holder_scan(version.as_str()), Err(HolderScanRefusal::Version { .. })));
    let nonce: String = text.replace("nonce 00112233445566778899aabbccddeeff", "nonce 00112233445566778899AABBCCDDEEFF");
    assert!(matches!(parse_holder_scan(nonce.as_str()), Err(HolderScanRefusal::Nonce(_))));
}
