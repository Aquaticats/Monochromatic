//! What:
//!  A backup push the resident service made after a commit returned and the remote refused,
//! shown by the next wrapper
//!  command in the repository.
//! Why:
//!  The service pushes after the commit command exited,
//! so nothing the commit printed can carry the refusal.
//! Without a
//!  record shown by a later command the person would never learn that their backup is missing
//!  (`push_failure_notice.rs`).
//! The remote here is a real bare repository whose `pre-receive` hook refuses every push,
//! the
//!  service is a real process,
//! and the commands are the built wrapper;
//! nothing is a stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await commitThroughService(); await untilRecorded(); expect(stderrOf(await run('git status'))).toContain('backup push');
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, repository, run_direct, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::claim_time::ClaimTime;
use git_policy_cli::iso_time::now_iso;
use git_policy_cli::push_failure_notice::{ORPHAN_NOTICE_DIRECTORY, notice_directory};
use git_policy_cli::service_client::{Activation, ClientConfig, wake};
use git_policy_cli::service_instance::socket_for;
use git_policy_cli::service_server::Bind;
use git_policy_cli::service_work::{CLAIM_LEASE, work_directory};
use git_policy_cli::work_item::{WorkItem, WorkKind, claim_file_name, enqueue};
use git_policy_cli::worktree_removal_rescue::SNAPSHOT_DIRECTORY;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// What the remote's hook says when it refuses a push;
/// the test looks for it in what the person is shown.
const REFUSAL: &str = "refusing this push by test policy";

/// The longest a control waits for the service's push,
/// far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with a tracked file,
/// a bare remote that has the branch,
/// and a service the commits go through.
struct Setup {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
    service: FixtureService,
}

fn setup(name: &str) -> Setup {
    return setup_with(name, &[]);
}

/// The same,
/// with variables the service starts with,
/// as the user manager's environment would give it.
fn setup_with(name: &str, service_environment: &[(&str, &str)]) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    // Given directly: this control always starts its own service, whether or not the run gives every fixture one.
    for (variable, value) in service_environment {
        fixture.service_environment.lock().expect("the service environment").push((std::ffi::OsString::from(variable), std::ffi::OsString::from(value)));
    }
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service };
}

/// The remote refuses every push from now on.
fn refuse_pushes(remote: &Path) {
    let hook: PathBuf = remote.join("hooks/pre-receive");
    std::fs::create_dir_all(hook.parent().expect("hooks directory")).expect("hooks");
    std::fs::write(&hook, format!("#!/bin/sh\necho '{REFUSAL}' >&2\nexit 1\n")).expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("executable");
}

/// The remote takes pushes again.
fn accept_pushes(remote: &Path) {
    std::fs::remove_file(remote.join("hooks/pre-receive")).expect("hook removed");
}

/// Commits `content` to `a.txt` through the service and returns the new commit's name.
fn commit_through_service(setup: &Setup, content: &str) -> String {
    return commit_through_service_in(setup, setup.repo.as_path(), content);
}

