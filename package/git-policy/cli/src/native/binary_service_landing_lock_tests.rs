//! What:
//!  A landing through the service while a foreign `git commit` holds the repository's index lock, with its editor open.
//! Why:
//!  Git's own commit takes the index lock before it moves the branch and gives it up after the index is written. A landing that
//!  moves the branch first and takes the lock afterwards leaves the index showing content a landed commit replaced for as long
//!  as the foreign holder keeps the lock (found by the concurrent end-to-end scenarios `foreign-index-lock-holder` and
//!  `foreign-index-lock-unproven`). The service therefore takes the lock first, waits for a holder without polling, and gives up
//!  with nothing changed when the holder outlives the limit.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::runtime_for;
use super::service_landing_selection::base_repository;
use super::support::{Fixture, direct, executable, fixture, git, observe, spawn_owned, while_test_lives};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SERVICE: &str = env!("CARGO_BIN_EXE_cli-git-service");
const PATIENCE: Duration = Duration::from_secs(60);

/// A service process that is stopped when the control ends, whether it passed or panicked.
struct Held(Child);

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

/// A service that waits `limit` seconds for a foreign index lock, as the unit file's environment would say.
fn service_with_limit(fixture: &Fixture, name: &str, limit: &str) -> Held {
    let runtime: PathBuf = runtime_for(name);
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(runtime.as_path(), git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let child: Child = Command::new(SERVICE)
        .args(["--journal"])
        .arg(fixture.root.join("journal"))
        .arg("--path")
        .arg(&socket)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("CLI_GIT_LOCK_WAIT_SECONDS", limit)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return Held(child);
}

/// A foreign `git commit` of `b.txt` whose editor runs `editor_body` (a shell script that ends by writing the message into
///  its first argument), started once its lock exists.
/// The commit leads a process group of its own that dies with the test.
fn foreign_commit_with_editor(fixture: &Fixture, repo: &Path, editor_body: &str) -> Child {
    let editor: PathBuf = fixture.root.join("holding-editor");
    executable(editor.as_path(), format!("#!/bin/sh\n{editor_body}printf 'foreign\\n' > \"$1\"\n").as_bytes());
    git(fixture, repo, &["config", "core.editor", editor.to_str().expect("UTF-8")]);
    // Git 2.54 can leave its process number beside the lock, which is what lets the service name the holder.
    git(fixture, repo, &["config", "core.lockfilePid", "true"]);
    std::fs::write(repo.join("b.txt"), "foreign edit\n").expect("b.txt");
    let child: Child = spawn_owned(direct(fixture).current_dir(repo).args(["commit", "--all", "--quiet"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped())).expect("foreign commit");
    let began: Instant = Instant::now();
    while !repo.join(".git/index.lock").exists() {
        assert!(began.elapsed() < PATIENCE, "the foreign commit never took the lock");
        std::thread::sleep(Duration::from_millis(5));
    }
    return child;
}

/// A foreign `git commit` of `b.txt` whose editor stays open for `hold` seconds, started once its lock exists.
pub(super) fn foreign_commit(fixture: &Fixture, repo: &Path, hold: u32) -> Child {
    return foreign_commit_with_editor(fixture, repo, format!("sleep {hold}\n").as_str());
}

/// What:
///  A foreign `git commit` of `b.txt` that keeps its lock until the returned file exists and then for `hold` more seconds,
/// started once its lock exists.
/// Why:
///   The commit that is to wait for the lock may reach it at any time on a loaded host; a holder that lets go after a fixed time
///  can be gone before it does. This holder lets go only after the test has seen the commit waiting and created the file, and
///  it lets go on its own if the test ends first, so the wait never depends on how fast the host runs the commit.
pub(super) fn foreign_commit_until_released(fixture: &Fixture, repo: &Path, hold: u32) -> (Child, PathBuf) {
    let release: PathBuf = fixture.root.join("holder-release");
    let body: String = format!("while [ ! -e '{}' ] && {}; do sleep 0.02; done\nsleep {hold}\n", release.display(), while_test_lives(fixture.root.as_path()));
    return (foreign_commit_with_editor(fixture, repo, body.as_str()), release);
}

/// The holder keeps the lock for longer than the old five tries of a tenth of a second: the commit waits, lands on top of the
///  holder's commit with an index that shows both, and the holder's own commit succeeds.
#[test]
fn a_commit_waits_for_a_foreign_commit_that_holds_the_index_and_lands_on_top_of_it() {
    let fixture: Fixture = fixture("landing-lock-holder");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("landing-lock-{}", std::process::id());
    let _service: Held = service_with_limit(&fixture, name.as_str(), "60");
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let holder: Child = foreign_commit(&fixture, &repo, 2);
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    let served = observe(&mut command, b"");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "the holder's own commit succeeds: {}", String::from_utf8_lossy(&held.stderr));
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    assert_eq!(git_text(&fixture, &repo, &["log", "--reverse", "--format=%s", format!("{base}..HEAD").as_str()]), "foreign\nmine\n");
    assert_eq!(git_text(&fixture, &repo, &["status", "--short"]), "", "the index shows both commits");
    assert!(!repo.join(".git/index.lock").exists(), "no lock is left");
    remove_settled(&fixture);
}

/// The seconds the commit is stalled after the foreign commit took the lock in the control of a late commit:
/// a loaded host delays the commit for longer than a stand-in that keeps the lock for a fixed time lasts
///  (a failing run at a load average of 38 to 44 spent 6760 ms in one phase against a hold of 4000 ms, 2026-11-03).
const STALL_SECONDS: u64 = 8;

/// The seconds the foreign commit keeps the lock after the control has released it, so that its commit ends on its own.
const RELEASED_HOLD_SECONDS: u32 = 0;

/// The holder outlives the limit: the commit is refused with a plain message that names the holder, nothing moved, the
///  holder's lock stands, and the holder's commit then succeeds on its own.
/// The holder keeps the lock until the control has seen the refusal, so the outcome does not depend on how fast the host runs
///  the commit.
#[test]
fn a_foreign_holder_that_outlives_the_limit_makes_the_commit_fail_with_nothing_moved() {
    refused_while_a_foreign_holder_outlives_the_limit("landing-lock-limit", Duration::ZERO);
}

/// The same claim when the host stalls the commit after the foreign commit took the lock:
/// however late the commit reaches the lock, the holder is still there and the commit is refused.
#[test]
fn a_commit_that_reaches_a_foreign_holder_late_is_still_refused_at_the_limit() {
    refused_while_a_foreign_holder_outlives_the_limit("landing-lock-late", Duration::from_secs(STALL_SECONDS));
}

/// Runs the claim of the two controls: the commit starts `stall` after the foreign commit took the lock.
fn refused_while_a_foreign_holder_outlives_the_limit(control: &str, stall: Duration) {
    let fixture: Fixture = fixture(control);
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("{control}-{}", std::process::id());
    let _service: Held = service_with_limit(&fixture, name.as_str(), "1");
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, RELEASED_HOLD_SECONDS);
    std::thread::sleep(stall);
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    let served = observe(&mut command, b"");
    let stderr: String = String::from_utf8_lossy(&served.stderr).into_owned();
    assert_ne!(served.code, Some(0), "{stderr}");
    assert!(stderr.contains("is using this repository's index") && stderr.contains("was not made"), "{stderr}");
    assert!(stderr.contains("Process "), "the holder is named: {stderr}");
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim(), base, "nothing moved while the holder still held the lock");
    std::fs::write(&release, b"").expect("the foreign commit is told to let go");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    assert_eq!(git_text(&fixture, &repo, &["log", "--format=%s", format!("{base}..HEAD").as_str()]), "foreign\n", "only the holder's commit is on the branch");
    remove_settled(&fixture);
}
