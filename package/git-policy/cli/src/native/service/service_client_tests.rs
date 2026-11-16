//! Controls for the client's wait bounds and for how it reads the lines the service sends.

use crate::service_client_listen_and_ask_to_retire::read_message;
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
        shadow: None,
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
    assert_eq!(
        failure,
        ClientFailure::DidNotStart { unit: None, answer: crate::service_messages_of_a_start_that_was_asked_and_did_not_come::ManagerLastWords::Silent, asked: crate::service_messages_of_a_start_that_was_asked_and_did_not_come::RequestedStart::Nothing },
        "nothing was sent, so nothing landed"
    );
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
        crate::service_client_interrupted::stop_requested_for_test(libc::SIGINT, Some(std::process::id() as i32));
    });
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &a_request("interrupted"), &mut std::io::sink()).expect_err("the service gave the request up");
    interrupting.join().expect("the interrupt is sent");
    drop(serving);
    let expected: ClientFailure = ClientFailure::Interrupted { signal: libc::SIGINT, sender: Some(std::process::id() as i32), blocker: Some(String::from("the commit waits for another program's index lock")) };
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
        crate::service_client_interrupted::stop_requested_for_test(libc::SIGINT, None);
    });
    let submitted: Submitted = submit(&reach_settings(name.as_str()), &a_request("interrupted-after-landing"), &mut std::io::sink()).expect("the commit landed");
    interrupting.join().expect("the interrupt is sent");
    drop(serving);
    assert_eq!(submitted, Submitted::Landed { commit, already: true }, "a commit on the branch is reported as landed");
}

/// What:
///  A give-up on the write of a request after an earlier attempt had delivered it whole is "not known",
///  not "the service never took it".
/// Why:
///  The review of the bounded write found that the two ends disagreed:
/// a first attempt that put the whole line on the wire may be the service's already,
/// so a later attempt that gives up cannot say the commit was not made — that verdict removes the scratch tree a
///  landing still reads and invites the person to make the commit twice.
///  The control has the stand-in read the first request whole and answer nothing, then take the second connection and read
///  nothing at all, so the client's second write fills the connection's buffers and ends at its silence bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = submit(config, aBigRequest); expect(failure.kind).toBe('may-not-have-landed')
/// ```
#[test]
fn a_give_up_after_a_whole_delivery_says_the_result_is_not_known() {
    let name: String = an_address("give-up-after-whole");
    let service: UnixListener = a_listener_on(name.as_str());
    let serving = std::thread::spawn(move || {
        // The first connection: the request arrives whole, and nothing is answered.
        let (first, _first_address) = service.accept().expect("the client connects");
        greet_and_read(&first);
        drop(first);
        // The second connection: greeted, and not one byte of the request read.
        let (second, _second_address) = service.accept().expect("the client sends again");
        (&second).write_all(the_line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice()).expect("greet");
        std::thread::sleep(STAND_IN_PATIENCE);
        drop(second);
    });
    let request: CommitRequest = a_request_of_many_changes("give-up-after-whole");
    let failure: ClientFailure = submit(&reach_settings(name.as_str()), &request, &mut std::io::sink())
        .expect_err("a request whose second write stalls cannot be answered");
    drop(serving);
    assert_eq!(
        failure,
        ClientFailure::MayNotHaveLanded,
        "an earlier attempt delivered the whole line, so nothing may say the service never took it"
    );
    assert!(!crate::service_commit_tree_ownership::the_service_never_took(&Err(failure.clone())), "and the scratch tree stays with the service's claim on it");
}

/// A request whose line is larger than one connection's buffers,
/// so a service that reads nothing makes its write stall.
fn a_request_of_many_changes(id: &str) -> CommitRequest {
    let mut request: CommitRequest = a_request(id);
    for index in 0..A_CHANGE_SET_OF_MANY_PATHS {
        let mut path: String = format!("directory-{index}/");
        while path.len() < A_PATH_THIS_LONG {
            path.push('p');
        }
        request.changes.push(crate::service_protocol::Change {
            path: std::path::PathBuf::from(path),
            mode: 33_188,
            blob: Some(String::from("0123456789abcdef0123456789abcdef01234567")),
        });
    }
    return request;
}

