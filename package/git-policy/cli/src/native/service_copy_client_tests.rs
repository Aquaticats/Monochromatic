//! The wrapper's side of a copy against a real service loop and against a service that is killed in the middle:
//!  the answers arrive, a broken connection is answered by the same request again, and every failure the person can meet
//!  is worded without an implementation term.

use super::*;
use crate::service_copy::tests::{COPIED_ENTRIES, Scene};
use crate::service_copy::encode_copy_reply;
use crate::service_land::LandContext;
use crate::service_messages::DENYLIST;
use crate::service_server::{Bind, serve};
use crate::test_support::{git_context, remove};
use crate::worktree_copy_request::CopyRequest;
use std::io::BufRead;
use std::os::unix::net::UnixListener;

/// A short directory for a socket: socket paths hold about a hundred bytes, which the fixture tree's depth does not allow.
fn socket_in_home(label: &str) -> (PathBuf, PathBuf) {
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory"));
    let directory: PathBuf = home.join("temp/agent").join(format!("cc-{}-{label}", std::process::id()));
    std::fs::create_dir_all(directory.as_path()).expect("socket directory");
    return (directory.join("s"), directory);
}

fn config_for(socket: &Path) -> ClientConfig {
    return ClientConfig {
        address: Bind::Path(socket.to_path_buf()),
        activation: Activation::Managed,
        start_wait: Duration::from_secs(20),
        commit_wait: Duration::from_secs(60),
    };
}

/// The real service loop on a socket of its own, retired when the test ends.
struct RunningService {
    socket: PathBuf,
    directory: PathBuf,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl RunningService {
    fn start(label: &str, scene: &Scene) -> RunningService {
        let (socket, directory) = socket_in_home(label);
        let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
        let journal: PathBuf = scene.root.join("service-journal");
        std::fs::create_dir_all(journal.as_path()).expect("journal");
        let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None };
        let thread = std::thread::spawn(move || serve(listener, &context, Duration::from_secs(120)));
        return RunningService { socket, directory, thread: Some(thread) };
    }
}

impl Drop for RunningService {
    fn drop(&mut self) {
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(self.socket.as_path()) {
            let mut retire: String = encode_client(&ClientMessage::Retire);
            retire.push('\n');
            let _sent = stream.write_all(retire.as_bytes());
        }
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
        let _removed = std::fs::remove_dir_all(self.directory.as_path());
    }
}

/// A request goes through the real service loop: the client is told the copy started, and then how it ended, and the files
///  are in the worktree.
#[test]
fn a_request_is_answered_by_the_service_loop() {
    let scene: Scene = Scene::new("client-roundtrip");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let service: RunningService = RunningService::start("roundtrip", &scene);
    let mut notices: Vec<u8> = Vec::new();
    let answers = request_copies(&config_for(service.socket.as_path()), scene.common.as_path(), std::slice::from_ref(&request.id), &mut notices)
        .expect("answered");
    assert_eq!(answers, vec![(request.id, CopyOutcome::Copied(COPIED_ENTRIES))]);
    // A slow disk may make the copy outlast the grace, so the only lines that may appear are progress lines.
    assert!(
        String::from_utf8_lossy(notices.as_slice()).lines().all(|line| return line.contains("still copying") || line.contains("waiting for")),
        "{}",
        String::from_utf8_lossy(notices.as_slice())
    );
    assert!(topic.join("cache/deep/data").exists());
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    drop(service);
    remove(scene.root.as_path());
}

