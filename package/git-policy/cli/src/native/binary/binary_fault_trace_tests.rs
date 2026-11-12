//! What:
//!  Slow storage,
//! a full disk,
//! failing writes and a process killed at any call,
//! injected into the commit service and into the
//!  wrapper's own merge and worktree-copy flows with `strace --inject` (a `ptrace` of the program:
//! no privilege,
//! no load).
//! Why:
//!  The README promises that a full disk,
//! slow storage and a process killed at any instruction do not strand the repository.
//!  A kill at a chosen phase marker or journal point shows that the recovery for that point works;
//! these controls cover the
//!  places between them.
//! Each control counts the calls of one kind that the flow makes (a run with no fault),
//! then repeats the
//!  flow with the fault at a spread of those positions,
//! first and last included,
//! and requires after every run that the command
//!  ended on its own,
//! that the next ordinary command succeeds without a person repairing anything,
//! that the repository passes
//!  `git fsck`,
//! and that nothing of the wrapper stays behind.
//! The counters of `strace` are per process,
//! so a position is the
//!  `n`th call of the busiest process.
//! A host without `strace` reports that and checks nothing,
//! as the budget controls do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const n of spread(await countCalls('write', flow))) { await runWithFault({ call: 'write', nth: n, effect: 'ENOSPC' }, flow); await expectRecovered(); }
//! ```

use super::commit::{git_text, remove_settled};
use super::fault::{COMMIT_ARGUMENTS, Effect, Injection, When, staged_repository, verify_recovery_with};
use super::fault_limits::check_limited_run;
use super::fault_run::{end_the_pressure, in_batches, run_unsettled, run_watched, spawn_watched, unbounded_wrapper, with_manager};
use super::strace::{strace_available, strace_program};
use super::support::{Fixture, ServiceLaunch, fixture, hermetic, service_runtime, wrapper_first_path};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many runs one sweep makes at most:
/// the positions are spread over the calls the flow makes,
/// first and last included.
const POSITIONS_PER_SWEEP: usize = 48;

/// How many runs one call's own kill control makes at most.
/// A position cost up to about twenty seconds with its fixture and the recovery commands under the host's load
/// (on 2026-11-09 the copy flow's 12 `pwrite64` positions took 234.1 s while its 16 `openat` ones took 2.3 s,
/// so the spread is the host's and not the control's),
/// and twelve positions keep the worst measured rate inside the five-minute per-test limit.
const POSITIONS_PER_CALL: usize = 12;

/// How long each call of the slow-storage control returns late,
/// in microseconds:
/// a slow disk's flush.
const SLOW_FLUSH_MICROSECONDS: u64 = 100_000;

/// The calls that write to storage or set a name,
/// for the sweeps of a service.
/// Removing a name is left out:
/// a disk that is full
///  or failing makes a write,
/// a flush or a rename fail,
/// while the lock files that a failed removal would leave behind need a
///  person to remove them,
/// with native Git as with the wrapper.
/// `fdatasync` is not listed:
/// the measured commit made no `fdatasync` in the service or in the Git it starts (2026-11-09),
///  so injecting there faulted no call.
const STORAGE_CALLS: [&str; 5] = ["write", "pwrite64", "fsync", "rename", "renameat2"];

/// The calls a kill of the service is swept over:
/// every state-changing call the service process itself makes in one commit with no forks followed
/// (measured 2026-11-09 with `strace --trace` over the whole set),
/// writes,
/// appends,
/// renames,
/// removals,
/// directory creation,
/// permissions,
/// times and the process start of a Git.
/// `openat` counts every open,
/// reads included,
/// because `strace --inject` names a call and an ordinal over all of them.
/// `fsync`,
/// `fdatasync`,
/// `mkdirat` and `vfork` are not listed:
/// the service makes none of them itself in this flow,
///  so a kill at each tested nothing while the list said the service was killed there.
const KILL_CALLS: [&str; 13] = [
    "write", "pwrite64", "ftruncate", "copy_file_range", "openat", "rename", "renameat2", "unlink", "unlinkat", "mkdir", "chmod", "utimensat", "clone3",
];

/// The `strace` words that run a program under a tracer,
/// in front of the program:
/// the tracer,
/// whether its children are traced,
///  the file it writes,
/// the calls it watches and the fault it injects,
/// if any.
/// One `--quiet` suppresses only the attach and personality messages;
/// the `+++ exited with N +++` lines stay,
/// and `busiest_count` reads a line only when it begins with a process ID and a traced call.
fn tracer(trace: &Path, syscall: &str, fault: Option<&Injection>, follow: bool) -> Vec<OsString> {
    let mut words: Vec<OsString> = vec![strace_program().expect("strace is installed").into_os_string()];
    if follow {
        words.push(OsString::from("--follow-forks"));
    }
    words.push(OsString::from("--quiet"));
    words.push(OsString::from(format!("--output={}", trace.display())));
    words.push(OsString::from(format!("--trace={syscall}")));
    if let Some(injection) = fault {
        words.push(OsString::from(format!("--inject={}", injection.specification())));
    }
    return words;
}

/// How many calls of `syscall` the busiest process of a trace made:
/// the trace of a followed tree has the process ID first on
///  each line,
/// and a trace of one process has none.
fn busiest_count(trace: &str, syscall: &str) -> usize {
    let mut per_process: HashMap<&str, usize> = HashMap::new();
    for line in trace.lines() {
        let (process, call): (&str, &str) = match line.split_once(' ') {
            // Some builds of `strace` pad the process ID with spaces, so the call starts after any number of them.
            Some((first, rest)) if first.bytes().all(|byte| { return byte.is_ascii_digit(); }) && !first.is_empty() => (first, rest.trim_start()),
            _ => ("", line),
        };
        if call.starts_with(syscall) && call[syscall.len()..].starts_with('(') {
            *per_process.entry(process).or_insert(0) += 1;
        }
    }
    return per_process.values().copied().max().unwrap_or(0);
}

