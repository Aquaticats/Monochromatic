//! Verifies the process-event connector datagram layout on byte fixtures,
//! and the live subscription in an ignored test.

/// Parser and subscription under test.
use crate::process_events::{
    build_exec_filter_message,
    build_ignore_message,
    unsubscribe,
    build_listen_message,
    enlarge_receive_buffer,
    subscribe_to_everything,
    drain_exec_events,
    drain_exec_events_from,
    parse_exec_event,
    receive_datagram,
    subscribe,
    DatagramSource,
    MAX_DATAGRAMS_PER_DRAIN,
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

/// The filter message has the layout of `struct proc_input`:
/// the same headers as the plain listen message,
/// a payload of eight bytes,
/// the listen operation and the exec event mask.
#[test]
fn the_exec_filter_message_carries_a_proc_input() {
    let message = build_exec_filter_message(99);
    assert_eq!(message.len(), 44);
    assert_eq!(u32::from_ne_bytes(message[0..4].try_into().expect("length field")), 44);
    assert_eq!(u16::from_ne_bytes(message[4..6].try_into().expect("type field")), 3);
    assert_eq!(u32::from_ne_bytes(message[12..16].try_into().expect("port field")), 99);
    assert_eq!(u32::from_ne_bytes(message[16..20].try_into().expect("idx field")), 1);
    assert_eq!(u32::from_ne_bytes(message[20..24].try_into().expect("val field")), 1);
    assert_eq!(u16::from_ne_bytes(message[32..34].try_into().expect("payload length")), 8);
    assert_eq!(u32::from_ne_bytes(message[36..40].try_into().expect("operation")), 1);
    assert_eq!(u32::from_ne_bytes(message[40..44].try_into().expect("event mask")), 2);
    assert_eq!(message[4..32], build_listen_message(99)[4..32]);
}

/// The ignore message has the layout of the listen message with the ignore operation.
#[test]
fn the_ignore_message_carries_the_ignore_operation() {
    let message = build_ignore_message(99);
    assert_eq!(message.len(), 40);
    assert_eq!(message[..32], build_listen_message(99)[..32]);
    assert_eq!(u32::from_ne_bytes(message[36..40].try_into().expect("operation")), 2);
    assert_eq!(u16::from_ne_bytes(message[32..34].try_into().expect("payload length")), 4);
}

/// The receive buffer request is honored up to the limit of the host without privilege,
/// and the socket reports at least that much.
#[test]
fn the_receive_buffer_is_enlarged() {
    let (socket, _port) = user_netlink_socket();
    let before = receive_buffer(&socket);
    enlarge_receive_buffer(&socket, 1_048_576).expect("raise the receive buffer");
    let after = receive_buffer(&socket);
    assert!(after >= 1_048_576, "the buffer is {after} bytes after the request, {before} before");
}

/// Reads the size of the receive buffer of a socket.
fn receive_buffer(socket: &OwnedFd) -> libc::c_int {
    let mut value: libc::c_int = 0;
    let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    // SAFETY: the pointers describe the integer and its length for the duration of the call.
    let result = unsafe { libc::getsockopt(socket.as_raw_fd(), libc::SOL_SOCKET, libc::SO_RCVBUF, std::ptr::addr_of_mut!(value).cast(), &mut length) };
    assert_eq!(result, 0);
    return value;
}

/// Counts the kinds of process events a socket receives while this test starts and ends a child.
/// Returns the exec,
/// fork and exit counts that arrived from the kernel.
fn event_kinds_during_a_child(socket: &OwnedFd) -> (usize, usize, usize) {
    let mut child = std::process::Command::new("/usr/bin/true").spawn().expect("start the child");
    child.wait().expect("child exits");
    let (mut exec, mut fork, mut exit) = (0, 0, 0);
    let mut buffer = [0_u8; 1024];
    for _ in 0..25 {
        loop {
            let Ok((length, sender)) = crate::process_events::receive_datagram(socket, &mut buffer) else {
                break;
            };
            if sender != 0 || length < 40 {
                continue;
            }
            match u32::from_ne_bytes(buffer[36..40].try_into().expect("event kind")) {
                1 => fork += 1,
                2 => exec += 1,
                0x8000_0000 => exit += 1,
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    return (exec, fork, exit);
}

/// The filtered subscription receives exec events and none of the fork and exit events that the plain subscription receives.
/// The plain subscription is the positive control:
/// without it,
/// a filtered socket that received nothing at all would pass.
/// It needs the kernel connector,
/// which this kernel allows without privilege and a container may not.
#[test]
#[ignore = "needs the kernel process-event connector"]
fn a_filtered_subscription_receives_exec_events_only() {
    let everything = subscribe_to_everything().expect("subscribe to everything");
    let (exec_all, fork_all, exit_all) = event_kinds_during_a_child(&everything);
    assert!(exec_all > 0 && fork_all > 0 && exit_all > 0, "the control saw exec {exec_all}, fork {fork_all}, exit {exit_all}");
    // The netlink port of the socket is the process id, so the control socket must be closed first.
    drop(everything);
    let filtered = crate::process_events::subscribe().expect("subscribe to exec events");
    let (exec, fork, exit) = event_kinds_during_a_child(&filtered);
    assert!(exec > 0, "the filtered socket saw no exec event");
    println!("events in one 0.5 s window: unfiltered exec {exec_all}, fork {fork_all}, exit {exit_all}; filtered exec {exec}");
    assert_eq!((fork, exit), (0, 0), "the filtered socket saw fork {fork} and exit {exit} events");
}

/// After the subscription is ended the kernel delivers nothing more to the socket.
/// The first child shows the subscription works,
/// so a socket that never received anything cannot pass.
/// It needs the kernel connector,
/// which this kernel allows without privilege and a container may not.
#[test]
#[ignore = "needs the kernel process-event connector"]
fn ending_the_subscription_ends_the_delivery() {
    let socket = crate::process_events::subscribe().expect("subscribe to exec events");
    let (before, _fork, _exit) = event_kinds_during_a_child(&socket);
    assert!(before > 0, "the subscription delivered no exec event before it was ended");
    unsubscribe(&socket).expect("end the subscription");
    // Events queued before the ignore are drained, then a new child must not be reported.
    let _queued = event_kinds_during_a_child(&socket);
    let (after, fork, exit) = event_kinds_during_a_child(&socket);
    assert_eq!((after, fork, exit), (0, 0, 0), "the socket still received events after the subscription was ended");
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

/// A source that never runs dry,
/// like a host-wide storm of process starts.
struct Endless {
    /// Datagrams handed out.
    handed: usize,
}

/// The endless source.
impl DatagramSource for Endless {
    /// Hands out a valid exec event from the kernel port every time.
    fn receive(&mut self, buffer: &mut [u8]) -> std::io::Result<(usize, u32)> {
        let datagram = event_datagram(2, 1000 + u32::try_from(self.handed % 50).expect("small"), 1);
        buffer[..datagram.len()].copy_from_slice(&datagram);
        self.handed += 1;
        return Ok((datagram.len(), 0));
    }
}

/// A drain returns after a bounded number of datagrams even when more keep arriving,
/// so signals and cgroup events get their turn,
/// and it reports each pid once however often it appeared.
#[test]
fn a_drain_is_bounded_and_deduplicates() {
    let mut source = Endless { handed: 0 };
    let drained = drain_exec_events_from(&mut source).expect("drain");
    assert_eq!(source.handed, MAX_DATAGRAMS_PER_DRAIN);
    assert_eq!(drained.pids.len(), 50);
    let mut sorted = drained.pids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 50);
}
