//! What:
//!  Controls of the whole report:
//!  the healthy route has no finding,
//!  every check the step names appears exactly once,
//!  and one changed fact moves exactly one answer.
//! Why:
//!  The report is what the wiring run prints,
//!  so its order and completeness are the piece's own contract:
//!  a check left out is a fact nobody sees,
//!  and a second report for one check is a line a person reads twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reports = route_checks(healthyRouteFacts());
//! expect(reports.filter((entry) => entry.check === 'device')).toHaveLength(1);
//! ```

use super::{
    RouteFacts,
    route_checks,
};
use crate::health::doctor_zfs_route::check_outcome::CheckReport;
use crate::health::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    DatasetBuilder,
    healthy_route_facts,
};
use crate::health::doctor_zfs_route::zfs_checks::{
    Reading,
    reading_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use std::collections::BTreeMap;

/// The healthy route produces no finding,
/// and the only unknown is the backup coverage no arm reads yet.
#[test]
fn the_healthy_route_has_no_finding() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    for report in &reports {
        assert!(
            !report.found(),
            "the healthy route found something: {}",
            report.line()
        );
    }
    let unknown: Vec<&'static str> = reports
        .iter()
        .filter(|report| return report.unknown())
        .map(|report| return report.check.name())
        .collect();
    assert_eq!(unknown, vec!["backup-coverage"], "{unknown:?}");
}

/// Every check the step names appears exactly once.
#[test]
fn every_check_appears_exactly_once() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for report in &reports {
        *counts.entry(report.check.name()).or_default() += 1;
    }
    for (name, count) in &counts {
        assert_eq!(*count, 1, "{name} appears {count} times");
    }
    let names: Vec<&'static str> = counts.keys().copied().collect();
    assert_eq!(
        names,
        vec![
            "abi",
            "backup-coverage",
            "binding-record",
            "blocked-removal",
            "boot-protection",
            "cancel-command",
            "checkout-dataset",
            "checkout-mount",
            "checkout-propagation",
            "checkout-snapdir",
            "clone-origin",
            "clone-properties",
            "clone-retention",
            "clone-snapshots",
            "clone-state",
            "container-dataset",
            "container-snapdir",
            "delegation",
            "device",
            "handler-configuration",
            "handler-folders",
            "handler-installation",
            "holders",
            "key",
            "managed-lock",
            "nested-reachability",
            "parent-identity",
            "pool",
            "readiness",
            "registration",
            "removal-plan",
            "root-modes",
            "sanoid",
            "service-unit",
            "staging-bind-propagation",
            "staging-folder",
            "staging-forms",
            "store-entries",
            "unknown-dot-entry",
            "unresolved-requests",
            "zed-wiring",
        ]
    );
}

/// One broken fact moves exactly the answer that judges it.
#[test]
fn one_broken_fact_moves_one_answer() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.datasets = facts
        .datasets
        .into_iter()
        .map(|dataset| {
            if dataset.name == CHECKOUT_DATASET {
                return crate::health::doctor_zfs_route::route_test_facts::dataset(
                    crate::health::doctor_zfs_route::zfs_checks::DatasetRole::Checkout {
                        mount_point: std::path::PathBuf::from(
                            crate::health::doctor_zfs_route::route_test_facts::CHECKOUT,
                        ),
                    },
                    DatasetBuilder::checkout().mountpoint("/elsewhere").build(),
                );
            }
            return dataset;
        })
        .collect();
    let reports: Vec<CheckReport> = route_checks(&facts);
    let findings: Vec<&CheckReport> = reports
        .iter()
        .filter(|report| return report.found())
        .collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].check.name(), "checkout-dataset");
}

/// A pool read that answered `ENOENT` is a finding,
/// not an unknown,
/// from the whole report as from the one check.
#[test]
fn a_gone_pool_is_a_finding_in_the_whole_report() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.pool = crate::health::doctor_zfs_route::zfs_checks::PoolFacts {
        pool: String::from("zpcachyos"),
        reading: reading_of(Err(ReadFailure {
            said: String::from("dataset does not exist"),
            errno: Some(2),
        })),
    };
    let reports: Vec<CheckReport> = route_checks(&facts);
    let pool: &CheckReport = reports
        .iter()
        .find(|report| return report.check.name() == "pool")
        .expect("the pool check reports");
    assert!(pool.found(), "{}", pool.line());
}