/// Up to `budget` positions in `1..=count`,
/// evenly spread,
/// the first and the last included.
fn positions(count: usize, budget: usize) -> Vec<usize> {
    if count <= budget {
        return (1..=count).collect();
    }
    let mut spread: Vec<usize> = (0..budget).map(|step| { return 1 + (count - 1) * step / (budget - 1); }).collect();
    spread.dedup();
    return spread;
}

/// Fails when a sweep's list names a call its flow never makes:
/// nothing would be faulted at that name,
/// so the list would claim a coverage it never had.
fn require_every_listed_call_is_made(label: &str, counts: &[(&'static str, usize)]) {
    let absent: Vec<&'static str> = counts.iter().filter(|(_, count)| { return *count == 0; }).map(|(syscall, _)| { return *syscall; }).collect();
    assert!(
        absent.is_empty(),
        "{label} lists calls its flow never makes, so faulting them would test nothing: {absent:?} (measured {counts:?})"
    );
}

/// Every `(call, position)` of a sweep over `counts`,
/// the budget shared in proportion to how often each call is made,
/// each
///  call that is made at all getting at least its first and last.
fn sweep_points(counts: &[(&'static str, usize)], budget: usize) -> Vec<(&'static str, usize)> {
    let total: usize = counts.iter().map(|(_, count)| { return *count; }).sum();
    let mut points: Vec<(&'static str, usize)> = Vec::new();
    for (syscall, count) in counts {
        if *count == 0 {
            continue;
        }
        let share: usize = (budget * *count / total.max(1)).max(2);
        points.extend(positions(*count, share).into_iter().map(|position| { return (*syscall, position); }));
    }
    return points;
}

/// What must hold after the first run met its fault and the pressure ended:
/// the label,
/// the repository and what the first run said.
type Recovery = fn(&Fixture, &Path, &str, &str) -> Result<(), String>;

/// Recovery as the first commands a person runs after a full or failing disk:
/// stage another file,
/// commit it.
fn recovers_any_way(fixture: &Fixture, repo: &Path, label: &str, said: &str) -> Result<(), String> {
    return verify_recovery_with(fixture, repo, label, said, run_unsettled);
}

/// Recovery after the service itself was killed.
/// A service killed holding Git's index lock leaves the lock with its mark and its
///  process identity;
/// the next landing removes it,
/// because only the service can prove the lock is its own dead instance's
///  (`service_index_lock.rs`).
/// A `git add` before that landing meets the lock as Git does,
/// so the first command after the
///  kill is the commit,
/// and the staging of another file follows it.
fn recovers_after_service_death(fixture: &Fixture, repo: &Path, label: &str, said: &str) -> Result<(), String> {
    let status = run_unsettled(fixture, repo, &["status", "--porcelain"]);
    if status.code != Some(0) {
        return Err(format!("{label}: the next command must succeed, exit {:?}: {}\nfirst run said: {said}", status.code, String::from_utf8_lossy(status.stderr.as_slice())));
    }
    let commits: String = git_text(fixture, repo, &["rev-list", "--count", "HEAD"]);
    let shown: String = String::from_utf8_lossy(status.stdout.as_slice()).into_owned();
    match commits.trim() {
        "1" if shown == "A  a.txt\n" => {
            let landed = run_unsettled(fixture, repo, &["commit", "--quiet", "--message=x", "--", "a.txt"]);
            if landed.code != Some(0) {
                return Err(format!("{label}: the commit after the kill must land, exit {:?}: {}\nfirst run said: {said}", landed.code, String::from_utf8_lossy(landed.stderr.as_slice())));
            }
        }
        "2" if shown.is_empty() => {}
        other => return Err(format!("{label}: {other} commits with status {shown:?}; first run said: {said}")),
    }
    return verify_recovery_with(fixture, repo, label, said, run_unsettled);
}

/// What a sweep of a service found:
/// how many runs the fault reached,
/// and how many commits landed despite it.
struct Found {
    /// Runs in which the tracer reports an injected call or a killed process.
    reached: AtomicUsize,
    /// Runs whose commit landed.
    landed: AtomicUsize,
}

/// Whether the trace shows the fault took effect.
fn took_effect(trace: &str) -> bool {
    return trace.contains("(INJECTED)") || trace.contains("(DELAYED)") || trace.contains("+++ killed by SIGKILL +++");
}

/// What:
///  Wait until the trace shows the fault,
///  for as long as the tracer may still be writing the calls' endings.
/// Why:
///   A traced call that is held is written in two lines:
/// `fsync(13 <unfinished ...>` while it is held,
/// and `... fsync resumed>) = 0 (DELAYED)` when it returns.
/// A trace read as soon as the commit is answered,
/// or as soon as the service is stopped,
/// can hold every held call unfinished and no mark at all,
/// which is what this control reported on 2026-11-09 in a run of the nine `fault_trace` controls
/// (`no flush was delayed, so nothing was tested: 114642 +++ exited with 0 +++`,
/// with the only flush lines reading `fsync(13 <unfinished ...>`) while a run of the control alone passed.
/// The fault is its own clock:
/// a held call returns after `SLOW_FLUSH_MICROSECONDS`,
/// and this waits a hundred of those before it calls the trace broken.
fn wait_for_the_fault(trace: &Path) -> String {
    let mut tracing: String = std::fs::read_to_string(trace).unwrap_or_default();
    let mut looked: u32 = 0;
    while !took_effect(tracing.as_str()) && looked < 100 {
        std::thread::sleep(std::time::Duration::from_micros(SLOW_FLUSH_MICROSECONDS));
        looked += 1;
        let now: String = std::fs::read_to_string(trace).unwrap_or_default();
        tracing = now;
    }
    return tracing;
}

/// One commit with the service under a tracer that injects `fault`,
/// then the fault ends (the service is stopped,
/// the next one
///  runs as in production),
/// and the repository must recover.
fn service_case(label: &str, fault: &Injection, found: &Found, recover: Recovery) -> Result<(), String> {
    let fixture: Fixture = fixture(label);
    let trace: PathBuf = fixture.root.join("service-trace.txt");
    *fixture.service_launch.lock().expect("the service launch") =
        ServiceLaunch { prefix: tracer(trace.as_path(), fault.syscall, Some(fault), fault.follow_forks), may_fail_to_start: true, ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    // A service that is killed is started again by the user manager at the client's next connection; a failing disk does not end it.
    let run = || { return run_watched(&fixture, &mut unbounded_wrapper(&fixture, repo.as_path(), &COMMIT_ARGUMENTS)); };
    let output: Output = if matches!(fault.effect, Effect::Kill) { with_manager(&fixture, run) } else { run() };
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    end_the_pressure(&fixture);
    let traced: String = std::fs::read_to_string(&trace).unwrap_or_default();
    if took_effect(traced.as_str()) {
        found.reached.fetch_add(1, Ordering::SeqCst);
    }
    if output.status.code() == Some(0) {
        found.landed.fetch_add(1, Ordering::SeqCst);
    }
    let verdict: Result<(), String> = check_limited_run(label, &output, false)
        .and_then(|()| { return recover(&fixture, repo.as_path(), label, said.as_str()); });
    remove_settled(&fixture);
    return verdict;
}

/// The busiest process's number of calls of `syscall` in one commit through a traced service.
fn service_call_count(syscall: &'static str, follow: bool) -> usize {
    let fixture: Fixture = fixture("count-service");
    let trace: PathBuf = fixture.root.join("service-trace.txt");
    *fixture.service_launch.lock().expect("the service launch") =
        ServiceLaunch { prefix: tracer(trace.as_path(), syscall, None, follow), ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let output: Output = run_watched(&fixture, &mut unbounded_wrapper(&fixture, repo.as_path(), &COMMIT_ARGUMENTS));
    assert_eq!(output.status.code(), Some(0), "the counting commit lands: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    end_the_pressure(&fixture);
    let traced: Result<String, std::io::Error> = std::fs::read_to_string(&trace);
    let count: usize = busiest_count(traced.as_deref().unwrap_or_default(), syscall);
    assert!(
        count > 0 || syscall != "write",
        "the counting commit's trace holds no write at all: {:?}",
        traced.as_ref().map(|text| { return text.chars().take(400).collect::<String>(); })
    );
    remove_settled(&fixture);
    return count;
}

/// Sweeps one fault over the service's calls:
/// `effect` at each position of each call in `calls`.
fn sweep_service(label: &str, calls: &[&'static str], effect: Effect, when: fn(usize) -> When, follow: bool, recover: Recovery) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was faulted");
        return;
    }
    let counts: Vec<(&'static str, usize)> = calls.iter().map(|syscall| { return (*syscall, service_call_count(syscall, follow)); }).collect();
    require_every_listed_call_is_made(label, counts.as_slice());
    let points: Vec<(&'static str, usize)> = sweep_points(counts.as_slice(), POSITIONS_PER_SWEEP);
    assert!(!points.is_empty(), "the commit made none of the calls {calls:?}: {counts:?}");
    let found: Found = Found { reached: AtomicUsize::new(0), landed: AtomicUsize::new(0) };
    let failures: Vec<String> = in_batches(points.as_slice(), &|(syscall, position): &(&'static str, usize)| {
        let fault: Injection = Injection { syscall, effect, when: when(*position), follow_forks: follow };
        return service_case(format!("{label}-{syscall}-{position}").as_str(), &fault, &found, recover);
    });
    assert!(failures.is_empty(), "{} of {} faults left the repository unrecovered (call counts {counts:?}):\n{}", failures.len(), points.len(), failures.join("\n"));
    assert!(found.reached.load(Ordering::SeqCst) > 0, "no run reached its fault, so the sweep tested nothing (call counts {counts:?})");
    // The manager starts a killed service again, so the client's resend must land at least some of the commits it interrupted.
    assert!(
        !matches!(effect, Effect::Kill) || found.landed.load(Ordering::SeqCst) > 0,
        "no commit landed through the restarted service, so the resend after a kill was not tested (call counts {counts:?})"
    );
}

/// Sweeps a kill of the service over the positions of one call of one commit.
/// One control covers one call,
/// so each call's positions stay inside the per-test limit.
/// The resend after a kill is covered on its own by
/// `binary_service_tests.rs` (`a_client_that_disappears_after_sending_still_lands_once`),
/// so this control does not require one of its own kills to land a commit.
fn sweep_service_call(label: &str, syscall: &'static str, recover: Recovery) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was killed");
        return;
    }
    assert!(KILL_CALLS.contains(&syscall), "{label} has no kill control for {syscall}, so a control here would test what its list does not name");
    let count: usize = service_call_count(syscall, false);
    require_every_listed_call_is_made(label, &[(syscall, count)]);
    let points: Vec<(&'static str, usize)> = sweep_points(&[(syscall, count)], POSITIONS_PER_CALL);
    let found: Found = Found { reached: AtomicUsize::new(0), landed: AtomicUsize::new(0) };
    let failures: Vec<String> = in_batches(points.as_slice(), &|(_, position): &(&'static str, usize)| {
        let fault: Injection = Injection { syscall, effect: Effect::Kill, when: When::Nth(*position), follow_forks: false };
        return service_case(format!("{label}-{syscall}-{position}").as_str(), &fault, &found, recover);
    });
    assert!(failures.is_empty(), "{} of {} kills left the repository unrecovered ({syscall} count {count}):\n{}", failures.len(), points.len(), failures.join("\n"));
    assert!(found.reached.load(Ordering::SeqCst) > 0, "no run was killed, so the sweep tested nothing ({syscall} count {count})");
}

/// Requires that every call the service kill list names is one the commit makes.
fn require_the_service_kill_list_is_made() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was counted");
        return;
    }
    let counts: Vec<(&'static str, usize)> = KILL_CALLS.iter().map(|syscall| { return (*syscall, service_call_count(syscall, false)); }).collect();
    require_every_listed_call_is_made("the service kill list", counts.as_slice());
}

/// The disk is full from the `n`th write on:
/// every write,
/// in the service and in each Git it starts,
/// fails with no space left.
#[test]
fn a_disk_that_fills_during_a_commit_leaves_a_recoverable_repository() {
    sweep_service("full-disk", &STORAGE_CALLS, Effect::Fail("ENOSPC"), When::FromNth, true, recovers_any_way);
}

/// The storage fails with an input/output error from the `n`th call on,
/// in the service and in each Git it starts.
#[test]
fn storage_that_fails_during_a_commit_leaves_a_recoverable_repository() {
    sweep_service("io-error", &STORAGE_CALLS, Effect::Fail("EIO"), When::FromNth, true, recovers_any_way);
}

/// Every call the service kill list names is one the commit makes.
#[test]
fn the_service_kill_list_names_only_calls_the_flow_makes() {
    require_the_service_kill_list_is_made();
}

/// The service is killed at each `write` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_write_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "write", recovers_after_service_death);
}

/// The service is killed at each `pwrite64` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_pwrite64_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "pwrite64", recovers_after_service_death);
}

/// The service is killed at each `ftruncate` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_ftruncate_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "ftruncate", recovers_after_service_death);
}

/// The service is killed at each `copy_file_range` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_copy_file_range_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "copy_file_range", recovers_after_service_death);
}

/// The service is killed at each `openat` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_openat_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "openat", recovers_after_service_death);
}

/// The service is killed at each `rename` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_rename_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "rename", recovers_after_service_death);
}

/// The service is killed at each `renameat2` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_renameat2_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "renameat2", recovers_after_service_death);
}

/// The service is killed at each `unlink` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_unlink_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "unlink", recovers_after_service_death);
}

/// The service is killed at each `unlinkat` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_unlinkat_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "unlinkat", recovers_after_service_death);
}

