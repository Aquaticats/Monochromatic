//! Controls for the add's read of the plan folder:
//!  a missing folder holds no plan,
//!  a plan naming another clone is no stop,
//!  a plan filed under this clone's GUID is this clone's,
//!  a plan naming this clone's field is a stop and answers its head word,
//!  a plan that cannot be read stops the add,
//!  and a file that is not filed under a clone's GUID is foreign and read by
//!  nobody.
//!  The name check those controls rest on is controlled here,
//!  because the removal's plan reader calls this one rather than repeating it.

use crate::testing::a_planted_plans_snapshot_line::a_line_of;
use crate::testing::the_destruction_plan_a_control_plants::{
    ACloneAPlanNames,
    APlanAControlPlants,
    plant,
};
use crate::worktree_clone_binding_record::add_state_commands::PlanHeadWord;
use crate::worktree_clone_binding_record::plan_folder_read::{
    APlanThatNamesTheClone,
    PlanUnreadable,
    is_canonical_guid_name,
    the_plan_naming_the_clone,
};
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_record::PlanAttempt;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// One fixture's common directory,
/// and the plan folder a planted plan makes.
fn with_plans(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture's common directory");
    let folder: PathBuf = plans_folder(common.as_path());
    return (root, folder);
}

/// The common directory above one plan folder.
fn common_of(folder: &Path) -> &Path {
    return folder
        .parent()
        .and_then(|path| return path.parent())
        .expect("the common directory is the grandparent");
}

/// The fixture's clone.
fn clone_guid() -> DatasetGuid {
    return a_guid("4821193048571903102");
}

/// One clone's GUID built from the decimal number a fixture names.
fn a_guid(digits: &str) -> DatasetGuid {
    return DatasetGuid::parse(digits).expect("the fixture's digits are a canonical GUID");
}

/// The first write's history,
/// which is one line per snapshot at attempt `0`, saying `planned`.
const A_PLANS_FIRST_WRITE: &[(u64, SnapshotOutcome)] = &[(0, SnapshotOutcome::Planned)];

/// One snapshot's history where the removal planned it and then proved it gone,
/// which is what a plan that witnesses a destroy holds.
const PLANNED_AND_THEN_DESTROYED: &[(u64, SnapshotOutcome)] = &[
    (0, SnapshotOutcome::Planned),
    (1, SnapshotOutcome::Destroyed),
];

/// What:
///  One plan planted in a fixture's plan folder,
///  under the name the plan's grammar gives it,
///  and answer its path.
/// Why:
///  The plan is written by the removal's own writer,
///  so these controls ask the reader about a record a removal could really leave
///  (issue 6595).
fn plant_a_plan(
    folder: &Path,
    clone_guid: DatasetGuid,
    clone_dataset: &str,
    head: HeadWord,
    history: &[(u64, SnapshotOutcome)],
) -> PathBuf {
    let clone: ACloneAPlanNames =
        ACloneAPlanNames::a_fixture_clone(common_of(folder), clone_dataset, clone_guid);
    let first_write: APlanAControlPlants = APlanAControlPlants::a_removals_first_write(clone);
    let attempts: Vec<PlanAttempt> = history
        .iter()
        .map(|(attempt, outcome)| return a_line_of(&first_write, *attempt, *outcome))
        .collect();
    let mut plan: APlanAControlPlants = first_write;
    plan.attempts = attempts;
    plan.head = head;
    return plant(&plan);
}

/// A planted plan filed under a name that is not its own key,
/// which is what a foreign entry and a plan whose fields disagree with its name
/// are.
fn refiled(
    plan: &Path,
    name: &str,
) -> PathBuf {
    let wanted: PathBuf = plan
        .parent()
        .expect("the plan stands in its folder")
        .join(name);
    std::fs::rename(plan, wanted.as_path()).expect("the planted plan is filed under another name");
    return wanted;
}

