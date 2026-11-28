//! Controls for one slot's state:
//!  a slot the route holds nothing in,
//!  a pending add at each unfinished phase,
//!  a pending removal,
//!  a known unmounted clone,
//!  a mounted managed clone,
//!  and each shape no reading proves.

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_slot_state::slot_state_classification::{
    ProofStoppedBy,
    SlotRouteState,
    classify_slot,
};
use crate::worktree_clone_slot_state::slot_state_mounts::{
    CloneMount,
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

/// The fixture's slot name.
fn slot() -> &'static str {
    return "topic";
}

/// The fixture's slot path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// One clone row at the slot.
fn a_clone_row() -> MountedDatasetInstance {
    return MountedDatasetInstance {
        dataset: String::from(CLONE_DATASET),
        mount_point: slot_path(),
        root: PathBuf::from("/"),
    };
}

/// The readings of a record at one phase with nothing mounted,
/// no plan and no intent.
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

/// A slot with no record and nothing mounted is not a managed clone.
#[test]
fn no_record_and_no_mount_is_not_a_managed_clone() {
    let readings: SlotReadings = SlotReadings {
        binding: BindingReading::Absent,
        plan: PlanReading::NotAsked,
        ..readings_at(Phase::Claimed)
    };
    assert_eq!(classify_slot(&readings), SlotRouteState::NotAManagedClone);
}

/// Every phase before the clone was mounted is a pending add.
#[test]
fn each_unfinished_add_phase_is_a_pending_add() {
    let phases: [Phase; 5] = [
        Phase::Claimed,
        Phase::Slot,
        Phase::Registered,
        Phase::Snapshotted,
        Phase::Cloned,
    ];
    for phase in phases {
        let state: SlotRouteState = classify_slot(&readings_at(phase));
        assert_eq!(
            state,
            SlotRouteState::PendingAdd {
                phase,
                clone_dataset: String::from(CLONE_DATASET)
            },
            "phase `{}`",
            phase.word()
        );
        assert_eq!(state.word(), "pending add");
    }
}

/// A fallback's own phase is the add's pending one too,
/// because the add still owns the slot and no plan stands.
#[test]
fn a_settling_binding_is_a_pending_add() {
    assert_eq!(
        classify_slot(&readings_at(Phase::Settling)),
        SlotRouteState::PendingAdd {
            phase: Phase::Settling,
            clone_dataset: String::from(CLONE_DATASET)
        }
    );
}

/// A plan that names the clone is a pending removal at any phase,
/// which is the design's order.
#[test]
fn a_plan_that_names_the_clone_is_a_pending_removal() {
    let plan: PathBuf = PathBuf::from(
        "/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/4821193048571903102",
    );
    for phase in [Phase::Mounted, Phase::Filled, Phase::Settling] {
        let readings: SlotReadings = SlotReadings {
            plan: PlanReading::NamesTheClone {
                plan: plan.clone(),
            },
            ..readings_at(phase)
        };
        assert_eq!(
            classify_slot(&readings),
            SlotRouteState::PendingRemoval {
                plan: plan.clone(),
                clone_dataset: String::from(CLONE_DATASET)
            },
            "phase `{}`",
            phase.word()
        );
    }
}

/// A filled clone no row mounts is the known unmounted clone.
#[test]
fn a_filled_clone_with_no_row_is_a_known_unmounted_clone() {
    for phase in [Phase::Mounted, Phase::Filled] {
        let state: SlotRouteState = classify_slot(&readings_at(phase));
        assert_eq!(
            state,
            SlotRouteState::KnownUnmountedClone {
                clone_dataset: String::from(CLONE_DATASET)
            },
            "phase `{}`",
            phase.word()
        );
        assert_eq!(state.word(), "known unmounted clone");
    }
}

/// A filled clone whose own root is at its slot is the mounted managed clone.
#[test]
fn a_filled_clone_at_its_slot_is_mounted() {
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Rows(vec![a_clone_row()]),
        slot_mount: SlotMount::Something {
            described: String::from("the clone at the slot"),
        },
        ..readings_at(Phase::Filled)
    };
    assert_eq!(
        classify_slot(&readings),
        SlotRouteState::MountedManagedClone {
            clone_dataset: String::from(CLONE_DATASET)
        }
    );
}

