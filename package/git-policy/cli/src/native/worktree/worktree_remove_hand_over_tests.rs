//! Controls of the removal of a worktree's subvolume handed to the commit service:
//! the name a handed-over subvolume takes,
//! the rename and the record the removal writes,
//! the service's own delete of it,
//! and the removal that waits for neither.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::service_work::Verdict;
use crate::test_support::{fixture, git_context, remove};
use crate::worktree_remove::{Held, RemoveOperations, RemoveOutcome, RemoveRequest, remove_with};
use crate::worktree_remove_plan::RemovePlan;
use crate::work_item::WorkKind;

/// What:
///  A subvolume inside a fresh disposable root,
/// made with the kernel call the wrapper uses.
/// Why:
///   The rename,
/// the record and the delete are about the object the kernel makes,
/// and nothing else stands for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = (name) => btrfsFixture(name)?.join('made').also((p) => subvolumeCreateAt(p));
/// ```
fn made_subvolume(name: &str) -> Option<(PathBuf, PathBuf)> {
    let root: PathBuf = btrfs_fixture(name)?;
    let made: PathBuf = root.join("worktree");
    subvolume_create_at(made.as_path()).expect("a subvolume");
    return Some((root, made));
}

/// A handed-over subvolume takes a name built from its own id,
/// which no person gives a worktree.
#[test]
fn a_handed_over_subvolume_takes_a_name_built_from_its_id() {
    assert_eq!(deleting_name(0x1f), OsString::from("cli-git-deleting-000000000000001f"));
    assert!(DELETING_PREFIX.starts_with("cli-git-"));
    assert_eq!(removal_item(Path::new("/r/.git"), Path::new("/r"), 0x2a).oid, "000000000000000000000000000000000000002a");
}

/// What:
///  The hand-over renames the subvolume out of the person's way and leaves a record that names the exact path,
///  and a wake that fails is not the removal's failure.
/// Why:
///  The person's command must not wait for the delete,
/// and the record is the only thing that outlives it:
/// a
///  record written before the rename and a directory renamed after it are both safe orders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWith({ ..., told: () => { throw new Error('no service') } });
/// assert(!exists(worktree)); assert(exists(deleting)); assert(pendingNames(commonDir).includes(id));
/// ```
#[test]
fn the_hand_over_renames_the_subvolume_and_records_where_it_stands() {
    let Some((root, made)) = made_subvolume("remove-hand-over") else { return };
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent_path: PathBuf = made.parent().expect("parent").to_path_buf();
    let name: OsString = made.file_name().expect("name").to_os_string();
    let parent: File = open_root(parent_path.as_path()).expect("parent handle");
    let woken: std::cell::RefCell<Vec<PathBuf>> = std::cell::RefCell::new(Vec::new());
    let told = |_context: &GitContext, at: &Path| -> Result<(), String> {
        woken.borrow_mut().push(at.to_path_buf());
        return Err(String::from("no service answers here"));
    };
    hand_over_with(&git_context(), common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), 7, made.as_path(), &told).expect("handed over");
    assert!(!made.exists(), "the path the person named is gone");
    let deleting: PathBuf = parent_path.join(deleting_name(7));
    assert!(deleting.is_dir(), "the subvolume stands under the name built from its id");
    assert_eq!(woken.borrow().clone(), vec![common.clone()], "the service of the common directory is told");
    let pending: PathBuf = crate::worktree_remove_hand_over::pending_record_path(common.as_path(), &removal_item(common.as_path(), parent_path.as_path(), 7));
    assert!(pending.is_file(), "the record that outlives the command is written: {}", pending.display());
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), 7);
    assert_eq!(item.kind, WorkKind::SubvolumeDelete);
    assert_eq!(item.parent, deleting.display().to_string(), "the record names the exact path the rename made");
    remove(root.as_path());
}

