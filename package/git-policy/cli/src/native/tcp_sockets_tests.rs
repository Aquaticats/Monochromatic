//! Controls for the kernel's socket report on real loopback connections made by the test itself: the state of a connected pair,
//!  the bytes each side has sent and had acknowledged and has received, a peer that closed, and the sockets a process holds.
//! Why:
//!  The push meter takes a socket's bytes as progress and an open connection as a reason to wait, so the numbers must be exact
//!  and the state must be right; each is checked against bytes the test counted itself.
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

/// Bytes the server sends and the client reads, a different number so the two directions cannot be mixed up.
const SERVER_BYTES: usize = 1024;

/// How long a control waits for the kernel to catch up (an acknowledgement, a close), so a broken report fails and never holds.
const PATIENCE: Duration = Duration::from_secs(20);

/// The pause between two looks at the report.
const LOOK: Duration = Duration::from_millis(20);

/// The inode of a socket this process holds, from the link of its descriptor.
fn inode_of(stream: &TcpStream) -> u64 {
    let link: std::path::PathBuf = std::fs::read_link(format!("/proc/self/fd/{}", stream.as_raw_fd())).expect("the descriptor link");
    let text: String = link.to_string_lossy().into_owned();
    let number: &str = text.strip_prefix("socket:[").and_then(|rest| { return rest.strip_suffix(']'); }).expect("a socket link");
    return number.parse::<u64>().expect("an inode");
}

/// The report for one inode, if the kernel lists it.
fn report_of(inode: u64) -> Option<TcpSocket> {
    return established_sockets().expect("the kernel answers").into_iter().find(|socket| { return socket.inode == inode; });
}

/// A connected pair on the loopback, client first.
fn pair() -> (TcpStream, TcpStream) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let client: TcpStream = TcpStream::connect(listener.local_addr().expect("its address")).expect("connect");
    let (server, _peer) = listener.accept().expect("accept");
    return (client, server);
}

/// A connected pair is reported established on both sides, with the bytes each side sent and had acknowledged and received.
#[test]
fn a_connected_pair_reports_its_state_and_the_bytes_of_both_directions() {
    let (mut client, mut server) = pair();
    client.write_all(&[1; CLIENT_BYTES]).expect("client sends");
    server.read_exact(&mut [0; CLIENT_BYTES]).expect("server reads");
    server.write_all(&[2; SERVER_BYTES]).expect("server sends");
    client.read_exact(&mut [0; SERVER_BYTES]).expect("client reads");
    let (client_inode, server_inode): (u64, u64) = (inode_of(&client), inode_of(&server));
    let began: Instant = Instant::now();
    loop {
        let settled: bool = report_of(client_inode).is_some_and(|socket| { return socket.bytes_acked == CLIENT_BYTES as u64; })
            && report_of(server_inode).is_some_and(|socket| { return socket.bytes_acked == SERVER_BYTES as u64; });
        if settled {
            break;
        }
        assert!(began.elapsed() < PATIENCE, "the acknowledgements never reached the report");
        std::thread::sleep(LOOK);
    }
    let on_client: TcpSocket = report_of(client_inode).expect("the client socket is listed");
    let on_server: TcpSocket = report_of(server_inode).expect("the server socket is listed");
    assert_eq!((on_client.state, on_server.state), (ESTABLISHED, ESTABLISHED));
    assert_eq!((on_client.bytes_acked, on_client.bytes_received), (CLIENT_BYTES as u64, SERVER_BYTES as u64));
    assert_eq!((on_server.bytes_acked, on_server.bytes_received), (SERVER_BYTES as u64, CLIENT_BYTES as u64));
}

/// A socket whose peer closed is no longer established, so it is not listed.
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

/// The sockets a process holds are the inodes of its socket links, and a process that does not exist holds none.
#[test]
fn the_sockets_of_a_process_are_the_inodes_of_its_socket_links() {
    let (client, server) = pair();
    let mine: Vec<u64> = socket_inodes_of(i32::try_from(std::process::id()).expect("a pid"));
    assert!(mine.contains(&inode_of(&client)) && mine.contains(&inode_of(&server)), "{mine:?}");
    assert_eq!(socket_inodes_of(i32::MAX), Vec::<u64>::new());
}
