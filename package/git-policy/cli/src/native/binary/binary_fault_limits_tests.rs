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

use super::commit::{git_text, remove_settled};
use super::fault::{COMMIT_ARGUMENTS, staged_repository, verify_recovery, verify_recovery_with};
use super::fault_run::{SWEEP_WIDTH, end_the_pressure, run_unsettled, run_watched, unbounded_wrapper};
use super::support::{Fixture, FixtureService, LimitKind, ServiceEnd, ServiceLaunch, fixture};
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

/// How long a service may take to end by itself, for the control whose subject is that end.
/// Why:
///   The service ends within a millisecond of binding where it has no worker to accept on (measured on this host),
/// so this bound is thousands of those;
/// a service that still runs when it passes is one the control refuses as never having ended.
const SERVICE_END_BOUND: std::time::Duration = std::time::Duration::from_secs(10);

/// The largest descriptor limit the searches for the smallest landing limit try,
/// so that a machine on which no commit lands at any limit the control examines is told apart from one whose flip is
///  below that ceiling,
/// and a flip at the ceiling itself from both:
/// a landing there is a landing the search proves,
/// and a case per limit below it would outlast the control.
const FURTHEST_DESCRIPTOR_LIMIT_SEARCHED: u64 = 1024;

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

/// What one case showed,
/// with the two facts apart:
/// what the wrapper did,
/// and what the repository holds.
struct Case {
    /// Whether the limited attempt's commit landed:
    /// the repository's branch holds one commit more than it did before the attempt,
    /// which is the repository's own state and neither the wrapper's exit nor a line a service wrote.
    landed: bool,
    /// Whether the wrapper returned success under the limit:
    /// its own exit status,
    /// and never whether a commit landed.
    returned_success: bool,
    /// The commits the repository's branch held before the limited attempt.
    commits_before: String,
    /// The commits it held after the attempt,
    /// read before the recovery ran,
    /// so the recovery's own commit is not counted here.
    commits_after: String,
    /// How the wrapper itself ended under the limit,
    /// as an exit status or a signal,
    /// for a failure that must say what happened rather than what it means.
    status: std::process::ExitStatus,
    /// What the wrapper wrote on its standard error under the limit,
    /// which is where it says why it did not return success.
    stderr: String,
    /// Whether the first run ended with a check failure,
    /// or the recovery after it did not hold.
    verdict: Result<(), String>,
    /// What the limited attempt's service wrote,
    /// read after the pressure ended and before the recovery ran,
    /// so it holds nothing the recovery's service wrote.
    limited_log: String,
    /// What the service wrote while the recovery ran:
    /// the log after the recovery, less what the limited attempt's service had written.
    recovery_log: String,
    /// How the limited service ended,
    /// for the control that says it ended,
    /// and `None` for every control that does not watch for it.
    service_end: Option<ServiceEnd>,
}

/// One commit with `service` limits on the commit service and `wrapper` limits on the wrapper,
/// then the pressure ends (and,
/// when
///  `verify` is set,
/// the repository must recover):
/// the services are stopped (the manager starts a new one at the next command),
/// and the repository must recover.
fn run_case(label: &str, service: &[Limit], wrapper: &[Limit], may_abort: bool, verify: bool, watched_end: Option<std::time::Duration>) -> Case {
    let fixture: Fixture = fixture(label);
    // A service limited to a size at which it aborts before it listens (the limit falls within a signal stack's size below its
    //  mapped size, so its last thread aborts: measured at 174 MiB on 2026-11-07, and the value moves with every build) is not a
    //  failure of this control: `may_abort` already allows an abort at the limit, the commit then lands through a service that
    //  starts afterwards or does not land, and the recovery after it still runs with an unlimited service.
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch { limits: service.to_vec(), may_fail_to_start: may_abort && !service.is_empty(), ..ServiceLaunch::default() };
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let commits_before: String = commits_of_the_branch(&fixture, repo.as_path());
    let output: Output = limited_commit(&fixture, repo.as_path(), wrapper);
    // The end is read from the process itself, before the pressure ends: what the service wrote about its own end
    //  arrives through a copy thread and says only what it claimed.
    let service_end: Option<ServiceEnd> =
        watched_end.map(|bound| { return super::support::wait_for_the_service_to_end(&fixture, repo.as_path(), bound); });
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    let first: Result<(), String> = check_limited_run(label, &output, may_abort);
    // The pressure ends here, which closes every service's stream; the limited attempt's words are then complete, and
    //  they are read now, because the recovery adds its own lines to the same file and its commit is not this case's.
    end_the_pressure(&fixture);
    let limited_log: String = the_service_log_of(&fixture);
    let commits_after: String = commits_of_the_branch(&fixture, repo.as_path());
    let judged: Result<(), String> = if verify {
        first.and_then(|()| { return verify_recovery_with(&fixture, repo.as_path(), label, said.as_str(), run_unsettled); })
    } else {
        Ok(())
    };
    let verdict: Result<(), String> = super::fault_run::keeping_the_service_log(&fixture, judged);
    let recovery_log: String = the_log_of_the_recovery(&limited_log, &the_service_log_of(&fixture));
    let status: std::process::ExitStatus = output.status;
    let stderr: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    remove_settled(&fixture);
    return Case {
        landed: commits_before != commits_after,
        returned_success: output.status.code() == Some(0),
        commits_before,
        commits_after,
        status,
        stderr,
        verdict,
        limited_log,
        recovery_log,
        service_end,
    };
}

