//! What:
//!  The injection and the invariants shared by the fault controls of the operating conditions in the README:
//!  a full disk,
//!  descriptor and process limits,
//!  a process killed at any system call,
//!  a closed terminal and slow or unusual filesystems.
//! Why:
//!  Each control runs one explicit-path commit through the built wrapper with a single fault injected at the `n`th
//!  call of one system call, with no privilege and no load: `strace --inject` (a `ptrace` of the child) makes that
//!  call fail with an errno, kills the process there, or delays its return.
//!  Without `--follow-forks` only the wrapper's own thread is traced, so the `n`th call is the same call on every run
//!  and a failing case replays exactly; with it, Git's processes are hit too, and the `n`th call shifts as Git's
//!  threads interleave, so no case is pinned to an index and the invariants must hold for whichever call fails.
//!  The invariants are the promise of the journals: whatever the fault was, the next ordinary command succeeds
//!  without a person repairing anything, the commit is either landed with a clean index or not made with its file
//!  still staged, nothing of the wrapper stays behind, and the repository passes `git fsck`.
//!  The sweeps need `strace`; a host without it reports that and checks nothing, as `binary_fork_failure_tests.rs` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const n of range(1, countCalls('write'))) {
//!   await runUnderStrace({ inject: `write:error=ENOSPC:when=${n}` }, wrappedCommit);
//!   await expectRecoveredRepository();
//! }
//! ```

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit};
use super::commit::COMMIT_BOUND_SECONDS;
use super::fork_failure::{strace_available, strace_program};
use super::support::{Fixture, Observed, bounded_for, fixture, git, repository, wrapper_first_path};
use std::ffi::OsStr;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many cases of a sweep run at once.
pub(super) const SWEEP_WIDTH: usize = 6;

/// What a fault does to the call it hits.
#[derive(Clone, Copy, Debug)]
pub(super) enum Effect {
    /// The call fails with this errno name, as a full disk or an exhausted limit makes it fail.
    Fail(&'static str),
    /// The process is killed with `SIGKILL` at the call, before the kernel does its work.
    Kill,
    /// The call returns after this many microseconds, as a slow disk makes it return.
    SlowReturn(u64),
}

/// Which calls of the traced set a fault hits.
#[derive(Clone, Copy, Debug)]
pub(super) enum When {
    /// Only the `n`th call, counted from one.
    Nth(usize),
    /// The `n`th call and every later one, as a limit that stays exhausted does.
    FromNth(usize),
}

/// One fault: a system call, what happens to it, and which of its calls.
#[derive(Clone, Copy, Debug)]
pub(super) struct Injection {
    /// The system call name, as `strace` spells it.
    pub(super) syscall: &'static str,
    /// What happens to the call.
    pub(super) effect: Effect,
    /// Which calls of the traced set it happens to.
    pub(super) when: When,
    /// Whether Git's processes are traced too, not only the wrapper's own thread.
    pub(super) follow_forks: bool,
}

impl Injection {
    /// The `--inject` argument for this fault.
    fn specification(&self) -> String {
        let effect: String = match self.effect {
            Effect::Fail(errno) => format!("error={errno}"),
            Effect::Kill => String::from("signal=KILL"),
            Effect::SlowReturn(microseconds) => format!("delay_exit={microseconds}us"),
        };
        let when: String = match self.when {
            When::Nth(call) => format!("{call}"),
            When::FromNth(call) => format!("{call}+"),
        };
        return format!("{}:{effect}:when={when}", self.syscall);
    }
}

/// What one run under a fault showed.
pub(super) struct Ran {
    /// What the traced process printed and how it ended.
    pub(super) output: Output,
    /// The `strace` log of the traced calls.
    pub(super) trace: String,
}

impl Ran {
    /// Whether the fault took effect: `strace` marks an injected call, and a kill ends the log with the kill.
    pub(super) fn injected(&self) -> bool {
        return self.trace.contains("(INJECTED)")
            || self.trace.contains("(DELAYED)")
            || self.trace.contains("+++ killed by SIGKILL +++");
    }

