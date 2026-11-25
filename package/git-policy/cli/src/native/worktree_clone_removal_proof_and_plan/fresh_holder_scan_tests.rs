//! What:
//!  The controls of the fresh scan's own lines and of its answer as a seam.
//! Why:
//!  A refusal and a report print the same holders,
//!  so one function spells them for both,
//!  and the scan's own namespace is reported apart from the other namespaces' holders.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theHolderLines(scan)).toHaveLength(2);
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::the_holder_lines;
use std::path::PathBuf;

/// One scan with one mount in the reader's own namespace and one holder elsewhere.
fn a_scan() -> HolderScan {
    return HolderScan {
        dataset: DatasetGuid::parse("7828944690444299989").expect("a GUID"),
        source: String::from("tank/wt/fix-flaky-test"),
        nonce: RequestNonce::parse("00112233445566778899aabbccddeeff").expect("a nonce"),
        own_namespace: Some(String::from("mnt:[4026532000]")),
        namespaces_scanned: 3,
        own_mounts: vec![HeldMount { mount_id: 40, root: PathBuf::from("/"), mount_point: PathBuf::from("/home/user/Mono/.claude/worktrees/fix-flaky-test") }],
        holders: vec![NamespaceHolder {
            namespace: String::from("mnt:[4026532001]"),
            task: PathBuf::from("/proc/1234"),
            mounts: vec![HeldMount { mount_id: 41, root: PathBuf::from("/"), mount_point: PathBuf::from("/srv/holder") }],
        }],
        incomplete: vec![IncompleteRead::NamespaceLinkUnreadable { thread: PathBuf::from("/proc/999/task/999"), reason: String::from("permission denied") }],
        time: String::from("2026-11-25T09:12:44-05:00"),
    };
}

#[test]
fn the_scan_names_its_own_mount_and_every_holders_mount() {
    let lines: Vec<String> = the_holder_lines(&a_scan());
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("this process's own namespace"));
    assert!(lines[1].contains("namespace mnt:[4026532001]"));
    assert!(lines[1].contains("/srv/holder"));
}

#[test]
fn a_clear_scan_prints_nothing() {
    let mut scan: HolderScan = a_scan();
    scan.own_mounts.clear();
    scan.holders.clear();
    assert!(the_holder_lines(&scan).is_empty());
}