/// The service is killed at each `mkdir` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_mkdir_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "mkdir", recovers_after_service_death);
}

/// The service is killed at each `chmod` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_chmod_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "chmod", recovers_after_service_death);
}

/// The service is killed at each `utimensat` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_utimensat_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "utimensat", recovers_after_service_death);
}

/// The service is killed at each `clone3` it makes in a commit:
/// the manager starts a new service,
/// the client's request is answered or fails plainly,
/// and no commit is lost or doubled.
#[test]
fn a_service_killed_at_each_clone3_leaves_a_recoverable_repository() {
    sweep_service_call("service-kill", "clone3", recovers_after_service_death);
}

/// Every flush returns late,
/// as on slow storage with a slow fsync:
/// the commit lands,
/// later than it would,
/// and leaves nothing.
#[test]
fn slow_flushes_delay_a_commit_and_change_nothing_else() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was delayed");
        return;
    }
    let fixture: Fixture = fixture("slow-flush");
    let trace: PathBuf = fixture.root.join("service-trace.txt");
    let fault: Injection = Injection { syscall: "fsync,fdatasync", effect: Effect::SlowReturn(SLOW_FLUSH_MICROSECONDS), when: When::FromNth(1), follow_forks: true };
    *fixture.service_launch.lock().expect("the service launch") =
        ServiceLaunch { prefix: tracer(trace.as_path(), fault.syscall, Some(&fault), true), ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let output: Output = run_watched(&fixture, &mut unbounded_wrapper(&fixture, repo.as_path(), &COMMIT_ARGUMENTS));
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    assert_eq!(output.status.code(), Some(0), "a slow flush only delays the commit: {said}");
    end_the_pressure(&fixture);
    let traced: String = wait_for_the_fault(trace.as_path());
    assert!(took_effect(traced.as_str()), "no flush was delayed, so nothing was tested: {traced}");
    assert_eq!(verify_recovery_with(&fixture, repo.as_path(), "slow-flush", said.as_str(), run_unsettled), Ok(()));
    remove_settled(&fixture);
}

