//! What:
//!  The injection and the invariants shared by the fault controls that remain: standard streams that are closed, unread or
//!  full, and a Git step held past its class bound.
//! Why:
//!  Each control runs one explicit-path commit through the built wrapper with a fault injected under `strace --inject` (a
//!  `ptrace` of the child), with no privilege and no load: the call is delayed in its return. The invariants are the promise
//!  of the service route: whatever the fault was, the next ordinary command succeeds without a person repairing anything,
//!  the commit is either landed with a clean index or not made with its file still staged, nothing of the wrapper stays
//!  behind, and the repository passes `git fsck`. The sweeps that once failed, killed or delayed every system call of the
//!  earlier landing retired with that landing. A host without `strace` reports that and checks nothing, as the budget
//!  controls do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runUnderStrace({ inject: 'execve:delay_exit=...' }, wrappedCommit);
//! await expectRecoveredRepository();
//! ```

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, run_commit};
use super::commit::COMMIT_BOUND_SECONDS;
use super::strace::strace_program;
use super::support::{Fixture, Observed, bounded_for, git, repository, wrapper_first_path};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// What a fault does to the call it hits.
#[derive(Clone, Copy, Debug)]
pub(super) enum Effect {
    /// The call returns after this many microseconds, as a slow disk makes it return.
    SlowReturn(u64),
    /// The call fails with this error name, as `strace` spells it (`ENOSPC`, `EIO`), and does nothing.
    Fail(&'static str),
    /// The calling process is ended with `SIGKILL` at the call.
    Kill,
}

/// Which calls of the traced set a fault hits.
#[derive(Clone, Copy, Debug)]
pub(super) enum When {
    /// The `n`th call and every later one, as a limit that stays exhausted does.
    FromNth(usize),
    /// Only the `n`th call.
    Nth(usize),
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
    pub(super) fn specification(&self) -> String {
        let effect: String = match self.effect {
            Effect::SlowReturn(microseconds) => format!("delay_exit={microseconds}us"),
            Effect::Fail(error) => format!("error={error}"),
            Effect::Kill => String::from("signal=SIGKILL"),
        };
        let when: String = match self.when {
            When::FromNth(call) => format!("{call}+"),
            When::Nth(call) => format!("{call}"),
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
/// Run one explicit-path commit with the fault injected and `environment` added to the wrapper's own, for a control
///  that sets a bound shorter than the fault's delay.
pub(super) fn run_injected_with_environment(
    fixture: &Fixture,
    repo: &Path,
    label: &str,
    fault: &Injection,
    environment: &[(&str, &str)],
) -> Ran {
    let trace: PathBuf = fixture.root.join(format!("trace-{label}.txt"));
    let mut command: Command = traced(
        fixture,
        repo,
        &COMMIT_ARGUMENTS,
        fault.syscall,
        Some(fault),
        fault.follow_forks,
        trace.as_path(),
    );
    for (name, value) in environment {
        command.env(name, value);
    }
    let output: Output = run_to_end(&mut command);
    let log: String = std::fs::read_to_string(&trace).unwrap_or_default();
    return Ran { output, trace: log };
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
/// The lock files and wrapper directories in `.git`, for a failure message that must say what a first run left.
fn names_in_git_directory(repo: &Path) -> String {
    let mut names: Vec<String> = Vec::new();
    for directory in [repo.join(".git"), repo.join(".git/cli-git-transactions"), repo.join(".git/refs/heads")] {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            if name.contains("lock") || name.starts_with("monochromatic-git-policy-cli") || name.starts_with("cli-git") || directory.ends_with("cli-git-transactions") || directory.ends_with("heads") {
                names.push(format!("{}/{name}", directory.strip_prefix(repo).unwrap_or(&directory).display()));
            }
        }
    }
    names.sort();
    return names.join(", ");
}

/// What the repository must show after any first run that met a fault, whatever the fault was:
///  the next ordinary command succeeds with the commit landed or not made,
///  the repository passes `git fsck`,
///  a further commit lands,
///  and nothing of the wrapper stays behind once that commit has landed.
///  `said` is what the first run printed, for the failure message.
pub(super) fn verify_recovery(fixture: &Fixture, repo: &Path, label: &str, said: &str) -> Result<(), String> {
    return verify_recovery_with(fixture, repo, label, said, run_commit);
}

/// `verify_recovery` with the way each command is run given. The settled runner (`run_commit`) waits, as every wrapped command of
///  the fixture does, for the after-commit work the service claimed; a service that was killed leaves its claims until their
///  lease runs out (`service_work.rs`), so a control that kills the service runs its commands with a runner that does not wait.
pub(super) fn verify_recovery_with(
    fixture: &Fixture,
    repo: &Path,
    label: &str,
    said: &str,
    run_commit: fn(&Fixture, &Path, &[&str]) -> Observed,
) -> Result<(), String> {
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
    // Through the wrapper, as a user stages.
    let staged: Observed = run_commit(fixture, repo, &["add", "--", "b.txt"]);
    if staged.code != Some(0) {
        return Err(format!(
            "{label}: staging the next file failed after the first run, exit {:?}: {}\nfirst run said: {said}\nleft in .git: {}",
            staged.code,
            String::from_utf8_lossy(&staged.stderr),
            names_in_git_directory(repo)
        ));
    }
    let again: Observed = run_commit(fixture, repo, &["commit", "--quiet", "--message=y", "--", "b.txt"]);
    if again.code != Some(0) {
        return Err(format!(
            "{label}: a further commit must land, exit {:?}: {}\nfirst run said: {said}\nleft in .git: {}",
            again.code,
            String::from_utf8_lossy(&again.stderr),
            names_in_git_directory(repo)
        ));
    }
    let leftovers = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { assert_nothing_left(fixture, repo); }));
    if let Err(payload) = leftovers {
        return Err(format!("{label}: {}; first run said: {said}", panic_text(payload.as_ref())));
    }
    return Ok(());
}
