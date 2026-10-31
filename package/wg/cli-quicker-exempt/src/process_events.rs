//! Subscribes to the kernel's process-event connector and reports processes that just called `exec`.
//! The connector is a netlink protocol:
//! one datagram per event,
//! delivered within the exec itself.
//! The subscription needs no BPF loader and no new dependency,
//! and the watcher already holds the privilege it needs.
//! Delivery is lossy when this process falls behind,
//! so an overrun is reported and the caller rescans `/proc` as its backstop.

/// Raw descriptor ownership.
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
/// Syscall failures.
use std::io;

/// `NETLINK_CONNECTOR` from `linux/netlink.h`.
const NETLINK_CONNECTOR: libc::c_int = 11;
/// `CN_IDX_PROC` and `CN_VAL_PROC` from `linux/connector.h`.
const CN_IDX_PROC: u32 = 1;
/// `CN_VAL_PROC` from `linux/connector.h`.
const CN_VAL_PROC: u32 = 1;
/// `PROC_CN_MCAST_LISTEN` from `linux/cn_proc.h`.
const PROC_CN_MCAST_LISTEN: u32 = 1;
/// `PROC_EVENT_EXEC` from `linux/cn_proc.h`.
const PROC_EVENT_EXEC: u32 = 2;
/// `NLMSG_DONE` from `linux/netlink.h`.
const NLMSG_DONE: u16 = 3;

/// `sizeof(struct nlmsghdr)`.
const NLMSG_HEADER_LEN: usize = 16;
/// `sizeof(struct cn_msg)`.
const CN_MSG_LEN: usize = 20;
/// `offsetof(struct proc_event, event_data)`.
const PROC_EVENT_DATA_OFFSET: usize = 16;
/// `offsetof(struct proc_event, event_data.exec.process_tgid)`.
const EXEC_TGID_OFFSET: usize = 20;
/// `sizeof(struct proc_event)`.
const PROC_EVENT_LEN: usize = 40;

// The values above were printed by a C program compiled against /usr/include/linux/{netlink,connector,cn_proc}.h
// on the target kernel family: nlmsghdr 16, cn_msg 20, proc_event 40, event_data at 16, exec tgid at 20.
/// The message header and connector header are 36 bytes.
const _: () = assert!(NLMSG_HEADER_LEN + CN_MSG_LEN == 36);
/// The exec thread-group id lies inside the event.
const _: () = assert!(EXEC_TGID_OFFSET + 4 <= PROC_EVENT_LEN);
/// The exec data starts before the thread-group id.
const _: () = assert!(PROC_EVENT_DATA_OFFSET < EXEC_TGID_OFFSET);

/// Receive buffer for one datagram,
/// larger than any process event.
const DATAGRAM_BYTES: usize = 1024;

/// What one drain of the socket found.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct DrainedEvents {
    /// Thread-group ids of processes that called `exec`,
    /// without duplicates.
    pub pids: Vec<i32>,
    /// Whether the kernel dropped events because this process fell behind.
    pub overrun: bool,
}

/// Builds the datagram that subscribes the socket to process events.
pub fn build_listen_message(port_id: u32) -> Vec<u8> {
    let total = NLMSG_HEADER_LEN + CN_MSG_LEN + 4;
    let mut message: Vec<u8> = Vec::with_capacity(total);
    message.extend_from_slice(&u32::try_from(total).unwrap_or(0).to_ne_bytes());
    message.extend_from_slice(&NLMSG_DONE.to_ne_bytes());
    message.extend_from_slice(&0_u16.to_ne_bytes());
    message.extend_from_slice(&0_u32.to_ne_bytes());
    message.extend_from_slice(&port_id.to_ne_bytes());
    message.extend_from_slice(&CN_IDX_PROC.to_ne_bytes());
    message.extend_from_slice(&CN_VAL_PROC.to_ne_bytes());
    message.extend_from_slice(&0_u32.to_ne_bytes());
    message.extend_from_slice(&0_u32.to_ne_bytes());
    message.extend_from_slice(&4_u16.to_ne_bytes());
    message.extend_from_slice(&0_u16.to_ne_bytes());
    message.extend_from_slice(&PROC_CN_MCAST_LISTEN.to_ne_bytes());
    return message;
}

/// Reads four bytes at `offset` as a native-endian `u32`.
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    let array: [u8; 4] = slice.try_into().ok()?;
    return Some(u32::from_ne_bytes(array));
}