/// One commit through the service while every call in `calls` fails from the first on with `EOPNOTSUPP`,
/// in the service and in each
///  Git it starts:
/// the commit still lands.
/// The native control runs the same commit with real Git under the same fault,
/// and must
///  land too ("what Git completes,
/// we complete").
fn commit_lands_while_calls_are_refused(label: &str, calls: &'static str) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was refused");
        return;
    }
    let fault: Injection = Injection { syscall: calls, effect: Effect::Fail("EOPNOTSUPP"), when: When::FromNth(1), follow_forks: true };
    let native: Fixture = fixture(format!("{label}-native").as_str());
    let native_trace: PathBuf = native.root.join("native-trace.txt");
    let native_repo: PathBuf = staged_repository(&native, "repo");
    let words: Vec<OsString> = tracer(native_trace.as_path(), fault.syscall, Some(&fault), true);
    let mut control: Command = Command::new(&words[0]);
    control.args(&words[1..]).arg("/usr/bin/git").args(COMMIT_ARGUMENTS);
    hermetic(&mut control, &native, OsString::from("/usr/bin:/bin").as_os_str());
    control.current_dir(native_repo.as_path());
    let native_output: Output = run_watched(&native, &mut control);
    assert_eq!(
        native_output.status.code(),
        Some(0),
        "the control: real Git lands the commit while links are refused: {}",
        String::from_utf8_lossy(native_output.stderr.as_slice())
    );
    remove_settled(&native);
    let fixture: Fixture = fixture(label);
    let trace: PathBuf = fixture.root.join("service-trace.txt");
    *fixture.service_launch.lock().expect("the service launch") =
        ServiceLaunch { prefix: tracer(trace.as_path(), fault.syscall, Some(&fault), true), ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let output: Output = run_watched(&fixture, &mut unbounded_wrapper(&fixture, repo.as_path(), &COMMIT_ARGUMENTS));
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    end_the_pressure(&fixture);
    let traced: String = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(took_effect(traced.as_str()), "no link was refused, so nothing was tested: {traced}");
    assert_eq!(output.status.code(), Some(0), "the commit lands although links are refused: {said}");
    assert_eq!(verify_recovery_with(&fixture, repo.as_path(), label, said.as_str(), run_unsettled), Ok(()));
    remove_settled(&fixture);
}