    /// Standard output and standard error as one text.
    pub(super) fn text(&self) -> String {
        return String::from_utf8_lossy(&[self.output.stdout.as_slice(), self.output.stderr.as_slice()].concat())
            .into_owned();
    }

    /// The exit code, or `-1` when a signal ended the process.
    pub(super) fn code(&self) -> i32 {
        return self.output.status.code().unwrap_or(-1);
    }
}

/// A repository with one commit and `a.txt` staged, ready for an explicit-path commit.
pub(super) fn staged_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// The wrapped command under `strace`: the commit bound outside, the tracer, then the fixture's `git` link.
///  `injection` is absent for a run that only counts calls.
fn traced(
    fixture: &Fixture,
    repo: &Path,
    arguments: &[&str],
    calls: &str,
    injection: Option<&Injection>,
    follow_forks: bool,
    trace: &Path,
) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        strace_program().expect("strace is installed").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    if follow_forks {
        command.arg("--follow-forks");
    }
    command
        .arg("--quiet")
        .arg("--quiet")
        .arg(format!("--output={}", trace.display()))
        .arg(format!("--trace={calls}"));
    if let Some(fault) = injection {
        command.arg(format!("--inject={}", fault.specification()));
    }
    command.arg(fixture.root.join("bin/git")).args(arguments).current_dir(repo);
    return command;
}

/// Run a prepared command to its end with no input, keeping every byte it printed.
pub(super) fn run_to_end(command: &mut Command) -> Output {
    return command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("the traced command starts");
}

/// The commit every sweep runs: an explicit-path commit of the staged `a.txt`.
pub(super) const COMMIT_ARGUMENTS: [&str; 5] = ["commit", "--quiet", "--message=x", "--", "a.txt"];

/// How many calls of `syscall` an uninjected explicit-path commit makes in the traced process, or, following forks,
///  in the process of the tree that makes the most: the fault counts each process on its own.
pub(super) fn count_calls(fixture: &Fixture, label: &str, syscall: &str, follow_forks: bool) -> usize {
    let repo: PathBuf = staged_repository(fixture, &format!("count-{label}"));
    let trace: PathBuf = fixture.root.join(format!("count-{label}.txt"));
    let landed: Output = run_to_end(&mut traced(
        fixture,
        repo.as_path(),
        &COMMIT_ARGUMENTS,
        syscall,
        None,
        follow_forks,
        trace.as_path(),
    ));
    assert_eq!(landed.status.code(), Some(0), "the uninjected commit must land: {}", String::from_utf8_lossy(&landed.stderr));
    let text: String = std::fs::read_to_string(&trace).expect("the call log");
    let prefix: String = format!("{syscall}(");
    // The fault counts the calls of each process on its own, so the sweep reaches as far as the busiest process.
    let mut per_process: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for line in text.lines() {
        let (process, call): (&str, &str) = if follow_forks { line.split_once(' ').unwrap_or(("", line)) } else { ("", line) };
        if call.starts_with(prefix.as_str()) {
            *per_process.entry(process).or_insert(0) += 1;
        }
    }
    return per_process.values().copied().max().unwrap_or(0);
}

/// Run one explicit-path commit with the fault injected.
pub(super) fn run_injected(fixture: &Fixture, repo: &Path, label: &str, fault: &Injection) -> Ran {
    let trace: PathBuf = fixture.root.join(format!("trace-{label}.txt"));
    let output: Output = run_to_end(&mut traced(
        fixture,
        repo,
        &COMMIT_ARGUMENTS,
        fault.syscall,
        Some(fault),
        fault.follow_forks,
        trace.as_path(),
    ));
    let log: String = std::fs::read_to_string(&trace).unwrap_or_default();
    return Ran { output, trace: log };
}

