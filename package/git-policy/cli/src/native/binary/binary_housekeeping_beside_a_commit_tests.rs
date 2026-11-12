//! What:
//!  A person's commit and its push, made while the service is inside a batch of housekeeping records it took from this
//!  repository's own queue:
//! the commit's own wall time and the time until the remote holds it.
//! Why:
//!  The rule is that a person's work never waits behind housekeeping,
//! and the audit of `0.6.1-alpha.12` found a wave of prewarm clones able to hold every place of the process's bound
//! (finding 3 of `doc/handover/`, the places kept free of slow work).
//! This is a measurement for the record and not a control:
//! it asserts only what every arm must give,
//! that the commit landed,
//! that its push reached the remote,
//! and that neither waited longer than a bound no queue of housekeeping may push a person's work past.
//! Run it alone, because it creates its own queue and wants the host to itself:
//! `mise run //package/git-policy/cli:native:test:binary:host -- --run-ignored only housekeeping_beside_a_commit --no-capture`
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! queue(PREWARMS); await commit(one); await insideAClone(); const t0 = now(); await commit(two); print(`commit_ms=${now()-t0} push_ms=${await pushed()}`);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, fixture, git, repository, start_fixture_service, FixtureService};
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{WorkItem, WorkKind, enqueue};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Output;
use std::time::{Duration, Instant};

/// The longest this measurement waits for anything it cannot time.
const PATIENCE: Duration = Duration::from_secs(180);

/// How long a person's commit and its push may wait while the queue of housekeeping runs:
/// twenty seconds,
/// which is four of the client's five-second windows (`HEARTBEAT_SILENCE`) and shorter than the queue this measurement
///  plants.
/// A wait past this one is the defect the measurement exists for:
/// the queue stood between the person and their work.
const A_WAIT_NO_QUEUE_MAY_CAUSE: Duration = Duration::from_secs(20);

/// How many prewarm records the queue holds.
/// Why:
///   Four of them run at once (the width of a wave of slow work) and the service's pool has four places,
/// so this is five waves:
/// the shape of a wave of prewarm clones,
/// sized to be inside the queue while the commit is made and no larger.
const THE_QUEUE: usize = 20;

/// How many entries the build directory each prewarm clones holds.
/// Why:
///   The clone is the slow step of a prewarm and the one this measurement is about;
/// this many entries makes one clone longer than the moment between two commits,
/// which is what the measurement needs to be inside,
/// and is small enough to plant in a moment.
const THE_CLONE_ENTRIES: usize = 2_000;

/// Why:
///   A prewarm record names the members its clean removes from the copy,
/// and a payload naming none is refused before anything is cloned (`package_repair_prewarm::decode`).
const THE_CLONED_WORKSPACES_OWN_CRATE: &str = "monochromatic-prewarm-measurement";

/// The reference a prewarm record carries, which names no branch (`package_repair_prewarm::PREWARM_REFERENCE`).
const PREWARM_REFERENCE: &str = "refs/monochromatic/cargo-prewarm";

/// A repository with a bare remote, a service the commits go through, and the queue's work directory.
struct Setup {
    /// The disposable fixture, removed at the end.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The service, stopped with this value.
    service: FixtureService,
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
}

/// A repository tracking a bare remote that has its branch,
/// with a service of its own.
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
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service };
}

