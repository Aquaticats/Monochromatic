//! What:
//!  One commit service per worktree:
//! a landing that waits for a foreign holder of one worktree's index lock does not delay
//!  another worktree's commits,
//! and a service refuses a request that names another worktree.
//! Why:
//!  Every worktree has its own service instance,
//! the way Bazel runs one server per workspace,
//! so a wait in one worktree (a
//!  foreign `index.lock`,
//! a slow push) is a wait for that worktree only.
//! An instance serves the worktree it was started for,
//!  and a request that reaches the wrong socket is refused where it can be seen,
//! never landed in the wrong worktree
//!  (`doc/planning/cli-git-resident-service.md`,
//! "One service per worktree").

use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_client::{Activation, ClientConfig, Submitted, submit};
use git_policy_cli::service_instance::{instance_of, socket_for};
use git_policy_cli::service_messages::ClientFailure;
use git_policy_cli::service_protocol::{ALLOWED_MODES, Change, CommitRequest};
use git_policy_cli::service_server::Bind;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

const COMMITTER: &str = "T <t@example.invalid> 1793535889 +0000";

/// The longest a control waits for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long a landing in a free worktree may take while another worktree's landing waits:
/// far below the time the other worktree's
///  landing is made to wait,
/// so a landing that waited behind the other would miss it.
const FREE_WORKTREE_BOUND: Duration = Duration::from_secs(60);

/// How long the control gives a request to reach its service and start waiting,
/// when the service has nothing to say about it.
const ARRIVAL_PAUSE: Duration = Duration::from_secs(3);

/// A repository with its main worktree and one linked worktree,
/// each with its own Git directory.
struct Worktrees {
    main: PathBuf,
    linked: PathBuf,
    main_git_dir: PathBuf,
    linked_git_dir: PathBuf,
}

fn worktrees(fixture: &Fixture) -> Worktrees {
    let main: PathBuf = repository(fixture, OsStr::new("main"));
    git(fixture, &main, &["config", "maintenance.autoDetach", "false"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(fixture, &main, &["worktree", "add", "--quiet", "-b", "linked", linked.to_str().expect("UTF-8")]);
    let linked_git_dir: PathBuf = main.join(".git/worktrees/linked");
    assert!(linked_git_dir.is_dir(), "the linked worktree's own Git directory");
    let main_git_dir: PathBuf = main.join(".git");
    return Worktrees { main, linked, main_git_dir, linked_git_dir };
}

/// The instances of both worktrees,
/// each on its own socket,
/// killed with the control.
struct Services {
    children: Vec<Child>,
    runtime: PathBuf,
}

impl Drop for Services {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _killed = child.kill();
            let _waited = child.wait();
        }
        let _removed = std::fs::remove_dir_all(&self.runtime);
    }
}

