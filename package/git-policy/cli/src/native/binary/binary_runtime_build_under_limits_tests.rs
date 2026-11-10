//! What:
//!  Controls that the runtime build finishes,
//! and the command carries on,
//! where the system limits threads or address space.
//! Why:
//!  A multi-thread runtime panics,
//! then hangs,
//! when the system refuses it a thread or address space
//!  (`doc/planning/git-policy-cli-full-tokio.md`,
//! "A multi-thread runtime panics,
//! then hangs"),
//! and the owner's rules are
//!  "Commits should never fail."
//! and "nothing should ever wait forever".
//! Each control starts the built executable with the
//!  test-only runtime probe under one limit,
//! and starts real Git under the same limit as the native control (choice 10,
//! "what
//!  Git completes,
//! we complete"):
//! where Git completes,
//! the probe must complete too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const git = run('git', ['--version'], { limits }); const probe = run(wrapper, [], { env: PROBE, limits });
//! if (git.code === 0) expect(probe.code).toBe(0);
//! ```

use super::support::{Fixture, Observed, direct, fixture, hermetic, observe, remove, wrapper_first_path};
use super::support_resource_limits::LimitKind;
use std::os::unix::process::CommandExt as _;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

/// The probe's environment variable (`foundation/runtime_probe.rs`).
const PROBE: &str = git_policy_cli::runtime_probe::RUNTIME_PROBE_VARIABLE;

/// The longest a probe run may take:
/// many times the build deadline of the runtime,
/// and a margin for process start under load.
const PROBE_BOUND: std::time::Duration = std::time::Duration::from_secs(30);

/// How often a running probe is looked at.
const PROBE_POLL: std::time::Duration = std::time::Duration::from_millis(10);

/// One mebibyte,
/// for address-space limits written in the plan's units.
const MEBIBYTE: u64 = 1 << 20;

/// What the probe printed,
/// line by line.
fn probe_lines(observed: &Observed) -> Vec<String> {
    return String::from_utf8_lossy(observed.stdout.as_slice()).lines().map(String::from).collect();
}

/// Runs the probe in `fixture` with `limit` applied between fork and exec;
/// `None` runs it without one.
///  The wrapper is started directly,
/// not under `timeout`,
/// because `timeout` forks and a process limit of one refuses that fork;
///  the bound is this function's own wait,
/// and a probe still running at `PROBE_BOUND` is killed and reported as no exit code.
fn probe_under(fixture: &Fixture, limit: Option<(LimitKind, u64)>) -> Observed {
    let mut command: Command = Command::new(fixture.root.join("bin/git"));
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    command.env(PROBE, "1").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some((kind, soft)) = limit {
        let hard: u64 = kind.current_hard();
        // SAFETY: between fork and exec only `setrlimit` runs.
        unsafe {
            command.pre_exec(move || { return kind.apply(soft.min(hard), hard); });
        }
    }
    let mut child: Child = command.spawn().expect("the probe starts");
    let started: Instant = Instant::now();
    loop {
        if child.try_wait().expect("the probe can be waited for").is_some() {
            break;
        }
        if started.elapsed() > PROBE_BOUND {
            child.kill().expect("a hung probe can be killed");
            break;
        }
        std::thread::sleep(PROBE_POLL);
    }
    let output = child.wait_with_output().expect("the probe's output");
    return Observed { code: output.status.code(), stdout: output.stdout, stderr: output.stderr };
}

/// Real Git's `--version` under the same limit:
/// the native control.
/// Git completes where the system leaves it enough.
fn git_completes_under(fixture: &Fixture, limit: Option<(LimitKind, u64)>) -> bool {
    let mut command: Command = direct(fixture);
    command.arg("--version");
    if let Some((kind, soft)) = limit {
        let hard: u64 = kind.current_hard();
        // SAFETY: between fork and exec only `setrlimit` runs.
        unsafe {
            command.pre_exec(move || { return kind.apply(soft.min(hard), hard); });
        }
    }
    return observe(&mut command, b"").code == Some(0);
}

/// Without a limit the probe builds a runtime,
/// and the blocking job runs on it.
#[test]
fn without_a_limit_the_runtime_is_built_and_a_blocking_job_runs() {
    let fixture: Fixture = fixture("runtime-probe-free");
    let seen: Observed = probe_under(&fixture, None);
    assert_eq!(seen.code, Some(0), "{}", String::from_utf8_lossy(seen.stderr.as_slice()));
    let lines: Vec<String> = probe_lines(&seen);
    assert!(lines.iter().any(|line| { return line.starts_with("flavor=multi-thread") || line == "flavor=current-thread"; }), "{lines:?}");
    assert!(lines.contains(&String::from("blocking=ok")), "{lines:?}");
    remove(&fixture);
}

/// Where the process limit leaves no thread,
/// the current-thread runtime stands in and the blocking job fails plainly.
#[test]
fn where_no_thread_can_start_the_runtime_is_current_thread_and_the_blocking_job_fails_plainly() {
    let fixture: Fixture = fixture("runtime-probe-nproc");
    let limit: Option<(LimitKind, u64)> = Some((LimitKind::Processes, 1));
    assert!(git_completes_under(&fixture, limit), "the native control: Git itself must complete under the limit");
    let seen: Observed = probe_under(&fixture, limit);
    assert_eq!(seen.code, Some(0), "{}", String::from_utf8_lossy(seen.stderr.as_slice()));
    let lines: Vec<String> = probe_lines(&seen);
    if lines.iter().any(|line| { return line.starts_with("flavor=multi-thread"); }) {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no thread start was refused");
        remove(&fixture);
        return;
    }
    assert!(lines.contains(&String::from("flavor=current-thread")), "the plan must see the limit and not need a stand-in: {lines:?}");
    // The refusal is reported as the plain error above; a notice that asks for a defect report would be a false alarm.
    let stderr: String = String::from_utf8_lossy(seen.stderr.as_slice()).into_owned();
    assert!(!stderr.contains("internal error"), "a caught refusal printed a panic notice: {stderr}");
    assert!(lines.iter().any(|line| { return line.starts_with("blocking=error the system refused a thread"); }), "{lines:?}");
    remove(&fixture);
}

