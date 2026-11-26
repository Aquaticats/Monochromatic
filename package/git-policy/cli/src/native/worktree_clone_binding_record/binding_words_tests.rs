//! Controls for the record's fixed words:
//!  every phase and reason a record may hold,
//!  the request forms,
//!  and the key order the writer and the reader share.

use crate::worktree_clone_binding_record::*;

/// Every phase's word reads back as the same phase.
#[test]
fn every_phase_reads_back_as_itself() {
    let phases: [Phase; 8] = [
        Phase::Claimed,
        Phase::Slot,
        Phase::Registered,
        Phase::Snapshotted,
        Phase::Cloned,
        Phase::Mounted,
        Phase::Filled,
        Phase::Settling,
    ];
    for phase in phases {
        assert_eq!(
            Phase::parse(phase.word()),
            Some(phase),
            "phase `{}`",
            phase.word()
        );
    }
    assert_eq!(Phase::parse("committed"), None);
}

/// Every phase's order rises,
/// which is what the parser compares a field's proof against.
#[test]
fn the_phases_are_ordered_by_how_far_the_add_got() {
    let phases: [Phase; 8] = [
        Phase::Claimed,
        Phase::Slot,
        Phase::Registered,
        Phase::Snapshotted,
        Phase::Cloned,
        Phase::Mounted,
        Phase::Filled,
        Phase::Settling,
    ];
    let orders: Vec<u8> = phases.iter().map(|phase| return phase.order()).collect();
    assert_eq!(orders, vec![0, 1, 2, 3, 4, 5, 6, 7]);
}

/// Every settle reason's word reads back as the same reason.
#[test]
fn every_settle_reason_reads_back_as_itself() {
    let reasons: [SettleReason; 7] = [
        SettleReason::Proof,
        SettleReason::Enospc,
        SettleReason::Mount,
        SettleReason::Clone,
        SettleReason::Snapshot,
        SettleReason::Destroy,
        SettleReason::Checkout,
    ];
    for reason in reasons {
        assert_eq!(
            SettleReason::parse(reason.word()),
            Some(reason),
            "reason `{}`",
            reason.word()
        );
    }
    assert_eq!(SettleReason::parse("proof"), Some(SettleReason::Proof));
    assert_eq!(SettleReason::parse("failed"), None);
}

/// Every request form's word reads back as the same form.
#[test]
fn every_request_form_reads_back_as_itself() {
    let forms: [RequestForm; 3] = [
        RequestForm::Branch,
        RequestForm::Commit,
        RequestForm::Detach,
    ];
    for form in forms {
        assert_eq!(
            RequestForm::parse(form.word()),
            Some(form),
            "form `{}`",
            form.word()
        );
    }
    assert_eq!(RequestForm::parse("tag"), None);
}

/// The grammar names 24 keys,
/// each once,
/// in the order the design writes them.
#[test]
fn the_keys_are_unique_and_in_the_designs_order() {
    assert_eq!(BINDING_KEYS_IN_ORDER.len(), 24);
    let mut seen: Vec<&str> = Vec::new();
    for key in BINDING_KEYS_IN_ORDER {
        assert!(!seen.contains(&key), "key `{key}` is written twice");
        seen.push(key);
    }
    assert_eq!(BINDING_KEYS_IN_ORDER[0], "claim");
    assert_eq!(BINDING_KEYS_IN_ORDER[3], "phase");
    assert_eq!(BINDING_KEYS_IN_ORDER[4], "settleReason");
    assert_eq!(
        BINDING_KEYS_IN_ORDER[BINDING_KEYS_IN_ORDER.len() - 1],
        "pointer"
    );
}

/// The two headers name their grammars and their versions.
#[test]
fn the_headers_name_the_version() {
    assert!(BINDING_HEADER.ends_with("binding, version 1"));
    assert!(INVENTORY_HEADER.ends_with("inventory, version 1"));
    assert!(BINDING_HEADER.starts_with("# monochromatic-git-policy-cli zfs worktree clone "));
}
