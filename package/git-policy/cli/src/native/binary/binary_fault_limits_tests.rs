//! What:
//!  Descriptor,
//! process and address-space limits close to exhaustion,
//! and a single processor,
//! set as resource limits on the
//!  wrapper or on the commit service (`setrlimit` and `sched_setaffinity` between `fork` and `exec`,
//! no privilege),
//! with the
//!  pressure ended afterwards.
//! Why:
//!  The README promises that limits close to exhaustion and a starved host do not strand the repository or leave a command
//!  that never ends.
//! A resource limit is the real mechanism:
//! `RLIMIT_NOFILE` makes every open,
//! pipe and socket fail with
//!  `EMFILE` from some point on,
//! `RLIMIT_NPROC` makes every thread and process start fail with `EAGAIN`,
//! and `RLIMIT_AS`
//!  makes an allocation fail,
//! which ends a Rust program with an abort.
//! The limit is set on one side at a time,
//! because the
//!  wrapper and the service are separate programs with separate limits in production.
//! The pressure ends when a person
//!  closes something else,
//! so each control ends it (a service restarted by the manager,
//! or the limit raised in place) and
//!  requires that the next ordinary command succeeds with nothing left behind (`verify_recovery`).
//!  Sizes are measured,
//! never assumed:
//! the smallest limit under which a program starts and the smallest under which a commit
//!  lands are found by running them,
//! so a change of the build moves the sweep with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const limit of range(startsAt, landsAt + 1)) { await runWithLimit({ nofile: limit }, wrappedCommit); await expectRecoveredRepository(); }
//! ```

use super::commit::remove_settled;
use super::fault::{COMMIT_ARGUMENTS, staged_repository, verify_recovery, verify_recovery_with};
use super::fault_run::{SWEEP_WIDTH, end_the_pressure, run_unsettled, run_watched, unbounded_wrapper};
use super::support::{Fixture, LimitKind, ServiceLaunch, fixture};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The unit of the address-space sweeps:
/// a mebibyte,
/// so that the search for the smallest working size takes few runs.
const MEBIBYTE: u64 = 1 << 20;

/// The most address space the searches try,
/// in mebibytes.
const MOST_MEBIBYTES: u64 = 1 << 16;

/// How many steps a memory sweep takes between its smallest and its largest limit.
const MEMORY_STEPS: u64 = 10;

/// One limit on one side:
/// what,
/// the soft value and the hard value.
type Limit = (LimitKind, u64, u64);

/// A limit that lets a person raise it again,
/// as closing other programs would:
/// the hard limit stays what it was.
fn soft(kind: LimitKind, value: u64) -> Limit {
    return (kind, value, kind.current_hard());
}

/// The wrapper's side of a case:
/// the limits its process (and the Git processes it starts) runs under.
fn limited_commit(fixture: &Fixture, repo: &Path, wrapper: &[Limit]) -> Output {
    let mut command: Command = unbounded_wrapper(fixture, repo, &COMMIT_ARGUMENTS);
    for &(kind, soft_value, hard_value) in wrapper {
        // SAFETY: between fork and exec only `setrlimit`, an async-signal-safe call, runs.
        unsafe {
            command.pre_exec(move || { return kind.apply(soft_value, hard_value); });
        }
    }
    return run_watched(fixture, &mut command);
}

/// What the first run of a limited commit must have shown its caller:
/// it ended (the watchdog did not),
/// as the wrapper (exit 0,
///  1 or 2) or as a process that never started (126 or 127),
/// or,
/// when `may_abort`,
/// by the abort that a failed allocation ends a
///  Rust program with;
/// a failed run says why.
pub(super) fn check_limited_run(label: &str, output: &Output, may_abort: bool) -> Result<(), String> {
    let text: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    if let Some(signal) = output.status.signal() {
        if may_abort && matches!(signal, libc::SIGABRT | libc::SIGSEGV | libc::SIGBUS) {
            return Ok(());
        }
        return Err(format!("{label}: the command was ended by signal {signal}: {text}"));
    }
    let code: i32 = output.status.code().unwrap_or(-1);
    if !matches!(code, 0 | 1 | 2 | 126 | 127) {
        return Err(format!("{label}: unexpected exit {code}: {text}"));
    }
    if code != 0 && text.trim().is_empty() {
        return Err(format!("{label}: a failed commit must say why"));
    }
    return Ok(());
}