/// A state directory on a filesystem that refuses hard links (a FUSE server such as `s3fs-fuse` answers `ENOTSUP`) must not stop
///  a commit:
/// the service's decision file is published without a link where the kernel's exclusive rename works.
#[test]
fn a_filesystem_that_refuses_hard_links_does_not_stop_a_commit() {
    commit_lands_while_calls_are_refused("links-refused", "link,linkat");
}

/// A repository on a filesystem that makes neither a hard link nor a rename that does not replace (a FUSE server such as `s3fs-fuse`
///  answers `ENOTSUP` to both) must not stop a commit:
/// Git creates `index.lock` by exclusive creation and needs neither call,
/// so the
///  service takes its lock that way too.
/// The native control lands the same commit with real Git under the same fault.
#[test]
fn a_filesystem_with_neither_no_replace_rename_nor_links_does_not_stop_a_commit() {
    commit_lands_while_calls_are_refused("neither-refused", "link,linkat,renameat2");
}

/// The service killed at each step of taking the lock on a filesystem that places no file whole,
/// and at each point of a batch.
#[path = "binary_fault_trace_lock_without_links_tests.rs"]
mod lock_without_links;

/// The wrapper under a tracer that injects `fault`,
/// for the flow in `repo`.
fn traced_wrapper(fixture: &Fixture, directory: &Path, arguments: &[&str], trace: &Path, syscall: &str, fault: Option<&Injection>) -> Command {
    let words: Vec<OsString> = tracer(trace, syscall, fault, false);
    let mut command: Command = Command::new(&words[0]);
    command.args(&words[1..]).arg(fixture.root.join("bin/git")).args(arguments);
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    if let Some(runtime) = service_runtime(fixture) {
        command.env("XDG_RUNTIME_DIR", runtime);
    }
    command.current_dir(directory);
    return command;
}

/// A flow of the wrapper that runs in its own process:
/// a way to set up a fresh fixture,
/// the command line,
/// and what must hold
///  after the wrapper was killed in it and the next commands ran.
struct Flow {
    /// The name of the flow,
    /// for labels.
    name: &'static str,
    /// Makes the repository the flow runs in,
    /// and returns it.
    setup: fn(&Fixture) -> PathBuf,
    /// The command line the wrapper runs.
    arguments: &'static [&'static str],
    /// The calls the wrapper is killed at in this flow,
    /// measured as the calls it makes itself with no forks followed.
    killed_at: &'static [&'static str],
    /// Runs the next commands and requires that the repository recovered;
    /// the command line's first run is `killed`.
    recovered: fn(&Fixture, &Path, &Output) -> Result<(), String>,
}

/// How many calls of `syscall` the wrapper makes in a flow with no fault.
fn flow_call_count(flow: &Flow, syscall: &'static str) -> usize {
    let fixture: Fixture = fixture(format!("count-{}", flow.name).as_str());
    let repo: PathBuf = (flow.setup)(&fixture);
    let trace: PathBuf = fixture.root.join("wrapper-trace.txt");
    let output: Output = run_watched(&fixture, &mut traced_wrapper(&fixture, repo.as_path(), flow.arguments, trace.as_path(), syscall, None));
    assert_eq!(output.status.code(), Some(0), "the counting run of {} ends well: {}", flow.name, String::from_utf8_lossy(output.stderr.as_slice()));
    let count: usize = busiest_count(std::fs::read_to_string(&trace).unwrap_or_default().as_str(), syscall);
    remove_settled(&fixture);
    return count;
}

/// The wrapper is killed at the `n`th call of `syscall` in `flow`;
/// the processes it started end by themselves;
/// the next commands
///  recover the repository.
/// One control covers one call,
/// so each call's positions stay inside the per-test limit.
fn sweep_wrapper_kill(flow: &Flow, syscall: &'static str) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process was killed");
        return;
    }
    assert!(
        flow.killed_at.contains(&syscall),
        "{} has no kill control for {syscall}, so a control here would test what its list does not name",
        flow.name
    );
    let count: usize = flow_call_count(flow, syscall);
    require_every_listed_call_is_made(flow.name, &[(syscall, count)]);
    let points: Vec<(&'static str, usize)> = sweep_points(&[(syscall, count)], POSITIONS_PER_CALL);
    let killed: AtomicUsize = AtomicUsize::new(0);
    let failures: Vec<String> = in_batches(points.as_slice(), &|(_, position): &(&'static str, usize)| {
        let label: String = format!("kill-{}-{syscall}-{position}", flow.name);
        let fixture: Fixture = fixture(label.as_str());
        let repo: PathBuf = (flow.setup)(&fixture);
        let trace: PathBuf = fixture.root.join("wrapper-trace.txt");
        let fault: Injection = Injection { syscall, effect: Effect::Kill, when: When::Nth(*position), follow_forks: false };
        let mut command: Command = traced_wrapper(&fixture, repo.as_path(), flow.arguments, trace.as_path(), syscall, Some(&fault));
        let output: Output = spawn_watched(&fixture, &mut command).finish_after_stragglers();
        if took_effect(std::fs::read_to_string(&trace).unwrap_or_default().as_str()) {
            killed.fetch_add(1, Ordering::SeqCst);
        }
        let verdict: Result<(), String> = (flow.recovered)(&fixture, repo.as_path(), &output).map_err(|message| { return format!("{label}: {message}"); });
        remove_settled(&fixture);
        return verdict;
    });
    assert!(failures.is_empty(), "{} of {} kills left the repository unrecovered ({syscall} count {count}):\n{}", failures.len(), points.len(), failures.join("\n"));
    assert!(killed.load(Ordering::SeqCst) > 0, "no run was killed, so the sweep tested nothing ({syscall} count {count})");
}

