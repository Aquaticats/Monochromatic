//! Verifies the process-event connector datagram layout on byte fixtures,
//! and the live subscription in an ignored test.

/// Parser and subscription under test.
use crate::process_events::{
    build_listen_message,
    drain_exec_events,
    drain_exec_events_from,
    parse_exec_event,
    receive_datagram,
    subscribe,
    DatagramSource,
};
/// Raw descriptors of the user-space netlink sockets.
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

/// Builds a kernel-style process-event datagram for `what` with `tgid` in the exec position.
fn event_datagram(what: u32, tgid: u32, idx: u32) -> Vec<u8> {
    let mut datagram: Vec<u8> = Vec::new();
    datagram.extend_from_slice(&76_u32.to_ne_bytes());
    datagram.extend_from_slice(&3_u16.to_ne_bytes());
    datagram.extend_from_slice(&0_u16.to_ne_bytes());
    datagram.extend_from_slice(&0_u32.to_ne_bytes());
    datagram.extend_from_slice(&0_u32.to_ne_bytes());
    datagram.extend_from_slice(&idx.to_ne_bytes());
    datagram.extend_from_slice(&1_u32.to_ne_bytes());
    datagram.extend_from_slice(&0_u32.to_ne_bytes());
    datagram.extend_from_slice(&0_u32.to_ne_bytes());
    datagram.extend_from_slice(&40_u16.to_ne_bytes());
    datagram.extend_from_slice(&0_u16.to_ne_bytes());
    datagram.extend_from_slice(&what.to_ne_bytes());
    datagram.extend_from_slice(&0_u32.to_ne_bytes());
    datagram.extend_from_slice(&0_u64.to_ne_bytes());
    datagram.extend_from_slice(&tgid.to_ne_bytes());
    datagram.extend_from_slice(&tgid.to_ne_bytes());
    datagram.resize(76, 0);
    return datagram;
}

/// Reads the exec thread-group id from a well-formed exec event.
#[test]
fn an_exec_event_yields_its_thread_group_id() {
    // Offsets match the C layout printed from the kernel headers: exec data at 16, tgid at 20.
    let mut datagram = event_datagram(2, 4242, 1);
    assert_eq!(datagram.len(), 76);
    assert_eq!(parse_exec_event(&datagram), Some(4242));
    datagram.truncate(75);
    assert_eq!(parse_exec_event(&datagram), None);
}

/// Ignores events that are not exec,
/// messages for another connector,
/// and truncated input.
#[test]
fn other_events_and_malformed_input_are_ignored() {
    assert_eq!(parse_exec_event(&event_datagram(1, 7, 1)), None);
    assert_eq!(parse_exec_event(&event_datagram(2, 7, 2)), None);
    assert_eq!(parse_exec_event(&[]), None);
    assert_eq!(parse_exec_event(&[0; 8]), None);
    let mut lying = event_datagram(2, 7, 1);
    lying[0..4].copy_from_slice(&400_u32.to_ne_bytes());
    assert_eq!(parse_exec_event(&lying), None);
}

/// The subscription message carries the port id,
/// the process connector ids and the listen operation.
#[test]
fn the_listen_message_has_the_documented_layout() {
    let message = build_listen_message(99);
    assert_eq!(message.len(), 40);
    assert_eq!(u32::from_ne_bytes(message[0..4].try_into().expect("length field")), 40);
    assert_eq!(u16::from_ne_bytes(message[4..6].try_into().expect("type field")), 3);
    assert_eq!(u32::from_ne_bytes(message[12..16].try_into().expect("port field")), 99);
    assert_eq!(u32::from_ne_bytes(message[16..20].try_into().expect("idx field")), 1);
    assert_eq!(u32::from_ne_bytes(message[20..24].try_into().expect("val field")), 1);
    assert_eq!(u16::from_ne_bytes(message[32..34].try_into().expect("payload length")), 4);
    assert_eq!(u32::from_ne_bytes(message[36..40].try_into().expect("operation")), 1);
}

/// Scripted receive results in order,
/// and the end of the queue reported as `EAGAIN`.
struct Scripted {
    /// Results still to hand out.
    queue: std::vec::IntoIter<std::io::Result<(Vec<u8>, u32)>>,
}

/// The scripted source.
impl DatagramSource for Scripted {
    /// Hands out the next scripted result.
    fn receive(&mut self, buffer: &mut [u8]) -> std::io::Result<(usize, u32)> {
        return match self.queue.next() {
            Some(Ok((bytes, port))) => {
                buffer[..bytes.len()].copy_from_slice(&bytes);
                Ok((bytes.len(), port))
            }
            Some(Err(error)) => Err(error),
            None => Err(std::io::Error::from_raw_os_error(libc::EAGAIN)),
        };
    }
}

/// Builds a scripted source from results.
fn scripted(results: Vec<std::io::Result<(Vec<u8>, u32)>>) -> Scripted {
    return Scripted { queue: results.into_iter() };
}

/// Only datagrams from netlink port 0 count,
/// and the same bytes from any other port are dropped.
#[test]
fn only_the_kernel_port_may_report_an_exec() {
    let exec = event_datagram(2, 4242, 1);
    let mut kernel = scripted(vec![Ok((exec.clone(), 0))]);
    assert_eq!(drain_exec_events_from(&mut kernel).expect("drain").pids, vec![4242]);
    let mut forged = scripted(vec![Ok((exec.clone(), 1)), Ok((exec, 31_337))]);
    let drained = drain_exec_events_from(&mut forged).expect("drain");
    assert!(drained.pids.is_empty());
    assert!(!drained.overrun);
}