/// What:
///  The commits the repository's checked-out branch holds now,
///  which is the repository's own state and the fact "a commit landed".
/// Why:
///  The wrapper's exit says whether the wrapper returned success,
/// and a service's `answered` line is what the service says it did;
/// the branch is neither of those,
/// so a control whose claim is about a commit reads this.
fn commits_of_the_branch(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["rev-list", "--count", "HEAD"]).trim().to_string();
}

/// What:
///  What the fixture's service log holds now, as the file has it:
///  evidence for a person reading why the case failed,
///  and never what a verdict rests on.
/// Why:
///  A copy runs on a thread of its own, so the file holds what a service wrote only once that thread has written it,
/// and the end of the copy is an event this control does not need to wait for when it decides nothing.
///  Every service of a fixture appends to this one file,
/// so a read made before the recovery runs holds the limited attempt's words alone,
/// and a read made after it holds both,
/// which `the_log_of_the_recovery` tells apart by this text rather than by a byte count that changes between the reads.
fn the_service_log_of(fixture: &Fixture) -> String {
    return std::fs::read_to_string(fixture.root.join(super::support::SERVICE_LOG_FILE)).unwrap_or_default();
}

/// What:
///  The part of the fixture's service log that the service of the recovery run wrote:
///  the log after the recovery, less what the limited attempt's service had written.
/// Why:
///  Both services append to one file,
/// so the recovery's successful commit would otherwise be read as the limited attempt's outcome,
/// which two reviews of this control mistook once already.
fn the_log_of_the_recovery(limited: &str, whole: &str) -> String {
    let Some(added) = whole.strip_prefix(limited) else {
        return format!("(the recovery's part of the log cannot be told apart from the limited attempt's; the whole log was {whole})");
    };
    return added.to_string();
}

/// What:
///  The smallest value in `low..=high` for which `works` holds,
///  found by halving and proven by a probe of that value,
///  or nothing when `works` holds for none of them.
/// Why:
///  `works` is taken to hold from some value on,
/// so the halving finds where it starts;
///  the ceiling the halving starts from is only ever lowered by a probe that held,
///  so an answer that is still the ceiling proves nothing,
///  and a caller that read it as a landing said no commit landed while one did exactly there.
///  The answer is therefore probed before it is returned,
///  and a range in which nothing works is answered as nothing rather than as its ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const smallest: number | undefined = smallestWorking(3, 1024, (limit) => lands(limit));
/// ```
pub(super) fn smallest_working(low: u64, high: u64, works: &dyn Fn(u64) -> bool) -> Option<u64> {
    let (mut below, mut above): (u64, u64) = (low, high);
    // Whether a probe of `above` held: the halving only lowers it after one did.
    let mut proven: bool = false;
    while below < above {
        let middle: u64 = below + (above - below) / 2;
        if works(middle) {
            above = middle;
            proven = true;
        } else {
            below = middle + 1;
        }
    }
    if proven || works(above) {
        return Some(above);
    }
    return None;
}

