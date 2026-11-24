//! What:
//!  Controls of the result's text:
//!  a mount point with a space in it stays one field,
//!  a failure's sentence with a line break in it stays one line,
//!  the reader's own namespace with nothing in it is the absent word,
//!  and every result the scan writes fits the bound its reader checks first.
//! Why:
//!  A path with a space is a folder a person may make,
//!  and a reason is the kernel's own sentence,
//!  so the text has to carry both without letting either end a field.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(escapePath('/a b')).toBe('/a\\040b');
//! ```

use super::super::held_mounts::HeldMount;
use super::super::holder_scan::HolderScan;
use super::super::incomplete_reads::IncompleteRead;
use super::super::namespace_holder::NamespaceHolder;
use super::{ABSENT_WORD, MAXIMUM_RESULT_BYTES, RESULT_KIND, RESULT_VERSION, TIME_KEY, escape_path, is_a_result_time, unescape_path};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::PathBuf;

/// The time every fixture result carries.
const TIME: &str = "2026-11-22T16:00:00-05:00";

/// One result with one namespace holding the dataset,
/// which each control here extends.
fn one_scan() -> HolderScan {
    return HolderScan {
        dataset: DatasetGuid::parse("42").expect("the fixture GUID is canonical"),
        source: String::from("zpcachyos/nosnap/clones/one"),
        nonce: RequestNonce::parse("00112233445566778899aabbccddeeff").expect("the fixture nonce is one"),
        own_namespace: Some(String::from("mnt:[7]")),
        namespaces_scanned: 2,
        own_mounts: Vec::new(),
        holders: vec![NamespaceHolder {
            namespace: String::from("mnt:[9]"),
            task: PathBuf::from("/proc/21"),
            mounts: vec![HeldMount { mount_id: 42, root: PathBuf::from("/"), mount_point: PathBuf::from("/srv/holder") }],
        }],
        incomplete: Vec::new(),
        time: String::from(TIME),
    };
}

/// One path with every byte a line cannot carry written as its escape,
/// and read back exactly.
#[test]
fn a_path_with_a_space_in_it_stays_one_field() {
    let awkward: PathBuf = PathBuf::from("/home/user/two words/back\\slash");
    let escaped: String = escape_path(awkward.as_path());
    assert_eq!(escaped, String::from("/home/user/two\\040words/back\\134slash"));
    assert!(!escaped.contains(' '));
    assert_eq!(unescape_path(escaped.as_str()), awkward);
}

/// A reason with a line break and a control byte stays one line,
/// and the result holds one incomplete line for it.
#[test]
fn a_reason_with_a_line_break_stays_one_line() {
    let mut scan: HolderScan = one_scan();
    scan.incomplete.push(IncompleteRead::TableUnreadable {
        namespace: String::from("mnt:[9]"),
        task: PathBuf::from("/proc/21"),
        reason: String::from("Permission denied\nsecond line\twith a tab"),
    });
    let text: String = scan.text();
    assert!(text.contains("\\x0a"), "{text}");
    assert!(text.contains("\\x09"), "{text}");
    assert_eq!(text.lines().filter(|line| return line.starts_with("incomplete ")).count(), 1);
}

/// The absent word carries a namespace the reader could not name,
/// and the time is checked as a time.
#[test]
fn an_unknown_namespace_is_the_absent_word_and_the_time_is_checked() {
    let mut scan: HolderScan = one_scan();
    scan.own_namespace = None;
    let text: String = scan.text();
    assert!(text.contains(format!("own-namespace {ABSENT_WORD}").as_str()), "{text}");
    assert!(is_a_result_time(scan.time.as_str()));
    assert!(!is_a_result_time("yesterday"));
}

/// Every result a scan can write fits the bound its reader checks first,
/// and its first and last lines are the version and the time.
#[test]
fn a_result_fits_the_bound_its_reader_checks() {
    let scan: HolderScan = one_scan();
    let text: String = scan.text();
    assert!(text.len() <= MAXIMUM_RESULT_BYTES, "{} > {MAXIMUM_RESULT_BYTES}", text.len());
    assert_eq!(text.lines().next(), Some(RESULT_VERSION));
    assert!(text.contains(format!("kind {RESULT_KIND}").as_str()));
    assert!(text.trim_end().ends_with(format!("{TIME_KEY} {TIME}").as_str()), "{text}");
}

/// A tab and a newline in a mount point are escaped,
/// and an escape sequence is too,
/// so an account's own name for a folder cannot forge a line or reach a
/// terminal.
#[test]
fn a_tab_a_newline_and_an_escape_sequence_are_escaped() {
    let awkward: PathBuf = PathBuf::from("/srv/tab\there/new\nline/esc\u{1b}[2Jforged");
    let escaped: String = escape_path(awkward.as_path());
    assert!(escaped.contains("\\011"), "{escaped}");
    assert!(escaped.contains("\\012"), "{escaped}");
    assert!(escaped.contains("\\033"), "{escaped}");
    assert!(!escaped.contains('\t') && !escaped.contains('\n') && !escaped.contains('\u{1b}'), "{escaped}");
    assert_eq!(unescape_path(escaped.as_str()), awkward);
}
