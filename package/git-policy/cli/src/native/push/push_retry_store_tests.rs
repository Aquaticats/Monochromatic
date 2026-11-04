//! Controls for the plan on disk: a plan is read back whole, a newer plan of a branch replaces the older, a removed plan is gone, and a
//! file that holds no plan is removed instead of being said at every start.

use super::*;
use crate::test_support::{fixture, remove as remove_fixture};
use crate::work_item::WorkKind;

/// The oid of the first commit of the controls.
const FIRST_OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// The oid of the second commit of the controls.
const SECOND_OID: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn plan(branch: &str, oid: &str, attempt: u32) -> StoredPlan {
    return StoredPlan {
        git_dir: PathBuf::from("/srv/repo/.git"),
        item: WorkItem {
            kind: WorkKind::Push,
            repository: String::from("/srv/repo"),
            git_dir: String::from("/srv/repo/.git"),
            branch_ref: branch.to_owned(),
            oid: oid.to_owned(),
            parent: String::new(),
            created: String::from("2026-11-03T05:40:00.000Z"),
            tries: 0,
        },
        attempt,
    };
}

/// A plan comes back as it went in, one per branch.
#[test]
fn a_saved_plan_is_read_back_whole() {
    let root: PathBuf = fixture("retry-store-roundtrip");
    let main: StoredPlan = plan("refs/heads/main", FIRST_OID, 3);
    let topic: StoredPlan = plan("refs/heads/topic", SECOND_OID, 1);
    save(root.as_path(), &main).expect("saved");
    save(root.as_path(), &topic).expect("saved");
    let mut loaded: Vec<StoredPlan> = load(root.as_path());
    loaded.sort_by(|left, right| { return left.item.branch_ref.cmp(&right.item.branch_ref); });
    assert_eq!(loaded, vec![main, topic]);
    remove_fixture(root.as_path());
}

/// A plan saved for a branch replaces the earlier plan of that branch, with another try number or another commit.
#[test]
fn a_newer_plan_of_a_branch_replaces_the_older() {
    let root: PathBuf = fixture("retry-store-replace");
    save(root.as_path(), &plan("refs/heads/main", FIRST_OID, 1)).expect("saved");
    save(root.as_path(), &plan("refs/heads/main", FIRST_OID, 2)).expect("saved");
    let newest: StoredPlan = plan("refs/heads/main", SECOND_OID, 1);
    save(root.as_path(), &newest).expect("saved");
    assert_eq!(load(root.as_path()), vec![newest]);
    assert_eq!(std::fs::read_dir(root.as_path()).expect("directory").count(), 1, "the earlier files went");
    remove_fixture(root.as_path());
}

/// A removed plan is gone, and the plan of another branch is not touched.
#[test]
fn a_removed_plan_is_gone_and_another_branch_stays() {
    let root: PathBuf = fixture("retry-store-remove");
    let kept: StoredPlan = plan("refs/heads/topic", SECOND_OID, 1);
    save(root.as_path(), &plan("refs/heads/main", FIRST_OID, 1)).expect("saved");
    save(root.as_path(), &kept).expect("saved");
    remove(root.as_path(), Path::new("/srv/repo/.git"), "refs/heads/main");
    assert_eq!(load(root.as_path()), vec![kept]);
    remove_fixture(root.as_path());
}

/// A file that holds no plan is not a plan, and is removed so that it is not said again.
#[test]
fn a_file_that_holds_no_plan_is_removed() {
    let root: PathBuf = fixture("retry-store-garbage");
    let garbage: PathBuf = root.join(format!("nothing.{PLAN_EXTENSION}"));
    std::fs::write(garbage.as_path(), "not a plan").expect("file");
    assert!(load(root.as_path()).is_empty());
    assert!(!garbage.exists());
    remove_fixture(root.as_path());
}
