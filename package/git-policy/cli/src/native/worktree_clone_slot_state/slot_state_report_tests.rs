//! Controls for one slot's report:
//!  one report for each state the classification names,
//!  with its slot, its dataset and pool,
//!  the evidence each state came from,
//!  and the one next safe action,
//!  plus the lines a command prints.

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_slot_state::slot_state_mounts::{MountRows, SlotMount};
use crate::worktree_clone_slot_state::slot_state_readings::{BindingReading, IntentFacts, PlanReading, SlotReadings};
use crate::worktree_clone_slot_state::slot_state_report::{NO_DATASET, SlotStateReport, report_of};
use crate::worktree_clone_slot_state::slot_state_test_records::{CLONE_DATASET, POOL, record_at};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::path::PathBuf;

/// The fixture's slot name.
fn slot() -> &'static str {
    return "topic";
}

/// The fixture's slot path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// The readings of a record at one phase with nothing mounted.
fn readings_at(phase: Phase) -> SlotReadings {
    return SlotReadings {
        slot: String::from(slot()),
        slot_path: slot_path(),
        binding: BindingReading::Record(Box::new(record_at(phase, slot_path().as_path()))),
        plan: PlanReading::NoPlan,
        intent: IntentFacts::Absent,
        mounts: MountRows::Rows(Vec::new()),
        slot_mount: SlotMount::Nothing,
        registration_stands: None,
    };
}

/// A report line, found by its prefix.
fn line_starting(report: &SlotStateReport, prefix: &str) -> String {
    return report
        .lines()
        .into_iter()
        .find(|line| return line.starts_with(prefix))
        .unwrap_or_else(|| panic!("the report holds no line starting `{prefix}`: {:?}", report.lines()));
}

/// A slot the route holds nothing in reports no dataset and one action.
#[test]
fn a_slot_with_nothing_reports_no_dataset() {
    let readings: SlotReadings = SlotReadings {
        binding: BindingReading::Absent,
        ..readings_at(Phase::Claimed)
    };
    let report: SlotStateReport = report_of(&readings);
    assert_eq!(report.state, "not a managed clone");
    assert_eq!(report.dataset, NO_DATASET);
    assert_eq!(report.pool, NO_DATASET);
    assert!(report.lines().contains(&String::from("dataset: none")), "{:?}", report.lines());
    assert!(report.next_safe_action.contains("ordinary worktree"), "{}", report.next_safe_action);
    assert!(line_starting(&report, "evidence: binding:").contains("no record is filed"), "{:?}", report.lines());
}

