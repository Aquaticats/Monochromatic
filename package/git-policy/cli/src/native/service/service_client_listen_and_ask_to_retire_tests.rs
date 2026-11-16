//! What:
//!  Controls for what one exchange with the service costs and what it learns about the process on the other end:
//! the write of a request, whose wait is on the bytes a service takes and not on a clock,
//! the read of one line, whose wait is the bound the connection carries,
//! and the account the kernel names for a connection.
//! Why:
//!  A request of one mebibyte fills both buffers of a local connection (233,152 bytes measured here), so a write of it
//!  waits in the kernel until the service reads, and a service that stopped reading never does:
//! the audit of release 0.6.1-alpha.12 found a commit there with no bound at all.
//! A line a service sends a byte at a time used to restart the operating system's read timeout with every byte, so one
//! `read_line` could outlive every bound of the client.
//! A socket file is protected by the directory it stands in, and an abstract name has no file and no permission bits,
//! so the kernel's word about the peer is what keeps another account out of a service that answers at one
//! (`service_address_of_a_worktree.rs`).
//! Another account's process cannot be started from a control, which runs as the account that started the test suite,
//! so the comparison is held at both answers and the read is proven to give this account's own id for a live connection.

use super::*;
use crate::service_protocol_encode_messages_to_lines::encode_service;

/// How long a control waits for anything a healthy step does in microseconds,
/// so a bound that is lost fails the control instead of holding it.
const PATIENCE: Duration = Duration::from_secs(30);

/// A connected pair,
/// which is one write and one read of a local connection without an address.
fn a_pair() -> (UnixStream, UnixStream) {
    return UnixStream::pair().expect("a connected pair");
}

/// What:
///  A write whose other end reads goes in whole and at once.
/// Why:
///  This is the state every healthy commit is in,
/// and the bound must cost it nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(writeWithin(client, payload, 5_000, 10_000)).toBe('all')
/// ```
#[test]
fn a_write_the_service_reads_goes_in_whole() {
    let (stream, mut service) = a_pair();
    let payload: Vec<u8> = vec![b'x'; 400 * 1024];
    let expected: usize = payload.len();
    let reading = std::thread::spawn(move || {
        let mut taken: Vec<u8> = vec![0; expected];
        let mut read: usize = 0;
        while read < expected {
            let took: usize = std::io::Read::read(&mut service, &mut taken[read..]).expect("the service reads");
            if took == 0 {
                break;
            }
            read += took;
        }
        return read;
    });
    let began: Instant = Instant::now();
    let wrote: Wrote = write_within(&stream, payload.as_slice(), Duration::from_secs(5), Duration::from_secs(10));
    let took: Duration = began.elapsed();
    assert_eq!(wrote, Wrote::All, "a write a reader drains goes in whole");
    assert_eq!(reading.join().expect("the reader ends"), expected, "the service took every byte");
    assert!(took < Duration::from_secs(5), "and it costs the silence bound nothing: {took:?}");
}

/// What:
///  A write whose other end reads nothing ends at the silence bound rather than waiting for ever.
/// Why:
///   This is the case the audit found:
/// a request of 652,332 bytes,
/// a service stopped with `SIGSTOP`,
/// and a wrapper in `sendto` that had made no progress after twelve seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(writeWithin(client, 400_000, 100, 5_000)).toBe('nothing-taken')
/// ```
#[test]
fn a_write_nothing_reads_ends_at_its_silence_bound() {
    let (stream, service) = a_pair();
    let payload: Vec<u8> = vec![b'x'; 400 * 1024];
    let began: Instant = Instant::now();
    let wrote: Wrote = write_within(&stream, payload.as_slice(), Duration::from_millis(100), Duration::from_secs(5));
    let took: Duration = began.elapsed();
    drop(service);
    assert_eq!(wrote, Wrote::NothingTaken, "a write nothing reads ends at its silence bound");
    assert!(took >= Duration::from_millis(100), "the bound is what it was given: {took:?}");
    assert!(took < PATIENCE, "and the wait ends: {took:?}");
}

