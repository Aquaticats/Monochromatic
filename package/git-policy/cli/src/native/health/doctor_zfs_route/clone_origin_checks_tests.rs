//! What:
//!  Controls of one clone's origin line:
//!  a record that names no origin,
//!  an unmarked origin,
//!  a marked one with its two counts,
//!  a mark whose hold count could not be read,
//!  a mark the reply could not carry,
//!  an origin the pool does not hold,
//!  and an origin nobody read.
//! Why:
//!  The mark decides whether a removal of this clone would destroy history
//!  another program asked to keep,
//!  so each arm is shown with the words that describe what a person can do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkCloneOrigin({ ...facts, live: Read(OriginMark.Marked({ clones: 1, holds: Counted(0) })) })).toMatchObject({ kind: 'finding', field: 'defer_destroy' });
//! ```

use super::{
    CloneOriginFacts,
    RecordedOrigin,
    check_clone_origin,
};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::health::doctor_zfs_route::zfs_checks::Reading;
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    OriginMark,
    UserHolds,
};

/// The clone these controls are about.
const CLONE: &str = "tank/wt/fix-flaky-test";

/// The origin snapshot the clone was made from.
const ORIGIN: &str =
    "tank/mono-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// One clone's origin facts.
fn facts(live: Option<Reading<OriginMark>>) -> CloneOriginFacts {
    return CloneOriginFacts {
        clone: String::from(CLONE),
        recorded: Some(RecordedOrigin {
            dataset: String::from("tank/mono-checkout"),
            snapshot: String::from(ORIGIN),
        }),
        live,
    };
}

/// A record that names no origin passes,
/// because the add has not snapshotted the checkout yet.
#[test]
fn a_record_without_an_origin_passes() {
    let mut held: CloneOriginFacts = facts(None);
    held.recorded = None;
    let answer: CheckOutcome = check_clone_origin(&held);
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("a record without an origin passes: {answer:?}");
    };
    assert!(detail.contains("names no origin yet"), "{detail}");
}

/// An unmarked origin passes,
/// and says the pool refuses to destroy it while the clone stands.
#[test]
fn an_unmarked_origin_passes() {
    let answer: CheckOutcome =
        check_clone_origin(&facts(Some(Reading::Read(OriginMark::Unmarked))));
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("an unmarked origin passes: {answer:?}");
    };
    assert!(detail.contains(ORIGIN), "{detail}");
    assert!(detail.contains("reads 0"), "{detail}");
    assert!(detail.contains("refuses to destroy it"), "{detail}");
}

/// A marked origin is a finding naming the counts and both ways out.
#[test]
fn a_marked_origin_is_a_finding_naming_the_counts() {
    let answer: CheckOutcome =
        check_clone_origin(&facts(Some(Reading::Read(OriginMark::Marked {
            clones: 2,
            holds: UserHolds::Counted(0),
        }))));
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a marked origin is a finding: {answer:?}");
    };
    assert_eq!(field, "defer_destroy");
    assert!(detail.contains("2 clone(s)"), "{detail}");
    assert!(detail.contains("0 user hold(s)"), "{detail}");
    assert!(detail.contains("cannot be cleared"), "{detail}");
    assert!(detail.contains("outside the wrapper"), "{detail}");
    assert!(
        detail.contains("refuses before its first delete"),
        "{detail}"
    );
}

/// A hold count the reply could not carry is never printed as zero.
#[test]
fn an_unreadable_hold_count_is_not_zero() {
    let answer: CheckOutcome =
        check_clone_origin(&facts(Some(Reading::Read(OriginMark::Marked {
            clones: 1,
            holds: UserHolds::Unreadable {
                said: String::from("`userrefs` is not in the reply"),
            },
        }))));
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a marked origin is a finding: {answer:?}");
    };
    assert!(
        detail.contains("a count that could not be read"),
        "{detail}"
    );
    assert!(!detail.contains("0 user hold(s)"), "{detail}");
}

/// A mark the reply could not carry leaves the answer unknown.
#[test]
fn an_unreadable_mark_is_unknown() {
    let answer: CheckOutcome =
        check_clone_origin(&facts(Some(Reading::Read(OriginMark::Unreadable {
            said: String::from("`defer_destroy` is not in the reply the kernel gave"),
        }))));
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("an unreadable mark is unknown: {answer:?}");
    };
    assert!(reason.contains("defer_destroy"), "{reason}");
}

/// An origin the pool does not hold is a finding naming the snapshot.
#[test]
fn a_gone_origin_is_a_finding() {
    let answer: CheckOutcome = check_clone_origin(&facts(Some(Reading::Missing {
        why: String::from("snapshot does not exist"),
    })));
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a gone origin is a finding: {answer:?}");
    };
    assert_eq!(field, "origin");
    assert!(detail.contains(ORIGIN), "{detail}");
    assert!(detail.contains("snapshot does not exist"), "{detail}");
}

/// An origin nobody read is unknown,
/// never a pass.
#[test]
fn an_unread_origin_is_unknown() {
    let answer: CheckOutcome = check_clone_origin(&facts(None));
    assert!(matches!(answer, CheckOutcome::Unknown { .. }), "{answer:?}");
}

/// An origin the pool refused to describe is unknown.
#[test]
fn an_unreadable_origin_is_unknown() {
    let answer: CheckOutcome = check_clone_origin(&facts(Some(Reading::Unreadable {
        why: String::from("permission denied"),
    })));
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("an unreadable origin is unknown: {answer:?}");
    };
    assert!(reason.contains("permission denied"), "{reason}");
}
