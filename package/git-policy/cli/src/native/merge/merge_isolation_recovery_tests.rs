//! Controls for the recovery of isolated merges,
//! each on a repository left in one of the states a killed merge
//! leaves.

use super::*;
use crate::index_holder_wait::IndexWait;
use crate::merge_isolation_journal::{intent_path, write_merge_intent};
use crate::merge_isolation_recovery_steps::may_install;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_owner::{TransactionOwner, current_transaction_owner};

/// A repository where `main` and `feature` have diverged and the merge commit exists but is not applied.
struct World {
    /// The fixture directory removed at the end.
    parent: PathBuf,
    /// The worktree.
    repo: PathBuf,
    /// The Git directory.
    git_dir: PathBuf,
    /// `main` before the merge.
    old_oid: String,
    /// The merge commit.
    new_oid: String,
}

/// Build the world:
/// `feature` changes `f.txt`,
/// adds `dir/new.txt` and deletes `gone.txt`;
/// `main` changes `h.txt`.
fn world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for file in ["u.txt", "f.txt", "gone.txt", "h.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--", "u.txt", "f.txt", "gone.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"new\n").expect("file");
    git(repo.as_path(), &["rm", "--quiet", "--", "gone.txt"]);
    git(repo.as_path(), &["add", "--", "f.txt", "dir/new.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// The recovery place for a world.
fn place(world: &World) -> RecoveryPlace {
    return RecoveryPlace { git: git_context(), cwd: world.repo.clone(), common_dir: world.git_dir.clone(), git_dir: world.git_dir.clone() };
}

/// An owner that is certainly gone:
/// this process's PID with a birth identity no process has.
fn dead_owner(id: &str) -> TransactionOwner {
    let mut owner: TransactionOwner = current_transaction_owner(id, "2026-10-29T00:00:00.000Z").expect("owner");
    owner.owner_identity = String::from("linux:1");
    return owner;
}

/// Write the record of a merge of the world,
/// owned by `owner`,
/// and return its file.
fn record_of(world: &World, owner: TransactionOwner, target_ref: Option<&str>) -> PathBuf {
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: world.repo.to_string_lossy().into_owned(),
        target_ref: target_ref.map(String::from),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
}

/// Everything about the staged entries and the worktree that another session would notice.
fn snapshot(world: &World) -> (String, String) {
    return (
        git_text(world.repo.as_path(), &["ls-files", "--stage"]),
        git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]),
    );
}

/// A merge killed before it touched anything is discarded,
/// and the checkout is as it was.
#[test]
fn a_merge_that_never_began_is_discarded() {
    let world: World = world("merge-recovery-never-began");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("11111111-0000-4000-8000-000000000001"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(
        outcomes,
        vec![MergeRecoveryOutcome {
            record: record.clone(),
            action: MergeRecoveryAction::Discarded,
            target_ref: Some(String::from("refs/heads/main")),
            new_oid: world.new_oid.clone(),
        }]
    );
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(snapshot(&world), before);
    remove(world.parent.as_path());
}

/// A record whose owner still runs is left alone and is not a reason to take the landing lock.
#[test]
fn a_live_owners_record_is_left_alone() {
    let world: World = world("merge-recovery-live");
    let live: TransactionOwner =
        current_transaction_owner("22222222-0000-4000-8000-000000000002", "2026-10-29T00:00:00.000Z").expect("owner");
    let record: PathBuf = record_of(&world, live, Some("refs/heads/main"));
    assert!(!merge_intents_to_recover(world.git_dir.as_path()).expect("scan"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::OwnerActive);
    assert!(record.exists());
    remove(world.parent.as_path());
}

/// A dead owner's record,
/// a malformed record and no record are told apart without a lock or a process.
#[test]
fn only_dead_or_unreadable_records_need_recovery() {
    let world: World = world("merge-recovery-scan");
    assert!(!merge_intents_to_recover(world.git_dir.as_path()).expect("no directory"));
    let record: PathBuf = record_of(&world, dead_owner("33333333-0000-4000-8000-000000000003"), None);
    assert!(merge_intents_to_recover(world.git_dir.as_path()).expect("dead owner"));
    std::fs::write(record.as_path(), b"{").expect("damage");
    assert!(merge_intents_to_recover(world.git_dir.as_path()).expect("malformed"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("fails closed");
    assert!(error.0.contains("malformed") && error.0.contains(record.to_string_lossy().as_ref()), "{error}");
    assert!(record.exists(), "a record that cannot be read is kept");
    remove(world.parent.as_path());
}

/// A branch that already names the merge needs only the record removed.
#[test]
fn a_landed_merge_only_loses_its_record() {
    let world: World = world("merge-recovery-landed");
    git(world.repo.as_path(), &["merge", "--ff-only", "--quiet", world.new_oid.as_str()]);
    let record: PathBuf = record_of(&world, dead_owner("44444444-0000-4000-8000-000000000004"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// A branch that moved to something else before the merge touched the checkout:
/// nothing to put back,
/// the record goes,
/// and
/// the merge can be computed again on the moved head.
#[test]
fn a_branch_that_moved_before_the_checkout_changed_only_loses_its_record() {
    let world: World = world("merge-recovery-moved");
    std::fs::write(world.repo.join("h.txt"), b"h.txt again\n").expect("file");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", "--message=elsewhere"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let record: PathBuf = record_of(&world, dead_owner("55555555-0000-4000-8000-000000000005"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// The branch moved after Git installed the merge's paths (the fast-forward lost the race for the branch between its
/// checkout and its ref update):
/// the paths go back to what the moved head holds,
/// so the checkout matches the head,
/// another
/// session's staged entry is untouched,
/// and nothing of the merge is left half applied.
#[test]
fn a_branch_that_moved_after_the_install_puts_the_merged_paths_back() {
    let world: World = world("merge-recovery-moved-after-install");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]);
    // The winner: a commit on top of the old head that adds a file the merge does not touch, checked out.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    // The loser: Git's two-way update from the old head to the merge, whose ref update then failed.
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    assert!(world.repo.join("dir/new.txt").exists(), "the merge was installed");
    let record: PathBuf = record_of(&world, dead_owner("77777777-0000-4000-8000-000000000007"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), moved, "the branch is the winner's");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt", "only the other session's work differs from the head");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]), staged_entry);
    assert!(!world.repo.join("dir").exists() && world.repo.join("gone.txt").exists(), "the merge's changes are put back");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"f.txt base\n");
    assert!(world.repo.join("z.txt").exists(), "the winner's file stays");
    remove(world.parent.as_path());
}

/// Git updated the worktree and the index and died before it moved the branch:
/// recovery moves the branch,
/// and
/// another session's staged entry is exactly as it was.
#[test]
fn a_merge_killed_after_the_checkout_update_is_completed() {
    let world: World = world("merge-recovery-after-checkout");
    std::fs::write(world.repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "u.txt"]);
    let staged_entry: String = git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]);
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    let record: PathBuf = record_of(&world, dead_owner("66666666-0000-4000-8000-000000000006"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt");
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "u.txt"]), staged_entry);
    assert!(!world.repo.join("gone.txt").exists());
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("added file"), b"new\n");
    remove(world.parent.as_path());
}

/// Git wrote some files and died before it wrote the index:
/// recovery finishes the files and the index.
#[test]
fn a_merge_killed_between_the_worktree_and_the_index_is_completed() {
    let world: World = world("merge-recovery-before-index");
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the file Git removed");
    let record: PathBuf = record_of(&world, dead_owner("77777777-0000-4000-8000-000000000007"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    assert_eq!(std::fs::read(world.repo.join("dir/new.txt")).expect("added file"), b"new\n");
    remove(world.parent.as_path());
}

/// A path that holds neither its old nor its merged content belongs to someone else:
/// nothing is changed.
#[test]
fn a_foreign_edit_stops_the_completion() {
    let world: World = world("merge-recovery-foreign");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    std::fs::write(world.repo.join("f.txt"), b"someone else's edit\n").expect("foreign edit");
    let record: PathBuf = record_of(&world, dead_owner("88888888-0000-4000-8000-000000000008"), Some("refs/heads/main"));
    let error = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("fails closed");
    assert!(error.0.contains("f.txt") && error.0.contains("Nothing was changed"), "{error}");
    assert!(record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"someone else's edit\n");
    remove(world.parent.as_path());
}

/// A change made by someone else before the merge began is not evidence that it began.
#[test]
fn an_unrelated_edit_does_not_make_a_merge_that_never_began_complete() {
    let world: World = world("merge-recovery-edit-only");
    std::fs::write(world.repo.join("f.txt"), b"an edit made while the merge was dead\n").expect("edit");
    let record: PathBuf = record_of(&world, dead_owner("99999999-0000-4000-8000-000000000009"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("file"), b"an edit made while the merge was dead\n");
    remove(world.parent.as_path());
}

/// Another session's staged change to a path the merge changes is not evidence that the merge began.
#[test]
fn a_staged_change_of_a_merged_path_is_not_a_started_merge() {
    let world: World = world("merge-recovery-staged-same-path");
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by another session\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("cccccccc-0000-4000-8000-00000000000c"), Some("refs/heads/main"));
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    assert_eq!(snapshot(&world), before);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    remove(world.parent.as_path());
}

/// A detached `HEAD` moves by `--no-deref`.
#[test]
fn a_detached_head_is_completed_too() {
    let world: World = world("merge-recovery-detached");
    git(world.repo.as_path(), &["switch", "--quiet", "--detach", "HEAD"]);
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let record: PathBuf = record_of(&world, dead_owner("aaaaaaaa-0000-4000-8000-00000000000a"), None);
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert!(!record.exists());
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), world.new_oid);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "");
    remove(world.parent.as_path());
}

