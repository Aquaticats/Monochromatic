//! What:
//!  The TCP sockets of this machine that are connected,
//! with the bytes each has sent and had acknowledged and the bytes it has
//!  received,
//! and the sockets a process holds.
//! Why:
//!  `rchar` and `wchar` of `/proc/<pid>/io` count `read` and `write`,
//! not `send` and `recv`,
//! so a transfer done through `libcurl`
//!  (`git-remote-http`) moves no byte the push meter can see,
//! and `/proc/net/tcp` has queue lengths and no running count.
//! The
//!  kernel's own socket diagnostics (`NETLINK_SOCK_DIAG`,
//! `man 7 sock_diag`,
//! the interface `ss` reads) return,
//! for every
//!  socket,
//! its state,
//! its inode and a `struct tcp_info` with `tcpi_bytes_acked` and `tcpi_bytes_received`;
//! a socket a process
//!  holds is the `socket:[<inode>]` link of its `/proc/<pid>/fd` (`man 5 proc_pid_fd`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sockets = establishedSockets(); const mine = socketInodesOf(pid); sockets.filter((socket) => mine.includes(socket.inode));
//! ```

/// The state number of a connected socket,
/// `TCP_ESTABLISHED` in `include/net/tcp_states.h`.
pub const ESTABLISHED: u8 = 1;

/// One connected TCP socket as the kernel reports it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TcpSocket {
    /// The socket's inode,
    /// the number in `socket:[<inode>]`.
    pub inode: u64,
    /// The state number (`ESTABLISHED`).
    pub state: u8,
    /// Bytes sent that the peer has acknowledged (`tcpi_bytes_acked`).
    pub bytes_acked: u64,
    /// Bytes received (`tcpi_bytes_received`).
    pub bytes_received: u64,
}

/// The netlink protocol number of the socket diagnostics,
/// `NETLINK_SOCK_DIAG` (`man 7 sock_diag`,
/// `include/uapi/linux/netlink.h`).
#[cfg(target_os = "linux")]
const NETLINK_SOCK_DIAG: libc::c_int = 4;

/// The message type that asks for the sockets of one address family,
/// `SOCK_DIAG_BY_FAMILY` (`include/uapi/linux/sock_diag.h`).
#[cfg(target_os = "linux")]
const SOCK_DIAG_BY_FAMILY: u16 = 20;

/// The request flags:
/// a request (`NLM_F_REQUEST`,
/// 0x01) for the whole list (`NLM_F_DUMP`,
/// 0x300),
/// `include/uapi/linux/netlink.h`.
#[cfg(target_os = "linux")]
const REQUEST_AND_DUMP: u16 = 0x01 | 0x300;

/// The message types that end a reply:
/// `NLMSG_ERROR` and `NLMSG_DONE` (`include/uapi/linux/netlink.h`).
#[cfg(target_os = "linux")]
const NLMSG_ERROR: u16 = 2;
/// The message type that ends a dump.
#[cfg(target_os = "linux")]
const NLMSG_DONE: u16 = 3;

/// The attribute that carries a `struct tcp_info`:
/// `INET_DIAG_INFO` (`include/uapi/linux/inet_diag.h`,
/// the third of its enum).
#[cfg(target_os = "linux")]
const INET_DIAG_INFO: u16 = 2;

/// The cookie value that means "no particular socket" (`INET_DIAG_NOCOOKIE`,
/// `include/uapi/linux/inet_diag.h`).
#[cfg(target_os = "linux")]
const INET_DIAG_NOCOOKIE: u32 = u32::MAX;

/// The length of a `struct nlmsghdr`:
/// length (4),
/// type (2),
/// flags (2),
/// sequence (4),
/// port (4).
#[cfg(target_os = "linux")]
const NETLINK_HEADER: usize = 16;

/// The length of a `struct inet_diag_req_v2`:
/// family,
/// protocol,
/// extension and pad (1 each),
/// states (4),
/// and a
///  `struct inet_diag_sockid` of ports (2 + 2),
/// source and destination (16 + 16),
/// interface (4) and cookie (8).
#[cfg(target_os = "linux")]
const REQUEST_BODY: usize = 56;

/// The length of a `struct inet_diag_msg`,
/// the fixed part of each reply (`man 7 sock_diag`,
/// "IPv4 and IPv6 sockets").
#[cfg(target_os = "linux")]
const MESSAGE_BODY: usize = 72;

