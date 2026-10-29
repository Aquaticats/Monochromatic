//! What:
//!  A landing in a child process that a phase marker kills or pauses,
//!  and the repository the native wrapper leaves after it.
//! Why:
//!  A landing that dies anywhere inside its critical section must leave a repository that the next
//!  wrapped command recovers to a sane state:
//!  the branch, the real index, the reflogs, the locks and the kept packs all agree
//!  (`src/policy-engine/commit-transaction-test-phase.ts`, `commit-landing.ts`).
//!  A paused child lets a control change the repository between two steps of a landing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! env.CLI_GIT_TEST_ONLY_PHASE_SIGNAL = 'ref-updated:kill:' + markers;
//! await spawnLandingChild(env); await run(wrappedGit, ['status']);
//! ```

/// The world the child lands in, and the shared fixtures.
use super::landing_support::{Opened, WORLD_VARIABLE, World, git_text, land};
use super::support::{Fixture, TRANSACTION_ID, bounded_for, fixture, git, observe, remove, wrapper_first_path};
use git_policy_cli::transaction_journal::Conclusion;
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Environment variable asking the child to land an amend.
const CONCLUSION_VARIABLE: &str = "CLI_GIT_LANDING_CONCLUSION";

/// How long a control waits for a child or a marker before it gives up.
const PATIENCE: Duration = Duration::from_secs(60);

/// The child process of the controls below:
///  it opens the world's transaction as its own and lands the prepared commit.
/// The marker variable, set by the parent, kills or pauses it at one phase.
#[test]
#[ignore = "child process of the landing crash controls"]
fn child_lands() {
    let root: PathBuf = PathBuf::from(std::env::var_os(WORLD_VARIABLE).expect("the world to land in"));
    let world: World = World::attach(root.as_path());
    let mut opened: Opened = world.open();
    // A control that needs an amend asks for it by name; the capture's conclusion is the only difference.
    if std::env::var_os(CONCLUSION_VARIABLE).is_some_and(|value| { return value == "amend"; }) {
        opened.capture.layout.conclusion = Conclusion::Amend;
    }
    let result = land(&world, &mut opened, &world.commit_payload());
    let line: String = match &result {
        Ok(outcome) => format!("ok {outcome:?}"),
        Err(error) => format!("error {error}"),
    };
    std::fs::write(root.join("child-result.txt"), format!("{line}\npreserved={}\n", opened.workspace.preserved))
        .expect("the child's result");
}

/// Start the child with a marker armed.
fn spawn_child(world: &World, signal: &str) -> Child {
    return spawn_child_with(world, signal, "none");
}

/// Start the child with a marker armed and a conclusion kind for its capture.
fn spawn_child_with(world: &World, signal: &str, conclusion: &str) -> Child {
    let log = std::fs::File::create(world.root.join("child.log")).expect("child log");
    return Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "landing_crash::child_lands", "--ignored", "--nocapture"])
        .env(WORLD_VARIABLE, &world.root)
        .env(CONCLUSION_VARIABLE, conclusion)
        .env(PHASE_SIGNAL_VARIABLE, signal)
        .stdin(Stdio::null())
        .stdout(log.try_clone().expect("log handle"))
        .stderr(log)
        .spawn()
        .expect("the child starts");
}

