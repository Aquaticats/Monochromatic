//! Controls of the removal of a worktree's subvolume handed to the commit service:
//! the name a handed-over subvolume takes,
//! the rename and the record the removal writes,
//! the service's own delete of it,
//! and the removal that waits for neither.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::service_work::Verdict;
use crate::shadow_pool::pool_root_for;
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
///  A wake nobody hears,
/// which every control that reaches no service and no session bus uses.
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
    /// The directories the wake was told of,
    /// in the order it was told.
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

/// What:
///  A repository with one linked worktree,
///  the common directory of it,
///  and the canonical path of the worktree.
/// Why:
///   Every control about a handed-over worktree removal needs a registration Git knows,
///  because the removal is Git's own and its proof is that registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithALinkedWorktree('name'); // { root, repo, common, worktree }
/// ```
struct Linked {
    /// The fixture directory that holds the repository and the worktree.
    root: PathBuf,
    /// Its common Git directory,
    /// which is where the record waits and where Git is started.
    common: PathBuf,
    /// The linked worktree.
    worktree: PathBuf,
}

impl Linked {
    /// The main checkout of the fixture,
    /// which is what a person runs `git -C` against.
    fn repo_root(&self) -> PathBuf {
        return self.root.join("repo");
    }
}

fn a_repository_with_a_linked_worktree(name: &str) -> Linked {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let tracked: PathBuf = repo.join("tracked.txt");
    std::fs::write(tracked.as_path(), b"tracked\n").expect("tracked");
    crate::test_support::age(tracked.as_path());
    crate::test_support::git(repo.as_path(), &["add", "tracked.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let written: PathBuf = root.join("topic");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", written.to_str().expect("utf8")]);
    let worktree: PathBuf = std::fs::canonicalize(written.as_path()).expect("canonical");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    crate::worktree_added_by_the_wrapper::record(common.as_path(), worktree.as_path()).expect("the record of a worktree this program made");
    return Linked { root, common, worktree };
}

