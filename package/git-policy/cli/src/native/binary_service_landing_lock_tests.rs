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
use super::support::{Fixture, direct, executable, fixture, git, observe};
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

/// A foreign `git commit` of `b.txt` whose editor stays open for `hold` seconds, started once its lock exists.
pub(super) fn foreign_commit(fixture: &Fixture, repo: &Path, hold: u32) -> Child {
    let editor: PathBuf = fixture.root.join("holding-editor");
    executable(editor.as_path(), format!("#!/bin/sh\nsleep {hold}\nprintf 'foreign\\n' > \"$1\"\n").as_bytes());
    git(fixture, repo, &["config", "core.editor", editor.to_str().expect("UTF-8")]);
    // Git 2.54 can leave its process number beside the lock, which is what lets the service name the holder.
    git(fixture, repo, &["config", "core.lockfilePid", "true"]);
    std::fs::write(repo.join("b.txt"), "foreign edit\n").expect("b.txt");
    let child: Child = direct(fixture).current_dir(repo).args(["commit", "--all", "--quiet"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().expect("foreign commit");
    let began: Instant = Instant::now();
    while !repo.join(".git/index.lock").exists() {
        assert!(began.elapsed() < PATIENCE, "the foreign commit never took the lock");
        std::thread::sleep(Duration::from_millis(5));
    }
    return child;
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

/// The holder outlives the limit: the commit is refused with a plain message that names the holder, nothing moved, the
///  holder's lock stands, and the holder's commit then succeeds on its own.
#[test]
fn a_foreign_holder_that_outlives_the_limit_makes_the_commit_fail_with_nothing_moved() {
    let fixture: Fixture = fixture("landing-lock-limit");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("landing-lock-limit-{}", std::process::id());
    let _service: Held = service_with_limit(&fixture, name.as_str(), "1");
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let holder: Child = foreign_commit(&fixture, &repo, 4);
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    let served = observe(&mut command, b"");
    let stderr: String = String::from_utf8_lossy(&served.stderr).into_owned();
    assert_ne!(served.code, Some(0), "{stderr}");
    assert!(stderr.contains("is using this repository's index") && stderr.contains("was not made"), "{stderr}");
    assert!(stderr.contains("Process "), "the holder is named: {stderr}");
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim(), base, "nothing moved while the holder still held the lock");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    assert_eq!(git_text(&fixture, &repo, &["log", "--format=%s", format!("{base}..HEAD").as_str()]), "foreign\n", "only the holder's commit is on the branch");
    remove_settled(&fixture);
}