/// The address-space sizes of the measured hang band (tokio's multi-thread build hung at 900 to 1150 MiB with default arenas)
///  and sizes either side,
/// sampled from this list.
/// The smallest sampled size is not the smallest this control judges:
/// see `smallest_address_space_that_runs_the_job`.
const ADDRESS_SPACE_SIZES_MEBIBYTES: [u64; 18] = [48, 52, 56, 60, 64, 80, 100, 200, 400, 600, 800, 900, 950, 1020, 1030, 1100, 1150, 1500];

/// The address space the search for the smallest that runs a blocking job starts at,
/// in mebibytes:
/// below it no build of this executable has been seen to start at all,
/// so the search needs no smaller one.
const ADDRESS_SPACE_SEARCH_FLOOR_MEBIBYTES: u64 = 40;

/// The address space the search is known to succeed at,
/// in mebibytes:
/// well under the measured hang band,
/// so the search never asks about a size the band is about.
const ADDRESS_SPACE_SEARCH_CEILING_MEBIBYTES: u64 = 200;

/// What:
///  Whether the probe builds a runtime and runs its blocking job in `mebibytes` of address space.
/// Why:
///   A blocking step runs on a thread of its own,
/// and a thread needs its stack mapped,
/// so an address space that leaves less than
///  that is one where no blocking job can run whatever the plan does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runs = (mib: number) => probeLines(probeUnder(fixture, ['as', mib * MEBIBYTE])).includes('blocking=ok');
/// ```
fn runs_the_job_in_address_space(fixture: &Fixture, mebibytes: u64) -> bool {
    let seen: Observed = probe_under(fixture, Some((LimitKind::AddressSpace, mebibytes * MEBIBYTE)));
    return seen.code == Some(0) && probe_lines(&seen).contains(&String::from("blocking=ok"));
}

/// What:
///  The smallest address space in which the probe builds a runtime and runs its blocking job,
/// searched around the sampled
///  sizes.
/// Why:
///   That size is a property of this build and of the system's thread stacks,
/// not of the wrapper's plan:
///  the plan reads the
///  limit and builds the current-thread runtime below it,
/// and the blocked thread is then refused with the plain error the
///  control above requires.
/// A number copied from another build is therefore a bet:
/// the same list failed on the host and
///  passed in the gate until this search was added,
/// because the gate's executable carries no debug information and its
///  addresses need a couple of mebibytes less.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const smallest = smallestWorking(40, 200, (mib) => runsTheJobInAddressSpace(fixture, mib));
/// ```
fn smallest_address_space_that_runs_the_job(fixture: &Fixture) -> u64 {
    assert!(
        runs_the_job_in_address_space(fixture, ADDRESS_SPACE_SEARCH_CEILING_MEBIBYTES),
        "the probe does not run a blocking job in {ADDRESS_SPACE_SEARCH_CEILING_MEBIBYTES} MiB, which this control needs to find the smallest size that does"
    );
    return super::fault_limits::smallest_working(ADDRESS_SPACE_SEARCH_FLOOR_MEBIBYTES, ADDRESS_SPACE_SEARCH_CEILING_MEBIBYTES, &|mebibytes| {
        return runs_the_job_in_address_space(fixture, mebibytes);
    });
}

/// At every address-space size where Git completes and the wrapper can run a thread,
/// the probe finishes by its bound
///  with a runtime and a blocking job.
#[test]
fn at_every_address_space_size_the_runtime_is_built_by_the_deadline_and_the_job_runs() {
    let fixture: Fixture = fixture("runtime-probe-address-space");
    let smallest: u64 = smallest_address_space_that_runs_the_job(&fixture);
    for mebibytes in ADDRESS_SPACE_SIZES_MEBIBYTES.iter().copied().filter(|mebibytes| { return *mebibytes >= smallest; }) {
        let limit: Option<(LimitKind, u64)> = Some((LimitKind::AddressSpace, mebibytes * MEBIBYTE));
        if !git_completes_under(&fixture, limit) {
            continue;
        }
        let seen: Observed = probe_under(&fixture, limit);
        assert_eq!(seen.code, Some(0), "{mebibytes} MiB: {}", String::from_utf8_lossy(seen.stderr.as_slice()));
        let lines: Vec<String> = probe_lines(&seen);
        assert!(lines.contains(&String::from("blocking=ok")), "{mebibytes} MiB: {lines:?}");
        // A plan that read the limit builds what it planned: a stand-in after a deadline or a failure means the plan asked for
        //  more than the limit left and only the deadline saved the command.
        assert!(
            lines.iter().any(|line| { return line == "flavor=current-thread" || line.starts_with("flavor=multi-thread"); }),
            "{mebibytes} MiB: the plan was not built as planned: {lines:?}"
        );
    }
    remove(&fixture);
}
