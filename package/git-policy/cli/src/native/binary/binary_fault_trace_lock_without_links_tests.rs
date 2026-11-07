//! What:
//!  The commit service killed at each step of taking `index.lock` on a filesystem that makes neither a rename without replacing
//!  nor a hard link (`link`, `linkat` and `renameat2` answer `EOPNOTSUPP` under `strace --inject`), and at each later instant of
//!  a batch.
//! Why:
//!  There the service takes the lock as Git does, by exclusive creation and then the marker, so a kill between the two leaves a
//!  lock without its marker, which plain Git leaves too. That lock is never removed unless its holder is proven gone: a wrapped
//!  `git add` meets it as Git does and leaves it, and after the person removes it the next commit lands with nothing lost.
//!  A kill at any other instant leaves a lock with the whole marker, which the next landing proves stale and removes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const point of [...lockSteps, ...batchPoints]) { await killServiceAt(point, { refuse: ['link', 'linkat', 'renameat2'] }); await expectRecovered(point); }
//! ```

use super::super::commit::remove_settled;
use super::super::fault::{COMMIT_ARGUMENTS, Effect, Injection, When, staged_repository};
use super::super::fault_run::{end_the_pressure, in_batches, run_unsettled, run_watched, unbounded_wrapper};
use super::super::strace::strace_available;
use super::super::support::{Fixture, ServiceLaunch, fixture};
use super::{recovers_after_service_death, took_effect, tracer};
use git_policy_cli::service_fault::{FAULT_POINTS, FAULT_VARIABLE, LOCK_CREATED_UNMARKED, LOCK_MARKED_UNSYNCED};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

/// The calls a filesystem that places no file whole refuses.
const REFUSED_CALLS: &str = "link,linkat,renameat2";

/// The instants a kill is tested at: the two steps of taking the lock by exclusive creation, then every point of a batch.
fn kill_points() -> Vec<&'static str> {
    let mut points: Vec<&'static str> = vec![LOCK_CREATED_UNMARKED, LOCK_MARKED_UNSYNCED];
    points.extend(FAULT_POINTS);
    return points;
}

/// One commit through a service that is killed at `point` while the filesystem refuses the calls, then the recovery the person
///  or the next landing makes.
fn kill_case(point: &str) -> Result<(), String> {
    let label: String = format!("lockless-kill-{point}");
    let fixture: Fixture = fixture(label.as_str());
    let trace: PathBuf = fixture.root.join("service-trace.txt");
    let fault: Injection = Injection { syscall: REFUSED_CALLS, effect: Effect::Fail("EOPNOTSUPP"), when: When::FromNth(1), follow_forks: true };
    // `env` arms the service's own fault point; the tracer refuses the calls in the service and in each Git it starts.
    let mut prefix: Vec<OsString> = vec![OsString::from("env"), OsString::from(format!("{FAULT_VARIABLE}={point}"))];
    prefix.extend(tracer(trace.as_path(), fault.syscall, Some(&fault), true));
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch { prefix, may_fail_to_start: true, ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let output: Output = run_watched(&fixture, &mut unbounded_wrapper(&fixture, repo.as_path(), &COMMIT_ARGUMENTS));
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    // The fault ends: the services stop and the next one starts as in production, on a filesystem that works.
    end_the_pressure(&fixture);
    let traced: String = std::fs::read_to_string(&trace).unwrap_or_default();
    if !traced.contains("+++ killed by SIGKILL +++") || !traced.contains("(INJECTED)") || !took_effect(traced.as_str()) {
        return Err(format!("{label}: the service was not killed after a refused call, so nothing was tested: {traced}\nthe run said: {said}"));
    }
    let lock: PathBuf = repo.join(".git/index.lock");
    if point == LOCK_CREATED_UNMARKED {
        let left: Vec<u8> = std::fs::read(&lock).map_err(|error| { return format!("{label}: the kill between creation and marker leaves the lock: {error}"); })?;
        if !left.is_empty() {
            return Err(format!("{label}: the lock must be without its marker, as plain Git leaves one: {left:?}"));
        }
        // A wrapped `git add` meets the lock as Git does, and nothing of the wrapper removes it.
        std::fs::write(repo.join("b.txt"), "b\n").map_err(|error| { return format!("{label}: {error}"); })?;
        let added = run_unsettled(&fixture, repo.as_path(), &["add", "b.txt"]);
        if added.code == Some(0) || !String::from_utf8_lossy(added.stderr.as_slice()).contains("index.lock") {
            return Err(format!("{label}: `git add` must be refused by the lock as Git refuses it, exit {:?}: {}", added.code, String::from_utf8_lossy(added.stderr.as_slice())));
        }
        if !matches!(std::fs::read(&lock), Ok(still) if still.is_empty()) {
            return Err(format!("{label}: nothing but the person may remove a lock without its marker"));
        }
        // The person's fix, as Git's own message says it.
        std::fs::remove_file(&lock).map_err(|error| { return format!("{label}: {error}"); })?;
        // The file the refused `git add` left untracked is the control's own, not the person's.
        std::fs::remove_file(repo.join("b.txt")).map_err(|error| { return format!("{label}: {error}"); })?;
    }
    let verdict: Result<(), String> = recovers_after_service_death(&fixture, repo.as_path(), label.as_str(), said.as_str());
    remove_settled(&fixture);
    return verdict;
}

/// The service killed holding or taking the lock on such a filesystem leaves a repository that recovers: a lock without its marker
/// stays until the person removes it and then the commit lands; a lock with its whole marker is removed by the next landing.
#[test]
fn a_service_killed_at_each_step_of_taking_the_lock_without_links_leaves_a_recoverable_repository() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was refused");
        return;
    }
    let points: Vec<&'static str> = kill_points();
    let failures: Vec<String> = in_batches(points.as_slice(), &|point: &&'static str| { return kill_case(point); });
    assert!(failures.is_empty(), "{} of {} kill points left the repository unrecovered:\n{}", failures.len(), points.len(), failures.join("\n"));
}
