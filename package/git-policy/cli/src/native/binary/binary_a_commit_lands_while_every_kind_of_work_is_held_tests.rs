//! What:
//!  A person's commit made while the commit service is inside every other kind of work it does at once:
//! two prewarm clones of new worktrees' Cargo build directories,
//! the background sync of a landed batch's records,
//! the removal of a landed request's scratch tree,
//! and the push of the commit before it.
//! Why:
//!  The owner's rules are "Commits should never fail" and "Agents should never wait on each other on commits or merges",
//! and the service that answers a commit must do so whatever work nobody waits for is running beside it
//! (`doc/handover/cli-git-rust-implementation.md`).
//! On `2026-11-12` the main checkout's service spent thirteen minutes on one commit while a housekeeping round,
//! four new worktrees' repairs, a push and a sync ran
//! (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
//! The holds are states and not durations:
//! each held step makes a file and goes on when the control makes another
//! (`service_hold_a_step_for_a_control.rs`),
//! so "while every kind of work is held" is what the control asserts about,
//! and no arm of this measurement loads the machine to get it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await holdEveryKindOfWork(); const t0 = now(); await commit('two'); expect(now() - t0).toBeLessThan(BOUND)
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use git_policy_cli::service_hold_a_step_for_a_control::{HOLD_DIRECTORY_VARIABLE, HOLD_VARIABLE, INSIDE_A_CLONE_STEP, INSIDE_A_PUSH, INSIDE_A_SCRATCH_REMOVAL, INSIDE_A_SYNC, let_go_file, reached_file};
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{WorkItem, WorkKind, enqueue};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};

/// The longest this measurement waits for a state it cannot time.
const PATIENCE: Duration = Duration::from_secs(180);

/// How long a person's commit may take while every other kind of work of the service is held open:
/// twenty seconds,
/// which is four of the client's five-second silence windows (`HEARTBEAT_SILENCE`) and far under the twenty minutes a
///  commit is allowed.
/// A wait past this one is the defect this measurement exists for:
/// work nobody waits for stood between the person and their commit.
const A_WAIT_NO_OTHER_WORK_MAY_CAUSE: Duration = Duration::from_secs(20);

/// How many prewarm records the queue holds,
/// which is how many clones of a build directory the lane starts side by side:
/// two,
/// so their steps alone ask for more places than one process keeps free of slow work,
/// which is the state that starved the commit of `2026-11-12`.
const THE_QUEUE: usize = 2;

/// How many entries the build directory each prewarm clones holds:
/// enough that one clone is a real walk of a directory and not a single step.
const THE_CLONE_ENTRIES: usize = 400;

/// The receipt a prewarm's clone of a build directory needs to find the tools it cleans the copy with.
const THE_CLONED_WORKSPACES_OWN_CRATE: &str = "monochromatic-prewarm-held-measurement";

/// The reference a prewarm record carries, which names no branch (`package_repair_prewarm::PREWARM_REFERENCE`).
const PREWARM_REFERENCE: &str = "refs/monochromatic/cargo-prewarm";

/// The reference a trash record carries; nothing is emptied here, so no trash is planted.
const _TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// A repository with a bare remote, a service the commits go through, and the directory the holds go in.
struct Setup {
    /// The disposable fixture, removed at the end.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The service, stopped with this value.
    service: FixtureService,
    /// Where the held steps say they were reached and are let go.
    holds: PathBuf,
}

impl Setup {
    /// The repository's own Git directory.
    fn git_dir(&self) -> PathBuf {
        return self.repo.join(".git");
    }

    /// The work directory the service drains.
    fn work(&self) -> PathBuf {
        return work_directory(self.git_dir().as_path());
    }

    /// Whether the service has reached the step `point`.
    fn reached(&self, point: &str) -> bool {
        return reached_file(self.holds.as_path(), point).exists();
    }

    /// Let the step `point` go on.
    fn let_go(&self, point: &str) {
        std::fs::write(let_go_file(self.holds.as_path(), point), b"").expect("the file that lets a held step go");
    }
}

