//! What:
//!  Controls of the holders answer:
//!  no holder with every read complete,
//!  a holder named by namespace and task,
//!  and a scan that could not read everything.
//! Why:
//!  A removal's destructive steps rest on this answer,
//!  so a holder must be a finding that names where it lives and an incomplete
//!  scan must never pass for a clean one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_holders({ holders: [], incomplete: ['task 90'] })).toMatchObject({ kind: 'unknown' });
//! ```

use super::{
    HolderFacts,
    NamespaceHold,
    check_holders,
};
use std::path::PathBuf;

/// The clone a control asks about.
const CLONE: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one";

/// One namespace holding the clone.
fn hold(
    namespace: &str,
    pid: u32,
) -> NamespaceHold {
    return NamespaceHold {
        namespace: String::from(namespace),
        representative_pid: pid,
        source: String::from(CLONE),
        mount_point: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"),
    };
}

/// No holder and every read complete passes.
#[test]
fn no_holder_and_every_read_complete_passes() {
    let facts: HolderFacts = HolderFacts {
        clone: String::from(CLONE),
        holders: Vec::new(),
        incomplete: Vec::new(),
    };
    assert!(matches!(
        check_holders(&facts),
        super::CheckOutcome::Pass { .. }
    ));
}

/// A holder is a finding naming the namespace,
/// the task and the path.
#[test]
fn a_holder_is_a_finding_naming_namespace_task_and_path() {
    let facts: HolderFacts = HolderFacts {
        clone: String::from(CLONE),
        holders: vec![hold("mnt:[4026532001]", 4242)],
        incomplete: Vec::new(),
    };
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_holders(&facts)
    else {
        panic!("a holder is a finding");
    };
    assert_eq!(field, "holder");
    assert!(detail.contains("mnt:[4026532001]"), "{detail}");
    assert!(detail.contains("4242"), "{detail}");
    assert!(detail.contains("worktrees/one"), "{detail}");
}

/// Every holder is named,
/// not only the first.
#[test]
fn every_holder_is_named() {
    let facts: HolderFacts = HolderFacts {
        clone: String::from(CLONE),
        holders: vec![hold("mnt:[1]", 1), hold("mnt:[2]", 2)],
        incomplete: Vec::new(),
    };
    let super::CheckOutcome::Finding {
        detail,
        ..
    } = check_holders(&facts)
    else {
        panic!("holders are a finding");
    };
    assert!(
        detail.contains("mnt:[1]") && detail.contains("mnt:[2]"),
        "{detail}"
    );
}

/// An incomplete scan with no holder leaves the answer unknown.
#[test]
fn an_incomplete_scan_with_no_holder_is_unknown() {
    let facts: HolderFacts = HolderFacts {
        clone: String::from(CLONE),
        holders: Vec::new(),
        incomplete: vec![String::from(
            "task 90's namespace link is unreadable (EACCES)",
        )],
    };
    let super::CheckOutcome::Unknown {
        reason,
    } = check_holders(&facts)
    else {
        panic!("an incomplete scan is unknown");
    };
    assert!(reason.contains("task 90"), "{reason}");
}

/// A holder beside an incomplete read stays a finding,
/// and the report says the scan was incomplete too.
#[test]
fn a_holder_beside_an_incomplete_read_stays_a_finding() {
    let facts: HolderFacts = HolderFacts {
        clone: String::from(CLONE),
        holders: vec![hold("mnt:[1]", 1)],
        incomplete: vec![String::from("one table could not be read")],
    };
    let super::CheckOutcome::Finding {
        detail,
        ..
    } = check_holders(&facts)
    else {
        panic!("a holder is a finding");
    };
    assert!(detail.contains("could not be completed"), "{detail}");
}
