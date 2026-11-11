//! What:
//!  A landing through the service while a foreign `git commit` holds the repository's index lock,
//! with its editor open.
//! Why:
//!  Git's own commit takes the index lock before it moves the branch and gives it up after the index is written.
//! A landing that
//!  moves the branch first and takes the lock afterwards leaves the index showing content a landed commit replaced for as long
//!  as the foreign holder keeps the lock (found by the concurrent end-to-end scenarios `foreign-index-lock-holder` and
//!  `foreign-index-lock-unproven`).
//! The service therefore takes the lock first and waits for a holder without polling:
//! for as
//!  long as a holder proven alive,
//! or an owner nothing proves alive,
//! keeps the lock.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::runtime_for;
use super::service_landing_selection::base_repository;
use super::support::{Fixture, direct_held, executable, fixture, git, observe, spawn_owned, while_test_lives};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const PATIENCE: Duration = Duration::from_secs(60);

/// A service process that is stopped when the control ends,
/// whether it passed or panicked.
pub(super) struct Held(Child);

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

/// A service for the control,
/// started the way the unit file starts it,
/// repeating its notices every `notice_seconds`.
fn service_for(fixture: &Fixture, name: &str, notice_seconds: &str) -> Held {
    return service_with_arguments(fixture, name, notice_seconds, &[]);
}

