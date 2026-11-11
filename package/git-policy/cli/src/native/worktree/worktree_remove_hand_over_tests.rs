//! Controls of the removal of a worktree's subvolume handed to the commit service:
//! the name a handed-over subvolume takes,
//! the rename and the record the removal writes,
//! the service's own delete of it,
//! and the removal that waits for neither.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::service_work::Verdict;
use crate::test_support::{fixture, remove};
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
    let told: RememberingTold = RememberingTold { told_of: std::cell::RefCell::new(Vec::new()), answer: Err(String::from("no service answers here")) };
    hand_over_with(common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), 7, made.as_path(), &told).expect("handed over");
    assert!(!made.exists(), "the path the person named is gone");
    let deleting: PathBuf = parent_path.join(deleting_name(7));
    assert!(deleting.is_dir(), "the subvolume stands under the name built from its id");
    assert_eq!(told.told_of.borrow().clone(), vec![common.clone()], "the service of the common directory is told");
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
/// and a path that stands must be deleted by the id the record holds,
///  which is the one the removal read from the held directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const id = subvolumeId(held); handOver(id); const item = removalItem(common, parent, id);
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
    let child: File = File::open(made.as_path()).expect("child handle");
    let id: u64 = subvolume_id(&child, made.as_path()).expect("the subvolume id the removal records");
    hand_over_with(common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), id, made.as_path(), &NobodyTold).expect("handed over");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), id);
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the service's own call deletes it");
    assert!(!parent_path.join(deleting_name(id)).exists(), "the directory is gone");
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
    let outcome: RemoveOutcome = remove_with(&stand, &RemoveRequest { context: &context, plan: &plan, common_dir: common_dir.as_path() });
    assert!(stand.handed.get(), "the removal handed the subvolume over");
    assert!(!stand.destroyed.get(), "and did not destroy it on the person's path");
    assert_eq!(outcome, RemoveOutcome::Removed, "{outcome:?}");
    remove(root.as_path());
}

/// What:
///  A subvolume another command put at the hidden name after the handed-over one was moved away is not the record's to delete.
/// Why:
///  The record names one subvolume for good,
/// and a name names whatever stands there when the call runs,
/// so a delete that acts on the name alone destroys another person's directory
///  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
/// "Step 1").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOver(); moveAside(hidden); subvolumeCreate(hidden); write(hidden + '/kept.txt'); assert(deleteSubvolumeOf(item) !== Done); assert(exists(hidden + '/kept.txt'));
/// ```
#[test]
fn a_subvolume_that_took_the_hidden_names_place_is_not_the_records_to_delete() {
    let Some((root, made)) = made_subvolume("remove-replacement") else { return };
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent_path: PathBuf = made.parent().expect("parent").to_path_buf();
    let name: OsString = made.file_name().expect("name").to_os_string();
    let parent: File = open_root(parent_path.as_path()).expect("parent handle");
    let child: File = File::open(made.as_path()).expect("child handle");
    let id: u64 = subvolume_id(&child, made.as_path()).expect("the subvolume id the removal records");
    hand_over_with(common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), id, made.as_path(), &NobodyTold).expect("handed over");
    let hidden: PathBuf = parent_path.join(deleting_name(id));
    let moved_away: PathBuf = root.join("the-original");
    std::fs::rename(hidden.as_path(), moved_away.as_path()).expect("the original is moved elsewhere");
    subvolume_create_at(hidden.as_path()).expect("another subvolume takes the name");
    let kept: PathBuf = hidden.join("kept.txt");
    std::fs::write(kept.as_path(), b"another command's bytes\n").expect("the replacement's file");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), id);
    let verdict: Verdict = delete_subvolume_of(&item);
    assert_ne!(verdict, Verdict::Done, "the name holds another subvolume than the record names, so nothing was deleted: {verdict:?}");
    assert!(kept.is_file(), "the record of one subvolume must never delete another subvolume that took its name");
    assert!(moved_away.is_dir(), "the subvolume the record does name was left where it was moved");
    remove(root.as_path());
}

