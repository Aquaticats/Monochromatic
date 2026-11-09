//! Controls for keeping the records of a removed worktree:
//! what is copied before a removal,
//! what a later settle makes of it,
//!  that a removal still under way is never settled,
//! that a failure keeps what is left and is told to the person,
//! and that a
//!  worktree which stays loses nothing and shows nothing.

use super::*;
use crate::push_failure_notice::{ORPHAN_NOTICE_DIRECTORY, orphaned_push_failures, record_push_failure};
use crate::worktree_removal_snapshot_promotion::{NOTICES, WORK, files_of};
use crate::work_item::WorkItem;
use crate::service_work::work_directory;
use crate::test_support::{fixture, remove};
use crate::work_item::enqueue;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const BRANCH: &str = "refs/heads/feature";

/// What a process that is not running is recorded as:
/// no process has this birth identity.
const ENDED_IDENTITY: &str = "ended:0";

fn plain(commit: &str) -> String {
    return commit.to_string();
}

/// A common directory with one linked worktree's Git directory in it,
/// and the checkout that Git directory names.
struct Repo {
    root: PathBuf,
    common: PathBuf,
    linked: PathBuf,
    checkout: PathBuf,
}

fn repo(name: &str) -> Repo {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let linked: PathBuf = common.join("worktrees").join("w");
    let checkout: PathBuf = root.join("checkout");
    std::fs::create_dir_all(&linked).expect("linked git directory");
    std::fs::create_dir_all(&checkout).expect("checkout");
    std::fs::write(checkout.join(".git"), format!("gitdir: {}\n", linked.display())).expect("checkout pointer");
    std::fs::write(linked.join("gitdir"), format!("{}\n", checkout.join(".git").display())).expect("registration");
    return Repo { root, common, linked, checkout };
}

fn item(kind: WorkKind, commit: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: PathBuf::from("/nowhere"),
        git_dir: PathBuf::from("/nowhere/.git"),
        branch_ref: BRANCH.to_string(),
        oid: commit.to_string(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
}

fn shown(repo: &Repo) -> Vec<String> {
    // A record goes only once its words are out, so these controls read them through the step a command shows them by.
    return crate::test_support::shown_notices(orphaned_push_failures(repo.common.as_path(), &plain));
}

/// The snapshot of the fixture's worktree.
fn snapshot_of(repo: &Repo) -> PathBuf {
    return repo.common.join(SNAPSHOT_DIRECTORY).join(instance_of(repo.linked.as_path()));
}

/// Makes every command that took the snapshot a command that has ended,
/// as it is once Git has refused or finished.
fn end_attempts(repo: &Repo) {
    for path in files_of(snapshot_of(repo).join(ATTEMPTS).as_path()) {
        std::fs::write(path, ENDED_IDENTITY).expect("the recorded command has ended");
    }
}

/// The record of a failed push and a pending push,
/// both under the fixture's worktree.
fn records(repo: &Repo) {
    record_push_failure(repo.linked.as_path(), BRANCH, COMMIT, crate::push_failure_notice::PushOutcome::Refused, "refused by test policy").expect("notice");
    enqueue(work_directory(repo.linked.as_path()).as_path(), &item(WorkKind::Push, OTHER_COMMIT)).expect("pending push");
}

/// A failed push nobody saw and a push still pending,
/// both under a worktree that is then removed,
/// are shown by whoever asks
///  next:
/// the failure with Git's words,
/// the pending push as one that may not have finished.
#[test]
fn the_records_of_a_removed_worktree_are_shown_to_the_next_command() {
    let repo: Repo = repo("rescue-removed");
    records(&repo);
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(texts.iter().any(|text| { return text.contains("branch feature") && text.contains(COMMIT) && text.contains("refused by test policy"); }), "{texts:?}");
    assert!(
        texts.iter().any(|text| { return text.contains("branch feature") && text.contains(OTHER_COMMIT) && text.contains("removed") && text.contains("may exist only"); }),
        "{texts:?}"
    );
    assert!(shown(&repo).is_empty(), "shown once");
    assert!(!snapshot_of(&repo).exists(), "the snapshot is gone");
    remove(repo.root.as_path());
}

/// What:
///  A snapshot of a worktree whose Git directory is not text is judged by that directory.
/// Why:
///   The settle looks at the directory the snapshot names:
/// a spelling that lost its bytes names nothing,
/// which reads as a removal that finished,
/// so the records of a worktree that never went away would be promoted and its snapshot deleted.
#[test]
fn a_snapshot_of_a_directory_that_is_not_text_is_judged_by_that_directory() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("rescue-non-utf8");
    let common: PathBuf = root.join("common");
    let linked: PathBuf = common.join("worktrees").join(OsStr::from_bytes(b"w\xff"));
    let checkout: PathBuf = root.join(OsStr::from_bytes(b"ch\xffckout"));
    std::fs::create_dir_all(&linked).expect("linked git directory");
    std::fs::create_dir_all(&checkout).expect("checkout");
    // Git writes this registration as bytes; the checkout is there, so the removal did not happen.
    // Git writes the path's bytes, and a lossy spelling would name a directory that does not exist.
    let mut registration: Vec<u8> = checkout.join(".git").as_os_str().as_bytes().to_vec();
    registration.push(b'\n');
    std::fs::write(linked.join("gitdir"), registration.as_slice()).expect("registration");
    std::fs::write(checkout.join(".git"), format!("gitdir: {}\n", linked.display())).expect("checkout pointer");
    let repo: Repo = Repo { root: root.clone(), common: common.clone(), linked: linked.clone(), checkout: PathBuf::from("/nonexistent-checkout") };
    records(&repo);
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    end_attempts(&repo);
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(shown(&repo).is_empty(), "a worktree that is still there shows nothing");
    remove(root.as_path());
}