/// What:
///  The same repository,
///  whose first commit also carries the ignore rules a control writes.
/// Why:
///  An ignored file is the one kind of work Git's own removal deletes without a word,
///  so a control about one needs a rule that really ignores it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithALinkedWorktreeAndIgnoreRules('name', '.claude/\n*.log\n');
/// ```
fn a_repository_with_a_linked_worktree_and_ignore_rules(name: &str, rules: &str) -> Linked {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let tracked: PathBuf = repo.join("tracked.txt");
    std::fs::write(tracked.as_path(), b"tracked\n").expect("tracked");
    let ignored: PathBuf = repo.join(".gitignore");
    std::fs::write(ignored.as_path(), rules.as_bytes()).expect("the ignore rules");
    crate::test_support::age(tracked.as_path());
    crate::test_support::age(ignored.as_path());
    crate::test_support::git(repo.as_path(), &["add", "--", "tracked.txt", ".gitignore"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let written: PathBuf = root.join("topic");
    crate::test_support::git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", written.to_str().expect("utf8")]);
    let worktree: PathBuf = std::fs::canonicalize(written.as_path()).expect("canonical");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    crate::worktree_added_by_the_wrapper::record(common.as_path(), worktree.as_path()).expect("the record of a worktree this program made");
    return Linked { root, common, worktree };
}

/// The commit as the notice words it,
/// with no abbreviation to resolve.
fn whole_name(commit: &str) -> String {
    return commit.to_owned();
}

/// What:
///  The record a landed sweep writes for this worktree,
///  with the worktree it compared the ignored files with,
///  and no service told.
/// Why:
///  This is the exact state a merge leaves: the record queued,
///  the worktree standing,
///  and the destruction owed to the commit service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = handedOver(linked, linked.repo);
/// ```
fn handed_over(linked: &Linked, compared_with: Option<&Path>) -> WorkItem {
    let registration: PathBuf = linked.common.join("worktrees").join("topic");
    let told = |_: &GitContext, _: &Path| -> Result<(), String> { return Err(String::from("no service answers here")); };
    return hand_over_worktree_removal(&git_context(), linked.common.as_path(), linked.worktree.as_path(), registration.as_path(), compared_with, &told).expect("handed over");
}

/// The words of every kept-worktree notice this repository holds,
/// read the way a command reads them.
fn kept_notice_texts(common: &Path) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for notice in crate::worktree_kept_notices::kept_worktree_notices(common) {
        let mut written: Vec<u8> = Vec::new();
        notice.show(&mut written).expect("a buffer takes the words");
        texts.push(String::from_utf8(written).expect("the words are UTF-8"));
    }
    return texts;
}

/// What:
///  The record a removal writes for a whole worktree names Git's own removal,
///  and its id comes from the path and the registration being removed,
///  so two sweeps of one landing are one record and a worktree added again at the same path is a new one.
/// Why:
///  The queue keys an item by its id,
/// and a person reading two merges that both found the same worktree must read one record,
///  not two that two service rounds race on;
///  a `done` record is never removed,
///  so a worktree added again at the same path must not be taken for the one that went.
#[test]
fn a_worktree_removal_record_is_keyed_by_its_path_and_registration_and_names_git() {
    let root: PathBuf = fixture("worktree-removal-record");
    let registration: PathBuf = root.join("registration");
    std::fs::create_dir(registration.as_path()).expect("a registration");
    let target: PathBuf = root.join("topic");
    let one: WorkItem = worktree_removal_item(Path::new("/r/.git"), target.as_path(), registration.as_path());
    let again: WorkItem = worktree_removal_item(Path::new("/r/.git"), target.as_path(), registration.as_path());
    let other: WorkItem = worktree_removal_item(Path::new("/r/.git"), root.join("elsewhere").as_path(), registration.as_path());
    // A worktree added again at the same path has a registration of its own, with an identity of its own.
    let added_again: PathBuf = root.join("registration-again");
    std::fs::create_dir(added_again.as_path()).expect("the registration a second worktree at one path gets");
    let second: WorkItem = worktree_removal_item(Path::new("/r/.git"), target.as_path(), added_again.as_path());
    assert_ne!(one.id(), second.id(), "a worktree added again at one path is a new removal");
    remove(root.as_path());
    assert_eq!(one.id(), again.id(), "one worktree is one record");
    assert_ne!(one.id(), other.id(), "two worktrees are two records");
    assert_eq!(one.oid.len(), 40, "the id keeps the shape every reader of a record requires: {}", one.oid);
    assert!(one.oid.as_bytes().iter().all(u8::is_ascii_hexdigit), "{}", one.oid);
    assert_eq!(one.branch_ref, WORKTREE_REMOVAL_REFERENCE, "the record says the removal is Git's own");
    assert_eq!(one.parent, target.display().to_string(), "the record names the worktree to remove");
    assert_eq!(one.repository, PathBuf::from("/r/.git"), "the record names the directory Git is started in, which outlives the merge");
    assert_eq!(one.kind, WorkKind::SubvolumeDelete);
}

/// What:
///  A handed-over record is written before the service is told,
///  and a wake that fails is not the hand-over's failure:
///  the worktree stands,
///  and the record outlives the command.
/// Why:
///  The record is the only thing that outlives the merge,
///  so a merge that wrote none would leave a worktree nothing ever removes.
#[test]
fn a_handed_over_worktree_is_recorded_before_it_is_told() {
    let linked: Linked = a_repository_with_a_linked_worktree("worktree-hand-over-record");
    let woken: std::cell::RefCell<Vec<PathBuf>> = std::cell::RefCell::new(Vec::new());
    let told = |_context: &GitContext, git_dir: &Path| -> Result<(), String> {
        woken.borrow_mut().push(git_dir.to_path_buf());
        return Err(String::from("no service answers here"));
    };
    let registration: PathBuf = linked.common.join("worktrees").join("topic");
    let item: WorkItem = hand_over_worktree_removal(&git_context(), linked.common.as_path(), linked.worktree.as_path(), registration.as_path(), None, &told).expect("handed over");
    assert_eq!(woken.borrow().clone(), vec![linked.common.clone()], "the service of the queue the record is in is told");
    let pending: PathBuf = pending_record_path(linked.common.as_path(), &item);
    assert!(pending.is_file(), "the record outlives the command: {}", pending.display());
    assert!(linked.worktree.is_dir(), "nothing here touches the worktree: Git's own read is its proof");
    remove(linked.root.as_path());
}

/// What:
///  The service's own call removes the worktree a record names,
///  with the registration,
///  its shadow pool and its files,
///  and a path that is gone is a finished record.
/// Why:
///  The removal happens after the merge ended,
/// so it is the service's call that must be complete,
///  and a worktree another attempt or a person already removed is nothing to remove.
#[test]
fn the_service_removes_the_worktree_a_record_names_and_a_gone_one_is_done() {
    let linked: Linked = a_repository_with_a_linked_worktree("worktree-hand-over-service");
    let pool: PathBuf = pool_root_for(linked.common.as_path(), &linked.common.join("worktrees/topic"));
    std::fs::create_dir_all(pool.join("member").join("objects")).expect("a pool as a fill leaves it");
    let registration: PathBuf = linked.common.join("worktrees").join("topic");
    let item: WorkItem = hand_over_worktree_removal(&git_context(), linked.common.as_path(), linked.worktree.as_path(), registration.as_path(), None, &|_: &GitContext, _: &Path| { return Ok(()); })
        .expect("handed over");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "Git removes the worktree the record names");
    assert!(!linked.worktree.exists(), "the worktree directory is gone");
    assert!(!linked.common.join("worktrees/topic").exists(), "and its registration with it");
    assert!(!pool.exists(), "and the pool its commits filled");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "a path that is gone is nothing to remove");
    remove(linked.root.as_path());
}