/// What:
///  A write to a connection the other end closed ends instead of waiting.
/// Why:
///  A closed connection is a service that went away,
/// which is not the same as one that stopped reading,
/// and the two
///  are sent on differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// service.destroy(); expect(writeWithin(client, payload, 5_000, 10_000)).toBe('ended')
/// ```
#[test]
fn a_write_to_a_closed_connection_ends() {
    let (stream, service) = a_pair();
    drop(service);
    let wrote: Wrote = write_within(&stream, vec![b'x'; 1024].as_slice(), Duration::from_secs(5), Duration::from_secs(5));
    assert_eq!(wrote, Wrote::Ended, "a connection the service closed is a send that ended");
}

/// What:
///  A write whose other end takes bytes but takes them too slowly ends at the ceiling.
/// Why:
///   The silence bound is what tells a service that stopped reading from one that reads,
/// so a service that reads a little is waited for;
/// what ends that wait is the request's own ceiling,
/// which is what keeps the whole exchange bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writes = setInterval(() => service.read(1024), 20); expect(writeWithin(client, 400_000, 5_000, 200)).toBe('too-slow')
/// ```
#[test]
fn a_write_a_reader_takes_too_slowly_ends_at_its_ceiling() {
    let (stream, mut service) = a_pair();
    let payload: Vec<u8> = vec![b'x'; 400 * 1024];
    let service_patience: Duration = PATIENCE;
    let reading = std::thread::spawn(move || {
        let began: Instant = Instant::now();
        let mut taken: usize = 0;
        while began.elapsed() < service_patience {
            let mut one: [u8; 1024] = [0; 1024];
            match std::io::Read::read(&mut service, &mut one) {
                Ok(0) => break,
                Ok(read) => taken += read,
                Err(_ended) => break,
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        return taken;
    });
    let began: Instant = Instant::now();
    let wrote: Wrote = write_within(&stream, payload.as_slice(), Duration::from_secs(5), Duration::from_millis(200));
    let took: Duration = began.elapsed();
    drop(stream);
    let _taken: usize = reading.join().expect("the reader ends");
    assert_eq!(wrote, Wrote::TooSlow, "a write the service takes too slowly ends at its ceiling");
    assert!(took >= Duration::from_millis(200), "the ceiling is what it was given: {took:?}");
    assert!(took < PATIENCE, "and the wait ends: {took:?}");
}

/// What:
///  A line a service sends a byte at a time and never ends is read as silence at the bound the connection carries.
/// Why:
///   One `read_line` restarted the operating system's read timeout with every byte it took,
/// so a service that dribbled
///  bytes held the client inside one call with no bound of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(listen(reader)).toBe('silent') // within the 100 ms the connection carries
/// ```
#[test]
fn a_line_that_never_ends_is_read_as_silence_at_the_bound() {
    let (client, mut service) = a_pair();
    client.set_read_timeout(Some(Duration::from_millis(100))).expect("a read timeout");
    let dribbling = std::thread::spawn(move || {
        let began: Instant = Instant::now();
        while began.elapsed() < PATIENCE {
            if std::io::Write::write_all(&mut service, b"x").is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    });
    let mut reader: BufReader<UnixStream> = BufReader::new(client);
    let began: Instant = Instant::now();
    let heard: Heard = listen(&mut reader);
    let took: Duration = began.elapsed();
    assert!(matches!(heard, Heard::Silent), "a line that never ends is silence at the bound");
    assert!(took >= Duration::from_millis(100), "the bound is the one the connection carries: {took:?}");
    assert!(took < PATIENCE, "and the read ends: {took:?}");
    drop(dribbling);
}

/// What:
///  A bound a line used does not shorten the bound the next line gets.
/// Why:
///   The bound is the connection's own read timeout,
/// and a message that came quickly would otherwise leave every later line a little
///  less time than the one before it,
/// until a service that is working is read as one that stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listen(reader); expect(reader.timeout).toBe(400)
/// ```
#[test]
fn a_line_leaves_the_bound_the_connection_carries_as_it_was() {
    let (client, mut service) = a_pair();
    let given: Duration = Duration::from_millis(400);
    client.set_read_timeout(Some(given)).expect("a read timeout");
    let mut reader: BufReader<UnixStream> = BufReader::new(client);
    for _ in 0..8 {
        let mut line: String = encode_service(&ServiceMessage::Heartbeat);
        line.push('\n');
        std::io::Write::write_all(&mut service, line.as_bytes()).expect("a message");
        let heard: Heard = listen(&mut reader);
        assert!(matches!(heard, Heard::Message(ServiceMessage::Heartbeat)), "each line is read as its message");
        assert_eq!(
            reader.get_ref().read_timeout().expect("the timeout is read").expect("a timeout is set"),
            given,
            "the bound the connection carries is unchanged"
        );
    }
}

/// What:
///  The comparison accepts this account's own id and refuses any other.
/// Why:
///  This is the whole of what stands in for a directory's mode on an address that has none,
/// and both answers have to be right:
///  a comparison that refused another id would let a foreign process be taken for the service,
///  and one that refused the account's own would fail every commit under a long runtime directory.
#[test]
fn a_connection_of_this_account_is_accepted_and_another_is_not() {
    let own: u32 = own_users_id();
    assert!(a_connection_of_the_account(Some(own), own), "this account's own connection");
    assert!(!a_connection_of_the_account(Some(own + 1), own), "a connection another account's process took");
    assert!(!a_connection_of_the_account(Some(0), own), "a connection of the root account");
    assert!(!a_connection_of_the_account(None, own), "a connection the kernel would not speak about");
}

/// What:
///  A directory under the short scratch folder of this test process,
///  short enough for a socket name.
/// Why:
///   A socket address carries 107 bytes of path,
/// and a fixture root under the build directory is far longer,
///  so a control of a socket stands where the package's own fixtures keep their runtime directories.
fn short_scratch(tag: &str) -> std::path::PathBuf {
    let directory: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory"))
        .join("temp/agent")
        .join(format!("cu{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    crate::test_support::track_for_this_thread(directory.as_path());
    return directory;
}

/// What:
///  The kernel reports this account's id for a connection this account's own process took,
///  and `connect` says the connection is ours.
/// Why:
///  The comparison above is only as good as the read behind it,
///  and a read that answered nothing would refuse every service this program starts.
#[test]
fn the_kernel_reports_this_account_for_a_connection_of_this_account() {
    let socket: std::path::PathBuf = short_scratch("account").join("s.sock");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(&socket).expect("a listening socket");
    let Reached::Ours(stream) = reach(&Bind::Path(socket)) else {
        panic!("a connection this account's own listener took is ours")
    };
    assert_eq!(the_users_id_on_the_other_end(&stream), Some(own_users_id()), "the kernel named this account");
    assert!(connection_is_this_account(&stream), "and the comparison accepts it");
    drop(listener);
}

/// What:
///  Nothing listens at an address whose socket file is not there,
///  and nothing listens at an abstract name nobody bound.
/// Why:
///  Every wait of a commit starts from what a connection came to,
/// and an address nothing answers must read as that and not as another account's.
#[test]
fn a_connection_to_nothing_is_nothing() {
    assert!(!matches!(reach(&Bind::Path(short_scratch("nothing").join("no-service-here.sock"))), Reached::Ours(_)), "no socket file, nothing");
    assert!(
        matches!(reach(&Bind::Abstract(String::from("monochromatic-git-policy-cli/nobody/listens/here"))), Reached::Nothing),
        "no such name, nothing"
    );
}