/// What:
///  The registration Git writes beside a linked worktree is read as the bytes it wrote,
/// so the checkout of a worktree whose path is not text is found.
/// Why:
///   The settle decides between a refused removal and one still under way by looking at that checkout:
/// read as text it names no directory for such a path,
/// and the snapshot of a refused removal is kept for a removal that will never come.
#[test]
fn the_checkout_of_a_directory_that_is_not_text_is_found_by_the_bytes_git_wrote() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("rescue-non-utf8-checkout");
    let common: PathBuf = root.join("common");
    let linked: PathBuf = common.join("worktrees").join(OsStr::from_bytes(b"w\xff"));
    let checkout: PathBuf = root.join(OsStr::from_bytes(b"ch\xffckout"));
    std::fs::create_dir_all(linked.join("git").as_path()).expect("linked git directory");
    std::fs::create_dir_all(checkout.join(".git").as_path()).expect("checkout");
    // The registration Git writes names the checkout's own `.git`, which is there, so the removal was refused.
    let mut registration: Vec<u8> = checkout.join(".git").as_os_str().as_bytes().to_vec();
    registration.push(b'\n');
    std::fs::write(linked.join("gitdir"), registration.as_slice()).expect("registration");
    let repo: Repo = Repo { root: root.clone(), common: common.clone(), linked: linked.clone(), checkout: checkout.clone() };
    records(&repo);
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    end_attempts(&repo);
    assert_eq!(removal_state::checkout_present(linked.as_path()), Some(true), "the registration names the checkout that is there");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(shown(&repo).is_empty(), "the worktree is there, so nothing is shown");
    assert!(!snapshot_of(&repo).exists(), "a refused removal's snapshot is discarded");
    remove(root.as_path());
}

/// What:
///  A checkout path that holds a line break is read whole from the registration Git wrote.
/// Why:
///   Git ends that file with one line break,
/// and a path may hold one of its own;
/// stopping at the first one examines a prefix of the checkout,
/// and an existing directory of that name makes an incomplete removal read as refused,
/// which discards the snapshot that keeps the removed worktree's records.
#[test]
fn a_checkout_path_that_holds_a_line_break_is_read_whole() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("rescue-line-break-checkout");
    let common: PathBuf = root.join("common");
    let linked: PathBuf = common.join("worktrees").join("w");
    let checkout: PathBuf = root.join(OsStr::from_bytes(b"keep\nremoved"));
    std::fs::create_dir_all(linked.as_path()).expect("linked git directory");
    std::fs::create_dir_all(checkout.join(".git").as_path()).expect("checkout");
    // A directory of the name the prefix of the path spells, which is no checkout of any worktree.
    std::fs::create_dir_all(root.join("keep").as_path()).expect("a directory of the prefix's name");
    let mut registration: Vec<u8> = checkout.join(".git").as_os_str().as_bytes().to_vec();
    registration.push(b'\n');
    std::fs::write(linked.join("gitdir"), registration.as_slice()).expect("registration");
    let repo: Repo = Repo { root: root.clone(), common: common.clone(), linked: linked.clone(), checkout: checkout.clone() };
    assert_eq!(removal_state::checkout_present(linked.as_path()), Some(true), "the checkout is there");
    records(&repo);
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    end_attempts(&repo);
    // Git deletes the checkout first and the Git directory last; the removal is interrupted between the two.
    std::fs::remove_dir_all(checkout.as_path()).expect("Git deleted the checkout");
    assert_eq!(removal_state::checkout_present(linked.as_path()), Some(false), "the checkout is gone, and so is no directory its prefix names");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    // The checkout the registration names is gone and the command that took the snapshot has ended, so the removal is
    // decided by that directory and not by the one its prefix spells: the records reach the person and the snapshot goes.
    // A prefix read as the checkout would discard the snapshot and show nothing.
    assert!(!snapshot_of(&repo).exists(), "the snapshot is settled on the checkout the registration names");
    assert_eq!(shown(&repo).len(), 2, "what the snapshot kept reaches the person");
    remove(root.as_path());
}