/// How many changes a control puts in a request so that its line cannot fit one connection's buffers.
/// Measured on this host: both buffers of a local socket hold about 233,152 bytes,
/// and a change of `A_PATH_THIS_LONG` characters is about 200 bytes of line.
const A_CHANGE_SET_OF_MANY_PATHS: usize = 2_000;

/// The length of a changed path in that request,
/// near the 99th percentile of this repository's own (118 characters).
const A_PATH_THIS_LONG: usize = 150;

/// What:
///  A stand-in service that greets and then reads nothing of the connection it greeted,
///  answering a connection the way `answer` says a service that read the line would.
/// Why:
///   The state a service is in when its own thread stopped inside a read is one no control on this machine may make by
///  loading the host,
///  so the stand-in is the test's own:
///  it accepts every connection for its whole patience,
///  greets each,
///  and does `after` with it.
fn a_service_that_reads_nothing(name: &str, after: AfterGreeting) -> std::thread::JoinHandle<()> {
    let service: UnixListener = a_listener_on(name);
    let _set = service.set_nonblocking(true);
    return std::thread::spawn(move || {
        let began: Instant = Instant::now();
        let mut served: usize = 0;
        while served < SERVED && began.elapsed() < STAND_IN_PATIENCE {
            let Ok((mut stream, _address)) = service.accept() else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            served += 1;
            (&stream).write_all(the_line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice()).expect("greet");
            match after {
                // Not one byte of the line is read, and the connection is left open: what a service that stopped reading
                //  leaves a wake to find.
                AfterGreeting::Nothing => std::thread::sleep(STAND_IN_PATIENCE),
                // The whole line is read and the connection kept open and busy, which is a service that has the line and
                //  breaks the protocol by never closing.
                AfterGreeting::ReadingTheLineAndTalking => {
                    use std::io::BufRead;
                    let mut line: String = String::new();
                    std::io::BufReader::new(&stream).read_line(&mut line).expect("the client sends its line");
                    for _ in 0..STAND_IN_PATIENCE.as_secs() {
                        std::thread::sleep(Duration::from_millis(500));
                    }
                }
                // The whole line is read and the connection closed without a word, which is what a service does once it has
                //  taken a work line.
                AfterGreeting::ReadingTheLine => {
                    use std::io::BufRead;
                    let mut line: String = String::new();
                    std::io::BufReader::new(&stream).read_line(&mut line).expect("the client sends its line");
                    drop(stream);
                }
                // Everything the connection holds is drained every pause, which is progress the write's clock sees, and
                //  the drains are too few to finish the line within the ceiling.
                AfterGreeting::Draining => {
                    use std::io::Read;
                    stream.set_nonblocking(true).expect("the draining stand-in reads without waiting");
                    let mut buffer: [u8; A_DRAINED_CHUNK] = [0; A_DRAINED_CHUNK];
                    let draining: Instant = Instant::now();
                    while draining.elapsed() < STAND_IN_PATIENCE {
                        while stream.read(&mut buffer).is_ok_and(|taken: usize| { return taken > 0; }) {}
                        std::thread::sleep(A_DRAIN_PAUSE);
                    }
                }
                // The line is read and answered as the service refuses a work line.
                AfterGreeting::RefusingTheLine(code) => {
                    use std::io::BufRead;
                    let mut line: String = String::new();
                    std::io::BufReader::new(&stream).read_line(&mut line).expect("the client sends its line");
                    (&stream).write_all(the_line(&ServiceMessage::Failed { id: String::new(), code }).as_slice()).expect("refuse the line");
                    drop(stream);
                }
            }
        }
    });
}

/// What a stand-in service does with a connection once it has greeted it.
#[derive(Clone, Copy)]
enum AfterGreeting {
    /// Reads not one byte of it and leaves it open.
    Nothing,
    /// Reads the client's one line and closes the connection silently.
    ReadingTheLine,
    /// Reads the client's one line and holds the connection open, sending nothing and closing nothing.
    ReadingTheLineAndTalking,
    /// Drains everything the connection holds every pause, and never finishes the line.
    Draining,
    /// Reads the client's one line and answers it with a refusal.
    RefusingTheLine(crate::service_protocol::FailureCode),
}

