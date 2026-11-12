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
    assert_eq!(read_message(&mut reader, &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE), Some(greeting));
    assert_eq!(read_message(&mut reader, &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE), None, "the service closed the connection after its line");
    assert_eq!(read_message(&mut reader_after(b""), &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE), None, "a connection closed with nothing sent");
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
    assert_eq!(read_message(&mut reader_after(b"not a message\n"), &a_patience(), crate::service_protocol::HEARTBEAT_SILENCE), None);
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
        // The budget is the silence bound a command of these controls had before there was a budget at all.
        patience: Duration::from_secs(5),
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

/// What:
///  The settings of a client whose whole budget for the service is short,
///  so a control that measures the budget ends in a fraction of a second.
/// Why:
///  These controls assert that the budget, and not a step's own bound, is what ends a wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reachSettingsWithPatience({ startWait: 300, silence: 200, patience: 300 })
/// ```
fn settings_with_a_short_budget(address: &str) -> ClientConfig {
    return ClientConfig { patience: Duration::from_millis(300), ..reach_settings(address) };
}

/// A request the fake services below never look at,
/// so it needs no repository.
fn a_request(id: &str) -> CommitRequest {
    return CommitRequest {
        id: String::from(id),
        repository: std::path::PathBuf::from("/nowhere"),
        common_dir: std::path::PathBuf::from("/nowhere/.git"),
        git_dir: std::path::PathBuf::from("/nowhere/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
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
        holder: String::new(),
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

/// How long the stand-in of the unterminated-line control talks without ever ending a line:
/// longer than the budget of any control here, and shorter than the control's own patience.
const DRIBBLING: Duration = Duration::from_secs(5);

/// How many connections a stand-in service of these controls serves before its thread ends,
/// and how long it may take to serve them:
/// a bound of its own,
/// so no thread waits on a control that ended.
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
/// the words "did not start" are right only while no request reached a service,
/// so the change that keeps them from a sent request must not take them from an unsent one.
#[test]
fn a_request_that_was_never_sent_keeps_the_answer_that_nothing_answered() {
    let name: String = an_address("never-sent");
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &a_request("never-sent"), &mut std::io::sink())
        .expect_err("nothing listens at this address");
    assert_eq!(failure, ClientFailure::DidNotStart, "nothing was sent, so nothing landed");
}

/// What:
///  A stand-in service that takes one request,
///  says why the commit is waiting when it is asked about it,
///  and answers the withdrawal of the request with `answer`.
/// Why:
///  The interrupt arrives while the client waits,
///  so the commit must be one the service has not landed:
///  the answer to the withdrawal is what decides whether the person is told the commit was not made or that it landed.
fn a_service_that_withdraws(
    name: &str,
    wait_line: &str,
    answer: ServiceMessage,
) -> (std::sync::mpsc::Receiver<()>, std::thread::JoinHandle<()>) {
    let service: UnixListener = a_listener_on(name);
    let (arrived, seen) = std::sync::mpsc::channel::<()>();
    let line: String = wait_line.to_string();
    let serving = std::thread::spawn(move || {
        use std::io::BufRead as _;
        // The connection the request came on: it stays open, so the client's wait is on this service's silence.
        let (requested, _address) = service.accept().expect("the client connects");
        greet_and_read(&requested);
        let _sent = arrived.send(());
        // The connection the withdrawal comes on, which the client opens itself.
        let (withdrawing, _withdrawn) = service.accept().expect("the client asks to withdraw");
        let _greeting = (&withdrawing).write_all(the_line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
        let mut asked: String = String::new();
        std::io::BufReader::new(&withdrawing).read_line(&mut asked).expect("the withdrawal reaches the service");
        assert!(asked.contains("withdraw"), "the line the client sends asks to withdraw: {asked}");
        let _waiting = (&withdrawing).write_all(the_line(&ServiceMessage::Waiting { text: line }).as_slice());
        let _answer = (&withdrawing).write_all(the_line(&answer).as_slice());
    });
    return (seen, serving);
}

/// What:
///  A commit the person interrupts is withdrawn,
///  and the person is told that it was not made.
/// Why:
///   The request this command sent is the service's,
///  so ending the process where the signal arrives would leave it to land with nobody to read the answer;
///  the wrapper's own words for such a commit are "nothing lands later by itself".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [seen] = aServiceThatWithdraws('withdrawn', ServiceMessage.withdrawn(id)); stopOnArrival(seen); const failure = submit(...); expect(failure.kind).toBe('interrupted');
/// ```
#[test]
fn a_commit_the_person_interrupts_is_withdrawn_and_never_lands() {
    let name: String = an_address("interrupted");
    let (seen, serving) = a_service_that_withdraws(name.as_str(), "the commit waits for another program's index lock", ServiceMessage::Withdrawn { id: String::from("interrupted") });
    let interrupting = std::thread::spawn(move || {
        seen.recv_timeout(Duration::from_secs(30)).expect("the request reaches the service");
        crate::service_client_interrupted::stop_requested_for_test(libc::SIGINT);
    });
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &a_request("interrupted"), &mut std::io::sink()).expect_err("the service gave the request up");
    interrupting.join().expect("the interrupt is sent");
    drop(serving);
    let expected: ClientFailure = ClientFailure::Interrupted { signal: libc::SIGINT, blocker: Some(String::from("the commit waits for another program's index lock")) };
    assert_eq!(failure, expected, "the person is told the commit was not made");
    let words: String = crate::service_messages::render(&failure);
    assert!(words.contains("interrupted") && words.contains("was not made"), "{words}");
    assert!(words.contains(crate::service_messages::NOTHING_CHANGED), "{words}");
}

/// What:
///  A commit that landed before the signal came stays landed,
///  and the command says so.
/// Why:
///  The withdrawal is answered by the service between two landings,
/// and a request that landed first is answered as landed:
///  a commit that is on the branch is never reported as one that was not made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [seen] = aServiceThatWithdraws('landed', ServiceMessage.landed('interrupted', commit)); stopOnArrival(seen); expect(submit(...)).toEqual({ kind: 'landed', already: true });
/// ```
#[test]
fn a_commit_that_landed_before_the_signal_came_stays_landed() {
    let name: String = an_address("interrupted-after-landing");
    let commit: String = String::from("b".repeat(40));
    let (seen, serving) = a_service_that_withdraws(
        name.as_str(),
        "the commit waits for another program's index lock",
        ServiceMessage::Landed { id: String::from("interrupted-after-landing"), commit: commit.clone(), already: false, landing: None },
    );
    let interrupting = std::thread::spawn(move || {
        seen.recv_timeout(Duration::from_secs(30)).expect("the request reaches the service");
        crate::service_client_interrupted::stop_requested_for_test(libc::SIGINT);
    });
    let submitted: Submitted = submit(&reach_settings(name.as_str()), &a_request("interrupted-after-landing"), &mut std::io::sink()).expect("the commit landed");
    interrupting.join().expect("the interrupt is sent");
    drop(serving);
    assert_eq!(submitted, Submitted::Landed { commit, already: true }, "a commit on the branch is reported as landed");
}

/// The budget of a control that reads a line: the silence bound every command gives the service.
fn a_patience() -> crate::service_client_patience::Patience {
    return crate::service_client_patience::Patience::of(crate::service_protocol::HEARTBEAT_SILENCE);
}

/// What:
///  A service that sends bytes and never ends its line costs the client its budget and nothing more.
/// Why:
///  A line read by one `read_line` restarts the system's read timeout with every chunk it returns,
///  so a service that dribbles bytes and never writes a newline keeps the client inside one call for ever:
///  the third audit of release 0.6.1-alpha.12 watched eight seconds of it with no bound in sight
///  (finding r01 of `~/temp/agent/kimi-trials/kimi-audit-alpha-12-commit-service-landing`).
///  The client reads a line in chunks and asks the budget between two of them,
///  so it gives up, and it gives up once: a second connection would be another eight seconds.
#[test]
fn a_service_that_never_ends_its_line_costs_the_client_its_budget_and_nothing_more() {
    let name: String = an_address("unterminated-line");
    let service: UnixListener = a_listener_on(name.as_str());
    let connections: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::clone(&connections);
    let serving = std::thread::spawn(move || {
        let _set = service.set_nonblocking(true);
        let began: Instant = Instant::now();
        while began.elapsed() < STAND_IN_PATIENCE {
            match service.accept() {
                Ok((stream, _address)) => {
                    let _counted = counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // Bytes that are the start of no message, and never a newline, for longer than the budget of any control
                    //  here: the service talks and answers nothing.
                    let talking: Instant = Instant::now();
                    while talking.elapsed() < DRIBBLING {
                        if (&stream).write_all(b"{\"welcome\":").is_err() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    let began: Instant = Instant::now();
    let failure: ClientFailure = submit(&settings_with_a_short_budget(name.as_str()), &a_request("unterminated-line"), &mut std::io::sink())
        .expect_err("a service that never ends its line answers nothing");
    let waited: Duration = began.elapsed();
    drop(serving);
    assert!(matches!(failure, ClientFailure::NothingAnswered { .. }), "the words are the ones for nothing answering: {failure:?}");
    assert!(waited < Duration::from_secs(3), "the budget ended the wait in {waited:?}");
    assert_eq!(connections.load(std::sync::atomic::Ordering::SeqCst), 1, "the client waits for one service and makes one connection");
}

/// What:
///  A request whose write the service does not take ends within the budget instead of in the write.
/// Why:
///  A request near the protocol's one-mebibyte limit fills the connection's buffers,
///  and a service that stopped reading keeps the write waiting with no end:
///  a real service, stopped, held a commit there for the whole observation of the first audit of release 0.6.1-alpha.12
///  (`~/temp/agent/kimi-trials/kimi-audit-alpha-12-commit-path`,
///  finding 1).
///  Nothing the client reads afterwards would run, so the write itself is what the budget has to end.
#[test]
fn a_request_the_service_does_not_read_ends_within_the_budget_instead_of_in_the_write() {
    let name: String = an_address("write-not-taken");
    let service: UnixListener = a_listener_on(name.as_str());
    let serving = std::thread::spawn(move || {
        let (stream, _address) = service.accept().expect("the client connects");
        // The greeting, and then nothing: the connection is never read again.
        (&stream).write_all(the_line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice()).expect("greet");
        std::thread::sleep(STAND_IN_PATIENCE);
        drop(stream);
    });
    let mut request: CommitRequest = a_request("write-not-taken");
    // A request large enough to fill the connection: the connection holds about two hundred kilobytes.
    request.message = format!("{}\n", "m".repeat(crate::service_protocol::MAX_LINE_BYTES.div_ceil(2)));
    let began: Instant = Instant::now();
    let failure: ClientFailure = submit(&settings_with_a_short_budget(name.as_str()), &request, &mut std::io::sink()).expect_err("the service takes no request");
    let waited: Duration = began.elapsed();
    drop(serving);
    assert_eq!(failure, ClientFailure::ServiceSilent, "a service that does not read the request may have landed it, and nothing here says it did not");
    assert!(waited < Duration::from_secs(3), "the budget ended the write in {waited:?}");
}

/// What:
///  A service that takes the request and then says nothing is given up on once, not once per silence.
/// Why:
///  The service beats every second while it works, so a silence of the whole budget is a process that is not running,
///  and a second send to it costs the same wait again and cannot be read.
///  The commit's caller is what makes the commit then, so the wait is spent there.
#[test]
fn a_service_that_takes_the_request_and_says_nothing_is_given_up_on_once() {
    let name: String = an_address("silent-after-request");
    let service: UnixListener = a_listener_on(name.as_str());
    let connections: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::clone(&connections);
    let serving = std::thread::spawn(move || {
        let _set = service.set_nonblocking(true);
        let began: Instant = Instant::now();
        let mut kept: Vec<UnixStream> = Vec::new();
        while began.elapsed() < STAND_IN_PATIENCE {
            match service.accept() {
                Ok((stream, _address)) => {
                    let _counted = counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _greeting = (&stream).write_all(the_line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
                    // The request is read and nothing is ever answered.
                    use std::io::BufRead as _;
                    let mut asked: String = String::new();
                    let _read = std::io::BufReader::new(&stream).read_line(&mut asked);
                    kept.push(stream);
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    let began: Instant = Instant::now();
    let failure: ClientFailure = submit(&settings_with_a_short_budget(name.as_str()), &a_request("silent-after-request"), &mut std::io::sink())
        .expect_err("a service that answers nothing leaves the result not known");
    let waited: Duration = began.elapsed();
    drop(serving);
    assert_eq!(failure, ClientFailure::ServiceSilent);
    assert_eq!(connections.load(std::sync::atomic::Ordering::SeqCst), 1, "the request is not sent to a service that said nothing, in {waited:?}");
    assert!(waited < Duration::from_secs(2), "the budget ended the wait in {waited:?}");
}