/// The snapshot-directory lines follow the dataset lines they report on,
/// so a person reads one dataset and the property this step adds about it.
#[test]
fn each_snapdir_line_follows_its_dataset() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    let names: Vec<&'static str> = reports
        .iter()
        .map(|report| return report.check.name())
        .collect();
    let at = |wanted: &str| {
        return names
            .iter()
            .position(|name| return *name == wanted)
            .unwrap_or_else(|| panic!("{wanted} is in the report: {names:?}"));
    };
    assert_eq!(
        at("checkout-snapdir"),
        at("checkout-dataset") + 1,
        "{names:?}"
    );
    assert_eq!(
        at("container-snapdir"),
        at("container-dataset") + 1,
        "{names:?}"
    );
}

/// A checkout dataset that lost its `snapdir` moves exactly the snapdir line,
/// so the setting this step adds is judged on its own and not folded into the
/// dataset check.
#[test]
fn a_container_that_can_show_its_snapshots_moves_one_answer() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.datasets = facts
        .datasets
        .into_iter()
        .map(|dataset| {
            if dataset.name == crate::health::doctor_zfs_route::route_test_facts::CONTAINER_DATASET {
                return crate::health::doctor_zfs_route::route_test_facts::dataset(crate::health::doctor_zfs_route::zfs_checks::DatasetRole::Container, DatasetBuilder::container().snapdir(Some(crate::health::doctor_zfs_route::snapshot_directory_checks::SNAPDIR_VISIBLE)).build());
            }
            return dataset;
        })
        .collect();
    let reports: Vec<CheckReport> = route_checks(&facts);
    let findings: Vec<&CheckReport> = reports
        .iter()
        .filter(|report| return report.found())
        .collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].check.name(), "container-snapdir");
}

/// The groups of the step all appear:
/// the facts about the pool,
/// the recorded identities and the store,
/// the delegation,
/// the handler and the staging folder,
/// and each clone's removal-side lines.
#[test]
fn the_three_groups_of_the_step_all_appear() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    let names: Vec<&'static str> = reports
        .iter()
        .map(|report| return report.check.name())
        .collect();
    for expected in [
        "device",
        "abi",
        "pool",
        "checkout-dataset",
        "container-dataset",
        "checkout-snapdir",
        "container-snapdir",
        "key",
        "clone-properties",
        "parent-identity",
        "store-entries",
        "staging-forms",
        "unknown-dot-entry",
        "delegation",
        "handler-installation",
        "zed-wiring",
        "readiness",
        "unresolved-requests",
        "staging-folder",
        "staging-bind-propagation",
        "clone-state",
        "registration",
        "managed-lock",
        "binding-record",
        "removal-plan",
        "clone-snapshots",
        "clone-origin",
        "clone-retention",
        "blocked-removal",
        "holders",
    ] {
        assert!(
            names.contains(&expected),
            "{expected} is in the report's names: {names:?}"
        );
    }
}

/// A second managed clone adds one line of each of its own checks,
/// so the report follows the clones the store lists rather than a fixed count.
#[test]
fn each_clone_adds_its_own_group() {
    let mut facts: RouteFacts = healthy_route_facts();
    let before: usize = route_checks(&facts)
        .iter()
        .filter(|report| return report.check.name() == "clone-state")
        .count();
    assert_eq!(before, 1);
    let mut second = crate::health::doctor_zfs_route::route_test_facts::healthy_clone();
    second.holders.clone =
        String::from("zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/two");
    facts.clones.push(second);
    let reports: Vec<CheckReport> = route_checks(&facts);
    for check in [
        "clone-state",
        "registration",
        "managed-lock",
        "binding-record",
        "removal-plan",
        "clone-snapshots",
        "clone-origin",
        "clone-retention",
        "blocked-removal",
        "holders",
    ] {
        let count: usize = reports
            .iter()
            .filter(|report| return report.check.name() == check)
            .count();
        assert_eq!(count, 2, "{check} appears {count} times");
    }
}

/// A slot record adds one slot-identity line,
/// so the report follows the store's set rather than a fixed count.
#[test]
fn a_slot_adds_one_slot_identity_line() {
    let mut facts: RouteFacts = healthy_route_facts();
    assert!(
        !route_checks(&facts)
            .iter()
            .any(|report| return report.check.name() == "slot-identity")
    );
    facts.slots.push(
        crate::health::doctor_zfs_route::identity_checks::SlotIdentityFacts {
            slot: String::from("one"),
            refusal: None,
        },
    );
    let reports: Vec<CheckReport> = route_checks(&facts);
    let slots: Vec<&CheckReport> = reports
        .iter()
        .filter(|report| return report.check.name() == "slot-identity")
        .collect::<Vec<&CheckReport>>();
    assert_eq!(slots.len(), 1);
    assert!(slots[0].passed());
}