/// What:
///  A worktree Git would keep is refused with Git's own sentence,
///  and nothing of it is changed.
/// Why:
///  Git's reading of every tracked and untracked file of the worktree is the whole proof of the removal;
///  a person's edit or an untracked file is what it keeps,
///  and the sentence it answers is what a person reads.
#[test]
fn the_service_leaves_a_worktree_git_would_keep_and_says_whose_sentence_refused_it() {
    let linked: Linked = a_repository_with_a_linked_worktree("worktree-hand-over-refused");
    let notes: PathBuf = linked.worktree.join("notes.txt");
    std::fs::write(notes.as_path(), b"what I was in the middle of\n").expect("a file the person wrote");
    let registration: PathBuf = linked.common.join("worktrees").join("topic");
    let item: WorkItem = hand_over_worktree_removal(&git_context(), linked.common.as_path(), linked.worktree.as_path(), registration.as_path(), None, &|_: &GitContext, _: &Path| { return Ok(()); })
        .expect("handed over");
    let verdict: Verdict = delete_subvolume_of(&item);
    match &verdict {
        Verdict::Failed(words) => assert!(words.contains("modified or untracked files"), "Git's own sentence is the reason: {words}"),
        other => panic!("Git refuses a worktree holding a file it would keep: {other:?}"),
    }
    assert!(linked.worktree.is_dir(), "the worktree stays");
    assert_eq!(std::fs::read(notes.as_path()).expect("the file is still there"), b"what I was in the middle of\n");
    remove(linked.root.as_path());
}

