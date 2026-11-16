//! What:
//!  Controls for finishing a removal whose destroy happened and whose registration did not go.
//! Why:
//!   The finish is the one path that ends a removal a kill left half done,
//! and it refuses anything below the registration it
//!  would not delete,
//! so the wrapper's own state in that registration has to be out of its way before it looks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishInterruptedDestroys(common)).toEqual([]); expect(admin.exists()).toBe(false);
//! ```

use super::*;
use crate::fs_beneath::identity_of;
use crate::fs_btrfs_ioctl::{subvolume_create_at, subvolume_destroy_at};
use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::test_support::{git, remove, repository};
use std::fs::File;
use std::path::PathBuf;

/// What:
///  The answer the hand-over's wake gives,
///  so a control reaches no commit service and no session bus.
/// Why:
///   This tree's hand-over asks a value that says who is told,
/// not a closure,
/// and a control needs one answer:
/// `Some(..)` is a wake that did not reach a service,
/// `None` one that did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const told = new Answer('nothing answered'); // told(commonDir) throws that reason
/// ```
struct Answer(Option<String>);

impl crate::worktree_remove_hand_over::ToldOfHandOver for Answer {
    fn told(&self, _common_dir: &Path) -> Result<(), String> {
        return match self.0.as_ref() {
            Some(reason) => Err(reason.clone()),
            None => Ok(()),
        };
    }
}

/// What:
///  A removal whose destroy happened and whose registration stayed is finished by the next command even when the only
///  thing below that registration is the pool of snapshots the wrapper made ahead of an add.
/// Why:
///   A worktree that was the source of one `git worktree add` holds that pool,
/// whose members are subvolumes,
/// and the
///  registration's emptying refuses a nested subvolume,
/// so every such removal ended as "the files are gone and the
///  registration is not",
/// with the whole pool left behind until the person ran `git worktree prune`
///  (a Sonnet finding,
/// F1).
///  A pool is the wrapper's own cache of the worktree,
/// and a later add that finds none makes its own.
#[test]
fn btrfs_a_recorded_destroy_is_finished_when_the_snapshots_ahead_are_the_only_thing_left() {
    let Some(root) = btrfs_fixture("recorded-destroy-ahead-pool") else { return };
    if !removal_allowed(root.as_path()) {
        // The destroy this record names cannot be made here at all, so there is nothing to finish.
        return;
    }
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    subvolume_create_at(root.join("wt").as_path()).expect("the subvolume of the worktree");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt").as_path()).expect("canonical");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees/wt");
    // The pool the commit service fills for a worktree an add copies from, with one finished snapshot in it.
    let pool: PathBuf = crate::worktree_snapshot_ahead::pool_folder(admin.as_path());
    std::fs::create_dir_all(pool.as_path()).expect("the folder of the snapshots ahead");
    let member: PathBuf = pool.join(".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef");
    subvolume_create_at(member.as_path()).expect("a snapshot the wrapper made ahead of an add");
    std::fs::write(member.join("tracked.txt"), b"the worktree's own file\n").expect("its files");
    // The state a removal killed between its kernel call and the emptying of the registration leaves.
    let identity = identity_of(&std::fs::File::open(admin.as_path()).expect("the registration")).expect("its identity");
    record_destroy(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path()).expect("the record of the destroy");
    subvolume_destroy_at(worktree.as_path()).expect("the destroy the record names");
    record_destroyed(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "the pool is the wrapper's own, so nothing stands in the way of finishing: {problems:?}");
    assert!(!admin.exists(), "the registration is gone and the path is free again");
    assert!(!pool.exists(), "and the pool went with it");
    remove(root.as_path());
}