/// A reading's failure is a value,
/// so the report is built from facts rather than from a host.
#[test]
fn the_report_is_built_from_facts_alone() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts
        .datasets
        .retain(|dataset| return dataset.name != CHECKOUT_DATASET);
    facts
        .datasets
        .push(crate::health::doctor_zfs_route::zfs_checks::DatasetFacts {
            role: crate::health::doctor_zfs_route::zfs_checks::DatasetRole::Checkout {
                mount_point: std::path::PathBuf::from(
                    crate::health::doctor_zfs_route::route_test_facts::CHECKOUT,
                ),
            },
            name: String::from(CHECKOUT_DATASET),
            reading: Reading::Unreadable {
                why: String::from("permission denied"),
            },
        });
    let reports: Vec<CheckReport> = route_checks(&facts);
    let checkout: &CheckReport = reports
        .iter()
        .find(|report| return report.check.name() == "checkout-dataset")
        .expect("the checkout check reports");
    assert!(checkout.unknown(), "{}", checkout.line());
}

/// The Sanoid line is the coverage comparison's own answer,
/// so a drift the comparison found is the line a person reads rather than an
/// unknown standing in its place.
#[test]
fn the_sanoid_line_is_the_coverage_answer() {
    let mut facts: RouteFacts = healthy_route_facts();
    let correction: String =
        String::from("correction: set `recursive = yes` under [zpcachyos/home]");
    facts.sanoid = crate::health::doctor_zfs_route::sanoid_report::SanoidReport::new(
        crate::health::doctor_zfs_route::check_outcome::finding(
            "recursion",
            "1 finding(s): recursion",
        ),
        vec![correction.clone()],
    );
    let reports: Vec<CheckReport> = route_checks(&facts);
    let sanoid: &CheckReport = reports
        .iter()
        .find(|report| return report.check.name() == "sanoid")
        .expect("the sanoid check reports");
    assert!(sanoid.found(), "{}", sanoid.line());
    assert!(sanoid.line().contains("recursion"), "{}", sanoid.line());
    assert_eq!(facts.sanoid.lines, vec![correction]);
}

/// A dataset under the checkout left `canmount`-mounted moves the boot
/// setup's line and nothing else,
/// so the property V4 decided is judged on its own.
#[test]
fn a_dataset_under_the_checkout_moves_one_answer() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.boot_setup.nested[0].canmount =
        crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue::Number(
            crate::health::doctor_zfs_route::zfs_checks_limits::CANMOUNT_ON,
        );
    let reports: Vec<CheckReport> = route_checks(&facts);
    let findings: Vec<&CheckReport> = reports
        .iter()
        .filter(|report| return report.found())
        .collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].check.name(), "service-unit");
}

/// An unprotected covered entry moves the boot protection's line,
/// and the line names the field a person repairs.
#[test]
fn an_unprotected_entry_moves_one_answer() {
    let mut facts: RouteFacts = healthy_route_facts();
    if let Ok(entry) = &mut facts.boot_protection.entry {
        entry.immutable = false;
    }
    let reports: Vec<CheckReport> = route_checks(&facts);
    let findings: Vec<&CheckReport> = reports
        .iter()
        .filter(|report| return report.found())
        .collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].check.name(), "boot-protection");
    assert!(
        findings[0].line().contains(": immutable: "),
        "{}",
        findings[0].line()
    );
}

/// The cancel command's line passes on every route,
/// because the only fact it reads is the command this build carries.
#[test]
fn the_cancel_command_line_passes_on_every_route() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.boot_setup.nested.clear();
    let reports: Vec<CheckReport> = route_checks(&facts);
    let command: &CheckReport = reports
        .iter()
        .find(|report| return report.check.name() == "cancel-command")
        .expect("the cancel command reports");
    assert!(command.passed(), "{}", command.line());
    assert!(
        command.line().contains("cancel-worktree-removal <slot>"),
        "{}",
        command.line()
    );
}

/// The four checks the report gained stand where the step lists them,
/// after every clone's group and before nothing else,
/// so a person reads one machine's checks in one order.
#[test]
fn the_step_nineteen_and_twenty_checks_follow_the_clone_groups() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    let names: Vec<&'static str> = reports
        .iter()
        .map(|report| return report.check.name())
        .collect();
    let last_clone: usize = names
        .iter()
        .rposition(|name| return *name == "holders")
        .expect("a clone's group is in the report");
    assert_eq!(
        &names[last_clone + 1..],
        [
            "cancel-command",
            "boot-protection",
            "service-unit",
            "backup-coverage",
            "sanoid"
        ],
        "{names:?}"
    );
}