/// What:
///  A snapshot an earlier release wrote for a worktree whose Git directory ends with a line break keeps its records.
/// Why:
///   That release wrote the path verbatim and nothing after it,
/// so trimming one line break from the record names the sibling worktree `topic`,
/// whose checkout is intact:
/// the finished removal reads as refused and the saved records are deleted with the snapshot.
#[test]
fn a_snapshot_of_an_earlier_release_whose_path_ends_with_a_line_break_keeps_its_records() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("rescue-line-break-path");
    let common: PathBuf = root.join("common");
    let removed: PathBuf = common.join("worktrees").join(OsStr::from_bytes(b"topic\n"));
    let sibling: PathBuf = common.join("worktrees").join("topic");
    let sibling_checkout: PathBuf = root.join("sibling-checkout");
    std::fs::create_dir_all(removed.as_path()).expect("linked git directory");
    std::fs::create_dir_all(sibling_checkout.join(".git").as_path()).expect("the sibling's checkout");
    std::fs::create_dir_all(sibling.as_path()).expect("the sibling's git directory");
    let mut sibling_registration: Vec<u8> = sibling_checkout.join(".git").as_os_str().as_bytes().to_vec();
    sibling_registration.push(b'\n');
    std::fs::write(sibling.join("gitdir"), sibling_registration.as_slice()).expect("the sibling's registration");
    let repo: Repo = Repo { root: root.clone(), common: common.clone(), linked: removed.clone(), checkout: PathBuf::from("/nonexistent-checkout") };
    records(&repo);
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    // The release before this one wrote the directory's text and nothing after it, so the last byte of this path is the path's.
    std::fs::write(snapshot_of(&repo).join(GIT_DIR_FILE), removed.as_os_str().as_bytes()).expect("an earlier release's name");
    end_attempts(&repo);
    std::fs::remove_dir_all(removed.as_path()).expect("Git removed the worktree");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 2, "the records the removed worktree left are shown: {texts:?}");
    assert!(shown(&repo).is_empty(), "shown once");
    remove(root.as_path());
}

/// A removal that did not happen (Git refused,
/// or a dry run) leaves the worktree,
/// its records and no notice of loss,
/// once the
///  command that took the snapshot has ended.
#[test]
fn a_worktree_that_stays_loses_nothing_and_shows_nothing_extra() {
    let repo: Repo = repo("rescue-stays");
    records(&repo);
    snapshot_linked_worktrees(repo.common.as_path());
    end_attempts(&repo);
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(shown(&repo).is_empty());
    assert_eq!(files_of(notice_directory(repo.linked.as_path()).as_path()).len(), 1, "the notice is still the worktree's own");
    assert!(!snapshot_of(&repo).exists(), "the snapshot was discarded");
    remove(repo.root.as_path());
}

/// Another command that settles while Git is still removing the worktree does not discard the snapshot:
/// the registration is
///  still there,
/// but the command that took the snapshot is still running,
/// and the originals go with the removal.
#[test]
fn a_command_that_settles_while_the_removal_is_under_way_keeps_the_snapshot() {
    let repo: Repo = repo("rescue-under-way");
    records(&repo);
    snapshot_linked_worktrees(repo.common.as_path());
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(snapshot_of(&repo).exists(), "the removal has not ended, so the snapshot is kept");
    assert!(shown(&repo).is_empty());
    // Git finishes the removal and deletes the originals.
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 2, "what the snapshot kept is shown: {texts:?}");
    remove(repo.root.as_path());
}