/// What:
///  A worktree a command added inside the handed-over one after the hand-over keeps the outer worktree,
///  with the nested worktree's edited and untracked files,
///  and the person reads why.
/// Why:
///  The sweep proved there was no worktree inside it,
///  and that proof was spent when the record was written.
///  The nested checkout sits under an ignored name,
///  which Git's own removal deletes without a word,
///  so the service removed the nested worktree's modified tracked file and its untracked file with the parent
///  (a review of the hand-over,
///  finding 3.2).
#[test]
fn the_service_keeps_a_worktree_another_worktree_was_added_inside_after_the_hand_over() {
    let linked: Linked = a_repository_with_a_linked_worktree_and_ignore_rules("worktree-hand-over-nested", ".claude/\n");
    let item: WorkItem = handed_over(&linked, Some(linked.root.as_path()));
    // A command that adds a worktree inside this one finishes after the record was written and exits.
    let inside: PathBuf = linked.worktree.join(".claude/worktrees/b");
    crate::test_support::git(linked.worktree.as_path(), &["worktree", "add", "--quiet", "-b", "sub", inside.to_str().expect("utf8")]);
    let changed: PathBuf = inside.join("tracked.txt");
    std::fs::write(changed.as_path(), b"the person changed this in the nested worktree\n").expect("a tracked file the person changed there");
    let notes: PathBuf = inside.join("notes.txt");
    std::fs::write(notes.as_path(), b"work the person has not committed\n").expect("an untracked file the person wrote there");
    let verdict: Verdict = delete_subvolume_of(&item);
    assert!(
        matches!(&verdict, Verdict::Failed(words) if words.contains("another worktree this repository registers is inside it")),
        "the worktree that holds another one stays, and the reason names it: {verdict:?}"
    );
    assert_eq!(std::fs::read(changed.as_path()).expect("the changed file is still there"), b"the person changed this in the nested worktree\n");
    assert_eq!(std::fs::read(notes.as_path()).expect("the untracked file is still there"), b"work the person has not committed\n");
    assert!(linked.worktree.is_dir(), "the worktree the merge landed stays");
    let texts: Vec<String> = kept_notice_texts(linked.common.as_path());
    assert_eq!(texts.len(), 1, "the person is told once: {texts:?}");
    assert!(texts[0].contains(linked.worktree.display().to_string().as_str()), "the notice names the worktree: {texts:?}");
    assert!(texts[0].contains("refs/heads/topic"), "and the branch: {texts:?}");
    assert!(texts[0].contains("another worktree this repository registers is inside it"), "and the reason: {texts:?}");
    remove(linked.root.as_path());
}

/// What:
///  An ignored file written into the handed-over worktree after the hand-over keeps it,
///  and the person reads which file.
/// Why:
///  The sweep compared the worktree's ignored files with the one the merge ran in,
///  and that reading was spent before the record was written.
///  Git deletes what it ignores without a word,
///  so the service removed a file the person wrote in the window between the two steps
///  (a review of the hand-over,
///  finding 3.2).
#[test]
fn the_service_keeps_a_worktree_whose_ignored_file_was_written_after_the_hand_over() {
    let linked: Linked = a_repository_with_a_linked_worktree_and_ignore_rules("worktree-hand-over-ignored", "*.log\n");
    let item: WorkItem = handed_over(&linked, Some(linked.repo_root().as_path()));
    let written: PathBuf = linked.worktree.join("person.log");
    std::fs::write(written.as_path(), b"what the person wrote after the hand over\n").expect("an ignored file the person wrote");
    let verdict: Verdict = delete_subvolume_of(&item);
    assert!(
        matches!(&verdict, Verdict::Failed(words) if words.contains("person.log")),
        "the worktree stays and the reason names the ignored file: {verdict:?}"
    );
    assert_eq!(std::fs::read(written.as_path()).expect("the ignored file is still there"), b"what the person wrote after the hand over\n");
    let texts: Vec<String> = kept_notice_texts(linked.common.as_path());
    assert_eq!(texts.len(), 1, "the person is told once: {texts:?}");
    assert!(texts[0].contains("person.log"), "the notice names the file: {texts:?}");
    remove(linked.root.as_path());
}

/// What:
///  A worktree whose ignored file the worktree the proof was made from holds with the same bytes is removed.
/// Why:
///   The proof that keeps a worktree must not keep every worktree:
///  a file both trees hold is state the repository already has,
///  and this is the positive control of the two controls above.
#[test]
fn the_service_removes_a_worktree_whose_ignored_file_the_compared_worktree_also_holds() {
    let linked: Linked = a_repository_with_a_linked_worktree_and_ignore_rules("worktree-hand-over-ignored-same", "*.log\n");
    let item: WorkItem = handed_over(&linked, Some(linked.repo_root().as_path()));
    let both: PathBuf = linked.worktree.join("build.log");
    std::fs::write(both.as_path(), b"the same words in both trees\n").expect("an ignored file the add carried");
    std::fs::write(linked.repo_root().join("build.log").as_path(), b"the same words in both trees\n").expect("the source's copy");
    let verdict: Verdict = delete_subvolume_of(&item);
    assert_eq!(verdict, Verdict::Done, "a copy the other tree also holds is nothing to lose");
    assert!(!linked.worktree.exists(), "the worktree is gone");
    assert!(kept_notice_texts(linked.common.as_path()).is_empty(), "nothing was kept, so nothing is said: {}", linked.root.display());
    remove(linked.root.as_path());
}

