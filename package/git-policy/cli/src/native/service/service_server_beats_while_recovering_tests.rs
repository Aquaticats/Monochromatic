//! Controls for a service that is still recovering the batch an earlier service left open:
//! a client that connects while that recovery runs is greeted and beaten,
//! so a slow recovery does not read as a service that stopped,
//! and the request it sends is landed after the recovery,
//! never before it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recovered = await serviceThatIsStillRecovering(); expect(recovered.beats).toBeGreaterThan(0);
//! ```

use super::*;
use crate::service_client_listen_and_ask_to_retire::read_message;
use crate::service_fault::hooks;
use crate::service_journal::{DONE_EXTENSION, OPEN_EXTENSION, batch_path};
use crate::service_land::{LandContext, land_batch};
use crate::service_protocol::{ALLOWED_MODES, Change, ClientMessage, CommitRequest, HEARTBEAT_EVERY, ServiceMessage, encode_client};
use crate::test_support::{git_text, repository};
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The most a control waits for the service:
/// far beyond what the loop needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// The commit of the batch the earlier service left open.
const FIRST_MESSAGE: &str = "first";

/// The commit of the request the client sends while the service recovers.
const SECOND_MESSAGE: &str = "second";

/// The fault point the recovery reaches once it holds the index lock,
/// which is where a control holds it still.
const IN_THE_RECOVERY: &str = "restart-locked";

/// One disposable world:
/// a repository,
/// a journal,
/// and a socket short enough for the kernel's hundred bytes.
struct World {
    root: PathBuf,
    socket_directory: PathBuf,
    repo: PathBuf,
    journal: PathBuf,
}