/// Wait for the child to end, killing it when it outlasts the patience.
fn finish(mut child: Child) -> ExitStatus {
    let deadline: Instant = Instant::now() + PATIENCE;
    loop {
        if let Some(status) = child.try_wait().expect("the child can be polled") {
            return status;
        }
        if Instant::now() > deadline {
            child.kill().expect("the stuck child is killed");
            panic!("the landing child outlasted {PATIENCE:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Wait for a marker file the child writes when it reaches a phase.
/// A child that ends without writing it never will, so that ends the wait at once instead of after the patience.
fn await_marker(path: &Path, child: &mut Child) {
    let deadline: Instant = Instant::now() + PATIENCE;
    while !path.exists() {
        if let Some(status) = child.try_wait().expect("the child can be polled") {
            assert!(path.exists(), "the child ended with {status} before {} appeared; see child.log", path.display());
        }
        assert!(Instant::now() < deadline, "{} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The marker directory of a world.
fn markers(world: &World) -> PathBuf {
    let directory: PathBuf = world.root.join("markers");
    std::fs::create_dir(&directory).expect("marker directory");
    return directory;
}

/// How many reflog subjects of a ref carry this transaction's nonce.
fn nonce_count(fixture: &Fixture, world: &World, reference: &str) -> usize {
    let reflog: String = git_text(fixture, world.repo.as_path(), &["reflog", "show", "--format=%gs", reference]);
    return reflog.matches(format!("commit (cli-git {TRANSACTION_ID}):").as_str()).count();
}

/// Kill the child at a phase, run the native wrapper's `git status`, and check the repository it leaves.
fn crash_at(phase: &str, landed: bool) {
    let fixture: Fixture = fixture(format!("landing-crash-{phase}").as_str());
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("{phase}:kill:{}", marker_directory.display());
    let status: ExitStatus = finish(spawn_child(&world, signal.as_str()));
    assert_eq!(status.signal(), Some(9), "the child died of the kill marker, not of a failure; see child.log");
    assert!(marker_directory.join(format!("{phase}.reached")).exists());
    assert!(world.directory.exists(), "the dead landing left its transaction directory");
    // Recovery runs real Git several times before the status, so the bound is wider than the 5 seconds
    // that bound a single forwarded command; a heavily loaded host otherwise ends the wrapper mid-recovery.
    let mut status_command: Command = bounded_for(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(&fixture).as_os_str(),
        "120",
    );
    status_command.current_dir(world.repo.as_path()).args(["status", "--porcelain=v1"]);
    let observed = observe(&mut status_command, b"");
    assert_eq!(observed.code, Some(0), "{}", String::from_utf8_lossy(&observed.stderr));
    let expected_branch: &str = if landed { world.prepared.as_str() } else { world.base.as_str() };
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), expected_branch);
    let expected_tree: &str = if landed { world.prepared_tree.as_str() } else { world.base_tree.as_str() };
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["write-tree"]), expected_tree);
    let expected_status: &str = if landed { "" } else { " M a.txt\n" };
    assert_eq!(String::from_utf8_lossy(&observed.stdout), expected_status);
    let expected_nonces: usize = usize::from(landed);
    assert_eq!(nonce_count(&fixture, &world, "refs/heads/main"), expected_nonces);
    assert_eq!(nonce_count(&fixture, &world, "HEAD"), expected_nonces);
    assert!(!world.index_lock().exists(), "recovery released or consumed the lock");
    assert!(!world.pid_file().exists(), "recovery removed the dead owner's PID file");
    assert!(world.keeps().is_empty(), "recovery removed the transaction's keep: {:?}", world.keeps());
    assert!(!world.directory.exists(), "recovery removed the transaction directory");
    assert!(!world.shadow.exists(), "recovery removed the shadow repository");
    git(&fixture, world.repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove(&fixture);
}

/// A kill with the locks held and nothing journaled beyond the lock record:
///  recovery discards the attempt and releases the lock.
#[test]
fn a_kill_at_landing_locked_is_discarded() {
    crash_at("landing-locked", false);
}

/// A kill after the objects migrated into the kept pack:
///  recovery discards the attempt, removes the keep, and the repository still passes `git fsck`.
#[test]
fn a_kill_at_objects_migrated_is_discarded() {
    crash_at("objects-migrated", false);
}

/// A kill after the branch advanced:
///  recovery completes the landing, installing the recorded index through the dead owner's lock.
#[test]
fn a_kill_at_ref_updated_is_completed() {
    crash_at("ref-updated", true);
}

/// A kill after the index was installed:
///  recovery recognizes the installed index and finishes the cleanup.
#[test]
fn a_kill_at_index_installed_is_completed() {
    crash_at("index-installed", true);
}

/// A target that moves while the landing is paused after its migration loses the compare-and-swap:
///  nothing lands, the keep is removed, the lock is released, the directory is no longer preserved,
///  and the nonce is in no reflog.
#[test]
fn a_target_moved_during_the_landing_loses_the_compare_and_swap() {
    let fixture: Fixture = fixture("landing-swap-lost");
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("objects-migrated:pause:{}", marker_directory.display());
    let mut child: Child = spawn_child(&world, signal.as_str());
    await_marker(marker_directory.join("objects-migrated.reached").as_path(), &mut child);
    let winner: String = git_text(
        &fixture,
        world.repo.as_path(),
        &["commit-tree", world.base_tree.as_str(), "-p", world.base.as_str(), "-m", "winner"],
    );
    git(&fixture, world.repo.as_path(), &["update-ref", "refs/heads/main", winner.as_str(), world.base.as_str()]);
    std::fs::write(marker_directory.join("objects-migrated.release"), b"").expect("release");
    let status: ExitStatus = finish(child);
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert_eq!(
        result,
        format!("ok HeadMoved {{ current: Commit(\"{winner}\"), replayable: true }}\npreserved=false\n")
    );
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), winner);
    assert_eq!(nonce_count(&fixture, &world, "refs/heads/main"), 0);
    assert_eq!(nonce_count(&fixture, &world, "HEAD"), 0);
    assert!(world.keeps().is_empty(), "the lost swap removed the keep: {:?}", world.keeps());
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert!(!world.directory.join("ref-updated.json").exists());
    assert!(world.directory.join("landing-1.json").exists(), "the record stays until the caller removes the directory");
    remove(&fixture);
}

/// A landing that fails after its objects migrated but before its landing record is written removes the pack keep itself:
///  no record hands the directory to recovery yet, and the caller removes the directory,
///  so nothing else would ever remove the keep and the pack would stay pinned for good.
#[test]
fn a_failure_before_the_landing_record_removes_the_keep() {
    let fixture: Fixture = fixture("landing-fails-before-record");
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("objects-migrated:pause:{}", marker_directory.display());
    let mut child: Child = spawn_child(&world, signal.as_str());
    await_marker(marker_directory.join("objects-migrated.reached").as_path(), &mut child);
    // A directory where the pre-landing index is copied makes that copy, the first step after the migration, fail.
    std::fs::create_dir(world.directory.join("pre-landing-1.index")).expect("block the copy");
    std::fs::write(marker_directory.join("objects-migrated.release"), b"").expect("release");
    let status: ExitStatus = finish(child);
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert!(result.starts_with("error "), "{result}");
    assert!(result.ends_with("\npreserved=false\n"), "{result}");
    assert!(world.keeps().is_empty(), "the failed landing removed its keep: {:?}", world.keeps());
    assert!(!world.index_lock().exists(), "the real index lock was released");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    assert!(!world.directory.join("landing-1.json").exists(), "no landing record was written");
    remove(&fixture);
}

/// An install that fails after the branch advanced leaves the real index lock in place for recovery,
///  and the directory preserved.
#[test]
fn an_install_failing_after_the_branch_advanced_leaves_the_lock_for_recovery() {
    let fixture: Fixture = fixture("landing-install-fails");
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("ref-updated:pause:{}", marker_directory.display());
    let mut child: Child = spawn_child(&world, signal.as_str());
    await_marker(marker_directory.join("ref-updated.reached").as_path(), &mut child);
    std::fs::remove_file(world.directory.join("post-1.index")).expect("remove the recorded post-index");
    std::fs::write(marker_directory.join("ref-updated.release"), b"").expect("release");
    let status: ExitStatus = finish(child);
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert!(result.starts_with("error "), "{result}");
    assert!(result.ends_with("\npreserved=true\n"), "{result}");
    assert!(world.index_lock().exists(), "recovery installs through this lock");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.prepared);
    assert_eq!(nonce_count(&fixture, &world, "refs/heads/main"), 1);
    assert!(world.directory.join("ref-updated.json").exists());
    assert!(world.keeps().is_empty(), "the keep was removed once the branch advanced");
    remove(&fixture);
}

/// A malformed marker variable fails the landing at the first phase it reaches,
///  and the lock the landing already took is released.
#[test]
fn a_malformed_marker_fails_the_landing_and_releases_the_lock() {
    let fixture: Fixture = fixture("landing-bad-marker");
    let world: World = World::build(&fixture);
    let status: ExitStatus = finish(spawn_child(&world, "not-a-marker"));
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert!(result.starts_with("error "), "{result}");
    assert!(result.contains(PHASE_SIGNAL_VARIABLE), "{result}");
    assert!(result.ends_with("\npreserved=false\n"), "{result}");
    assert!(!world.index_lock().exists());
    assert!(!world.pid_file().exists());
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.base);
    remove(&fixture);
}