/// What:
///  Two subvolumes that carry the same filesystem-local id on two filesystems are two records.
/// Why:
///  A record's id is its file name and its idempotency key,
/// so it must hold every choice it stands for;
///  a subvolume id is unique on one filesystem only,
/// and a second removal that answered "known" would be renamed with no record naming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = removalItem(common, firstParent, 257); const second = removalItem(common, secondParent, 257); assert(first.id() !== second.id()); enqueueBoth(); assert(both === Added);
/// ```
#[test]
fn two_subvolumes_of_one_id_on_two_filesystems_are_two_records() {
    let root: PathBuf = fixture("remove-equal-ids");
    let common: PathBuf = root.join("common");
    let first_parent: PathBuf = root.join("first-parent");
    let second_parent: PathBuf = root.join("second-parent");
    std::fs::create_dir(common.as_path()).expect("common");
    std::fs::create_dir(first_parent.as_path()).expect("first parent");
    std::fs::create_dir(second_parent.as_path()).expect("second parent");
    let first: WorkItem = removal_item(common.as_path(), first_parent.as_path(), 257);
    let second: WorkItem = removal_item(common.as_path(), second_parent.as_path(), 257);
    assert_ne!(first.id(), second.id(), "the id must hold the directory that holds the removed subvolume, not the id alone");
    assert_eq!(enqueue(work_directory(common.as_path()).as_path(), &first).expect("first enqueue"), crate::work_item::Enqueued::Added);
    assert_eq!(
        enqueue(work_directory(common.as_path()).as_path(), &second).expect("second enqueue"),
        crate::work_item::Enqueued::Added,
        "a subvolume on another filesystem is its own work, whatever id it carries there"
    );
    remove(root.as_path());
}

/// What:
///  A claim on the record,
/// whose consumer already looked and found nothing to delete,
///  is no proof that a delete is still coming,
///  so the removal deletes the renamed subvolume itself.
/// Why:
///  A consumer claims the record before it looks at the directory,
/// so a claim is taken while the directory still stands at its old name;
///  the consumer then finds the hidden path absent,
/// holds the claim while it settles the record,
///  and the removal's own look at the record still sees that claim.
/// A removal that trusted it would leave the worktree's files under a name no record names,
///  which is the state this control builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimRecord(id); assert(deleteSubvolumeOf(item) === Done); handOverWith(...); assert(!exists(hidden));
/// ```
#[test]
fn a_claimed_record_whose_consumer_found_no_directory_is_deleted_here() {
    let Some((root, made)) = made_subvolume("remove-claimed-before-rename") else { return };
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent_path: PathBuf = made.parent().expect("parent").to_path_buf();
    let name: OsString = made.file_name().expect("name").to_os_string();
    let parent: File = open_root(parent_path.as_path()).expect("parent handle");
    let child: File = File::open(made.as_path()).expect("child handle");
    let id: u64 = subvolume_id(&child, made.as_path()).expect("the subvolume id the removal records");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), id);
    let work: PathBuf = work_directory(common.as_path());
    enqueue(work.as_path(), &item).expect("the record");
    let claimed: crate::work_item::Claimed = crate::work_item::claim_next_of(work.as_path(), &crate::claim_time::ClaimTime::read(), &[WorkKind::SubvolumeDelete])
        .expect("claim")
        .expect("the record is claimable");
    assert_eq!(
        delete_subvolume_of(&claimed.item),
        Verdict::Done,
        "a directory that is not at its hidden name yet is nothing to delete, and the consumer has yet to settle the claim"
    );
    hand_over_with(common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), id, made.as_path(), &NobodyTold).expect("handed over");
    let hidden: PathBuf = parent_path.join(deleting_name(id));
    assert!(!hidden.exists(), "a claim is no promise of a delete, so the removal deletes the directory: {}", hidden.display());
    remove(root.as_path());
}