/// The merge's own process settles a failure the same way,
/// whatever its owner.
#[test]
fn a_failed_merge_settles_without_a_liveness_check() {
    let world: World = world("merge-recovery-settle");
    let live: TransactionOwner =
        current_transaction_owner("bbbbbbbb-0000-4000-8000-00000000000b", "2026-10-29T00:00:00.000Z").expect("owner");
    let record: PathBuf = record_of(&world, live.clone(), Some("refs/heads/main"));
    let intent: MergeIntent = parse_merge_intent(std::fs::read(record.as_path()).expect("read").as_slice()).expect("intent");
    assert_eq!(intent_path(world.git_dir.as_path(), live.transaction_id.as_str()), record);
    let action = settle_failed_merge(&place(&world), record.as_path(), &intent, &mut Vec::new()).expect("settled");
    assert_eq!(action, MergeRecoveryAction::Discarded);
    assert!(!record.exists());
    remove(world.parent.as_path());
}

/// What:
///  The starts a recovery makes while it completes a merge that wrote only some of its files.
/// Why:
///   The owner's rule is that no Git start of the wrapper gets `--quiet` or `-q`.
/// The recovery's `checkout-index` is the one
///  that carried it (Git 2.56.0 `builtin/checkout-index.c` gates only the message on it,
/// and the exit status is 1 for a
///  path that is not in the index with or without it);
/// the control looks at the start that checks out the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInterruptedMerges(place(recordingGit)); expect(startsOf('checkout-index')).toHaveLength(1); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_recovery_checks_files_out_without_quiet() {
    let world: World = world("merge-recovery-no-quiet");
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the file Git removed");
    let _record: PathBuf = record_of(&world, dead_owner("88888888-0000-4000-8000-000000000008"), Some("refs/heads/main"));
    let (mut recording, log) = crate::recording_git::recording_context(world.parent.as_path());
    // The checkout and the index update hand their paths on standard input, which a request makes in the common directory.
    recording.common_dir = Some(world.git_dir.clone());
    let recording_place: RecoveryPlace =
        RecoveryPlace { git: recording, cwd: world.repo.clone(), common_dir: world.git_dir.clone(), git_dir: world.git_dir.clone() };
    let outcomes = recover_interrupted_merges(&recording_place, world.git_dir.as_path(), &mut Vec::new()).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    let checkouts: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.contains("checkout-index"); }).collect();
    assert_eq!(checkouts.len(), 1, "one start checks the files out: {checkouts:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(world.parent.as_path());
}