/// An amend that loses the compare-and-swap is not replayable, even though the winner names a commit.
#[test]
fn an_amend_that_loses_the_compare_and_swap_is_not_replayable() {
    let fixture: Fixture = fixture("landing-swap-lost-amend");
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("objects-migrated:pause:{}", marker_directory.display());
    let mut child: Child = spawn_child_with(&world, signal.as_str(), "amend");
    await_marker(marker_directory.join("objects-migrated.reached").as_path(), &mut child);
    let winner: String = git_text(
        &fixture,
        world.repo.as_path(),
        &["commit-tree", world.base_tree.as_str(), "-p", world.base.as_str(), "-m", "winner"],
    );
    git(&fixture, world.repo.as_path(), &["update-ref", "refs/heads/main", winner.as_str(), world.base.as_str()]);
    std::fs::write(marker_directory.join("objects-migrated.release"), b"").expect("release");
    let status: ExitStatus = finish(child);
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert_eq!(
        result,
        format!("ok HeadMoved {{ current: Commit(\"{winner}\"), replayable: false }}\npreserved=false\n")
    );
    remove(&fixture);
}

/// A completion step that fails after the branch advanced says that the commit landed, so a consumer that reads the
///  failure never retries work that is already committed (issue 1781).
#[test]
fn a_completion_step_failing_after_the_branch_advanced_says_the_commit_landed() {
    let fixture: Fixture = fixture("landing-completion-fails");
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("ref-updated:pause:{}", marker_directory.display());
    let mut child: Child = spawn_child(&world, signal.as_str());
    await_marker(marker_directory.join("ref-updated.reached").as_path(), &mut child);
    // The name of the completion marker is taken by a directory, so writing the marker fails after the index installed.
    std::fs::create_dir(world.directory.join("index-installed")).expect("occupy the completion marker's name");
    std::fs::write(marker_directory.join("ref-updated.release"), b"").expect("release");
    let status: ExitStatus = finish(child);
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert!(result.starts_with("error "), "{result}");
    assert!(result.contains(world.prepared.as_str()), "the failure names the landed commit: {result}");
    assert!(result.contains("landed"), "the failure says the commit landed: {result}");
    assert!(result.contains("do not retry"), "the failure says not to retry: {result}");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.prepared);
    remove(&fixture);
}