/// A missing plan folder holds no plan,
/// which is the state every add meets with no removal standing.
#[test]
fn a_missing_folder_holds_no_plan() {
    let root: PathBuf = crate::test_support::fixture("plans-missing");
    let common: PathBuf = root.join(".git");
    assert_eq!(
        the_plan_naming_the_clone(
            common.as_path(),
            Some(clone_guid()),
            "tank/clones/add-cache-layer"
        ),
        Ok(None)
    );
}

/// A plan for another clone is no stop,
/// and one whose field names this clone's GUID is this clone's.
#[test]
fn a_plan_naming_this_clone_by_its_field_is_found() {
    let (_root, folder) = with_plans("plans-naming");
    plant_a_plan(
        folder.as_path(),
        a_guid("111"),
        "tank/clones/other",
        HeadWord::Present,
        A_PLANS_FIRST_WRITE,
    );
    assert_eq!(
        the_plan_naming_the_clone(
            common_of(folder.as_path()),
            Some(clone_guid()),
            "tank/clones/add-cache-layer"
        ),
        Ok(None)
    );
    let ours: PathBuf = plant_a_plan(
        folder.as_path(),
        clone_guid(),
        "tank/clones/add-cache-layer",
        HeadWord::Requested,
        A_PLANS_FIRST_WRITE,
    );
    let answer: Option<APlanThatNamesTheClone> = the_plan_naming_the_clone(
        common_of(folder.as_path()),
        Some(clone_guid()),
        "tank/clones/add-cache-layer",
    )
    .expect("a readable plan naming the clone is found");
    let found: APlanThatNamesTheClone = answer.expect("the plan names this clone");
    assert_eq!(found.plan, ours.as_path());
    assert_eq!(found.clone_dataset, "tank/clones/add-cache-layer");
    assert_eq!(found.clone_guid, clone_guid());
    assert_eq!(found.head, PlanHeadWord::Requested);
    assert!(
        found.every_tuple_planned,
        "the fixture's one attempt line still says `planned`"
    );
}

/// A plan filed under this clone's own GUID is this clone's whatever its fields
/// say,
/// which is the key's own proof.
#[test]
fn a_plan_under_this_clones_key_is_this_clones() {
    let (_root, folder) = with_plans("plans-key");
    let planted: PathBuf = plant_a_plan(
        folder.as_path(),
        a_guid("999"),
        "tank/clones/another-name",
        HeadWord::Gone,
        A_PLANS_FIRST_WRITE,
    );
    let ours: PathBuf = refiled(planted.as_path(), clone_guid().file_name().as_str());
    let answer: Option<APlanThatNamesTheClone> = the_plan_naming_the_clone(
        common_of(folder.as_path()),
        Some(clone_guid()),
        "tank/clones/add-cache-layer",
    )
    .expect("the plan under the key is found");
    let found: APlanThatNamesTheClone = answer.expect("the key names this clone");
    assert_eq!(found.plan, ours.as_path());
    assert_eq!(found.head, PlanHeadWord::Gone);
}

/// A plan naming the clone's dataset is found even when the add names no GUID,
/// which is how both of the add's own reads ask.
#[test]
fn a_plan_naming_the_clones_dataset_is_found() {
    let (_root, folder) = with_plans("plans-dataset");
    plant_a_plan(
        folder.as_path(),
        a_guid("333"),
        "tank/clones/add-cache-layer",
        HeadWord::Present,
        A_PLANS_FIRST_WRITE,
    );
    let found: Option<APlanThatNamesTheClone> = the_plan_naming_the_clone(
        common_of(folder.as_path()),
        None,
        "tank/clones/add-cache-layer",
    )
    .expect("the plan is found by its dataset");
    assert_eq!(
        found.map(|plan| return plan.head),
        Some(PlanHeadWord::Present)
    );
}

