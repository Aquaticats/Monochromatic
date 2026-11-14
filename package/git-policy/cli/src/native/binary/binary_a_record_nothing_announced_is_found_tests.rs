//! What:
//!  A record that waits and nothing announced it,
//! and the running commit service that finds it by itself.
//! Why:
//!   A wrapper writes a record and then wakes the service,
//! and a process killed at any instruction can be killed between the two
//! (`service_fault.rs`,
//! `AFTER_PROMOTE`);
//! a wake is also lost on a socket that was full,
//! a service that was still starting,
//! or a client that was killed while it spoke.
//! The record
//!  then stays in `pending`,
//! where nothing runs it until a command's wake or a service that starts again looks.
//! The look
//!  (`service_claim_watch.rs`,
//! `LOOK_AGAIN_AFTER`) is what closes that:
//! a running service asks its journal's notes which repositories hold a
//!  record no claim took,
//! and finds every kind of record that way.
//! These controls run the built wrapper and a real service against real Git and a
//!  bare remote.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! killTheWrapperAfterThePushIsPromoted(); expect(await onRemoteWithinTheBound()).toBe(true);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, fixture, git, repository, start_fixture_service, FixtureService};
use git_policy_cli::service_claim_watch::LOOK_AGAIN_AFTER;
use git_policy_cli::service_fault::AFTER_PROMOTE;
use git_policy_cli::service_work::work_directory;
use git_policy_cli::service_work_handlers_replanning_and_acceptance::SERVICE_KINDS;
use git_policy_cli::work_item::{DONE, FAILED, PENDING, WorkItem, enqueue, state_directory};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// A repository with one tracked file,
/// a bare remote that has the branch,
/// and a service the commits go through.
struct Setup {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
    service: FixtureService,
}

fn setup(name: &str) -> Setup {
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
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service };
}

fn git_dir(setup: &Setup) -> PathBuf {
    return setup.repo.join(".git");
}

fn work_dir(setup: &Setup) -> PathBuf {
    return work_directory(git_dir(setup).as_path());
}

/// The names of the records waiting in one state directory,
/// and nothing when the directory is not there.
fn records_in(setup: &Setup, state: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(state_directory(work_dir(setup).as_path(), state))
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.sort();
    return names;
}