/// What:
///  The search answers only a limit its own probe accepted,
///  and never the ceiling it started from without a probe of that ceiling.
/// Why:
///  The halving begins with the ceiling as its upper end and lowers it only on a probe that held,
///  so an answer that is still the ceiling was never probed:
///  where a commit lands at the ceiling and nowhere below it the descriptor control then said no commit landed,
///  and it skipped the sweep that landing at the ceiling would have had.
#[test]
fn the_search_answers_only_a_limit_its_own_probe_accepted() {
    let probed: Mutex<Vec<u64>> = Mutex::new(Vec::new());
    let records_and_refuses = |value: u64| -> bool {
        probed.lock().expect("the probe list").push(value);
        return false;
    };
    let answered: Option<u64> = smallest_working(3, FURTHEST_DESCRIPTOR_LIMIT_SEARCHED, &records_and_refuses);
    assert_eq!(answered, None, "no limit in the search's range let a commit land, so the search answers nothing");
    assert!(
        probed.lock().expect("the probe list").contains(&FURTHEST_DESCRIPTOR_LIMIT_SEARCHED),
        "the search says nothing in its range worked, so it must have probed its ceiling"
    );
    // A commit that lands at the ceiling and nowhere below it is a landing, and a control is told so.
    let only_the_ceiling = |value: u64| -> bool { return value == FURTHEST_DESCRIPTOR_LIMIT_SEARCHED; };
    assert_eq!(
        smallest_working(3, FURTHEST_DESCRIPTOR_LIMIT_SEARCHED, &only_the_ceiling),
        Some(FURTHEST_DESCRIPTOR_LIMIT_SEARCHED),
        "the search answers a landing at its own ceiling instead of saying no commit landed"
    );
    // A landing below the ceiling is answered where it starts, not at the ceiling.
    let half_upwards = |value: u64| -> bool { return value >= FURTHEST_DESCRIPTOR_LIMIT_SEARCHED / 2; };
    assert_eq!(
        smallest_working(3, FURTHEST_DESCRIPTOR_LIMIT_SEARCHED, &half_upwards),
        Some(FURTHEST_DESCRIPTOR_LIMIT_SEARCHED / 2),
        "the search answers the smallest limit that worked"
    );
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
                    let case: Case = run_case(format!("{label}-{value}").as_str(), service.as_slice(), wrapper.as_slice(), may_abort, true, None);
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
    return run_case(format!("{label}-probe-{value}").as_str(), service.as_slice(), wrapper.as_slice(), true, false, None).landed;
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
    let found: Option<u64> = smallest_working(3, high, &starts);
    remove_settled(&probes);
    let Some(smallest): Option<u64> = found else {
        panic!("the commit service started and greeted no client at any {kind:?} limit from 3 to {high} that the search probed, so this control has no limit to sweep from");
    };
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

/// What:
///  The limits from `low` to the smallest under which a commit lands,
///  each run once with the sweep's own breadth,
///  and one case on its own at twice that smallest landing limit,
///  with the recovery checked.
/// Why:
///  The smallest limit under which a commit lands is the limit at which the outcome flips,
///  and the flip is one descriptor wide:
///  measured on a loaded host on 2026-11-11,
///  a commit run one at a time refused at 23 twice and landed at 24 twice.
///  The flip moves with everything else that holds a descriptor while a commit runs,
///  because a step that waits holding one keeps it held into the next step,
///  so the search that finds it can answer a limit that the next case refuses,
///  and `SWEEP_WIDTH` cases at once move it further still.
///  The landing this control needs is therefore asked for at twice the limit the search landed at,
///  where a commit needs the same descriptors it needed at the boundary and the limit leaves as much room again,
///  which is a landing rather than the boundary's coin flip:
///  the caller counts it with the batch's landings,
///  so it supplies the landing exactly when the batch refused every limit.
///  The batch proves the rest of the promise:
///  every limit fails with a message or lands,
///  and the repository recovers after each.
///  Twice a figure the search measured is the only margin this control states.
fn sweep_up_to_the_smallest_landing(label: &str, low: u64, lands_at: u64, limits_for: fn(u64) -> (Vec<Limit>, Vec<Limit>)) -> (usize, usize) {
    let values: Vec<u64> = (low..=lands_at).collect();
    let (landed, refused): (usize, usize) = sweep(label, values.as_slice(), limits_for, false);
    let (service, wrapper): (Vec<Limit>, Vec<Limit>) = limits_for(lands_at.saturating_mul(2));
    let alone: Case = run_case(format!("{label}-twice-the-smallest-landing-{lands_at}").as_str(), service.as_slice(), wrapper.as_slice(), false, true, None);
    assert!(alone.verdict.is_ok(), "twice the smallest landing limit left the repository unrecovered: {:?}", alone.verdict);
    return (landed + usize::from(alone.landed), refused + usize::from(!alone.landed));
}

/// What:
///  The smallest descriptor limit under which a commit lands,
///  when the search proved one below its own ceiling.
/// Why:
///  The sweep runs one case per limit from the floor to that landing,
///  so a landing at the ceiling would run a case per descriptor the search examined
///  and outlast the five minutes this crate's nextest profile gives one test
///  (`.config/nextest.toml`).
///  Measured on this host on 2026-11-11:
///  `a_descriptor_limit_close_to_exhaustion_on_the_wrapper_leaves_a_recoverable_repository`
///  ran its 23 cases in 72 s alone and in 26.8 s beside the module's own tests,
///  so a case per descriptor up to 1024 would need between about 1 200 s and 3 000 s,
///  four to ten times that bound.
///  A landing at the ceiling is a fact of the machine and not a defect of the product,
///  so the control says it did not run instead of failing or claiming no commit landed
///  (`FURTHEST_DESCRIPTOR_LIMIT_SEARCHED`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landsAt = smallestLandingBelowTheCeiling('nofile-wrapper', 3, (limit) => lands(limit)); // number | undefined
/// ```
fn smallest_landing_below_the_ceiling(label: &str, low: u64, probe: &dyn Fn(u64) -> bool) -> Option<u64> {
    let found: Option<u64> = smallest_working(low, FURTHEST_DESCRIPTOR_LIMIT_SEARCHED, probe);
    let Some(lands_at): Option<u64> = found else {
        panic!(
            "a commit landed at none of the descriptor limits {low} to {FURTHEST_DESCRIPTOR_LIMIT_SEARCHED} that the search probed, so this control has no landing to be close to exhaustion"
        );
    };
    if lands_at < FURTHEST_DESCRIPTOR_LIMIT_SEARCHED {
        return Some(lands_at);
    }
    eprintln!(
        "control not run: a commit lands at {label}'s search ceiling {lands_at} and at no lower limit the search probed, so one case per limit below it would outlast this control; raise FURTHEST_DESCRIPTOR_LIMIT_SEARCHED to sweep it"
    );
    return None;
}

/// What:
///  The landing search answers a landing below its ceiling,
///  and answers nothing when the landing is the ceiling itself.
/// Why:
///  A landing at the ceiling is the branch that says the control did not run,
///  and nothing else would ever run it:
///  no machine this control has run on lands a commit anywhere near 1024 descriptors
///  (`FURTHEST_DESCRIPTOR_LIMIT_SEARCHED`).
#[test]
fn the_landing_search_answers_below_the_ceiling_and_says_so_at_it() {
    let halfway = |value: u64| -> bool { return value >= FURTHEST_DESCRIPTOR_LIMIT_SEARCHED / 2; };
    assert_eq!(
        smallest_landing_below_the_ceiling("control-probe-below", 3, &halfway),
        Some(FURTHEST_DESCRIPTOR_LIMIT_SEARCHED / 2),
        "a landing below the search's ceiling is the landing the sweep examines"
    );
    let at_the_ceiling = |value: u64| -> bool { return value >= FURTHEST_DESCRIPTOR_LIMIT_SEARCHED; };
    assert_eq!(
        smallest_landing_below_the_ceiling("control-probe-ceiling", 3, &at_the_ceiling),
        None,
        "a landing at the ceiling itself is not the landing the sweep can examine"
    );
}

/// What:
///  A range in which no limit lands fails the control,
///  naming the probed fact rather than claiming no commit landed under a limit nothing probed.
#[test]
#[should_panic(expected = "a commit landed at none of the descriptor limits")]
fn the_landing_search_names_a_machine_where_no_limit_lands() {
    let nothing = |_value: u64| -> bool { return false; };
    let _answered: Option<u64> = smallest_landing_below_the_ceiling("control-probe-none", 3, &nothing);
}

/// A descriptor limit of each size on the wrapper,
/// from the smallest under which a commit lands downward:
/// the commit fails with a
///  message or lands,
/// and the repository recovers either way.
#[test]
fn a_descriptor_limit_close_to_exhaustion_on_the_wrapper_leaves_a_recoverable_repository() {
    let Some(lands_at): Option<u64> = smallest_landing_below_the_ceiling("nofile-wrapper", 3, &|value| { return lands_with("nofile-wrapper", value, wrapper_descriptors); }) else {
        return;
    };
    let (landed, refused): (usize, usize) = sweep_up_to_the_smallest_landing("nofile-wrapper", 3, lands_at, wrapper_descriptors);
    assert!(refused > 0, "no descriptor limit stopped a commit, so the limits tested nothing (smallest landing limit {lands_at})");
    assert!(landed > 0, "no descriptor limit let the commit land, so none was close to exhaustion (the smallest limit at which a commit landed, {lands_at}, refused in the sweep and at twice itself alone)");
}

/// A descriptor limit of each size on the commit service,
/// from the smallest under which it starts,
/// up to the smallest under which a
///  commit lands:
/// the commit fails with a message or lands,
/// and the repository recovers either way.
#[test]
fn a_descriptor_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository() {
    let starts_at: u64 = smallest_service_limit(LimitKind::OpenFiles, FURTHEST_DESCRIPTOR_LIMIT_SEARCHED);
    let Some(lands_at): Option<u64> = smallest_landing_below_the_ceiling("nofile-service", starts_at, &|value| { return lands_with("nofile-service", value, service_descriptors); }) else {
        return;
    };
    let (landed, refused): (usize, usize) = sweep_up_to_the_smallest_landing("nofile-service", starts_at, lands_at, service_descriptors);
    assert!(refused > 0, "no descriptor limit stopped a commit, so the limits tested nothing (service starts at {starts_at}, commit lands at {lands_at})");
    assert!(landed > 0, "no descriptor limit let the commit land, so none was close to exhaustion (the smallest limit at which a commit landed, {lands_at}, refused in the sweep and at twice itself alone)");
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
///  The copy of that stream runs on a thread of its own, so this control ends the service and waits for the copy's
///  end before it reads:
///  the words are then an event and not a race with a thread,
///  and a copy that missed its bound fails here by name instead of letting a failure carry nothing.
#[test]
fn a_failure_carries_what_the_fixture_service_logged() {
    let fixture: Fixture = fixture("service-log");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let _runtime: Option<PathBuf> = super::support::service_runtime(&fixture);
    end_the_pressure(&fixture);
    assert!(
        super::support::settle_the_service_log(&fixture, super::support::SERVICE_LOG_END_BOUND),
        "the copy of the service's stream must end once the service has, or what a failure carries is a race with a thread"
    );
    let message: String = super::fault_run::with_service_log(&fixture, String::from("the commit did not land"));
    assert!(message.starts_with("the commit did not land"), "{message}");
    assert!(message.contains("ready"), "the log of the service that started is carried: {message}");
    remove_settled(&fixture);
    drop(repo);
}

/// How long one wait for the log's copies is let run in the controls that hold a copy open:
/// short, because the copy they hold cannot end in it,
/// and a wait is judged by what it answers and not by how long it waited.
const A_WAIT_GIVES_UP_AFTER: std::time::Duration = std::time::Duration::from_millis(150);

/// How many callers the control of two waits puts on one fixture.
const TWO_WAITS: usize = 2;

/// What:
///  A wait for the copies of a service's stream that gives up leaves the copy it did not see end where a later wait finds
///  it,
///  and a copy whose copier died without a word is never answered as ended.
/// Why:
///  The whole meaning of this wait is its answer:
/// `true` says every copy of the service's words has ended,
/// so a read that decides anything may trust the file.
///  A wait that empties the list and gives up would make every later answer `true` for a copy still running,
/// which is the read of a channel taken for an event again, one level up.
///  The test holds the writing end of one copy here, so the copy can end only when the test closes it,
/// and it closes it halfway through.
#[test]
fn a_wait_that_gives_up_leaves_the_copy_it_did_not_see_end_to_be_waited_for() {
    let fixture: Fixture = fixture("log-copies-held-open");
    let service: FixtureService = super::support::start_fixture_service(&fixture);
    // The test keeps the writing end: nothing but this test can close it.
    let (kept_open, when_the_copy_ends): (std::os::fd::OwnedFd, std::sync::mpsc::Receiver<()>) =
        super::support::kept_stderr_pipe(fixture.root.as_path()).expect("a pipe whose copy the test holds open");
    service.record_a_log_copy(when_the_copy_ends);
    let first: bool = service.settle_the_service_log(A_WAIT_GIVES_UP_AFTER);
    assert!(!first, "a wait that gave up answered that every copy of the service's words had ended while the test still held one open");
    let second: bool = service.settle_the_service_log(A_WAIT_GIVES_UP_AFTER);
    assert!(!second, "the second wait answered that every copy had ended, for the copy the first wait timed out on: a give-up must leave what it did not see end to be waited for");
    // The test closes its end, so the copy ends now, and the wait must say so.
    drop(kept_open);
    assert!(
        service.settle_the_service_log(super::support::SERVICE_LOG_END_BOUND),
        "a copy whose writer is closed ends, and a wait made after that must answer that every copy ended"
    );
    assert!(service.settle_the_service_log(super::support::SERVICE_LOG_END_BOUND), "with no copy left to wait for the answer is that every copy ended");
    // A copier that died before it said anything is not an end either: nothing will ever prove that copy wrote its words.
    let (never_says_it_ended, gone_without_a_word): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
    drop(never_says_it_ended);
    service.record_a_log_copy(gone_without_a_word);
    assert!(!service.settle_the_service_log(A_WAIT_GIVES_UP_AFTER), "a copy whose copier died without a word is not an end");
    assert!(
        !service.settle_the_service_log(super::support::SERVICE_LOG_END_BOUND),
        "and no later wait may answer that it ended, because nothing proves that copy wrote what the service said"
    );
    remove_settled(&fixture);
}

/// What:
///  Two waits on one fixture, standing at the seam together, never answer that a copy the test holds open ended.
/// Why:
///  A wait that takes every copy out of the fixture and gives up leaves nothing for the other,
/// so the second answers `true` for a copy still running.
///  The two callers are released together by a barrier and not by a sleep,
/// so whichever reaches the wait first, the answer of the other is the one this control judges.
#[test]
fn two_waits_on_one_fixture_never_answer_that_a_running_copy_ended() {
    let fixture: Fixture = fixture("log-copies-two-waits");
    let service: FixtureService = super::support::start_fixture_service(&fixture);
    let (kept_open, when_the_copy_ends): (std::os::fd::OwnedFd, std::sync::mpsc::Receiver<()>) =
        super::support::kept_stderr_pipe(fixture.root.as_path()).expect("a pipe whose copy the test holds open");
    service.record_a_log_copy(when_the_copy_ends);
    let together: std::sync::Barrier = std::sync::Barrier::new(TWO_WAITS);
    let answers: Mutex<Vec<bool>> = Mutex::new(Vec::new());
    let waited_on: &FixtureService = &service;
    std::thread::scope(|scope| {
        for _ in 0..TWO_WAITS {
            let (at_the_seam, collected, waited): (&std::sync::Barrier, &Mutex<Vec<bool>>, &FixtureService) = (&together, &answers, waited_on);
            scope.spawn(move || {
                let _released: std::sync::BarrierWaitResult = at_the_seam.wait();
                let answered: bool = waited.settle_the_service_log(A_WAIT_GIVES_UP_AFTER);
                collected.lock().expect("the answers").push(answered);
            });
        }
    });
    let mut found: Vec<bool> = answers.into_inner().expect("the answers");
    found.sort_unstable();
    assert_eq!(
        found,
        vec![false, false],
        "two waits on one fixture: neither may answer that a copy the test held open had ended, and one did"
    );
    drop(kept_open);
    assert!(
        service.settle_the_service_log(super::support::SERVICE_LOG_END_BOUND),
        "once the test closes its end both copies end, and a wait made after that must answer that every copy ended"
    );
    remove_settled(&fixture);
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
    let case: Case = run_case("nproc-wrapper", &[], &[soft(LimitKind::Processes, 1)], false, true, None);
    assert_eq!(case.verdict, Ok(()));
    assert!(!case.landed, "the wrapper could start no process, so the commit cannot have landed through it");
}

/// What:
///  An address space of each size on the wrapper,
///  from the smallest under which a commit lands downward (sizes in mebibytes),
///  and one limit above that smallest:
///  a failed allocation ends a Rust program with an abort, whatever it was doing,
///  and the repository recovers after each.
/// Why:
///  The sweep's largest limit is the search's own answer, which is a whole mebibyte at most one above the size at which a
///  commit first lands,
///  so on a build whose threshold sits a few kilobytes under that mebibyte the largest case is a coin flip.
///  The landing this control needs is therefore asked for once more at
///  `mebibytes_of_the_search_step_and_one_thread_pair` above the search's answer,
///  where the plan leaves two workers whatever the build's mapped size is
///  (`runtime_plan::workers_by_address_space`),
///  and that case carries the landing: this control asserts that the commit there landed, and not only that some case of
///  the sweep did,
///  because a refused case above the boundary is what this control exists to catch.
///  Measured on this host at 2026-11-15T08:39-05:00 in a scope held to `CPUQuota=400%`: the commit at the search's answer
///  and at 1, 4, 5, 6, 7, 8 and 10 mebibytes above it landed every time, three rounds each,
///  so the room this control adds is one a commit lands at and not a coin flip.
///  Measured on this host at 2026-11-14T23:10-05:00 in a scope held to `CPUQuota=400%`, by halving over one mebibyte:
///  the search answered 50 MiB while a commit first landed at 49 MiB + 180 KiB,
///  so the largest case of the sweep stood 180 KiB above the threshold.
///  What it no longer proves is that the sweep's own largest case lands: the boundary is the boundary, and a case at it
///  may refuse on any host.
#[test]
fn a_memory_limit_close_to_exhaustion_on_the_wrapper_leaves_a_recoverable_repository() {
    let Some(lands_at): Option<u64> = smallest_working(1, MOST_MEBIBYTES, &|value| { return lands_with("as-wrapper", value, wrapper_memory); }) else {
        panic!("a commit landed at none of the address-space sizes from 1 to {MOST_MEBIBYTES} MiB that the search probed, so this control has no landing to be close to exhaustion");
    };
    let values: Vec<u64> = spread(1, lands_at, MEMORY_STEPS);
    let (swept_landed, swept_refused): (usize, usize) = sweep("as-wrapper", values.as_slice(), wrapper_memory, true);
    let room: u64 = lands_at.saturating_add(mebibytes_of_the_search_step_and_one_thread_pair());
    let (service, wrapper): (Vec<Limit>, Vec<Limit>) = wrapper_memory(room);
    let above: Case = run_case(format!("as-wrapper-above-the-smallest-landing-{room}").as_str(), service.as_slice(), wrapper.as_slice(), true, true, None);
    assert!(above.verdict.is_ok(), "a limit {room} MiB above the smallest landing left the repository unrecovered: {:?}", above.verdict);
    // The case above the boundary carries the landing, so a case that lands no commit there fails here and is not counted
    //  as one more refusal of the sweep: the totals below would hide it while any case of the sweep landed.
    //  The message says what happened in the two facts apart: the branch, which is the commit, and the wrapper, which is
    //  its own ending and words; each service log is under its own label, so the recovery's landing cannot be read as the
    //  limited attempt's.
    assert!(
        above.landed,
        "a commit must land at {room} MiB, one search step and one thread pair above the smallest limit at which a commit lands ({lands_at} MiB), and the commits on the repository's branch were {} before the limited attempt and {} after it, before the recovery ran; the wrapper ended {} and its standard error said: {}
what the limited attempt's service wrote: {}
what the service wrote while the recovery ran, whose commit is not the one this case is about: {}",
        above.commits_before,
        above.commits_after,
        above.status,
        above.stderr,
        above.limited_log,
        above.recovery_log
    );
    let landed: usize = swept_landed + usize::from(above.landed);
    let refused: usize = swept_refused + usize::from(!above.landed);
    assert!(refused > 0, "no address-space limit stopped a commit, so the limits tested nothing (smallest landing limit {lands_at})");
    assert!(
        landed > 0,
        "no address-space limit let the commit land, so none was close to exhaustion (the smallest limit at which a commit landed, {lands_at}, refused, and so did {room}, one search step and one thread pair above it)"
    );
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
    let commits_before: String = commits_of_the_branch(&fixture, repo.as_path());
    let output: Output = limited_commit(&fixture, repo.as_path(), &[]);
    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    let first: Result<(), String> = check_limited_run(label, &output, false);
    // The pressure ends here, so the limited attempt's words are complete and are read before the recovery adds its own.
    end_the_pressure(&fixture);
    let limited_log: String = the_service_log_of(&fixture);
    let commits_after: String = commits_of_the_branch(&fixture, repo.as_path());
    let judged: Result<(), String> = first.and_then(|()| { return verify_recovery_with(&fixture, repo.as_path(), label, said.as_str(), run_unsettled); });
    let verdict: Result<(), String> = super::fault_run::keeping_the_service_log(&fixture, judged);
    let recovery_log: String = the_log_of_the_recovery(&limited_log, &the_service_log_of(&fixture));
    let status: std::process::ExitStatus = output.status;
    let stderr: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    remove_settled(&fixture);
    return Case {
        landed: commits_before != commits_after,
        returned_success: output.status.code() == Some(0),
        commits_before,
        commits_after,
        status,
        stderr,
        verdict,
        limited_log,
        recovery_log,
        service_end: None,
    };
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

/// What:
///  A service whose address-space limit leaves the runtime plan no worker ends before it accepts a connection:
/// the limited process itself ends, by itself, with the status the program gives a service that cannot run a task it
/// cannot serve without,
/// within `SERVICE_END_BOUND` of the commit's return.
/// No commit lands through it,
/// and the repository recovers with a fresh service that has no limit.
/// Why:
///  A limit only a little under the size at which the plan first leaves a worker is a knife edge,
///  because the threshold that size marks sits wherever the build's own mapped size puts it:
///  the plan leaves a worker from `ADDRESS_SPACE_RESERVE_BYTES` plus one thread pair upward
///  (`runtime_plan::workers_by_address_space`),
///  and the search answers a whole mebibyte.
///  Measured on this host at 2026-11-14T21:18-05:00 in a scope held to `CPUQuota=400%`,
///  by halving over one mebibyte:
///  the service's own start path first landed a commit at 117 MiB + 8 KiB,
///  while the search's own start path first greeted a client at 117 MiB + 60 KiB.
///  So the search answered 118 MiB and one mebibyte below it still ran in the case's own path,
///  where a start whose mapped size was 8 KiB smaller needed one worker less.
///  That is what the integration pass of 2026-11-14 measured at its tip (`b7af79961f`):
///  "a service under 117 MiB cannot run, so no commit can have landed through it" at a tree where the same path first
///  landed at 117 MiB or lower, with the wrapper right in both runs.
///  This control sets the limit `mebibytes_of_the_search_step_and_one_thread_pair` below the size the search answers,
///  which is below the size at which any worker fits whatever the build's mapped size is.
///  It proves the state it asserts on by the event itself, and not by a line:
///  a review of 2026-11-15 replaced the three `std::process::exit` of the no-worker branches with `return`,
///  and the control passed while the service stayed alive for twenty seconds, because it read the service's own claim
///  that it ends;
///  with the copy of the service's stream held for 250 ms it failed a service that had ended, because that claim is
///  copied by a thread nothing waited for.
///  So the end is the limited process's own exit status, read from the child within the bound,
///  and what the service wrote is carried only by a failure, to explain it.
///  What it no longer proves is where the boundary is:
///  the descriptor and address-space sweeps of this module measure the sizes,
///  and this control proves what the far side of the boundary does to a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startsAt = await smallestServiceLimit('address-space', 65536);
/// const { status } = await runWithLimitedService({ mib: startsAt - mebibytesOfTheSearchStepAndOneThreadPair(), endedWithin: SERVICE_END_BOUND });
/// expect(status).toBe(EXIT_NO_WORKER); expect(recovered).toBe(true);
/// ```
#[test]
fn a_service_that_ends_before_it_listens_leaves_a_recoverable_repository() {
    let starts_at: u64 = smallest_service_limit(LimitKind::AddressSpace, MOST_MEBIBYTES);
    let room: u64 = mebibytes_of_the_search_step_and_one_thread_pair();
    let Some(below): Option<u64> = starts_at.checked_sub(room) else {
        panic!("the search answered {starts_at} MiB, which is not above the {room} MiB this control sets its limit by, so it has no limit at which a service binds and cannot accept");
    };
    let (service, wrapper): (Vec<Limit>, Vec<Limit>) = service_memory(below);
    let case: Case = run_case("as-service-below-start", service.as_slice(), wrapper.as_slice(), true, true, Some(SERVICE_END_BOUND));
    match case.service_end {
        Some(ServiceEnd::Ended(status)) => assert_eq!(
            status.code(),
            Some(git_policy_cli::service_runtime_build_report::EXIT_NO_WORKER),
            "the service at {below} MiB must end by itself with the status the program gives a service that cannot run a task it cannot serve without, and it ended with {status}; what it wrote explains it: {}",
            case.limited_log
        ),
        Some(ServiceEnd::StillRunning) => panic!(
            "the service at {below} MiB still ran {SERVICE_END_BOUND:?} after the commit returned, so nothing shows it ends before it accepts a connection; what it wrote: {}",
            case.limited_log
        ),
        Some(ServiceEnd::NotServed) => panic!("the fixture served no service for this worktree, so this control has no end to observe: {}", case.limited_log),
        None => panic!("this control asked for the service's end and was given none"),
    }
    assert!(
        !case.landed,
        "a service that ended before it accepted a connection cannot have landed a commit, and the commits on the repository's branch were {} before the limited attempt and {} after it; the wrapper ended {} and returned success: {}; its standard error said: {}",
        case.commits_before, case.commits_after, case.status, case.returned_success, case.stderr
    );
    assert_eq!(case.verdict, Ok(()), "the repository must recover with an unlimited service");
}

/// What:
///  The room one step of the search and one worker with its blocking-pool partner take, in mebibytes.
/// Why:
///  A search here answers a whole mebibyte (`MEBIBYTE`), so its answer is at most one mebibyte above the size at which the
///  runtime plan first leaves a worker,
///  and one worker and its blocking-pool partner take `THREAD_VIRTUAL_BYTES` twice
///  (`runtime_plan::workers_by_address_space`).
///  A limit this far below an answer fits no worker, and one this far above a landing leaves two,
///  whatever the build's own mapped size is,
///  because the build moves the search's answer and the limit together.
///  So a control that needs one of those states sets its limit by this much and never by the search's step alone,
///  which is a few kilobytes of a build away from the threshold on a build of any size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const room = Math.ceil((MiB + 2 * THREAD_VIRTUAL_BYTES) / MiB);
/// ```
pub(super) fn mebibytes_of_the_search_step_and_one_thread_pair() -> u64 {
    return (MEBIBYTE + 2 * git_policy_cli::runtime_plan::THREAD_VIRTUAL_BYTES).div_ceil(MEBIBYTE);
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
///  The limit is `mebibytes_of_the_search_step_and_one_thread_pair` below the size the search answers and not one
///  mebibyte below it:
///  one mebibyte below an answer is inside the search's own rounding, a few kilobytes of the build away from the
///  threshold.
///  Measured on this host at 2026-11-14T23:10-05:00 in a scope held to `CPUQuota=400%`, by halving over one mebibyte:
///  the search answered 118 MiB while the probe's own path first greeted a client at 117 MiB + 76 KiB,
///  so one mebibyte below the answer stood 76 KiB under the threshold,
///  and a start whose mapped size was 76 KiB lighter would have greeted and failed this control with the wrapper right.
///  Below this control's limit the plan fits no worker whatever the build's mapped size is, and the socket is still
///  bound, because the bind comes before the plan is read.
///  What it no longer proves is that the one mebibyte the search subtracts is itself in this state: that was the
///  search's step, not a fact about what the service can do.
#[test]
fn a_limit_that_lets_the_service_bind_but_not_run_is_never_counted_as_starting() {
    let starts_at: u64 = smallest_service_limit(LimitKind::AddressSpace, MOST_MEBIBYTES);
    let room: u64 = mebibytes_of_the_search_step_and_one_thread_pair();
    let Some(below_mebibytes): Option<u64> = starts_at.checked_sub(room) else {
        panic!("the search answered {starts_at} MiB, which is not above the {room} MiB this control sets its limit by, so it has no limit at which a service binds and cannot run");
    };
    let below: u64 = below_mebibytes * MEBIBYTE;
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