/// What one case showed.
struct Case {
    /// Whether the commit landed under the limit.
    landed: bool,
    /// Whether the first run ended with a check failure,
    /// or the recovery after it did not hold.
    verdict: Result<(), String>,
}

/// One commit with `service` limits on the commit service and `wrapper` limits on the wrapper,
/// then the pressure ends (and,
/// when
///  `verify` is set,
/// the repository must recover):
/// the services are stopped (the manager starts a new one at the next command),
/// and the repository must recover.
fn run_case(label: &str, service: &[Limit], wrapper: &[Limit], may_abort: bool, verify: bool) -> Case {
    let fixture: Fixture = fixture(label);
    // A service limited to a size at which it aborts before it listens (the limit falls within a signal stack's size below its
    //  mapped size, so its last thread aborts: measured at 174 MiB on 2026-11-07, and the value moves with every build) is not a
    //  failure of this control: `may_abort` already allows an abort at the limit, the commit then lands through a service that
    //  starts afterwards or does not land, and the recovery after it still runs with an unlimited service.
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch { limits: service.to_vec(), may_fail_to_start: may_abort && !service.is_empty(), ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let output: Output = limited_commit(&fixture, repo.as_path(), wrapper);
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    let first: Result<(), String> = check_limited_run(label, &output, may_abort);
    end_the_pressure(&fixture);
    let judged: Result<(), String> = if verify {
        first.and_then(|()| { return verify_recovery_with(&fixture, repo.as_path(), label, said.as_str(), run_unsettled); })
    } else {
        Ok(())
    };
    let verdict: Result<(), String> = super::fault_run::keeping_the_service_log(&fixture, judged);
    let landed: bool = output.status.code() == Some(0);
    remove_settled(&fixture);
    return Case { landed, verdict };
}

/// The smallest value in `low..=high` for which `works` holds,
/// found by halving;
/// `works` is taken to hold from some value on.
pub(super) fn smallest_working(low: u64, high: u64, works: &dyn Fn(u64) -> bool) -> u64 {
    let (mut below, mut above): (u64, u64) = (low, high);
    while below < above {
        let middle: u64 = below + (above - below) / 2;
        if works(middle) {
            above = middle;
        } else {
            below = middle + 1;
        }
    }
    return above;
}

/// Runs a case for each value,
/// `SWEEP_WIDTH` at a time,
/// and requires that every repository recovered.
/// `limits_for` gives the
///  service's and the wrapper's limits for a value.
/// Returns how many cases landed their commit and how many did not.
fn sweep(label: &str, values: &[u64], limits_for: fn(u64) -> (Vec<Limit>, Vec<Limit>), may_abort: bool) -> (usize, usize) {
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let landed: AtomicUsize = AtomicUsize::new(0);
    let refused: AtomicUsize = AtomicUsize::new(0);
    for batch in values.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for value in batch {
                let collected: &Mutex<Vec<String>> = &failures;
                let (landings, refusals): (&AtomicUsize, &AtomicUsize) = (&landed, &refused);
                scope.spawn(move || {
                    let (service, wrapper): (Vec<Limit>, Vec<Limit>) = limits_for(*value);
                    let case: Case = run_case(format!("{label}-{value}").as_str(), service.as_slice(), wrapper.as_slice(), may_abort, true);
                    if case.landed {
                        landings.fetch_add(1, Ordering::SeqCst);
                    } else {
                        refusals.fetch_add(1, Ordering::SeqCst);
                    }
                    if let Err(message) = case.verdict {
                        collected.lock().expect("failure list").push(message);
                    }
                });
            }
        });
    }
    let found: Vec<String> = failures.into_inner().expect("failure list");
    assert!(found.is_empty(), "{} of {} limits left the repository unrecovered:\n{}", found.len(), values.len(), found.join("\n"));
    return (landed.load(Ordering::SeqCst), refused.load(Ordering::SeqCst));
}

/// Whether a commit lands with this side limited to `value`,
/// for the search for the smallest value that lets it land.
fn lands_with(label: &str, value: u64, limits_for: fn(u64) -> (Vec<Limit>, Vec<Limit>)) -> bool {
    let (service, wrapper): (Vec<Limit>, Vec<Limit>) = limits_for(value);
    return run_case(format!("{label}-probe-{value}").as_str(), service.as_slice(), wrapper.as_slice(), true, false).landed;
}

