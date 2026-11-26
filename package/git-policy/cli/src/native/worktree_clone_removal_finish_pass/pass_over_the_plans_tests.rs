//! What:
//!  The finish pass's controls over the plan folder itself:
//!  one look while no folder stands,
//!  a plan of every head word other than `gone` left untouched,
//!  a plan that does not read reported and left,
//!  a foreign entry never read,
//!  an empty folder removed with nothing printed,
//!  and a folder that is not a folder named once.
//! Why:
//!  These are the design's folder controls,
//!  and each one is red without the guard it proves
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass and the folder").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(thePass(fixture, liveFolder, gate, probe, settle)).toEqual([]);
//! ```

use crate::worktree_clone_binding_record::registration_home::wrapper_folder;
use crate::worktree_clone_removal_finish_pass::pass_fixtures::{
    AFolder,
    AGate,
    AProbe,
    ASettle,
    a_plan_for,
    stands,
    the_home,
    the_pass,
    the_plan_folder,
    write_the_plan,
};
use crate::worktree_clone_removal_finish_pass::the_plans_folder::{
    FolderLook,
    TheLivePlansFolder,
};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE_GUID,
    RemovalFixture,
};
use std::path::PathBuf;

/// Every head word a pass must leave alone.
const OTHER_HEADS: &[HeadWord] = &[
    HeadWord::Present,
    HeadWord::Requested,
    HeadWord::Refused,
    HeadWord::Unknown,
];

#[test]
fn a_repository_with_no_plan_folder_is_looked_at_once_and_says_nothing() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-no-folder");
    let folder: AFolder = AFolder::answering(FolderLook::NoFolder);
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &folder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert!(
        lines.is_empty(),
        "a repository with no plan folder prints nothing, and this one printed {lines:?}"
    );
    assert_eq!(
        folder.looks.get(),
        1,
        "the pass takes one look at the folder and no more"
    );
    assert!(
        !stands(the_plan_folder(&fixture).as_path()),
        "the pass never creates the plan folder"
    );
    assert_eq!(settle.calls.get(), 0, "nothing is settled without a plan");
}

#[test]
fn a_plan_whose_head_is_not_gone_is_left_without_a_word() {
    for &head in OTHER_HEADS {
        let name: String = format!("finish-pass-head-{}", head.word());
        let fixture: RemovalFixture = RemovalFixture::healthy(name.as_str());
        let plan = a_plan_for(&fixture, head);
        let path: PathBuf = write_the_plan(&fixture, &plan);
        let settle: ASettle = ASettle::succeeding();
        let lines: Vec<String> = the_pass(
            &fixture,
            &TheLivePlansFolder,
            &AGate::passing(),
            &AProbe::standing(),
            &settle,
        );
        assert!(
            lines.is_empty(),
            "a plan whose head is `{}` is left without a word, and this run printed {lines:?}",
            head.word()
        );
        assert!(stands(path.as_path()), "the plan stands after the pass");
        assert!(
            stands(
                the_home(&fixture)
                    .as_path()
                    .join("fix-flaky-test.binding")
                    .as_path()
            ),
            "its binding stands"
        );
        assert_eq!(
            settle.calls.get(),
            0,
            "no registration is settled for a plan the pass does not take"
        );
    }
}

#[test]
fn a_plan_that_does_not_read_is_named_and_left_where_it_stands() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-unreadable");
    let plan = a_plan_for(&fixture, HeadWord::Gone);
    let path: PathBuf = write_the_plan(&fixture, &plan);
    std::fs::write(path.as_path(), b"this is not a destruction plan\n")
        .expect("the plan's name holds other bytes");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(lines.len(), 1, "one line for one record that does not read");
    assert!(
        lines[0].contains(path.to_str().expect("the fixture's path is text")),
        "the line names the record: {}",
        lines[0]
    );
    assert!(
        stands(path.as_path()),
        "a record that does not read is never moved or removed"
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "nothing is settled on a record that does not read"
    );
}

#[test]
fn a_foreign_entry_is_named_and_read_by_nobody() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-foreign");
    let folder: PlanFolder =
        PlanFolder::open_or_create(fixture.common()).expect("the plan folder opens");
    let foreign: PathBuf = folder.path().join("notes.txt");
    std::fs::write(foreign.as_path(), b"a person's note\n").expect("the foreign entry");
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &ASettle::succeeding(),
    );
    assert_eq!(lines.len(), 1, "one line for one foreign entry");
    assert!(
        lines[0].contains("notes.txt"),
        "the line names the entry: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("carries no clone GUID's name"),
        "the entry is refused by its name, never read as a plan: {}",
        lines[0]
    );
    assert!(stands(foreign.as_path()), "a foreign entry is left alone");
    assert!(
        stands(folder.path()),
        "the folder still holds an entry, so it stays"
    );
}

#[test]
fn an_empty_plan_folder_goes_with_nothing_printed() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-empty-folder");
    let folder: PlanFolder =
        PlanFolder::open_or_create(fixture.common()).expect("the plan folder opens");
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &ASettle::succeeding(),
    );
    assert!(
        lines.is_empty(),
        "an empty folder is removed with nothing printed, and this run printed {lines:?}"
    );
    assert!(!stands(folder.path()), "the empty folder went");
}

#[test]
fn a_plan_folder_that_is_a_file_is_named_once_and_nothing_else_happens() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-folder-is-a-file");
    std::fs::create_dir_all(wrapper_folder(fixture.common()).as_path())
        .expect("the wrapper folder");
    let not_a_folder: PathBuf = the_plan_folder(&fixture);
    std::fs::write(not_a_folder.as_path(), b"not a folder\n").expect("a file at the folder's path");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(lines.len(), 1, "one line for the path that is no folder");
    assert!(
        lines[0].contains(not_a_folder.to_str().expect("the fixture's path is text")),
        "the line names the path: {}",
        lines[0]
    );
    assert!(
        stands(not_a_folder.as_path()),
        "the pass removes nothing at a path that is no folder"
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "no plan was read, so nothing is settled"
    );
}

#[test]
fn the_plan_is_filed_under_its_clone_guid_and_no_other_name() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-filed-name");
    let plan = a_plan_for(&fixture, HeadWord::Gone);
    let path: PathBuf = write_the_plan(&fixture, &plan);
    assert_eq!(
        path.file_name()
            .map(|name| return name.to_string_lossy().into_owned()),
        Some(CLONE_GUID.to_string()),
        "the plan is filed under its clone's GUID"
    );
}
