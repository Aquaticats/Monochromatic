//! What:
//!  The controls of the pass's refusal lines:
//!  each sentence names what it is about,
//!  and the lines that must say nothing was removed do.
//! Why:
//!  A person acts on these lines between their own commands,
//!  so a line that named no plan or no refusal would be the one thing they cannot use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theGateRefused(plan, 'the device cannot be opened')).toContain(plan);
//! ```

use crate::worktree_clone_removal_finish_pass::pass_refusal_lines::{
    the_binding_could_not_be_read, the_binding_disagrees, the_binding_names_another_slot, the_claim_could_not_be_taken, the_clone_stands, the_entry_is_no_plan_name, the_folder_could_not_be_listed,
    the_gate_refused, the_name_was_reused_with_a_registration, the_plan_could_not_be_removed, the_plan_was_not_read, the_pool_could_not_prove, the_registration_could_not_be_read,
    the_registration_could_not_be_settled, the_slot_is_not_a_name,
};
use std::path::Path;

/// The plan every line here names.
const PLAN: &str = "/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989";

#[test]
fn every_line_names_the_plan_it_is_about() {
    let plan: &Path = Path::new(PLAN);
    let lines: Vec<String> = vec![
        the_gate_refused(plan, "the device cannot be opened"),
        the_pool_could_not_prove(plan, "the pool refused the read"),
        the_clone_stands(plan, "tank/wt/fix-flaky-test", "7828944690444299989"),
        the_name_was_reused_with_a_registration(plan, "tank/wt/fix-flaky-test", "7828944690444299989", "1"),
        the_binding_could_not_be_read(plan, Path::new("/repo/.git/home/fix-flaky-test.binding"), "mode 0666"),
        the_binding_names_another_slot(plan, Path::new("/repo/.git/home/fix-flaky-test.binding"), "aa", "bb"),
        the_binding_disagrees(plan, Path::new("/repo/.git/home/fix-flaky-test.binding"), "originSnapshotGuid", "1", "2"),
        the_registration_could_not_be_settled(plan, "the registration", "the lock is held"),
        the_registration_could_not_be_read(plan, Path::new("/repo/.git/worktrees/fix-flaky-test"), "permission denied"),
        the_slot_is_not_a_name(plan, "/", "the path names no folder"),
        the_claim_could_not_be_taken(plan, "the folder is read-only"),
        the_plan_could_not_be_removed(plan, "the unlink answered `EISDIR`"),
        the_plan_was_not_read(plan, "it stops before the header fields it declares"),
    ];
    for line in lines {
        assert!(line.contains(PLAN), "every line names the plan: {line}");
    }
}

#[test]
fn the_lines_about_a_record_say_what_it_left_alone() {
    let plan: &Path = Path::new(PLAN);
    assert!(the_plan_was_not_read(plan, "it stops before the header fields it declares").contains("nothing was destroyed by reading it"));
    assert!(the_plan_could_not_be_removed(plan, "the unlink answered `EISDIR`").contains("nothing else was removed"));
    let entry: String = the_entry_is_no_plan_name(Path::new("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/notes.txt"));
    assert!(entry.contains("notes.txt") && entry.contains("read nothing in it"), "the foreign entry is named and never read: {entry}");
    let folder: String = the_folder_could_not_be_listed(Path::new("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans"), "not a folder");
    assert!(folder.contains("not a folder"), "the folder's line carries what the call answered: {folder}");
}