/// A removal cut short after Git deleted the checkout is settled by the next command,
/// which shows what the snapshot kept,
///  and the registration is left to the person.
#[test]
fn a_removal_cut_short_after_the_checkout_was_deleted_settles_the_snapshot() {
    let repo: Repo = repo("rescue-cut-short");
    records(&repo);
    snapshot_linked_worktrees(repo.common.as_path());
    end_attempts(&repo);
    std::fs::remove_dir_all(&repo.checkout).expect("Git deleted the checkout");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(!snapshot_of(&repo).exists(), "the checkout cannot come back, so the snapshot is settled");
    assert_eq!(shown(&repo).len(), 2, "the records are shown, not kept for ever");
    // The registration is the person's to remove: the wrapper changes no registration it was not asked to.
    assert!(repo.linked.is_dir());
    std::fs::remove_dir_all(&repo.linked).expect("a later prune removes the registration");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    remove(repo.root.as_path());
}

/// What:
///  A registered worktree whose checkout went while its registration stayed is a removal that will never finish,
/// so the
///  snapshot taken for it is settled by the next command instead of sitting there for ever.
/// Why:
///   A worktree whose directory is gone cannot run a command,
/// so nothing would ever show the records its Git directory still holds:
///  they reach the person only through the settle.
/// The live-taker arm is the seam:
/// a command that took the snapshot and may still be removing keeps it,
///  because the originals go with the removal.
#[test]
fn a_snapshot_whose_checkout_is_gone_is_settled_and_shown() {
    for (label, ended) in [("ended", true), ("live", false)] {
        let repo: Repo = repo(format!("rescue-checkout-gone-{label}").as_str());
        records(&repo);
        snapshot_linked_worktrees(repo.common.as_path());
        if ended {
            end_attempts(&repo);
        }
        std::fs::remove_dir_all(&repo.checkout).expect("the checkout is gone");
        assert!(settle_removals(repo.common.as_path()).is_empty());
        if ended {
            assert!(!snapshot_of(&repo).exists(), "the taker ended and the checkout is gone, so the removal is decided");
            assert_eq!(shown(&repo).len(), 2, "both records reached the person");
        } else {
            assert!(snapshot_of(&repo).exists(), "the taker may still be removing, so the snapshot is kept");
            assert!(shown(&repo).is_empty());
        }
        remove(repo.root.as_path());
    }
}

/// A pending push of a commit whose failure was already recorded does not replace Git's words with the vaguer ones.
#[test]
fn a_recorded_failure_keeps_its_words_over_the_pending_push_of_the_same_commit() {
    let repo: Repo = repo("rescue-same-commit");
    record_push_failure(repo.linked.as_path(), BRANCH, COMMIT, crate::push_failure_notice::PushOutcome::Refused, "refused by test policy").expect("notice");
    enqueue(work_directory(repo.linked.as_path()).as_path(), &item(WorkKind::Push, COMMIT)).expect("pending push");
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    settle_removals(repo.common.as_path());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 1, "{texts:?}");
    assert!(texts[0].contains("refused by test policy"), "{texts:?}");
    remove(repo.root.as_path());
}

/// A claimed push counts as unfinished,
/// and work of another kind (upkeep,
/// a review) is not a backup and is not reported.
#[test]
fn a_claimed_push_is_reported_and_other_work_is_not() {
    let repo: Repo = repo("rescue-claimed");
    let work: PathBuf = work_directory(repo.linked.as_path());
    enqueue(work.as_path(), &item(WorkKind::Push, COMMIT)).expect("push");
    enqueue(work.as_path(), &item(WorkKind::Maintenance, OTHER_COMMIT)).expect("upkeep");
    let claimed = crate::work_item::claim_next_of(work.as_path(), &crate::claim_time::ClaimTime::read(), &[WorkKind::Push]).expect("claim");
    assert!(claimed.is_some());
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    settle_removals(repo.common.as_path());
    let texts: Vec<String> = shown(&repo);
    assert_eq!(texts.len(), 1, "{texts:?}");
    assert!(texts[0].contains(COMMIT), "{texts:?}");
    remove(repo.root.as_path());
}

/// A worktree with no records is not snapshotted,
/// so the commands that find nothing pay nothing.
#[test]
fn a_worktree_without_records_has_no_snapshot() {
    let repo: Repo = repo("rescue-nothing");
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    assert!(!repo.common.join(SNAPSHOT_DIRECTORY).exists());
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(shown(&repo).is_empty());
    remove(repo.root.as_path());
}

