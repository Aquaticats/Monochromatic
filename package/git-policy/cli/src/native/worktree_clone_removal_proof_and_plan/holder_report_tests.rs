//! What:
//!  The controls of one scan's answer as a report that names holders reads it:
//!  a holder's namespace and task,
//!  the reader's own mounts kept apart,
//!  and an incomplete read in the scan's own words.
//! Why:
//!  A live scan cannot be made on this machine's own pool by a control,
//!  so the mapping is proved on a value the control writes,
//!  and the live half is covered by the arm that runs in a guest.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = holdersOf(scanWithOneHolder);
//! expect(answer.holders[0]).toEqual({ namespace: 'mnt:[4026532001]', task: 777, mountPoint: '/cache/staged-7' });
//! ```

/// The mapping,
/// the scan it reads,
/// and the scan's own cause of an incomplete read.
use super::{
    AProvenHolder,
    holders_of,
};
use crate::clone_holder_scan::NamespaceHolder;
use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use std::path::PathBuf;

/// The namespace the control's holder is in.
const A_HOLDERS_NAMESPACE: &str = "mnt:[4026532001]";

/// The task the control's holder names.
const A_HOLDERS_TASK: u32 = 777;

/// The namespace the reader's own task is in.
const THE_READERS_NAMESPACE: &str = "mnt:[4026531840]";

/// A scan of one dataset that found nothing.
fn a_clear_scan() -> HolderScan {
    return HolderScan {
        dataset: DatasetGuid::parse("7").expect("a canonical GUID"),
        source: String::from("tank/clones/one"),
        nonce: RequestNonce::parse("0123456789abcdef0123456789abcdef").expect("a nonce"),
        own_namespace: Some(String::from(THE_READERS_NAMESPACE)),
        namespaces_scanned: 1,
        own_mounts: Vec::new(),
        holders: Vec::new(),
        incomplete: Vec::new(),
        time: String::from("2026-11-27T00:00:00-05:00"),
    };
}

/// One mount the control's scan matched.
fn a_mount(mount_point: &str) -> HeldMount {
    return HeldMount {
        mount_id: 41,
        root: PathBuf::from("/"),
        mount_point: PathBuf::from(mount_point),
    };
}

/// What:
///  A holder is named by the namespace it is in and one live task there,
///  which is what the plan asks a report to print.
#[test]
fn a_holder_is_named_by_its_namespace_and_its_task() {
    let mut scan: HolderScan = a_clear_scan();
    scan.holders = Vec::from([NamespaceHolder {
        namespace: String::from(A_HOLDERS_NAMESPACE),
        task: PathBuf::from(format!("/proc/{A_HOLDERS_TASK}/task/{A_HOLDERS_TASK}")),
        mounts: Vec::from([a_mount("/home/user/.cache/cli-git/staged-7")]),
    }]);
    let answer = holders_of(&scan);
    assert_eq!(
        answer.holders,
        Vec::from([AProvenHolder {
            namespace: String::from(A_HOLDERS_NAMESPACE),
            task: A_HOLDERS_TASK,
            mount_point: PathBuf::from("/home/user/.cache/cli-git/staged-7"),
        }])
    );
    assert!(answer.own_mounts.is_empty());
    assert!(answer.incomplete.is_empty());
}

/// What:
///  The reader's own mounts are kept apart from the namespaces a removal must
///  ask to unmount,
///  and they are still named.
#[test]
fn the_readers_own_mounts_are_kept_apart() {
    let mut scan: HolderScan = a_clear_scan();
    scan.own_mounts = Vec::from([a_mount("/repo/.claude/worktrees/one")]);
    let answer = holders_of(&scan);
    assert!(answer.holders.is_empty());
    assert_eq!(
        answer.own_mounts,
        Vec::from([PathBuf::from("/repo/.claude/worktrees/one")])
    );
    assert_eq!(
        answer.own_namespace,
        Some(String::from(THE_READERS_NAMESPACE))
    );
}

/// What:
///  An incomplete read is printed in the scan's own words,
///  with the namespace and the task it belongs to.
/// Why:
///  A scan that could not read every table proves no absence,
///  and a person needs to know which read kept it from proving one.
#[test]
fn an_incomplete_read_names_its_cause_and_its_namespace() {
    let mut scan: HolderScan = a_clear_scan();
    scan.incomplete = Vec::from([IncompleteRead::TableUnreadable {
        namespace: String::from(A_HOLDERS_NAMESPACE),
        task: PathBuf::from(format!("/proc/{A_HOLDERS_TASK}/task/{A_HOLDERS_TASK}")),
        reason: String::from("Permission denied (os error 13)"),
    }]);
    let answer = holders_of(&scan);
    assert!(answer.holders.is_empty());
    assert_eq!(answer.incomplete.len(), 1);
    let said: &str = answer.incomplete[0].as_str();
    assert!(said.contains(A_HOLDERS_NAMESPACE), "{said}");
    assert!(said.contains(A_HOLDERS_TASK.to_string().as_str()), "{said}");
}

/// What:
///  A task name that is not a number names no task,
///  and the holder is still named by its namespace.
#[test]
fn a_task_name_that_is_not_a_number_names_no_task() {
    let mut scan: HolderScan = a_clear_scan();
    scan.holders = Vec::from([NamespaceHolder {
        namespace: String::from(A_HOLDERS_NAMESPACE),
        task: PathBuf::from("/proc/not-a-number/task/not-a-number"),
        mounts: Vec::from([a_mount("/cache/staged-7")]),
    }]);
    let answer = holders_of(&scan);
    assert_eq!(answer.holders.len(), 1);
    assert_eq!(answer.holders[0].task, 0);
    assert_eq!(answer.holders[0].namespace, A_HOLDERS_NAMESPACE);
}