/// What:
///  A wake of a few hundred bytes into a service that greets and reads nothing answers that nothing witnessed the read,
///  within the bound a wake has today.
/// Why:
///   This is the claim of the review of 2026-11-13 ("Out-of-scope observation"),
///  run rather than read:
///  a wake line fits one connection's buffers,
///  so its write goes in whole even where the service reads nothing,
///  and the write's own return said nothing about the service.
///  The answer a command must not hand work over on is therefore the close, not the write.
///  The bound is the two the wake already had:
///  the write's silence window and the client's silence bound for the close
///  (`wait_ceilings.rs`).
#[test]
fn a_wake_the_service_read_nothing_of_is_no_answer_that_it_holds_the_record() {
    let name: String = an_address("wake-nothing-read");
    let serving = a_service_that_reads_nothing(name.as_str(), AfterGreeting::Nothing);
    // The waits are the milliseconds of every control here, so a wake of this file ends in well under a second; the
    //  production bounds are the same two waits with the numbers of `wait_ceilings.rs`.
    let began: Instant = Instant::now();
    let woken: Result<Woken, ClientFailure> = wake(&reach_settings(name.as_str()), std::path::Path::new("/nowhere/.git"), None, &mut std::io::sink());
    let took: Duration = began.elapsed();
    drop(serving);
    assert_eq!(woken, Ok(Woken::Buffered), "the whole line went in and the connection still holds it, so the service never read it");
    assert!(took >= Duration::from_millis(200), "the answer came once the close bound passed, not before ({took:?})");
    assert!(took < Duration::from_secs(2), "and within the bound a wake has ({took:?})");
}

/// What:
///  A service that read the line and then keeps talking without closing is work it holds,
///  because the connection holds none of the line any more.
/// Why:
///   The control of the control above:
///  a service that never closes is told from one that never read by the bytes it took out of the connection,
///  and a wake reported as unread there would take work out of a service that has it.
///  The whole hand-over this answers for returns at its own bound whatever the service keeps sending
///  (`worktree_trash_hand_over_tests.rs`,
///  `a_service_that_keeps_talking_does_not_hold_the_hand_over`).
#[test]
fn a_wake_the_service_read_and_closed_nothing_of_is_work_it_holds() {
    let name: String = an_address("wake-read-chatty");
    let serving = a_service_that_reads_nothing(name.as_str(), AfterGreeting::ReadingTheLineAndTalking);
    let began: Instant = Instant::now();
    let woken: Result<Woken, ClientFailure> = wake(&reach_settings(name.as_str()), std::path::Path::new("/nowhere/.git"), None, &mut std::io::sink());
    let took: Duration = began.elapsed();
    drop(serving);
    assert_eq!(woken, Ok(Woken::Taken), "the service took every byte of the line out of the connection, so it holds the record");
    assert!(took >= Duration::from_millis(200), "the bound still ends the wait for a close that never comes ({took:?})");
}

/// What:
///  A wake whose line the service reads and closes answers that it holds the record,
///  which is the one answer a command may hand work over on.
/// Why:
///   The control of the control above:
///  the close is what says the service read the line,
///  so a stand-in that reads and closes must be told from one that only greets.
#[test]
fn a_wake_the_service_reads_and_closes_is_work_it_holds() {
    let name: String = an_address("wake-read");
    let serving = a_service_that_reads_nothing(name.as_str(), AfterGreeting::ReadingTheLine);
    let woken: Result<Woken, ClientFailure> = wake(&reach_settings(name.as_str()), std::path::Path::new("/nowhere/.git"), None, &mut std::io::sink());
    drop(serving);
    assert_eq!(woken, Ok(Woken::Taken), "the service read the line and closed the connection");
}

