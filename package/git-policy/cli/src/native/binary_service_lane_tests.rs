//! What:
//!  A landing that waits for a foreign holder of one repository's index lock, beside a landing in another repository.
//! Why:
//!  One service lands the commits of every repository an account uses. A wait for a foreign `index.lock` is a wait in one
//!  repository, so it must not hold back the commits of the others: the service gives each repository its own landing, and
//!  within a repository the order stays the order of arrival.

use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_client::{Activation, ClientConfig, Submitted, submit};
use git_policy_cli::service_messages::ClientFailure;
use git_policy_cli::service_protocol::{ALLOWED_MODES, Change, CommitRequest};
use git_policy_cli::service_server::Bind;
use std::ffi::OsStr;
use std::io::Write;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

const SERVICE: &str = env!("CARGO_BIN_EXE_cli-git-service");
const COMMITTER: &str = "T <t@example.invalid> 1793535889 +0000";

/// The longest a control waits for anything: far beyond what a healthy run needs, so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long a landing in a free repository may take while another repository's landing waits: far below the wait limit the
///  service runs under, so a landing that waited behind the other would miss it.
const FREE_REPOSITORY_BOUND: Duration = Duration::from_secs(15);

/// How long the control gives a request to reach the service and start waiting, when the service has nothing to say about it.
const ARRIVAL_PAUSE: Duration = Duration::from_secs(3);

/// The wait for a foreign lock the service is started with, long enough that no control reaches it before it is done.
const LOCK_WAIT_SECONDS: &str = "120";

/// A service process that is stopped when the control ends, whether it passed or panicked.
struct Held(Child);

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

fn start(fixture: &Fixture, name: &str) -> Held {
    let child: Child = Command::new(SERVICE)
        .args(["--journal"])
        .arg(fixture.root.join("journal"))
        .args(["--abstract", name])
        .env("CLI_GIT_LOCK_WAIT_SECONDS", LOCK_WAIT_SECONDS)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    return Held(child);
}

fn config(name: &str) -> ClientConfig {
    return ClientConfig { address: Bind::Abstract(name.to_string()), activation: Activation::Managed, start_wait: PATIENCE, commit_wait: PATIENCE };
}

/// A request that sets `path` to `content` in `repo`, prepared against the branch as it is now.
fn request(fixture: &Fixture, repo: &Path, id: &str, path: &str, content: &str) -> CommitRequest {
    std::fs::write(repo.join(path), content).expect("write");
    let query = |arguments: &[&str]| -> String { return super::commit::git_text(fixture, repo, arguments).trim_end().to_string(); };
    let blob: String = query(&["hash-object", "-w", "--", path]);
    return CommitRequest {
        id: id.to_string(),
        repository: repo.to_path_buf(),
        common_dir: repo.join(".git"),
        reference: String::from("refs/heads/main"),
        base: query(&["rev-parse", "HEAD"]),
        changes: vec![Change { path: path.to_string(), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("message of {id}\n"),
        author: None,
        committer: COMMITTER.to_string(),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: repo.join(".git/index"),
    };
}

/// A notice sink that tells the control when the service first said something, which for a landing is that it waits.
struct Heard(Sender<()>);

impl Write for Heard {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let _told = self.0.send(());
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

fn submitted_in_thread(name: &str, sent: CommitRequest, heard: Sender<()>) -> Receiver<Result<Submitted, ClientFailure>> {
    let (done, result) = channel();
    let config: ClientConfig = config(name);
    std::thread::spawn(move || {
        let _sent = done.send(submit(&config, &sent, &mut Heard(heard)));
    });
    return result;
}

/// A landing that waits for a foreign holder of one repository's index lock does not hold back the landing in another
///  repository, and lands itself, in its turn, once the holder is gone.
#[test]
fn a_landing_waiting_for_a_foreign_index_lock_does_not_hold_back_another_repository() {
    let fixture: Fixture = fixture("service-lanes");
    let waiting_repo: PathBuf = repository(&fixture, OsStr::new("waiting"));
    let free_repo: PathBuf = repository(&fixture, OsStr::new("free"));
    for repo in [&waiting_repo, &free_repo] {
        git(&fixture, repo, &["config", "maintenance.autoDetach", "false"]);
    }
    let name: String = format!("service-lanes-{}", std::process::id());
    let _service: Held = start(&fixture, name.as_str());
    let began: Instant = Instant::now();
    while !SocketAddr::from_abstract_name(name.as_bytes()).and_then(|socket| { return UnixStream::connect_addr(&socket); }).is_ok() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    // A program other than the service holds the waiting repository's index lock.
    let lock: PathBuf = waiting_repo.join(".git/index.lock");
    std::fs::write(&lock, b"").expect("the foreign lock");
    let (heard, waiting_notice) = channel();
    let waiting: Receiver<Result<Submitted, ClientFailure>> =
        submitted_in_thread(name.as_str(), request(&fixture, &waiting_repo, "waiting", "waiting.txt", "w\n"), heard);
    // The landing says that it waits when it can name the holder; when it cannot, a pause is enough for the request to arrive.
    let _heard = waiting_notice.recv_timeout(ARRIVAL_PAUSE);
    let (silent, _unheard) = channel();
    let started: Instant = Instant::now();
    let free: Receiver<Result<Submitted, ClientFailure>> = submitted_in_thread(name.as_str(), request(&fixture, &free_repo, "free", "free.txt", "f\n"), silent);
    let landed = free.recv_timeout(FREE_REPOSITORY_BOUND);
    // Whatever happened, the holder goes away so that nothing outlives the control.
    std::fs::remove_file(&lock).expect("the foreign holder lets go");
    let answer = landed.expect("the other repository's landing was held back by the wait");
    assert!(matches!(answer, Ok(Submitted::Landed { .. })), "{answer:?}");
    assert!(started.elapsed() < FREE_REPOSITORY_BOUND);
    let finally = waiting.recv_timeout(PATIENCE).expect("the waiting landing ends once the holder is gone");
    assert!(matches!(finally, Ok(Submitted::Landed { .. })), "{finally:?}");
    super::commit::remove_settled(&fixture);
}