/// What:
///  A commit a reference lock holds after the hand-over is kept on a reference of the common Git directory
///  before the service removes the worktree.
/// Why:
///  The held commit lives on the worktree's own reference and in its own Git directory,
///  and the removal deletes both,
///  so the sweep kept what was held when it ran.
///  A batch that met a reference lock afterwards is held since,
///  and the service's removal deleted the last thing that named it
///  (a review of the hand-over,
///  finding 3.1).
#[test]
fn the_service_keeps_a_commit_held_after_the_hand_over_on_a_side_reference() {
    let linked: Linked = a_repository_with_a_linked_worktree("worktree-hand-over-held-commit");
    // The branch's commit, so the worktree is one a `git worktree remove` would take.
    let tracked: PathBuf = linked.worktree.join("tracked.txt");
    std::fs::write(tracked.as_path(), b"tracked, edited in the worktree\n").expect("a tracked file");
    crate::test_support::age(tracked.as_path());
    crate::test_support::git(linked.worktree.as_path(), &["add", "--", "tracked.txt"]);
    crate::test_support::git(linked.worktree.as_path(), &["commit", "--quiet", "--message=the branch's own work"]);
    let item: WorkItem = handed_over(&linked, None);
    // The hold is recorded after the hand-over, as a commit whose branch move met a reference lock leaves it.
    let admin: PathBuf = linked.common.join("worktrees").join("topic");
    let tree: String = crate::test_support::git_text(linked.worktree.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let held: String = crate::test_support::git_text(linked.worktree.as_path(), &["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "the held edit"]);
    let reference: String = String::from("refs/worktree/cli-git-service/7");
    crate::test_support::git(linked.worktree.as_path(), &["update-ref", "--no-deref", reference.as_str(), held.as_str()]);
    crate::test_support::git(linked.common.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    let record: crate::held_commit_record::Held = crate::held_commit_record::Held {
        seq: 7,
        journal: linked.root.join("journal"),
        repository: linked.worktree.clone(),
        common_dir: linked.common.clone(),
        git_dir: admin.clone(),
        reference: crate::ref_name::RefName::from_text("refs/heads/topic"),
        old: crate::test_support::git_text(linked.worktree.as_path(), &["rev-parse", "HEAD"]),
        new: held.clone(),
        commits: vec![held.clone()],
        index: admin.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![linked.common.join("refs/heads/topic.lock")],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    crate::held_commit_record::write(admin.as_path(), &record).expect("the record of the hold");
    assert_eq!(
        crate::test_support::git_text(linked.worktree.as_path(), &["for-each-ref", "--contains", held.as_str(), "--format=%(refname)"]).trim(),
        reference,
        "nothing but the worktree's own reference names the held commit"
    );
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the commit is kept, so the worktree may go");
    assert!(!linked.worktree.exists(), "the worktree is gone");
    assert_eq!(
        crate::test_support::git_text(linked.common.as_path(), &["rev-parse", "--verify", "refs/cli-git/kept/7"]).trim(),
        held,
        "the commit the reference lock held is named by a reference no removal deletes"
    );
    assert!(
        std::process::Command::new(crate::test_support::REAL_GIT)
            .arg("-C")
            .arg(linked.common.as_path())
            .args(["gc", "--quiet", "--prune=now"])
            .status()
            .expect("Git can be asked to prune")
            .success(),
        "the repository can be pruned"
    );
    assert!(
        std::process::Command::new(crate::test_support::REAL_GIT)
            .arg("-C")
            .arg(linked.common.as_path())
            .args(["cat-file", "-e", held.as_str()])
            .status()
            .expect("Git can be asked whether the commit is there")
            .success(),
        "the commit survives a prune that takes every unreachable object"
    );
    remove(linked.root.as_path());
}

/// What:
///  An unshown push failure recorded after the hand-over still reaches the person from the next command.
/// Why:
///  The removal deletes the worktree's own Git directory,
///  and the record of the failed push lives there.
///  The sweep took its copy before it wrote the record,
///  so the service's removal is where the copy is owed,
///  and without it a person never learns that a push failed.
#[test]
fn the_service_carries_a_push_failure_recorded_after_the_hand_over_to_the_person() {
    let linked: Linked = a_repository_with_a_linked_worktree("worktree-hand-over-push-record");
    let item: WorkItem = handed_over(&linked, None);
    let admin: PathBuf = linked.common.join("worktrees").join("topic");
    let commit: String = crate::test_support::git_text(linked.worktree.as_path(), &["rev-parse", "HEAD"]);
    crate::push_failure_notice::record_push_failure(
        admin.as_path(),
        "refs/heads/topic",
        commit.as_str(),
        crate::push_failure_notice::PushOutcome::Refused,
        "refused after the hand over",
    )
    .expect("a failed push nobody has seen yet, recorded after the hand over");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "Git removes the worktree the record names");
    assert!(!admin.exists(), "the removal took the record's original with the Git directory");
    // What the next command does with the removal this one made.
    crate::worktree_removal_rescue::settle_removals(linked.common.as_path());
    let texts: Vec<String> = crate::test_support::shown_notices(crate::push_failure_notice::orphaned_push_failures(linked.common.as_path(), &whole_name));
    assert!(
        texts.iter().any(|text| return text.contains("refused after the hand over")),
        "the push failure recorded after the hand over is shown to the person: {texts:?}"
    );
    remove(linked.root.as_path());
}

/// What:
///  A record of a removal whose worktree was removed and whose path a person added a worktree at again removes nothing,
///  and leaves the new worktree and its ignored files exactly as they are.
/// Why:
///  The record names a path,
///  and a path names whatever stands there when the service runs.
///  The README's own advice,
///  `git worktree remove` by hand,
///  is what makes this sequence ordinary:
///  a launcher that restarts a session adds a worktree at the path the earlier run used,
///  and the stale record removed that new worktree with everything Git ignores in it
///  (a review of the hand-over,
///  finding 3.3).
#[test]
fn a_record_of_a_removal_whose_worktree_is_gone_removes_nothing_at_the_path_it_named() {
    let linked: Linked = a_repository_with_a_linked_worktree_and_ignore_rules("worktree-hand-over-reused-path", "*.log\n");
    let item: WorkItem = handed_over(&linked, Some(linked.root.as_path()));
    // The person takes the README's advice and removes the handed-over worktree by hand.
    crate::test_support::git(linked.repo_root().as_path(), &["worktree", "remove", "--force", linked.worktree.to_str().expect("utf8")]);
    assert!(!linked.worktree.exists(), "the worktree the record names is gone");
    // The next run of whatever launcher this is adds a worktree at the same path and writes ignored state there.
    crate::test_support::git(linked.repo_root().as_path(), &["worktree", "add", "--quiet", "-b", "hotfix", linked.worktree.to_str().expect("utf8")]);
    let written: PathBuf = linked.worktree.join("build.log");
    std::fs::write(written.as_path(), b"the new worktree's own ignored state\n").expect("an ignored file of the new worktree");
    let verdict: Verdict = delete_subvolume_of(&item);
    assert!(
        matches!(&verdict, Verdict::Skipped(words) if words.contains("another one stands at")),
        "the record's worktree is gone, and the one standing there now is nothing to it: {verdict:?}"
    );
    assert!(linked.worktree.is_dir(), "the new worktree stays");
    assert_eq!(std::fs::read(written.as_path()).expect("the ignored file is still there"), b"the new worktree's own ignored state\n");
    remove(linked.root.as_path());
}