/// What:
///  A removal whose destroy happened and whose registration stayed is finished by the next command even when the only
///  thing below that registration is a pool a killed clear left under its taken name.
/// Why:
///   A clear renames the pool of snapshots ahead to a name no fill reads before it deletes a member of it,
///  so a command
///  killed in that moment leaves a member under that name,
///  and the finish's walk refuses every nested subvolume:
///  a
///  clear that took only the pool's own name would refuse its own leftover for ever,
///  and every later command would say the
///  removal cannot be finished (the incident of 2026-11-11 named exactly such a member under the pool's
///  own name).
#[test]
fn btrfs_a_recorded_destroy_is_finished_when_a_pool_a_killed_clear_left_is_the_only_thing_left() {
    let Some(root) = btrfs_fixture("recorded-destroy-pool-left-taken") else { return };
    if !removal_allowed(root.as_path()) {
        // The destroy this record names cannot be made here at all, so there is nothing to finish.
        return;
    }
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    subvolume_create_at(root.join("wt").as_path()).expect("the subvolume of the worktree");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt").as_path()).expect("canonical");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees/wt");
    // The state a clear killed between its rename and its deletions leaves: a member of the pool under the taken name.
    let data: PathBuf = admin.join("monochromatic-git-policy-cli");
    let left: PathBuf = data.join("removing-0f0e0d0c-0b0a-4908-8706-050403020100");
    std::fs::create_dir_all(left.as_path()).expect("the folder a killed clear had renamed");
    let member: PathBuf = left.join(".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef");
    subvolume_create_at(member.as_path()).expect("the member a killed clear left under its taken name");
    std::fs::write(member.join("tracked.txt"), b"a copy of the worktree\n").expect("its files");
    let identity = identity_of(&std::fs::File::open(admin.as_path()).expect("the registration")).expect("its identity");
    record_destroy(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path()).expect("the record of the destroy");
    subvolume_destroy_at(worktree.as_path()).expect("the destroy the record names");
    record_destroyed(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "the leftover of the wrapper's own clear is not something to report: {problems:?}");
    assert!(!admin.exists(), "the registration is gone and the path is free again");
    assert!(!left.exists(), "and the pool the killed clear left went with it");
    remove(root.as_path());
}

/// What:
///  A record an earlier removal of the same registration left behind is replaced when this removal records its own
///  destroy,
///  so the next command never reads an older removal's proof as this one's,
///  and a sweep never drops the record of a removal that is still running.
/// Why:
///   A worktree name is used again and again,
/// and a record is named by the registration's id,
/// so a stale record
///  marked as destroyed stood for a removal that had not destroyed anything yet:
/// a sweep found the older
///  removal's target gone and forgot the record the running removal had just written,
///  which is why a
///  successful removal warned that its record "cannot be marked (No such file or directory)" twice on
///  2026-11-11 (22:30:57 and 22:37:25 -05:00).
/// The record that is written says only that a destroy was intended,
///  which is what a record of a running removal must say.
#[test]
fn a_record_left_by_an_earlier_removal_is_replaced_and_never_read_as_this_removals_proof() {
    let root: PathBuf = crate::test_support::fixture("recorded-destroy-stale");
    let common_dir: PathBuf = root.join("common");
    let admin: PathBuf = common_dir.join("worktrees/wt");
    std::fs::create_dir_all(admin.as_path()).expect("the registration");
    let identity = identity_of(&std::fs::File::open(admin.as_path()).expect("the registration")).expect("its identity");
    // The record of an earlier removal of a worktree of the same name, whose destroy happened and whose target is gone.
    record_destroy_with_owner(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, root.join("an-earlier-worktree").as_path(), Some("1:ended")).expect("the older record");
    record_destroyed(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    let record: PathBuf = record_path(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    let older: String = std::fs::read_to_string(record.as_path()).expect("the older record");
    assert!(older.starts_with("destroyed=1\n"), "the older record is the proof of an earlier destroy: {older:?}");
    // This removal records its own destroy for the worktree that stands at its own path.
    let worktree: PathBuf = root.join("wt");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    record_destroy(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path()).expect("the record of this destroy");
    let mine: String = std::fs::read_to_string(record.as_path()).expect("this removal's record");
    assert!(!mine.starts_with("destroyed=1\n"), "nothing is proved destroyed before this removal destroys it: {mine:?}");
    assert!(mine.contains(worktree.display().to_string().as_str()), "the record names the worktree of this removal: {mine:?}");
    // A sweep leaves a record whose owner may still be running, so this removal can still mark it.
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "nothing is reported about a removal that is still running: {problems:?}");
    assert!(record.is_file(), "the record of the running removal stays for it to mark");
    assert!(admin.is_dir(), "and its registration was not emptied by a sweep reading an older removal's proof");
    remove(root.as_path());
}

/// What:
///  A repository with one subvolume worktree registered as `wt`,
///  and what a removal of it records its destroy with.
/// Why:
///   A removal killed at any step leaves files on disk and a record,
/// and the step under test is the one that follows
///  the record,
/// so a control builds the state before the kill and lets the next command read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { common, worktree, admin, identity } = aRemovalReadyToBeKilled(root, name);
/// ```
fn a_removal_ready_to_be_killed(root: &Path, name: &str) -> (PathBuf, PathBuf, PathBuf, ObjectId) {
    let repo: PathBuf = repository(root, name);
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    subvolume_create_at(root.join("wt").as_path()).expect("the subvolume of the worktree");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt").as_path()).expect("canonical");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees/wt");
    let identity = identity_of(&std::fs::File::open(admin.as_path()).expect("the registration")).expect("its identity");
    record_destroy_with_owner(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some("1:ended")).expect("the record of the destroy");
    return (common_dir, worktree, admin, identity);
}

