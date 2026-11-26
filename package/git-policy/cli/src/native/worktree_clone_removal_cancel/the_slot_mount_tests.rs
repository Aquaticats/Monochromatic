//! What:
//!  Controls for where the clone stands:
//!  at its slot,
//!  nowhere,
//!  elsewhere,
//!  behind something else,
//!  and behind a table that could not be read.
//! Why:
//!  The command writes no second request where the clone already stands at its slot,
//!  and it refuses a `done` answer whose mount table read says the clone stands anywhere else
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The act" section).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(whereTheCloneStands(scriptedTable, 'tank/clone', '/repo/.claude/worktrees/one')).toEqual({ kind: 'at-its-slot' });
//! ```

use super::{
    counts_as_at_its_slot,
    of_a_table,
    word,
};
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountHoldingAPath,
    MountedDatasetInstance,
    MountedDatasets,
};
use std::path::{
    Path,
    PathBuf,
};

/// The clone the controls read.
const CLONE: &str = "tank/clone";

/// The clone's slot.
fn slot() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/one");
}

/// What:
///  A mount table a control writes.
/// Why:
///  No control on a person's machine may read that machine's mounts,
///  and the comparison is the same code for a planted table and the live one.
struct ScriptedTable {
    rows: Result<Vec<MountedDatasetInstance>, String>,
    holding: Option<MountHoldingAPath>,
    holding_refusal: Option<String>,
}

impl MountedDatasets for ScriptedTable {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return self.rows.clone();
    }

    fn mount_holding(
        &self,
        _path: &Path,
    ) -> Result<Option<MountHoldingAPath>, String> {
        if let Some(reason) = &self.holding_refusal {
            return Err(reason.clone());
        }
        return Ok(self.holding.clone());
    }
}

/// A table where nothing but the clone's own mount stands.
fn a_table(rows: Vec<MountedDatasetInstance>) -> ScriptedTable {
    return ScriptedTable {
        rows: Ok(rows),
        holding: None,
        holding_refusal: None,
    };
}

#[test]
fn a_row_of_the_clone_at_its_slot_is_the_state_the_answer_must_produce() {
    let table: ScriptedTable = ScriptedTable {
        rows: Ok(vec![MountedDatasetInstance {
            dataset: String::from(CLONE),
            mount_point: slot(),
            root: PathBuf::from("/"),
        }]),
        holding: Some(MountHoldingAPath {
            dataset: Some(String::from(CLONE)),
            filesystem_type: String::from("zfs"),
            mount_point: slot(),
        }),
        holding_refusal: None,
    };
    let state: CloneMount = of_a_table(&table, CLONE, slot().as_path());
    assert_eq!(state, CloneMount::AtItsSlot);
    assert!(counts_as_at_its_slot(&state));
    assert_eq!(word(&state), "mounted at its slot");
}

#[test]
fn a_table_with_no_row_and_nothing_at_the_slot_is_mounted_nowhere() {
    let state: CloneMount = of_a_table(&a_table(Vec::new()), CLONE, slot().as_path());
    assert_eq!(state, CloneMount::Nowhere);
    assert!(!counts_as_at_its_slot(&state));
    assert_eq!(word(&state), "mounted nowhere");
}

#[test]
fn the_clone_under_another_path_is_elsewhere() {
    let table: ScriptedTable = ScriptedTable {
        rows: Ok(vec![MountedDatasetInstance {
            dataset: String::from(CLONE),
            mount_point: PathBuf::from("/home/user/.cache/staging"),
            root: PathBuf::from("/"),
        }]),
        holding: None,
        holding_refusal: None,
    };
    let state: CloneMount = of_a_table(&table, CLONE, slot().as_path());
    assert!(matches!(state, CloneMount::Elsewhere { .. }), "{state:?}");
    assert!(!counts_as_at_its_slot(&state));
    assert!(word(&state).contains("elsewhere"), "{}", word(&state));
}

#[test]
fn something_else_standing_at_the_slot_is_its_own_state() {
    let table: ScriptedTable = ScriptedTable {
        rows: Ok(Vec::new()),
        holding: Some(MountHoldingAPath {
            dataset: None,
            filesystem_type: String::from("tmpfs"),
            mount_point: slot(),
        }),
        holding_refusal: None,
    };
    let state: CloneMount = of_a_table(&table, CLONE, slot().as_path());
    assert!(
        matches!(state, CloneMount::SomethingElseAtTheSlot { .. }),
        "{state:?}"
    );
    assert!(!counts_as_at_its_slot(&state));
}

#[test]
fn a_table_that_could_not_be_read_is_its_own_state_and_never_absence() {
    let table: ScriptedTable = ScriptedTable {
        rows: Err(String::from("cannot read /proc/self/mountinfo")),
        holding: None,
        holding_refusal: None,
    };
    let state: CloneMount = of_a_table(&table, CLONE, slot().as_path());
    assert!(matches!(state, CloneMount::Unreadable { .. }), "{state:?}");
    assert!(!counts_as_at_its_slot(&state));
    assert!(
        word(&state).contains("could not be read"),
        "{}",
        word(&state)
    );
}

#[test]
fn a_bind_mount_of_a_subtree_is_not_the_clones_own_root() {
    let table: ScriptedTable = ScriptedTable {
        rows: Ok(vec![MountedDatasetInstance {
            dataset: String::from(CLONE),
            mount_point: slot(),
            root: PathBuf::from("/some/subtree"),
        }]),
        holding: None,
        holding_refusal: None,
    };
    let state: CloneMount = of_a_table(&table, CLONE, slot().as_path());
    assert!(
        matches!(state, CloneMount::SomethingElseAtTheSlot { .. }),
        "a subtree bind is not the clone: {state:?}"
    );
}
