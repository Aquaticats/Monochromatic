//! Controls for the plan on disk:
//! a plan is read back whole,
//! a newer plan of a branch replaces the older,
//! a removed plan is gone,
//! and a
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
            repository: PathBuf::from("/srv/repo"),
            git_dir: PathBuf::from("/srv/repo/.git"),
            branch_ref: branch.to_owned(),
            oid: oid.to_owned(),
            parent: String::new(),
            created: String::from("2026-11-03T05:40:00.000Z"),
            tries: 0,
        },
        attempt,
    };
}

/// A plan comes back as it went in,
/// one per branch.
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

/// What:
///  A plan whose Git directory is not text is read back whole.
/// Why:
///   The plan names the directory its push belongs to,
/// and a restart finds the push by that name,
/// so a name that lost the path's bytes would leave the push untried and its file unremoved.
#[test]
fn a_plan_of_a_directory_that_is_not_text_is_read_back_whole() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("retry-store-non-utf8");
    let git_dir: PathBuf = root.join(OsStr::from_bytes(b"c\xffmmon"));
    let worktree: PathBuf = root.join(OsStr::from_bytes(b"w\xffrktree"));
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let saved: StoredPlan = StoredPlan {
        git_dir: git_dir.clone(),
        item: WorkItem { repository: worktree, git_dir: git_dir.clone(), ..plan("refs/heads/main", FIRST_OID, 2).item },
        attempt: 2,
    };
    save(root.as_path(), &saved).expect("saved");
    assert_eq!(load(root.as_path()), vec![saved.clone()], "the plan comes back as it went in");
    remove_fixture(root.as_path());
}

/// What:
///  A service that starts again finds the plan of a directory that is not text,
/// and tries it when it is due.
/// Why:
///   This is the plan's whole purpose:
/// a push that waits across a restart is pushed again.
/// The list is keyed by the directory the drain names,
/// which is the real one.
#[test]
fn a_restored_list_keeps_the_plan_of_a_directory_that_is_not_text() {
    use crate::push_retry::{Due, PushRetries};
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::time::{Duration, Instant};
    let root: PathBuf = fixture("retry-store-non-utf8-restored");
    let git_dir: PathBuf = root.join(OsStr::from_bytes(b"c\xffmmon"));
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let saved: StoredPlan = StoredPlan { git_dir: git_dir.clone(), ..plan("refs/heads/main", FIRST_OID, 1) };
    save(root.as_path(), &saved).expect("saved");
    let mut retries: PushRetries = PushRetries::restored(Duration::from_secs(1), root.as_path());
    assert_eq!(retries.directories(), vec![git_dir.clone()], "the plan is held under the directory it was saved for");
    let later: Instant = Instant::now() + Duration::from_secs(600);
    let due: Vec<Due> = retries.take_due(git_dir.as_path(), later);
    assert_eq!(due.len(), 1, "the plan of this directory is due");
    assert_eq!(due[0].git_dir, git_dir);
    remove_fixture(root.as_path());
}

/// A plan saved for a branch replaces the earlier plan of that branch,
/// with another try number or another commit.
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

/// A removed plan is gone,
/// and the plan of another branch is not touched.
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

/// A file that holds no plan is not a plan,
/// and is removed so that it is not said again.
#[test]
fn a_file_that_holds_no_plan_is_removed() {
    let root: PathBuf = fixture("retry-store-garbage");
    let garbage: PathBuf = root.join(format!("nothing.{PLAN_EXTENSION}"));
    std::fs::write(garbage.as_path(), "not a plan").expect("file");
    assert!(load(root.as_path()).is_empty());
    assert!(!garbage.exists());
    remove_fixture(root.as_path());
}