/// The same in the worktree `directory` of the repository.
fn commit_through_service_in(setup: &Setup, directory: &Path, content: &str) -> String {
    std::fs::write(directory.join("a.txt"), content).expect("edit");
    let mut command: Command = wrapped_commit(&setup.fixture, directory, &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let named = git(&setup.fixture, directory, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// Waits until the condition holds;
/// the service's push has no signal a control could block on.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn common_dir(setup: &Setup) -> PathBuf {
    return setup.repo.join(".git");
}

/// How many failure records wait in the main worktree.
fn waiting_records(setup: &Setup) -> usize {
    return waiting_records_in(common_dir(setup).as_path());
}

/// How many failure records wait under the Git directory of one worktree.
fn waiting_records_in(git_dir: &Path) -> usize {
    return std::fs::read_dir(notice_directory(git_dir)).map_or(0, |listing| { return listing.flatten().count(); });
}

/// Waits until the service has pushed `commit` and the item is settled.
fn wait_until_settled(setup: &Setup, commit: &str) {
    wait_until_settled_in(common_dir(setup).as_path(), "refs/heads/main", commit);
}

/// The same for the work records of `branch` under the Git directory of one worktree.
fn wait_until_settled_in(git_dir: &Path, branch: &str, commit: &str) {
    let work: PathBuf = git_dir.join("monochromatic-git-policy-cli/work");
    wait_until("the service settled the push", &|| {
        // The record's own id, which carries the branch as well as the commit.
        let record: WorkItem = WorkItem {
            kind: WorkKind::Push,
            repository: PathBuf::new(),
            git_dir: PathBuf::new(),
            branch_ref: String::from(branch),
            oid: String::from(commit),
            parent: String::new(),
            created: String::new(),
            tries: 0,
        };
        return ["done", "failed"].iter().any(|state| { return work.join(state).join(record.id()).exists(); });
    });
}

fn stderr_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// A push the remote refused after the commit returned is shown by the next wrapper command,
/// with the branch,
/// the commit,
///  Git's own reason,
/// the fact that the commit is saved here,
/// and what to do.
#[test]
fn a_rejected_service_push_is_shown_on_the_next_command() {
    let setup: Setup = setup("push-failure-shown");
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let shown: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    let text: String = stderr_text(&shown);
    assert_eq!(shown.code, Some(0), "{text}");
    let short: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "--short", "HEAD"]).stdout).trim().to_string();
    for expected in ["branch main", short.as_str(), REFUSAL, "saved in this repository", "`git push`"] {
        assert!(text.contains(expected), "missing {expected:?} in: {text}");
    }
    remove_settled(&setup.fixture);
}

/// The failure is shown once:
/// the command after the one that showed it prints nothing about it.
#[test]
fn a_shown_failure_is_not_shown_again() {
    let setup: Setup = setup("push-failure-once");
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let first: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(first.contains(REFUSAL), "{first}");
    let second: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    assert_eq!(stderr_text(&second), "", "the same failure again");
    let third: Observed = run_wrapped(&setup.fixture, &setup.repo, &["log", "--oneline"]);
    assert_eq!(stderr_text(&third), "", "the same failure again, in another command");
    assert_eq!(waiting_records(&setup), 0);
    remove_settled(&setup.fixture);
}

/// The words the service's skipped push gives for the setting these tests make:
/// the reason names the setting and the value
///  that leaves the branch no destination.
const SKIP_REASON: &str = "`push.default` is `nothing`";

/// A push the service skips because `push.default=nothing` leaves the branch no destination is told to the person by the next
///  wrapper command,
/// with the branch,
/// the commit,
/// the reason and what to do,
/// and the remote has not received the commit.
#[test]
fn a_skipped_service_push_is_shown_on_the_next_command() {
    let setup: Setup = setup("push-skip-shown");
    git(&setup.fixture, &setup.repo, &["config", "push.default", "nothing"]);
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let shown: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    let text: String = stderr_text(&shown);
    assert_eq!(shown.code, Some(0), "{text}");
    let short: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "--short", "HEAD"]).stdout).trim().to_string();
    for expected in ["branch main", short.as_str(), SKIP_REASON, "saved in this repository", "`git push <remote> main`"] {
        assert!(text.contains(expected), "missing {expected:?} in: {text}");
    }
    let remote_tip = git(&setup.fixture, setup.remote.as_path(), &["rev-parse", "refs/heads/main"]);
    assert_ne!(String::from_utf8_lossy(&remote_tip.stdout).trim(), commit, "nothing was pushed");
    let again: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    assert_eq!(stderr_text(&again), "", "a skip is shown once");
    remove_settled(&setup.fixture);
}

/// A skipped push is not a successful one:
/// it leaves an earlier unshown failure of the branch in place,
/// and adds its own record.
#[test]
fn a_skipped_service_push_keeps_an_earlier_failure() {
    let setup: Setup = setup("push-skip-keeps-failure");
    refuse_pushes(setup.remote.as_path());
    let failed: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, failed.as_str());
    assert_eq!(waiting_records(&setup), 1, "the refusal was recorded");
    git(&setup.fixture, &setup.repo, &["config", "push.default", "nothing"]);
    // Real Git commits directly: a wrapper command would show the waiting record before the second push could run.
    std::fs::write(setup.repo.join("a.txt"), "third\n").expect("edit");
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--all", "--message=third"]);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    let skipped: String = String::from_utf8_lossy(&named.stdout).trim().to_string();
    push_through_service(&setup, skipped.as_str());
    assert_eq!(waiting_records(&setup), 2, "the refusal stays, and the skip is recorded beside it");
    let shown: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    for expected in [REFUSAL, SKIP_REASON] {
        assert!(shown.contains(expected), "missing {expected:?} in: {shown}");
    }
    remove_settled(&setup.fixture);
}

