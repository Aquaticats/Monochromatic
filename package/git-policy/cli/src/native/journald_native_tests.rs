//! What:
//!  Controls of the native journal entry, of the detection that standard error is the journal, and of the fallback.
//! Why:
//!  A message that starts a field of its own, a sink chosen from a variable that a redirection left behind, or an entry
//!  that waits for a full socket each cost a commit or a record; none touches the real journal socket here: every
//!  socket is a disposable one in a fixture directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseDatagram(datagram('info', 'a\nPRIORITY=0', []))).toEqual({ PRIORITY: '6', MESSAGE: 'a PRIORITY=0' });
//! ```

use super::{CUT_MARK, Delivery, FIELD_LIMIT, IDENTIFIER, MESSAGE_LIMIT, Priority, Sink, StreamWriter, Target, datagram, raw_address, one_line, stream_matches, valid_field_name};
use std::collections::BTreeMap;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};
use std::sync::atomic::{AtomicU32, Ordering};

/// The device and inode a fixture says standard error has.
const DEVICE: u64 = 64_769;
const INODE: u64 = 123_456_789;

/// The `NAME=value` lines of a datagram, in simple form; a repeated name keeps its last value.
fn parsed(bytes: &[u8]) -> BTreeMap<String, String> {
    let text: &str = std::str::from_utf8(bytes).expect("an entry is text");
    return text
        .lines()
        .map(|line| {
            let (name, value) = line.split_once('=').expect("every line of an entry is a field");
            return (name.to_string(), value.to_string());
        })
        .collect();
}

/// Distinguishes the abstract names of the receivers of one test process.
static RECEIVERS: AtomicU32 = AtomicU32::new(0);

/// A bound receiving socket on a fresh abstract name (a fixture path can be longer than a socket path may be, and an
/// abstract socket leaves nothing behind), and the target that names it.
fn receiver(name: &str) -> (UnixDatagram, Target) {
    let abstract_name: String = format!("cli-git-test-journal-{}-{}-{name}", std::process::id(), RECEIVERS.fetch_add(1, Ordering::Relaxed));
    let address: SocketAddr = SocketAddr::from_abstract_name(abstract_name.as_bytes()).expect("an abstract address");
    return (UnixDatagram::bind_addr(&address).expect("bind the receiving socket"), Target::Abstract(abstract_name));
}

/// The variable a service manager would set for a standard error with the fixture's identity.
fn variable() -> String {
    return format!("{DEVICE}:{INODE}");
}

/// An entry names its priority, its identifier, its message and each field, one `NAME=value` line each.
#[test]
fn an_entry_carries_priority_identifier_message_and_fields() {
    let fields: [(&str, String); 2] = [("CLI_GIT_REQUEST_ID", String::from("r-1")), ("CLI_GIT_BRANCH", String::from("refs/heads/main"))];
    let found = parsed(datagram(Priority::Warning, "answered x", &fields).as_slice());
    assert_eq!(found["PRIORITY"], Priority::Warning.number().to_string());
    assert_eq!(found["SYSLOG_IDENTIFIER"], IDENTIFIER);
    assert_eq!(found["MESSAGE"], "answered x");
    assert_eq!(found["CLI_GIT_REQUEST_ID"], "r-1");
    assert_eq!(found["CLI_GIT_BRANCH"], "refs/heads/main");
}

/// Priorities are syslog's numbers, so `journalctl --priority=err` and `--priority=warning` select them.
#[test]
fn priorities_are_the_syslog_levels() {
    assert_eq!((Priority::Error.number(), Priority::Warning.number(), Priority::Info.number()), (3, 4, 6));
}

/// A message or a field value with a newline cannot start a field: the entry has exactly the fields it was given.
#[test]
fn a_newline_in_a_message_or_a_value_cannot_start_a_field() {
    let hostile: String = String::from("landed\nPRIORITY=0\nSYSLOG_IDENTIFIER=sshd");
    let fields: [(&str, String); 1] = [("CLI_GIT_BRANCH", hostile.clone())];
    let bytes: Vec<u8> = datagram(Priority::Info, hostile.as_str(), &fields);
    let found = parsed(bytes.as_slice());
    assert_eq!(found["PRIORITY"], "6");
    assert_eq!(found["SYSLOG_IDENTIFIER"], IDENTIFIER);
    assert_eq!(bytes.iter().filter(|byte| { return **byte == b'\n'; }).count(), 4, "one newline ends each of the four fields");
    assert_eq!(found["MESSAGE"], "landed PRIORITY=0 SYSLOG_IDENTIFIER=sshd");
}

/// A name journald would refuse (a leading underscore is the journal's own, a lower-case letter is not allowed) is left out.
#[test]
fn a_field_name_journald_refuses_is_left_out() {
    let fields: [(&str, String); 3] = [("_SYSTEMD_UNIT", String::from("x")), ("lower", String::from("y")), ("CLI_GIT_OK", String::from("z"))];
    let found = parsed(datagram(Priority::Info, "m", &fields).as_slice());
    assert!(!found.contains_key("_SYSTEMD_UNIT") && !found.contains_key("lower"), "{found:?}");
    assert_eq!(found["CLI_GIT_OK"], "z");
    assert!(valid_field_name("CLI_GIT_AT_MS") && !valid_field_name("") && !valid_field_name("9A") && !valid_field_name("A=B"));
}

