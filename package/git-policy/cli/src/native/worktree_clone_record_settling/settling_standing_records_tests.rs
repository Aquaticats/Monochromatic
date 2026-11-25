//! Controls for the two records that may stand beside a binding:
//!  a standing origin cleanup record that names the same objects is found and left,
//!  one that names another object refuses,
//!  a standing plan is found and its lost names are read,
//!  a plan whose head is gone refuses as the finish pass's,
//!  and no plan at all is no stop.

use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_record_settling::settling_fixtures::{SLOT, TheFixture, publish_the_record, the_fixture};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_record::{TheRecord, read_the_record};
use crate::worktree_clone_record_settling::settling_standing_records::{a_standing_origin_cleanup, a_standing_plan, the_names_the_plan_lost};
use crate::worktree_clone_removal_proof_and_plan::fixtures::{guid, nonce};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanFacts, PlanSnapshot};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, SnapshotOutcome};

/// The moment a control's record carries.
const NOW: &str = "2026-11-25T09:12:44-05:00";

/// The snapshot a control's plan records as destroyed.
const LOST_SNAPSHOT: &str = "tank/monochromatic-git-worktree-clone-datasets/topic@worktree-own";

/// The published record of one fixture.
fn the_published(name: &str) -> (TheFixture, TheRecord) {
    let made: TheFixture = the_fixture(name);
    publish_the_record(&made, crate::worktree_clone_binding_record::binding_words::Phase::Filled);
    let record: TheRecord = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    return (made, record);
}

/// One origin cleanup record naming the binding's own objects.
fn the_cleanup(record: &TheRecord) -> OriginCleanupRecord {
    let own_name: String = record.recorded.origin_snapshot.rsplit_once('@').map_or_else(|| return record.recorded.origin_snapshot.clone(), |pair| return String::from(pair.1));
    return OriginCleanupRecord::owed(
        record.recorded.origin_dataset.as_str(),
        record.recorded.origin_dataset_guid,
        own_name.as_str(),
        record.recorded.origin_snapshot_guid,
        record.recorded.origin_snapshot_createtxg,
        record.recorded.clone_dataset.as_str(),
        record.recorded.clone_guid,
        record.recorded.slot_path.as_path(),
        NOW,
    );
}

/// One destruction plan naming the binding's own objects,
/// with one snapshot recorded as destroyed.
fn the_plan(record: &TheRecord) -> DestructionPlan {
    let facts: PlanFacts = PlanFacts {
        run: nonce("00112233445566778899aabbccddeeff"),
        owner: String::from("48213:linux:9481733"),
        started: String::from(NOW),
        clone_dataset: record.recorded.clone_dataset.clone(),
        clone_guid: record.recorded.clone_guid,
        clone_createtxg: record.recorded.clone_createtxg,
        origin_dataset: record.recorded.origin_dataset.clone(),
        origin_dataset_guid: record.recorded.origin_dataset_guid,
        origin_snapshot: record.recorded.origin_snapshot.clone(),
        origin_snapshot_guid: record.recorded.origin_snapshot_guid,
        origin_snapshot_createtxg: record.recorded.origin_snapshot_createtxg,
        registration_id: record.recorded.registration_id.clone(),
        admin: String::from("2049:131081:1763750412:987654321"),
        target: record.recorded.slot_path.clone(),
        unmount_nonce: nonce("0a1b2c3d4e5f60718293a4b5c6d7e8f9"),
        unmount_mount_id: None,
    };
    let mut plan: DestructionPlan = DestructionPlan::first_write(facts, &[PlanSnapshot { name: String::from(LOST_SNAPSHOT), guid: guid(17_157_080_670_377_245_572), createtxg: 18 }]);
    plan.attempts[0].outcome = SnapshotOutcome::Destroyed;
    plan.head = HeadWord::Present;
    return plan;
}