/// What:
///  A service that answers the line with a refusal is reported as refusing it,
///  not as holding it.
/// Why:
///  The service refuses a work line for another instance's repository or when it cannot tell whose it is,
///  and nothing of this wake reaches a queue in either case
///  (`service_server_connection_handling.rs`, `answer_request`).
#[test]
fn a_wake_the_service_refuses_is_not_work_it_holds() {
    let name: String = an_address("wake-refused");
    let serving = a_service_that_reads_nothing(name.as_str(), AfterGreeting::RefusingTheLine(crate::service_protocol::FailureCode::Rejected));
    let woken: Result<Woken, ClientFailure> = wake(&reach_settings(name.as_str()), std::path::Path::new("/nowhere/.git"), None, &mut std::io::sink());
    drop(serving);
    assert_eq!(woken, Ok(Woken::Refused { code: crate::service_protocol::FailureCode::Rejected }), "the service answered that it will not act on the line");
}

/// What:
///  A wake line larger than both buffers of the connection,
///  sent to a service that reads nothing,
///  answers that no byte of it was taken.
/// Why:
///  This is the case the review's sentence named,
///  and it is reachable only where the line is too large to fit the buffers:
///  the write then waits in the one connection it has,
///  and the silence window ends it.
///  A repository's Git directory path is nowhere near this long,
///  so the case is proved with a path a wake carries and nothing reads.
#[test]
fn a_wake_line_too_large_for_the_buffers_is_no_byte_taken() {
    let name: String = an_address("wake-too-large");
    let serving = a_service_that_reads_nothing(name.as_str(), AfterGreeting::Nothing);
    let git_dir: std::path::PathBuf = std::path::PathBuf::from(format!("/nowhere/{}", "d".repeat(A_WAKE_LINE_LARGER_THAN_THE_BUFFERS)));
    let woken: Result<Woken, ClientFailure> = wake(&reach_settings(name.as_str()), git_dir.as_path(), None, &mut std::io::sink());
    drop(serving);
    assert_eq!(woken, Ok(Woken::NothingTaken), "the line filled both buffers and the window passed with no byte taken");
}

/// What:
///  A wake line a service reads slowly, without ever letting the silence window pass,
///  answers that part of it was taken.
/// Why:
///  The ceiling of a write is what ends that one,
///  and the service takes a line only when the whole of it and its newline arrived,
///  so part of a line is a wake it never acted on.
#[test]
fn a_wake_line_read_too_slowly_is_partly_taken() {
    let name: String = an_address("wake-too-slow");
    let serving = a_service_that_reads_nothing(name.as_str(), AfterGreeting::Draining);
    let git_dir: std::path::PathBuf = std::path::PathBuf::from(format!("/nowhere/{}", "d".repeat(A_WAKE_LINE_MANY_BUFFERS_LONG)));
    let began: Instant = Instant::now();
    let woken: Result<Woken, ClientFailure> = wake(&reach_settings(name.as_str()), git_dir.as_path(), None, &mut std::io::sink());
    let took: Duration = began.elapsed();
    drop(serving);
    assert_eq!(woken, Ok(Woken::PartTaken), "the ceiling of a write ended it with part of the line in ({took:?})");
}

/// The path a wake's line is made larger than both buffers of one connection with.
/// Why:
///  Both buffers of a local stream socket hold about 233,152 bytes (measured on this host, 2026-11-12,
///  `wait_ceilings.rs`), so a directory path of this many characters fills them and the rest waits in the kernel.
const A_WAKE_LINE_LARGER_THAN_THE_BUFFERS: usize = 400_000;

/// The path a wake's line is made many buffers long with,
/// so a stand-in that drains the connection every pause still has line left when the ceiling of a write passes.
/// Why:
///  A drain re-arms the write only once the peer's queue is empty,
///  so the line is carried to the service one bufferful at a time,
///  and the ceiling is the only bound that can end such a write.
const A_WAKE_LINE_MANY_BUFFERS_LONG: usize = 1_600_000;

/// How much of the connection a draining stand-in reads at a time, and how long it pauses between drains:
/// inside the silence window, so the write sees progress, and too few drains to finish many buffers within the ceiling.
const A_DRAINED_CHUNK: usize = 65_536;
const A_DRAIN_PAUSE: Duration = Duration::from_secs(2);
