//! Controls for the steps after real Git returned: the snapshot rung serves a clean source with or without a tree
//!  difference and prints Git's line; every refusal destroys the stage and completes the worktree with `reset --hard`;
//!  `--quiet` is honored on both; and no step runs a hook.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::test_support::{REAL_GIT, git, git_context, git_text, remove, repository};
use crate::worktree_copy::GitStep;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_run::begin_ladder;
use crate::worktree_copy_journal::journal_root;
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked};
use std::cell::RefCell;
use std::process::{Command, Stdio};

/// Every step the runner was asked for, as the words of its arguments.
type Steps = RefCell<Vec<Vec<String>>>;

/// A runner that records each step and runs the commands of `Other` steps with real Git in an isolated environment.
fn recording_runner(steps: &Steps, fail_reset: bool) -> impl FnMut(Option<&str>, &GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> + '_ {
    return move |_token, step| {
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        steps.borrow_mut().push(arguments.iter().map(|word: &OsString| { return word.to_string_lossy().into_owned(); }).collect());
        if fail_reset && arguments.iter().any(|word: &OsString| { return word == "reset"; }) {
            return Ok(GitStatus { code: Some(1) });
        }
        let status = Command::new(REAL_GIT)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .envs(environment.iter().cloned())
            .args(arguments.iter())
            .stdin(Stdio::null())
            .status()
            .expect("Git starts");
        return Ok(GitStatus { code: status.code() });
    };
}

/// Run Git without a checkout the way the lifecycle does, then the ladder's `finish`; return the result, the steps and
///  what was printed.
fn add_through_the_ladder(repo: &Path, root: &Path, extra: &[&str], fail_reset: bool, spoil: impl FnOnce(&Path)) -> (LadderResult, Vec<Vec<String>>, String) {
    let destination: PathBuf = root.join("topic");
    let mut plan = plan_to(destination.clone());
    plan.quiet = extra.contains(&"--quiet");
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "the source is eligible");
    let mut words: Vec<&str> = vec!["worktree", "add", "--no-checkout"];
    words.extend_from_slice(extra);
    words.extend(["-b", "topic", "../topic"]);
    git(repo, words.as_slice());
    spoil(destination.as_path());
    let steps: Steps = RefCell::new(Vec::new());
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = {
        let mut runner = recording_runner(&steps, fail_reset);
        ladder.finish(GitStatus { code: Some(0) }, None, &mut runner, &git_context(), &mut printed)
    };
    return (result, steps.into_inner(), String::from_utf8_lossy(printed.as_slice()).into_owned());
}

/// The first, unchanged tree needs no Git step at all and prints Git's line.
#[test]
fn btrfs_a_source_at_the_target_commit_is_served_with_no_git_step() {
    let Some(root) = btrfs_fixture("finish-same") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(0) }, carried: Some(root.join("topic")) });
    assert!(steps.is_empty(), "{steps:?}");
    let head: String = git_text(repo.as_path(), &["log", "-1", "--format=HEAD is now at %h %s"]);
    assert_eq!(printed, format!("{head}\n"));
    assert!(is_subvolume_root(root.join("topic").as_path()).expect("probe"));
    assert!(root.join("topic/ignored/pkg/index.js").is_file());
    remove(root.as_path());
}

/// `--quiet` prints nothing.
#[test]
fn btrfs_quiet_prints_no_line() {
    let Some(root) = btrfs_fixture("finish-quiet") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, _steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], false, |_| {});
    assert!(result.carried.is_some());
    assert_eq!(printed, "");
    remove(root.as_path());
}

/// A destination that holds more than Git's pointer is not served: the stage is destroyed, `reset --hard` completes the
///  worktree with the work tree and git directory Git gives its own checkout, and nothing is carried.
#[test]
fn btrfs_a_refused_step_destroys_the_stage_and_completes_the_checkout_like_git() {
    let Some(root) = btrfs_fixture("finish-fallback") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(0) }, carried: None });
    assert_eq!(printed, "", "Git's own reset prints the line, not the ladder");
    assert_eq!(steps.len(), 1, "{steps:?}");
    assert_eq!(&steps[0][2..], ["reset", "--hard", "--no-recurse-submodules"], "{steps:?}");
    assert!(root.join("topic/shared.txt").is_file(), "the checkout completed");
    assert!(!is_subvolume_root(root.join("topic").as_path()).expect("probe"), "the stage was not swapped in");
    let leftovers: Vec<String> = std::fs::read_dir(root.as_path()).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name: &String| { return name.starts_with(".cli-git"); }).collect();
    assert_eq!(leftovers, Vec::<String>::new(), "the stage is gone");
    remove(root.as_path());
}