/// Requires that every call a flow's kill controls cover is one the flow makes,
/// so a list cannot name a call whose positions would be empty.
fn require_the_kill_list_is_made(flow: &Flow) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was counted");
        return;
    }
    let counts: Vec<(&'static str, usize)> = flow.killed_at.iter().map(|syscall| { return (*syscall, flow_call_count(flow, syscall)); }).collect();
    require_every_listed_call_is_made(flow.name, counts.as_slice());
}

/// A repository with another session's work staged and a feature branch to merge,
/// as the merge controls make it.
fn merge_setup(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = super::merge_isolation::diverged(fixture, "repo");
    super::merge_isolation::stage_unrelated_work(fixture, repo.as_path());
    return repo;
}

/// After a merge killed anywhere:
/// the next command recovers;
/// either nothing changed or the merge is whole;
/// the other session's
///  staged work is intact;
/// nothing is left;
/// the merge can be asked for again.
fn merge_recovered(fixture: &Fixture, repo: &Path, killed: &Output) -> Result<(), String> {
    let status = run_unsettled(fixture, repo, &["status", "--porcelain=v1"]);
    if status.code != Some(0) {
        return Err(format!("the next command must succeed, exit {:?}: {} (the killed run: {:?})", status.code, String::from_utf8_lossy(status.stderr.as_slice()), killed.status));
    }
    let merges: String = super::commit::git_text(fixture, repo, &["rev-list", "--count", "--merges", "HEAD"]);
    let shown: String = String::from_utf8_lossy(status.stdout.as_slice()).into_owned();
    let staged_lines: Vec<&str> = shown.lines().collect();
    let other_work_intact: bool = staged_lines.len() == 3
        && staged_lines.iter().any(|line| { return line.starts_with("M  u.txt"); })
        && staged_lines.iter().any(|line| { return line.starts_with("A  new.txt"); })
        && staged_lines.iter().any(|line| { return line.starts_with("D  v.txt"); });
    if merges.trim() == "1" && !other_work_intact {
        return Err(format!("the merge landed but the other session's staged work is not what it was: {shown:?}"));
    }
    if merges.trim() == "0" && shown.lines().count() < 3 {
        return Err(format!("no merge landed but staged work is missing: {shown:?}"));
    }
    if repo.join(".git/index.lock").exists() {
        return Err(String::from("the index lock stays"));
    }
    // A record being written is a dot-file that is renamed into place; one left by a kill is no record, no reader takes it, and
    //  a later merge does not trip on it.
    let records: Vec<PathBuf> = super::merge_isolation::records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path())
        .into_iter()
        .filter(|path| { return path.extension().is_none_or(|extension| { return extension != "partial"; }); })
        .collect();
    if !records.is_empty() {
        return Err(format!("the merge's records stay: {records:?}"));
    }
    let checked = Command::new("/usr/bin/git")
        .current_dir(repo)
        .args(["fsck", "--strict", "--no-dangling"])
        .env_clear()
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .output()
        .expect("git fsck starts");
    if !checked.status.success() || !checked.stderr.is_empty() {
        return Err(format!("git fsck: {}", String::from_utf8_lossy(&checked.stderr)));
    }
    let again = run_unsettled(fixture, repo, &["merge", "--no-edit", "feature"]);
    if again.code != Some(0) {
        return Err(format!("the merge asked for again must end well, exit {:?}: {}", again.code, String::from_utf8_lossy(again.stderr.as_slice())));
    }
    return Ok(());
}

/// The calls the merge flow's kill controls cover:
/// what the wrapper itself makes in it with no forks followed (measured 2026-11-09).
/// `unlinkat`,
/// `mkdirat` and `vfork` are not listed:
/// the wrapper removes with `unlink`,
/// creates directories with `mkdir` and starts Git with `clone3` there,
///  and made none of the three.
/// `openat`,
/// `ftruncate` and `mkdir` are every other state-changing call the flow makes,
/// and `openat` counts reads too because `strace --inject` names a call and an ordinal over all of them.
const MERGE_KILL_CALLS: [&str; 10] = ["write", "pwrite64", "fsync", "ftruncate", "openat", "rename", "renameat2", "unlink", "mkdir", "clone3"];

/// The merge flow's shape,
/// shared by each of its kill controls.
fn merge_flow() -> Flow {
    return Flow {
        name: "merge",
        setup: merge_setup,
        arguments: &["merge", "--no-edit", "feature"],
        killed_at: &MERGE_KILL_CALLS,
        recovered: merge_recovered,
    };
}

/// Every call the merge flow's kill controls cover is one the flow makes.
#[test]
fn the_merge_kill_list_names_only_calls_the_flow_makes() {
    require_the_kill_list_is_made(&merge_flow());
}

/// A merge over another session's staged work,
/// killed at each `write` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_write_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "write");
}

/// A merge over another session's staged work,
/// killed at each `pwrite64` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_pwrite64_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "pwrite64");
}

/// A merge over another session's staged work,
/// killed at each `fsync` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_fsync_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "fsync");
}

/// A merge over another session's staged work,
/// killed at each `rename` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_rename_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "rename");
}

/// A merge over another session's staged work,
/// killed at each `renameat2` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_renameat2_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "renameat2");
}

/// A merge over another session's staged work,
/// killed at each `unlink` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_unlink_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "unlink");
}

/// A merge over another session's staged work,
/// killed at each `clone3` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_clone3_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "clone3");
}

/// A merge over another session's staged work,
/// killed at each `mkdir` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_mkdir_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "mkdir");
}

/// A merge over another session's staged work,
/// killed at each `ftruncate` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_ftruncate_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "ftruncate");
}

/// A merge over another session's staged work,
/// killed at each `openat` the wrapper makes in it.
#[test]
fn a_merge_killed_at_each_openat_leaves_a_recoverable_checkout() {
    sweep_wrapper_kill(&merge_flow(), "openat");
}

