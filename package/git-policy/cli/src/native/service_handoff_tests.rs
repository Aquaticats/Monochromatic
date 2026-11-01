//! Controls for the wrapper's side of after-commit work.

use super::*;
use crate::service_client::Activation;
use crate::service_messages::push_failed_notice;
use crate::service_server::Bind;
use crate::service_work::write_notice;
use crate::test_support::{fixture, remove};
use crate::work_item::{PENDING, state_directory};
use std::time::Duration;

const OID: &str = "0123456789abcdef0123456789abcdef01234567";
const PARENT: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn handoff(root: &std::path::Path, activation: Activation) -> ServiceHandoff {
    return ServiceHandoff {
        config: ClientConfig {
            address: Bind::Path(root.join("no-service.sock")),
            activation,
            start_wait: Duration::from_millis(100),
            commit_wait: Duration::from_secs(1),
        },
        repository: root.join("worktree"),
        common_dir: root.join("common"),
        reference: String::from("refs/heads/main"),
        parent: PARENT.to_string(),
    };
}

/// A handoff writes one pending record per handed-over kind, named for the kind and the commit, before it wakes anyone;
///  with no service installed it says so plainly and leaves the records for the next wake.
#[test]
fn the_records_are_written_before_the_wake_and_survive_a_failed_one() {
    let root: std::path::PathBuf = fixture("handoff-records");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::NotInstalled);
    let mut sink: Vec<u8> = Vec::new();
    let failure = hand_over(&handoff, OID, &mut sink).expect_err("no service is installed");
    assert_eq!(failure, ClientFailure::NotInstalled);
    let pending: std::path::PathBuf = state_directory(work_directory(handoff.common_dir.as_path()).as_path(), PENDING);
    let mut found: Vec<String> = std::fs::read_dir(pending).expect("pending").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    let mut expected: Vec<String> = HANDED_OVER.iter().map(|kind| { return format!("{}-{OID}", kind.name()); }).collect();
    expected.sort();
    assert_eq!(found, expected);
    remove(root.as_path());
}

/// A second handoff of the same commit adds nothing (the record names are derived), so a retry cannot duplicate work.
#[test]
fn a_repeated_handoff_adds_no_second_record() {
    let root: std::path::PathBuf = fixture("handoff-repeat");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::NotInstalled);
    let mut sink: Vec<u8> = Vec::new();
    let _first = hand_over(&handoff, OID, &mut sink);
    let _second = hand_over(&handoff, OID, &mut sink);
    let pending: std::path::PathBuf = state_directory(work_directory(handoff.common_dir.as_path()).as_path(), PENDING);
    assert_eq!(std::fs::read_dir(pending).expect("pending").count(), HANDED_OVER.len());
    remove(root.as_path());
}

/// Notices left by earlier work print once, in the order of their names, and are gone afterwards.
#[test]
fn notices_print_once() {
    let root: std::path::PathBuf = fixture("handoff-notices");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::NotInstalled);
    write_notice(handoff.common_dir.as_path(), "push-a", push_failed_notice(OID, "first").as_str()).expect("notice");
    write_notice(handoff.common_dir.as_path(), "push-b", push_failed_notice(PARENT, "second").as_str()).expect("notice");
    let mut shown: Vec<u8> = Vec::new();
    show_notices(&handoff, &mut shown);
    let text: String = String::from_utf8(shown).expect("UTF-8");
    assert_eq!(text, format!("{}\n{}\n", push_failed_notice(OID, "first"), push_failed_notice(PARENT, "second")));
    let mut again: Vec<u8> = Vec::new();
    show_notices(&handoff, &mut again);
    assert!(again.is_empty());
    remove(root.as_path());
}

/// A review record is left only where the worktree keeps a shadow log; elsewhere the handed-over kinds are all there is.
#[test]
fn a_review_record_is_left_only_where_the_repository_keeps_a_shadow_log() {
    let root: std::path::PathBuf = fixture("handoff-review");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::NotInstalled);
    let pending: std::path::PathBuf = state_directory(work_directory(handoff.common_dir.as_path()).as_path(), PENDING);
    let mut sink: Vec<u8> = Vec::new();
    let _without = hand_over(&handoff, OID, &mut sink);
    assert!(!pending.join(format!("{}-{OID}", WorkKind::Review.name())).exists());
    let log: std::path::PathBuf = handoff.repository.join(crate::standards_review_input::LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directory");
    std::fs::write(log.as_path(), "").expect("log");
    let _with = hand_over(&handoff, PARENT, &mut sink);
    assert!(pending.join(format!("{}-{PARENT}", WorkKind::Review.name())).exists());
    assert_eq!(std::fs::read_dir(pending).expect("pending").count(), 2 * HANDED_OVER.len() + 1);
    remove(root.as_path());
}