/// A record whose owner is in another PID namespace and older than the patience is neither alive nor dead (owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"):
/// recovery stops with an error that names the record,
/// the owner,
/// the namespace,
/// the age and
/// the `rm`,
/// and changes nothing:
/// the record stays,
/// the branch stays,
/// the staged entries and the worktree stay.
#[test]
fn an_old_record_of_another_namespace_is_refused_and_named_and_nothing_changes() {
    let world: World = world("merge-recovery-foreign-expired");
    let mut owner: TransactionOwner = dead_owner("11111111-0000-4000-8000-000000000042");
    owner.owner_pid_namespace = Some(String::from("pid:[4026531999]"));
    owner.owner_pid = 31337;
    let record: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    std::fs::File::options().write(true).open(record.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age the record");
    let before = snapshot(&world);
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("an expired foreign record is refused");
    let text: String = refused.0;
    assert!(text.contains(record.to_string_lossy().as_ref()), "the record file is named: {text}");
    assert!(text.contains("31337") && text.contains("pid:[4026531999]"), "the owner and its namespace are named: {text}");
    assert!(text.contains("minutes old") && text.contains("another container") && text.contains("rm -- "), "age, the live-owner possibility and the rm: {text}");
    assert!(text.contains("left running"), "the condition covers Git processes the wrapper left running, not only a monochromatic-git-policy-cli command: {text}");
    assert!(record.exists(), "the record is untouched");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(snapshot(&world), before);
    assert!(merge_intents_to_recover(world.git_dir.as_path()).is_err(), "the check that decides whether recovery is needed refuses as well");
    remove(world.parent.as_path());
}

/// Records `[dead A, expired-foreign B]`:
/// recovery refuses before it settles A (sol-1 round 3,
/// P1 C:
/// the refusal said nothing was
/// changed after the first record had been settled).
#[test]
fn an_old_record_of_another_namespace_after_a_dead_one_stops_recovery_before_the_dead_one_is_settled() {
    let world: World = world("merge-recovery-foreign-after-dead");
    let dead: PathBuf = record_of(&world, dead_owner("00000000-0000-4000-8000-000000000001"), Some("refs/heads/main"));
    let mut owner: TransactionOwner = dead_owner("ffffffff-0000-4000-8000-000000000002");
    owner.owner_pid_namespace = Some(String::from("pid:[4026531999]"));
    owner.owner_pid = 31337;
    let foreign: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    std::fs::File::options().write(true).open(foreign.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age the record");
    assert!(dead.file_name() < foreign.file_name(), "the dead owner's record is read first");
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new()).expect_err("an expired foreign record is refused");
    assert!(refused.0.contains("31337"), "{}", refused.0);
    assert!(dead.exists() && foreign.exists(), "nothing was settled before the refusal");
    remove(world.parent.as_path());
}

/// What:
///  A merge that recovery finishes tells the person so,
///  naming the ref and the commit,
///  and a merge it discards tells them nothing.
/// Why:
///  A command run only to look at the worktree must not move the branch onto a merge commit in silence
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 1);
/// the notice is the one record of that change outside the debug log,
/// and it goes to standard error alone,
/// because scripts read the command's standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = []; await recoverInterruptedMerges(place, gitDir, notices); expect(notices).toEqual([line]); expect(stdout).toBe('');
/// ```
#[test]
fn a_completed_merge_is_reported_with_the_ref_and_the_commit() {
    let world: World = world("merge-recovery-reported");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let _record: PathBuf = record_of(&world, dead_owner("dddddddd-0000-4000-8000-00000000000d"), Some("refs/heads/main"));
    let mut notices: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut notices).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    let said: String = String::from_utf8(notices).expect("the notices are UTF-8");
    assert_eq!(said.lines().count(), 1, "one line for the one merge that changed the branch: {said}");
    assert!(said.contains("refs/heads/main"), "the ref is named: {said}");
    assert!(said.contains(world.new_oid.as_str()), "the merge commit is named: {said}");
    assert!(said.starts_with("monochromatic-git-policy-cli: "), "the line is the wrapper's: {said}");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// A record that is only discarded,
