//! What:
//!  The controls of the plan's own removal:
//!  the file unlinked and proved gone,
//!  a file another command removed read as done,
//!  and a name that still answers refused.
//! Why:
//!  The plan is the last witness of a removal,
//!  so a removal that could not be proved is said rather than reported as finished.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(removeThePlan(commonDir, plan)).toBeUndefined();
//! ```

use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_finish_pass::pass_fixtures::{a_plan_for, write_the_plan};
use crate::worktree_clone_removal_finish_pass::the_plans_file::remove_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture;
use std::path::PathBuf;

#[test]
fn the_plans_own_file_is_removed_and_proved_gone() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-plan-file");
    let plan = a_plan_for(&fixture, HeadWord::Gone);
    let path: PathBuf = write_the_plan(&fixture, &plan);
    remove_the_plan(fixture.common(), &plan).expect("the plan's own file goes");
    assert!(!path.exists(), "the plan's file is gone");
}

#[test]
fn a_plan_another_command_already_removed_is_the_same_removal() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-plan-file-gone");
    let plan = a_plan_for(&fixture, HeadWord::Gone);
    let path: PathBuf = write_the_plan(&fixture, &plan);
    std::fs::remove_file(path.as_path()).expect("another command's look removed the plan");
    remove_the_plan(fixture.common(), &plan).expect("a name already gone is the same removal");
}

#[test]
fn a_name_that_still_answers_is_a_refusal_and_not_a_finished_removal() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-plan-file-directory");
    let plan = a_plan_for(&fixture, HeadWord::Gone);
    let folder: PathBuf = plans_folder(fixture.common());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    // A directory at the plan's own name answers after any unlink of a file there, so this arm needs
    // no permission bit and reads the same as root.
    std::fs::create_dir_all(folder.join(plan.facts.clone_guid.file_name()).as_path()).expect("a directory at the plan's name");
    let said: String = remove_the_plan(fixture.common(), &plan).expect_err("a directory is not a plan's file");
    assert!(said.contains("the unlink answered"), "the refusal carries what the call answered: {said}");
    assert!(folder.join(plan.facts.clone_guid.file_name()).exists(), "the directory still answers");
}
