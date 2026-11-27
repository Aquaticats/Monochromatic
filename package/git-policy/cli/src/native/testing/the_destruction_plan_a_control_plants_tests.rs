//! What:
//!  The controls of the plan a control plants:
//!  the add's reader and the grammar's own reader are asked what they make of it,
//!  for the plan a removal's first write leaves and for one whose head word and
//!  attempt lines a control names.
//! Why:
//!  The slot-state guest arm planted a hand-written plan in a guest,
//!  the add's reader refused it,
//!  and nothing on this host could see it (issue 6595).
//!  These controls run on the host with no pool,
//!  and they ask the same readers the arm's slot classification asks,
//!  so a plan a control plants that either reader refuses fails here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = thePlanNamingTheClone(common, guid, clone); // the reader the slot arm's class rests on
//! expect(found.head.word()).toBe('present');
//! ```

use super::a_planted_plans_snapshot_line::a_line_of;
use super::the_destruction_plan_a_control_plants::{
    ACloneAPlanNames,
    APlanAControlPlants,
    plant,
};
use crate::test_support::fixture;
use crate::worktree_clone_binding_record::add_state_commands::PlanHeadWord;
use crate::worktree_clone_binding_record::plan_folder_read::{
    APlanThatNamesTheClone,
    the_plan_naming_the_clone,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use std::path::{
    Path,
    PathBuf,
};

/// The clone every control's plan names.
const THE_CLONE: &str = "tank/worktree-clones/topic";

/// The GUID every control's plan records for that clone.
const THE_CLONES_GUID: u64 = 4_821_193_048_571_903_102;

/// The five words the plan's head destroy may hold,
/// which one control walks.
const EVERY_HEAD_WORD: &[HeadWord] = &[
    HeadWord::Present,
    HeadWord::Requested,
    HeadWord::Gone,
    HeadWord::Refused,
    HeadWord::Unknown,
];

/// One fixture with the common directory a plan folder is made in.
fn a_common_directory(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture's common directory");
    return common;
}

/// The GUID every control's plan records for the clone.
fn the_clone_guid() -> DatasetGuid {
    return DatasetGuid::parse(THE_CLONES_GUID.to_string().as_str())
        .expect("the fixture clone's GUID reads");
}

/// The plan a removal's first write leaves for the fixture's clone.
fn a_first_write_plan(common_dir: &Path) -> APlanAControlPlants {
    return APlanAControlPlants::a_removals_first_write(ACloneAPlanNames::a_fixture_clone(
        common_dir,
        THE_CLONE,
        the_clone_guid(),
    ));
}

/// The plan the add's reader answers for the fixture's clone,
/// which is the read the slot classification makes.
fn what_the_adds_reader_answers(common_dir: &Path) -> APlanThatNamesTheClone {
    return the_plan_naming_the_clone(common_dir, Some(the_clone_guid()), THE_CLONE)
        .expect("the planted plan is read as a plan")
        .expect("the planted plan names the clone it was planted for");
}

/// What:
///  A plan a control plants is the plan the add's reader accepts,
///  with the head word and the planned lines its own first write records.
/// Why:
///  This is the reader the slot classification rests on:
///  it answers `pending removal` only where this read answers,
///  and a plan it refuses makes the slot `cannot be proved`.
#[test]
fn the_plan_a_control_plants_is_the_one_the_adds_reader_accepts() {
    let common: PathBuf = a_common_directory("plant-the-adds-reader");
    let planted: PathBuf = plant(&a_first_write_plan(common.as_path()));
    let found: APlanThatNamesTheClone = what_the_adds_reader_answers(common.as_path());
    assert_eq!(
        found.plan, planted,
        "the plan the writer filed is the one read"
    );
    assert_eq!(found.clone_dataset, THE_CLONE);
    assert_eq!(found.clone_guid, the_clone_guid());
    assert_eq!(found.head, PlanHeadWord::Present);
    assert!(
        found.every_tuple_planned,
        "a first write destroys nothing, so every line it records says `planned`"
    );
}

/// What:
///  The same planted plan is the record the design's own grammar reads back,
///  field for field.
/// Why:
///  The add's reader asks the four fields its stop needs and the grammar asks all
///  eighteen,
///  so a plan that satisfies one may still be a record the removal refuses;
///  this control holds the two readers to one record.
#[test]
fn the_plan_a_control_plants_is_the_record_the_grammars_reader_answers() {
    let common: PathBuf = a_common_directory("plant-the-grammars-reader");
    let planted: APlanAControlPlants = a_first_write_plan(common.as_path());
    plant(&planted);
    let read: DestructionPlan = read_the_plan(common.as_path(), the_clone_guid())
        .expect("the grammar's reader accepts the planted plan");
    assert_eq!(read.head, HeadWord::Present);
    assert_eq!(read.attempts, planted.attempts);
    assert_eq!(read.facts.clone_dataset, THE_CLONE);
    assert_eq!(read.facts.clone_guid, the_clone_guid());
    assert_eq!(read.facts.clone_createtxg, planted.clone.clone_createtxg);
    assert_eq!(read.facts.origin_snapshot, planted.clone.origin_snapshot);
    assert_eq!(
        read.facts.registration_id,
        Vec::from("topic".as_bytes()),
        "the registration is the slot the clone's dataset ends in"
    );
    assert_eq!(
        read.facts.target,
        common
            .parent()
            .expect("the common directory stands in a checkout")
            .join(".claude")
            .join("worktrees")
            .join("topic"),
        "the slot's path is the checkout's worktrees folder and the slot's name"
    );
}

/// What:
///  Every one of the five head words a control names is the word both readers
///  answer.
/// Why:
///  The head word is what the command beside a standing plan follows from,
///  so a word a control writes must survive both readers as that same word.
#[test]
fn the_head_word_a_control_names_is_the_word_both_readers_answer() {
    for &head in EVERY_HEAD_WORD {
        let common: PathBuf = a_common_directory(format!("plant-head-{}", head.word()).as_str());
        let first_write: APlanAControlPlants = a_first_write_plan(common.as_path());
        let planted: APlanAControlPlants = APlanAControlPlants {
            head,
            ..first_write
        };
        plant(&planted);
        assert_eq!(
            what_the_adds_reader_answers(common.as_path()).head.word(),
            head.word()
        );
        let read: DestructionPlan = read_the_plan(common.as_path(), the_clone_guid())
            .expect("the grammar's reader accepts the head word the control names");
        assert_eq!(read.head, head);
        assert_eq!(read.attempts, planted.attempts);
    }
}

/// What:
///  A planted line past `planned` is a plan that witnesses a destroy,
///  which is the fact the cancel command's clause reads,
///  and the grammar still reads the history that ends in `destroyed`.
/// Why:
///  The one outcome a plan's first write records is `planned`,
///  so a control of any later state needs a line the grammar allows after it.
#[test]
fn a_planted_line_past_planned_is_a_plan_that_witnesses_a_destroy() {
    let common: PathBuf = a_common_directory("plant-destroyed");
    let mut planted: APlanAControlPlants = a_first_write_plan(common.as_path());
    let planned = a_line_of(&planted, 0, SnapshotOutcome::Planned);
    let destroyed = a_line_of(&planted, 1, SnapshotOutcome::Destroyed);
    planted.attempts = vec![planned, destroyed];
    plant(&planted);
    let found: APlanThatNamesTheClone = what_the_adds_reader_answers(common.as_path());
    assert!(
        !found.every_tuple_planned,
        "a line whose outcome left `planned` is a plan that witnesses a destroy"
    );
    let read: DestructionPlan = read_the_plan(common.as_path(), the_clone_guid())
        .expect("the grammar's reader accepts a history that ends `destroyed`");
    assert_eq!(read.attempts, planted.attempts);
}