/// a live owner's record and a branch that already names the merge change nothing of the branch,
/// so they say nothing.
#[test]
fn a_merge_that_changed_nothing_is_not_reported() {
    let discarded: World = world("merge-recovery-unreported");
    std::fs::write(discarded.repo.join("f.txt"), b"an edit made while the merge was dead\n").expect("edit");
    let _record: PathBuf = record_of(&discarded, dead_owner("eeeeeeee-0000-4000-8000-00000000000e"), Some("refs/heads/main"));
    let mut notices: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&discarded), discarded.git_dir.as_path(), &mut notices).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert_eq!(String::from_utf8_lossy(&notices), "", "a discarded merge was never in the checkout");
    remove(discarded.parent.as_path());

    let landed: World = world("merge-recovery-unreported-landed");
    git(landed.repo.as_path(), &["merge", "--ff-only", "--quiet", landed.new_oid.as_str()]);
    let _landed_record: PathBuf = record_of(&landed, dead_owner("ffffffff-0000-4000-8000-00000000000f"), Some("refs/heads/main"));
    let mut landed_notices: Vec<u8> = Vec::new();
    let landed_outcomes = recover_interrupted_merges(&place(&landed), landed.git_dir.as_path(), &mut landed_notices).expect("recovery");
    assert_eq!(landed_outcomes[0].action, MergeRecoveryAction::AlreadyLanded);
    assert_eq!(String::from_utf8_lossy(&landed_notices), "", "a branch that already names the merge did not move");
    remove(landed.parent.as_path());
}

