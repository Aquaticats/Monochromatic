//! Controls for a client's greeting and beats while the journal of the after-commit work gives no answer:
//! the acceptor starts before the plan of the pushes waiting for a new try and the repositories an earlier
//!  service was woken for are read,
//! and those two are read on a thread of their own,
//! so a journal that stopped answering delays a push and never a commit's client.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stalled = stallThePlanWithAFifo(); expect(await greetingOf(service)).toBeDefined();
//! ```

use super::*;
use crate::service_client_listen_and_ask_to_retire::read_message;
use crate::service_land::LandContext;
use crate::service_protocol::{ALLOWED_MODES, Change, ClientMessage, CommitRequest, HEARTBEAT_EVERY, ServiceMessage, encode_client};
use crate::test_support::{git_text, repository};
use std::io::{BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// The most a control waits for the service:
/// far beyond what the greeting,
/// the beats and the landing need,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long the service reads the journal for in this control:
/// long enough that a read of a pipe with no writer never ends inside it,
/// and longer than two beats,
/// so a client that waits for its commit is shown to be beaten while the read has not returned.
const SHORT_CEILING: Duration = Duration::from_millis(2_500);

/// The name of the pipe that stands where a plan file of the journal would be.
const STALLED_PLAN: &str = "a-push-waiting-for-a-new-try.plan";

/// The commit the client asks for.
const MESSAGE: &str = "waited";

/// What:
///  One disposable world:
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
    let socket_directory: PathBuf = home.join("temp/agent").join(format!("sbjr-{}-{label}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&socket_directory);
    std::fs::create_dir_all(&socket_directory).expect("short directory");
    let root: PathBuf = crate::test_support::fixture(format!("service-greets-before-the-journal-{label}").as_str());
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    return World { root, socket_directory, repo, journal };
}

impl World {
    /// The context the service lands in.
    fn context(&self) -> LandContext {
        return LandContext {
            git: crate::test_support::git_context(),
            journal: self.journal.clone(),
            on_objects_written: None,
            lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
        };
    }

    /// The socket the service listens on.
    fn socket(&self) -> PathBuf {
        return self.socket_directory.join("s");
    }

    /// The settings of a service that serves any worktree,
    /// reads the journal for `SHORT_CEILING`,
    /// and ends only when it is told to retire.
    fn settings(&self) -> ServeSettings {
        let mut settings: ServeSettings = ServeSettings::new(IDLE_EXIT);
        settings.journal_restore_ceiling = SHORT_CEILING;
        return settings;
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.root);
        let _removed_short = std::fs::remove_dir_all(&self.socket_directory);
    }
}

/// A request that sets a file of the repository,
/// prepared against the branch as it is now.
fn a_request(world: &World, id: &str) -> CommitRequest {
    let path: PathBuf = world.repo.join(format!("{MESSAGE}.txt"));
    std::fs::write(&path, format!("{MESSAGE}\n")).expect("the worktree file");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", path.to_str().expect("UTF-8")]);
    return CommitRequest {
        id: String::from(id),
        repository: world.repo.clone(),
        common_dir: world.repo.join(".git"),
        git_dir: world.repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: git_text(world.repo.as_path(), &["rev-parse", "HEAD"]),
        changes: vec![Change { path: PathBuf::from(format!("{MESSAGE}.txt")), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("{MESSAGE}\n"),
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
///  Stand a pipe with no writer where the journal's plan of the pushes waiting for a new try keeps its files.
/// Why:
///  A read of a pipe's other end waits in the kernel until a writer opens it,
/// which is the one way to make a read of a plan file hold a thread with no end
///  (`push_retry_store.rs`,
/// `load`),
/// and finding 3 of the review of `b3d00b3be4` is a service whose landing thread sat in that read
/// while its clients read the silence as a service that stopped.
fn stall_the_plan(journal: &Path) -> PathBuf {
    let directory: PathBuf = journal.join(PLAN_DIRECTORY);
    std::fs::create_dir_all(directory.as_path()).expect("the plan's directory");
    let path: PathBuf = directory.join(STALLED_PLAN);
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  directory takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where a plan file would be: {}", std::io::Error::last_os_error());
    return path;
}

/// What:
///  Open the pipe for writing and close it again,
/// which ends the read the service left behind when its ceiling passed.
/// Why:
///  That read is abandoned,
/// never cancelled (`work_awaited_within_a_ceiling.rs`),
/// so the thread that holds it ends only when a writer appears;
/// a control that left it would keep one thread of the test binary,
/// and no more.
/// The open does not wait for a reader:
///  a pipe with no reader is reported instead,
/// and the control tries again until `PATIENCE`.
fn release_the_pipe(path: &Path) {
    let began: Instant = Instant::now();
    while began.elapsed() < PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            // The reader is there, and closing this end ends its read.
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            // No reader yet: ask again until the patience is over.
            Err(_not_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// Reads the greeting of the service on `reader`,
/// and says whether it came.
fn greeted(reader: &mut BufReader<UnixStream>) -> bool {
    return matches!(read_message(reader), Some(ServiceMessage::Welcome { .. }));
}

/// What:
///  A client that connects while the journal's plan cannot be read is greeted and beaten,
/// and the commit it sends is landed once the journal's ceiling passed.
/// Why:
///  The greeting and the beat are the proof of life a client's wait rests on,
/// and a service that reads the journal of its after-commit work before it accepts a connection answers no one
///  while that read waits,
///  which is a commit that reads as a service that stopped.
/// The acceptor starts first and the plan is read on a thread of its own,
/// so the greeting,
/// the beats and the landing all happen while that read has not returned.
#[test]
fn a_client_is_greeted_beaten_and_landed_while_the_journals_plan_gives_no_answer() {
    let world: World = world("restore-stalled");
    let stalled: PathBuf = stall_the_plan(world.journal.as_path());
    let listener: UnixListener = UnixListener::bind(world.socket()).expect("bind");
    let context: LandContext = world.context();
    let settings: ServeSettings = world.settings();
    let serving = std::thread::spawn(move || {
        serve(listener, &context, &settings);
    });
    let stream: UnixStream = UnixStream::connect(world.socket()).expect("the service listens");
    let _timeout = stream.set_read_timeout(Some(HEARTBEAT_EVERY * 3));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream.try_clone().expect("a second handle on the connection"));
    let mut request: String = encode_client(&ClientMessage::Commit(Box::new(a_request(&world, "waited"))));
    request.push('\n');
    (&stream).write_all(request.as_bytes()).expect("the client sends its request");
    assert!(greeted(&mut reader), "the service greets a client although the plan of its pushes gave no answer");
    let mut beats: usize = 0;
    let mut landed: Option<String> = None;
    let waited: Instant = Instant::now();
    while waited.elapsed() < PATIENCE && landed.is_none() {
        match read_message(&mut reader) {
            Some(ServiceMessage::Heartbeat) => beats += 1,
            Some(ServiceMessage::Landed { commit, .. }) => landed = Some(commit),
            Some(_other) => {}
            None => break,
        }
    }
    assert!(landed.is_some(), "the commit of a client that waited is landed once the journal's ceiling passed");
    assert!(beats >= 1, "a service whose journal gave no answer still proves it lives, once a second");
    // The service is told to retire on a connection of its own, so its thread ends here rather than at its idle exit.
    retire(world.socket().as_path());
    let joined: bool = join_within(serving);
    release_the_pipe(stalled.as_path());
    assert!(joined, "the service ends when it is told to retire, so a read that gave no answer did not hold it");
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