/// A directory for the notices of removed worktrees that cannot be made keeps what the snapshot holds and says so,
/// for the
///  notice and for the push alike;
/// once it can be made,
/// the next command shows both and the snapshot goes.
#[test]
fn notices_that_cannot_be_promoted_stay_in_the_snapshot_and_are_told() {
    let repo: Repo = repo("rescue-blocked");
    records(&repo);
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    let blocker: PathBuf = repo.common.join(ORPHAN_NOTICE_DIRECTORY);
    std::fs::create_dir_all(blocker.parent().expect("parent")).expect("parent directory");
    std::fs::write(&blocker, "a file where the directory belongs").expect("blocker");
    let problems: Vec<String> = settle_removals(repo.common.as_path());
    assert_eq!(problems.len(), 2, "one for the notice and one for the push: {problems:?}");
    assert!(problems.iter().any(|text| { return text.contains(snapshot_of(&repo).to_string_lossy().as_ref()); }), "{problems:?}");
    assert!(snapshot_of(&repo).exists(), "the snapshot is the last copy and stays");
    assert_eq!(files_of(snapshot_of(&repo).join(NOTICES).as_path()).len(), 1);
    assert_eq!(files_of(snapshot_of(&repo).join(WORK).as_path()).len(), 1);
    std::fs::remove_file(&blocker).expect("the cause is fixed");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert_eq!(shown(&repo).len(), 2);
    assert!(!snapshot_of(&repo).exists());
    remove(repo.root.as_path());
}

/// A push that was promoted is not promoted again when another record of the same snapshot had to wait:
/// it is removed from
///  the snapshot as it is promoted.
#[test]
fn a_promoted_push_is_not_recorded_again_when_the_snapshot_is_tried_again() {
    let repo: Repo = repo("rescue-partial");
    records(&repo);
    snapshot_linked_worktrees(repo.common.as_path());
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    // The notice cannot be moved onto a directory of its name; the push record can be written.
    let notice: String = name_of(files_of(snapshot_of(&repo).join(NOTICES).as_path())[0].as_path());
    let in_the_way: PathBuf = repo.common.join(ORPHAN_NOTICE_DIRECTORY).join(notice.as_str());
    std::fs::create_dir_all(in_the_way.join("inner")).expect("a directory of the notice's name");
    let problems: Vec<String> = settle_removals(repo.common.as_path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(snapshot_of(&repo).exists());
    assert!(files_of(snapshot_of(&repo).join(WORK).as_path()).is_empty(), "the promoted push is out of the snapshot");
    let first: Vec<String> = shown(&repo);
    assert_eq!(first.len(), 1, "{first:?}");
    assert!(first[0].contains(OTHER_COMMIT), "{first:?}");
    std::fs::remove_dir_all(&in_the_way).expect("the cause is fixed");
    assert!(settle_removals(repo.common.as_path()).is_empty());
    let second: Vec<String> = shown(&repo);
    assert_eq!(second.len(), 1, "only the notice is left to show: {second:?}");
    assert!(second[0].contains("refused by test policy"), "{second:?}");
    remove(repo.root.as_path());
}

/// A snapshot that cannot be made at all is told to the person,
/// and so is one that could only be made in part,
/// which keeps
///  what was copied:
/// the push is still shown after the removal though the notice could not be copied.
#[test]
fn a_snapshot_that_could_not_be_taken_is_told_and_what_was_copied_is_kept() {
    let whole: Repo = repo("rescue-uncreatable");
    records(&whole);
    std::fs::write(whole.common.join("monochromatic-git-policy-cli"), "a file where the directory belongs").expect("blocker");
    let problems: Vec<String> = snapshot_linked_worktrees(whole.common.as_path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains(whole.linked.to_string_lossy().as_ref()), "the worktree is named: {problems:?}");
    remove(whole.root.as_path());

    let partial: Repo = repo("rescue-partial-copy");
    records(&partial);
    std::fs::create_dir_all(snapshot_of(&partial)).expect("snapshot directory");
    std::fs::write(snapshot_of(&partial).join(NOTICES), "a file where the directory belongs").expect("blocker");
    let said: Vec<String> = snapshot_linked_worktrees(partial.common.as_path());
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("could not all be saved"), "{said:?}");
    std::fs::remove_dir_all(&partial.linked).expect("Git removes the worktree");
    let settled: Vec<String> = settle_removals(partial.common.as_path());
    let texts: Vec<String> = shown(&partial);
    assert!(texts.iter().any(|text| { return text.contains(OTHER_COMMIT); }), "the push that was copied is shown: {texts:?} {settled:?}");
    remove(partial.root.as_path());
}

