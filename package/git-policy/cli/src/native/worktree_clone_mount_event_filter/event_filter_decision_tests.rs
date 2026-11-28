//! What:
//!  Controls of the filter's answer:
//!  a pass says which of the route's own cases it fell in,
//!  and a drop names both sides of the comparison it made and says that no
//!  pass ran and that nothing was opened.
//! Why:
//!  A dropped event is a trigger that did not arrive,
//!  so the line is the only thing a person has to read when a clone waits for
//!  one
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  veto entry V2;
//!  "What doctor reports for a clone whose last event was lost").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(Dropped(line).line()).toContain('no pass ran');
//! ```

use super::{
    DroppedWhy,
    FilterDecision,
    PassedWhy,
};

/// The clone parent the drop lines were built for.
const PARENT: &str = "zpcachyos/nosnap/zfsroute-clones";

/// A drop is not a pass.
#[test]
fn a_drop_is_not_a_pass() {
    let dropped: FilterDecision = FilterDecision::Dropped(DroppedWhy::OutsideTheParent {
        parent: String::from(PARENT),
        dataset: String::from("zpcachyos/nosnap/home"),
    });
    assert!(!dropped.passes());
    let passed: FilterDecision = FilterDecision::Passed(PassedWhy::BelowTheParent);
    assert!(passed.passes());
}

/// A drop of another pool's event names both pools and says nothing ran.
#[test]
fn a_drop_of_another_pools_event_names_both_pools() {
    let dropped: FilterDecision = FilterDecision::Dropped(DroppedWhy::AnotherPool {
        parent_pool: String::from("zpcachyos"),
        event_pool: String::from("tank"),
    });
    let line: String = dropped.line();
    assert!(line.contains("`tank`"), "the event's pool is named: {line}");
    assert!(
        line.contains("`zpcachyos`"),
        "the parent's pool is named: {line}"
    );
    assert!(
        line.contains("no pass ran"),
        "the line says no pass ran: {line}"
    );
    assert!(
        line.contains("`/dev/zfs` was not opened"),
        "the line says nothing was opened: {line}"
    );
    assert_eq!(
        line.lines().count(),
        1,
        "a drop is one line a journal holds: {line}"
    );
}

/// A drop outside the parent names the parent and the dataset it saw.
#[test]
fn a_drop_outside_the_parent_names_both_datasets() {
    let dropped: FilterDecision = FilterDecision::Dropped(DroppedWhy::OutsideTheParent {
        parent: String::from(PARENT),
        dataset: String::from("zpcachyos/nosnap/zfsroute-clones-two"),
    });
    let line: String = dropped.line();
    assert!(line.contains(PARENT), "the parent is named: {line}");
    assert!(
        line.contains("zfsroute-clones-two"),
        "the dataset the event named is in the line: {line}"
    );
    assert!(
        line.contains("no pass ran"),
        "the line says no pass ran: {line}"
    );
    assert_eq!(line.lines().count(), 1, "a drop is one line: {line}");
}

/// Every pass names what the delivery proved,
/// and no pass claims that anything was dropped.
#[test]
fn every_pass_line_names_its_own_case() {
    let cases: &[(PassedWhy, &str)] = &[
        (
            PassedWhy::TheParentsOwnDataset,
            "the clone parent dataset itself",
        ),
        (
            PassedWhy::BelowTheParent,
            "a dataset below the clone parent",
        ),
        (
            PassedWhy::TheParentCouldNotBeResolved,
            "the handler's own checks decide",
        ),
        (PassedWhy::TheEventNamedNoPool, "named no pool"),
        (PassedWhy::TheEventNamedNoDataset, "named no dataset"),
    ];
    for (why, named) in cases {
        let line: String = FilterDecision::Passed(*why).line();
        assert!(
            line.contains(named),
            "{why:?} names its case with {named:?}: {line}"
        );
        assert!(!line.contains("dropped"), "{why:?} claims no drop: {line}");
        assert_eq!(line.lines().count(), 1, "{why:?} is one line: {line}");
    }
}
