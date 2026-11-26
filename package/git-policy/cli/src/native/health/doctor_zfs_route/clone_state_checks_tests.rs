//! What:
//!  Controls of one slot's state line:
//!  a clone mounted at its slot,
//!  a clone unmounted from it,
//!  a clone a removal left staged,
//!  an add that stopped part-way,
//!  a removal in flight,
//!  and a state the readings cannot prove.
//! Why:
//!  The design asks `doctor` to name unmounted and staged clones and
//!  interrupted actions,
//!  so each arm is shown here with the field a finding carries and the words
//!  that send a person to the right place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkCloneState({ readings, stagingBinds: [] })).toMatchObject({ kind: 'pass' });
//! ```

use super::{
    CloneStateFacts,
    check_clone_state,
    described,
};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::health::doctor_zfs_route::staging_folder_checks::StagingBind;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_slot_state::slot_state_mounts::{
    MountRows,
    SlotMount,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
};
use crate::worktree_clone_slot_state::slot_state_test_records::{
    CLONE_DATASET,
    POOL,
    record_at,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::path::{
    Path,
    PathBuf,
};

/// The slot every control here is about.
const SLOT: &str = "topic";

/// The checkout every slot path hangs under.
const CHECKOUT: &str = "/repo";

/// The account's cache,
/// where a removal's staging folders stand.
const CACHE: &str = "/home/user/.cache";

/// The slot's whole path.
fn slot_path() -> PathBuf {
    return PathBuf::from(format!("{CHECKOUT}/.claude/worktrees/{SLOT}"));
}

/// The record a slot of one phase holds.
fn record(phase: Phase) -> BindingReading {
    return BindingReading::Record(Box::new(record_at(phase, slot_path().as_path())));
}

/// One slot's readings,
/// with the mount table and the intent a control chooses.
fn readings(
    binding: BindingReading,
    rows: Vec<MountedDatasetInstance>,
    slot_mount: SlotMount,
    intent: IntentFacts,
    plan: PlanReading,
) -> SlotReadings {
    let registration_stands: Option<bool> = match &binding {
        BindingReading::Record(record) => record.registration_path().map(|_path| return true),
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => None,
    };
    return SlotReadings {
        slot: String::from(SLOT),
        slot_path: slot_path(),
        binding,
        plan,
        intent,
        mounts: MountRows::Rows(rows),
        slot_mount,
        registration_stands,
    };
}

/// A row of the clone mounted at one path.
fn clone_row(mount_point: &str) -> MountedDatasetInstance {
    return MountedDatasetInstance {
        dataset: String::from(CLONE_DATASET),
        mount_point: PathBuf::from(mount_point),
        root: PathBuf::from("/"),
    };
}

/// The facts a control asks about,
/// with no staging row unless it names one.
fn facts(
    readings: SlotReadings,
    staging_binds: Vec<StagingBind>,
) -> CloneStateFacts {
    return CloneStateFacts {
        readings,
        staging_binds,
    };
}

/// The staging bind one removal leaves for this clone.
fn staging_bind_at(folder: &str) -> StagingBind {
    return StagingBind {
        mount_point: PathBuf::from(folder),
        source: String::from(CLONE_DATASET),
        sharing: Vec::new(),
    };
}

/// A clone mounted at its slot passes,
/// with its dataset and pool.
#[test]
fn a_mounted_clone_passes() {
    let held: SlotReadings = readings(
        record(Phase::Filled),
        vec![clone_row(slot_path().to_string_lossy().as_ref())],
        SlotMount::Something {
            described: String::from("a zfs mount of the clone"),
        },
        IntentFacts::Absent,
        PlanReading::NoPlan,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("a mounted clone passes: {answer:?}");
    };
    assert!(detail.contains("mounted managed clone"), "{detail}");
    assert!(detail.contains(CLONE_DATASET), "{detail}");
    assert!(detail.contains(POOL), "{detail}");
    assert!(detail.contains("next safe action"), "{detail}");
}

/// A clone no row mounts anywhere passes as an unmounted clone,
/// and says the checkout's files live on its dataset.
#[test]
fn an_unmounted_clone_passes_and_names_its_dataset() {
    let held: SlotReadings = readings(
        record(Phase::Filled),
        Vec::new(),
        SlotMount::Nothing,
        IntentFacts::Unmounting {
            nonce: String::from("0123456789abcdef0123456789abcdef"),
        },
        PlanReading::NoPlan,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("an unmounted clone passes: {answer:?}");
    };
    assert!(detail.contains("known unmounted clone"), "{detail}");
    assert!(detail.contains(CLONE_DATASET), "{detail}");
    assert!(detail.contains("no tracked file is deleted"), "{detail}");
}

/// A clone a removal left mounted in a staging folder is a finding that names
/// the folder,
/// and says the release has no removal.
#[test]
fn a_staged_clone_is_a_finding_naming_the_folder() {
    let folder: String = format!("{CACHE}/monochromatic-git-policy-cli-staging-1837462836473829");
    let held: SlotReadings = readings(
        record(Phase::Filled),
        vec![clone_row(folder.as_str())],
        SlotMount::Nothing,
        IntentFacts::Unmounting {
            nonce: String::from("0123456789abcdef0123456789abcdef"),
        },
        PlanReading::NoPlan,
    );
    let answer: CheckOutcome =
        check_clone_state(&facts(held, vec![staging_bind_at(folder.as_str())]));
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a staged clone is a finding: {answer:?}");
    };
    assert_eq!(field, "staging");
    assert!(detail.contains(folder.as_str()), "{detail}");
    assert!(detail.contains("no removal for a clone"), "{detail}");
}

