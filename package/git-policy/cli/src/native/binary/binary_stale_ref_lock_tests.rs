//! What:
//!  Consumer-level controls for the person's own `git fetch` after a Git was killed while it held reference locks.
//! Why:
//!  The wrapper becomes Git for a plain fetch,
//! so what the person sees is the wrapper's notice and then Git's own words.
//! The
//!  locks come from a real Git killed with `SIGKILL`,
//! the command is the built wrapper,
//! and a lock that a running Git holds
//!  must survive it.
//! Where the wrapper runs in the initial PID namespace it can prove a pid file's process gone and removes
//!  the lock;
//! anywhere else (a container) it cannot,
//! and it describes the lock and leaves it.
//! Each control pins the branch its
//!  environment takes,
//! and the unit controls pin both branches everywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! killHeldFetch(); const seen = await runWrapped(['fetch', '--prune']); expect(seen.stderr).toContain('removed the reference lock');
//! ```

use super::support::{Fixture, Observed, direct, fixture, git, remove, repository, run_wrapped};
use git_policy_cli::stale_ref_lock_judge::MARGIN;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

/// The longest a control waits for a Git it started to take its locks.
const PATIENCE: Duration = Duration::from_secs(60);

/// The tracking reference the remote no longer has.
const GONE_REF: &str = "refs/remotes/origin/gone";

/// The hook that holds a transaction where every lock is taken and writes the process that holds it.
const HOLD_HOOK: &str = "#!/bin/sh\nif [ \"$1\" = prepared ] && [ -n \"$STALE_LOCK_READY\" ]; then\n  echo $$ > \"$STALE_LOCK_READY.part\"\n  mv \"$STALE_LOCK_READY.part\" \"$STALE_LOCK_READY\"\n  exec sleep 600\nfi\nexit 0\n";

/// A Git a control started,
/// ended when the control ends however it ends.
struct Held {
    /// The Git.
    child: Child,
    /// Its input,
    /// kept open so a transaction waits.
    _input: Option<ChildStdin>,
    /// The process the hook runs as.
    hook: Option<i32>,
}

impl Held {
    /// Kill the Git with `SIGKILL`,
    /// so it runs no cleanup,
    /// and its hook.
    fn kill(&mut self) {
        self.child.kill().expect("SIGKILL the Git");
        self.child.wait().expect("reap the Git");
        if let Some(hook) = self.hook.take() {
            // SAFETY: `kill` takes a process number and a signal; the number is the hook of the Git this control started.
            unsafe { libc::kill(hook, libc::SIGKILL); }
        }
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.child.kill();
        let _reaped = self.child.wait();
        if let Some(hook) = self.hook.take() {
            // SAFETY: as above.
            unsafe { libc::kill(hook, libc::SIGKILL); }
        }
    }
}

/// A repository whose remote lost a branch,
/// with the tracking references packed.
struct Stage {
    /// The fixture.
    fixture: Fixture,
    /// The repository.
    repo: PathBuf,
}

impl Stage {
    /// The Git directory.
    fn git_dir(&self) -> PathBuf {
        return self.repo.join(".git");
    }

    /// The lock of the deleted tracking reference.
    fn gone_lock(&self) -> PathBuf {
        return self.git_dir().join(format!("{GONE_REF}.lock"));
    }
}

/// Build the stage;
/// `lockfile_pid` writes `core.lockfilePid` into the repository's own configuration.
fn stage(name: &str, lockfile_pid: bool) -> Stage {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["branch", "gone"]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "main", "gone"]);
    git(&fixture, repo.as_path(), &["fetch", "--quiet", "origin"]);
    git(&fixture, repo.as_path(), &["pack-refs", "--all"]);
    git(&fixture, remote.as_path(), &["branch", "--delete", "--force", "gone"]);
    if lockfile_pid {
        git(&fixture, repo.as_path(), &["config", "core.lockfilePid", "true"]);
    }
    return Stage { fixture, repo };
}