/// A cleanup record that names the binding's own objects is found,
/// and the five fields are compared before it is printed.
#[test]
fn a_standing_cleanup_of_the_same_objects_is_found() {
    let (made, record) = the_published("settling-cleanup-same");
    let folder: CleanupFolder = CleanupFolder::open_or_create(made.common.as_path()).expect("the cleanup folder");
    folder.publish(&the_cleanup(&record)).expect("the cleanup record writes");
    let found: OriginCleanupRecord = a_standing_origin_cleanup(made.common.as_path(), &record.recorded).expect("the record reads").expect("the record stands");
    assert_eq!(found.origin_snapshot_guid, record.recorded.origin_snapshot_guid);
}

/// A cleanup record naming another origin snapshot refuses,
/// and nothing is removed.
#[test]
fn a_cleanup_of_another_object_refuses() {
    let (made, record) = the_published("settling-cleanup-other");
    let folder: CleanupFolder = CleanupFolder::open_or_create(made.common.as_path()).expect("the cleanup folder");
    let mut other: OriginCleanupRecord = the_cleanup(&record);
    other.origin_snapshot_guid = guid(11_050_373_805_378_911_967);
    let path: std::path::PathBuf = folder.publish(&other).expect("the cleanup record writes");
    let refused = a_standing_origin_cleanup(made.common.as_path(), &record.recorded).expect_err("another object refuses");
    assert_eq!(refused, SettlingStop::TheOriginCleanupDisagrees { field: String::from("originSnapshotGuid"), recorded: record.recorded.origin_snapshot_guid.file_name(), standing: String::from("11050373805378911967") });
    assert!(path.exists(), "the record is compared and never removed");
}

/// A standing plan names its lost snapshots,
/// and the plan is found for the records' removal.
#[test]
fn a_standing_plan_is_found_with_its_lost_names() {
    let (made, record) = the_published("settling-plan-present");
    let folder: PlanFolder = PlanFolder::open_or_create(made.common.as_path()).expect("the plan folder");
    let plan: DestructionPlan = the_plan(&record);
    folder.publish(&plan).expect("the plan writes");
    let found: DestructionPlan = a_standing_plan(made.common.as_path(), &record.recorded).expect("the plan reads").expect("the plan stands");
    assert_eq!(found.head, HeadWord::Present);
    assert_eq!(the_names_the_plan_lost(&found), vec![String::from(LOST_SNAPSHOT)]);
}

/// A plan whose head is gone is the finish pass's,
/// so the settling command refuses it.
#[test]
fn a_plan_whose_head_is_gone_refuses() {
    let (made, record) = the_published("settling-plan-gone");
    let folder: PlanFolder = PlanFolder::open_or_create(made.common.as_path()).expect("the plan folder");
    let mut plan: DestructionPlan = the_plan(&record);
    plan.head = HeadWord::Gone;
    folder.publish(&plan).expect("the plan writes");
    let refused = a_standing_plan(made.common.as_path(), &record.recorded).expect_err("a gone head refuses");
    let SettlingStop::ThePlanIsTheFinishPasss { path } = &refused else { panic!("the gone head is the finish pass's arm, and this answered {refused:?}") };
    assert!(path.ends_with(record.recorded.clone_guid.file_name()), "{}", path.display());
    assert!(refused.sentence().contains("finish pass"), "{}", refused.sentence());
}

/// A plan filed under this clone's GUID whose identities disagree refuses.
#[test]
fn a_plan_whose_identities_disagree_refuses() {
    let (made, record) = the_published("settling-plan-disagrees");
    let folder: PlanFolder = PlanFolder::open_or_create(made.common.as_path()).expect("the plan folder");
    let mut plan: DestructionPlan = the_plan(&record);
    plan.facts.target = std::path::PathBuf::from("/elsewhere/.claude/worktrees/topic");
    folder.publish(&plan).expect("the plan writes");
    let refused = a_standing_plan(made.common.as_path(), &record.recorded).expect_err("another target refuses");
    assert_eq!(refused, SettlingStop::ThePlanDisagrees { field: String::from("target"), binding: record.recorded.slot_path.display().to_string(), plan: String::from("/elsewhere/.claude/worktrees/topic") });
}

/// No plan folder and no plan are both no stop.
#[test]
fn no_plan_is_no_stop() {
    let (made, record) = the_published("settling-plan-none");
    assert!(a_standing_plan(made.common.as_path(), &record.recorded).expect("no plan reads").is_none());
}