/// The push record the wrapper's commit leaves for `commit`.
fn push_item(setup: &Setup, commit: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: setup.repo.to_path_buf(),
        git_dir: common_dir(setup).to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: commit.to_string(),
        parent: String::new(),
        created: now_iso(),
        tries: 0,
    };
}

/// Wakes the fixture's service for the repository,
/// as the wrapper's commit does.
fn wake_service(setup: &Setup) {
    let config: ClientConfig = ClientConfig {
        address: Bind::Path(socket_for(setup.service.runtime.as_path(), setup.repo.join(".git").as_path())),
        activation: Activation::Managed,
        start_wait: PATIENCE,
        heartbeat_silence: PATIENCE, start_watch: None, keep_fresh: None, answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING, extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: git_policy_cli::wait_ceilings::START_CEILING, waiting_notice_after: PATIENCE,
        patience: Duration::from_secs(5),
            units_record_dir: None,
};
    wake(&config, common_dir(setup).as_path(), None, &mut std::io::sink()).expect("the service takes the wake");
}

/// Hands the service a push of `commit` as the wrapper's commit does,
/// without running a wrapper command,
/// and returns once
///  the service has settled it.
fn push_through_service(setup: &Setup, commit: &str) {
    enqueue(work_directory(common_dir(setup).as_path()).as_path(), &push_item(setup, commit)).expect("the push is recorded for the service");
    wake_service(setup);
    wait_until_settled(setup, commit);
}

/// A claim a service left when it was killed mid-push,
/// whose lease is almost over,
/// is not run when the next service wakes for
///  it,
/// because its holder might still be running;
/// nothing else wakes the service,
/// yet the push is made once the lease is
///  over:
/// the service looks again by itself.
#[test]
fn a_claim_left_by_a_killed_service_is_pushed_once_its_lease_is_over() {
    let setup: Setup = setup("push-claim-left");
    std::fs::write(setup.repo.join("a.txt"), "second\n").expect("edit");
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--all", "--message=second"]);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(&named.stdout).trim().to_string();
    let item: WorkItem = push_item(&setup, commit.as_str());
    let work: PathBuf = work_directory(common_dir(&setup).as_path());
    enqueue(work.as_path(), &item).expect("the push is recorded for the service");
    // The killed service had claimed it, and last renewed its claim a lease minus a few seconds ago.
    let last_renewal: ClaimTime = ClaimTime::read().rewound(CLAIM_LEASE - Duration::from_secs(4));
    let claimed: PathBuf = work.join("claimed").join(claim_file_name(item.id().as_str(), "0123456789abcdef0123456789abcdef", &last_renewal));
    std::fs::rename(work.join("pending").join(item.id()), claimed.as_path()).expect("claimed by the dead service");
    wake_service(&setup);
    wait_until_settled(&setup, commit.as_str());
    assert!(work.join("done").join(item.id()).exists(), "the push was made and settled as done");
    let remote_tip = git(&setup.fixture, setup.remote.as_path(), &["rev-parse", "refs/heads/main"]);
    assert_eq!(String::from_utf8_lossy(&remote_tip.stdout).trim(), commit, "the remote has the commit");
    assert_eq!(waiting_records(&setup), 0, "nothing failed");
    remove_settled(&setup.fixture);
}

/// A later push of the branch that goes through clears a failure nobody has seen yet,
/// so the person is not told about a
///  backup that has since been made.
#[test]
fn a_later_successful_push_clears_the_unshown_failure() {
    let setup: Setup = setup("push-failure-cleared");
    refuse_pushes(setup.remote.as_path());
    let failed: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, failed.as_str());
    assert_eq!(waiting_records(&setup), 1, "the refusal was recorded");
    accept_pushes(setup.remote.as_path());
    // Real Git commits directly: a wrapper command would show the waiting record before the push could clear it.
    std::fs::write(setup.repo.join("a.txt"), "third\n").expect("edit");
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--all", "--message=third"]);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    let pushed: String = String::from_utf8_lossy(&named.stdout).trim().to_string();
    push_through_service(&setup, pushed.as_str());
    assert_eq!(waiting_records(&setup), 0, "the push that went through cleared the record");
    let after: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    assert_eq!(stderr_text(&after), "");
    remove_settled(&setup.fixture);
}