/// `service_for` with more arguments for the service program (a control gives a short lock ceiling this way).
pub(super) fn service_with_arguments(fixture: &Fixture, name: &str, notice_seconds: &str, extra: &[&str]) -> Held {
    let runtime: PathBuf = runtime_for(name);
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(runtime.as_path(), git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let child: Child = super::support::service_command()
        .args(["--journal"])
        .arg(fixture.root.join("journal"))
        .arg("--path")
        .arg(&socket)
        .args(extra)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", notice_seconds)
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
///  its first argument),
/// started once its lock exists.
/// The commit leads a process group of its own that dies with the test.
///
/// It names `b.txt`,
/// and only `b.txt`:
/// `--all` would stage every change in the worktree,
/// including the file the control
///  commits while the editor is open,
/// so the control's own change would land in the foreign commit and the landing that is
///  built again after the branch moved would meet the same file changed on both sides and refuse the commit
///  (`concurrent-commit/replay-conflict`,
/// 2026-11-08).
fn foreign_commit_with_editor(fixture: &Fixture, repo: &Path, editor_body: &str) -> Child {
    let editor: PathBuf = fixture.root.join("holding-editor");
    executable(editor.as_path(), format!("#!/bin/sh\n{editor_body}printf 'foreign\\n' > \"$1\"\n").as_bytes());
    git(fixture, repo, &["config", "core.editor", editor.to_str().expect("UTF-8")]);
    // Git 2.54 can leave its process number beside the lock, which is what lets the service name the holder.
    git(fixture, repo, &["config", "core.lockfilePid", "true"]);
    std::fs::write(repo.join("b.txt"), "foreign edit\n").expect("b.txt");
    // No `timeout` in front: the control holds this commit open itself and lets it go when the control says so.
    let child: Child = spawn_owned(direct_held(fixture).current_dir(repo).args(["commit", "--quiet", "--", "b.txt"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped())).expect("foreign commit");
    let began: Instant = Instant::now();
    while !repo.join(".git/index.lock").exists() {
        assert!(began.elapsed() < PATIENCE, "the foreign commit never took the lock");
        std::thread::sleep(Duration::from_millis(5));
    }
    return child;
}

/// A foreign `git commit` of `b.txt` whose editor stays open for `hold` seconds,
/// started once its lock exists.
pub(super) fn foreign_commit(fixture: &Fixture, repo: &Path, hold: u32) -> Child {
    return foreign_commit_with_editor(fixture, repo, format!("sleep {hold}\n").as_str());
}

/// What:
///  A foreign `git commit` of `b.txt` that keeps its lock until the returned file exists and then for `hold` more seconds,
/// started once its lock exists.
/// Why:
///   The commit that is to wait for the lock may reach it at any time on a loaded host;
/// a holder that lets go after a fixed time
///  can be gone before it does.
/// This holder lets go only after the test has seen the commit waiting and created the file,
/// and
///  it lets go on its own if the test ends first,
/// so the wait never depends on how fast the host runs the commit.
pub(super) fn foreign_commit_until_released(fixture: &Fixture, repo: &Path, hold: u32) -> (Child, PathBuf) {
    let release: PathBuf = fixture.root.join("holder-release");
    let body: String = format!("while [ ! -e '{}' ] && {}; do sleep 0.02; done\nsleep {hold}\n", release.display(), while_test_lives(fixture.root.as_path()));
    return (foreign_commit_with_editor(fixture, repo, body.as_str()), release);
}

/// What:
///  Check that the foreign holder's commit changes only the file the control leaves alone.
/// Why:
///   The control writes its own file while the holder's editor is open,
/// and a holder that stages the whole worktree would land that
///  change too,
/// so the commit built again after the branch moved would conflict with the holder on a file the control never meant to share.
/// The control then failed with `left: Some(1)`,
/// `right: Some(0)` and the wrapper's `replay-conflict` finding (2026-11-08).
fn assert_the_holder_changed_only_its_own_file(fixture: &Fixture, repo: &Path) {
    let foreign: String = git_text(fixture, repo, &["log", "--max-count=1", "--format=%H", "--grep", "^foreign$"]).trim().to_string();
    assert!(!foreign.is_empty(), "the holder's commit is on the branch");
    assert_eq!(
        git_text(fixture, repo, &["show", "--name-only", "--format=", foreign.as_str()]).trim(),
        "b.txt",
        "the holder's commit changes only b.txt, never the file the control commits"
    );
}

/// The holder keeps the lock for longer than the old five tries of a tenth of a second:
/// the commit waits,
/// lands on top of the
///  holder's commit with an index that shows both,
/// and the holder's own commit succeeds.
#[test]
fn a_commit_waits_for_a_foreign_commit_that_holds_the_index_and_lands_on_top_of_it() {
    let fixture: Fixture = fixture("landing-lock-holder");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("landing-lock-{}", std::process::id());
    let _service: Held = service_for(&fixture, name.as_str(), "1");
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let holder: Child = foreign_commit(&fixture, &repo, 2);
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    let served = observe(&mut command, b"");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "the holder's own commit succeeds: {}", String::from_utf8_lossy(&held.stderr));
    assert_the_holder_changed_only_its_own_file(&fixture, &repo);
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    assert_eq!(git_text(&fixture, &repo, &["log", "--reverse", "--format=%s", format!("{base}..HEAD").as_str()]), "foreign\nmine\n");
    assert_eq!(git_text(&fixture, &repo, &["status", "--short"]), "", "the index shows both commits");
    assert!(!repo.join(".git/index.lock").exists(), "no lock is left");
    remove_settled(&fixture);
}

/// The seconds the foreign commit keeps the lock after the control has released it,
/// so that its commit ends on its own.
const RELEASED_HOLD_SECONDS: u32 = 0;

/// The notices of the holder a control reads before it lets the holder go:
/// at one notice a second,
/// more seconds of waiting than
///  the second an owner nothing proves alive used to be given,
/// so a wait that lasts past that second is what the control reads.
const NOTICES_BEYOND_BACKOFF: usize = 4;

/// The holder is proven alive (Git's lock PID file names it),
/// so the commit waits for it until the holder lets go,
/// with
///  the holder named,
/// and lands on top of the holder's commit when the holder lets go:
/// nothing fails,
/// nothing is given up.
/// The control releases the holder on the commit's own fourth notice,
/// so the outcome does not depend on how fast the host runs
///  the commit and the wait is known to have gone on past that second.
#[test]
fn a_foreign_holder_proven_alive_is_waited_for_until_it_lets_go_and_the_commit_lands_on_top() {
    let fixture: Fixture = fixture("landing-lock-no-bound");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("ll-nobound-{}", std::process::id());
    let _service: Held = service_for(&fixture, name.as_str(), "1");
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, RELEASED_HOLD_SECONDS);
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let mut command: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str())).env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
    let watched = super::wait_notice::watch_notices(&mut command, "waiting for PID", NOTICES_BEYOND_BACKOFF, || {
        std::fs::write(&release, b"").expect("the foreign commit is told to let go");
    });
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    assert_the_holder_changed_only_its_own_file(&fixture, &repo);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    assert!(!watched.stderr.contains("was not made") && !watched.stderr.contains("giving up"), "{}", watched.stderr);
    assert_eq!(git_text(&fixture, &repo, &["log", "--reverse", "--format=%s", format!("{base}..HEAD").as_str()]), "foreign\nmine\n");
    assert_eq!(git_text(&fixture, &repo, &["status", "--short"]), "", "the index shows both commits");
    remove_settled(&fixture);
}