/// Two clients ask together for one repository: each gets the answer for its own request, and each copy is made once.
#[test]
fn two_clients_each_get_their_own_answer_and_each_copy_runs_once() {
    let scene: Scene = Scene::new("client-two");
    let first: PathBuf = scene.add_worktree("first");
    let second: PathBuf = scene.add_worktree("second");
    let first_request: CopyRequest = scene.request(first.as_path());
    let second_request: CopyRequest = scene.request(second.as_path());
    let service: RunningService = RunningService::start("two", &scene);
    let config: ClientConfig = config_for(service.socket.as_path());
    let (one, two) = std::thread::scope(|threads| {
        let one = threads.spawn(|| return request_copies(&config, scene.common.as_path(), std::slice::from_ref(&first_request.id), &mut std::io::sink()));
        let two = threads.spawn(|| return request_copies(&config, scene.common.as_path(), std::slice::from_ref(&second_request.id), &mut std::io::sink()));
        return (one.join().expect("first"), two.join().expect("second"));
    });
    assert_eq!(one.expect("first answered"), vec![(first_request.id.clone(), CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert_eq!(two.expect("second answered"), vec![(second_request.id.clone(), CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    drop(service);
    remove(scene.root.as_path());
}

/// A client that sends its request and goes away leaves the copy to the service, which finishes it.
#[test]
fn a_client_killed_after_sending_leaves_the_copy_to_the_service() {
    let scene: Scene = Scene::new("client-gone");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let service: RunningService = RunningService::start("gone", &scene);
    {
        let mut stream = std::os::unix::net::UnixStream::connect(service.socket.as_path()).expect("connect");
        let mut line: String = encode_client(&ClientMessage::Copy { common_dir: scene.common.clone(), ids: vec![request.id.clone()] });
        line.push('\n');
        stream.write_all(line.as_bytes()).expect("sent");
    }
    // A second client asks for the same request later: the service has run it, whoever asked first.
    let mut notices: Vec<u8> = Vec::new();
    let answers = request_copies(&config_for(service.socket.as_path()), scene.common.as_path(), std::slice::from_ref(&request.id), &mut notices)
        .expect("answered");
    assert!(matches!(answers.as_slice(), [(id, CopyOutcome::Copied(_) | CopyOutcome::AlreadyCopied)] if *id == request.id), "{answers:?}");
    assert_eq!(cache_names(topic.as_path()), vec!["cache", "cache/deep", "cache/deep/data", "cache/top"]);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    drop(service);
    remove(scene.root.as_path());
}

/// The files of the destination's `cache`, sorted.
fn cache_names(destination: &Path) -> Vec<String> {
    return crate::service_copy::tests::cache_tree(destination);
}

/// A service that dies in the middle of the conversation is asked again with the same request, and the answers it gave
///  before are kept.
#[test]
fn a_service_that_dies_mid_conversation_is_asked_again() {
    let (socket, directory) = socket_in_home("breaks");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let welcome: String = format!("{}\n", crate::service_protocol::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: PROTOCOL_VERSION }));
    let served = std::thread::spawn(move || {
        let mut sent_lines: Vec<String> = Vec::new();
        for attempt in 0..2 {
            let (mut stream, _address) = listener.accept().expect("accept");
            stream.write_all(welcome.as_bytes()).expect("welcome");
            let mut line: String = String::new();
            std::io::BufReader::new(stream.try_clone().expect("clone")).read_line(&mut line).expect("request");
            sent_lines.push(line);
            if attempt == 0 {
                // The service answers one request, begins the second, and dies.
                let mut done: String = encode_copy_reply(&CopyReply::Done { id: String::from("one"), outcome: CopyOutcome::Copied(2) });
                done.push('\n');
                stream.write_all(done.as_bytes()).expect("done");
                let mut started: String = encode_copy_reply(&CopyReply::Started { id: String::from("two"), destination: String::from("/worktrees/two") });
                started.push('\n');
                stream.write_all(started.as_bytes()).expect("started");
            } else {
                let mut done: String = encode_copy_reply(&CopyReply::Done { id: String::from("two"), outcome: CopyOutcome::Copied(3) });
                done.push('\n');
                done.push_str(encode_copy_reply(&CopyReply::Finished).as_str());
                done.push('\n');
                stream.write_all(done.as_bytes()).expect("done");
            }
        }
        return sent_lines;
    });
    let answers = request_copies(
        &config_for(socket.as_path()),
        Path::new("/repository/.git"),
        &[String::from("one"), String::from("two")],
        &mut std::io::sink(),
    )
    .expect("answered");
    assert_eq!(
        answers,
        vec![(String::from("one"), CopyOutcome::Copied(2)), (String::from("two"), CopyOutcome::Copied(3))]
    );
    let lines: Vec<String> = served.join().expect("server thread");
    assert!(lines[0].contains("\"ids\":[\"one\",\"two\"]"), "{}", lines[0]);
    assert!(lines[1].contains("\"ids\":[\"two\"]"), "the second send asks only for what is unanswered: {}", lines[1]);
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// A service that never starts is a failure that says what was saved and what to run.
#[test]
fn an_absent_service_is_a_plain_failure_that_names_the_saved_copy() {
    let (socket, directory) = socket_in_home("absent");
    let config: ClientConfig = ClientConfig {
        address: Bind::Path(socket),
        activation: Activation::Managed,
        start_wait: Duration::from_millis(200),
        commit_wait: Duration::from_secs(1),
    };
    let failure = request_copies(&config, Path::new("/repository/.git"), &[String::from("one")], &mut std::io::sink())
        .expect_err("no service");
    assert_eq!(failure, ClientFailure::DidNotStart);
    let text: String = copy_unavailable_text(&failure);
    assert!(text.contains("The copy is saved"), "{text}");
    assert!(text.contains(LOG_COMMAND), "{text}");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// Every failure the person can meet is worded without an implementation term, names what was saved, and never says the
///  worktree was not created.
#[test]
fn every_copy_failure_is_plain() {
    let failures: Vec<ClientFailure> = vec![
        ClientFailure::DidNotStart,
        ClientFailure::NoActivation(Platform::LinuxWithoutUserServices),
        ClientFailure::NoActivation(Platform::Windows),
        ClientFailure::NoActivation(Platform::Bsd),
        ClientFailure::NoActivation(Platform::Other),
        ClientFailure::OtherAccount,
        ClientFailure::NoCommonVersion,
        ClientFailure::ProgramMissing(String::from("/usr/bin/cli-git-service")),
        ClientFailure::CouldNotStart(String::from("the session bus refused")),
        ClientFailure::UnitInTheWay(String::from("/home/user/.config/systemd/user/cli-git.service")),
        ClientFailure::Unavailable,
        ClientFailure::MayNotHaveLanded,
    ];
    for failure in failures {
        let text: String = copy_unavailable_text(&failure);
        // A broken or silent connection says nothing about whether the files were copied, and the text does not claim it.
        let result_unknown: bool = matches!(failure, ClientFailure::Unavailable | ClientFailure::MayNotHaveLanded);
        if result_unknown {
            assert!(text.starts_with("cli-git: the ignored files may not have been copied into the new worktree"), "{text}");
            assert!(!text.contains("were not copied"), "{text}");
        } else {
            assert!(text.starts_with("cli-git: the ignored files were not copied into the new worktree, which was created."), "{text}");
        }
        assert!(text.contains("The copy is saved"), "{text}");
        let without_commands: String = text.split('`').step_by(2).collect::<Vec<&str>>().join(" ").to_lowercase();
        for word in DENYLIST {
            let present: bool = without_commands
                .split(|character: char| return !character.is_alphanumeric() && character != '-')
                .any(|token| return token == word);
            assert!(!present, "{word} reached the person in: {text}");
        }
    }
}

/// With no service route the requester fails loudly and the wake does nothing.
#[test]
fn without_a_route_the_requester_fails_loudly_and_the_wake_is_silent() {
    use crate::worktree_copy::CopyRequester;
    let mut requester: ServiceCopyRequester = ServiceCopyRequester {
        reach: Err(ClientFailure::NoActivation(Platform::LinuxWithoutUserServices)),
        provisioned: false,
        failed: Vec::new(),
        provisioner: count_provisioning,
    };
    let failure: WorktreeCopyError = requester
        .request(Path::new("/repository/.git"), &[String::from("one")], &mut std::io::sink())
        .expect_err("no route");
    assert!(failure.0.contains("loginctl enable-linger"), "{}", failure.0);
    requester.wake(Path::new("/repository/.git"));
}

//region The second review

/// A server's greeting for a given protocol version.
fn greeting(version: u32) -> String {
    return format!("{}\n", crate::service_protocol::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version }));
}

/// One reply line.
fn reply_line(reply: &CopyReply) -> String {
    return format!("{}\n", encode_copy_reply(reply));
}

/// A client whose requests are all answered returns at once; it does not wait for the service to finish the rest of a drain.
#[test]
fn a_client_returns_when_its_own_requests_are_answered() {
    let (socket, directory) = socket_in_home("early");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let (release, released) = std::sync::mpsc::channel::<()>();
    let served = std::thread::spawn(move || {
        let (mut stream, _address) = listener.accept().expect("accept");
        stream.write_all(greeting(PROTOCOL_VERSION).as_bytes()).expect("welcome");
        let mut line: String = String::new();
        std::io::BufReader::new(stream.try_clone().expect("clone")).read_line(&mut line).expect("request");
        stream
            .write_all(reply_line(&CopyReply::Done { id: String::from("one"), outcome: CopyOutcome::Copied(2) }).as_bytes())
            .expect("done");
        // The drain goes on with another worktree: the connection stays open and silent until the client has returned, and
        // closes after a few seconds if it has not.
        let _released = released.recv_timeout(Duration::from_secs(4));
    });
    let began: std::time::Instant = std::time::Instant::now();
    let answers = request_copies(&config_for(socket.as_path()), Path::new("/repository/.git"), &[String::from("one")], &mut std::io::sink())
        .expect("answered");
    assert_eq!(answers, vec![(String::from("one"), CopyOutcome::Copied(2))]);
    assert!(began.elapsed() < Duration::from_secs(3), "the client waited for the end of the drain: {:?}", began.elapsed());
    release.send(()).expect("release");
    served.join().expect("server thread");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// A copy that outlasts the grace is announced again and again while it runs, not once.
#[test]
fn progress_is_repeated_while_a_copy_runs() {
    let (socket, directory) = socket_in_home("progress");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let served = std::thread::spawn(move || {
        let (mut stream, _address) = listener.accept().expect("accept");
        stream.write_all(greeting(PROTOCOL_VERSION).as_bytes()).expect("welcome");
        let mut line: String = String::new();
        std::io::BufReader::new(stream.try_clone().expect("clone")).read_line(&mut line).expect("request");
        stream
            .write_all(reply_line(&CopyReply::Started { id: String::from("one"), destination: String::from("/worktrees/one") }).as_bytes())
            .expect("started");
        std::thread::sleep(Duration::from_millis(600));
        stream
            .write_all(reply_line(&CopyReply::Done { id: String::from("one"), outcome: CopyOutcome::Copied(2) }).as_bytes())
            .expect("done");
    });
    let mut notices: Vec<u8> = Vec::new();
    request_copies_paced(
        &config_for(socket.as_path()),
        Path::new("/repository/.git"),
        &[String::from("one")],
        &mut notices,
        (Duration::from_millis(60), Duration::from_millis(60)),
    )
    .expect("answered");
    let text: String = String::from_utf8_lossy(notices.as_slice()).into_owned();
    assert!(text.matches("still copying ignored files into /worktrees/one").count() >= 3, "{text}");
    served.join().expect("server thread");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// A client queued behind another copy, which has no copy of its own started, hears that it waits, and again.
#[test]
fn a_queued_client_hears_that_it_waits() {
    let (socket, directory) = socket_in_home("queued");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let served = std::thread::spawn(move || {
        let (mut stream, _address) = listener.accept().expect("accept");
        stream.write_all(greeting(PROTOCOL_VERSION).as_bytes()).expect("welcome");
        let mut line: String = String::new();
        std::io::BufReader::new(stream.try_clone().expect("clone")).read_line(&mut line).expect("request");
        std::thread::sleep(Duration::from_millis(500));
        stream
            .write_all(reply_line(&CopyReply::Done { id: String::from("one"), outcome: CopyOutcome::AlreadyCopied }).as_bytes())
            .expect("done");
    });
    let mut notices: Vec<u8> = Vec::new();
    request_copies_paced(
        &config_for(socket.as_path()),
        Path::new("/repository/.git"),
        &[String::from("one")],
        &mut notices,
        (Duration::from_millis(60), Duration::from_millis(60)),
    )
    .expect("answered");
    let text: String = String::from_utf8_lossy(notices.as_slice()).into_owned();
    assert!(text.matches("waiting for the commit service to get to this copy").count() >= 2, "{text}");
    served.join().expect("server thread");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// The wait for the replacement of an older service is bounded by time and not by a count of tries: the older service may
/// need several connections' worth of time to exit, and nothing but a request to retire is ever sent to it.
#[test]
fn a_client_waits_through_an_older_service_that_is_slow_to_exit() {
    let (socket, directory) = socket_in_home("slow-exit");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let older: usize = REPLACEMENTS_SEEN_BY_OLD_CLIENT + 2;
    let served = std::thread::spawn(move || {
        let mut lines: Vec<String> = Vec::new();
        for connection in 0..=older {
            let (mut stream, _address) = listener.accept().expect("accept");
            let version: u32 = if connection < older { PROTOCOL_VERSION - 1 } else { PROTOCOL_VERSION };
            stream.write_all(greeting(version).as_bytes()).expect("welcome");
            let mut line: String = String::new();
            let _timeout = stream.set_read_timeout(Some(Duration::from_secs(5)));
            std::io::BufReader::new(stream.try_clone().expect("clone")).read_line(&mut line).expect("a line");
            lines.push(line);
            if connection == older {
                stream
                    .write_all(reply_line(&CopyReply::Done { id: String::from("one"), outcome: CopyOutcome::Copied(1) }).as_bytes())
                    .expect("done");
            }
        }
        return lines;
    });
    let answers = request_copies(&config_for(socket.as_path()), Path::new("/repository/.git"), &[String::from("one")], &mut std::io::sink())
        .expect("answered after the older service went");
    assert_eq!(answers, vec![(String::from("one"), CopyOutcome::Copied(1))]);
    let lines: Vec<String> = served.join().expect("server thread");
    let (retires, copies): (Vec<&String>, Vec<&String>) = lines.iter().partition(|line| return line.contains("retire"));
    assert_eq!(retires.len(), older, "{lines:?}");
    assert_eq!(copies.len(), 1, "a copy was sent to an older service: {lines:?}");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// The number of retire attempts an older client made before it gave up; the wait here must outlast it.
const REPLACEMENTS_SEEN_BY_OLD_CLIENT: usize = crate::service_client::REPLACEMENTS;

/// A wake reaches the service the way a request does: an older service is asked to retire, and the current one is sent the
/// wake.
#[test]
fn a_wake_retires_an_older_service_and_wakes_the_current_one() {
    let (socket, directory) = socket_in_home("wake-retire");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let served = std::thread::spawn(move || {
        let mut lines: Vec<String> = Vec::new();
        for version in [PROTOCOL_VERSION - 1, PROTOCOL_VERSION] {
            let (mut stream, _address) = listener.accept().expect("accept");
            stream.write_all(greeting(version).as_bytes()).expect("welcome");
            let mut line: String = String::new();
            let _timeout = stream.set_read_timeout(Some(Duration::from_secs(5)));
            std::io::BufReader::new(stream).read_line(&mut line).expect("a line");
            lines.push(line);
        }
        return lines;
    });
    wake_copies(&config_for(socket.as_path()), Path::new("/repository/.git"));
    let lines: Vec<String> = served.join().expect("server thread");
    assert!(lines[0].contains("retire"), "{lines:?}");
    assert!(lines[1].contains("\"type\":\"copy\"") && lines[1].contains("\"ids\":[]"), "{lines:?}");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// The number of times the control provisioner was called.
static PROVISIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn count_provisioning(_host: &ProvisionHost) -> Result<(), ClientFailure> {
    PROVISIONS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    return Ok(());
}

/// A wake sets the service up through the session bus once, as the request that left the work pending did.
#[test]
fn a_wake_sets_the_service_up_once() {
    use crate::worktree_copy::CopyRequester;
    let (socket, directory) = socket_in_home("wake-provision");
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: directory.join("runtime"),
        user_unit_dir: None,
        bus: directory.join("no-bus"),
        service_binary: directory.join("service"),
        git: PathBuf::from("/usr/bin/git"),
    };
    let config: ClientConfig = ClientConfig {
        start_wait: Duration::from_millis(50),
        ..config_for(socket.as_path())
    };
    let mut requester: ServiceCopyRequester = ServiceCopyRequester {
        reach: Ok((config, Some(host))),
        provisioned: false,
        failed: Vec::new(),
        provisioner: count_provisioning,
    };
    let before: usize = PROVISIONS.load(std::sync::atomic::Ordering::SeqCst);
    requester.wake(Path::new("/repository/.git"));
    requester.wake(Path::new("/repository/.git"));
    assert_eq!(PROVISIONS.load(std::sync::atomic::Ordering::SeqCst) - before, 1);
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// A failure that was shown withdraws the notice the service left; a failure that was not shown leaves it.
#[test]
fn a_shown_failure_withdraws_its_notice() {
    use crate::worktree_copy::CopyRequester;
    let scene: Scene = Scene::new("client-notice");
    crate::service_work::write_notice(scene.common.as_path(), "copy-one", "cli-git: it failed.").expect("notice");
    crate::service_work::write_notice(scene.common.as_path(), "copy-other", "cli-git: another.").expect("notice");
    let mut requester: ServiceCopyRequester = ServiceCopyRequester {
        reach: Err(ClientFailure::DidNotStart),
        provisioned: false,
        failed: vec![String::from("one")],
        provisioner: count_provisioning,
    };
    requester.failures_shown(scene.common.as_path());
    let directory: PathBuf = crate::service_work::notice_directory(scene.common.as_path());
    assert!(!directory.join("copy-one.txt").exists(), "a shown failure left its notice");
    assert!(directory.join("copy-other.txt").exists(), "a failure that was not shown lost its notice");
    assert!(requester.failed.is_empty());
    remove(scene.root.as_path());
}

//endregion