/// A pending add names its dataset and pool and tells the person not to add here.
#[test]
fn a_pending_add_reports_its_dataset_and_the_add_again() {
    let report: SlotStateReport = report_of(&readings_at(Phase::Snapshotted));
    assert_eq!(report.state, "pending add");
    assert_eq!(report.dataset, CLONE_DATASET);
    assert_eq!(report.pool, POOL);
    assert!(report.next_safe_action.contains("do not run Git in this folder"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains("Run the add again at `topic`"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains("phase=snapshotted"), "{}", report.next_safe_action);
    assert!(line_starting(&report, "evidence: binding:").contains("phase=snapshotted"), "{:?}", report.lines());
}

/// A settling binding reports the fallback and every command its state can take.
#[test]
fn a_settling_binding_reports_its_settle_reason_and_commands() {
    let report: SlotStateReport = report_of(&readings_at(Phase::Settling));
    assert_eq!(report.state, "pending add");
    assert!(report.next_safe_action.contains("phase=settling"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains("settle reason `enospc`"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains("settle-worktree-clone-record"), "{}", report.next_safe_action);
    assert!(line_starting(&report, "evidence: binding:").contains("settleReason=enospc"), "{:?}", report.lines());
}

/// A pending removal names its plan and the removal that ends it.
#[test]
fn a_pending_removal_reports_the_plan() {
    let plan: PathBuf = PathBuf::from("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/4821193048571903102");
    let readings: SlotReadings = SlotReadings {
        plan: PlanReading::NamesTheClone { plan: plan.clone() },
        ..readings_at(Phase::Mounted)
    };
    let report: SlotStateReport = report_of(&readings);
    assert_eq!(report.state, "pending removal");
    assert_eq!(report.dataset, CLONE_DATASET);
    assert!(report.next_safe_action.contains("resumes from the plan and ends it"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains(plan.to_str().expect("utf8 plan")), "{}", report.next_safe_action);
}

/// A known unmounted clone says where the checkout lives and what each intent means.
#[test]
fn a_known_unmounted_clone_reports_its_files_and_each_intent() {
    let absent: SlotStateReport = report_of(&readings_at(Phase::Filled));
    assert_eq!(absent.state, "known unmounted clone");
    assert_eq!(absent.dataset, CLONE_DATASET);
    assert!(absent.next_safe_action.contains("nothing under the slot is a person's file"), "{}", absent.next_safe_action);
    assert!(absent.next_safe_action.contains("no tracked file is deleted"), "{}", absent.next_safe_action);
    assert!(absent.next_safe_action.contains("No mount request stands"), "{}", absent.next_safe_action);
    assert!(absent.next_safe_action.contains("zfs set org.monochromatic:git-worktree-clone-mount-desired-state"), "{}", absent.next_safe_action);
    let mounting: SlotReadings = SlotReadings {
        intent: IntentFacts::Mounting { nonce: String::from("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f") },
        ..readings_at(Phase::Filled)
    };
    let waited: String = report_of(&mounting).next_safe_action;
    assert!(waited.contains("no sweep has attached it yet"), "{waited}");
    assert!(waited.contains("reconcile-git-worktree-clone-mounts.service"), "{waited}");
    let unmounting: SlotReadings = SlotReadings {
        intent: IntentFacts::Unmounting { nonce: String::from("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f") },
        ..readings_at(Phase::Filled)
    };
    let parked: String = report_of(&unmounting).next_safe_action;
    assert!(parked.contains("keeps it unmounted"), "{parked}");
    assert!(parked.contains("run the removal"), "{parked}");
}

/// A mounted managed clone reports the mount and the fixed slot.
#[test]
fn a_mounted_clone_reports_the_mount_and_the_fixed_slot() {
    let mounted: MountedDatasetInstance = MountedDatasetInstance { dataset: String::from(CLONE_DATASET), mount_point: slot_path(), root: PathBuf::from("/") };
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Rows(vec![mounted]),
        slot_mount: SlotMount::Something { described: String::from("the clone at the slot") },
        ..readings_at(Phase::Filled)
    };
    let report: SlotStateReport = report_of(&readings);
    assert_eq!(report.state, "mounted managed clone");
    assert!(report.next_safe_action.contains("is mounted at its slot"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains("`git worktree move` is refused"), "{}", report.next_safe_action);
    assert!(line_starting(&report, "evidence: mount:").contains("the clone's own root stands at its slot"), "{:?}", report.lines());
}

/// A state nothing proves reports the reason and says to change nothing.
#[test]
fn a_state_nothing_proves_reports_its_reason() {
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Unreadable { reason: String::from("the table is too large") },
        ..readings_at(Phase::Filled)
    };
    let report: SlotStateReport = report_of(&readings);
    assert_eq!(report.state, "cannot be proved");
    assert!(report.next_safe_action.contains("the table is too large"), "{}", report.next_safe_action);
    assert!(report.next_safe_action.contains("Do not add here"), "{}", report.next_safe_action);
    assert!(line_starting(&report, "evidence: mount table:").contains("the table is too large"), "{:?}", report.lines());
}

/// The printed lines carry the slot, the state, the dataset, the evidence and the action once.
#[test]
fn the_lines_carry_each_field_once() {
    let lines: Vec<String> = report_of(&readings_at(Phase::Filled)).lines();
    assert_eq!(lines.first().map(|line| return line.as_str()), Some("slot `topic`"));
    assert_eq!(lines.get(1).map(|line| return line.as_str()), Some("state: known unmounted clone"));
    let dataset_line: String = format!("dataset: `{CLONE_DATASET}` in pool `{POOL}`");
    assert_eq!(lines.get(2).map(|line| return line.as_str()), Some(dataset_line.as_str()));
    assert_eq!(lines.iter().filter(|line| return line.starts_with("next safe action: ")).count(), 1);
    assert!(lines.iter().any(|line| return line == "evidence: intent: no request of the clone's own"));
    assert_eq!(lines.last().map(|line| return line.starts_with("next safe action: ")), Some(true));
}