/// What:
///  A record a service settled before the removal renamed the subvolume leaves the rename's subvolume named by nothing,
///  so the removal deletes it here.
/// Why:
///  The record is written before the rename so that a kill leaves one of two safe states,
/// and a service that claims it in the window between
///  the two finds the hidden path absent and settles it,
/// which would leave the worktree's files under a name no record names.
///  The state after that service ran is a record in `done`,
/// which is what this control builds before the hand-over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleRecordInDone(id); handOverWith(...); assert(!exists(hidden)); assert(!exists(pending));
/// ```
#[test]
fn a_renamed_subvolume_whose_record_was_already_settled_is_deleted_here() {
    let Some((root, made)) = made_subvolume("remove-settled-before-rename") else { return };
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent_path: PathBuf = made.parent().expect("parent").to_path_buf();
    let name: OsString = made.file_name().expect("name").to_os_string();
    let parent: File = open_root(parent_path.as_path()).expect("parent handle");
    let child: File = File::open(made.as_path()).expect("child handle");
    let id: u64 = subvolume_id(&child, made.as_path()).expect("the subvolume id the removal records");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), id);
    let work: PathBuf = work_directory(common.as_path());
    enqueue(work.as_path(), &item).expect("the record");
    let claimed: crate::work_item::Claimed = crate::work_item::claim_next_of(work.as_path(), &crate::claim_time::ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("claim").expect("the record is claimable");
    crate::work_item::settle(work.as_path(), &claimed, crate::work_item::DONE).expect("the service settles it before the rename");
    hand_over_with(common.as_path(), &parent, parent_path.as_path(), name.as_os_str(), id, made.as_path(), &NobodyTold).expect("handed over");
    let hidden: PathBuf = parent_path.join(deleting_name(id));
    assert!(!hidden.exists(), "the subvolume the removal renamed is named by no record, so the removal deletes it: {}", hidden.display());
    assert!(!pending_record_path(common.as_path(), &item).exists(), "no record is left for a directory that is gone");
    remove(root.as_path());
}

/// What:
///  A removal whose parent path is not valid UTF-8 is not handed over:
/// nothing is renamed and no record is written.
/// Why:
///  A work item names its path in text,
/// so a path with a byte that is not UTF-8 would be recorded as a different path
///  (`worktree_remove_hand_over.rs`,
/// `record_the_removal`),
/// and a record that names another path can never find the directory it renamed.
/// The removal then
///  destroys the subvolume where it stands,
/// as it did before the hand-over existed,
/// and the command still finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = handOverWith(parentWithBadByte); assert(said.isErr()); assert(stillAtItsName); assert(noPendingRecord);
/// ```
#[test]
fn a_parent_path_no_record_can_name_is_not_handed_over() {
    use std::os::unix::ffi::OsStringExt as _;
    let root: PathBuf = fixture("remove-non-utf8-parent");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent_path: PathBuf = root.join(OsString::from_vec(vec![b'p', 0xff]));
    std::fs::create_dir(parent_path.as_path()).expect("parent");
    let made: PathBuf = parent_path.join("worktree");
    std::fs::create_dir(made.as_path()).expect("worktree");
    let parent: File = open_root(parent_path.as_path()).expect("parent handle");
    let told: RememberingTold = RememberingTold { told_of: std::cell::RefCell::new(Vec::new()), answer: Ok(()) };
    let answer: Result<(), String> = hand_over_with(common.as_path(), &parent, parent_path.as_path(), OsStr::new("worktree"), 47, made.as_path(), &told);
    assert!(answer.is_err(), "a path no record can name is not handed over: {answer:?}");
    assert!(made.is_dir(), "nothing was renamed, so the removal destroys the subvolume in place");
    assert!(told.told_of.borrow().is_empty(), "no service is told of a hand-over that did not happen");
    let pending: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING);
    let recorded: usize = std::fs::read_dir(pending.as_path()).map(|entries| { return entries.flatten().count(); }).unwrap_or(0);
    assert_eq!(recorded, 0, "no record a different path could be read from is written");
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

/// What:
///  A wake nobody hears, which every control that reaches no service and no session bus uses.
/// Why:
///   A hand-over is a record and a rename without a service,
/// and the absence of a wake must not be the removal's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NobodyTold { told() { return undefined } }
/// ```
struct NobodyTold;

impl ToldOfHandOver for NobodyTold {
    fn told(&self, _common_dir: &Path) -> Result<(), String> {
        return Ok(());
    }
}

/// What:
///  A wake that remembers every directory it was told of and answers as the control asked.
/// Why:
///   The hand-over writes the record and renames the subvolume before it tells anyone,
/// so a control reads that the wake was told once,
/// of the right directory,
///  and what the removal did with the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RememberingTold { at = []; answer; told(dir) { this.at.push(dir); return this.answer } }
/// ```
struct RememberingTold {
    /// The directories the wake was told of, in the order it was told.
    told_of: std::cell::RefCell<Vec<PathBuf>>,
    /// What the wake answers.
    answer: Result<(), String>,
}

impl ToldOfHandOver for RememberingTold {
    fn told(&self, common_dir: &Path) -> Result<(), String> {
        self.told_of.borrow_mut().push(common_dir.to_path_buf());
        return self.answer.clone();
    }
}