/// What:
///  Give the fixture's service the tools the prewarm's clone needs,
///  so the clone it runs is the shipped one and not a refusal before it.
/// Why:
///   The service runs with the fixture's own `PATH` (`<fixture>/bin:/usr/bin:/bin`) and its own `HOME`,
/// and a prewarm refuses before it clones when no `cargo` can be found to clean the copy with.
/// The tools are this run's own, linked into the fixture's bin directory rather than found on the host's path.
fn the_clone_keeps_its_tools(fixture: &Fixture) {
    let mut environment = fixture.service_environment.lock().expect("the fixture's service environment");
    for name in ["CARGO_HOME", "RUSTUP_HOME"] {
        if let Some(value) = std::env::var_os(name) {
            environment.push((std::ffi::OsString::from(name), value));
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
/// written here so the measurement does not need a Cargo build of its own to reach the queue:
/// the lane clones the directory for real with the shipped handler,
/// which is the step the audit found taking the places a person's work needs.
fn plant_the_queue(setup: &Setup) {
    let source: PathBuf = setup.fixture.root.join("prewarm-source");
    std::fs::create_dir_all(source.as_path()).expect("the source build directory");
    for entry in 0..THE_CLONE_ENTRIES {
        std::fs::write(source.join(format!("dependency-{entry}")), b"a build's own state\n").expect("a source entry");
    }
    for at in 0..THE_QUEUE {
        // The record's object name, which is what names one prewarm of one worktree.
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
            created: String::from("2026-11-12T00:00:00.000Z"),
            tries: 0,
        };
        enqueue(setup.work().as_path(), &item).expect("the prewarm record is queued");
    }
}

/// The staging directory one prewarm clones into, beside the build directory it fills.
fn staging_of(setup: &Setup, at: usize) -> PathBuf {
    return setup.fixture.root.join(format!("prewarm-build-{at}.monochromatic-prewarm"));
}

/// Whether the lane is inside a clone now.
fn inside_a_clone(setup: &Setup) -> bool {
    return (0..THE_QUEUE).any(|at| { return staging_of(setup, at).exists(); });
}

/// Commit `content` through the service and answer how long the command took.
fn commit_and_time(setup: &Setup, content: &str) -> (Output, Duration) {
    std::fs::write(setup.repo.join("a.txt"), content).expect("the edit");
    let mut command: std::process::Command =
        super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
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
///  A commit and its push, made while the service is inside a batch of the repository's own housekeeping records,
///  with the commit's own wall time and the push's wait reported.
/// Why:
///   The measurement answers the one question the queue raises for a person:
/// how long did the commit take, and how long did the backup wait,
/// while the service was already busy with work nobody waits for.
/// What it asserts is what every run must give:
/// both are inside `A_WAIT_NO_QUEUE_MAY_CAUSE`,
/// which is shorter than the queue it runs beside.
#[test]
#[ignore = "a measurement for the record, not a control; run it alone"]
fn housekeeping_beside_a_commit_and_its_push() {
    let setup: Setup = setup("housekeeping-beside-a-commit");
    plant_the_queue(&setup);
    // The first commit wakes the service: its round claims the push and the lane claims the queue.
    let (first, _first_ms): (Output, Duration) = commit_and_time(&setup, "one\n");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(first.stderr.as_slice()));
    // The queue is running once a prewarm of it has been settled, and the waves after that one are still ahead.
    wait_until("the lane has settled a prewarm of the queue", &|| { return settled_prewarms(&setup) >= 1; });
    let (inside_before, queued_before): (bool, usize) = (inside_a_clone(&setup), pending_records(&setup));
    // The measured commit: the queue is running now, and nobody else is making work for this service.
    let (second, commit_ms): (Output, Duration) = commit_and_time(&setup, "two\n");
    assert!(second.status.success(), "{}", String::from_utf8_lossy(second.stderr.as_slice()));
    let inside_after: bool = inside_a_clone(&setup);
    let landed: String = head(&setup);
    let push_began: Instant = Instant::now();
    wait_until("the remote holds the commit", &|| { return remote_has(&setup, landed.as_str()); });
    let push_ms: Duration = push_began.elapsed();
    eprintln!(
        "MEASURE queue_at_the_commit={queued_before} inside_a_clone_before={inside_before} inside_a_clone_after={inside_after} commit_ms={} push_ms={}",
        commit_ms.as_millis(),
        push_ms.as_millis()
    );
    assert!(commit_ms < A_WAIT_NO_QUEUE_MAY_CAUSE, "the commit waited {commit_ms:?} behind the queue");
    assert!(push_ms < A_WAIT_NO_QUEUE_MAY_CAUSE, "the push waited {push_ms:?} behind the queue");
    remove_settled(&setup.fixture);
}

/// How many prewarm records of the queue the service has settled,
/// which is what says the lane is running the queue now.
fn settled_prewarms(setup: &Setup) -> usize {
    let mut settled: usize = 0;
    for state in ["done", "failed"] {
        let directory: PathBuf = setup.work().join(state);
        settled += std::fs::read_dir(directory.as_path())
            .map(|listing| { return listing.flatten().filter(|entry| { return entry.file_name().to_string_lossy().starts_with("prewarm-"); }).count(); })
            .unwrap_or_default();
    }
    return settled;
}

/// The records the queue still holds in `pending` and `claimed`,
/// so the report says how much housekeeping was still ahead of the commit.
fn pending_records(setup: &Setup) -> usize {
    let mut waiting: usize = 0;
    for state in ["pending", "claimed"] {
        let directory: PathBuf = setup.work().join(state);
        waiting += std::fs::read_dir(directory.as_path()).map(|listing| { return listing.flatten().count(); }).unwrap_or_default();
    }
    return waiting;
}