/// Wait for a condition,
/// at most `PATIENCE`.
fn wait_for(what: &str, condition: impl Fn() -> bool) {
    let started: Instant = Instant::now();
    while !condition() {
        assert!(started.elapsed() < PATIENCE, "gave up waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Start `git fetch --prune origin` straight on real Git,
/// held by the hook where its transaction has taken its locks.
fn hold_fetch(stage: &Stage) -> Held {
    let ready: PathBuf = stage.fixture.root.join("ready");
    let hook: PathBuf = stage.git_dir().join("hooks").join("reference-transaction");
    std::fs::write(hook.as_path(), HOLD_HOOK).expect("hook");
    std::fs::set_permissions(hook.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("hook mode");
    let mut command: Command = direct(&stage.fixture);
    command.current_dir(stage.repo.as_path()).args(["fetch", "--prune", "--quiet", "origin"]).env("STALE_LOCK_READY", ready.as_os_str());
    let child: Child = command.spawn().expect("start git fetch");
    let mut held: Held = Held { child, _input: None, hook: None };
    wait_for("the fetch to take its locks", || { return ready.exists(); });
    held.hook = std::fs::read_to_string(ready.as_path()).expect("hook pid").trim().parse::<i32>().ok();
    return held;
}

/// Start `git update-ref --stdin` on real Git with `commands` prepared,
/// and keep it waiting with its locks held.
fn hold_transaction(stage: &Stage, commands: &[&str]) -> Held {
    let mut command: Command = direct(&stage.fixture);
    command.current_dir(stage.repo.as_path()).args(["update-ref", "--stdin"]).stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child: Child = command.spawn().expect("start git update-ref");
    let mut input: ChildStdin = child.stdin.take().expect("input");
    let output = child.stdout.take().expect("output");
    input.write_all(format!("start\n{}\nprepare\n", commands.join("\n")).as_bytes()).expect("send");
    input.flush().expect("flush");
    let (tell, hear) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            if line == "prepare: ok" {
                let _told = tell.send(());
            }
        }
    });
    let held: Held = Held { child, _input: Some(input), hook: None };
    hear.recv_timeout(PATIENCE).expect("the transaction to be prepared");
    return held;
}

/// Let the locks grow older than the margin the wrapper waits for,
/// so only the rules decide.
fn age_past_margin() {
    std::thread::sleep(MARGIN + Duration::from_millis(500));
}

/// Remove the hook and run the fetch the way the person does,
/// through the wrapper.
fn wrapped_fetch(stage: &Stage) -> Observed {
    let _removed = std::fs::remove_file(stage.git_dir().join("hooks").join("reference-transaction"));
    return run_wrapped(&stage.fixture, stage.repo.as_path(), &["fetch", "--prune", "--quiet", "origin"]);
}

/// The text of what the caller saw on standard error.
fn stderr_text(seen: &Observed) -> String {
    return String::from_utf8_lossy(seen.stderr.as_slice()).into_owned();
}

/// A lock left by a killed fetch with a pid file beside it:
/// the wrapper names it and gives the command,
/// because nothing ties that number to this lock,
/// and Git refuses.
#[test]
fn a_fetch_after_a_killed_fetch_keeps_the_locks_and_names_them() {
    let stage: Stage = stage("stale-ref-locks-killed-fetch", true);
    let mut held: Held = hold_fetch(&stage);
    assert!(stage.gone_lock().exists(), "the fetch holds the lock before it is killed");
    held.kill();
    age_past_margin();
    let seen: Observed = wrapped_fetch(&stage);
    let words: String = stderr_text(&seen);
    assert_ne!(seen.code, Some(0), "{words}");
    assert!(words.contains("monochromatic-git-policy-cli: left the reference lock"), "{words}");
    assert!(words.contains("rm -- "), "{words}");
    assert!(words.contains("cannot lock ref") || words.contains("Unable to create"), "Git's own refusal follows: {words}");
    assert!(stage.gone_lock().exists());
    remove(&stage.fixture);
}

/// With no pid file nothing proves the holder gone anywhere:
/// the lock stays,
/// the notice gives the command,
/// Git refuses.
#[test]
fn a_fetch_after_a_killed_fetch_with_no_pid_file_keeps_the_lock_and_gives_the_command() {
    let stage: Stage = stage("stale-ref-locks-no-pid-file", false);
    let mut held: Held = hold_fetch(&stage);
    held.kill();
    age_past_margin();
    let seen: Observed = wrapped_fetch(&stage);
    let words: String = stderr_text(&seen);
    assert_ne!(seen.code, Some(0), "{words}");
    assert!(words.contains("monochromatic-git-policy-cli: left the reference lock"), "{words}");
    assert!(words.contains(&format!("monochromatic-git-policy-cli:   rm -- {}", stage.gone_lock().display())) || words.contains("monochromatic-git-policy-cli:   rm -- "), "{words}");
    assert!(words.contains("cannot lock ref") || words.contains("Unable to create"), "Git's own refusal follows: {words}");
    assert!(stage.gone_lock().exists());
    remove(&stage.fixture);
}

/// A Git that is still running holds its locks with no descriptor open,
/// and a fetch must leave them.
#[test]
fn a_fetch_leaves_the_locks_of_a_git_that_is_still_running() {
    let stage: Stage = stage("stale-ref-locks-running", true);
    let held: Held = hold_transaction(&stage, &["delete refs/remotes/origin/gone"]);
    age_past_margin();
    let seen: Observed = wrapped_fetch(&stage);
    let words: String = stderr_text(&seen);
    assert!(stage.gone_lock().exists(), "{words}");
    assert!(!words.contains("monochromatic-git-policy-cli: removed"), "{words}");
    drop(held);
    remove(&stage.fixture);
}

/// A command that is not a fetch or a push is not preceded by a notice,
/// and no lock is judged for it.
#[test]
fn another_command_gets_no_notice_and_clears_nothing() {
    let stage: Stage = stage("stale-ref-locks-other-command", true);
    let mut held: Held = hold_fetch(&stage);
    held.kill();
    age_past_margin();
    let seen: Observed = run_wrapped(&stage.fixture, stage.repo.as_path(), &["status", "--short"]);
    assert!(!stderr_text(&seen).contains("reference lock"), "{}", stderr_text(&seen));
    assert!(stage.gone_lock().exists());
    remove(&stage.fixture);
}