/// A second snapshot of a worktree adds to the first and does not replace it,
/// so records the removal in progress has already
///  deleted from the worktree are still in the snapshot.
#[test]
fn a_second_snapshot_keeps_what_the_first_copied() {
    let repo: Repo = repo("rescue-twice");
    records(&repo);
    snapshot_linked_worktrees(repo.common.as_path());
    for path in files_of(notice_directory(repo.linked.as_path()).as_path()) {
        std::fs::remove_file(path).expect("the removal in progress deleted the notice");
    }
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty(), "a copy that is already there is not a failure");
    assert_eq!(files_of(snapshot_of(&repo).join(NOTICES).as_path()).len(), 1);
    assert_eq!(files_of(snapshot_of(&repo).join(ATTEMPTS).as_path()).len(), 1, "one command, one attempt");
    std::fs::remove_dir_all(&repo.linked).expect("Git removes the worktree");
    settle_removals(repo.common.as_path());
    assert_eq!(shown(&repo).len(), 2);
    remove(repo.root.as_path());
}

/// An attempt left by an earlier process that had this process's number does not stand for this process:
/// the snapshot names
///  the identity of the process that took it now,
/// so the removal it starts is not read as one that has ended.
#[test]
fn an_attempt_left_by_an_earlier_process_of_the_same_number_is_replaced() {
    let repo: Repo = repo("rescue-reused-number");
    records(&repo);
    let attempts: PathBuf = snapshot_of(&repo).join(ATTEMPTS);
    std::fs::create_dir_all(&attempts).expect("attempts directory");
    let own: PathBuf = attempts.join(std::process::id().to_string());
    std::fs::write(&own, ENDED_IDENTITY).expect("an attempt of an ended process that had this number");
    assert!(snapshot_linked_worktrees(repo.common.as_path()).is_empty());
    assert_eq!(std::fs::read_to_string(&own).expect("attempt"), current_birth_identity().expect("identity"));
    assert!(settle_removals(repo.common.as_path()).is_empty());
    assert!(snapshot_of(&repo).exists(), "the removal this process starts is not read as ended");
    remove(repo.root.as_path());
}

/// A push record is found in whichever state it is in now:
/// the service moves it from `pending` to `claimed` between the
///  listing and the read,
/// and a record that finished is in neither.
#[test]
fn a_push_record_that_moved_between_states_is_found_where_it_is() {
    let repo: Repo = repo("rescue-moved");
    let work: PathBuf = work_directory(repo.linked.as_path());
    enqueue(work.as_path(), &item(WorkKind::Push, COMMIT)).expect("push");
    let names: Vec<String> = push_record_names(work.as_path());
    assert_eq!(names.len(), 1);
    let pending = locate_push_record(work.as_path(), names[0].as_str()).expect("read").expect("found");
    assert_eq!(pending.0, PENDING);
    let claimed = crate::work_item::claim_next_of(work.as_path(), &crate::claim_time::ClaimTime::read(), &[WorkKind::Push]).expect("claim");
    assert!(claimed.is_some());
    let moved = locate_push_record(work.as_path(), names[0].as_str()).expect("read").expect("found where it moved to");
    assert_eq!(moved.0, CLAIMED);
    assert_eq!(moved.1, pending.1, "the same record");
    assert!(locate_push_record(work.as_path(), "push-finished").expect("read").is_none(), "a record in neither state has finished");
    remove(repo.root.as_path());
}

/// A file that is gone by the time it is copied was shown or finished by someone quicker:
/// nothing to keep,
/// nothing wrong.
#[test]
fn a_file_that_vanished_before_it_was_copied_is_not_a_failure() {
    let repo: Repo = repo("rescue-vanished");
    let target: PathBuf = repo.root.join("copies");
    assert!(!copy_into(target.as_path(), "gone", repo.root.join("absent").as_path()).expect("not an error"));
    assert!(!target.exists(), "nothing was made for it");
    let source: PathBuf = repo.root.join("present");
    std::fs::write(&source, "kept").expect("source");
    assert!(copy_into(target.as_path(), "present", source.as_path()).expect("copied"));
    assert_eq!(std::fs::read_to_string(target.join("present")).expect("copy"), "kept");
    remove(repo.root.as_path());
}