/// The wrapper keeps `value` descriptors.
fn wrapper_descriptors(value: u64) -> (Vec<Limit>, Vec<Limit>) {
    return (Vec::new(), vec![soft(LimitKind::OpenFiles, value)]);
}

/// The service keeps `value` descriptors.
fn service_descriptors(value: u64) -> (Vec<Limit>, Vec<Limit>) {
    return (vec![soft(LimitKind::OpenFiles, value)], Vec::new());
}

/// The wrapper has `value` mebibytes of address space.
fn wrapper_memory(value: u64) -> (Vec<Limit>, Vec<Limit>) {
    return (Vec::new(), vec![soft(LimitKind::AddressSpace, value * MEBIBYTE)]);
}

/// The service has `value` mebibytes of address space.
fn service_memory(value: u64) -> (Vec<Limit>, Vec<Limit>) {
    return (vec![soft(LimitKind::AddressSpace, value * MEBIBYTE)], Vec::new());
}

/// How long a service started under a limit may take to greet a client before it counts as not starting.
const START_PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

/// How long one read of a service's greeting waits before the service is polled again.
const GREETING_READ: std::time::Duration = std::time::Duration::from_secs(1);

/// What one start of the commit service under a limit showed.
struct StartOutcome {
    /// Whether a client was greeted:
    /// the service bound its socket,
    /// started its acceptor and answered.
    greeted: bool,
    /// What the service wrote to its log before it ended or was stopped.
    log: String,
}