/// A clone mounted somewhere that is no staging folder stays unproven,
/// because a person may have moved it and only the staging rows say a removal
/// did.
#[test]
fn a_clone_mounted_elsewhere_is_unknown() {
    let held: SlotReadings = readings(
        record(Phase::Filled),
        vec![clone_row("/somewhere/else")],
        SlotMount::Nothing,
        IntentFacts::Absent,
        PlanReading::NoPlan,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("a clone mounted elsewhere is unknown: {answer:?}");
    };
    assert!(reason.contains("mounted somewhere else"), "{reason}");
}

/// An add that stopped part-way is a finding naming the phase,
/// with the add's own resume action.
#[test]
fn an_unfinished_add_is_a_finding_naming_the_phase() {
    let held: SlotReadings = readings(
        record(Phase::Settling),
        Vec::new(),
        SlotMount::Nothing,
        IntentFacts::Unasked,
        PlanReading::NoPlan,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("an unfinished add is a finding: {answer:?}");
    };
    assert_eq!(field, "phase");
    assert!(detail.contains("settling"), "{detail}");
    assert!(detail.contains("settle-worktree-clone-record"), "{detail}");
}

/// A removal in flight is a finding naming the plan.
#[test]
fn a_removal_in_flight_is_a_finding_naming_the_plan() {
    let plan: PathBuf =
        PathBuf::from("/repo/.git/monochromatic-git-policy-cli/plans/4821193048571903102");
    let held: SlotReadings = readings(
        record(Phase::Filled),
        Vec::new(),
        SlotMount::Nothing,
        IntentFacts::Unmounting {
            nonce: String::from("0123456789abcdef0123456789abcdef"),
        },
        PlanReading::NamesTheClone {
            plan: plan.clone(),
        },
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a removal in flight is a finding: {answer:?}");
    };
    assert_eq!(field, "plan");
    assert!(
        detail.contains(plan.display().to_string().as_str()),
        "{detail}"
    );
}

/// A record no reader may act on leaves the state unknown,
/// and never passes for the unmounted clone it might be.
#[test]
fn an_unreadable_record_is_unknown() {
    let held: SlotReadings = readings(
        BindingReading::Unreadable {
            reason: String::from("the record is 9,300 bytes"),
        },
        Vec::new(),
        SlotMount::Nothing,
        IntentFacts::Unasked,
        PlanReading::NotAsked,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("an unreadable record is unknown: {answer:?}");
    };
    assert!(reason.contains("cannot be proved"), "{reason}");
    assert!(reason.contains("9,300 bytes"), "{reason}");
}

/// A slot the route holds nothing in passes,
/// and says an ordinary worktree may be added there.
#[test]
fn an_empty_slot_passes() {
    let held: SlotReadings = readings(
        BindingReading::Absent,
        Vec::new(),
        SlotMount::Nothing,
        IntentFacts::Unasked,
        PlanReading::NotAsked,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("an empty slot passes: {answer:?}");
    };
    assert!(detail.contains("not a managed clone"), "{detail}");
}

/// The evidence of every source reaches the line,
/// so a person can check the state against what was read.
#[test]
fn the_evidence_of_every_source_reaches_the_line() {
    let held: SlotReadings = readings(
        record(Phase::Filled),
        vec![clone_row(slot_path().to_string_lossy().as_ref())],
        SlotMount::Something {
            described: String::from("a zfs mount of the clone"),
        },
        IntentFacts::Mounting {
            nonce: String::from("0123456789abcdef0123456789abcdef"),
        },
        PlanReading::NoPlan,
    );
    let answer: CheckOutcome = check_clone_state(&facts(held, Vec::new()));
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("a mounted clone passes: {answer:?}");
    };
    for source in ["binding:", "registration:", "plan:", "intent:"] {
        assert!(detail.contains(source), "{source} is missing from {detail}");
    }
}

/// The report's line is the described one,
/// so the check and its helper cannot disagree.
#[test]
fn the_line_is_the_report_described() {
    let held: SlotReadings = readings(
        record(Phase::Filled),
        vec![clone_row(slot_path().to_string_lossy().as_ref())],
        SlotMount::Something {
            described: String::from("a zfs mount of the clone"),
        },
        IntentFacts::Absent,
        PlanReading::NoPlan,
    );
    let report: crate::worktree_clone_slot_state::slot_state_report::SlotStateReport =
        crate::worktree_clone_slot_state::slot_state_report::report_of(&held);
    assert_eq!(
        described(&report),
        match check_clone_state(&facts(held, Vec::new())) {
            CheckOutcome::Pass {
                detail,
            } => detail,
            other => panic!("a mounted clone passes: {other:?}"),
        }
    );
    assert_eq!(
        Path::new(slot_path().as_path())
            .file_name()
            .map(|name| return name.to_string_lossy().into_owned()),
        Some(String::from(SLOT))
    );
}
