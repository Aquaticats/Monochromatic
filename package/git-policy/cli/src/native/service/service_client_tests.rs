//! Controls for the client's wait bounds and for how it reads the lines the service sends.

use crate::service_protocol::{PROTOCOL_VERSION, RELEASE, decode_service};
use super::*;
use crate::service_protocol::{HEARTBEAT_EVERY, HEARTBEAT_SILENCE, encode_service};
/// The socket a stand-in service listens on and the address it names.
use crate::service_server::Bind;
use std::os::unix::net::UnixListener;

/// A commit queued behind a full batch of landings has no queue wait of its own,
/// so the wait ends on the silence of the service,
/// which is the service's own beat several times over,
/// or at `ANSWER_CEILING`.
#[test]
fn the_silence_bound_is_several_beats_so_one_late_beat_is_not_a_stopped_service() {
    assert!(HEARTBEAT_SILENCE >= HEARTBEAT_EVERY * MINIMUM_BEATS_MISSED, "{HEARTBEAT_SILENCE:?} against a beat every {HEARTBEAT_EVERY:?}");
}

/// The fewest beats of silence a stopped service is told from a late one by.
const MINIMUM_BEATS_MISSED: u32 = 3;

/// A reader over a connection whose service sent `bytes` and then closed it.
fn reader_after(bytes: &[u8]) -> BufReader<UnixStream> {
    let (client, mut service) = UnixStream::pair().expect("a connected pair");
    service.write_all(bytes).expect("the service writes");
    drop(service);
    return BufReader::new(client);
}

/// A line the service sent is read as its message,
/// and a connection the service closed reads as no message.
#[test]
fn a_line_is_read_as_a_message_and_a_closed_connection_as_none() {
    let greeting: ServiceMessage = ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) };
    let mut line: String = encode_service(&greeting);
    line.push('\n');
    let mut reader: BufReader<UnixStream> = reader_after(line.as_bytes());
    assert_eq!(read_message(&mut reader), Some(greeting));
    assert_eq!(read_message(&mut reader), None, "the service closed the connection after its line");
    assert_eq!(read_message(&mut reader_after(b"")), None, "a connection closed with nothing sent");
}

/// A line that is not a message reads as none,
/// which is what the empty line of a closed connection is too:
/// the check that a
/// read took some bytes,
/// before decoding,
/// changes no result,
/// because the empty line decodes to nothing by itself.
#[test]
fn an_empty_or_damaged_line_decodes_to_no_message() {
    assert!(decode_service("").is_err(), "an empty line is not a message");
    assert_eq!(read_message(&mut reader_after(b"not a message\n")), None);
}

/// A read-only failure is the frozen-move one only when the move's record covers the repository:
/// with no record,
/// or a record of another place,
/// it stays the plain read-only one.
#[test]
fn a_read_only_failure_is_the_frozen_move_one_only_when_the_move_record_covers_the_repository() {
    let state: std::path::PathBuf = crate::test_support::fixture("client-read-only-freeze");
    assert_eq!(read_only_failure("/work/repo", None), ClientFailure::RepositoryReadOnly(String::from("/work/repo")), "no state folder");
    assert_eq!(read_only_failure("/work/repo", Some(state.as_path())), ClientFailure::RepositoryReadOnly(String::from("/work/repo")), "no manifest");
    std::fs::write(state.join("freeze-manifest.tsv"), "0a1b\t/work/repo\n").expect("manifest");
    assert_eq!(read_only_failure("/work/repo/package/x", Some(state.as_path())), ClientFailure::RepositoryFrozen(String::from("/work/repo/package/x")));
    assert_eq!(read_only_failure("/work/other", Some(state.as_path())), ClientFailure::RepositoryReadOnly(String::from("/work/other")), "a repository the move did not freeze");
    crate::test_support::remove(state.as_path());
}

/// What:
///  The settings of a client that reaches `address` and asks no manager:
/// the waits are milliseconds,
/// so a control ends in well under a second.
/// Why:
///   `reach` ends at the start wait when nothing answers and no manager can be asked,
/// which is the wait these controls turn into a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reachConfig({ socket: address, startWait: 300, silence: 200 })
/// ```
fn reach_settings(address: &str) -> ClientConfig {
    return ClientConfig {
        address: Bind::Abstract(String::from(address)),
        activation: Activation::Managed,
        start_wait: Duration::from_millis(300),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: None,
        keep_fresh: None,
        answer_ceiling: crate::wait_ceilings::ANSWER_CEILING,
        withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING,
        waiting_notice_after: crate::wait_ceilings::WAITING_NOTICE_AFTER,
        extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT,
        start_ceiling: crate::wait_ceilings::START_CEILING,
        units_record_dir: None,
    };
}

/// A request the fake services below never look at,
/// so it needs no repository.
fn a_request(id: &str) -> CommitRequest {
    return CommitRequest {
        id: String::from(id),
        repository: std::path::PathBuf::from("/nowhere"),
        common_dir: std::path::PathBuf::from("/nowhere/.git"),
        git_dir: std::path::PathBuf::from("/nowhere/.git"),
        reference: String::from("refs/heads/main"),
        base: "a".repeat(40),
        changes: Vec::new(),
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: std::path::PathBuf::from("/nowhere/.git/index"),
        scratch: std::path::PathBuf::from("/nowhere/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-a-0"),
        captured: 0,
    };
}

