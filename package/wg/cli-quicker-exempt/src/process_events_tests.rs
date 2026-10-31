//! Verifies the process-event connector datagram layout on byte fixtures,
//! and the live subscription in an ignored test.

/// Parser and subscription under test.
use crate::process_events::{build_listen_message, drain_exec_events, parse_exec_event, subscribe};

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