/// A mount standing at the slot while the record says the add never mounted is a disagreement.
#[test]
fn a_mount_the_record_never_verified_cannot_be_proved() {
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Rows(vec![a_clone_row()]),
        slot_mount: SlotMount::Something {
            described: String::from("the clone at the slot"),
        },
        ..readings_at(Phase::Cloned)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(
                reason.contains("phase=cloned") && reason.contains("mounted at the slot"),
                "{reason}"
            );
            assert_eq!(
                stopped_by,
                ProofStoppedBy::AnotherReading,
                "a phase that disagrees with its mount stays Git's"
            );
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// A clone mounted at another path cannot be proved to be the one at its slot.
#[test]
fn a_clone_mounted_elsewhere_cannot_be_proved() {
    let other: MountedDatasetInstance = MountedDatasetInstance {
        dataset: String::from(CLONE_DATASET),
        mount_point: PathBuf::from("/repo/.claude/worktrees/other"),
        root: PathBuf::from("/"),
    };
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Rows(vec![other]),
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(
                reason.contains("elsewhere") || reason.contains("/repo/.claude/worktrees/other"),
                "{reason}"
            );
            assert_eq!(stopped_by, ProofStoppedBy::AnotherReading);
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// A record that cannot be read proves no state.
#[test]
fn an_unreadable_record_cannot_be_proved() {
    let readings: SlotReadings = SlotReadings {
        binding: BindingReading::Unreadable {
            reason: String::from("permission denied"),
        },
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(reason.contains("permission denied"), "{reason}");
            assert_eq!(
                stopped_by,
                ProofStoppedBy::AnotherReading,
                "the removal refuses an unreadable record before the mount table, so the state is no other reading's"
            );
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// A mount at a slot with no record cannot be proved to be the route's.
#[test]
fn a_mount_with_no_record_cannot_be_proved() {
    let readings: SlotReadings = SlotReadings {
        binding: BindingReading::Absent,
        slot_mount: SlotMount::Something {
            described: String::from("`tank/other` at the slot, mounted as `zfs`"),
        },
        ..readings_at(Phase::Claimed)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(reason.contains("nothing proves"), "{reason}");
            assert_eq!(stopped_by, ProofStoppedBy::AMountNoRecordNames);
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// A record filed under one leaf name for another path is not this slot's record.
#[test]
fn a_record_naming_another_path_cannot_be_proved() {
    let readings: SlotReadings = SlotReadings {
        binding: BindingReading::Record(Box::new(record_at(
            Phase::Filled,
            PathBuf::from("/other/repo/.claude/worktrees/topic").as_path(),
        ))),
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(reason.contains("not this slot's record"), "{reason}");
            assert_eq!(stopped_by, ProofStoppedBy::ARecordOfAnotherPath);
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// What:
///  A plan no reader could read is the reading a removal refuses without force
///  and passes with one.
/// Why:
///  The owner answered on 2026-11-28 that such a slot refuses a removal,
///  naming the plan file and the settling command,
///  and that one force goes through it
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
///  "A destruction plan that cannot be read").
///  This control failed on `main`'s own line,
///  which read the state as `AnotherReading` and left the slot to Git.
#[test]
fn an_unreadable_plan_is_a_reading_a_removal_refuses_without_force() {
    let readings: SlotReadings = SlotReadings {
        plan: PlanReading::Unreadable {
            plan: PathBuf::from("/repo/.git/plans/1"),
            reason: String::from("not a plan"),
        },
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(reason.contains("not a plan"), "{reason}");
            assert_eq!(stopped_by, ProofStoppedBy::ADestructionPlanThatCannotBeRead);
            assert!(
                stopped_by.refuses_a_removal(false),
                "a removal without force refuses a plan it cannot read"
            );
            assert!(
                !stopped_by.refuses_a_removal(true),
                "one force goes through a plan it cannot read"
            );
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// What:
///  A clone mounted on a staging folder with no plan is the second reading a
///  removal refuses without force and passes with one.
/// Why:
///  It is where an unmount a killed removal asked for left the clone,
///  and the owner answered on 2026-11-28 that the refusal names
///  `cancel-worktree-removal <slot>` and that one force unmounts it there and
///  destroys it
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
///  "A clone mounted at the staging folder, with no plan").
///  This control failed on `main`'s own line,
///  which read the mount as `Elsewhere` and left the slot to Git.
#[test]
fn a_clone_at_the_staging_folder_is_a_reading_a_removal_refuses_without_force() {
    let staging: PathBuf =
        PathBuf::from("/home/u/.cache/monochromatic-git-policy-cli-staging-7828944690444299989");
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Rows(vec![MountedDatasetInstance {
            dataset: String::from(CLONE_DATASET),
            mount_point: staging.clone(),
            root: PathBuf::from("/"),
        }]),
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(reason.contains("staging folder"), "{reason}");
            assert_eq!(stopped_by, ProofStoppedBy::ACloneAtTheStagingFolder);
            assert!(
                stopped_by.refuses_a_removal(false),
                "a removal without force refuses a clone on the staging folder"
            );
            assert!(
                !stopped_by.refuses_a_removal(true),
                "one force goes through a clone on the staging folder"
            );
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// What:
///  A clone mounted somewhere that is no staging folder stays Git's.
/// Why:
///  The owner's answer of 2026-11-28 names the staging folder and no other
///  mount,
///  and a mount a person made elsewhere is not this route's to refuse.
#[test]
fn a_clone_mounted_elsewhere_is_still_no_removals_state() {
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Rows(vec![MountedDatasetInstance {
            dataset: String::from(CLONE_DATASET),
            mount_point: PathBuf::from("/mnt/backup/topic"),
            root: PathBuf::from("/"),
        }]),
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            stopped_by,
            ..
        } => {
            assert_eq!(stopped_by, ProofStoppedBy::AnotherReading);
            assert!(
                !stopped_by.refuses_a_removal(false),
                "a person's mount stays Git's"
            );
            assert!(
                !stopped_by.refuses_a_removal(true),
                "and no force makes it this route's"
            );
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// An unreadable mount table proves nothing about a clone the record calls mounted.
#[test]
fn an_unreadable_mount_table_cannot_be_proved() {
    let readings: SlotReadings = SlotReadings {
        mounts: MountRows::Unreadable {
            reason: String::from("the table is too large"),
        },
        ..readings_at(Phase::Filled)
    };
    match classify_slot(&readings) {
        SlotRouteState::CannotProve {
            reason,
            stopped_by,
        } => {
            assert!(reason.contains("the table is too large"), "{reason}");
            assert_eq!(stopped_by, ProofStoppedBy::AMountThatCannotBeRead);
            assert!(stopped_by.refuses_a_removal(false));
            assert!(
                stopped_by.refuses_a_removal(true),
                "no force passes a mount table nobody could read"
            );
        }
        found => panic!("expected a state nothing proves, got {found:?}"),
    }
}

/// The pool name the record names is the report's,
/// and the state's dataset is the record's clone.
#[test]
fn the_state_names_the_records_clone_dataset() {
    let readings: SlotReadings = readings_at(Phase::Filled);
    match &readings.binding {
        BindingReading::Record(record) => {
            assert_eq!(record.pool, POOL);
            assert_eq!(record.clone_dataset, CLONE_DATASET);
        }
        found => panic!("expected the fixture record, got {found:?}"),
    }
    assert!(matches!(CloneMount::Nowhere, CloneMount::Nowhere));
}

/// A path's last component names the slot,
/// and a path no record can be keyed by says why.
#[test]
fn a_paths_slot_name_is_its_last_component() {
    use crate::worktree_clone_slot_state::slot_state_classification::slot_name_of;
    assert_eq!(
        slot_name_of(Path::new("/repo/.claude/worktrees/topic")).expect("the slot name reads"),
        "topic"
    );
    let root: String = slot_name_of(Path::new("/")).expect_err("a root names no slot");
    assert!(root.contains("names no folder"), "{root}");
    let spaced: String = slot_name_of(Path::new("/repo/.claude/worktrees/two words"))
        .expect_err("a name with a space is no slot");
    assert!(spaced.contains("not a plain slot name"), "{spaced}");
}