fn start(fixture: &Fixture, name: &str, worktrees: &Worktrees) -> Services {
    let runtime: PathBuf = super::service_commit::runtime_for(name);
    std::fs::create_dir_all(runtime.join("monochromatic-git-policy-cli")).expect("runtime directory");
    let mut services: Services = Services { children: Vec::new(), runtime };
    let mut sockets: Vec<PathBuf> = Vec::new();
    for git_dir in [&worktrees.main_git_dir, &worktrees.linked_git_dir] {
        let socket: PathBuf = socket_for(services.runtime.as_path(), git_dir.as_path());
        let child: Child = super::support::service_command()
            .args(["--journal"])
            .arg(fixture.root.join("journal"))
            .arg("--path")
            .arg(&socket)
            .args(["--instance", instance_of(git_dir.as_path()).as_str()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the service starts");
        services.children.push(child);
        sockets.push(socket);
    }
    let began: Instant = Instant::now();
    for socket in sockets {
        while std::os::unix::net::UnixStream::connect(&socket).is_err() {
            assert!(began.elapsed() < PATIENCE, "the service never listened");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    return services;
}

fn config(runtime: &Path, git_dir: &Path) -> ClientConfig {
    return ClientConfig {
        address: Bind::Path(socket_for(runtime, git_dir)),
        activation: Activation::Managed,
        start_wait: PATIENCE,
        heartbeat_silence: PATIENCE, start_watch: None, keep_fresh: None, answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING, extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: git_policy_cli::wait_ceilings::START_CEILING, waiting_notice_after: PATIENCE, units_record_dir: None,
    };
}

/// A request that sets `path` to `content` in the worktree at `root`,
/// whose own Git directory is `git_dir`,
/// on `branch`.
/// The tree it names is the one this release makes:
/// a directory of the repository scratch place under the common Git directory,
///  which is the place the sweep of a later command also covers.
fn request(fixture: &Fixture, root: &Path, git_dir: &Path, branch: &str, id: &str, path: &str) -> CommitRequest {
    std::fs::write(root.join(path), format!("{id}\n")).expect("write");
    let query = |arguments: &[&str]| -> String { return super::commit::git_text(fixture, root, arguments).trim_end().to_string(); };
    let blob: String = query(&["hash-object", "-w", "--", path]);
    let common_dir: PathBuf = query(&["rev-parse", "--path-format=absolute", "--git-common-dir"]).into();
    let scratch: PathBuf = common_dir
        .join("monochromatic-git-policy-cli-scratch")
        .join(format!("monochromatic-git-policy-cli-service-{id}-0"));
    std::fs::create_dir_all(scratch.as_path()).expect("the tree the request names");
    return CommitRequest {
        id: id.to_string(),
        repository: root.to_path_buf(),
        common_dir,
        git_dir: git_dir.to_path_buf(),
        reference: format!("refs/heads/{branch}"),
        base: query(&["rev-parse", "HEAD"]),
        changes: vec![Change { path: PathBuf::from(path), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("message of {id}\n"),
        author: None,
        committer: COMMITTER.to_string(),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: git_dir.join("index"),
        scratch,
        captured: 0,
        holder: String::new(),
    };
}

/// A notice sink that tells the control when the service first said something,
/// which for a landing is that it waits.
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

fn submitted_in_thread(config: ClientConfig, sent: CommitRequest, heard: Sender<()>) -> Receiver<Result<Submitted, ClientFailure>> {
    let (done, result) = channel();
    std::thread::spawn(move || {
        let _sent = done.send(submit(&config, &sent, &mut Heard(heard)));
    });
    return result;
}

/// A landing that waits for a foreign holder of one worktree's index lock does not delay the landing in another worktree of
///  the same repository,
/// and lands itself,
/// in its turn,
/// once the holder is gone.
#[test]
fn a_landing_waiting_for_a_foreign_index_lock_does_not_delay_another_worktree() {
    let fixture: Fixture = fixture("service-worktrees");
    let trees: Worktrees = worktrees(&fixture);
    let name: String = format!("wt-{}", std::process::id());
    let services: Services = start(&fixture, name.as_str(), &trees);
    // A program other than the service holds the main worktree's index lock.
    let lock: PathBuf = trees.main_git_dir.join("index.lock");
    // With content, as a person's `git commit --all` in its editor leaves it: an empty lock that nothing holds is removed.
    std::fs::write(&lock, b"a foreign program's lock\n").expect("the foreign lock");
    let (heard, waiting_notice) = channel();
    let waiting = submitted_in_thread(
        config(services.runtime.as_path(), trees.main_git_dir.as_path()),
        request(&fixture, &trees.main, &trees.main_git_dir, "main", "waiting", "waiting.txt"),
        heard,
    );
    // The landing says that it waits when it can name the holder; when it cannot, a pause is enough for the request to arrive.
    let _heard = waiting_notice.recv_timeout(ARRIVAL_PAUSE);
    let (silent, _unheard) = channel();
    let free = submitted_in_thread(
        config(services.runtime.as_path(), trees.linked_git_dir.as_path()),
        request(&fixture, &trees.linked, &trees.linked_git_dir, "linked", "free", "free.txt"),
        silent,
    );
    let landed = free.recv_timeout(FREE_WORKTREE_BOUND);
    // Whatever happened, the holder goes away so that nothing outlives the control.
    std::fs::remove_file(&lock).expect("the foreign holder lets go");
    let answer = landed.expect("the other worktree's landing was held back by the wait");
    assert!(matches!(answer, Ok(Submitted::Landed { .. })), "{answer:?}");
    let finally = waiting.recv_timeout(PATIENCE).expect("the waiting landing ends once the holder is gone");
    assert!(matches!(finally, Ok(Submitted::Landed { .. })), "{finally:?}");
    drop(services);
    super::commit::remove_settled(&fixture);
}

/// A request for one worktree that reaches another worktree's socket is refused,
/// and nothing moves in either worktree.
#[test]
fn a_service_refuses_a_request_for_another_worktree() {
    let fixture: Fixture = fixture("service-worktrees-refused");
    let trees: Worktrees = worktrees(&fixture);
    let name: String = format!("wtr-{}", std::process::id());
    let services: Services = start(&fixture, name.as_str(), &trees);
    let main_tip: String = super::commit::git_text(&fixture, &trees.main, &["rev-parse", "main"]).trim_end().to_string();
    let linked_tip: String = super::commit::git_text(&fixture, &trees.linked, &["rev-parse", "linked"]).trim_end().to_string();
    // The linked worktree's request, sent to the main worktree's service.
    let wrong = request(&fixture, &trees.linked, &trees.linked_git_dir, "linked", "wrong-door", "wrong.txt");
    let refused = submit(&config(services.runtime.as_path(), trees.main_git_dir.as_path()), &wrong, &mut std::io::sink());
    assert_eq!(refused, Err(ClientFailure::Refused));
    assert_eq!(super::commit::git_text(&fixture, &trees.main, &["rev-parse", "main"]).trim_end(), main_tip, "the main worktree's branch did not move");
    assert_eq!(super::commit::git_text(&fixture, &trees.linked, &["rev-parse", "linked"]).trim_end(), linked_tip, "the linked worktree's branch did not move");
    // The same request at its own worktree's service lands.
    let right = submit(&config(services.runtime.as_path(), trees.linked_git_dir.as_path()), &wrong, &mut std::io::sink());
    assert!(matches!(right, Ok(Submitted::Landed { .. })), "{right:?}");
    drop(services);
    super::commit::remove_settled(&fixture);
}