/// A failing `reset --hard` gives its status to the person, as Git's own would.
#[test]
fn btrfs_a_failing_checkout_step_gives_its_status() {
    let Some(root) = btrfs_fixture("finish-reset-fails") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], true, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(1) }, carried: None });
    assert!(!root.join("topic").exists(), "Git removes the worktree whose checkout failed, and so does the ladder");
    assert!(!repo.join(".git/worktrees/topic").exists(), "the registration goes with it");
    remove(root.as_path());
}

/// A clean ordinary repository with two commits: the clone rung serves it.
fn ordinary_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    commit_all(repo.as_path(), "second");
    return repo;
}

/// A file another process put into the destination after Git made it is no clone's to remove: the destination holds more
///  than Git's pointer, so the rung is refused, and Git's own checkout leaves a file it has no entry for.
#[test]
fn btrfs_a_refused_clone_rung_leaves_a_file_it_did_not_create() {
    let Some(root) = btrfs_fixture("finish-clone-stray") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let (result, steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |destination| {
        std::fs::write(destination.join("notes"), b"mine\n").expect("a file of someone else");
    });
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(steps.len(), 1, "{steps:?}");
    assert_eq!(std::fs::read(root.join("topic/notes")).expect("kept"), b"mine\n", "a non-conflicting file is not deleted");
    assert!(root.join("topic/shared.txt").is_file(), "the checkout completed");
    remove(root.as_path());
}

/// When the checkout that completes a refused clone rung fails, the worktree and its registration are removed, as Git's own
///  `worktree add` removes what it registered, and the failing status is the person's.
#[test]
fn btrfs_a_failing_checkout_after_a_refused_clone_rung_removes_the_worktree_and_its_registration() {
    let Some(root) = btrfs_fixture("finish-clone-rollback") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], true, |destination| {
        std::fs::write(destination.join("notes"), b"x").expect("a stray file");
    });
    assert_eq!(result.git, GitStatus { code: Some(1) });
    assert!(!root.join("topic").exists(), "no half-made worktree");
    assert!(!repo.join(".git/worktrees/topic").exists(), "no registration for it");
    remove(root.as_path());
}

/// A writer that reports whether the completion marker was on disk when the line for the person was written.
struct MarkerProbe {
    /// The directory of the claims.
    claims: PathBuf,
    /// Whether a marker existed at the first write.
    marker_seen: Option<bool>,
    /// Whether the add's record named the directories Git made at the first write.
    bound_seen: Option<bool>,
}

impl Write for MarkerProbe {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let marker: bool = std::fs::read_dir(self.claims.as_path()).is_ok_and(|listing| { return listing.flatten().any(|entry| { return entry.file_name().to_string_lossy().ends_with(".done"); }); });
        self.marker_seen.get_or_insert(marker);
        let bound: bool = std::fs::read_dir(self.claims.as_path()).is_ok_and(|listing| {
            return listing.flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".add"); }).any(|entry| { return std::fs::read_to_string(entry.path()).is_ok_and(|text: String| { return text.contains("phase=registered"); }); });
        });
        self.bound_seen.get_or_insert(bound);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// Both rungs write the completion marker before the person is told, and release the add's records after.
#[test]
fn btrfs_each_rung_marks_the_add_complete_before_it_prints_and_releases_the_records_after() {
    for (name, ordinary) in [("finish-marker-clone", true), ("finish-marker-snapshot", false)] {
        let Some(root) = btrfs_fixture(name) else { return };
        let repo: PathBuf = if ordinary { ordinary_repository(root.as_path()) } else { subvolume_repository(root.as_path()) };
        let plan = plan_to(root.join("topic"));
        let claims: PathBuf = journal_root(repo.join(".git").as_path());
        let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
        let records: Vec<String> = std::fs::read_dir(claims.as_path()).expect("claims").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
        assert!(records.iter().any(|record: &String| { return record.ends_with(".add"); }), "{name}: the add is recorded before Git runs: {records:?}");
        git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
        let mut probe: MarkerProbe = MarkerProbe { claims: claims.clone(), marker_seen: None, bound_seen: None };
        let steps: Steps = RefCell::new(Vec::new());
        let result: LadderResult = {
            let mut runner = recording_runner(&steps, false);
            ladder.finish(GitStatus { code: Some(0) }, None, &mut runner, &git_context(), &mut probe)
        };
        assert_eq!(result.git, GitStatus { code: Some(0) }, "{name}");
        assert_eq!(probe.marker_seen, Some(true), "{name}: the marker is on disk when the line is printed");
        assert_eq!(probe.bound_seen, Some(true), "{name}: the record names the directories Git made when the line is printed");
        let left: Vec<String> = std::fs::read_dir(claims.as_path()).expect("claims").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
        assert_eq!(left, Vec::<String>::new(), "{name}: the records are released");
        remove(root.as_path());
    }
}

