//! What:
//!  Index-writer coordination through the built executable:
//!  a `git add` waits for a landing that
//!  holds the lock,
//!  a live foreign `index.lock` is waited for and named,
//!  a dead one is refused with
//!  exit status 2,
//!  and a `git rebase --exec 'git add'` finishes because the nested wrapper inherits
//!  the lease instead of waiting on the lock its ancestor holds.
//! Why:
//!  Without the lease hand-off the nested `git add` waits for its own ancestor forever,
//!  and a
//!  writer that skips the wait would race a landing commit with nothing to show it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await runWrapped(['rebase', '--exec', 'git add -- b.txt', base])).toMatchObject({ code: 0 });
//! ```

/// The shared fixtures and bounded process helpers.
use super::support::{
    Fixture, fixture, git, porcelain, remove, repository, wrapped,
};
use git_policy_cli::owner_lock::{SilentWait, acquire_owner_lock_for};
use git_policy_cli::transaction_registry::ensure_transaction_root;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

/// How long a holder keeps the lock in the controls before the test releases it.
const HOLD: Duration = Duration::from_millis(400);

/// Hold the landing lock as this live process until the returned lock is released.
fn hold_landing(repo: &Path) -> git_policy_cli::owner_lock::OwnerLock {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    ensure_transaction_root(registry.as_path()).expect("registry");
    // `mut` is required by the notice interface; this wait is silent, as the incumbent's.
    let mut silent: SilentWait = SilentWait;
    return acquire_owner_lock_for(
        registry.join("landing.lock").as_path(),
        Duration::from_millis(5),
        &mut silent,
        Duration::from_secs(20),
    )
    .expect("the test holds the landing lock");
}

/// Commit `name` with the given text,
///  so the repository has a history to rebase.
fn commit_file(fixture: &Fixture, repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("file");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", "--message", name]);
}

/// Wait for a spawned command to finish,
///  with its output.
fn finish(child: Child) -> Output {
    return child.wait_with_output().expect("the command finishes");
}

/// A `git add` waits while a landing holds the lock,
///  then stages the file once the landing releases it.
#[test]
fn git_add_waits_for_a_landing_then_stages() {
    let fixture: Fixture = fixture("coordination-landing-wait");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let held = hold_landing(repo.as_path());
    let mut command: Command = wrapped(&fixture);
    command.current_dir(&repo).args(["add", "--", "a.txt"]);
    let child: Child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("start the wrapped add");
    std::thread::sleep(HOLD);
    held.release().expect("release the landing lock");
    let output: Output = finish(child);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(porcelain(&fixture, repo.as_path()).trim(), "A  a.txt");
    remove(&fixture);
}

/// `git rebase --exec 'git add -- b.txt'` finishes:
///  the wrapper that the rebase's exec starts inherits
/// the lease and does not wait on the lock its ancestor holds.
#[test]
fn a_nested_git_add_inside_rebase_does_not_deadlock() {
    let fixture: Fixture = fixture("coordination-rebase");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    commit_file(&fixture, repo.as_path(), "a.txt", "one\n");
    commit_file(&fixture, repo.as_path(), "b.txt", "two\n");
    // `git rebase` puts its own libexec directory first on PATH for exec steps, so a bare `git` there
    // is real Git and never reaches the wrapper; the exec step names the wrapper's path instead.
    let wrapper_path: String = fixture.root.join("bin/git").display().to_string();
    let exec_step: String = format!("{wrapper_path} add -- b.txt");
    let mut command: Command = wrapped(&fixture);
    command
        .current_dir(&repo)
        .args(["rebase", "--exec", exec_step.as_str(), "HEAD~1"]);
    let output: Output = finish(command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("start the wrapped rebase"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "the rebase must finish, not time out: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    remove(&fixture);
}

/// A dead owner's `index.lock` is refused with `index-lock-unproven-owner`,
///  exit status 2,
///  and the
/// lock is left where it is.
#[test]
fn a_dead_foreign_lock_is_refused_and_left_in_place() {
    let fixture: Fixture = fixture("coordination-dead-lock");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("foreign lock");
    std::fs::write(repo.join(".git/index~pid.lock"), "pid 2147483646\n").expect("dead PID file");
    let mut command: Command = wrapped(&fixture);
    command.current_dir(&repo).args(["add", "--", "a.txt"]);
    let output: Output = finish(command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("start the wrapped add"));
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("\"code\":\"index-lock-unproven-owner\""), "{stderr}");
    assert!(stderr.contains("did not run git add"), "{stderr}");
    assert!(repo.join(".git/index.lock").exists(), "the lock is left in place");
    remove(&fixture);
}

/// A live owner that holds the lock open is waited for without limit,
///  and its name is printed once.
#[test]
fn a_live_foreign_owner_is_waited_for_and_named() {
    let fixture: Fixture = fixture("coordination-live-owner");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let lock: PathBuf = repo.join(".git/index.lock");
    // The owner starts before it takes the lock, as Git does, so its start precedes the lock's change.
    // The holder's output goes nowhere and it execs its sleep, so killing it leaves no orphan holding a pipe.
    let mut holder: Child = Command::new("sh")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-c",
            "while [ ! -e \"$0\" ]; do sleep 0.05; done; exec 3<\"$0\"; exec sleep 30",
            lock.to_str().expect("UTF-8 path"),
        ])
        .spawn()
        .expect("start the live owner");
    std::thread::sleep(Duration::from_millis(200));
    std::fs::write(&lock, b"").expect("foreign lock");
    std::fs::write(repo.join(".git/index~pid.lock"), format!("pid {}\n", holder.id()))
        .expect("PID file naming the live owner");
    let mut command: Command = wrapped(&fixture);
    command.current_dir(&repo).args(["add", "--", "a.txt"]);
    let mut child: Child = command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().expect("start the wrapped add");
    // The owner keeps the lock until the wrapper has named it on standard error, so the wrapper provably waited for the
    // owner, however slowly a loaded host starts it.
    let mut reader: std::io::BufReader<std::process::ChildStderr> = std::io::BufReader::new(child.stderr.take().expect("piped"));
    let mut stderr: String = String::new();
    loop {
        let mut line: String = String::new();
        let read: usize = std::io::BufRead::read_line(&mut reader, &mut line).expect("a line of standard error");
        assert!(read > 0, "the wrapper ended before it waited for the owner: {stderr}");
        stderr.push_str(line.as_str());
        if line.contains("cli-git: waiting for PID") {
            break;
        }
    }
    let _ = holder.kill();
    let _ = holder.wait();
    std::fs::remove_file(&lock).expect("the owner released the lock");
    std::fs::remove_file(repo.join(".git/index~pid.lock")).expect("the owner removed its PID file");
    std::io::Read::read_to_string(&mut reader, &mut stderr).expect("the rest of standard error");
    let status = child.wait().expect("the wrapped add ends");
    assert_eq!(status.code(), Some(0), "{stderr}");
    // The first line is the only one named "waiting for PID": a repeat reads "still waiting" and is an interval of ten
    // seconds away.
    assert_eq!(
        stderr.matches("cli-git: waiting for PID").count(),
        1,
        "the holder is named once: {stderr}"
    );
    remove(&fixture);
}