/// What:
///  A repository tracking a bare remote that has its branch,
///  with a service of its own that holds every kind of work this measurement names.
/// Why:
///   The holds are named before the first command starts the service,
/// because the service reads them once for its own life.
fn setup(name: &str) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    // The upkeep of a measurement is not what it measures, so it runs in front of the command rather than behind it.
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    the_clone_keeps_its_tools(&fixture);
    let holds: PathBuf = fixture.root.join("holds");
    std::fs::create_dir_all(holds.as_path()).expect("the directory the holds go in");
    name_the_holds(&fixture, holds.as_path());
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service, holds };
}

/// Name the steps this measurement holds in the fixture service's environment.
fn name_the_holds(fixture: &Fixture, holds: &Path) {
    let named: Vec<&str> = vec![INSIDE_A_CLONE_STEP, INSIDE_A_SYNC, INSIDE_A_SCRATCH_REMOVAL, INSIDE_A_PUSH];
    let mut environment = fixture.service_environment.lock().expect("the fixture's service environment");
    environment.push((OsString::from(HOLD_VARIABLE), OsString::from(named.join(","))));
    environment.push((OsString::from(HOLD_DIRECTORY_VARIABLE), holds.as_os_str().to_os_string()));
}

/// What:
///  Give the fixture's service the tools the prewarm's clone needs,
///  so the clone it runs is the shipped one and not a refusal before it.
/// Why:
///   The service runs with the fixture's own `PATH` (`<fixture>/bin:/usr/bin:/bin`) and its own `HOME`,
/// and a prewarm refuses before it clones when no `cargo` can be found to clean the copy with.
fn the_clone_keeps_its_tools(fixture: &Fixture) {
    let mut environment = fixture.service_environment.lock().expect("the fixture's service environment");
    for name in ["CARGO_HOME", "RUSTUP_HOME"] {
        if let Some(value) = std::env::var_os(name) {
            environment.push((OsString::from(name), value));
        }
    }
    drop(environment);
    let cargo: PathBuf = std::env::var_os("CARGO").map_or_else(
        || {
            return PathBuf::from("cargo");
        },
        PathBuf::from,
    );
    let toolchain: PathBuf = cargo.parent().map_or_else(
        || {
            return PathBuf::from("/usr/bin");
        },
        PathBuf::from,
    );
    for tool in ["cargo", "rustc"] {
        let link: PathBuf = fixture.root.join("bin").join(tool);
        let _removed = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(toolchain.join(tool), link.as_path()).expect("the fixture's own link to the toolchain");
    }
}

/// What:
///  Plant the build directory the queue clones, and the prewarm records that name it.
/// Why:
///   These are the records `git worktree add` writes for a new worktree's Cargo build directory
/// (`package_repair_prewarm.rs`),
/// and the clone of one is the step of a prewarm that holds a place of the process's pool.
fn plant_the_queue(setup: &Setup) {
    let source: PathBuf = setup.fixture.root.join("prewarm-source");
    std::fs::create_dir_all(source.as_path()).expect("the source build directory");
    for entry in 0..THE_CLONE_ENTRIES {
        std::fs::write(source.join(format!("dependency-{entry}")), b"a build's own state\n").expect("a source entry");
    }
    for at in 0..THE_QUEUE {
        let destination: PathBuf = setup.fixture.root.join(format!("prewarm-build-{at}"));
        // The payload `package_repair_prewarm::encode` writes, built without the `json!` macro, which unwraps inside.
        let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        fields.insert(String::from("root"), serde_json::Value::from(setup.repo.display().to_string()));
        fields.insert(String::from("from"), serde_json::Value::from(source.display().to_string()));
        fields.insert(String::from("to"), serde_json::Value::from(destination.display().to_string()));
        fields.insert(String::from("members"), serde_json::Value::from(vec![String::from(THE_CLONED_WORKSPACES_OWN_CRATE)]));
        let payload: serde_json::Value = serde_json::Value::Array(vec![serde_json::Value::Object(fields)]);
        let item: WorkItem = WorkItem {
            kind: WorkKind::Prewarm,
            repository: setup.repo.clone(),
            git_dir: setup.git_dir(),
            branch_ref: String::from(PREWARM_REFERENCE),
            oid: format!("{at:040x}"),
            parent: payload.to_string(),
            created: String::from("2026-11-13T00:00:00.000Z"),
            tries: 0,
        };
        enqueue(setup.work().as_path(), &item).expect("the prewarm record is queued");
    }
}

