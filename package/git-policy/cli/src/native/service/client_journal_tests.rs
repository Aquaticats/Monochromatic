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
