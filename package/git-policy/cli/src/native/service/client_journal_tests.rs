//! Controls of the entry the wrapper sends to the journal:
//! it carries the client's identifier,
//! priority,
//! message and fields,
//! and an address nobody listens at costs the command nothing.

use super::*;
use std::collections::BTreeMap;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};

/// The `NAME=value` lines of a datagram in simple form.
fn parsed(bytes: &[u8]) -> BTreeMap<String, String> {
    return std::str::from_utf8(bytes)
        .expect("an entry is text")
        .lines()
        .map(|line: &str| {
            let (name, value) = line.split_once('=').expect("every line of an entry is a field");
            return (name.to_string(), value.to_string());
        })
        .collect();
}

/// A bound receiving socket on a fresh abstract name,
/// and the target that names it.
fn receiver(name: &str) -> (UnixDatagram, Target) {
    let abstract_name: String = format!("cli-git-test-client-journal-{}-{name}", std::process::id());
    let address: SocketAddr = SocketAddr::from_abstract_name(abstract_name.as_bytes()).expect("an abstract address");
    return (UnixDatagram::bind_addr(&address).expect("bind the receiving socket"), Target::Abstract(abstract_name));
}

/// An entry reaches the receiver with the client's identifier,
/// its priority,
/// its message and each field.
#[test]
fn an_entry_reaches_the_receiver_as_the_client() {
    let (socket, target) = receiver("reach");
    let fields: [(&str, String); 1] = [("CLI_GIT_SLOW_CAUSE", String::from("uncommitted-changes"))];
    assert!(send_entry(&target, Priority::Warning, "the worktree was slow", &fields));
    let mut buffer: Vec<u8> = vec![0; 4096];
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("a read timeout");
    let length: usize = socket.recv(buffer.as_mut_slice()).expect("an entry");
    let found = parsed(&buffer[..length]);
    assert_eq!(found["SYSLOG_IDENTIFIER"], CLIENT_IDENTIFIER);
    assert_ne!(found["SYSLOG_IDENTIFIER"], crate::journald_native::IDENTIFIER, "the service keeps its own name");
    assert_eq!(found["PRIORITY"], Priority::Warning.number().to_string());
    assert_eq!(found["MESSAGE"], "the worktree was slow");
    assert_eq!(found["CLI_GIT_SLOW_CAUSE"], "uncommitted-changes");
}

/// An address nobody listens at is no failure of the command:
/// the entry is not sent and the answer says so.
#[test]
fn an_address_nobody_listens_at_takes_no_entry() {
    let nobody: Target = Target::Abstract(format!("cli-git-test-client-journal-nobody-{}", std::process::id()));
    assert!(!send_entry(&nobody, Priority::Warning, "x", &[]));
}

/// A name too long for a socket address is refused without a panic.
#[test]
fn a_name_too_long_for_a_socket_is_refused() {
    let long: Target = Target::Path("x".repeat(4096));
    assert!(!send_entry(&long, Priority::Info, "x", &[]));
}

/// A unit test that names no stand-in reaches no journal.
#[test]
fn a_unit_test_without_a_stand_in_reaches_nothing() {
    if std::env::var_os(TEST_SOCKET_VARIABLE).is_none() {
        assert!(!record_entry(Priority::Warning, "never sent", &[]));
    }
}

/// An entry sent as the service carries the service's identifier,
/// and the same send as the client keeps the client's.
#[test]
fn an_entry_can_carry_the_services_identifier() {
    let (socket, target) = receiver("service");
    assert!(send_entry_as(&target, crate::journald_native::IDENTIFIER, Priority::Warning, "from the service", &[]));
    let mut buffer: Vec<u8> = vec![0; 4096];
    let length: usize = socket.recv(buffer.as_mut_slice()).expect("an entry");
    assert_eq!(parsed(&buffer[..length]).get("SYSLOG_IDENTIFIER").map(String::as_str), Some("monochromatic-git-policy-cli-service"));
    assert!(send_entry(&target, Priority::Warning, "from the wrapper", &[]));
    let second: usize = socket.recv(buffer.as_mut_slice()).expect("an entry");
    assert_eq!(parsed(&buffer[..second]).get("SYSLOG_IDENTIFIER").map(String::as_str), Some("monochromatic-git-policy-cli"));
}

