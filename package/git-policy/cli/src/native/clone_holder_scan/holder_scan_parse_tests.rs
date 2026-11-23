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
use crate::clone_holder_scan::holder_scan_text::{MAXIMUM_NAMESPACE_BYTES, MAXIMUM_RESULT_BYTES, MAXIMUM_SOURCE_BYTES};
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
        incomplete: vec![
            IncompleteRead::NoLiveMember { namespace: String::from("mnt:[5]"), tasks_tried: 2, reason: String::from("No such file or directory (os error 2)") },
            IncompleteRead::MountsPastTheBound { namespace: String::from("mnt:[9]"), bound: 8 },
        ],
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

/// A count of namespaces is not the number of findings below it,
/// so a result that read many namespaces and lists fewer findings parses.
#[test]
fn a_count_of_namespaces_is_not_the_number_of_lines_below_it() {
    let mut scan: HolderScan = one_scan();
    scan.namespaces_scanned = 40;
    let read: HolderScan = parse_holder_scan(scan.text().as_str()).expect("a namespace count is not a section count");
    assert_eq!(read.namespaces_scanned, 40);
    assert_eq!(read.incomplete_reads(), 2);
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

/// An empty value is refused rather than read as the absent word,
/// and a count with a sign or a leading zero is no count.
#[test]
fn an_empty_value_and_a_non_canonical_count_are_refused() {
    let text: String = one_scan().text();
    let empty_source: String = text.replace("source zpcachyos/nosnap/clones/one", "source ");
    assert!(matches!(parse_holder_scan(empty_source.as_str()), Err(HolderScanRefusal::Value { shown, .. }) if shown == "<empty>"));
    for count in ["+1", "01", "-1", "1_0"] {
        let signed: String = text.replace("holder-namespaces 1", format!("holder-namespaces {count}").as_str());
        assert!(matches!(parse_holder_scan(signed.as_str()), Err(HolderScanRefusal::Count { .. })), "{count}");
    }
}

/// A holder line whose mounts are not whole triples is refused,
/// so a cut or padded line is never read as a mount.
#[test]
fn a_holder_line_that_is_no_whole_triples_is_refused() {
    let text: String = one_scan().text();
    let padded: String = text.replace(" 42 / /srv/holder", " 42 / /srv/holder 43");
    assert!(matches!(parse_holder_scan(padded.as_str()), Err(HolderScanRefusal::Value { .. })), "{padded}");
}

/// A text past the result's bound is refused before any line is read.
#[test]
fn a_text_past_the_bound_is_refused() {
    let mut text: String = String::from("v1\n");
    text.push_str("x".repeat(MAXIMUM_RESULT_BYTES).as_str());
    assert!(matches!(parse_holder_scan(text.as_str()), Err(HolderScanRefusal::TooLarge { .. })));
}

/// A source or a namespace longer than its own bound is refused,
/// and a time that is no instant is refused at its line.
#[test]
fn a_long_source_namespace_or_a_time_that_is_no_instant_is_refused() {
    let text: String = one_scan().text();
    let long_source: String = text.replace("source zpcachyos/nosnap/clones/one", format!("source {}", "a".repeat(MAXIMUM_SOURCE_BYTES + 1)).as_str());
    assert!(matches!(parse_holder_scan(long_source.as_str()), Err(HolderScanRefusal::Value { key, .. }) if key == "source"));
    let long_namespace: String = text.replace("own-namespace mnt:[7]", format!("own-namespace {}", "n".repeat(MAXIMUM_NAMESPACE_BYTES + 1)).as_str());
    assert!(matches!(parse_holder_scan(long_namespace.as_str()), Err(HolderScanRefusal::Value { key, .. }) if key == "own-namespace"));
    let no_instant: String = text.replace("time 2026-11-22T16:00:00-05:00", "time yesterday");
    assert!(matches!(parse_holder_scan(no_instant.as_str()), Err(HolderScanRefusal::Time { .. })));
}

/// The two causes of this round's fixes are written and read back:
/// a namespace link the scan could not read and a namespace with no task at
/// its own root.
#[test]
fn the_new_causes_round_trip_through_their_text() {
    let mut scan: HolderScan = one_scan();
    scan.incomplete.push(IncompleteRead::NamespaceLinkUnreadable { thread: PathBuf::from("/proc/31/task/31"), reason: String::from("Permission denied (os error 13)") });
    scan.incomplete.push(IncompleteRead::NoTaskAtTheNamespaceRoot { namespace: String::from("mnt:[9]") });
    let read: HolderScan = parse_holder_scan(scan.text().as_str()).expect("the text this route wrote reads back");
    assert_eq!(read, scan);
}