/// Commit `content` through the service and answer how long the command took.
fn commit_and_time(setup: &Setup, content: &str) -> (Output, Duration) {
    std::fs::write(setup.repo.join("a.txt"), content).expect("the edit");
    let mut command: std::process::Command = wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let began: Instant = Instant::now();
    let output: Output = command.output().expect("the wrapped commit runs");
    return (output, began.elapsed());
}

/// The commit the repository's branch names now.
fn head(setup: &Setup) -> String {
    let named: Output = super::support::direct(&setup.fixture).current_dir(setup.repo.as_path()).args(["rev-parse", "HEAD"]).output().expect("real Git answers");
    return String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
}

/// Whether the bare remote holds `commit`.
fn remote_has(setup: &Setup, commit: &str) -> bool {
    let asked: Output = super::support::direct(&setup.fixture)
        .current_dir(setup.remote.as_path())
        .args(["cat-file", "-e", format!("{commit}^{{commit}}").as_str()])
        .output()
        .expect("real Git answers");
    return asked.status.success();
}

/// Wait, bounded, until `condition` holds.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened within {PATIENCE:?}: {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// What:
///  Every kind of work the service does beside a landing is held open,
///  and a commit made meanwhile lands within the bound,
///  and its push reaches the remote once the held push is let go.
/// Why:
///   The table of "which kind of work delays a commit, and by how much" is this measurement with one row per arm;
/// here every row is held at once,
/// which is the state the service of `2026-11-12` was in.
/// What it asserts is what every run must give:
/// the commit lands and the command succeeds.
#[test]
fn a_commit_lands_while_every_kind_of_work_is_held() {
    let setup: Setup = setup("a-commit-lands-while-work-is-held");
    plant_the_queue(&setup);
    // The first commit wakes the service: its round claims the push, and the lane claims the queue.
    let (first, _first_ms): (Output, Duration) = commit_and_time(&setup, "one\n");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(first.stderr.as_slice()));
    // The held steps are reached one after another, so each is waited for by name.
    wait_until("a prewarm's clone was reached", &|| { return setup.reached(INSIDE_A_CLONE_STEP); });
    wait_until("the background sync of the landing was reached", &|| { return setup.reached(INSIDE_A_SYNC); });
    wait_until("the removal of the landing's scratch tree was reached", &|| { return setup.reached(INSIDE_A_SCRATCH_REMOVAL); });
    wait_until("the push of the first commit was reached", &|| { return setup.reached(INSIDE_A_PUSH); });
    // The measured commit: every held step is still running, and nothing else asks this service for anything.
    let (second, commit_ms): (Output, Duration) = commit_and_time(&setup, "two\n");
    let landed: String = head(&setup);
    eprintln!("MEASURE held_every_kind commit_ms={} landed={landed}", commit_ms.as_millis());
    assert!(second.status.success(), "{}", String::from_utf8_lossy(second.stderr.as_slice()));
    assert!(commit_ms < A_WAIT_NO_OTHER_WORK_MAY_CAUSE, "the commit waited {commit_ms:?} behind work nobody waits for");
    // The work is let go, and the push that was held reaches the remote.
    for point in [INSIDE_A_CLONE_STEP, INSIDE_A_SYNC, INSIDE_A_SCRATCH_REMOVAL, INSIDE_A_PUSH] {
        setup.let_go(point);
    }
    wait_until("the remote holds the measured commit", &|| { return remote_has(&setup, landed.as_str()); });
    remove_settled(&setup.fixture);
}

/// How many rounds each arm of the measurement is made,
/// so the range of a figure is a range of the same arm and not of the machine's minutes.
const ARMS: usize = 3;