/// What:
///  Starts the commit service under `limit` of `kind` (bytes for the address space),
/// and reports whether it greeted a client.
/// Why:
///   A service that cannot start its acceptor thread still binds its socket first,
/// and then exits within a millisecond,
/// so
///  its socket accepts a connection only in that moment.
/// A probe that counted any connection as starting gave an answer
///  that depended on whether the poll landed in that moment,
/// which a loaded host changes:
/// the sweep then began below the
///  limits at which a service can run,
/// and the fixture waited for services that had ended.
/// Only a greeting proves the
///  service runs,
/// and a service that ended without one does not start at this limit.
fn start_service_under(kind: LimitKind, limit: u64, directory: &Path) -> StartOutcome {
    let name: String = format!("cli-git-limit-probe-{}-{limit}-{}", std::process::id(), START_COUNTER.fetch_add(1, Ordering::SeqCst));
    let journal: PathBuf = directory.join(name.as_str());
    let mut command: Command = super::support::service_command();
    command.args(["--journal"]).arg(&journal).args(["--abstract", name.as_str()]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    let hard: u64 = kind.current_hard();
    // SAFETY: between fork and exec only `setrlimit`, an async-signal-safe call, runs.
    unsafe {
        command.pre_exec(move || { return kind.apply(limit, hard); });
    }
    // Owned: the service's idle exit is hours away, and the loop below can end by a failed expectation before the kill.
    let mut child = super::support::spawn_owned(&mut command).expect("the service program starts");
    let began = std::time::Instant::now();
    let greeted: bool = loop {
        if child.try_wait().expect("the service's state").is_some() {
            break false;
        }
        let reached = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes())
            .and_then(|address| { return std::os::unix::net::UnixStream::connect_addr(&address); });
        if let Ok(stream) = reached {
            let _timeout = stream.set_read_timeout(Some(GREETING_READ));
            let mut line: String = String::new();
            // An empty read is a connection that ended without a greeting: the service went away as it was starting.
            if std::io::BufRead::read_line(&mut std::io::BufReader::new(stream), &mut line).is_ok_and(|read| { return read > 0; }) {
                break true;
            }
        }
        if began.elapsed() > START_PATIENCE {
            break false;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let _killed = child.kill();
    let _waited = child.wait();
    let mut log: String = String::new();
    if let Some(mut stream) = child.stderr.take() {
        let _read = std::io::Read::read_to_string(&mut stream, &mut log);
    }
    let _removed = std::fs::remove_dir_all(&journal);
    return StartOutcome { greeted, log };
}

/// Numbers the starts of one test process,
/// so that no two services share an abstract socket name.
static START_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// The smallest value of `kind` (mebibytes for the address space) under which the commit service starts and greets a client:
/// the sweep of the service's limits begins
///  here,
/// because a service that cannot greet has no commit to fail (the manager reports it as a failed unit).
fn smallest_service_limit(kind: LimitKind, high: u64) -> u64 {
    let probes: Fixture = fixture("limit-probe");
    let starts = |limit: u64| -> bool {
        let applied: u64 = if kind == LimitKind::AddressSpace { limit * MEBIBYTE } else { limit };
        return start_service_under(kind, applied, probes.root.as_path()).greeted;
    };
    let smallest: u64 = smallest_working(3, high, &starts);
    remove_settled(&probes);
    return smallest;
}

/// `count` values from `low` to `high`,
/// evenly spaced,
/// both ends included,
/// each once.
fn spread(low: u64, high: u64, count: u64) -> Vec<u64> {
    let mut values: Vec<u64> = (0..=count).map(|step| { return low + (high - low) * step / count; }).collect();
    values.dedup();
    return values;
}

/// A descriptor limit of each size on the wrapper,
/// from the smallest under which a commit lands downward:
/// the commit fails with a
///  message or lands,
/// and the repository recovers either way.
#[test]
fn a_descriptor_limit_close_to_exhaustion_on_the_wrapper_leaves_a_recoverable_repository() {
    let lands_at: u64 = smallest_working(3, 1024, &|value| { return lands_with("nofile-wrapper", value, wrapper_descriptors); });
    let values: Vec<u64> = (3..=lands_at).collect();
    let (landed, refused): (usize, usize) = sweep("nofile-wrapper", values.as_slice(), wrapper_descriptors, false);
    assert!(refused > 0, "no descriptor limit stopped a commit, so the limits tested nothing (smallest landing limit {lands_at})");
    assert!(landed > 0, "no descriptor limit let the commit land, so none was close to exhaustion");
}

/// A descriptor limit of each size on the commit service,
/// from the smallest under which it starts,
/// up to the smallest under which a
///  commit lands:
/// the commit fails with a message or lands,
/// and the repository recovers either way.
#[test]
fn a_descriptor_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository() {
    let starts_at: u64 = smallest_service_limit(LimitKind::OpenFiles, 1024);
    let lands_at: u64 = smallest_working(starts_at, 1024, &|value| { return lands_with("nofile-service", value, service_descriptors); });
    let values: Vec<u64> = (starts_at..=lands_at).collect();
    let (landed, refused): (usize, usize) = sweep("nofile-service", values.as_slice(), service_descriptors, false);
    assert!(refused > 0, "no descriptor limit stopped a commit, so the limits tested nothing (service starts at {starts_at}, commit lands at {lands_at})");
    assert!(landed > 0, "no descriptor limit let the commit land, so none was close to exhaustion");
}

/// Whether a process limit makes the next start fail for this user (a privileged test run is exempt from `RLIMIT_NPROC`),
/// so
///  the controls that need the limit to bite can say they did not run instead of passing without testing.
/// The probe is a shell
///  that must start a second process:
/// with the limit at one it cannot.
fn process_limit_bites() -> bool {
    let mut command: Command = Command::new("/bin/sh");
    command.args(["-c", "/bin/true && echo started"]).stdin(Stdio::null()).stderr(Stdio::null());
    // SAFETY: between fork and exec only `setrlimit`, an async-signal-safe call, runs.
    unsafe {
        command.pre_exec(|| { return LimitKind::Processes.apply(1, LimitKind::Processes.current_hard()); });
    }
    let output: Output = command.output().expect("the probe shell starts");
    return !String::from_utf8_lossy(output.stdout.as_slice()).contains("started");
}

/// A process limit that is exhausted for the commit service:
/// the commit fails with a message and ends;
/// once the limit is raised
///  in place (the service is not restarted),
/// the next commit lands through the same service and nothing is left behind.
#[test]
fn a_commit_service_at_its_process_limit_fails_plainly_and_serves_again_when_it_is_raised() {
    if !process_limit_bites() {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no start was refused");
        return;
    }
    let fixture: Fixture = fixture("nproc-service");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    // The service starts first and meets the limit while it serves: a service that cannot start a thread at all has nothing to
    //  answer with, and the manager reports it as a failed unit.
    super::support::service_runtime(&fixture);
    let services: Vec<u32> = fixture.service.lock().expect("the fixture's service lock").as_ref().map(|started| { return started.process_ids(); }).unwrap_or_default();
    assert!(!services.is_empty(), "a service ran");
    let hard: u64 = LimitKind::Processes.current_hard();
    for pid in &services {
        LimitKind::Processes.lift_for(*pid, 1, hard).expect("a soft limit is lowered");
    }
    let output: Output = limited_commit(&fixture, repo.as_path(), &[]);
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    if let Err(reason) = check_limited_run("nproc-service", &output, false) {
        panic!("{}", super::fault_run::with_service_log(&fixture, reason));
    }
    assert_ne!(
        output.status.code(),
        Some(0),
        "{}",
        super::fault_run::with_service_log(&fixture, format!("the service could start no thread or process, so the commit cannot have landed: {said}"))
    );
    for pid in &services {
        LimitKind::Processes.lift_for(*pid, hard, hard).expect("the soft limit is raised up to the hard limit");
    }
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch::default();
    if let Err(reason) = verify_recovery(&fixture, repo.as_path(), "nproc-service", said.as_str()) {
        panic!("{}", super::fault_run::with_service_log(&fixture, reason));
    }
    remove_settled(&fixture);
}

/// A failure of these controls carries what the service itself logged.
/// Why:
///  The recorded failure of 2026-11-07 was a service that could not start the thread for its after-commit work;
/// its line
///  names the error and the exit that followed,
/// so a failure that keeps it is settled without a rerun,
/// and the sweep runs
///  several cases at once so the inherited stream alone says nothing about which case wrote it.
#[test]
fn a_failure_carries_what_the_fixture_service_logged() {
    let fixture: Fixture = fixture("service-log");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let _runtime: Option<PathBuf> = super::support::service_runtime(&fixture);
    let message: String = super::fault_run::with_service_log(&fixture, String::from("the commit did not land"));
    assert!(message.starts_with("the commit did not land"), "{message}");
    assert!(message.contains("ready"), "the log of the service that started is carried: {message}");
    remove_settled(&fixture);
    drop(repo);
}

/// A process limit that is exhausted for the wrapper,
/// which then cannot start any Git:
/// the commit fails with a message and ends,
///  and the next command,
/// once the limit is gone,
/// succeeds.
#[test]
fn a_wrapper_at_its_process_limit_fails_plainly_and_leaves_a_recoverable_repository() {
    if !process_limit_bites() {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no start was refused");
        return;
    }
    let case: Case = run_case("nproc-wrapper", &[], &[soft(LimitKind::Processes, 1)], false, true);
    assert_eq!(case.verdict, Ok(()));
    assert!(!case.landed, "the wrapper could start no process, so the commit cannot have landed through it");
}

/// An address space of each size on the wrapper,
/// from the smallest under which a commit lands downward (sizes in mebibytes):
/// a failed allocation ends
///  a Rust program with an abort,
/// whatever it was doing,
/// and the repository recovers after each.
#[test]
fn a_memory_limit_close_to_exhaustion_on_the_wrapper_leaves_a_recoverable_repository() {
    let lands_at: u64 = smallest_working(1, MOST_MEBIBYTES, &|value| { return lands_with("as-wrapper", value, wrapper_memory); });
    let values: Vec<u64> = spread(1, lands_at, MEMORY_STEPS);
    let (landed, refused): (usize, usize) = sweep("as-wrapper", values.as_slice(), wrapper_memory, true);
    assert!(refused > 0, "no address-space limit stopped a commit, so the limits tested nothing (smallest landing limit {lands_at})");
    assert!(landed > 0, "no address-space limit let the commit land, so none was close to exhaustion");
}

/// The address space process `pid` has mapped now,
/// in mebibytes rounded up,
/// read from its status (`VmSize`).
fn mapped_mebibytes(pid: u32) -> u64 {
    let status: String = std::fs::read_to_string(format!("/proc/{pid}/status")).expect("the status of the service");
    let line: &str = status.lines().find(|line| return line.starts_with("VmSize:")).expect("a VmSize line");
    let kibibytes: u64 = line.trim_start_matches("VmSize:").trim().trim_end_matches("kB").trim().parse().expect("a size");
    return kibibytes.div_ceil(1024);
}

/// One commit through a service that has started and greeted,
/// whose address-space limit is then lowered in place to `step` of `MEMORY_STEPS` parts of its mapped size plus the reserve the
/// runtime keeps back,
/// so the allocation that fails falls in the middle of the landing whatever the size of the build
/// (the service's own heap is mostly free by then,
/// so the first limits that bite are the ones under which the Git programs it starts for the landing cannot map their own space);
/// then the pressure ends and the repository must recover.
fn run_case_lowered(label: &str, step: u64) -> Case {
    let fixture: Fixture = fixture(label);
    let repo: PathBuf = staged_repository(&fixture, "repo");
    // The service starts first and meets the limit while it serves: a limit set before it starts moves the instant it ends
    //  with the size of the build (it aborts before it listens when the limit falls within a signal stack of its mapped size).
    super::support::service_runtime(&fixture);
    let services: Vec<u32> = fixture.service.lock().expect("the fixture's service lock").as_ref().map(|started| { return started.process_ids(); }).unwrap_or_default();
    assert!(!services.is_empty(), "a service ran");
    let hard: u64 = LimitKind::AddressSpace.current_hard();
    for pid in &services {
        let reserve_mebibytes: u64 = git_policy_cli::runtime_plan::ADDRESS_SPACE_RESERVE_BYTES / MEBIBYTE;
        let limit: u64 = (mapped_mebibytes(*pid) + reserve_mebibytes) * step / MEMORY_STEPS * MEBIBYTE;
        LimitKind::AddressSpace.lift_for(*pid, limit, hard).expect("the soft limit of a running service is lowered");
    }
    let output: Output = limited_commit(&fixture, repo.as_path(), &[]);
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    let first: Result<(), String> = check_limited_run(label, &output, false);
    end_the_pressure(&fixture);
    let judged: Result<(), String> = first.and_then(|()| { return verify_recovery_with(&fixture, repo.as_path(), label, said.as_str(), run_unsettled); });
    let verdict: Result<(), String> = super::fault_run::keeping_the_service_log(&fixture, judged);
    let landed: bool = output.status.code() == Some(0);
    remove_settled(&fixture);
    return Case { landed, verdict };
}

/// A running service whose address-space limit is lowered in place to each of `MEMORY_STEPS` parts of its mapped size plus the
/// reserve the runtime keeps back,
/// and one commit through it each:
/// the repository recovers after every one,
/// some limit stops the commit and some lets it land.
/// Why:
///  The runtime reserves its address space when the service starts (`ADDRESS_SPACE_RESERVE_BYTES`),
/// so no limit set before the start leaves a service that starts and cannot land a commit;
/// real exhaustion (the control group's `memory.max`,
/// the host's pressure) arrives later and the reserve protects nothing there,
/// so an allocation that fails in the middle of a landing is reached by lowering the limit of the service that is running.
#[test]
fn a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository() {
    let headrooms: Vec<u64> = (0..=MEMORY_STEPS).collect();
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let landed_count: AtomicUsize = AtomicUsize::new(0);
    let refused_count: AtomicUsize = AtomicUsize::new(0);
    for batch in headrooms.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for headroom in batch {
                let (collected, landings, refusals): (&Mutex<Vec<String>>, &AtomicUsize, &AtomicUsize) = (&failures, &landed_count, &refused_count);
                scope.spawn(move || {
                    let case: Case = run_case_lowered(format!("as-service-lowered-{headroom}").as_str(), *headroom);
                    if case.landed {
                        landings.fetch_add(1, Ordering::SeqCst);
                    } else {
                        refusals.fetch_add(1, Ordering::SeqCst);
                    }
                    if let Err(message) = case.verdict {
                        collected.lock().expect("failure list").push(message);
                    }
                });
            }
        });
    }
    let found: Vec<String> = failures.into_inner().expect("failure list");
    assert!(found.is_empty(), "{} of {} limits left the repository unrecovered:\n{}", found.len(), headrooms.len(), found.join("\n"));
    let (landed, refused): (usize, usize) = (landed_count.load(Ordering::SeqCst), refused_count.load(Ordering::SeqCst));
    assert!(refused > 0, "no limit stopped a commit, so the limits tested nothing (steps {headrooms:?})");
    assert!(landed > 0, "no limit let the commit land, so none was close to exhaustion (steps {headrooms:?})");
}