/// The options that start a command in new user and PID namespaces, with its own view of the process table.
const NEW_PID_NAMESPACE: [&str; 5] = ["--user", "--map-root-user", "--pid", "--fork", "--mount-proc"];

/// Whether this system lets an unprivileged process start a command in new user and PID namespaces.
fn new_pid_namespaces_are_allowed() -> bool {
    return Command::new("unshare")
        .args(NEW_PID_NAMESPACE)
        .arg("/usr/bin/true")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| { return status.success(); });
}

/// A wrapper in another PID namespace cannot interpret a live owner's PID, so it leaves the landing alone (issues
/// 1781 and 1784); before the owner records named their namespace it judged the owner dead and recovered the
/// transaction under the landing, and the landing then failed after its branch advanced.
#[test]
fn a_wrapper_in_another_pid_namespace_leaves_a_live_landing_alone() {
    if !new_pid_namespaces_are_allowed() {
        eprintln!("skipped: this system does not allow a new user and PID namespace");
        return;
    }
    let fixture: Fixture = fixture("landing-foreign-namespace");
    let world: World = World::build(&fixture);
    let marker_directory: PathBuf = markers(&world);
    let signal: String = format!("ref-updated:pause:{}", marker_directory.display());
    let mut child: Child = spawn_child(&world, signal.as_str());
    await_marker(marker_directory.join("ref-updated.reached").as_path(), &mut child);
    // The bound wraps the namespace change, not the other way round: `timeout` fails inside a user namespace.
    let mut foreign: Command = bounded_for(
        &fixture,
        Path::new("/usr/bin/unshare"),
        wrapper_first_path(&fixture).as_os_str(),
        "120",
    );
    foreign
        .args(NEW_PID_NAMESPACE)
        .arg(fixture.root.join("bin/git"))
        .current_dir(world.repo.as_path())
        .args(["status", "--porcelain=v1"]);
    let observed = observe(&mut foreign, b"");
    assert_eq!(observed.code, Some(0), "{}", String::from_utf8_lossy(&observed.stderr));
    assert!(world.directory.exists(), "the wrapper of another namespace recovered a live transaction");
    std::fs::write(marker_directory.join("ref-updated.release"), b"").expect("release");
    let status: ExitStatus = finish(child);
    assert!(status.success(), "the child failed; see child.log");
    let result: String = std::fs::read_to_string(world.root.join("child-result.txt")).expect("the child's result");
    assert!(result.starts_with("ok "), "the landing must complete: {result}");
    assert_eq!(git_text(&fixture, world.repo.as_path(), &["rev-parse", "refs/heads/main"]), world.prepared);
    remove(&fixture);
}
