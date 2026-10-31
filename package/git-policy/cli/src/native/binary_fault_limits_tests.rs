//! What:
//!  Descriptor, process and file-size limits that are close to exhausted, set as a resource limit on the child
//!  (`setrlimit` between `fork` and `exec`, no privilege) or injected as the error a limit produces
//!  (`strace --inject`, `binary_fault_support.rs`).
//! Why:
//!  The README promises that process and descriptor limits close to exhaustion do not strand the repository.
//!  A resource limit on the child is the real mechanism: `RLIMIT_NOFILE` makes every open and pipe fail with
//!  `EMFILE` from some point on, and `RLIMIT_FSIZE` makes a write stop at a byte count, which leaves a file half
//!  written and then fails the rest of the write with `EFBIG` (a half write that a failed call cannot show).
//!  The injected errors cover each call in turn, so every place the commit opens a file, makes a pipe or starts a
//!  process is reached, which no single limit value does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const limit of [8, 12, 16]) await runWithLimit({ nofile: limit }, wrappedCommit); await expectRecoveredRepository();
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::fault::{Effect, SWEEP_WIDTH, sweep, sweep_persistent, run_to_end, staged_repository, verify_recovery};
use super::support::Fixture;
use super::support::fixture;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Every `openat` fails with too many open files, as the descriptor limit makes it fail.
#[test]
fn a_descriptor_limit_hit_at_any_open_leaves_a_recoverable_repository() {
    sweep("emfile-openat", "openat", Effect::Fail("EMFILE"), false);
}

/// Every `pipe2` fails with too many open files: the pipes to and from the Git children of a commit.
#[test]
fn a_descriptor_limit_hit_at_any_pipe_leaves_a_recoverable_repository() {
    sweep("emfile-pipe", "pipe2", Effect::Fail("EMFILE"), false);
}

/// Every process start from the `n`th on fails with resource temporarily unavailable, as a process limit that stays
///  exhausted makes it fail, where `binary_fork_failure_tests.rs` fails only a short run of starts.
#[test]
fn a_process_limit_that_stays_exhausted_from_any_start_leaves_a_recoverable_repository() {
    sweep_persistent("nproc-clone3", "clone3", Effect::Fail("EAGAIN"), false, 6);
}

/// What the first run of a limited commit must have shown its caller: it ended without the time bound, as the
///  wrapper (exit 0, 1 or 2) or as a process that never started (126 or 127); a started wrapper that fails says why.
fn check_limited_run(label: &str, output: &Output) -> Result<(), String> {
    let text: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
    if output.status.signal().is_some() {
        return Err(format!("{label}: the command was ended by signal {:?}: {text}", output.status.signal()));
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

/// A wrapped commit whose process, and every process it starts, may keep at most `limit` descriptors open.
fn commit_with_open_file_limit(fixture: &Fixture, repo: &std::path::Path, limit: u64) -> Command {
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    // SAFETY: between fork and exec only `setrlimit`, an async-signal-safe call, runs.
    unsafe {
        command.pre_exec(move || {
            let cap: libc::rlimit = libc::rlimit { rlim_cur: limit, rlim_max: limit };
            if libc::setrlimit(libc::RLIMIT_NOFILE, &cap) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    return command;
}

/// A wrapped commit whose process, and every process it starts, may not write a file past `limit` bytes, with
///  `SIGXFSZ` ignored so that the write that crosses the limit fails with `EFBIG` after the bytes below it.
fn commit_with_file_size_limit(fixture: &Fixture, repo: &std::path::Path, limit: u64) -> Command {
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    // SAFETY: between fork and exec only `setrlimit` and `signal`, both async-signal-safe, run.
    unsafe {
        command.pre_exec(move || {
            libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
            let cap: libc::rlimit = libc::rlimit { rlim_cur: limit, rlim_max: limit };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &cap) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    return command;
}

/// Run one commit per limit value, `SWEEP_WIDTH` at a time, and check the repository after each.
///  `limited` builds the limited command for a repository and a value.
///  `refusal` is the operating system's wording of the error the limit causes: at least one run must show it
///  (a positive control: a sweep whose limits never bit would pass without having tested anything), and at least
///  one run must have landed its commit despite the limit (the control that the limits are not all too small to test).
fn sweep_limits(label: &str, values: &[u64], limited: fn(&Fixture, &std::path::Path, u64) -> Command, refusal: &str) {
    let fixture: Fixture = fixture(label);
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let refused: AtomicUsize = AtomicUsize::new(0);
    let landed: AtomicUsize = AtomicUsize::new(0);
    for batch in values.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for value in batch {
                let shared_fixture: &Fixture = &fixture;
                let collected: &Mutex<Vec<String>> = &failures;
                let refusals: &AtomicUsize = &refused;
                let landings: &AtomicUsize = &landed;
                scope.spawn(move || {
                    let case_label: String = format!("{label}-{value}");
                    let repo: PathBuf = staged_repository(shared_fixture, &format!("case-{value}"));
                    let output: Output = run_to_end(&mut limited(shared_fixture, repo.as_path(), *value));
                    let said: String = String::from_utf8_lossy(&[output.stdout.as_slice(), output.stderr.as_slice()].concat()).into_owned();
                    if said.contains(refusal) {
                        refusals.fetch_add(1, Ordering::SeqCst);
                    }
                    if output.status.code() == Some(0) {
                        landings.fetch_add(1, Ordering::SeqCst);
                    }
                    let verdict: Result<(), String> = check_limited_run(case_label.as_str(), &output)
                        .and_then(|()| { return verify_recovery(shared_fixture, repo.as_path(), case_label.as_str(), said.as_str()); });
                    if let Err(message) = verdict {
                        collected.lock().expect("failure list").push(message);
                    }
                });
            }
        });
    }
    let found: Vec<String> = failures.into_inner().expect("failure list");
    assert!(found.is_empty(), "{} of {} limits left the repository unrecovered:\n{}", found.len(), values.len(), found.join("\n"));
    assert!(refused.load(Ordering::SeqCst) > 0, "no limit made a run report {refusal:?}, so the limits tested nothing");
    assert!(landed.load(Ordering::SeqCst) > 0, "no limit let the commit land, so none was close to exhaustion");
    remove_settled(&fixture);
}

/// A descriptor limit of each size from a few descriptors above the standard three to the point where the commit
///  has room: it fails with a message or lands, and the repository recovers either way.
#[test]
fn a_descriptor_limit_close_to_exhaustion_leaves_a_recoverable_repository() {
    let values: Vec<u64> = (4..=48).collect();
    sweep_limits("rlimit-nofile", values.as_slice(), commit_with_open_file_limit, "Too many open files");
}

/// A file-size limit of each size, so that the first file larger than it is cut after `limit` bytes and the rest of
///  its write fails: the records and objects of a commit are written half and then refused.
#[test]
fn a_file_size_limit_that_cuts_each_file_leaves_a_recoverable_repository() {
    let values: Vec<u64> = vec![0, 1, 8, 32, 64, 100, 128, 160, 200, 256, 300, 400, 512, 700, 1000, 1500, 2000, 3000, 5000, 10_000];
    sweep_limits("rlimit-fsize", values.as_slice(), commit_with_file_size_limit, "File too large");
}