/// A service whose address-space limit is below the smallest under which it greets a client ends before it listens:
/// no commit
///  lands,
/// and the repository recovers with a fresh service that has no limit.
/// Why:
///  The sweep above meets this case by chance at the one limit just under the service's mapped size,
/// which moves with every build
///  (174 MiB for the build of 2026-11-07);
/// this control meets it at every build.
#[test]
fn a_service_that_ends_before_it_listens_leaves_a_recoverable_repository() {
    let starts_at: u64 = smallest_service_limit(LimitKind::AddressSpace, MOST_MEBIBYTES);
    let (service, wrapper): (Vec<Limit>, Vec<Limit>) = service_memory(starts_at - 1);
    let case: Case = run_case("as-service-below-start", service.as_slice(), wrapper.as_slice(), true, true);
    assert!(!case.landed, "a service under {} MiB cannot run, so no commit can have landed through it", starts_at - 1);
    assert_eq!(case.verdict, Ok(()), "the repository must recover with an unlimited service");
}

/// How many times the service is started under a limit it cannot run under,
/// for the test that it is never counted as starting.
const BAND_STARTS: usize = 100;

/// What:
///  A limit under which the service binds its socket but cannot start its acceptor is never counted as one under which it
///  starts,
/// even when the poll and the service share one processor.
/// Why:
///   The service listens for about a millisecond before it exits there.
/// On one processor the poll lands in that moment
///  often (a few starts in a hundred on the measured host,
/// none on an idle many-processor host),
/// so a probe that counted a
///  connection as starting began the sweep below the working limits on a loaded host,
/// and the fixture then waited a minute
///  for each service that had already ended.
#[test]
fn a_limit_that_lets_the_service_bind_but_not_run_is_never_counted_as_starting() {
    let starts_at: u64 = smallest_service_limit(LimitKind::AddressSpace, MOST_MEBIBYTES);
    let below: u64 = (starts_at - 1) * MEBIBYTE;
    let probes: Fixture = fixture("limit-band");
    let outcomes: Vec<StartOutcome> = std::thread::scope(|scope| {
        return scope
            .spawn(|| {
                super::support::restrict_to_one_cpu().expect("the poll and the services share one processor");
                return (0..BAND_STARTS).map(|_| { return start_service_under(LimitKind::AddressSpace, below, probes.root.as_path()); }).collect::<Vec<StartOutcome>>();
            })
            .join()
            .expect("the pinned thread ends");
    });
    remove_settled(&probes);
    for outcome in &outcomes {
        assert!(outcome.log.contains("ready"), "the service must bind before this limit stops it, or the limit tests nothing: {}", outcome.log);
        assert!(!outcome.greeted, "a service that ended without running was counted as starting at {below} bytes: {}", outcome.log);
    }
}