/// What:
///  One commit's time against real Git's for the same commit,
///  with the service idle and with every kind of work it does held open.
/// Why:
///   The owner's rule is that what the wrapper adds on top of real Git's own time for the same commit is at most the larger
/// of five seconds and twice real Git's time
/// (`doc/handover/cli-git-rust-implementation.md`, the section of `2026-11-12T14:03:46-05:00`),
/// and the measurement is the only evidence for it.
/// The same round measures real Git on the same repository, so the disk and the host are the control arm's
/// (`AGENTS.md`, rule `TVC`):
/// an absolute figure here is context and not a finding.
/// An arm is repeated `ARMS` times and the range is reported, because a difference smaller than the run-to-run spread is
/// noise (rule `NQB`).
/// Run alone, because it wants the host's and the fixture's disk to itself:
/// `mise run //package/git-policy/cli:native:test:binary:host -- --run-ignored only --no-capture -E 'test(/a_commit_timed/)'`
#[test]
#[ignore = "a measurement for the record, not a control; run it alone"]
fn a_commit_timed_against_real_git_with_every_kind_of_work_held() {
    let setup: Setup = setup("a-commit-timed-against-real-git");
    let mut idle: Vec<u128> = Vec::new();
    let mut real: Vec<u128> = Vec::new();
    for at in 0..ARMS {
        // Real Git on the same repository, which is the arm the rule compares with.
        std::fs::write(setup.repo.join("a.txt"), format!("real {at}\n")).expect("the edit");
        let began: Instant = Instant::now();
        super::support::direct(&setup.fixture).current_dir(setup.repo.as_path()).args(["commit", "--quiet", "-m", "real", "--", "a.txt"]).output().expect("real Git commits");
        real.push(began.elapsed().as_millis());
        // The wrapped commit through the service, with nothing else asked of it.
        let (_output, took): (Output, Duration) = commit_and_time(&setup, format!("wrapped {at}\n").as_str());
        idle.push(took.as_millis());
    }
    // The queue, held, and the arm that measures what it costs a person.
    plant_the_queue(&setup);
    let (_waking, _waking_ms): (Output, Duration) = commit_and_time(&setup, "waking\n");
    wait_until("a prewarm's clone was reached", &|| { return setup.reached(INSIDE_A_CLONE_STEP); });
    wait_until("the background sync of the landing was reached", &|| { return setup.reached(INSIDE_A_SYNC); });
    wait_until("the removal of the landing's scratch tree was reached", &|| { return setup.reached(INSIDE_A_SCRATCH_REMOVAL); });
    wait_until("the push of the waking commit was reached", &|| { return setup.reached(INSIDE_A_PUSH); });
    let mut held: Vec<u128> = Vec::new();
    for at in 0..ARMS {
        let (_output, took): (Output, Duration) = commit_and_time(&setup, format!("held {at}\n").as_str());
        held.push(took.as_millis());
    }
    eprintln!("MEASURE real_git_ms={real:?} idle_ms={idle:?} held_ms={held:?}");
    for point in [INSIDE_A_CLONE_STEP, INSIDE_A_SYNC, INSIDE_A_SCRATCH_REMOVAL, INSIDE_A_PUSH] {
        setup.let_go(point);
    }
    remove_settled(&setup.fixture);
}

/// What:
///  One commit's time against real Git's for the same commit with nothing else asked of the service.
/// Why:
///   The control arm of the measurement above:
/// the rule compares what the wrapper adds with real Git's own time for the same commit,
/// so the same round measures both,
/// and the difference between two trees of the wrapper is only a finding when it is larger than the spread of one arm.
/// Run alone:
/// `mise run //package/git-policy/cli:native:test:binary:host -- --run-ignored only --no-capture -E 'test(/a_commit_timed_while_the_service_is_idle/)'`
#[test]
#[ignore = "a measurement for the record, not a control; run it alone"]
fn a_commit_timed_while_the_service_is_idle() {
    let setup: Setup = setup("a-commit-timed-while-idle");
    let mut idle: Vec<u128> = Vec::new();
    let mut real: Vec<u128> = Vec::new();
    for at in 0..ARMS {
        std::fs::write(setup.repo.join("a.txt"), format!("real {at}\n")).expect("the edit");
        let began: Instant = Instant::now();
        super::support::direct(&setup.fixture).current_dir(setup.repo.as_path()).args(["commit", "--quiet", "-m", "real", "--", "a.txt"]).output().expect("real Git commits");
        real.push(began.elapsed().as_millis());
        let (_output, took): (Output, Duration) = commit_and_time(&setup, format!("wrapped {at}\n").as_str());
        idle.push(took.as_millis());
    }
    eprintln!("MEASURE idle_only real_git_ms={real:?} idle_ms={idle:?}");
    remove_settled(&setup.fixture);
}