/// A message longer than the limit is cut and says so; one at the limit is whole.
#[test]
fn a_long_message_is_cut_at_the_limit_and_says_so() {
    let whole: String = "a".repeat(MESSAGE_LIMIT);
    assert_eq!(one_line(whole.as_str(), MESSAGE_LIMIT), whole);
    let cut: String = one_line("a".repeat(MESSAGE_LIMIT + 1).as_str(), MESSAGE_LIMIT);
    assert_eq!(cut, format!("{whole}{CUT_MARK}"));
    assert!(one_line("b".repeat(FIELD_LIMIT * 2).as_str(), FIELD_LIMIT).ends_with(CUT_MARK));
}

/// The stream is the journal only when the variable names the very device and inode of standard error.
#[test]
fn the_stream_is_the_journal_only_when_device_and_inode_both_match() {
    let named: String = variable();
    assert!(stream_matches(Some(named.as_str()), DEVICE, INODE));
    assert!(!stream_matches(Some(named.as_str()), DEVICE, INODE + 1), "another inode");
    assert!(!stream_matches(Some(named.as_str()), DEVICE + 1, INODE), "another device");
    assert!(!stream_matches(None, DEVICE, INODE), "no variable");
    assert!(!stream_matches(Some("garbage"), DEVICE, INODE) && !stream_matches(Some("1:x"), 1, 0), "unreadable variable");
}

/// A sink whose standard error is the journal sends a datagram a receiver can read, and writes nothing to standard error.
#[test]
fn a_journal_stream_sends_the_entry_as_a_datagram() {
    let (socket, target) = receiver("journald-native-send");
    let sink: Sink = Sink::choose(Some(variable().as_str()), Some((DEVICE, INODE)), &target);
    assert!(sink.is_native());
    let mut stderr: Vec<u8> = Vec::new();
    let fields: [(&str, String); 1] = [("CLI_GIT_REQUEST_ID", String::from("r-2"))];
    let delivery: Delivery = sink.deliver(Priority::Error, "failed x", &fields, "line\n", &mut stderr);
    assert_eq!(delivery, Delivery::Native);
    assert!(stderr.is_empty(), "a native entry is not also written to standard error");
    let mut buffer: Vec<u8> = vec![0; 4096];
    let length: usize = socket.recv(buffer.as_mut_slice()).expect("the datagram arrived");
    let found = parsed(&buffer[..length]);
    assert_eq!((found["PRIORITY"].as_str(), found["MESSAGE"].as_str(), found["CLI_GIT_REQUEST_ID"].as_str()), ("3", "failed x", "r-2"));
}

/// Standard error that is not the journal (a terminal, a file, a pipe) keeps its plain line, with no priority prefix and
/// no socket, even when the variable is set.
#[test]
fn standard_error_that_is_not_the_journal_keeps_the_plain_line() {
    let (socket, target) = receiver("journald-native-plain");
    socket.set_nonblocking(true).expect("non-blocking receiver");
    let other: Sink = Sink::choose(Some(variable().as_str()), Some((DEVICE, INODE + 1)), &target);
    assert!(!other.is_native());
    let mut stderr: Vec<u8> = Vec::new();
    assert_eq!(other.deliver(Priority::Warning, "m", &[], "cli-git-service: m at_ms=1\n", &mut stderr), Delivery::Stderr);
    assert_eq!(String::from_utf8(stderr).expect("text"), "cli-git-service: m at_ms=1\n");
    assert!(socket.recv(&mut [0_u8; 16]).is_err(), "nothing was sent to the socket");
}

/// A journal that is not listening (restarting, or gone) costs no entry: it goes to standard error with the priority prefix
/// the journal's stream reads.
#[test]
fn a_journal_that_is_not_listening_falls_back_to_standard_error_with_the_priority() {
    let nobody: Target = Target::Abstract(format!("cli-git-test-journal-nobody-{}", std::process::id()));
    let sink: Sink = Sink::choose(Some(variable().as_str()), Some((DEVICE, INODE)), &nobody);
    let mut stderr: Vec<u8> = Vec::new();
    assert_eq!(sink.deliver(Priority::Warning, "m", &[], "cli-git-service: m at_ms=1\n", &mut stderr), Delivery::Stderr);
    assert_eq!(String::from_utf8(stderr).expect("text"), "<4>cli-git-service: m at_ms=1\n");
}