/// What:
///  A removal killed after it recorded its destroy and before it renamed anything leaves the worktree where it was,
///  and the next command drops the record and changes nothing.
/// Why:
///   The first of the three steps of a hand-over is the record,
/// and the removal wrote no proof yet that the files
///  left the path,
/// so the path that still holds them is what the next command reads.
#[test]
fn btrfs_a_removal_killed_after_its_destroy_is_recorded_leaves_the_worktree_and_nothing_to_finish() {
    let Some(root) = btrfs_fixture("recorded-kill-before-the-rename") else { return };
    let (common_dir, worktree, admin, _identity) = a_removal_ready_to_be_killed(root.as_path(), "repo");
    let record: PathBuf = record_path(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    assert!(record.is_file(), "the record of the removal was written");
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "nothing is reported about a removal that took nothing: {problems:?}");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is where it was, with its files");
    assert!(admin.is_dir(), "and its registration stays, so the worktree can be removed again");
    assert!(!record.exists(), "the record of a removal that took nothing is dropped");
    remove(root.as_path());
}

/// What:
///  A removal killed after it renamed its subvolume out of the worktree and before it marked the record
///  is finished by the next command,
///  so the path is free again and nothing is left for the person.
/// Why:
///   The rename and the mark are not one step,
/// and the whole wait of the wake of the service sits between them;
///  a command killed there left a record that said only that a destroy was intended,
/// with nothing at the
///  worktree's path,
/// and the next command reported the state once and kept the registration,
/// after which
///  `git worktree add` at that path failed with "missing but already registered worktree" (a review finding,
/// F4).
/// The place the subvolume takes is written into the record before the rename,
///  which is what makes this state decidable.
#[test]
fn btrfs_a_removal_killed_after_the_rename_and_before_the_mark_is_finished_by_the_next_command() {
    let Some(root) = btrfs_fixture("recorded-kill-in-the-window") else { return };
    let (common_dir, worktree, admin, _identity) = a_removal_ready_to_be_killed(root.as_path(), "repo");
    let parent_path: PathBuf = worktree.parent().expect("the folder the worktree stands in").to_path_buf();
    let parent: File = crate::fs_beneath::open_root(parent_path.as_path()).expect("the folder the worktree stands in");
    let id: u64 = crate::fs_btrfs_ioctl::subvolume_id(&std::fs::File::open(worktree.as_path()).expect("the worktree"), worktree.as_path()).expect("the id of its subvolume");
    let handed = crate::worktree_remove_hand_over::hand_over_with(
        common_dir.as_path(),
        &parent,
        parent_path.as_path(),
        worktree.file_name().expect("its name"),
        id,
        worktree.as_path(),
        std::ffi::OsStr::new("wt"),
        &Answer(Some(String::from("the kill took the process before the service answered"))),
    );
    handed.expect("the record, the place and the rename are the hand-over's own steps");
    // The kill lands here, between the rename and the mark.
    let leaving: PathBuf = parent_path.join(crate::worktree_remove_hand_over::deleting_name(id));
    assert!(leaving.is_dir(), "the subvolume stands where the record says it will");
    assert!(!worktree.exists(), "and nothing is at the path the person named");
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "the record names where the files stand, so nothing is left for a person: {problems:?}");
    assert!(!admin.exists(), "the registration is gone, which is what the killed removal had left");
    let repo: PathBuf = root.join("repo");
    let again = git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "again", worktree.to_str().expect("a UTF-8 fixture path")]);
    assert!(again.status.success(), "the path is free again: {}", String::from_utf8_lossy(again.stderr.as_slice()));
    remove(root.as_path());
}