/// The end of an add needs the marker, or the withdrawal of the record, before success: a marker that cannot be written and a
///  record that can be withdrawn is a success with a warning; when neither works the add fails and its objects are removed.
#[test]
fn confirming_an_add_needs_the_marker_or_the_withdrawal_and_otherwise_fails() {
    let withdrawn: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let removed: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let mut remove = || -> String { removed.set(removed.get() + 1); return String::from("the worktree was removed"); };
    let mut withdraw_ok = || -> Result<(), String> { withdrawn.set(withdrawn.get() + 1); return Ok(()); };
    assert_eq!(confirm_with(&mut || { return Ok(()); }, &mut withdraw_ok, &mut remove), Ok(()));
    assert_eq!((withdrawn.get(), removed.get()), (0, 0), "a marker needs no withdrawal");
    assert_eq!(confirm_with(&mut || { return Err(String::from("no space")); }, &mut withdraw_ok, &mut remove), Ok(()));
    assert_eq!((withdrawn.get(), removed.get()), (1, 0), "the withdrawal stands for the marker");
    let failure: Result<(), String> = confirm_with(&mut || { return Err(String::from("no space")); }, &mut || { return Err(String::from("busy")); }, &mut remove);
    let reason: String = failure.expect_err("neither record can be changed");
    assert!(reason.contains("no space") && reason.contains("busy") && reason.contains("the worktree was removed"), "{reason}");
    assert_eq!(removed.get(), 1);
}

/// A stage name that holds somebody else's directory after a failed swap back is not the wrapper's to destroy.
#[test]
fn an_error_from_a_failed_swap_back_does_not_own_the_stage() {
    let foreign: Ended = Ended::from(PlaceError { reason: String::from("x"), stage_is_ours: false });
    let own: Ended = Ended::from(PlaceError { reason: String::from("y"), stage_is_ours: true });
    assert!(!foreign.stage_is_ours && own.stage_is_ours);
    assert!(Ended::from(String::from("a plain reason")).stage_is_ours, "an error before the exchange left the stage ours");
}

/// When the records cannot be written and the end of the add cannot be confirmed, the add fails and the worktree goes with it:
///  no success line is printed and recovery has nothing to delete that the person was told about.
#[test]
fn btrfs_an_add_that_cannot_record_its_end_fails_and_removes_the_worktree() {
    let Some(root) = btrfs_fixture("finish-records-unwritable") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let claims: PathBuf = journal_root(repo.join(".git").as_path());
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    std::fs::set_permissions(claims.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o500)).expect("read-only records");
    let steps: Steps = RefCell::new(Vec::new());
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = {
        let mut runner = recording_runner(&steps, false);
        ladder.finish(GitStatus { code: Some(0) }, None, &mut runner, &git_context(), &mut printed)
    };
    std::fs::set_permissions(claims.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("writable again");
    assert_eq!(result.git, GitStatus { code: Some(1) });
    assert!(printed.is_empty(), "no success is printed");
    assert!(!root.join("topic").exists() && !repo.join(".git/worktrees/topic").exists(), "the worktree was removed");
    remove(root.as_path());
}

/// A rollback that cannot finish keeps the add's record and claim, so the next command reports it and retries.
#[test]
fn btrfs_a_rollback_that_fails_keeps_the_record() {
    let Some(root) = btrfs_fixture("finish-rollback-retained") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    std::fs::write(root.join("topic/notes"), b"x").expect("a stray file, so the clone rung is refused");
    let steps: Steps = RefCell::new(Vec::new());
    let mut printed: Vec<u8> = Vec::new();
    let mut runner = recording_runner(&steps, true);
    // The reset fails and, as it does, the directory the worktree stands in becomes unwritable.
    let locking_root: PathBuf = root.clone();
    let mut failing = move |token: Option<&str>, step: &GitStep<'_>| {
        let outcome = runner(token, step);
        std::fs::set_permissions(locking_root.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o500)).expect("read-only parent");
        return outcome;
    };
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, None, &mut failing, &git_context(), &mut printed);
    std::fs::set_permissions(root.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("writable again");
    assert_eq!(result.git, GitStatus { code: Some(1) });
    assert!(root.join("topic").exists() && repo.join(".git/worktrees/topic").exists(), "nothing could be removed");
    assert!(crate::worktree_copy_stage_claim::stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the record stays for the next command");
    remove(root.as_path());
}