/// What:
///  Inside the service the client's sender sends nothing at all,
///  and an entry recorded there goes through the service's own sink under the identifier the caller names.
/// Why:
///   A reviewer of 2026-11-14 found the road after the streams were closed:
/// the service's own sink was kept off this machine's journal,
///  and code that runs inside the service still called this module's sender,
///  which with no test address names `/run/systemd/journal/socket` and looks at nothing else.
///  So the guard's reason is a route and a refusal:
/// a service's entry takes the sink,
///  and the sender it might have used refuses,
/// even when a caller hands it a target itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(inService()).toBe(true); expect(sendEntry(standIn, ...)).toBe(false); expect(entriesOf(() => recordEntryAs(CLIENT_IDENTIFIER, ...))[0].message).toContain('word-level merge')
/// ```
#[test]
fn a_service_s_entry_takes_the_sink_and_the_client_s_sender_refuses_inside_the_service() {
    use crate::service_timing::capture::{Captured, entries_of};
    let (socket, target) = receiver("service-route");
    let entries: Vec<Captured> = entries_of(|| {
        // A capture is what makes this process read as the service here (`service_timing::in_service`).
        assert!(in_service(), "this control runs as the service");
        assert!(!send_entry(&target, Priority::Warning, "sent by the client's sender", &[]), "the sender refuses inside the service");
        let fields: [(&str, String); 1] = [("CLI_GIT_WORD_MERGE_FILE", String::from("a.md"))];
        assert!(record_entry_as(CLIENT_IDENTIFIER, Priority::Warning, "word-level merge: a.md", &fields), "the recorder answers yes when the sink took it");
    });
    // The entry is one of the service's own, under the identifier the caller named.
    let recorded: &Captured = entries.iter().find(|entry| { return entry.message.contains("word-level merge"); }).expect("the entry the service recorded");
    assert_eq!(recorded.priority, Priority::Warning);
    assert!(recorded.fields.iter().any(|(name, value)| { return name == "CLI_GIT_WORD_MERGE_FILE" && value == "a.md"; }), "{recorded:?}");
    // Nothing reached the stand-in the sender was handed.
    socket.set_read_timeout(Some(std::time::Duration::from_millis(200))).expect("a read timeout");
    let mut buffer: Vec<u8> = vec![0; 4096];
    assert!(socket.recv(buffer.as_mut_slice()).is_err(), "the client's sender sent nothing inside the service");
}

/// The variable by which the parent of the routing control tells its child which route to drive,
/// and the one that tells it which name to record under.
const ROUTING_ROUTE_VARIABLE: &str = "CLI_GIT_TEST_JOURNAL_ROUTING_ROUTE";
const ROUTING_NAME_VARIABLE: &str = "CLI_GIT_TEST_JOURNAL_ROUTING_NAME";

/// The route the journal's socket is taken by.
const NATIVE_ROUTE: &str = "native";

/// The route standard error is taken by, the socket being absent.
const STREAM_ROUTE: &str = "stream";

/// The route nothing is taken by: the socket absent and standard error full.
const DROPPED_ROUTE: &str = "dropped";

/// What:
///  An entry the service records is written under the identifier it was given and answered with what became of it,
///  on every route: taken by the journal's socket, taken by standard error, and taken by neither.
/// Why:
///   A review of 2026-11-15 found the fallback line carrying the service's prefix whatever identifier the entry had,
/// and a dropped entry answered as sent.
///  The routes differ in the two variables a process must set before its sink is made and in whether standard error has
///  room,
///  which only a parent can make full: so each (route, identifier) pair runs in a child of this test binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const route of ['native', 'stream', 'dropped']) for (const name of [CLIENT_IDENTIFIER, IDENTIFIER]) expect(child(route, name)).toKeep(name)
/// ```
#[test]
fn an_entry_keeps_its_identifier_and_its_answer_on_every_route() {
    if std::env::var_os(ROUTING_ROUTE_VARIABLE).is_none() {
        let named: [(&str, &str); 2] = [("client", CLIENT_IDENTIFIER), ("service", crate::journald_native::IDENTIFIER)];
        for route in [NATIVE_ROUTE, STREAM_ROUTE, DROPPED_ROUTE] {
            for (tag, identifier) in named {
                drive_the_route_through_a_child(route, tag, identifier);
            }
        }
        return;
    }
    record_in_a_child();
}

/// This test binary, so a child of it can drive one route with an environment and a standard error of its own.
#[allow(clippy::disallowed_methods, reason = "starts this test binary (libtest), not the wrapper; the ban on `current_exe` guards the wrapper's own start")]
fn this_test_binary() -> std::path::PathBuf {
    return std::env::current_exe().expect("this test binary");
}

/// Fills a stream socket so its next write takes no byte.
fn fill(stream: &mut std::os::unix::net::UnixStream) {
    stream.set_nonblocking(true).expect("a non-blocking filler");
    let chunk: [u8; 4096] = [b'x'; 4096];
    let mut writes: usize = 0;
    loop {
        match std::io::Write::write(stream, &chunk) {
            Ok(0) => break,
            Ok(_) => writes += 1,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error) => panic!("filling the child's standard error failed: {error}"),
        }
        assert!(writes < 1024, "the child's standard error never filled");
    }
}

