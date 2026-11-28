//! What:
//!  Controls of the filter's rule,
//!  one case per name a delivery can carry:
//!  the parent itself,
//!  a dataset below it,
//!  a snapshot of one,
//!  a dataset outside it,
//!  a name that only begins like the parent,
//!  another pool,
//!  a delivery with no pool,
//!  one with no dataset,
//!  and no parent name at all.
//! Why:
//!  The rule is the design's step 19 filter,
//!  and each case is one decision a real delivery can meet
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  veto entry V2,
//!  and "The tests a build must have",
//!  its test 1).
//!  Nothing here times anything:
//!  every case is one reading of two strings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(decideAboutTheEvent({ pool, dataset: clone + "@auto" }, parent)).toEqual({ kind: 'passed', why: 'below-the-parent' });
//! ```

use super::decide_about_the_event;
use crate::worktree_clone_mount_event_filter::event_filter_decision::{
    DroppedWhy,
    FilterDecision,
    PassedWhy,
};
use crate::worktree_clone_mount_event_filter::event_identity::EventIdentity;
use crate::worktree_clone_mount_event_filter::parent_scope::ParentScope;

/// The pool the route's parent lives in.
const POOL: &str = "zpcachyos";

/// The pinned clone parent dataset.
const PARENT: &str = "zpcachyos/nosnap/zfsroute-clones";

/// The route's parent names,
/// which every case below is compared against.
fn parent() -> ParentScope {
    return ParentScope::of_dataset(PARENT).expect("a dataset name");
}

/// The decision about one delivery,
/// with the parent this folder names.
fn decision_about(
    pool: Option<&str>,
    dataset: Option<&str>,
) -> FilterDecision {
    return decide_about_the_event(&EventIdentity::of(pool, dataset), Some(&parent()));
}

/// One case of the rule:
/// the two names a delivery carries and the answer they decide.
struct Case {
    /// What the case is,
    /// as a person reads it in a failure.
    subject: &'static str,
    /// The delivery's pool,
    /// absent when the delivery carried none.
    pool: Option<&'static str>,
    /// The delivery's dataset,
    /// absent when the delivery carried none.
    dataset: Option<&'static str>,
    /// The answer the rule must give.
    wanted: FilterDecision,
}

/// The answer a delivery about a dataset below the parent gets.
fn below_the_parent() -> FilterDecision {
    return FilterDecision::Passed(PassedWhy::BelowTheParent);
}

/// The answer a delivery outside the parent gets,
/// with the dataset the filter saw.
fn outside_the_parent(dataset: &str) -> FilterDecision {
    return FilterDecision::Dropped(DroppedWhy::OutsideTheParent {
        parent: String::from(PARENT),
        dataset: String::from(dataset),
    });
}

/// The cases this folder's rule is held to.
fn the_cases() -> Vec<Case> {
    return vec![
        Case {
            subject: "the clone parent dataset itself",
            pool: Some(POOL),
            dataset: Some(PARENT),
            wanted: FilterDecision::Passed(PassedWhy::TheParentsOwnDataset),
        },
        Case {
            subject: "a clone below the parent",
            pool: Some(POOL),
            dataset: Some("zpcachyos/nosnap/zfsroute-clones/one"),
            wanted: below_the_parent(),
        },
        Case {
            subject: "a snapshot of a clone",
            pool: Some(POOL),
            dataset: Some("zpcachyos/nosnap/zfsroute-clones/one@auto-2026-11-27"),
            wanted: below_the_parent(),
        },
        Case {
            subject: "a snapshot of the parent itself",
            pool: Some(POOL),
            dataset: Some("zpcachyos/nosnap/zfsroute-clones@manual"),
            wanted: FilterDecision::Passed(PassedWhy::TheParentsOwnDataset),
        },
        Case {
            subject: "a dataset below a clone",
            pool: Some(POOL),
            dataset: Some("zpcachyos/nosnap/zfsroute-clones/one/nested"),
            wanted: below_the_parent(),
        },
        Case {
            subject: "a dataset outside the parent",
            pool: Some(POOL),
            dataset: Some("zpcachyos/ROOT/cos/home"),
            wanted: outside_the_parent("zpcachyos/ROOT/cos/home"),
        },
        Case {
            subject: "a dataset whose name only begins like the parent's",
            pool: Some(POOL),
            dataset: Some("zpcachyos/nosnap/zfsroute-clones-two"),
            wanted: outside_the_parent("zpcachyos/nosnap/zfsroute-clones-two"),
        },
        Case {
            subject: "another pool",
            pool: Some("tank"),
            dataset: Some("tank/nosnap/zfsroute-clones/one"),
            wanted: FilterDecision::Dropped(DroppedWhy::AnotherPool {
                parent_pool: String::from(POOL),
                event_pool: String::from("tank"),
            }),
        },
        Case {
            subject: "a delivery that named no pool",
            pool: None,
            dataset: Some("zpcachyos/nosnap/zfsroute-clones/one"),
            wanted: FilterDecision::Passed(PassedWhy::TheEventNamedNoPool),
        },
        Case {
            subject: "a delivery that named no dataset",
            pool: Some(POOL),
            dataset: None,
            wanted: FilterDecision::Passed(PassedWhy::TheEventNamedNoDataset),
        },
        Case {
            subject: "a delivery that named neither",
            pool: None,
            dataset: None,
            wanted: FilterDecision::Passed(PassedWhy::TheEventNamedNoPool),
        },
    ];
}

/// Every case decides exactly what the design's scope says it does.
#[test]
fn every_scope_case_decides_its_answer() {
    for case in the_cases() {
        let decided: FilterDecision = decision_about(case.pool, case.dataset);
        assert_eq!(decided, case.wanted, "{}", case.subject);
        assert_eq!(
            decided.passes(),
            case.wanted.passes(),
            "{} passes or drops as it says",
            case.subject
        );
    }
}

/// A parent the route could not resolve lets the event through,
/// whatever the delivery names,
/// because the handler's own checks decide on live facts and the filter never
/// fails closed.
#[test]
fn an_unresolved_parent_lets_every_delivery_through() {
    let deliveries: &[(Option<&str>, Option<&str>)] = &[
        (Some(POOL), Some("zpcachyos/ROOT/cos/home")),
        (Some("tank"), Some("tank/other")),
        (None, None),
    ];
    for (pool, dataset) in deliveries {
        let decided: FilterDecision =
            decide_about_the_event(&EventIdentity::of(*pool, *dataset), None);
        assert_eq!(
            decided,
            FilterDecision::Passed(PassedWhy::TheParentCouldNotBeResolved),
            "{pool:?} {dataset:?}"
        );
    }
}

/// A parent name that resolves to nothing is the unresolved case too,
/// so the filter's own reading of the configuration cannot drop a request.
#[test]
fn a_parent_name_that_resolves_to_nothing_is_unresolved() {
    let decided: FilterDecision = decide_about_the_event(
        &EventIdentity::of(Some(POOL), Some("zpcachyos/ROOT")),
        ParentScope::of_dataset(POOL).as_ref(),
    );
    assert_eq!(
        decided,
        FilterDecision::Passed(PassedWhy::TheParentCouldNotBeResolved)
    );
}