/// A repository whose committed configuration names an ignored rules file,
/// as the worktree-copy controls make it.
fn copy_setup(fixture: &Fixture) -> PathBuf {
    return super::worktree_copy::repository_with_rules(fixture, true);
}

/// The names a directory holds,
/// or none when it does not exist.
fn names_in(directory: &Path) -> Vec<String> {
    return std::fs::read_dir(directory).map(|entries| { return entries.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); }).unwrap_or_default();
}

/// Real Git's answer to `git -C directory arguments`,
/// with no system or global configuration,
/// which is what a walk of a checkout's ignored entries needs.
fn real_git_text(directory: &Path, arguments: &[&str]) -> Vec<u8> {
    let output: Output = Command::new("/usr/bin/git")
        .current_dir(directory)
        .args(arguments)
        .env_clear()
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .output()
        .expect("real Git starts");
    assert!(output.status.success(), "git {arguments:?} in {}: {}", directory.display(), String::from_utf8_lossy(output.stderr.as_slice()));
    return output.stdout;
}

/// What one ignored entry of a checkout is:
/// its kind,
/// its permission bits,
/// and its bytes or its link target.
/// Why:
///   A worktree whose copy finished holds exactly what its source ignores,
/// so a comparison of the two sets is the whole promise the copy makes,
/// and a file that is present but empty or truncated is not the source's entry.
fn ignored_state(where_: &Path) -> std::collections::BTreeMap<Vec<u8>, String> {
    use std::os::unix::ffi::OsStrExt as _;
    let mut state: std::collections::BTreeMap<Vec<u8>, String> = std::collections::BTreeMap::new();
    if !where_.join(".git").exists() {
        return state;
    }
    let listed: Vec<u8> = real_git_text(where_, &["ls-files", "--others", "--ignored", "--exclude-standard", "--directory", "-z"]);
    for name in listed.split(|byte| { return *byte == 0_u8; }) {
        if name.is_empty() {
            continue;
        }
        let text: &Path = Path::new(std::ffi::OsStr::from_bytes(name.strip_suffix(b"/").unwrap_or(name)));
        visit_ignored(where_, text, &mut state);
    }
    return state;
}

/// What one ignored entry of a checkout is,
/// and every entry below it when it is a directory.
/// Why:
///   A comparison of two checkouts must see the files a copy installs,
/// not the directory a listing collapses them into.
fn visit_ignored(where_: &Path, relative: &Path, state: &mut std::collections::BTreeMap<Vec<u8>, String>) {
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::MetadataExt as _;
    let path: PathBuf = where_.join(relative);
    let Ok(held) = std::fs::symlink_metadata(path.as_path()) else { return };
    let name: Vec<u8> = relative.as_os_str().as_bytes().to_vec();
    if held.file_type().is_symlink() {
        state.insert(name, format!("link:{}", std::fs::read_link(path.as_path()).expect("target").to_string_lossy()));
        return;
    }
    if held.is_dir() {
        state.insert(name, format!("dir:{:o}", held.mode() & 0o7777));
        for entry in std::fs::read_dir(path.as_path()).expect("directory").flatten() {
            visit_ignored(where_, relative.join(entry.file_name()).as_path(), state);
        }
        return;
    }
    let bytes: Vec<u8> = std::fs::read(path.as_path()).expect("bytes");
    state.insert(name, format!("file:{:o}:{}:{}", held.mode() & 0o7777, bytes.len(), git_policy_cli::hexadecimal_of_bytes::hexadecimal_of_bytes(bytes.as_slice())));
}