/// The notice interval of the service in the heartbeat control:
/// far beyond the control,
/// so no notice of the wait reaches the
///  client after the first,
/// and only heartbeats keep it from hearing nothing.
const QUIET_NOTICE_SECONDS: &str = "3600";

/// How long the foreign holder keeps the lock after the queued request has been told to wait:
/// more than twice the silence bound
///  the client under test is given,
/// so only a service that goes on sending heartbeats keeps the client from calling it stopped.
const HOLD_AFTER_NOTICE: Duration = Duration::from_secs(8);

/// How long the client under test takes no message from the service for a stopped one.
const CLIENT_SILENCE: Duration = Duration::from_secs(3);

/// A writer that tells the control when the client first prints a notice,
/// and keeps the text.
struct SignalOnNotice {
    told: Option<std::sync::mpsc::Sender<()>>,
    text: Vec<u8>,
}

impl std::io::Write for SignalOnNotice {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.text.extend_from_slice(bytes);
        if let Some(channel) = self.told.take() {
            let _sent = channel.send(());
        }
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// The real service sends its heartbeats while a request waits behind a foreign hold that lasts several times the client's
///  silence bound:
/// the client is never silent,
/// waits it out,
/// and the commit lands.
/// This is the control of the service's side of the heartbeat;
/// the fake services of `binary_service_heartbeat_tests.rs` are
///  the control of the client's side.
#[test]
fn the_service_sends_heartbeats_to_a_request_that_waits_behind_a_long_foreign_hold() {
    use git_policy_cli::service_client::{Activation, ClientConfig, Submitted, submit};
    use git_policy_cli::service_protocol::{ALLOWED_MODES, Change, CommitRequest};
    use git_policy_cli::service_server::Bind;
    let fixture: Fixture = fixture("landing-lock-heartbeat");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("ll-beat-{}", std::process::id());
    let _service: Held = service_for(&fixture, name.as_str(), QUIET_NOTICE_SECONDS);
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(runtime_for(name.as_str()).as_path(), git_dir.as_path());
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, RELEASED_HOLD_SECONDS);
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let blob: String = git_text(&fixture, &repo, &["hash-object", "-w", "--", "a.txt"]).trim().to_string();
    let request: CommitRequest = CommitRequest {
        id: String::from("heartbeat-landing"),
        repository: repo.clone(),
        common_dir: git_dir.clone(),
        git_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        base: git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string(),
        changes: vec![Change { path: PathBuf::from("a.txt"), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: String::from("mine\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: git_dir.join("index"),
        scratch: git_dir.join("monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-heartbeat-landing-0"),
        captured: 0,
        holder: String::new(),
    };
    let config: ClientConfig = ClientConfig { address: Bind::Path(socket), activation: Activation::Managed, start_wait: PATIENCE, heartbeat_silence: CLIENT_SILENCE, start_watch: None, keep_fresh: None, answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING, extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: git_policy_cli::wait_ceilings::START_CEILING, waiting_notice_after: PATIENCE, units_record_dir: None };
    let (told, hearing) = std::sync::mpsc::channel::<()>();
    let letting_go = std::thread::spawn(move || {
        hearing.recv_timeout(PATIENCE).expect("the client was told the request waits");
        std::thread::sleep(HOLD_AFTER_NOTICE);
        std::fs::write(&release, b"").expect("the foreign commit is told to let go");
    });
    let mut notices: SignalOnNotice = SignalOnNotice { told: Some(told), text: Vec::new() };
    let answered = submit(&config, &request, &mut notices);
    letting_go.join().expect("releaser");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    assert!(matches!(answered, Ok(Submitted::Landed { .. })), "{answered:?} {}", String::from_utf8_lossy(&notices.text));
    assert!(!String::from_utf8_lossy(&notices.text).contains("stopped answering"), "{}", String::from_utf8_lossy(&notices.text));
    remove_settled(&fixture);
}
