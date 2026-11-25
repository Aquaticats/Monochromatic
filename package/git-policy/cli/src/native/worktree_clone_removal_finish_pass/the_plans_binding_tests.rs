//! What:
//!  The controls of the list's step 12 for one plan:
//!  the three files removed in their order and proved gone,
//!  a binding of another clone left,
//!  an inventory that outlived its binding removed as the same step's leftover,
//!  and each refusal that leaves both records.
//! Why:
//!  The plan is the last witness,
//!  so this step's order and its proofs are what let the plan go at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishThePlansBinding(commonDir, slot, plan)).toEqual({ kind: 'removed', files });
//! ```

use crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal;
use crate::worktree_clone_binding_record::registration_home::{binding_name, clone_inventory_name, home_folder, origin_inventory_name};
use crate::worktree_clone_removal_finish_pass::pass_fixtures::{SLOT, a_plan_for, stands, the_home};
use crate::worktree_clone_removal_finish_pass::the_plans_binding::{TheBindingEnd, finish_the_plans_binding};
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture;
use std::path::{Path, PathBuf};

/// One fixture with a binding,
/// its two inventories and a plan whose head is `gone`.
fn a_slot( name: &str) -> (RemovalFixture, crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan) {
    let fixture: RemovalFixture = RemovalFixture::healthy(name);
    let plan = a_plan_for(&fixture, HeadWord::Gone);
    std::fs::write(the_home(&fixture).join(clone_inventory_name(SLOT)).as_path(), b"# a clone inventory\n").expect("the clone inventory");
    return (fixture, plan);
}

/// Rewrite one binding line's value.
fn rewrite_the_binding(fixture: &RemovalFixture, key: &str, value: &str) {
    let path: PathBuf = the_home(fixture).join(binding_name(SLOT));
    let text: String = std::fs::read_to_string(path.as_path()).expect("the binding reads");
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        lines.push(match line.starts_with(format!("{key}=").as_str()) {
            true => format!("{key}={value}"),
            false => String::from(line),
        });
    }
    std::fs::write(path.as_path(), format!("{}\n", lines.join("\n"))).expect("the binding is rewritten");
}

#[test]
fn the_two_inventories_go_first_and_the_binding_last_with_every_file_proved_gone() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-healthy");
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    assert_eq!(
        end,
        TheBindingEnd::Removed { files: vec![origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] },
        "the inventories go first and the binding last"
    );
    for name in [origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] {
        assert!(!stands(home_folder(fixture.common()).join(name.as_str()).as_path()), "`{name}` is gone");
    }
}

#[test]
fn a_binding_of_another_clone_is_left_with_its_inventories() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-anothers");
    rewrite_the_binding(&fixture, "cloneGuid", "1");
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    assert_eq!(end, TheBindingEnd::AnotherClones, "a binding that names another clone is a new slot's");
    let home: PathBuf = home_folder(fixture.common());
    for name in [origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] {
        assert!(stands(home.join(name.as_str()).as_path()), "`{name}` is left");
    }
}

#[test]
fn an_inventory_that_outlived_its_binding_is_removed_as_the_same_steps_leftover() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-leftover");
    let home: PathBuf = home_folder(fixture.common());
    std::fs::remove_file(home.join(binding_name(SLOT)).as_path()).expect("the binding went in a kill inside the step");
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    assert_eq!(end, TheBindingEnd::Missing { files: vec![origin_inventory_name(SLOT), clone_inventory_name(SLOT)] }, "the leftover inventories go in their own order");
    assert!(!stands(home.join(origin_inventory_name(SLOT)).as_path()), "the origin inventory went");
    assert!(!stands(home.join(clone_inventory_name(SLOT)).as_path()), "the clone inventory went");
}

#[test]
fn a_binding_whose_origin_fields_differ_leaves_both_records() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-differs");
    rewrite_the_binding(&fixture, "originSnapshotCreatetxg", "1");
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    let TheBindingEnd::Differs { field, binding, plan: planned, .. } = end else { panic!("a disagreement is reported as one, and this answer was {end:?}") };
    assert_eq!(field, "originSnapshotCreatetxg", "the field that differs is named");
    assert_eq!(binding, "1", "the binding's value is carried");
    assert_eq!(planned, plan.facts.origin_snapshot_createtxg.to_string(), "and the plan's");
    assert!(stands(home_folder(fixture.common()).join(binding_name(SLOT)).as_path()), "the binding stands");
}

#[test]
fn a_binding_that_names_another_slot_leaves_both_records() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-slot");
    rewrite_the_binding(&fixture, "slot", path_as_hexadecimal(Path::new("/elsewhere").join(SLOT).as_path()).as_str());
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    assert!(matches!(end, TheBindingEnd::Differs { field: "slot", .. }), "a binding that names another path is a disagreement about the slot, and this answer was {end:?}");
}

#[test]
fn a_binding_that_cannot_be_read_leaves_both_records() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-unreadable");
    let binding: PathBuf = home_folder(fixture.common()).join(binding_name(SLOT));
    std::fs::write(binding.as_path(), b"not a binding\n").expect("the binding's name holds other bytes");
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    assert!(matches!(end, TheBindingEnd::Unreadable { .. }), "a binding that does not parse is unreadable, and this answer was {end:?}");
    assert!(stands(binding.as_path()), "the binding stands");
}

#[test]
fn a_slot_with_no_binding_and_no_inventory_is_the_step_already_done() {
    let (fixture, plan) = a_slot("finish-pass-binding-end-none");
    let home: PathBuf = home_folder(fixture.common());
    for name in [origin_inventory_name(SLOT), clone_inventory_name(SLOT), binding_name(SLOT)] {
        std::fs::remove_file(home.join(name.as_str())).expect("the record is there before the arm removes it");
    }
    let end: TheBindingEnd = finish_the_plans_binding(fixture.common(), SLOT, &plan);
    assert_eq!(end, TheBindingEnd::Missing { files: Vec::new() }, "nothing stood, so nothing was removed");
}