/// What:
///  A writer that lets another program's `index.lock` go the first time the wait says what it waits for,
///  and keeps everything the wait wrote.
/// Why:
///  The control must not sleep until a lock is released:
/// the notice of the wait is the instant the holder is known to be
///  waited for (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReleasingSink { write(bytes) { this.said += bytes; if (exists(lock)) rm(lock); return bytes.length; } }
/// ```
struct ReleasingSink<'a> {
    /// The lock the other program holds.
    lock: &'a Path,
    /// Everything the wait wrote.
    said: Vec<u8>,
}

impl std::io::Write for ReleasingSink<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.said.extend_from_slice(bytes);
        // A wait that has said what it waits for is waiting; the other program lets go at that instant.
        if self.lock.exists() {
            std::fs::remove_file(self.lock)?;
        }
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A recovery waits for another program's `index.lock`,
///  says what it waits for,
///  and finishes the merge when the lock is free.
/// Why:
///  `git update-index` dies on a held lock,
/// so one held lock used to end every command in the worktree with
///  status 2 until it was gone (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sink = new ReleasingSink(lock); await recoverInterruptedMerges(place, dir, sink); expect(outcome.action).toBe('completed');
/// ```
#[test]
fn recovery_waits_for_another_programs_index_lock() {
    let world: World = world("merge-recovery-waits-for-lock");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let _record: PathBuf = record_of(&world, dead_owner("12121212-0000-4000-8000-000000000012"), Some("refs/heads/main"));
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("another program holds the index");
    let mut sink: ReleasingSink<'_> = ReleasingSink { lock: lock.as_path(), said: Vec::new() };
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut sink).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed, "the merge is finished once the lock is free");
    let said: String = String::from_utf8(sink.said).expect("the notices are UTF-8");
    assert!(said.contains(lock.to_string_lossy().as_ref()), "the lock is named: {said}");
    assert!(said.contains("waits for the index lock"), "it says what it waits for: {said}");
    assert!(!lock.exists(), "the lock was the other program's and nothing removed it on the wrapper's side");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.new_oid);
    remove(world.parent.as_path());
}

/// What:
///  A wait that ends with the index still held writes nothing,
///  and only a wait that ends with the lock free lets the merge go on.
/// Why:
///  The decision is the whole of recovery's behaviour on a held lock,
/// and both halves of it are checked:
///  a recovery that wrote the index after giving up would put Git's own sentence in place of the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(mayInstall({ kind: 'not-held' })).toBe(true); expect(mayInstall({ kind: 'owner-gone' })).toBe(false);
/// ```
#[test]
fn only_a_lock_that_is_free_lets_the_merge_be_written() {
    let second: std::time::Duration = std::time::Duration::from_secs(1);
    assert!(may_install(IndexWait::NotHeld));
    assert!(may_install(IndexWait::Released(second)));
    assert!(!may_install(IndexWait::GaveUp(second)));
    assert!(!may_install(IndexWait::OwnerGone(second)));
}

/// What:
///  A lock whose PID file names a process that is gone is not waited for,
///  and the merge is left for the next command with its record kept and nothing written.
/// Why:
///  This ends at once,
/// without a ceiling,
/// so the control proves the deferred outcome:
/// a command in the worktree goes on,
/// and the next one finishes
///  the same merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePidFile(index, gonePid); const action = await settleFailedMerge(place, record, intent); expect(action).toBe('index-held'); expect(record.exists()).toBe(true);
/// ```
#[test]
fn a_dead_holders_lock_leaves_the_merge_for_the_next_command() {
    let world: World = world("merge-recovery-dead-holder-lock");
    git(world.repo.as_path(), &["read-tree", "-m", "-u", world.old_oid.as_str(), world.new_oid.as_str()]);
    let before = snapshot(&world);
    let record: PathBuf = record_of(&world, dead_owner("13131313-0000-4000-8000-000000000013"), Some("refs/heads/main"));
    let intent: MergeIntent = parse_merge_intent(std::fs::read(record.as_path()).expect("read").as_slice()).expect("intent");
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a Git left the index lock");
    std::fs::write(crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()), b"pid 4294967\n").expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let action = settle_failed_merge(&place(&world), record.as_path(), &intent, &mut said).expect("settled");
    assert_eq!(action, MergeRecoveryAction::IndexHeld, "nothing was written");
    assert!(record.exists(), "the record stays for the next command");
    assert!(lock.exists(), "the lock is not the recovery's to remove");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), world.old_oid, "the branch did not move");
    assert_eq!(snapshot(&world), before, "the index and the worktree are exactly as they were");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("no longer runs") && text.contains("stays unfinished"), "it says what is unfinished: {text}");
    remove(world.parent.as_path());
}