/// Extracts the exec thread-group id from one datagram,
/// or `None` for any other event or a malformed message.
pub fn parse_exec_event(datagram: &[u8]) -> Option<i32> {
    let declared = usize::try_from(read_u32(datagram, 0)?).ok()?;
    if declared < NLMSG_HEADER_LEN + CN_MSG_LEN + PROC_EVENT_LEN || declared > datagram.len() {
        return None;
    }
    if read_u32(datagram, NLMSG_HEADER_LEN)? != CN_IDX_PROC || read_u32(datagram, NLMSG_HEADER_LEN + 4)? != CN_VAL_PROC {
        return None;
    }
    let event = NLMSG_HEADER_LEN + CN_MSG_LEN;
    if read_u32(datagram, event)? != PROC_EVENT_EXEC {
        return None;
    }
    let tgid = read_u32(datagram, event + EXEC_TGID_OFFSET)?;
    return i32::try_from(tgid).ok();
}

/// Opens a nonblocking connector socket and subscribes it to process events.
pub fn subscribe() -> io::Result<OwnedFd> {
    // SAFETY: socket returns a new descriptor on success.
    let raw = unsafe {
        libc::socket(
            libc::AF_NETLINK,
            libc::SOCK_DGRAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
            NETLINK_CONNECTOR,
        )
    };
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful socket returned a descriptor owned by this function.
    let socket = unsafe { OwnedFd::from_raw_fd(raw) };
    let port_id = std::process::id();
    // SAFETY: a zeroed sockaddr_nl is a valid initial value for the fields set below.
    let mut address: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
    address.nl_family = libc::AF_NETLINK as libc::sa_family_t;
    address.nl_pid = port_id;
    address.nl_groups = CN_IDX_PROC;
    // SAFETY: the address pointer and length describe the initialized sockaddr_nl.
    let bound = unsafe {
        libc::bind(
            socket.as_raw_fd(),
            std::ptr::addr_of!(address).cast(),
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    };
    if bound < 0 {
        return Err(io::Error::last_os_error());
    }
    let message = build_listen_message(port_id);
    // SAFETY: the buffer pointer and length describe the message built above.
    let sent = unsafe { libc::send(socket.as_raw_fd(), message.as_ptr().cast(), message.len(), 0) };
    if sent < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(socket);
}

/// One source of datagrams.
pub trait DatagramSource {
    /// Fills the buffer and returns the byte count with the netlink port that sent it,
    /// or the error of the receive call.
    fn receive(&mut self, buffer: &mut [u8]) -> io::Result<(usize, u32)>;
}

/// The connector socket as a source.
pub struct SocketSource<'a> {
    /// The subscribed socket.
    pub socket: &'a OwnedFd,
}

/// Reads from the socket without blocking.
impl DatagramSource for SocketSource<'_> {
    /// Receives one datagram from the socket.
    fn receive(&mut self, buffer: &mut [u8]) -> io::Result<(usize, u32)> {
        return receive_datagram(self.socket, buffer);
    }
}

/// Receives one datagram from the socket without blocking,
/// returning its length and the sender's netlink port.
pub fn receive_datagram(socket: &OwnedFd, buffer: &mut [u8]) -> io::Result<(usize, u32)> {
    // SAFETY: a zeroed sockaddr_nl is valid output storage.
    let mut sender: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
    let mut sender_len = std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t;
    // SAFETY: the buffer and sender pointers are valid for the lengths passed.
    let count = unsafe {
        libc::recvfrom(
            socket.as_raw_fd(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            0,
            std::ptr::addr_of_mut!(sender).cast(),
            &mut sender_len,
        )
    };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok((usize::try_from(count).unwrap_or(0), sender.nl_pid));
}

/// Collects the exec events from `source` until it reports that nothing is queued.
/// Only datagrams from netlink port 0,
/// the kernel,
/// count,
/// because a local process can unicast forged datagrams to this socket's port.
/// An `ENOBUFS` error means the kernel dropped events,
/// which sets `overrun` so the caller rescans.
pub fn drain_exec_events_from(source: &mut dyn DatagramSource) -> io::Result<DrainedEvents> {
    let mut drained = DrainedEvents::default();
    let mut buffer = [0_u8; DATAGRAM_BYTES];
    loop {
        let (length, sender_port) = match source.receive(&mut buffer) {
            Ok(received) => received,
            Err(error) => match error.raw_os_error() {
                Some(libc::EAGAIN) => return Ok(drained),
                Some(libc::ENOBUFS) => {
                    drained.overrun = true;
                    continue;
                }
                Some(libc::EINTR) => continue,
                _ => return Err(error),
            },
        };
        if sender_port != 0 {
            continue;
        }
        if let Some(pid) = parse_exec_event(&buffer[..length.min(DATAGRAM_BYTES)])
            && !drained.pids.contains(&pid)
        {
            drained.pids.push(pid);
        }
    }
}

/// Reads every queued datagram of the connector socket and collects the exec events sent by the kernel.
pub fn drain_exec_events(socket: &OwnedFd) -> io::Result<DrainedEvents> {
    return drain_exec_events_from(&mut SocketSource { socket });
}