/// A command whose output a program reads gets exactly Git's bytes on both streams and leaves the record for a person:
///  porcelain status,
/// null-separated status and plumbing all pass untouched,
/// and the next ordinary command still shows it.
#[test]
fn machine_read_output_is_untouched_and_keeps_the_failure_for_a_person() {
    let setup: Setup = setup("push-failure-porcelain");
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    std::fs::write(setup.repo.join("untracked.txt"), "new\n").expect("untracked");
    for arguments in [&["status", "--porcelain=v2", "--branch"][..], &["status", "-z"], &["rev-parse", "HEAD"], &["log", "-1", "--format=%H"]] {
        let wrapped: Observed = run_wrapped(&setup.fixture, &setup.repo, arguments);
        let direct: Observed = run_direct(&setup.fixture, &setup.repo, arguments);
        assert_eq!(wrapped.stdout, direct.stdout, "{arguments:?}");
        assert_eq!(stderr_text(&wrapped), stderr_text(&direct), "{arguments:?}");
        assert_eq!(waiting_records(&setup), 1, "{arguments:?} took the record meant for a person");
    }
    let human: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(human.contains(REFUSAL), "{human}");
    remove_settled(&setup.fixture);
}

/// The commit name in the notice has the length Git gives it,
/// `core.abbrev` honored,
/// and the text does not call the commit
///  the reader's own,
/// because the command that shows it may be anyone's.
#[test]
fn the_notice_names_the_commit_as_git_abbreviates_it() {
    let setup: Setup = setup("push-failure-abbrev");
    git(&setup.fixture, &setup.repo, &["config", "core.abbrev", "12"]);
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let text: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(text.contains(&format!("commit {}, so", &commit[..12])), "twelve characters, as core.abbrev asks: {text}");
    assert!(!text.contains("your commit"), "{text}");
    remove_settled(&setup.fixture);
}

/// A linked worktree on a branch of its own.
fn linked_worktree(setup: &Setup, branch: &str) -> PathBuf {
    let path: PathBuf = setup.fixture.root.join(format!("linked-{branch}"));
    git(&setup.fixture, &setup.repo, &["worktree", "add", "--quiet", "-b", branch, path.to_str().expect("UTF-8")]);
    git(&setup.fixture, &path, &["config", "maintenance.autoDetach", "false"]);
    return path;
}

/// The Git directory of a linked worktree,
/// which holds its own work records and notices.
fn linked_git_dir(setup: &Setup, linked: &Path) -> PathBuf {
    let named = git(&setup.fixture, linked, &["rev-parse", "--absolute-git-dir"]);
    return PathBuf::from(String::from_utf8_lossy(&named.stdout).trim());
}

/// A failed push is recorded under the Git directory of the worktree that made the commit and shown to the commands of
///  that worktree,
/// and not to commands of another worktree of the repository,
/// which keep their own output;
/// the text names
///  the branch.
#[test]
fn a_failed_push_is_shown_in_the_worktree_that_made_the_commit() {
    let setup: Setup = setup("push-failure-worktree");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    // The linked worktree has a service of its own.
    setup.service.serve(&setup.fixture, own.as_path());
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service_in(&setup, linked.as_path(), "second\n");
    wait_until_settled_in(own.as_path(), "refs/heads/feature", commit.as_str());
    assert_eq!(waiting_records_in(own.as_path()), 1, "the refusal was recorded in the worktree's own Git directory");
    assert_eq!(waiting_records(&setup), 0, "nothing was recorded in the main worktree");
    let elsewhere: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    assert_eq!(stderr_text(&elsewhere), "", "another worktree is not shown a commit it did not make");
    assert_eq!(waiting_records_in(own.as_path()), 1, "the other worktree left the record for its owner");
    let owner: String = stderr_text(&run_wrapped(&setup.fixture, &linked, &["status", "--short"]));
    assert!(owner.contains("branch feature") && owner.contains(REFUSAL), "{owner}");
    assert_eq!(waiting_records_in(own.as_path()), 0, "shown once");
    remove_settled(&setup.fixture);
}

