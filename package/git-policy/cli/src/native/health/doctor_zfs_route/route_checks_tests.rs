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

use crate::health::doctor_zfs_route::check_outcome::CheckReport;
use super::{RouteFacts, route_checks};
use crate::health::doctor_zfs_route::route_test_facts::{healthy_route_facts, DatasetBuilder, CHECKOUT_DATASET};
use crate::health::doctor_zfs_route::zfs_checks::{Reading, reading_of};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use std::collections::BTreeMap;

/// The healthy route produces no finding,
/// and the only unknowns are the five checks later steps owe.
#[test]
fn the_healthy_route_has_no_finding() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    for report in &reports {
        assert!(!report.found(), "the healthy route found something: {}", report.line());
    }
    let unknown: Vec<&'static str> = reports.iter().filter(|report| return report.unknown()).map(|report| return report.check.name()).collect();
    assert_eq!(unknown, vec!["deferred-origin", "removal-plans", "cancel-command", "backup-coverage", "sanoid"], "{unknown:?}");
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
            "abi", "backup-coverage", "cancel-command", "checkout-dataset", "checkout-mount", "checkout-propagation", "clone-properties", "container-dataset", "deferred-origin", "delegation", "device", "handler-configuration",
            "handler-folders", "handler-installation", "holders", "key", "nested-reachability", "parent-identity", "pool", "readiness", "removal-plans", "root-modes", "sanoid", "staging-bind-propagation", "staging-folder", "staging-forms",
            "store-entries", "unknown-dot-entry", "unresolved-requests", "zed-wiring",
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
                return crate::health::doctor_zfs_route::route_test_facts::dataset(crate::health::doctor_zfs_route::zfs_checks::DatasetRole::Checkout { mount_point: std::path::PathBuf::from(crate::health::doctor_zfs_route::route_test_facts::CHECKOUT) }, DatasetBuilder::checkout().mountpoint("/elsewhere").build());
            }
            return dataset;
        })
        .collect();
    let reports: Vec<CheckReport> = route_checks(&facts);
    let findings: Vec<&CheckReport> = reports.iter().filter(|report| return report.found()).collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].check.name(), "checkout-dataset");
}

/// A pool read that answered `ENOENT` is a finding,
/// not an unknown,
/// from the whole report as from the one check.
#[test]
fn a_gone_pool_is_a_finding_in_the_whole_report() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.pool = crate::health::doctor_zfs_route::zfs_checks::PoolFacts { pool: String::from("zpcachyos"), reading: reading_of(Err(ReadFailure { said: String::from("dataset does not exist"), errno: Some(2) })) };
    let reports: Vec<CheckReport> = route_checks(&facts);
    let pool: &CheckReport = reports.iter().find(|report| return report.check.name() == "pool").expect("the pool check reports");
    assert!(pool.found(), "{}", pool.line());
}

/// The three groups of the step all appear:
/// the facts about the pool,
/// the recorded identities and the store,
/// and the delegation,
/// the handler,
/// the staging folder and the holders.
#[test]
fn the_three_groups_of_the_step_all_appear() {
    let reports: Vec<CheckReport> = route_checks(&healthy_route_facts());
    let names: Vec<&'static str> = reports.iter().map(|report| return report.check.name()).collect();
    for expected in ["device", "abi", "pool", "checkout-dataset", "container-dataset", "key", "clone-properties", "parent-identity", "store-entries", "staging-forms", "unknown-dot-entry", "delegation", "handler-installation", "zed-wiring", "readiness", "unresolved-requests", "staging-folder", "staging-bind-propagation", "holders"] {
        assert!(names.contains(&expected), "{expected} is in the report's names: {names:?}");
    }
}

/// A slot record adds one slot-identity line,
/// so the report follows the store's set rather than a fixed count.
#[test]
fn a_slot_adds_one_slot_identity_line() {
    let mut facts: RouteFacts = healthy_route_facts();
    assert!(!route_checks(&facts).iter().any(|report| return report.check.name() == "slot-identity"));
    facts.slots.push(crate::health::doctor_zfs_route::identity_checks::SlotIdentityFacts { slot: String::from("one"), refusal: None });
    let reports: Vec<CheckReport> = route_checks(&facts);
    let slots: Vec<&CheckReport> = reports.iter().filter(|report| return report.check.name() == "slot-identity").collect::<Vec<&CheckReport>>();
    assert_eq!(slots.len(), 1);
    assert!(slots[0].passed());
}

/// A reading's failure is a value,
/// so the report is built from facts rather than from a host.
#[test]
fn the_report_is_built_from_facts_alone() {
    let mut facts: RouteFacts = healthy_route_facts();
    facts.datasets.retain(|dataset| return dataset.name != CHECKOUT_DATASET);
    facts.datasets.push(crate::health::doctor_zfs_route::zfs_checks::DatasetFacts { role: crate::health::doctor_zfs_route::zfs_checks::DatasetRole::Checkout { mount_point: std::path::PathBuf::from(crate::health::doctor_zfs_route::route_test_facts::CHECKOUT) }, name: String::from(CHECKOUT_DATASET), reading: Reading::Unreadable { why: String::from("permission denied") } });
    let reports: Vec<CheckReport> = route_checks(&facts);
    let checkout: &CheckReport = reports.iter().find(|report| return report.check.name() == "checkout-dataset").expect("the checkout check reports");
    assert!(checkout.unknown(), "{}", checkout.line());
}
