//! What:
//!  The controls of the removal's own records:
//!  the inventories before the binding,
//!  the plan only after them,
//!  and a binding of another clone left alone.
//! Why:
//!  The plan is the last witness of a removal,
//!  so it goes only after the binding that names the same clone is proved gone,
//!  and a binding that names another clone is a new slot's work rather than this removal's leftover
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "When it is removed").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gone = removeTheRemovalRecords(commonDir, slot, plan);
//! expect(gone.planRemoved).toBe(true);
//! ```

use crate::worktree_clone_binding_record::registration_home::{binding_name, clone_inventory_name, home_folder, origin_inventory_name, plans_folder};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{ADoneUnmount, ScriptedClock};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{RemovalEntry, enter_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture;
use crate::worktree_clone_removal_proof_and_plan::removal_records_end::{RecordsGone, remove_the_removal_records};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::PathBuf;

/// The slot every control removes.
const SLOT: &str = "fix-flaky-test";

/// One fixture with a plan written by a removal's own entry,
/// and the two inventories the add wrote.
fn a_fixture(name: &str) -> (RemovalFixture, DestructionPlan) {
    let fixture: RemovalFixture = RemovalFixture::healthy(name);
    let pool = fixture.healthy_pool();
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount { clone_guid: recorded.clone_guid };
    let at: RemovalEntry<'_> = RemovalEntry { common_dir: fixture.common(), slot: SLOT };
    let entered = enter_the_removal(&at, &pool, &unmount, &ScriptedClock).expect("the entry writes the plan");
    let home: PathBuf = home_folder(fixture.common());
    std::fs::write(home.join(clone_inventory_name(SLOT)), b"# a clone inventory\n").expect("the clone inventory");
    return (fixture, entered.plan);
}

/// Whether one name stands in the wrapper's home or the plan folder.
fn stands(path: &std::path::Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

#[test]
fn the_inventories_go_before_the_binding_and_the_plan_after_them() {
    let (fixture, plan) = a_fixture("records-end-healthy");
    let home: PathBuf = home_folder(fixture.common());
    let gone: RecordsGone = remove_the_removal_records(fixture.common(), SLOT, &plan).expect("the records go");
    assert_eq!(
        gone.files,
        vec![String::from(origin_inventory_name(SLOT)), String::from(clone_inventory_name(SLOT)), String::from(binding_name(SLOT))],
        "the inventories first and the binding last"
    );
    assert!(gone.plan_removed && gone.folder_removed, "the plan and its empty folder went");
    for name in [origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] {
        assert!(!stands(home.join(name.as_str()).as_path()), "`{name}` is gone");
    }
    assert!(!stands(plans_folder(fixture.common()).as_path()), "the plan folder is gone");
}

#[test]
fn a_binding_of_another_clone_is_left_and_the_plan_with_it() {
    let (fixture, plan) = a_fixture("records-end-other");
    let home: PathBuf = home_folder(fixture.common());
    let binding: PathBuf = home.join(binding_name(SLOT));
    let text: String = std::fs::read_to_string(binding.as_path()).expect("the binding reads");
    let changed: String = text.replace(&format!("cloneGuid={}", plan.facts.clone_guid.file_name()), "cloneGuid=1");
    std::fs::write(binding.as_path(), changed).expect("the binding names another clone");
    let stop: RemovalStop = remove_the_removal_records(fixture.common(), SLOT, &plan).expect_err("another clone's binding is not this removal's");
    assert!(matches!(stop, RemovalStop::TheRecordsCouldNotGo { .. }));
    assert!(stands(binding.as_path()), "the binding stands");
    assert!(stands(home.join(origin_inventory_name(SLOT)).as_path()), "and its inventories stand");
    assert!(stands(plans_folder(fixture.common()).join(plan.facts.clone_guid.file_name()).as_path()), "the plan stands as the last witness");
}

#[test]
fn the_plan_folder_is_made_by_the_entry_and_read_by_this_step() {
    let (fixture, _plan) = a_fixture("records-end-folder");
    let folder: PlanFolder = PlanFolder::open_or_create(fixture.common()).expect("the folder opens");
    assert_eq!(folder.path(), plans_folder(fixture.common()));
}