/// The kernel reports lost events as `ENOBUFS`,
/// which sets the overrun flag and keeps draining.
#[test]
fn a_dropped_event_report_sets_the_overrun_flag_and_draining_continues() {
    let mut source = scripted(vec![
        Err(std::io::Error::from_raw_os_error(libc::ENOBUFS)),
        Err(std::io::Error::from_raw_os_error(libc::EINTR)),
        Ok((event_datagram(2, 7, 1), 0)),
    ]);
    let drained = drain_exec_events_from(&mut source).expect("drain");
    assert!(drained.overrun);
    assert_eq!(drained.pids, vec![7]);
    let mut quiet = scripted(vec![Ok((event_datagram(2, 8, 1), 0)), Ok((event_datagram(2, 8, 1), 0))]);
    let quiet_drained = drain_exec_events_from(&mut quiet).expect("drain");
    assert!(!quiet_drained.overrun);
    assert_eq!(quiet_drained.pids, vec![8]);
}

/// Any other receive error is returned and not swallowed.
#[test]
fn an_unexpected_receive_error_is_returned() {
    let mut source = scripted(vec![Err(std::io::Error::from_raw_os_error(libc::EBADF))]);
    let error = drain_exec_events_from(&mut source).expect_err("a bad descriptor");
    assert_eq!(error.raw_os_error(), Some(libc::EBADF));
}

/// Opens a user-space netlink socket bound to an automatic port,
/// and returns it with that port.
fn user_netlink_socket() -> (OwnedFd, u32) {
    // SAFETY: socket returns a new descriptor on success.
    let raw = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, libc::NETLINK_USERSOCK) };
    assert!(raw >= 0, "{}", std::io::Error::last_os_error());
    // SAFETY: successful socket returned a descriptor owned here.
    let socket = unsafe { OwnedFd::from_raw_fd(raw) };
    // SAFETY: a zeroed sockaddr_nl is a valid initial value.
    let mut address: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
    address.nl_family = libc::AF_NETLINK as libc::sa_family_t;
    let mut length = std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t;
    // SAFETY: the pointer and length describe the initialized address.
    let bound = unsafe { libc::bind(socket.as_raw_fd(), std::ptr::addr_of!(address).cast(), length) };
    assert_eq!(bound, 0, "{}", std::io::Error::last_os_error());
    // SAFETY: the pointer and length describe writable address storage.
    let named = unsafe { libc::getsockname(socket.as_raw_fd(), std::ptr::addr_of_mut!(address).cast(), &mut length) };
    assert_eq!(named, 0);
    return (socket, address.nl_pid);
}

/// A real socket receiving a valid exec event from another local socket drops it,
/// because the sender port is not the kernel's.
#[test]
fn a_local_process_cannot_forge_an_exec_event_over_a_real_socket() {
    let (receiver, receiver_port) = user_netlink_socket();
    let (sender, sender_port) = user_netlink_socket();
    assert_ne!(sender_port, 0);
    let datagram = event_datagram(2, 4242, 1);
    // SAFETY: a zeroed sockaddr_nl is a valid initial value.
    let mut target: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
    target.nl_family = libc::AF_NETLINK as libc::sa_family_t;
    target.nl_pid = receiver_port;
    // SAFETY: the buffer and the target address are valid for the lengths passed.
    let sent = unsafe {
        libc::sendto(
            sender.as_raw_fd(),
            datagram.as_ptr().cast(),
            datagram.len(),
            0,
            std::ptr::addr_of!(target).cast(),
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    };
    assert_eq!(usize::try_from(sent).expect("sent"), datagram.len());
    let mut buffer = [0_u8; 1024];
    let (length, port) = receive_datagram(&receiver, &mut buffer).expect("the datagram arrives");
    assert_eq!(length, datagram.len());
    assert_eq!(port, sender_port);
    assert_eq!(parse_exec_event(&buffer[..length]), Some(4242));
    // The same datagram through the draining path is dropped.
    // SAFETY: the buffer and the target address are valid for the lengths passed.
    let resent = unsafe {
        libc::sendto(
            sender.as_raw_fd(),
            datagram.as_ptr().cast(),
            datagram.len(),
            0,
            std::ptr::addr_of!(target).cast(),
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    };
    assert_eq!(usize::try_from(resent).expect("sent"), datagram.len());
    let drained = drain_exec_events(&receiver).expect("drain");
    assert!(drained.pids.is_empty());
}

/// Receives the exec event of a child this test starts.
/// It needs the kernel connector,
/// which this kernel allows without privilege and a container may not.
#[test]
#[ignore = "needs the kernel process-event connector"]
fn a_live_subscription_reports_the_exec_of_a_child() {
    let socket = subscribe().expect("subscribe to process events");
    let mut child = std::process::Command::new("/usr/bin/true").spawn().expect("start the child");
    let pid = i32::try_from(child.id()).expect("pid fits");
    child.wait().expect("child exits");
    let mut seen = false;
    for _ in 0..50 {
        let drained = drain_exec_events(&socket).expect("drain");
        if drained.pids.contains(&pid) {
            seen = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(seen, "no exec event for {pid}");
}