/// A push stopped at its deadline in the service names a remedy that reaches the service:
/// the variable of a terminal never
///  does,
/// so the text gives the user manager's command and the restart,
/// and does not tell the person to set a variable.
#[test]
fn a_deadline_notice_names_a_remedy_that_reaches_the_service() {
    let setup: Setup = setup_with("push-failure-deadline", &[("CLI_GIT_PUSH_DEADLINE_SECONDS", "1")]);
    let slow: PathBuf = setup.fixture.root.join("slow-receive-pack");
    super::support::executable(slow.as_path(), b"#!/bin/sh\nsleep 30\nexec git-receive-pack \"$@\"\n");
    git(&setup.fixture, &setup.repo, &["config", "remote.origin.receivepack", slow.to_str().expect("UTF-8")]);
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let text: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(text.contains("made no progress for 1 seconds and was stopped"), "{text}");
    assert!(text.contains("systemctl --user set-environment CLI_GIT_PUSH_DEADLINE_SECONDS="), "{text}");
    assert!(text.contains(git_policy_cli::service_messages::RESTART_COMMAND), "{text}");
    assert!(!text.contains("set CLI_GIT_PUSH_DEADLINE_SECONDS to allow longer"), "a variable of the terminal never reaches the service: {text}");
    remove_settled(&setup.fixture);
}

/// A push work record for the branch of a linked worktree,
/// left pending under that worktree's own Git directory.
fn leave_pending_push(setup: &Setup, linked: &Path, own: &Path, branch: &str) -> String {
    let named = git(&setup.fixture, linked, &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(&named.stdout).trim().to_string();
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: linked.to_path_buf(),
        git_dir: own.to_path_buf(),
        branch_ref: format!("refs/heads/{branch}"),
        oid: commit.clone(),
        parent: String::new(),
        created: now_iso(),
        tries: 0,
    };
    enqueue(work_directory(own).as_path(), &item).expect("the push is recorded");
    return commit;
}

/// A failure nobody has seen yet survives the removal of the worktree it belongs to:
/// the next command anywhere shows it,
/// so
///  the person learns that the backup of that branch is missing.
#[test]
fn a_failed_push_of_a_branch_no_worktree_holds_is_shown_anywhere() {
    let setup: Setup = setup("push-failure-orphan");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    setup.service.serve(&setup.fixture, own.as_path());
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service_in(&setup, linked.as_path(), "second\n");
    wait_until_settled_in(own.as_path(), "refs/heads/feature", commit.as_str());
    let removed: Observed = run_wrapped(&setup.fixture, &setup.repo, &["worktree", "remove", "--force", linked.to_str().expect("UTF-8")]);
    assert_eq!(removed.code, Some(0), "{}", stderr_text(&removed));
    let shown: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(shown.contains("branch feature") && shown.contains(REFUSAL), "{shown}");
    let again: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert_eq!(again, "", "shown once");
    remove_settled(&setup.fixture);
}

/// A push the service had not run yet when its worktree was removed is not lost without a word:
/// the next command anywhere
///  names the branch and the commit.
#[test]
fn a_push_pending_when_its_worktree_is_removed_is_reported() {
    let setup: Setup = setup("push-pending-removed");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    let commit: String = leave_pending_push(&setup, linked.as_path(), own.as_path(), "feature");
    let removed: Observed = run_wrapped(&setup.fixture, &setup.repo, &["worktree", "remove", linked.to_str().expect("UTF-8")]);
    assert_eq!(removed.code, Some(0), "{}", stderr_text(&removed));
    assert!(!own.exists(), "the worktree is gone");
    let shown: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(shown.contains("branch feature") && shown.contains(&commit[..7]) && shown.contains("removed"), "{shown}");
    remove_settled(&setup.fixture);
}