/// The names in the transaction registry that recovery keeps on purpose: staging candidates of a dead process and
///  retired lock copies; nothing reads them as state, and the registry documents that it leaves them.
fn is_kept_by_design(name: &str) -> bool {
    return name.ends_with(".pending") || name.ends_with(".stale");
}

/// Remove the registry entries that are kept by design, so the check for what a commit leaves can run on the rest.
fn drop_kept_by_design(repo: &Path) {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    let Ok(entries) = std::fs::read_dir(&registry) else {
        return;
    };
    for entry in entries.flatten() {
        if is_kept_by_design(entry.file_name().to_string_lossy().as_ref()) {
            std::fs::remove_dir_all(entry.path()).expect("remove an entry kept by design");
        }
    }
}

/// Whether a panic payload is a message, and the message.
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    if let Some(text) = payload.downcast_ref::<&str>() {
        return String::from(*text);
    }
    return String::from("a check failed without a message");
}

/// What the first run, the one that met the fault, must have shown its caller:
///  it ended inside its bound (or was killed by the injection itself), and a failure says why.
fn check_first_run(label: &str, fault: &Injection, ran: &Ran) -> Result<(), String> {
    if matches!(fault.effect, Effect::Kill) {
        // The kill ended the traced wrapper, which `strace` reports by ending the same way.
        if ran.output.status.signal() == Some(9) {
            return Ok(());
        }
        // In a traced tree the kill may have ended one of Git's processes instead, and the wrapper then reports that
        // like any failure of Git; with only the wrapper traced, an injected kill that did not end it is a mistake.
        if !fault.follow_forks && ran.injected() {
            return Err(format!("{label}: the process was to be killed at the call but ended with {:?}", ran.output.status));
        }
    } else if ran.output.status.signal() == Some(9) {
        return Err(format!("{label}: the command hit its time bound: {}", ran.text()));
    }
    if !matches!(ran.code(), 0 | 1 | 2) {
        return Err(format!("{label}: unexpected exit {}: {}", ran.code(), ran.text()));
    }
    // A fault that hit the write of the message itself leaves nothing to read; any other failure must say why.
    let message_lost: bool = ran
        .trace
        .lines()
        .any(|line| { return line.contains("(INJECTED)") && (line.contains("write(1,") || line.contains("write(2,")); });
    if ran.code() != 0 && !message_lost && ran.text().trim().is_empty() {
        return Err(format!("{label}: a failed commit must say why"));
    }
    return Ok(());
}

/// What the repository must show after any first run that met a fault, whatever the fault was:
///  the next ordinary command succeeds with the commit landed or not made,
///  the repository passes `git fsck`,
///  a further commit lands,
///  and nothing of the wrapper stays behind once that commit has landed (a lock released by a dead process is
///  retired by the next landing, not by a status).
///  `said` is what the first run printed, for the failure message.
pub(super) fn verify_recovery(fixture: &Fixture, repo: &Path, label: &str, said: &str) -> Result<(), String> {
    let next: Observed = run_commit(fixture, repo, &["status", "--porcelain"]);
    if next.code != Some(0) {
        return Err(format!(
            "{label}: the next command must succeed, exit {:?}: {}\nfirst run said: {}",
            next.code,
            String::from_utf8_lossy(&next.stderr),
            said.chars().take(600).collect::<String>()
        ));
    }
    let commits: String = git_text(fixture, repo, &["rev-list", "--count", "HEAD"]);
    let status: String = String::from_utf8_lossy(next.stdout.as_slice()).into_owned();
    match commits.trim() {
        "1" if status == "A  a.txt\n" => {}
        "2" if status.is_empty() => {}
        other => return Err(format!("{label}: {other} commits with status {status:?}; first run said: {said}")),
    }
    let checked: Output = Command::new("/usr/bin/git")
        .current_dir(repo)
        .args(["fsck", "--strict", "--no-dangling"])
        .env_clear()
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .output()
        .expect("git fsck starts");
    if !checked.status.success() || !checked.stderr.is_empty() {
        return Err(format!("{label}: git fsck: {}", String::from_utf8_lossy(&checked.stderr)));
    }
    std::fs::write(repo.join("b.txt"), b"two\n").expect("b");
    git(fixture, repo, &["add", "--", "b.txt"]);
    let again: Observed = run_commit(fixture, repo, &["commit", "--quiet", "--message=y", "--", "b.txt"]);
    if again.code != Some(0) {
        return Err(format!(
            "{label}: a further commit must land, exit {:?}: {}\nfirst run said: {said}",
            again.code,
            String::from_utf8_lossy(&again.stderr)
        ));
    }
    drop_kept_by_design(repo);
    let leftovers = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { assert_nothing_left(fixture, repo); }));
    if let Err(payload) = leftovers {
        return Err(format!("{label}: {}; first run said: {said}", panic_text(payload.as_ref())));
    }
    return Ok(());
}