/// What:
///  A removal killed after it marked the record and before its registration was removed
///  is finished by the next command,
///  as a marked record always was.
/// Why:
///   The mark is the proof that the files left the path,
/// and the registration is the only thing a kill after it
///  can leave.
#[test]
fn btrfs_a_removal_killed_after_the_mark_is_finished_by_the_next_command() {
    let Some(root) = btrfs_fixture("recorded-kill-after-the-mark") else { return };
    let (common_dir, worktree, admin, _identity) = a_removal_ready_to_be_killed(root.as_path(), "repo");
    let parent_path: PathBuf = worktree.parent().expect("the folder the worktree stands in").to_path_buf();
    let parent: File = crate::fs_beneath::open_root(parent_path.as_path()).expect("the folder the worktree stands in");
    let id: u64 = crate::fs_btrfs_ioctl::subvolume_id(&std::fs::File::open(worktree.as_path()).expect("the worktree"), worktree.as_path()).expect("the id of its subvolume");
    crate::worktree_remove_hand_over::hand_over_with(
        common_dir.as_path(),
        &parent,
        parent_path.as_path(),
        worktree.file_name().expect("its name"),
        id,
        worktree.as_path(),
        std::ffi::OsStr::new("wt"),
        &Answer(None),
    )
    .expect("the hand-over");
    record_destroyed(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    // The kill lands after the mark and before the registration is removed.
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "a marked record is finished with nothing to report: {problems:?}");
    assert!(!admin.exists(), "the registration is gone");
    assert!(!record_path(common_dir.as_path(), std::ffi::OsStr::new("wt")).exists(), "and its record with it");
    remove(root.as_path());
}

/// What:
///  A record of a removal whose owner runs,
///  and whose owner's identity is spelled another way,
///  is the record of a removal that may still be running.
/// Why:
///  Another command's recovery finishes and forgets a record whose owner it takes for gone:
///  the record is the only place the removal that runs can mark its success,
///  and the registration it names is emptied and forgotten under that command.
///  The record is written in one spelling of this process's identity,
///  so a second one came from a writer that is not this release (an older release,
///  another tool,
///  a damaged file),
///  and a spelling a reader cannot compare is no proof that its owner ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(ownerMayBeRunning(respelledOwnerOfThisProcess())).toBe(true);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_record_whose_live_owner_is_named_in_another_spelling_is_a_running_removal() {
    let birth: String = crate::process_identity::current_birth_identity().expect("this process has an identity");
    let pid: i64 = i64::from(std::process::id());
    let owners: [(&str, String); 4] = [
        (
            "a leading zero on the start tick",
            format!("{pid}:{}", crate::process_identity_spellings::birth_with_a_leading_zero(birth.as_str())),
        ),
        (
            "the platform's tag in capitals",
            format!("{pid}:{}", crate::process_identity_spellings::birth_with_its_platform_tag_in_capitals(birth.as_str())),
        ),
        (
            "a space before the birth",
            format!("{pid}:{}", crate::process_identity_spellings::birth_behind_a_space(birth.as_str())),
        ),
        (
            "a leading zero on the process number",
            format!("{}:{birth}", crate::process_identity_spellings::process_number_with_a_leading_zero(pid)),
        ),
    ];
    // The record a decision is measured against: a file that has just been written, so the patience is no factor here.
    let root: PathBuf = crate::test_support::fixture("removal-record-spelling");
    let record: PathBuf = root.join("a-record");
    std::fs::write(record.as_path(), b"x").expect("the record");
    let mut wrong: Vec<String> = Vec::new();
    for (what, owner) in owners {
        if crate::worktree_removal_finishes_an_interrupted_destroy::owner_standing(Some(&owner), record.as_path())
            != crate::worktree_removal_finishes_an_interrupted_destroy::OwnerOfTheRecord::Runs
        {
            wrong.push(format!("{what}: the owner {owner} was not taken for a command that runs, and it runs"));
        }
    }
    // The other answer of the same reader, so an arm that passes is not a reader that never answers "ended": the same
    // record of a process that ended is no running command.
    let ended: crate::worktree_copy_owner::CopyOwner = crate::test_support::dead_copy_owner();
    let ended_owner: String = format!("{}:{}", ended.pid, ended.birth_identity);
    assert_eq!(
        crate::worktree_removal_finishes_an_interrupted_destroy::owner_standing(Some(&ended_owner), record.as_path()),
        crate::worktree_removal_finishes_an_interrupted_destroy::OwnerOfTheRecord::Ended,
        "the record {ended_owner} of a process that ended is no running removal"
    );
    crate::test_support::remove(root.as_path());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// What:
///  A record of a removal whose owner's birth names no value, and which has stood past the patience,
///  is named to the person instead of being skipped in silence.
/// Why:
///  Another command's recovery finishes and forgets the record of a command it takes for gone,
///  and leaves alone one whose command may run:
///  the record is the only place the removal that runs can mark its success.
///  A birth that names no value is neither of those.
///  Read as a command that runs it leaves the record skipped on every command for ever;
///  read as one that ended it finishes a removal that may be running.
///  The record's own age is the bound,
///  and past the patience the person reads which record it is and the command that removes it.
///  The last record below has a birth this command can read, and it is finished,
///  so the row is not a reader that never answers.
#[cfg(target_os = "linux")]
#[test]
fn a_record_of_an_owner_that_cannot_be_told_is_named_past_the_patience() {
    let root: PathBuf = crate::test_support::fixture("removal-record-unreadable-birth");
    // A directory stands where the registration of the removal stands, so the record names something real; the owner
    //  check comes before anything reads it.
    let admin: crate::fs_beneath::ObjectId =
        crate::fs_beneath::identity_of(&std::fs::File::open(root.as_path()).expect("the fixture root")).expect("an identity");
    let pid: i64 = i64::from(std::process::id());
    let aged: std::time::Duration = crate::pid_namespace::FOREIGN_OWNER_PATIENCE + std::time::Duration::from_secs(60);
    let unreadable: String = format!("{pid}:linux:not-a-birth");
    let ended: String = format!("{pid}:linux:1");
    let mut wrong: Vec<String> = Vec::new();
    for (index, owner) in [unreadable.as_str(), ended.as_str()].into_iter().enumerate() {
        // Each row has a common directory of its own, so one row's record is not read again by the next row's command.
        let common: PathBuf = root.join(format!("common-{index}"));
        std::fs::create_dir_all(common.as_path()).expect("the common directory");
        let target: PathBuf = root.join(format!("gone-{index}"));
        let record: PathBuf = record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new(format!("wt-{index}").as_str()), &admin, target.as_path(), Some(owner))
            .map(|()| { return crate::worktree_removal_interrupted_destroy::record_path(common.as_path(), std::ffi::OsStr::new(format!("wt-{index}").as_str())); })
            .expect("the record of a removal");
        std::fs::File::options()
            .write(true)
            .open(record.as_path())
            .expect("open the record")
            .set_modified(std::time::SystemTime::now() - aged)
            .expect("age the record");
        let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
        if owner == unreadable.as_str() {
            let named: bool = problems.len() == 1 && problems[0].contains(record.to_string_lossy().as_ref()) && problems[0].contains(pid.to_string().as_str());
            if !named || !problems[0].contains("rm -- ") || !problems[0].contains("birth identity") {
                wrong.push(format!("the record of an owner that cannot be told: the person read {problems:?}, and the record {record:?} should be named with the command that removes it"));
            }
            if !record.exists() {
                wrong.push(String::from("the record of an owner that cannot be told was removed, and nothing proves its command ended"));
            }
        } else if !problems.is_empty() || record.exists() {
            wrong.push(format!("the record of a command that ended: the person read {problems:?} and the record exists: {}", record.exists()));
        }
    }
    crate::test_support::remove(root.as_path());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