/// Runs one (route, identifier) pair in a child and reads what it wrote and what it answered.
fn drive_the_route_through_a_child(route: &str, tag: &str, identifier: &str) {
    let (mut reader, mut writer) = std::os::unix::net::UnixStream::pair().expect("a standard error pair");
    if route == DROPPED_ROUTE {
        fill(&mut writer);
    }
    let child: std::process::Output = std::process::Command::new(this_test_binary())
        .args(["--exact", "service::client_journal::tests::an_entry_keeps_its_identifier_and_its_answer_on_every_route", "--nocapture", "--test-threads=1"])
        .env(ROUTING_ROUTE_VARIABLE, route)
        .env(ROUTING_NAME_VARIABLE, tag)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::from(std::os::fd::OwnedFd::from(writer)))
        .output()
        .expect("the child of this test binary runs");
    let said: String = String::from_utf8_lossy(child.stdout.as_slice()).into_owned();
    let mut written: String = String::new();
    std::io::Read::read_to_string(&mut reader, &mut written).expect("the child's standard error is read");
    assert!(child.status.success(), "the child that drives {route} as {tag} failed:\n{said}\n{written}");
    assert!(said.contains("running 1 test"), "the child ran the control it was told to run: {said}");
    match route {
        NATIVE_ROUTE => {
            assert!(said.contains("answer=true"), "the journal's socket took the entry: {said}");
            assert!(said.contains(&format!("SYSLOG_IDENTIFIER={identifier}")), "the datagram names the entry under its own identifier: {said}");
        }
        STREAM_ROUTE => {
            assert!(said.contains("answer=true"), "standard error took the entry: {said}");
            assert!(written.contains(&format!("<4>{identifier}: ")), "the line on the journal's stream names the entry under its own identifier: {written:?}");
        }
        _ => assert!(said.contains("answer=false"), "an entry that went nowhere is not answered as sent: {said}"),
    }
}

/// The child: one route, one identifier, its own standard error, and this process's first entry.
fn record_in_a_child() {
    let route: String = std::env::var(ROUTING_ROUTE_VARIABLE).expect("the route the parent names");
    let tag: String = std::env::var(ROUTING_NAME_VARIABLE).expect("the name the parent names");
    let identifier: &str = match tag.as_str() {
        "client" => CLIENT_IDENTIFIER,
        "service" => crate::journald_native::IDENTIFIER,
        other => panic!("the parent names no such entry: {other}"),
    };
    let (device, inode) = crate::journald_native::stderr_identity().expect("standard error has an identity");
    // SAFETY: this child sets its own two variables before its sink is made, and no other thread of it reads them.
    unsafe { std::env::set_var(crate::journald_native::STREAM_VARIABLE, format!("{device}:{inode}")); }
    let stand_in: Option<UnixDatagram> = if route == NATIVE_ROUTE {
        let (socket, target) = receiver("routing");
        let Target::Abstract(name) = target else { panic!("the stand-in journal is an abstract socket") };
        // SAFETY: as above: the child names its own stand-in before the sink is made.
        unsafe { std::env::set_var(TEST_SOCKET_VARIABLE, format!("@{name}")); }
        Some(socket)
    } else {
        // An abstract name nothing listens at: the native delivery fails at once.
        // SAFETY: as above.
        unsafe { std::env::set_var(TEST_SOCKET_VARIABLE, format!("@cli-git-test-routing-absent-{}", std::process::id())); }
        None
    };
    // The context is what makes this process read as the service, and the service records through its own sink.
    crate::service_timing::set_context("review", None);
    assert!(crate::service_timing::in_service(), "this child runs as the service");
    let sent: bool = record_entry_as(identifier, Priority::Warning, "word-level merge: a.md", &[]);
    println!("answer={sent}");
    match stand_in {
        Some(socket) => {
            socket.set_read_timeout(Some(std::time::Duration::from_secs(10))).expect("a read timeout");
            let mut buffer: Vec<u8> = vec![0; 4096];
            let length: usize = socket.recv(buffer.as_mut_slice()).expect("the entry reached the stand-in journal");
            println!("SYSLOG_IDENTIFIER={}", parsed(&buffer[..length])["SYSLOG_IDENTIFIER"]);
            assert!(sent, "the journal's socket took the entry, so the answer says so");
        }
        None if route == STREAM_ROUTE => assert!(sent, "standard error took the entry, so the answer says so"),
        None => assert!(!sent, "an entry that went nowhere is not answered as sent"),
    }
}