/// Run every case of one sweep, `SWEEP_WIDTH` at a time, and check each: the fault hits the `n`th call of
///  `syscall` (and, for a persistent fault, every call after it) for `n` from one through the number of calls an
///  uninjected commit makes.
fn sweep_calls(label: &str, syscall: &'static str, effect: Effect, follow_forks: bool, persistent: bool, stride: usize) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no call was faulted");
        return;
    }
    let fixture: Fixture = fixture(label);
    let calls: usize = count_calls(&fixture, label, syscall, follow_forks);
    assert!(calls > 0, "an uninjected commit makes no `{syscall}` call, so there is nothing to fault");
    let cases: Vec<usize> = (1..=calls).step_by(stride).collect();
    let injected_cases: AtomicUsize = AtomicUsize::new(0);
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    for batch in cases.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for call in batch {
                let shared_fixture: &Fixture = &fixture;
                let counter: &AtomicUsize = &injected_cases;
                let collected: &Mutex<Vec<String>> = &failures;
                scope.spawn(move || {
                    let case_label: String = format!("{label}-{call}");
                    let repo: PathBuf = staged_repository(shared_fixture, &format!("case-{call}"));
                    let when: When = if persistent { When::FromNth(*call) } else { When::Nth(*call) };
                    let fault: Injection = Injection { syscall, effect, when, follow_forks };
                    let ran: Ran = run_injected(shared_fixture, repo.as_path(), case_label.as_str(), &fault);
                    if ran.injected() {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                    let verdict: Result<(), String> = check_first_run(case_label.as_str(), &fault, &ran)
                        .and_then(|()| { return verify_recovery(shared_fixture, repo.as_path(), case_label.as_str(), ran.text().as_str()); });
                    if let Err(message) = verdict {
                        collected.lock().expect("failure list").push(message);
                    }
                });
            }
        });
    }
    let found: Vec<String> = failures.into_inner().expect("failure list");
    assert!(found.is_empty(), "{} of {calls} faulted `{syscall}` calls left the repository unrecovered:\n{}", found.len(), found.join("\n"));
    // Positive control: a sweep whose faults never took effect would pass without having tested anything.
    let injected_total: usize = injected_cases.load(Ordering::SeqCst);
    assert!(
        injected_total * 2 >= cases.len(),
        "the fault took effect in {injected_total} of {} cases, so the sweep tested too little",
        cases.len()
    );
    remove_settled(&fixture);
}

/// Fault each call of `syscall` in turn, once, and check the repository after each.
pub(super) fn sweep(label: &str, syscall: &'static str, effect: Effect, follow_forks: bool) {
    sweep_calls(label, syscall, effect, follow_forks, false, 1);
}

/// Fault every `stride`th call of `syscall`, and every call after it, as a limit that stays exhausted does, and
///  check the repository after each.
///  A stride above one samples the starting points, for a fault the wrapper rides out slowly (a refused process start
///  is retried for seconds, at every start).
pub(super) fn sweep_persistent(label: &str, syscall: &'static str, effect: Effect, follow_forks: bool, stride: usize) {
    sweep_calls(label, syscall, effect, follow_forks, true, stride);
}
