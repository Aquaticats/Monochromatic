//! Controls for the add's read of the plan folder:
//!  a missing folder holds no plan,
//!  a plan naming another clone is no stop,
//!  a plan whose name is this clone's GUID stops the add without a read,
//!  a plan naming this clone's field stops it,
//!  and a plan under this clone's key that cannot be read stops it too.
//!  The name check those controls rest on is controlled here,
//!  because the removal's plan reader calls this one rather than repeating it.

use crate::worktree_clone_binding_record::plan_folder_read::{PLAN_HEADER, PlanStop, is_canonical_guid_name, refuse_while_a_plan_names_the_clone};
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// One fixture's common directory,
/// with the plan folder made.
fn with_plans(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let common: PathBuf = root.join(".git");
    let folder: PathBuf = plans_folder(common.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder is made");
    return (root, folder);
}

/// The common directory above one plan folder.
fn common_of(folder: &Path) -> &Path {
    return folder.parent().and_then(|path| return path.parent()).expect("the common directory is the grandparent");
}

/// The fixture's clone.
fn clone_guid() -> DatasetGuid {
    return DatasetGuid::parse("4821193048571903102").expect("the fixture guid reads");
}

/// One plan's bytes,
/// its header and the two fields this reader asks.
fn plan_text(clone_guid: &str, clone_dataset: &str) -> String {
    return format!("{PLAN_HEADER}\ncloneDataset={clone_dataset}\ncloneGuid={clone_guid}\nhead=present\n");
}

/// A missing plan folder holds no plan,
/// which is the state every add meets until step 14.
#[test]
fn a_missing_folder_holds_no_plan() {
    let root: PathBuf = crate::test_support::fixture("plans-missing");
    let common: PathBuf = root.join(".git");
    assert_eq!(refuse_while_a_plan_names_the_clone(common.as_path(), Some(clone_guid()), "tank/clones/add-cache-layer"), Ok(()));
    crate::test_support::remove(root.as_path());
}

/// A plan for another clone is no stop,
/// and one whose field names this clone's GUID stops the add.
#[test]
fn a_plan_naming_this_clone_by_its_field_stops_the_add() {
    let (root, folder) = with_plans("plans-naming");
    let other: PathBuf = folder.join("111");
    std::fs::write(other.as_path(), plan_text("111", "tank/clones/other").as_bytes()).expect("the other plan is written");
    assert_eq!(refuse_while_a_plan_names_the_clone(common_of(folder.as_path()), Some(clone_guid()), "tank/clones/add-cache-layer"), Ok(()));
    let ours: PathBuf = folder.join("222");
    std::fs::write(ours.as_path(), plan_text(clone_guid().file_name().as_str(), "tank/clones/add-cache-layer").as_bytes()).expect("the plan is written");
    let stop: PlanStop = refuse_while_a_plan_names_the_clone(common_of(folder.as_path()), Some(clone_guid()), "tank/clones/add-cache-layer").expect_err("the plan stops the add");
    assert!(matches!(stop, PlanStop::NamesTheClone(_)), "{}", stop.sentence());
    assert_eq!(stop.plan(), ours.as_path());
    crate::test_support::remove(root.as_path());
}

/// A plan filed under this clone's own key stops the add without being read,
/// which is the key's own proof.
#[test]
fn a_plan_under_this_clones_key_stops_the_add() {
    let (root, folder) = with_plans("plans-key");
    let ours: PathBuf = folder.join(clone_guid().file_name());
    std::fs::write(ours.as_path(), b"not a plan at all").expect("the file under the key is written");
    let stop: PlanStop = refuse_while_a_plan_names_the_clone(common_of(folder.as_path()), Some(clone_guid()), "tank/clones/add-cache-layer").expect_err("the key stops the add");
    assert!(matches!(stop, PlanStop::NamesTheClone(_)), "{}", stop.sentence());
    crate::test_support::remove(root.as_path());
}

/// A plan naming the clone's dataset is a stop too.
#[test]
fn a_plan_naming_the_clones_dataset_stops_the_add() {
    let (root, folder) = with_plans("plans-dataset");
    std::fs::write(folder.join("333"), plan_text("333", "tank/clones/add-cache-layer").as_bytes()).expect("the plan is written");
    let stop: PlanStop = refuse_while_a_plan_names_the_clone(common_of(folder.as_path()), None, "tank/clones/add-cache-layer").expect_err("the plan stops the add");
    assert!(matches!(stop, PlanStop::NamesTheClone(_)), "{}", stop.sentence());
    crate::test_support::remove(root.as_path());
}

/// A plan under a clone's GUID that cannot be read stops the add,
/// because nothing proves it does not name this clone.
#[test]
fn an_unreadable_plan_stops_the_add() {
    let (root, folder) = with_plans("plans-unreadable");
    let unreadable: PathBuf = folder.join("444");
    std::fs::write(unreadable.as_path(), plan_text("444", "tank/clones/other").as_bytes()).expect("the plan is written");
    let mut permissions: std::fs::Permissions = std::fs::metadata(unreadable.as_path()).expect("the plan's metadata reads").permissions();
    permissions.set_mode(0o000);
    std::fs::set_permissions(unreadable.as_path(), permissions).expect("the plan's mode is closed");
    let stop: PlanStop = refuse_while_a_plan_names_the_clone(common_of(folder.as_path()), Some(clone_guid()), "tank/clones/add-cache-layer").expect_err("the unreadable plan stops the add");
    assert!(matches!(stop, PlanStop::Unreadable(_)), "{}", stop.sentence());
    let mut opened: std::fs::Permissions = std::fs::metadata(unreadable.as_path()).expect("the plan's metadata reads").permissions();
    opened.set_mode(0o600);
    std::fs::set_permissions(unreadable.as_path(), opened).expect("the plan's mode is opened for removal");
    crate::test_support::remove(root.as_path());
}

/// A file that is not filed under a clone's key is foreign and read by nobody,
/// which is what the plan design says the folder holds besides its plans and staging names.
#[test]
fn a_foreign_entry_is_not_read() {
    let (root, folder) = with_plans("plans-foreign");
    std::fs::write(folder.join("foreign-entry"), plan_text(clone_guid().file_name().as_str(), "tank/clones/add-cache-layer").as_bytes()).expect("the foreign entry is written");
    assert_eq!(refuse_while_a_plan_names_the_clone(common_of(folder.as_path()), Some(clone_guid()), "tank/clones/add-cache-layer"), Ok(()));
    crate::test_support::remove(root.as_path());
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
    assert!(is_canonical_guid_name("78289446904442999890"), "a twenty-digit GUID is canonical");
    for refused in ["", "007", "+1", "7a", ".7828944690444299989", "7828944690444299989\n"] {
        assert!(!is_canonical_guid_name(refused), "`{refused}` is not a plan name");
    }
    let twenty_one_digits: String = format!("1{}", "0".repeat(20));
    assert!(!is_canonical_guid_name(twenty_one_digits.as_str()), "21 digits are past the bound");
}
