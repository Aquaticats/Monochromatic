//! What:
//!  Real Git processes that hold reference locks in a disposable repository and are then killed with `SIGKILL`.
//! Why:
//!  The shape of a lock left by a killed Git is a fact about Git,
//! not about this crate,
//! so the controls produce it with Git:
//!  a `git fetch --prune` held by a `reference-transaction` hook at the point where it has taken its locks,
//! and a
//!  `git update-ref --stdin` transaction held after `prepare`.
//! Nothing here touches a real repository (rule THR).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = await startFetchHeldByHook(stage); held.kill('SIGKILL'); expect(lockFiles(stage.gitDir)).not.toEqual([]);
//! ```
#![cfg(unix)]

use crate::test_support::{executable, fixture, git, remove, repository};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

/// The longest a control waits for a Git it started to reach the point where it holds its locks.
pub const HOLD_PATIENCE: Duration = Duration::from_secs(60);

/// The tracking reference the remote no longer has,
/// so `fetch --prune` deletes it and locks it empty.
pub const GONE_REF: &str = "refs/remotes/origin/gone";

/// A repository whose remote lost a branch,
/// with every tracking reference packed.
pub struct Stage {
    /// The fixture directory,
    /// removed at the end.
    pub root: PathBuf,
    /// The working repository.
    pub repo: PathBuf,
    /// Its Git directory.
    pub git_dir: PathBuf,
    /// The file the hook writes its own process number into once the fetch holds its locks.
    pub ready: PathBuf,
}

impl Stage {
    /// The lock file of a reference.
    pub fn lock_of(&self, reference: &str) -> PathBuf {
        return self.git_dir.join(format!("{reference}.lock"));
    }

    /// The pid file beside the lock of a reference.
    pub fn pid_file_of(&self, reference: &str) -> PathBuf {
        return self.git_dir.join(format!("{reference}~pid.lock"));
    }

    /// The lock of the packed references,
    /// and the staging file Git writes beside it.
    pub fn packed_lock(&self) -> PathBuf {
        return self.git_dir.join("packed-refs.lock");
    }

    /// The reftable backend's lock of its list of tables,
    /// which every writer of a reference takes (`reftable/stack.c`,
    /// `reftable_stack_init_addition`).
    pub fn tables_lock(&self) -> PathBuf {
        return self.git_dir.join("reftable").join("tables.list.lock");
    }

    /// The pid file Git writes beside that lock under `core.lockfilePid`.
    pub fn tables_pid_file(&self) -> PathBuf {
        return self.git_dir.join("reftable").join("tables.list~pid.lock");
    }

    /// The staging file of the packed references.
    pub fn packed_staging(&self) -> PathBuf {
        return self.git_dir.join("packed-refs.new");
    }

    /// The Git directory as the roots a clearing pass searches.
    pub fn roots(&self) -> Vec<PathBuf> {
        return vec![self.git_dir.clone()];
    }

    /// Remove the fixture.
    pub fn finish(self) {
        remove(self.root.as_path());
    }
}

/// Build the stage:
/// a bare remote with `main` and `gone`,
/// a clone-like repository that fetched both and packed them,
/// and
///  then a remote that no longer has `gone`.
/// `lockfile_pid` writes `core.lockfilePid` into the repository's own
///  configuration,
/// as the wrapper turns it on for the Git it starts.
pub fn stage(name: &str, lockfile_pid: bool) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(repo.as_path(), &["branch", "gone"]);
    git(repo.as_path(), &["push", "--quiet", "origin", "main", "gone"]);
    git(repo.as_path(), &["fetch", "--quiet", "origin"]);
    git(repo.as_path(), &["pack-refs", "--all"]);
    git(remote.as_path(), &["branch", "--delete", "--force", "gone"]);
    if lockfile_pid {
        git(repo.as_path(), &["config", "core.lockfilePid", "true"]);
    }
    let git_dir: PathBuf = repo.join(".git");
    let ready: PathBuf = root.join("ready");
    return Stage { root, repo, git_dir, ready };
}

/// Build a stage whose references the reftable backend keeps:
/// a repository with one commit and no remote,
/// which is all a held `update-ref --stdin` needs.
/// The reftable backend keeps its references and its lock in the `reftable` directory of the Git directory,
/// so none of the files a files-backend repository locks exists.
pub fn reftable_stage(name: &str, lockfile_pid: bool) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    if lockfile_pid {
        git(repo.as_path(), &["config", "core.lockfilePid", "true"]);
    }
    let git_dir: PathBuf = repo.join(".git");
    let ready: PathBuf = root.join("ready");
    return Stage { root, repo, git_dir, ready };
}

/// A Git the control started and must end however the control ends.
pub struct Held {
    /// The Git process.
    child: Child,
    /// The input of a transaction held open,
    /// kept so the Git does not see the end of its input.
    _input: Option<ChildStdin>,
    /// The process number the hook wrote,
    /// when the Git is held by a hook.
    hook: Option<i32>,
}

impl Held {
    /// The process number of the Git.
    pub fn pid(&self) -> i32 {
        return i32::try_from(self.child.id()).expect("process number");
    }

    /// Kill the Git with `SIGKILL` and wait for it,
    /// so it can run no cleanup,
    /// and end its hook.
    pub fn kill(&mut self) {
        self.child.kill().expect("SIGKILL the Git");
        self.child.wait().expect("reap the Git");
        if let Some(hook) = self.hook.take() {
            // SAFETY: `kill` takes a process number and a signal and touches no memory; the number is the hook of the Git
            // this control started.
            unsafe { libc::kill(hook, libc::SIGKILL); }
        }
    }
}

/// Whatever the control did,
/// no process it started is left.
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