/// Where the state byte sits in an `inet_diag_msg` (`idiag_family` is the first byte).
#[cfg(target_os = "linux")]
const STATE_OFFSET: usize = 1;

/// Where `idiag_inode` sits in an `inet_diag_msg`:
/// after the four bytes of family,
/// state,
/// timer and retransmits,
/// the 48 of
///  the socket id,
/// and `idiag_expires`,
/// `idiag_rqueue`,
/// `idiag_wqueue` and `idiag_uid` of 4 each.
#[cfg(target_os = "linux")]
const INODE_OFFSET: usize = 68;

/// The length of an attribute header (`struct rtattr`):
/// length (2) and type (2).
#[cfg(target_os = "linux")]
const ATTRIBUTE_HEADER: usize = 4;

/// Where `tcpi_bytes_acked` sits in a `struct tcp_info` (`include/uapi/linux/tcp.h`):
/// eight one-byte fields,
/// then
///  `tcpi_rto` to `tcpi_rcv_mss` (4 x 4),
/// `tcpi_unacked` to `tcpi_fackets` (5 x 4),
/// the four `tcpi_last_*` times (4 x 4),
///  `tcpi_pmtu` to `tcpi_reordering` (8 x 4),
/// `tcpi_rcv_rtt`,
/// `tcpi_rcv_space` and `tcpi_total_retrans` (3 x 4),
/// and the two
///  eight-byte rates,
/// which puts the two eight-byte counters at 120 and 128.
/// A control checks both against bytes it counted.
#[cfg(target_os = "linux")]
const BYTES_ACKED_OFFSET: usize = 120;

/// Where `tcpi_bytes_received` sits in a `struct tcp_info`:
/// right after `tcpi_bytes_acked`.
#[cfg(target_os = "linux")]
const BYTES_RECEIVED_OFFSET: usize = BYTES_ACKED_OFFSET + 8;

/// How much of a `struct tcp_info` a reply must hold for both counters to be in it (an older kernel's is shorter).
#[cfg(target_os = "linux")]
const BYTES_END: usize = BYTES_RECEIVED_OFFSET + 8;

/// How long the kernel may take to answer one read of the diagnostics before the answer is given up (`SO_RCVTIMEO`).
#[cfg(target_os = "linux")]
const REPLY_WAIT: libc::timeval = libc::timeval { tv_sec: 2, tv_usec: 0 };

/// How many bytes one read of the reply takes:
/// a reply is cut into messages of this size or smaller.
#[cfg(target_os = "linux")]
const READ_BUFFER: usize = 64 * 1024;

/// The address families asked,
/// IPv4 first.
#[cfg(target_os = "linux")]
const FAMILIES: [libc::c_int; 2] = [libc::AF_INET, libc::AF_INET6];

/// Read four bytes at `at` as a native-endian number,
/// or nothing where the buffer ends first.
#[cfg(target_os = "linux")]
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let window: &[u8] = bytes.get(at..at + 4)?;
    return Some(u32::from_ne_bytes([window[0], window[1], window[2], window[3]]));
}

/// Read two bytes at `at` as a native-endian number,
/// or nothing where the buffer ends first.
#[cfg(target_os = "linux")]
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let window: &[u8] = bytes.get(at..at + 2)?;
    return Some(u16::from_ne_bytes([window[0], window[1]]));
}

/// Read eight bytes at `at` as a native-endian number,
/// or nothing where the buffer ends first.
#[cfg(target_os = "linux")]
fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    let window: &[u8] = bytes.get(at..at + 8)?;
    let mut array: [u8; 8] = [0; 8];
    array.copy_from_slice(window);
    return Some(u64::from_ne_bytes(array));
}