/// What:
///  The service deletes the subvolume a record names,
///  and a record whose path is gone deletes nothing.
/// Why:
///  The delete happens after the person's command ended,
/// so a path that is gone must be a finished item and never an error,
/// and a path that stands must be deleted by its id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(deleteSubvolumeOf(item) === Done); assert(!exists(deleting)); assert(deleteSubvolumeOf(item) === Done);
/// ```
#[test]
fn the_service_deletes_the_subvolume_a_record_names_and_a_gone_path_is_done() {
    let Some((root, made)) = made_subvolume("remove-service-delete") else { return };
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent_path: PathBuf = made.parent().expect("parent").to_path_buf();
    let name: OsString = made.file_name().expect("name").to_os_string();
    let parent: File = open_root(parent_path.as_path()).expect("parent handle");
    hand_over_with(&git_context(), common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), 11, made.as_path(), &|_: &GitContext, _: &Path| { return Ok(()); }).expect("handed over");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), 11);
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the service's own call deletes it");
    assert!(!parent_path.join(deleting_name(11)).exists(), "the directory is gone");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "a path that is gone is nothing to delete");
    remove(root.as_path());
}

/// What:
///  A removal hands the subvolume over and never destroys it in place,
///  so the person waits for one rename.
/// Why:
///  This is the promise of the change:
/// no subvolume destroy and nothing the cleaner does sits on a person's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remove = removeWith(handingOverStand, request); assert(handed === true); assert(destroyed === false); assert(remove === Removed);
/// ```
#[test]
fn a_removal_hands_the_subvolume_over_and_does_not_destroy_it_here() {
    let root: PathBuf = fixture("remove-never-destroys");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    crate::test_support::git(repo.as_path(), &["add", "tracked.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let written: PathBuf = root.join("topic");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", written.to_str().expect("utf8")]);
    let target: PathBuf = std::fs::canonicalize(written.as_path()).expect("canonical");
    crate::test_support::age(target.join("tracked.txt").as_path());
    crate::test_support::git(target.as_path(), &["update-index", "--refresh"]);
    let stand = HandingOverStand { target: target.clone(), handed: std::cell::Cell::new(false), destroyed: std::cell::Cell::new(false) };
    let context = crate::test_support::git_context();
    let plan = RemovePlan { target: target.clone(), operand: target.as_os_str().to_os_string(), force: false };
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let outcome: RemoveOutcome = remove_with(&stand, &RemoveRequest { context: &context, plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    assert!(stand.handed.get(), "the removal handed the subvolume over");
    assert!(!stand.destroyed.get(), "and did not destroy it on the person's path");
    assert_eq!(outcome, RemoveOutcome::Removed, "{outcome:?}");
    remove(root.as_path());
}

/// A stand-in whose kernel is the real one for every proof,
/// whose destroy deletes the directory,
/// and which records which of the two ways the removal took.
struct HandingOverStand {
    /// The worktree the removal is about.
    target: PathBuf,
    /// Whether the hand-over was asked for.
    handed: std::cell::Cell<bool>,
    /// Whether the destroy in place was asked for.
    destroyed: std::cell::Cell<bool>,
}

impl RemoveOperations for HandingOverStand {
    fn is_subvolume_root(&self, path: &Path) -> bool {
        return path == self.target.as_path();
    }

    fn removal_allowed(&self, _path: &Path) -> bool {
        return true;
    }

    fn check_clean(&self, _request: &RemoveRequest<'_>, _admin: &Path, _common_dir: &Path) -> Result<(), String> {
        return Ok(());
    }

    fn destroy(&self, held: &Held) -> Result<(), String> {
        self.destroyed.set(true);
        return std::fs::remove_dir_all(held.parent_path.join(held.name.as_os_str())).map_err(|error| { return error.to_string(); });
    }

    fn hand_over(&self, _context: &GitContext, _common_dir: &Path, _held: &Held, _target: &Path) -> Result<(), String> {
        self.handed.set(true);
        return Ok(());
    }
}