fn world(label: &str) -> World {
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory"));
    let socket_directory: PathBuf = home.join("temp/agent").join(format!("sbr-{}-{label}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&socket_directory);
    std::fs::create_dir_all(&socket_directory).expect("short directory");
    let root: PathBuf = crate::test_support::fixture(format!("service-beats-while-recovering-{label}").as_str());
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    return World { root, socket_directory, repo, journal };
}

impl World {
    /// The context the service lands in.
    fn context(&self) -> LandContext {
        return LandContext { git: crate::test_support::git_context(), journal: self.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING };
    }

    /// The socket the service listens on.
    fn socket(&self) -> PathBuf {
        return self.socket_directory.join("s");
    }

    /// The settings of a service that serves any worktree and ends only when it is told to retire.
    fn settings(&self) -> ServeSettings {
        return ServeSettings::new(IDLE_EXIT);
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.root);
        let _removed_short = std::fs::remove_dir_all(&self.socket_directory);
    }
}

/// A request that sets `path` to `content`,
/// prepared against the branch as it is now.
fn a_request(world: &World, id: &str, message: &str) -> CommitRequest {
    let path: PathBuf = world.repo.join(format!("{message}.txt"));
    std::fs::write(&path, format!("{message}\n")).expect("the worktree file");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", path.to_str().expect("UTF-8")]);
    return CommitRequest {
        id: String::from(id),
        repository: world.repo.clone(),
        common_dir: world.repo.join(".git"),
        git_dir: world.repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: git_text(world.repo.as_path(), &["rev-parse", "HEAD"]),
        changes: vec![Change { path: PathBuf::from(format!("{message}.txt")), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("{message}\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: world.repo.join(".git/index"),
        scratch: world.repo.join(".git/monochromatic-git-policy-cli-scratch").join(format!("monochromatic-git-policy-cli-service-{id}-0")),
        captured: 0,
        holder: String::new(),
    };
}

/// What:
///  Leaves the batch of the first commit open,
/// as a service killed after the branch moved leaves it,
/// so the service that starts next has a recovery to make.
/// Why:
///   The recovery is where the audit's restarted service sat while its clients gave up,
/// so the control needs one that is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landWithFailedIndex(world, 1, [first]);
/// ```
fn leave_a_batch_open(world: &World) {
    hooks::fail_at("build-index");
    let outcomes = land_batch(&world.context(), 1, &[a_request(world, "first", FIRST_MESSAGE)]).expect("the commit lands whatever the index does");
    hooks::clear_failures();
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(batch_path(world.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the batch stays open for the recovery");
}

/// Reads the greeting of the service on `reader`,
/// and says whether it came.
fn greeted(reader: &mut BufReader<UnixStream>) -> bool {
    return matches!(read_message(reader, &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE), Some(ServiceMessage::Welcome { .. }));
}

/// What:
///  A client that connects while the service recovers its earlier batch is greeted and beaten,
/// and the request it sends is landed once the recovery is over.
/// Why:
///  Finding C of the audit `commit-kills-and-concurrency` (2026-11-09):
/// the service is killed at its `after-ref` fault point and restarted,
///  its recovery of the open batch waits on the disk,
/// and every client exited 1 after about 5.5 s with
///  "The commit service stopped answering ... not known whether the commit was made"
/// although the commits were in history afterwards.
/// The heartbeat
///  every `HEARTBEAT_EVERY` is the proof of life the client's wait rests on,
/// so the connection that carries it must be accepted before the recovery
///  runs,
/// not after it.
#[test]
fn a_client_that_connects_while_the_service_recovers_is_greeted_and_beaten() {
    let world: World = world("recovers");
    leave_a_batch_open(&world);
    let (entered, in_the_recovery) = mpsc::channel::<()>();
    let (let_go, held) = mpsc::channel::<()>();
    let listener: UnixListener = UnixListener::bind(world.socket()).expect("bind");
    let context: LandContext = world.context();
    let settings: ServeSettings = world.settings();
    let serving = std::thread::spawn(move || {
        // The recovery runs on this thread, so the hook that holds it is set here, on this thread's own record of it.
        let waiting: mpsc::Receiver<()> = held;
        hooks::set(IN_THE_RECOVERY, std::rc::Rc::new(move || {
            let _said = entered.send(());
            let _let_go = waiting.recv();
        }));
        serve(listener, &context, &settings);
    });
    in_the_recovery.recv_timeout(PATIENCE).expect("the service reached the recovery and is held inside it");
    // The client connects and sends its request while the recovery is held.
    let stream: UnixStream = UnixStream::connect(world.socket()).expect("the service listens while it recovers");
    let _timeout = stream.set_read_timeout(Some(HEARTBEAT_EVERY * 5));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream.try_clone().expect("a second handle on the connection"));
    let mut request: String = encode_client(&ClientMessage::Commit(Box::new(a_request(&world, "second", SECOND_MESSAGE))));
    request.push('\n');
    (&stream).write_all(request.as_bytes()).expect("the client sends its request");
    assert!(greeted(&mut reader), "the service greets a client that connects while it recovers");
    let mut beats: usize = 0;
    let waited: Instant = Instant::now();
    while waited.elapsed() < HEARTBEAT_EVERY * 4 && beats == 0 {
        match read_message(&mut reader, &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE) {
            Some(ServiceMessage::Heartbeat) => beats += 1,
            // The answer cannot come before the recovery is let go; a line that is not a beat is read and the wait goes on.
            Some(_other) => {}
            None => break,
        }
    }
    assert_eq!(beats, 1, "a service that is recovering still proves it lives, once a second");
    // The recovery is let go, and the request the client sent is landed after it.
    let _let_go = let_go.send(());
    let mut landed: Option<String> = None;
    let answered: Instant = Instant::now();
    while answered.elapsed() < PATIENCE && landed.is_none() {
        match read_message(&mut reader, &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE) {
            Some(ServiceMessage::Landed { commit, .. }) => landed = Some(commit),
            Some(_other) => {}
            None => break,
        }
    }
    assert!(landed.is_some(), "the request sent during the recovery is landed once the recovery is over");
    assert!(batch_path(world.journal.as_path(), 1, DONE_EXTENSION).exists(), "the recovery finished the batch it found open");
    let subjects: Vec<String> = git_text(world.repo.as_path(), &["log", "--format=%s", "main"]).lines().map(str::to_string).collect();
    assert_eq!(subjects, vec![SECOND_MESSAGE, FIRST_MESSAGE, "initial"], "the recovered commit is first, and the request behind it second");
    // The service is told to retire, so its thread ends here rather than at its idle exit.
    retire(world.socket().as_path());
    let joined: bool = join_within(serving);
    assert!(joined, "the service ends when it is told to retire");
}

/// Tells the service at `socket` to retire,
/// which ends it once the batch it holds is landed.
fn retire(socket: &Path) {
    let stream: UnixStream = UnixStream::connect(socket).expect("the service listens");
    let _timeout = stream.set_read_timeout(Some(HEARTBEAT_EVERY * 5));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream.try_clone().expect("a second handle on the connection"));
    let _greeted: bool = greeted(&mut reader);
    let mut line: String = encode_client(&ClientMessage::Retire);
    line.push('\n');
    (&stream).write_all(line.as_bytes()).expect("the retire reaches the service");
}

/// Waits at most `PATIENCE` for a serving thread to end,
/// and says whether it did.
fn join_within(thread: std::thread::JoinHandle<()>) -> bool {
    let began: Instant = Instant::now();
    while !thread.is_finished() {
        if began.elapsed() >= PATIENCE {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _joined = thread.join();
    return true;
}

/// The budget of a control that reads a line: the silence bound every command gives the service.
fn a_patience() -> crate::service_client_patience::Patience {
    return crate::service_client_patience::Patience::of(crate::service_protocol::HEARTBEAT_SILENCE);
}
