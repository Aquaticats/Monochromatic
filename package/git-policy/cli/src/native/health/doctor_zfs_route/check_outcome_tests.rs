//! What:
//!  Controls of one check's answer and the line it prints.
//! Why:
//!  The line's shape is what a person reads and a control greps,
//!  and the three states must stay apart in it,
//!  so the shape is proved on built answers rather than on a run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(reportLine({ check: 'device', outcome: pass('opened') })).toBe('zfs: device: pass: opened');
//! ```

use super::{Check, CheckOutcome, CheckReport, finding, pass, unknown};
use std::collections::BTreeSet;

/// A pass prints its detail after the state.
#[test]
fn a_pass_line_carries_what_the_check_read() {
    let report: CheckReport = CheckReport::new(Check::Device, pass("/dev/zfs is there"));
    assert_eq!(report.line(), "zfs: device: pass: /dev/zfs is there");
    assert!(report.passed());
    assert!(!report.found());
    assert!(!report.unknown());
}

/// A finding prints the failing field before its detail.
#[test]
fn a_finding_line_carries_the_failing_field() {
    let report: CheckReport = CheckReport::new(Check::ContainerDataset, finding("mountpoint", "the container is not legacy"));
    assert_eq!(report.line(), "zfs: container-dataset: finding: mountpoint: the container is not legacy");
    assert!(report.found());
    assert!(!report.passed());
}

/// An unknown prints its reason.
#[test]
fn an_unknown_line_carries_the_reason() {
    let report: CheckReport = CheckReport::new(Check::Abi, unknown("/dev/zfs could not be opened"));
    assert_eq!(report.line(), "zfs: abi: unknown: /dev/zfs could not be opened");
    assert!(report.unknown());
    assert!(!report.found());
}

/// Every check's name is its own,
/// so one report line names one check.
#[test]
fn every_check_name_is_unique() {
    let checks: [Check; 41] = [
        Check::Device,
        Check::Abi,
        Check::Pool,
        Check::CheckoutDataset,
        Check::ContainerDataset,
        Check::CheckoutSnapdir,
        Check::ContainerSnapdir,
        Check::Key,
        Check::CloneProperties,
        Check::CheckoutMount,
        Check::CheckoutPropagation,
        Check::NestedReachability,
        Check::ParentIdentity,
        Check::SlotIdentity,
        Check::StoreEntries,
        Check::StagingForms,
        Check::UnknownDotEntry,
        Check::Delegation,
        Check::HandlerInstallation,
        Check::RootModes,
        Check::HandlerConfiguration,
        Check::HandlerFolders,
        Check::ZedWiring,
        Check::Readiness,
        Check::UnresolvedRequests,
        Check::StagingFolder,
        Check::StagingBindPropagation,
        Check::CloneState,
        Check::Registration,
        Check::BindingRecord,
        Check::RemovalPlan,
        Check::CloneSnapshots,
        Check::CloneOrigin,
        Check::CloneRetention,
        Check::BlockedRemoval,
        Check::Holders,
        Check::CancelCommand,
        Check::BootProtection,
        Check::ServiceUnit,
        Check::BackupCoverage,
        Check::Sanoid,
    ];
    let names: BTreeSet<&'static str> = checks.iter().map(|check| return check.name()).collect();
    assert_eq!(names.len(), checks.len(), "two checks share a name: {names:?}");
}

/// A finding's field is a word or a component,
/// never empty,
/// so a line always says what failed.
#[test]
fn a_finding_field_is_never_empty() {
    let outcome: CheckOutcome = finding("mode", "the folder is 0755");
    let CheckOutcome::Finding { field, .. } = outcome else {
        panic!("the constructor builds a finding");
    };
    assert!(!field.is_empty());
}