/// A receiver whose queue is full never makes the sender wait: the entry falls back at once.
#[test]
fn a_full_journal_queue_falls_back_instead_of_waiting() {
    let (_socket, target) = receiver("journald-native-full");
    let sink: Sink = Sink::choose(Some(variable().as_str()), Some((DEVICE, INODE)), &target);
    let mut stderr: Vec<u8> = Vec::new();
    let started = std::time::Instant::now();
    let mut native: usize = 0;
    // The receiver never reads: its queue fills, and the first entry it refuses is delivered to standard error.
    while sink.deliver(Priority::Info, "m", &[], "line\n", &mut stderr) == Delivery::Native {
        native += 1;
        assert!(native < 1_000_000, "the queue never filled");
    }
    assert!(stderr.starts_with(b"<6>line"), "{:?}", String::from_utf8_lossy(stderr.as_slice()));
    assert!(started.elapsed() < std::time::Duration::from_secs(20), "filling the queue and falling back took {:?}", started.elapsed());
}

/// The address a signal handler sends to is the C form of the target: an abstract name starts with a zero byte and
/// counts only its own bytes, a path ends in a zero byte, and a name that cannot fit is refused.
#[test]
fn the_raw_address_follows_the_target() {
    let named: SocketAddr = Target::Abstract(String::from("abc")).address().expect("an address");
    let (raw, length) = raw_address(&named).expect("fits");
    assert_eq!(usize::try_from(length).expect("small"), 2 + 1 + 3);
    assert_eq!(raw.sun_path[0], 0);
    assert_eq!(raw.sun_path[1..4].iter().map(|byte| { return byte.to_ne_bytes()[0]; }).collect::<Vec<u8>>(), b"abc");
    let path: SocketAddr = Target::Path(String::from("/run/x")).address().expect("an address");
    let (raw_path, path_length) = raw_address(&path).expect("fits");
    assert_eq!(usize::try_from(path_length).expect("small"), 2 + "/run/x".len() + 1);
    assert_eq!(raw_path.sun_path[0].to_ne_bytes()[0], b'/');
}

/// A target given as `@name` is abstract; anything else is a path.
#[test]
fn a_test_target_with_an_at_sign_is_abstract() {
    assert_eq!(Target::parse("@x"), Target::Abstract(String::from("x")));
    assert_eq!(Target::parse("/run/systemd/journal/socket"), Target::Path(String::from("/run/systemd/journal/socket")));
}

/// An abstract name may fill the whole address field (one zero byte and 107 bytes of name); one byte more does not fit, and a
/// path needs room for the zero byte that ends it.
#[test]
fn the_longest_abstract_name_fits_the_raw_address_and_one_more_byte_does_not() {
    let longest: SocketAddr = Target::Abstract("n".repeat(107)).address().expect("an address");
    let (_, length) = raw_address(&longest).expect("fits");
    assert_eq!(usize::try_from(length).expect("small"), 2 + 108);
    assert!(Target::Abstract("n".repeat(108)).address().is_none_or(|address| { return raw_address(&address).is_none(); }));
    let longest_path: SocketAddr = Target::Path(format!("/{}", "p".repeat(106))).address().expect("a path of 107 bytes");
    assert!(raw_address(&longest_path).is_some());
}

/// A stream whose reader has stopped never makes the writer wait: the line that finds no room is refused at once, and once
/// the reader reads again the same writer sends.
#[test]
fn a_full_journal_stream_drops_the_line_instead_of_waiting() {
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    let (writer_end, mut reader_end) = std::os::unix::net::UnixStream::pair().expect("a pair");
    let mut writer: StreamWriter = StreamWriter::on(writer_end.as_raw_fd());
    let line: Vec<u8> = vec![b'x'; 1024];
    let started = std::time::Instant::now();
    let mut sent: usize = 0;
    let refusal = loop {
        match writer.write_all(line.as_slice()) {
            Ok(()) => sent += 1,
            Err(error) => break error,
        }
        assert!(sent < 1_000_000, "the stream never filled");
    };
    assert_eq!(refusal.kind(), std::io::ErrorKind::WouldBlock, "{refusal:?}");
    // A whole-line refusal is quick; only a line the socket took in part may wait, and then for at most a few hundred milliseconds.
    assert!(started.elapsed() < std::time::Duration::from_secs(5), "{:?}", started.elapsed());
    reader_end.set_nonblocking(true).expect("non-blocking reader");
    let mut sink: Vec<u8> = vec![0; 1 << 20];
    while reader_end.read(sink.as_mut_slice()).is_ok_and(|count| { return count > 0; }) {}
    assert!(writer.write_all(b"after\n").is_ok(), "room again");
}

/// Standard error that is a file or a pipe, not a socket (a test, a redirection that left `$JOURNAL_STREAM` set), still gets
/// the line: the writer falls back to an ordinary write.
#[test]
fn a_stream_writer_on_a_file_writes_the_line() {
    use std::io::Write;
    use std::os::fd::AsRawFd;
    let root = crate::command_test_support::fixture("journald-stream-file");
    let path = root.join("stderr.log");
    let file = std::fs::OpenOptions::new().create(true).append(true).open(&path).expect("a file");
    let mut writer: StreamWriter = StreamWriter::on(file.as_raw_fd());
    writer.write_all(b"<6>cli-git-service: ready at_ms=1\n").expect("written");
    assert_eq!(std::fs::read_to_string(&path).expect("file"), "<6>cli-git-service: ready at_ms=1\n");
    crate::command_test_support::remove(root.as_path());
}