/// An attempt line whose outcome left `planned` is a plan that witnesses a
/// destroy,
/// which is the fact the cancel command's clause reads.
#[test]
fn an_attempt_past_planned_is_not_a_plan_witnessing_nothing() {
    let (_root, folder) = with_plans("plans-destroyed");
    plant_a_plan(
        folder.as_path(),
        a_guid("555"),
        "tank/clones/add-cache-layer",
        HeadWord::Present,
        PLANNED_AND_THEN_DESTROYED,
    );
    let found: APlanThatNamesTheClone = the_plan_naming_the_clone(
        common_of(folder.as_path()),
        None,
        "tank/clones/add-cache-layer",
    )
    .expect("the plan is found by its dataset")
    .expect("the plan names this clone");
    assert!(
        !found.every_tuple_planned,
        "a destroyed attempt witnesses a destroy"
    );
}

/// A plan under a clone's GUID that cannot be read stops the add,
/// because nothing proves it does not name this clone.
#[test]
fn an_unreadable_plan_stops_the_add() {
    let (_root, folder) = with_plans("plans-unreadable");
    let unreadable: PathBuf = plant_a_plan(
        folder.as_path(),
        a_guid("444"),
        "tank/clones/other",
        HeadWord::Present,
        &[],
    );
    let mut permissions: std::fs::Permissions = std::fs::metadata(unreadable.as_path())
        .expect("the plan's metadata reads")
        .permissions();
    permissions.set_mode(0o000);
    std::fs::set_permissions(unreadable.as_path(), permissions).expect("the plan's mode is closed");
    let stop: PlanUnreadable = the_plan_naming_the_clone(
        common_of(folder.as_path()),
        Some(clone_guid()),
        "tank/clones/add-cache-layer",
    )
    .expect_err("the unreadable plan stops the add");
    assert_eq!(stop.plan, unreadable.as_path());
    assert!(
        stop.sentence.contains("could not be read"),
        "{}",
        stop.sentence
    );
    let mut opened: std::fs::Permissions = std::fs::metadata(unreadable.as_path())
        .expect("the plan's metadata reads")
        .permissions();
    opened.set_mode(0o600);
    std::fs::set_permissions(unreadable.as_path(), opened)
        .expect("the plan's mode is opened for removal");
}

/// A file that is not filed under a clone's key is foreign and read by nobody,
/// which is what the plan design says the folder holds besides its plans and staging names.
#[test]
fn a_foreign_entry_is_not_read() {
    let (_root, folder) = with_plans("plans-foreign");
    let planted: PathBuf = plant_a_plan(
        folder.as_path(),
        clone_guid(),
        "tank/clones/add-cache-layer",
        HeadWord::Present,
        &[],
    );
    refiled(planted.as_path(), "foreign-entry");
    assert_eq!(
        the_plan_naming_the_clone(
            common_of(folder.as_path()),
            Some(clone_guid()),
            "tank/clones/add-cache-layer"
        ),
        Ok(None)
    );
}

/// What:
///  A plan's file name is the clone's canonical GUID and nothing else.
/// Why:
///  The removal's plan reader calls this check rather than carrying its own copy,
///  so a name this control passes is a name both readers accept,
///  and a leading zero or a sign is a second spelling of one GUID.
#[test]
fn a_plan_name_is_the_clones_canonical_guid_and_nothing_else() {
    assert!(is_canonical_guid_name("7828944690444299989"));
    assert!(is_canonical_guid_name("0"));
    assert!(
        is_canonical_guid_name("78289446904442999890"),
        "a twenty-digit GUID is canonical"
    );
    for refused in [
        "",
        "007",
        "+1",
        "7a",
        ".7828944690444299989",
        "7828944690444299989\n",
    ] {
        assert!(
            !is_canonical_guid_name(refused),
            "`{refused}` is not a plan name"
        );
    }
    let twenty_one_digits: String = format!("1{}", "0".repeat(20));
    assert!(
        !is_canonical_guid_name(twenty_one_digits.as_str()),
        "21 digits are past the bound"
    );
}