/// Wait for a condition,
/// at most `HOLD_PATIENCE`.
fn wait_for(what: &str, condition: impl Fn() -> bool) {
    let started: Instant = Instant::now();
    while !condition() {
        assert!(started.elapsed() < HOLD_PATIENCE, "gave up waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A Git command in the stage with no system or global configuration.
fn git_command(stage: &Stage) -> Command {
    let mut command: Command = Command::new(crate::test_support::REAL_GIT);
    command
        .current_dir(stage.repo.as_path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid");
    return command;
}

/// The hook that holds a transaction at its `prepared` state,
/// where every lock is taken,
/// and tells which process holds.
const HOLD_HOOK: &str = "#!/bin/sh\nif [ \"$1\" = prepared ] && [ -n \"$STALE_LOCK_READY\" ]; then\n  echo $$ > \"$STALE_LOCK_READY.part\"\n  mv \"$STALE_LOCK_READY.part\" \"$STALE_LOCK_READY\"\n  exec sleep 600\nfi\nexit 0\n";

/// Start `git fetch --prune origin` and hold it at the moment its transaction has taken its locks.
pub fn hold_fetch(stage: &Stage) -> Held {
    let hooks: PathBuf = stage.git_dir.join("hooks");
    executable(hooks.join("reference-transaction").as_path(), HOLD_HOOK.as_bytes());
    let mut command: Command = git_command(stage);
    command.args(["fetch", "--prune", "--quiet", "origin"]).env("STALE_LOCK_READY", stage.ready.as_os_str());
    let child: Child = command.spawn().expect("start git fetch");
    let mut held: Held = Held { child, _input: None, hook: None };
    let ready: PathBuf = stage.ready.clone();
    wait_for("the fetch to take its locks", || { return ready.exists(); });
    held.hook = std::fs::read_to_string(stage.ready.as_path()).expect("hook pid").trim().parse::<i32>().ok();
    return held;
}

/// Start `git update-ref --stdin`,
/// send it `commands` and `prepare`,
/// and keep it waiting with its locks held.
pub fn hold_transaction(stage: &Stage, commands: &[&str]) -> Held {
    let mut command: Command = git_command(stage);
    command.args(["update-ref", "--stdin"]).stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child: Child = command.spawn().expect("start git update-ref");
    let mut input: ChildStdin = child.stdin.take().expect("the input");
    let output = child.stdout.take().expect("the output");
    let script: String = format!("start\n{}\nprepare\n", commands.join("\n"));
    input.write_all(script.as_bytes()).expect("send the transaction");
    input.flush().expect("flush the transaction");
    let (tell, hear) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            if line == "prepare: ok" {
                let _told = tell.send(());
            }
        }
    });
    let held: Held = Held { child, _input: Some(input), hook: None };
    hear.recv_timeout(HOLD_PATIENCE).expect("the transaction to be prepared");
    return held;
}

/// A long-lived process whose command name is `git`,
/// that holds no lock:
/// a Git that waits for input.
pub fn idle_git(stage: &Stage) -> Held {
    let mut command: Command = git_command(stage);
    command.args(["update-ref", "--stdin"]).stdin(Stdio::piped());
    let mut child: Child = command.spawn().expect("start git update-ref");
    let input: ChildStdin = child.stdin.take().expect("the input");
    return Held { child, _input: Some(input), hook: None };
}

/// Create a regular file with `content`.
pub fn write_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
    std::fs::write(path, content).expect("write the file");
}

/// A helper process of a control,
/// ended when the control ends however it ends.
pub struct Helper(pub Child);

impl Helper {
    /// The process number as the pid files carry it.
    pub fn pid(&self) -> i64 {
        return i64::from(self.0.id());
    }
}

/// Whatever the control did,
/// the helper does not outlive it.
impl Drop for Helper {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _reaped = self.0.wait();
    }
}

/// A judge that sees the processes of this kernel completely,
/// takes any lock as old enough,
/// and trusts local disks.
///  The scan for open holders is given no process:
/// every control of this module plants the processes it judges,
/// and a scan of the whole machine would let another program's Git veto a clearance
/// (a non-dumpable process denies its descriptors to its own account);
/// the veto keeps its own controls,
/// which name the holder they plant.
pub fn judge() -> crate::stale_ref_lock_judge::Judge {
    return crate::stale_ref_lock_judge::Judge { margin: Duration::ZERO, boot: None, scan_scope: Some(Vec::new()), ..crate::stale_ref_lock_judge::Judge::here() };
}

/// Make a new commit and start `git push origin main`,
/// held at the moment its transaction has taken the lock of the
///  tracking reference,
/// which is after the remote has taken the commit.
pub fn hold_push(stage: &Stage) -> Held {
    git(stage.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=ahead"]);
    let hooks: PathBuf = stage.git_dir.join("hooks");
    executable(hooks.join("reference-transaction").as_path(), HOLD_HOOK.as_bytes());
    let mut command: Command = git_command(stage);
    command.args(["push", "--quiet", "origin", "main"]).env("STALE_LOCK_READY", stage.ready.as_os_str());
    let child: Child = command.spawn().expect("start git push");
    let mut held: Held = Held { child, _input: None, hook: None };
    let ready: PathBuf = stage.ready.clone();
    wait_for("the push to take its locks", || { return ready.exists(); });
    held.hook = std::fs::read_to_string(stage.ready.as_path()).expect("hook pid").trim().parse::<i32>().ok();
    return held;
}

/// The command that runs real Git in the stage the way the wrapper's own runners start it:
/// no system or global
///  configuration,
/// the stage as the working directory.
pub fn plain_git(stage: &Stage, arguments: &[&str]) -> Command {
    let mut command: Command = git_command(stage);
    command.args(arguments);
    return command;
}