/// The wrapper,
/// the service and every Git they start share one processor,
/// as on a host with no other free capacity:
/// the commit
///  lands,
/// the next one lands,
/// and nothing is left behind.
/// The processes then wait for each other instead of running side by
///  side,
/// which is where an assumption about timing between them shows.
#[test]
fn a_commit_on_a_single_processor_lands_and_leaves_nothing() {
    let fixture: Fixture = fixture("single-cpu");
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch { single_cpu: true, ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let mut command: Command = unbounded_wrapper(&fixture, repo.as_path(), &COMMIT_ARGUMENTS);
    // SAFETY: the closure calls only `sched_getaffinity` and `sched_setaffinity`.
    unsafe {
        command.pre_exec(super::support::restrict_to_one_cpu);
    }
    let output: Output = run_watched(&fixture, &mut command);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(output.stderr.as_slice()));
    assert_eq!(verify_recovery(&fixture, repo.as_path(), "single-cpu", ""), Ok(()));
    remove_settled(&fixture);
}

/// What:
///  A service whose after-commit work thread is refused says which limit refused it before it ends.
/// Why:
///  The service that cannot start the thread for its after-commit work answers no commit,
/// so it ends and the manager starts another.
///  What it says is all a person has,
/// and a line that names only the system's error sends them looking for a defect of this program.
///  The limit is real (`RLIMIT_NPROC` on the service),
/// not armed by a control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { greeted, log } = await startServiceUnder('processes', 1, directory);
/// expect(greeted).toBe(false); expect(log).toContain('ulimit -u');
/// ```
#[test]
fn a_service_whose_work_thread_is_refused_says_which_limit_refused_it() {
    if !process_limit_bites() {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no start was refused");
        return;
    }
    let probes: Fixture = fixture("nproc-service-work-thread");
    let outcome: StartOutcome = start_service_under(LimitKind::Processes, 1, probes.root.as_path());
    assert!(!outcome.greeted, "a service that can start no thread greets no client: {}", outcome.log);
    assert!(
        outcome.log.contains("ulimit -u") && outcome.log.contains("pids.max"),
        "the line the service ends with names the limits that refused the thread: {}",
        outcome.log
    );
    remove_settled(&probes);
}