/// The same for a worktree whose directory was deleted by hand and that `git worktree prune` then forgets.
#[test]
fn a_push_pending_when_its_worktree_is_pruned_is_reported() {
    let setup: Setup = setup("push-pending-pruned");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    let commit: String = leave_pending_push(&setup, linked.as_path(), own.as_path(), "feature");
    std::fs::remove_dir_all(&linked).expect("the worktree directory is deleted by hand");
    let pruned: Observed = run_wrapped(&setup.fixture, &setup.repo, &["worktree", "prune"]);
    assert_eq!(pruned.code, Some(0), "{}", stderr_text(&pruned));
    assert!(!own.exists(), "the registration is gone");
    let shown: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(shown.contains("branch feature") && shown.contains(&commit[..7]) && shown.contains("removed"), "{shown}");
    remove_settled(&setup.fixture);
}

/// A record that cannot be saved before the removal is told on standard error before Git runs,
/// and the removal still happens.
#[test]
fn a_record_that_cannot_be_saved_before_a_removal_is_told_to_the_person() {
    let setup: Setup = setup("push-pending-unsaved");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    leave_pending_push(&setup, linked.as_path(), own.as_path(), "feature");
    let common: PathBuf = setup.repo.join(".git");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli")).expect("the wrapper's directory");
    std::fs::write(common.join(SNAPSHOT_DIRECTORY), "a file where the directory belongs").expect("blocker");
    let removed: Observed = run_wrapped(&setup.fixture, &setup.repo, &["worktree", "remove", linked.to_str().expect("UTF-8")]);
    assert_eq!(removed.code, Some(0), "the removal the person asked for still happens: {}", stderr_text(&removed));
    let said: String = stderr_text(&removed);
    assert!(said.contains("could not all be saved") && said.contains(own.to_string_lossy().as_ref()), "{said}");
    remove_settled(&setup.fixture);
}

/// A record of a removed worktree that cannot be kept for the person is told on standard error by the next command,
/// and shown
///  by the one after it once the cause is gone.
#[test]
fn a_record_of_a_removed_worktree_that_cannot_be_kept_is_told_and_shown_later() {
    let setup: Setup = setup("push-pending-unkept");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    let commit: String = leave_pending_push(&setup, linked.as_path(), own.as_path(), "feature");
    let removed: Observed = run_wrapped(&setup.fixture, &setup.repo, &["worktree", "remove", linked.to_str().expect("UTF-8")]);
    assert_eq!(removed.code, Some(0), "{}", stderr_text(&removed));
    let blocker: PathBuf = setup.repo.join(".git").join(ORPHAN_NOTICE_DIRECTORY);
    std::fs::create_dir_all(blocker.parent().expect("parent")).expect("the wrapper's directory");
    std::fs::write(&blocker, "a file where the directory belongs").expect("blocker");
    let told: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(told.contains("had not finished when its worktree was removed, and noting that failed"), "{told}");
    std::fs::remove_file(&blocker).expect("the cause is fixed");
    let shown: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(shown.contains("branch feature") && shown.contains(&commit[..7]) && shown.contains("removed"), "{shown}");
    remove_settled(&setup.fixture);
}

/// A removal Git refuses leaves the worktree and its work as they were:
/// nothing is reported for a worktree that still exists.
#[test]
fn a_refused_removal_reports_nothing_and_keeps_the_pending_push() {
    let setup: Setup = setup("push-pending-kept");
    let linked: PathBuf = linked_worktree(&setup, "feature");
    let own: PathBuf = linked_git_dir(&setup, linked.as_path());
    leave_pending_push(&setup, linked.as_path(), own.as_path(), "feature");
    std::fs::write(linked.join("untracked.txt"), "not committed\n").expect("an untracked file makes the removal refuse");
    let refused: Observed = run_wrapped(&setup.fixture, &setup.repo, &["worktree", "remove", linked.to_str().expect("UTF-8")]);
    assert_ne!(refused.code, Some(0), "Git refuses to remove a worktree with untracked files");
    let shown: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert_eq!(shown, "", "the worktree still exists, so nothing was lost");
    assert!(own.join("monochromatic-git-policy-cli/work/pending").read_dir().expect("the work directory").count() >= 1, "the pending push is still queued");
    remove_settled(&setup.fixture);
}