/// After a worktree add killed anywhere:
/// the next command in the repository answers;
/// a command in the new worktree recovers what a dead owner left and names it;
/// the worktree,
/// if Git made it,
/// holds exactly the ignored entries its source holds,
/// so no worktree is left looking ordinary and holding less than its source ignores;
/// and no journal,
/// claim,
/// record or stage of a copy into it stays.
/// A kill before Git made the destination leaves a record the recovery keeps until the window in which that Git may
/// still be creating it has passed,
/// and the same command works again.
fn copy_recovered(fixture: &Fixture, repo: &Path, killed: &Output) -> Result<(), String> {
    let status = run_unsettled(fixture, repo, &["status", "--porcelain=v1"]);
    if status.code != Some(0) {
        return Err(format!("the next command must succeed, exit {:?}: {} (the killed run: {:?})", status.code, String::from_utf8_lossy(status.stderr.as_slice()), killed.status));
    }
    let worktree: PathBuf = fixture.root.join("one");
    // Whether Git made the worktree at all: a kill before its Git ran leaves a record whose Git process was never named,
    // and the recovery keeps that record for the window in which that Git may still be creating the destination.
    let made_by_git: bool = worktree.join(".git").exists();
    // What a recovery pass told the person: the first command in the repository settles a dead add's records and says
    // which worktree it filled or left unfilled, and a linked worktree runs such a pass before it forwards too.
    let mut told: String = String::from_utf8_lossy(status.stderr.as_slice()).into_owned();
    if made_by_git {
        let inside = run_unsettled(fixture, worktree.as_path(), &["rev-parse", "--is-inside-work-tree"]);
        if inside.code != Some(0) {
            return Err(format!("the worktree Git made must answer, exit {:?}: {}", inside.code, String::from_utf8_lossy(inside.stderr.as_slice())));
        }
        told.push_str(String::from_utf8_lossy(inside.stderr.as_slice()).as_ref());
    } else {
        // Git had not made it (or its half-made directory was removed by Git's own cleanup): the same command works again.
        let _pruned = run_unsettled(fixture, repo, &["worktree", "prune"]);
        let _removed = std::fs::remove_dir_all(&worktree);
        let again = run_unsettled(fixture, repo, &["worktree", "add", "--quiet", "-b", "one-again", "../one"]);
        told.push_str(String::from_utf8_lossy(again.stderr.as_slice()).as_ref());
        if again.code != Some(0) {
            // The branch name of the killed run may exist: that is Git's answer, not a wrapper defect, when it says so.
            let words: String = String::from_utf8_lossy(again.stderr.as_slice()).into_owned();
            if !words.contains("already exists") {
                return Err(format!("the worktree add asked for again must end well, exit {:?}: {words}", again.code));
            }
        }
    }
    let made: bool = worktree.join(".git").exists();
    if !made {
        return Err(format!("the next commands made no worktree at {}: {told:?}", worktree.display()));
    }
    let source_state: std::collections::BTreeMap<Vec<u8>, String> = ignored_state(repo);
    let worktree_state: std::collections::BTreeMap<Vec<u8>, String> = ignored_state(worktree.as_path());
    if source_state != worktree_state {
        let missing: Vec<String> = source_state.keys().filter(|name| { return !worktree_state.contains_key(*name); }).map(|name| { return String::from_utf8_lossy(name).into_owned(); }).collect();
        let differing: Vec<String> = source_state.iter().filter(|(name, value)| { return worktree_state.get(*name).is_some_and(|found| { return found != *value; }); }).map(|(name, value)| { return format!("{}: {} is not {value}", String::from_utf8_lossy(name), worktree_state.get(name).cloned().unwrap_or_default()); }).collect();
        let extra: Vec<String> = worktree_state.keys().filter(|name| { return !source_state.contains_key(*name); }).map(|name| { return String::from_utf8_lossy(name).into_owned(); }).collect();
        return Err(format!("the worktree does not hold the ignored state of its source: missing {missing:?}, differing {differing:?}, extra {extra:?}; the next commands said {told:?}"));
    }
    // A record of an add whose Git never made its destination may stay: the recovery keeps a record that names no Git until
    // the window in which that Git may still be creating the destination has passed, and settles it then, saying nothing
    // about a worktree nothing needs.
    if !made_by_git {
        return Ok(());
    }
    let journal_root: PathBuf = repo.join(".git/monochromatic-git-policy-cli-worktree-copy/v1");
    let left: Vec<String> = names_in(journal_root.as_path());
    if !left.is_empty() {
        return Err(format!("records of the copy stay after the next commands: {left:?}"));
    }
    let staging: Vec<String> = names_in(fixture.root.as_path()).into_iter().filter(|name| { return name.starts_with(".cli-git-worktree-copy-"); }).collect();
    if !staging.is_empty() {
        return Err(format!("a stage of the copy stays beside the worktree: {staging:?}; the next commands said {told:?}"));
    }
    return Ok(());
}

/// The calls the worktree-copy flow's kill controls cover:
/// what the wrapper itself makes in it with no forks followed (measured 2026-11-09).
/// `unlinkat`,
/// `mkdirat` and `vfork` are not listed:
/// the wrapper removes with `unlink` and `rmdir`,
/// creates directories with `mkdir` and starts Git with `clone3` there,
///  and made none of the three.
/// `openat`,
/// `openat2`,
/// `ftruncate`,
/// `rmdir`,
/// `mkdir`,
/// `chmod` and `ioctl` are every other state-changing call it makes;
/// `ioctl` carries the `FICLONE` that fills each staged file and the btrfs subvolume creation (measured 2026-11-09),
/// and `openat` counts reads too,
/// because `strace --inject` names a call and an ordinal over all of them.
const COPY_KILL_CALLS: [&str; 14] = [
    "write", "pwrite64", "fsync", "ftruncate", "openat", "openat2", "rename", "renameat2", "unlink", "rmdir", "mkdir", "chmod", "ioctl", "clone3",
];

/// The worktree-copy flow's shape,
/// shared by each of its kill controls.
fn copy_flow() -> Flow {
    return Flow {
        name: "copy",
        setup: copy_setup,
        arguments: &["worktree", "add", "--quiet", "-b", "one", "../one"],
        killed_at: &COPY_KILL_CALLS,
        recovered: copy_recovered,
    };
}

/// Every call the worktree-copy flow's kill controls cover is one the flow makes.
#[test]
fn the_worktree_copy_kill_list_names_only_calls_the_flow_makes() {
    require_the_kill_list_is_made(&copy_flow());
}

/// A worktree add with the ignored-state copy,
/// killed at each `write` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_write_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "write");
}

/// A worktree add with the ignored-state copy,
/// killed at each `pwrite64` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_pwrite64_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "pwrite64");
}

/// A worktree add with the ignored-state copy,
/// killed at each `fsync` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_fsync_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "fsync");
}

/// A worktree add with the ignored-state copy,
/// killed at each `rename` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_rename_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "rename");
}

/// A worktree add with the ignored-state copy,
/// killed at each `renameat2` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_renameat2_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "renameat2");
}

/// A worktree add with the ignored-state copy,
/// killed at each `unlink` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_unlink_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "unlink");
}

/// A worktree add with the ignored-state copy,
/// killed at each `clone3` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_clone3_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "clone3");
}

/// A worktree add with the ignored-state copy,
/// killed at each `openat` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_openat_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "openat");
}

/// A worktree add with the ignored-state copy,
/// killed at each `openat2` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_openat2_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "openat2");
}

/// A worktree add with the ignored-state copy,
/// killed at each `ftruncate` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_ftruncate_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "ftruncate");
}

/// A worktree add with the ignored-state copy,
/// killed at each `rmdir` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_rmdir_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "rmdir");
}

/// A worktree add with the ignored-state copy,
/// killed at each `mkdir` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_mkdir_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "mkdir");
}

/// A worktree add with the ignored-state copy,
/// killed at each `chmod` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_chmod_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "chmod");
}

/// A worktree add with the ignored-state copy,
/// killed at each `ioctl` the wrapper makes in it.
#[test]
fn a_worktree_copy_killed_at_each_ioctl_leaves_a_recoverable_repository() {
    sweep_wrapper_kill(&copy_flow(), "ioctl");
}