fn head_of(setup: &Setup, reference: &str) -> String {
    return String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", reference]).stdout).trim().to_string();
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

fn wait_until(what: &str, patience: Duration, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < patience, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// What:
///  Commits `content` to `a.txt` and ends the wrapper at the instant after its push was promoted,
/// so a runnable record waits and nothing was told about it.
/// Why:
///   The fault point is the only way to stand inside that window,
/// and it is inert unless the variable names it.
fn commit_and_die_after_promote(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    command.env(git_policy_cli::service_fault::FAULT_VARIABLE, AFTER_PROMOTE);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(!output.status.success(), "the wrapper was killed at the fault point: {:?}", output.status);
    return head_of(setup, "HEAD");
}

/// Commits `content` through the service and answers the new commit's name.
fn commit_through(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    return head_of(setup, "HEAD");
}

/// What:
///  The service's own log,
/// where every push it makes is one line naming the branch and the commit.
/// Why:
///   A control that asks whether a record was acted on twice reads the lines the service wrote for it,
/// not the
///  remote's state,
/// which is the same whether a push was made once or twice.
fn push_lines_naming(setup: &Setup, commit: &str) -> usize {
    let log: String = crate::support_fixture_service_log::service_log(setup.fixture.root.as_path());
    return log.lines().filter(|line| { return line.contains("pushed branch=") && line.contains(commit); }).count();
}

/// The clock ticks one process of this fixture used,
/// as the kernel counts them.
fn cpu_ticks(pid: u32) -> Option<u64> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_name: &str = stat.rsplit_once(')')?.1;
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    return Some(user + system);
}

fn ticks_per_second() -> f64 {
    // SAFETY: `sysconf` reads a constant of the system and touches no memory of this process.
    let per_second: i64 = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    return if per_second > 0 { per_second as f64 } else { 100.0 };
}

/// What:
///  A service that was already running when the wrapper died before its wake pushes the record by itself,
/// within the bound it says it looks in.
/// Why:
///   This is the state the audit of `0.6.1-alpha.14` left (`report.md`, finding F2):
/// a `pending` push record that
///  nothing ran for 229 seconds,
/// while a service ran the whole time.
/// Nothing restarts the service here and no command is
///  made:
/// the look is the only thing that can find the record.
#[test]
fn a_running_service_pushes_a_promoted_record_within_the_bound() {
    let setup: Setup = setup("nothing-announced-push");
    let commit: String = commit_and_die_after_promote(&setup, "one\n");
    assert_eq!(records_in(&setup, PENDING).len(), 1, "the promotion left one record waiting");
    assert!(!remote_has(&setup, commit.as_str()), "nothing pushed it before the look");
    wait_until("the running service pushed the record", LOOK_AGAIN_AFTER * 3, &|| { return remote_has(&setup, commit.as_str()); });
    assert!(records_in(&setup, PENDING).is_empty(), "the record no longer waits: {:?}", records_in(&setup, PENDING));
    remove_settled(&setup.fixture);
}

/// What:
///  Every kind of record a client leaves is claimed by a running service when nothing woken for it,
/// and no kind waits for a wake that never comes.
/// Why:
///   The push is the kind the audit met;
/// the trash of an add,
/// the prewarm and relink of a new worktree,
/// the delete of a
///  removed worktree's subvolume,
/// a snapshot made ahead,
/// the upkeep of a commit and a shadow review are written the same way,
///  and each of them is a step the wrapper took on and nobody would finish.
/// A record that is claimed leaves `pending` whatever its handler
///  answers:
/// done,
/// failed,
/// skipped or a retry that keeps its claim,
/// so the claim is what this control asserts.
#[test]
fn every_kind_of_record_waiting_with_no_wake_is_claimed_within_the_bound() {
    let setup: Setup = setup("nothing-announced-every-kind");
    let tip: String = head_of(&setup, "HEAD");
    let parent: String = head_of(&setup, "HEAD~1");
    let mut written: Vec<String> = Vec::new();
    for kind in SERVICE_KINDS {
        let item: WorkItem = WorkItem {
            kind,
            repository: setup.repo.clone(),
            git_dir: git_dir(&setup),
            branch_ref: String::from("refs/heads/main"),
            oid: tip.clone(),
            parent: parent.clone(),
            created: git_policy_cli::iso_time::now_iso(),
            tries: 0,
        };
        written.push(item.id());
        enqueue(work_dir(&setup).as_path(), &item).expect("the record is written");
    }
    assert_eq!(records_in(&setup, PENDING).len(), written.len(), "every kind waits, and nothing was woken for any of them");
    // The kinds whose steps can take minutes are claimed by one lane, one at a time, so the bound each record is found in is
    //  the look's bound and each may wait for the ones beside it.
    let patience: Duration = LOOK_AGAIN_AFTER * 3;
    for (id, kind) in written.iter().zip(SERVICE_KINDS) {
        wait_until(
            format!("the running service claimed the {} record within the bound", kind.name()).as_str(),
            patience,
            &|| { return !records_in(&setup, PENDING).contains(id); },
        );
    }
    assert!(records_in(&setup, PENDING).is_empty(), "every kind was claimed: {:?}", records_in(&setup, PENDING));
    remove_settled(&setup.fixture);
}

/// What:
///  A record the look ran is not run again by the wake that arrives afterwards,
/// which is the other side of the same promise.
/// Why:
///   The look and a wake can both name one repository,
/// and the queue's claim is what makes the second one a pass over
///  records that are already settled.
/// The service's own line for a push is one per push it makes
/// (`service_work_drain_pending_claims.rs`),
///  so a second run of the same record would show there.
#[test]
fn a_record_the_look_ran_is_not_run_again_by_a_late_wake() {
    let setup: Setup = setup("nothing-announced-late-wake");
    let looked: String = commit_and_die_after_promote(&setup, "one\n");
    wait_until("the running service pushed the record", LOOK_AGAIN_AFTER * 3, &|| { return remote_has(&setup, looked.as_str()); });
    assert_eq!(push_lines_naming(&setup, looked.as_str()), 1, "the look made the push once");
    // A later commit wakes the service for the same repository, which is the late wake the record's own wake would have been.
    let later: String = commit_through(&setup, "two\n");
    wait_until("the later commit's push settled", LOOK_AGAIN_AFTER * 6, &|| { return records_in(&setup, PENDING).is_empty(); });
    wait_until("the later commit reached the remote", LOOK_AGAIN_AFTER * 6, &|| { return remote_has(&setup, later.as_str()); });
    assert_eq!(push_lines_naming(&setup, looked.as_str()), 1, "the late wake did not push the settled record again");
    assert!(records_in(&setup, DONE).iter().any(|name| { return name.ends_with(looked.as_str()); }), "the record is where a push that was made is kept");
    remove_settled(&setup.fixture);
}

/// What:
///  A service with nothing to do runs no work and stays cheap.
/// Why:
///   The look is asked every `LOOK_AGAIN_AFTER`,
/// and a look that found work would prove nothing about a service that has none:
///  this control waits several bounds with no record anywhere,
/// and asserts that the service wrote no line about a push or a claim,
///  settled nothing,
/// and spent a tenth of the wall time or less on the whole of it.
/// A service that scanned its folders in a busy loop,
/// or ran
///  a round of work for the clock alone,
/// spends far more than that.
#[test]
fn an_idle_service_runs_no_work() {
    let setup: Setup = setup("nothing-announced-idle");
    // The service of this worktree, and the one process the fixture started for it.
    let pid: u32 = match setup.service.process_ids().as_slice() {
        [only] => *only,
        other => panic!("one service serves this worktree: {other:?}"),
    };
    let before: u64 = cpu_ticks(pid).expect("the service's own counters");
    let waited: Duration = LOOK_AGAIN_AFTER * 4;
    let began: Instant = Instant::now();
    while began.elapsed() < waited {
        std::thread::sleep(Duration::from_millis(100));
    }
    let after: u64 = cpu_ticks(pid).expect("the service is still there");
    let used: Duration = Duration::from_secs_f64((after - before) as f64 / ticks_per_second());
    let log: String = crate::support_fixture_service_log::service_log(setup.fixture.root.as_path());
    assert!(!log.contains("pushed branch="), "an idle service made no push: {log}");
    assert!(!log.contains("claim"), "an idle service claimed nothing: {log}");
    assert!(records_in(&setup, PENDING).is_empty() && records_in(&setup, DONE).is_empty() && records_in(&setup, FAILED).is_empty(), "no record was written and none was settled");
    assert!(
        used * 10 < waited,
        "an idle service used {used:?} of {waited:?}, which is more than a tenth: {}",
        log.lines().filter(|line| { return !line.contains("ready") && !line.contains("started"); }).count()
    );
    remove_settled(&setup.fixture);
}