/// What:
///  The request for every connected TCP socket of one address family,
/// with the `struct tcp_info` of each.
/// Why:
///   One dump request returns all of them;
/// the extension byte asks for `INET_DIAG_INFO` (bit `INET_DIAG_INFO - 1`) and the
///  state mask for the connected state only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat([netlinkHeader({ type: SOCK_DIAG_BY_FAMILY, flags: REQUEST_AND_DUMP, sequence }), diagRequest({ family, protocol: TCP, ext: 1 << (INET_DIAG_INFO - 1), states: 1 << ESTABLISHED })])
/// ```
#[cfg(target_os = "linux")]
fn dump_request(family: libc::c_int, sequence: u32) -> Vec<u8> {
    let mut request: Vec<u8> = Vec::with_capacity(NETLINK_HEADER + REQUEST_BODY);
    request.extend_from_slice(&u32::try_from(NETLINK_HEADER + REQUEST_BODY).unwrap_or(0).to_ne_bytes());
    request.extend_from_slice(&SOCK_DIAG_BY_FAMILY.to_ne_bytes());
    request.extend_from_slice(&REQUEST_AND_DUMP.to_ne_bytes());
    request.extend_from_slice(&sequence.to_ne_bytes());
    // The port number of the sender: zero lets the kernel answer to the socket this was sent from.
    request.extend_from_slice(&0u32.to_ne_bytes());
    // `sdiag_family`, `sdiag_protocol`, `idiag_ext` and the pad byte.
    request.push(u8::try_from(family).unwrap_or(0));
    request.push(u8::try_from(libc::IPPROTO_TCP).unwrap_or(0));
    request.push(1 << (INET_DIAG_INFO - 1));
    request.push(0);
    request.extend_from_slice(&(1u32 << ESTABLISHED).to_ne_bytes());
    // The socket id is ignored in a dump: ports, addresses and interface zero, the cookie says "none".
    request.extend_from_slice(&[0; 2 + 2 + 16 + 16 + 4]);
    request.extend_from_slice(&INET_DIAG_NOCOOKIE.to_ne_bytes());
    request.extend_from_slice(&INET_DIAG_NOCOOKIE.to_ne_bytes());
    return request;
}

/// What:
///  The sockets in one block of reply,
/// and whether the block ended the dump (or failed it).
/// Why:
///   A reply is a run of netlink messages,
/// each a header,
/// an `inet_diag_msg` and attributes;
/// the dump ends with `NLMSG_DONE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { sockets, ended } = readReply(bytes);
/// ```
#[cfg(target_os = "linux")]
fn read_reply(block: &[u8], into: &mut Vec<TcpSocket>) -> Result<bool, ()> {
    let mut at: usize = 0;
    while at + NETLINK_HEADER <= block.len() {
        let length: usize = u32_at(block, at).ok_or(())? as usize;
        let kind: u16 = u16_at(block, at + 4).ok_or(())?;
        if length < NETLINK_HEADER || at + length > block.len() {
            return Err(());
        }
        if kind == NLMSG_ERROR {
            return Err(());
        }
        if kind == NLMSG_DONE {
            return Ok(true);
        }
        if kind == SOCK_DIAG_BY_FAMILY {
            let message: &[u8] = &block[at + NETLINK_HEADER..at + length];
            if let Some(socket) = read_message(message) {
                into.push(socket);
            }
        }
        // Each message starts on a four-byte boundary (`NLMSG_ALIGN`).
        at += (length + 3) & !3;
    }
    return Ok(false);
}

/// One `inet_diag_msg` and its attributes:
/// the socket's state and inode,
/// and the byte counters of its `struct tcp_info`.
#[cfg(target_os = "linux")]
fn read_message(message: &[u8]) -> Option<TcpSocket> {
    let state: u8 = *message.get(STATE_OFFSET)?;
    let inode: u64 = u64::from(u32_at(message, INODE_OFFSET)?);
    let mut counters: (u64, u64) = (0, 0);
    let mut at: usize = MESSAGE_BODY;
    while at + ATTRIBUTE_HEADER <= message.len() {
        let length: usize = u16_at(message, at)? as usize;
        let kind: u16 = u16_at(message, at + 2)?;
        if length < ATTRIBUTE_HEADER || at + length > message.len() {
            break;
        }
        if kind == INET_DIAG_INFO {
            let info: &[u8] = &message[at + ATTRIBUTE_HEADER..at + length];
            if info.len() >= BYTES_END {
                counters = (u64_at(info, BYTES_ACKED_OFFSET)?, u64_at(info, BYTES_RECEIVED_OFFSET)?);
            }
        }
        // Each attribute starts on a four-byte boundary (`RTA_ALIGN`).
        at += (length + 3) & !3;
    }
    return Some(TcpSocket { inode, state, bytes_acked: counters.0, bytes_received: counters.1 });
}

