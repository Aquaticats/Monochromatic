//! Controls for the kernel's socket report on real loopback connections made by the test itself:
//! the state of a connected pair,
//!  the bytes each side has sent and had acknowledged and has received,
//! a peer that closed,
//! and the sockets a process holds.
//! Why:
//!  The push meter takes a socket's bytes as progress and an open connection as a reason to wait,
//! so the numbers must be exact
//!  and the state must be right;
//! each is checked against bytes the test counted itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! client.write(bytes); await server.read(bytes.length); expect(socketOf(client).bytesAcked).toBe(bytes.length);
//! ```

use super::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

/// Bytes the client sends and the server reads.
const CLIENT_BYTES: usize = 3 * 1024;

/// Bytes the server sends and the client reads,
/// a different number so the two directions cannot be mixed up.
const SERVER_BYTES: usize = 1024;

/// How long a control waits for the kernel to catch up (an acknowledgement,
/// a close),
/// so a broken report fails and never holds.
const PATIENCE: Duration = Duration::from_secs(20);

/// The pause between two looks at the report.
const LOOK: Duration = Duration::from_millis(20);

/// The inode of a socket this process holds,
/// from the link of its descriptor.
fn inode_of(stream: &TcpStream) -> u64 {
    let link: std::path::PathBuf = std::fs::read_link(format!("/proc/self/fd/{}", stream.as_raw_fd())).expect("the descriptor link");
    let text: String = link.to_string_lossy().into_owned();
    let number: &str = text.strip_prefix("socket:[").and_then(|rest| { return rest.strip_suffix(']'); }).expect("a socket link");
    return number.parse::<u64>().expect("an inode");
}

/// The report for one inode,
/// if the kernel lists it.
fn report_of(inode: u64) -> Option<TcpSocket> {
    return established_sockets().expect("the kernel answers").into_iter().find(|socket| { return socket.inode == inode; });
}

/// A connected pair on the loopback,
/// client first.
fn pair() -> (TcpStream, TcpStream) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let client: TcpStream = TcpStream::connect(listener.local_addr().expect("its address")).expect("connect");
    let (server, _peer) = listener.accept().expect("accept");
    return (client, server);
}

/// A connected pair is reported established on both sides,
/// and the bytes each side sent and had acknowledged and received grow
///  by exactly the bytes the test moved.
/// The counts are compared as growth from the connected pair,
/// because the kernel counts
///  the opening side's `SYN` as an acknowledged byte (measured on 2026-11-03:
/// 3073 for 3072 bytes on the client,
/// 1024 for 1024 on
///  the server),
/// and growth is what the push meter uses.
#[test]
fn a_connected_pair_reports_its_state_and_the_bytes_of_both_directions() {
    let (mut client, mut server) = pair();
    let (client_inode, server_inode): (u64, u64) = (inode_of(&client), inode_of(&server));
    let before_client: TcpSocket = report_of(client_inode).expect("the client socket is listed");
    let before_server: TcpSocket = report_of(server_inode).expect("the server socket is listed");
    client.write_all(&[1; CLIENT_BYTES]).expect("client sends");
    server.read_exact(&mut [0; CLIENT_BYTES]).expect("server reads");
    server.write_all(&[2; SERVER_BYTES]).expect("server sends");
    client.read_exact(&mut [0; SERVER_BYTES]).expect("client reads");
    let grown = |now: &TcpSocket, before: &TcpSocket| -> (u64, u64) {
        return (now.bytes_acked - before.bytes_acked, now.bytes_received - before.bytes_received);
    };
    let began: Instant = Instant::now();
    loop {
        let on_client: Option<TcpSocket> = report_of(client_inode);
        let on_server: Option<TcpSocket> = report_of(server_inode);
        let settled: bool = on_client.is_some_and(|socket| { return grown(&socket, &before_client) == (CLIENT_BYTES as u64, SERVER_BYTES as u64); })
            && on_server.is_some_and(|socket| { return grown(&socket, &before_server) == (SERVER_BYTES as u64, CLIENT_BYTES as u64); });
        if settled {
            assert_eq!((on_client.expect("listed").state, on_server.expect("listed").state), (ESTABLISHED, ESTABLISHED));
            return;
        }
        assert!(
            began.elapsed() < PATIENCE,
            "the counts never grew by the bytes moved: client {on_client:?} from {before_client:?}, server {on_server:?} from {before_server:?}"
        );
        std::thread::sleep(LOOK);
    }
}

/// A socket whose peer closed is no longer established,
/// so it is not listed.
#[test]
fn a_socket_whose_peer_closed_is_not_listed_as_connected() {
    let (client, server) = pair();
    let server_inode: u64 = inode_of(&server);
    assert!(report_of(server_inode).is_some(), "a connected socket is listed");
    drop(client);
    let began: Instant = Instant::now();
    while report_of(server_inode).is_some() {
        assert!(began.elapsed() < PATIENCE, "a socket in CLOSE_WAIT is still listed as connected");
        std::thread::sleep(LOOK);
    }
}

/// The sockets a process holds are the inodes of its socket links,
/// and a process that does not exist answers that its descriptors cannot be read,
/// which is not the same answer as holding no socket.
#[test]
fn the_sockets_of_a_process_are_the_inodes_of_its_socket_links() {
    let (client, server) = pair();
    let mine: Vec<u64> = socket_inodes_of(i32::try_from(std::process::id()).expect("a pid")).expect("this process's descriptors");
    assert!(mine.contains(&inode_of(&client)) && mine.contains(&inode_of(&server)), "{mine:?}");
    assert_eq!(socket_inodes_of(i32::MAX), None);
    // The positive control for the other answer: a live process whose descriptors were read answers a list, whatever it holds.
    let mut command: std::process::Command = std::process::Command::new("sleep");
    command.arg("30");
    let mut sleeper: std::process::Child = crate::test_support::spawn_owned(&mut command).expect("a sleeper");
    let read: Option<Vec<u64>> = socket_inodes_of(i32::try_from(sleeper.id()).expect("pid"));
    let _killed = sleeper.kill();
    let _reaped = sleeper.wait();
    assert!(read.is_some(), "the descriptors of a live process are read");
}