/// An address no other control of this process uses.
fn an_address(label: &str) -> String {
    return format!("cli-git-client-test-{}-{label}", std::process::id());
}

/// A listener on the abstract name `name`.
fn a_listener_on(name: &str) -> UnixListener {
    use std::os::linux::net::SocketAddrExt;
    let socket: std::os::unix::net::SocketAddr = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    return UnixListener::bind_addr(&socket).expect("bind the stand-in service");
}

/// The bytes of one line of `message`.
fn the_line(message: &ServiceMessage) -> Vec<u8> {
    let mut text: String = encode_service(message);
    text.push('\n');
    return text.into_bytes();
}

/// Greets as a service of this release and reads the request the client sends,
/// so the client's send is a send that happened.
fn greet_and_read(stream: &UnixStream) {
    use std::io::BufRead;
    (&*stream).write_all(the_line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice()).expect("greet");
    let mut request: String = String::new();
    std::io::BufReader::new(stream).read_line(&mut request).expect("the client sends its request");
}

/// How many connections a stand-in service of these controls serves before its thread ends,
/// and how long it may take to serve them:
/// a bound of its own, so no thread waits on a control that ended.
const SERVED: usize = 8;
const STAND_IN_PATIENCE: Duration = Duration::from_secs(30);

/// What:
///  A request that was sent and whose service could not be reached again is not reported as a commit that was not made.
/// Why:
///   The audit of service restarts and version skew (2026-11-09,
/// `~/temp/agent/sonnet-audits/service-restart-and-version-skew/final.md`)
/// saw exit 1 with "this commit was not made ... Nothing in the repository changed" while the commit was on the branch:
/// the client reached the service with `?` after a send,
/// and a failure to reach one became that sentence.
/// A request that was sent may have landed,
/// so no failure to reach a service again may say it did not;
/// the words for a result that is not known are the ones that are true.
#[test]
fn a_request_that_was_sent_is_not_reported_as_a_commit_that_was_not_made() {
    let name: String = an_address("sent-then-gone");
    let service: UnixListener = a_listener_on(name.as_str());
    let serving = std::thread::spawn(move || {
        // The one service this release has: it takes the request, and then it is gone.
        let (stream, _address) = service.accept().expect("the client connects");
        greet_and_read(&stream);
        drop(stream);
        drop(service);
    });
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &a_request("sent-then-gone"), &mut std::io::sink())
        .expect_err("the service is gone after the request was sent");
    let _joined = serving.join();
    assert_eq!(failure, ClientFailure::ServiceSilent, "a request that was sent may have landed, so nothing may say it was not made");
}

/// What:
///  A request that was sent and then met by a service of a newer release,
/// which reads no retire of this one,
/// is not reported as a commit that was not made either.
/// Why:
///   The same audit reproduced this route on the worktree build
/// (`f1c_new_client_other_release_after_commit_landed.ts`:
/// the service was killed after the branch moved and the unit named another release),
/// and the sentence it printed was false.
#[test]
fn a_request_that_was_sent_is_not_reported_as_a_commit_that_was_not_made_when_a_newer_service_answers() {
    let name: String = an_address("sent-then-newer");
    let service: UnixListener = a_listener_on(name.as_str());
    let _set = service.set_nonblocking(true);
    let serving = std::thread::spawn(move || {
        let began: Instant = Instant::now();
        let mut served: usize = 0;
        while served < SERVED && began.elapsed() < STAND_IN_PATIENCE {
            match service.accept() {
                Ok((stream, _address)) => {
                    served += 1;
                    if served == 1 {
                        greet_and_read(&stream);
                        continue;
                    }
                    // From the second connection on: a service of a newer release, which this client cannot ask to retire.
                    let newer: ServiceMessage = ServiceMessage::Welcome { version: PROTOCOL_VERSION + 1, release: Some(String::from("a-newer-release")) };
                    (&stream).write_all(the_line(&newer).as_slice()).expect("greet as a newer release");
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &a_request("sent-then-newer"), &mut std::io::sink())
        .expect_err("a service of a newer release cannot answer this client");
    // The stand-in ends by its own bound: the client needs one greeting, and waiting for the thread would make this control
    //  as long as that bound.
    drop(serving);
    assert_eq!(failure, ClientFailure::ServiceSilent, "a request that was sent may have landed, so nothing may say it was not made");
}

/// What:
///  A request that was never sent keeps the answer that nothing answered,
/// which is true when nothing was sent.
/// Why:
///   The control of the two controls above:
/// the words "did not start, so this commit was not made" are right only while no request reached a service,
/// so the change that keeps them from a sent request must not take them from an unsent one.
#[test]
fn a_request_that_was_never_sent_keeps_the_answer_that_nothing_answered() {
    let name: String = an_address("never-sent");
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &a_request("never-sent"), &mut std::io::sink())
        .expect_err("nothing listens at this address");
    assert_eq!(failure, ClientFailure::DidNotStart, "nothing was sent, so nothing landed");
}