/// Ask for the sockets of one family on an open diagnostics socket and read the whole dump.
#[cfg(target_os = "linux")]
fn dump_family(descriptor: libc::c_int, family: libc::c_int, sequence: u32) -> Option<Vec<TcpSocket>> {
    let request: Vec<u8> = dump_request(family, sequence);
    // SAFETY: `send` reads `request.len()` bytes from the buffer, which lives through the call, on a descriptor this module owns.
    let sent: isize = unsafe { libc::send(descriptor, request.as_ptr().cast(), request.len(), 0) };
    if sent < 0 {
        return None;
    }
    let mut sockets: Vec<TcpSocket> = Vec::new();
    let mut buffer: Vec<u8> = vec![0; READ_BUFFER];
    loop {
        // SAFETY: `recv` writes at most `buffer.len()` bytes into the buffer, which lives through the call.
        let received: isize = unsafe { libc::recv(descriptor, buffer.as_mut_ptr().cast(), buffer.len(), 0) };
        if received <= 0 {
            return None;
        }
        let block: &[u8] = &buffer[..usize::try_from(received).ok()?];
        if read_reply(block, &mut sockets).ok()? {
            return Some(sockets);
        }
    }
}

/// Every connected TCP socket of the network namespace of this process,
/// or nothing where the kernel cannot say.
#[cfg(target_os = "linux")]
pub fn established_sockets() -> Option<Vec<TcpSocket>> {
    // SAFETY: `socket` takes plain integers and returns a descriptor or -1; the descriptor is closed below on every path.
    let descriptor: libc::c_int = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW | libc::SOCK_CLOEXEC, NETLINK_SOCK_DIAG) };
    if descriptor < 0 {
        return None;
    }
    // A copy of the constant on the stack, because the call needs an address to read from.
    let patience: libc::timeval = REPLY_WAIT;
    // SAFETY: `setsockopt` reads `size_of::<timeval>()` bytes from the value, which lives through the call.
    unsafe {
        libc::setsockopt(descriptor, libc::SOL_SOCKET, libc::SO_RCVTIMEO, (&raw const patience).cast(), std::mem::size_of::<libc::timeval>() as libc::socklen_t);
    }
    let mut all: Option<Vec<TcpSocket>> = None;
    for (sequence, family) in (1u32..).zip(FAMILIES) {
        // A family the kernel does not offer (no IPv6) makes no list; the others still answer.
        if let Some(found) = dump_family(descriptor, family, sequence) {
            all.get_or_insert_with(Vec::new).extend(found);
        }
    }
    // SAFETY: `close` ends the descriptor opened above, once.
    unsafe {
        libc::close(descriptor);
    }
    return all;
}

/// Only Linux reports sockets this way;
/// elsewhere nothing is measured.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn established_sockets() -> Option<Vec<TcpSocket>> {
    return None;
}

/// The inodes of the sockets a process holds open,
/// or nothing where its descriptors cannot be read.
/// Why nothing rather than an empty list:
///   A process that holds no socket and one whose descriptors could not be read are different answers,
/// and the push meter may not read the second as the first
/// (`read_its_descriptors` in `push_progress_signs.rs`).
#[cfg(target_os = "linux")]
pub fn socket_inodes_of(pid: i32) -> Option<Vec<u64>> {
    let mut inodes: Vec<u64> = Vec::new();
    let descriptors = std::fs::read_dir(format!("/proc/{pid}/fd")).ok()?;
    for entry in descriptors.flatten() {
        let Ok(link) = std::fs::read_link(entry.path()) else { continue };
        let text: String = link.to_string_lossy().into_owned();
        // A socket's link reads `socket:[<inode>]` (`man 5 proc_pid_fd`).
        if let Some(rest) = text.strip_prefix("socket:[")
            && let Some(number) = rest.strip_suffix(']')
            && let Ok(inode) = number.parse::<u64>()
        {
            inodes.push(inode);
        }
    }
    return Some(inodes);
}

/// Only Linux lists the sockets of a process;
/// elsewhere none is found.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn socket_inodes_of(_pid: i32) -> Option<Vec<u64>> {
    return Some(Vec::new());
}

/// Controls for what the kernel reports.
#[cfg(test)]
#[path = "tcp_sockets_tests.rs"]
mod tests;
