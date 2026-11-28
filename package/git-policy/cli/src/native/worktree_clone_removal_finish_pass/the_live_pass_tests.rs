//! What:
//!  The controls of the live pass:
//!  a repository the route never served costs one look and prints nothing,
//!  and the look leaves nothing behind.
//! Why:
//!  The pass runs before every forwarded command,
//!  so the healthy state is the state that matters most:
//!  it must not open a pool,
//!  write a record or create the folder it looked for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishTheRemovalPlansLive(commonDir)).toEqual([]);
//! ```

use super::finish_the_removal_plans_live;
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use std::path::PathBuf;

#[test]
fn a_repository_with_no_plan_folder_prints_nothing_and_gains_nothing() {
    let common: PathBuf = fixture("the_live_pass_with_no_plan_folder");
    let said: Vec<String> = finish_the_removal_plans_live(common.as_path());
    assert!(
        said.is_empty(),
        "a repository the route never served prints no line, and this one printed {said:?}"
    );
    assert!(
        !plans_folder(common.as_path()).exists(),
        "the pass looked for the plan folder and left it exactly as it found it"
    );
}

#[test]
fn the_look_asks_the_configured_place_and_no_other() {
    let common: PathBuf = fixture("the_live_pass_asks_the_records_place");
    let before: usize = std::fs::read_dir(common.as_path())
        .expect("the fixture folder reads")
        .count();
    let _ = finish_the_removal_plans_live(common.as_path());
    let after: usize = std::fs::read_dir(common.as_path())
        .expect("the fixture folder reads")
        .count();
    assert_eq!(
        before, after,
        "the healthy state adds no entry to the repository's common folder"
    );
}
