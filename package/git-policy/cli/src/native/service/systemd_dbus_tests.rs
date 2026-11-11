//! Controls for the minimal D-Bus client:
//! the layout of a call against the specification worked out by hand,
//! the reading
//!  of replies built by hand,
//! and whole exchanges against a peer that speaks the bus's side.

use super::*;
use crate::systemd_dbus_wire_format::*;
use crate::test_support::remove;

/// A directory whose path is short enough for a socket,
/// in the agent scratch directory.
fn short_directory(name: &str) -> std::path::PathBuf {
    let scratch = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    let directory = scratch.join(format!("dbus-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("scratch directory");
    return directory;
}
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

/// A string-like header field written out byte by byte:
/// code,
/// a one-character signature,
/// the length,
/// the text and its end.
fn hand_field(code: u8, kind: u8, text: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![code, 1, kind, 0];
    bytes.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
    bytes.extend(text.as_bytes());
    bytes.push(0);
    return bytes;
}

/// Pads to the next multiple of eight.
fn pad8(bytes: &mut Vec<u8>) {
    while bytes.len() % 8 != 0 {
        bytes.push(0);
    }
}

/// A message built by hand:
/// the fixed header,
/// the header fields each starting on an eight-byte boundary,
/// and the body.
fn hand_message(kind: u8, serial: u32, fields: &[Vec<u8>], body: &[u8]) -> Vec<u8> {
    let mut array: Vec<u8> = Vec::new();
    for field in fields {
        pad8(&mut array);
        array.extend(field);
    }
    let mut message: Vec<u8> = vec![b'l', kind, 0, 1];
    message.extend(u32::try_from(body.len()).expect("short").to_le_bytes());
    message.extend(serial.to_le_bytes());
    message.extend(u32::try_from(array.len()).expect("short").to_le_bytes());
    message.extend(array);
    pad8(&mut message);
    message.extend(body);
    return message;
}

/// A call without arguments has the layout the specification gives:
/// sixteen bytes of fixed header and array length,
/// then
///  four fields on eight-byte boundaries,
/// the whole header padded to eight,
/// and no body.
#[test]
fn a_call_without_arguments_has_the_specified_layout() {
    let wanted: Vec<u8> = hand_message(
        METHOD_CALL,
        1,
        &[
            hand_field(FIELD_PATH, b'o', MANAGER_PATH),
            hand_field(FIELD_DESTINATION, b's', "org.freedesktop.systemd1"),
            hand_field(FIELD_INTERFACE, b's', MANAGER_INTERFACE),
            hand_field(FIELD_MEMBER, b's', "Reload"),
        ],
        &[],
    );
    assert_eq!(encode_call(1, &RELOAD, &[]), wanted);
    // The numbers worked out by hand from the specification: 34, 33, 41 and 15 bytes of fields at offsets 16, 56, 96 and 144.
    assert_eq!(wanted.len(), 160);
    assert_eq!(u32::from_le_bytes([wanted[12], wanted[13], wanted[14], wanted[15]]), 143);
}

/// A call with two strings carries their signature and the strings,
/// each on a four-byte boundary,
/// in the body.
#[test]
fn a_call_with_two_strings_carries_them_in_the_body() {
    let message: Vec<u8> = encode_call(7, &START_UNIT, &["cli-git.socket", "replace"]);
    let mut body: Vec<u8> = Vec::new();
    for text in ["cli-git.socket", "replace"] {
        while body.len() % 4 != 0 {
            body.push(0);
        }
        body.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
        body.extend(text.as_bytes());
        body.push(0);
    }
    let split: usize = message.len() - body.len();
    assert_eq!(split % 8, 0, "the body starts on an eight-byte boundary");
    assert_eq!(&message[split..], body.as_slice());
    assert_eq!(u32::from_le_bytes([message[4], message[5], message[6], message[7]]) as usize, body.len());
    assert_eq!(u32::from_le_bytes([message[8], message[9], message[10], message[11]]), 7);
    let signature: Vec<u8> = vec![FIELD_SIGNATURE, 1, b'g', 0, 2, b's', b's', 0];
    assert!(message.windows(signature.len()).any(|window| { return window == signature.as_slice(); }));
}

/// The reply-serial field of a hand-built reply.
fn serial_field(serial: u32) -> Vec<u8> {
    let mut field: Vec<u8> = vec![FIELD_REPLY_SERIAL, 1, b'u', 0];
    field.extend(serial.to_le_bytes());
    return field;
}

/// A return reply whose only field is the number of the call it answers is read,
/// and an error reply gives its name and text.
#[test]
fn replies_built_by_hand_are_read() {
    let returned: Vec<u8> = hand_message(METHOD_RETURN, 2, &[serial_field(5)], &[]);
    assert_eq!(decode_reply(&returned), Ok(Reply { kind: METHOD_RETURN, reply_serial: Some(5), error_name: None, first_text: None }));
    let mut body: Vec<u8> = Vec::new();
    body.extend(7u32.to_le_bytes());
    body.extend(b"no room\0");
    let failed: Vec<u8> = hand_message(
        ERROR,
        3,
        &[hand_field(FIELD_ERROR_NAME, b's', "org.freedesktop.systemd1.NoSuchUnit"), serial_field(9), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]],
        &body,
    );
    assert_eq!(
        decode_reply(&failed),
        Ok(Reply {
            kind: ERROR,
            reply_serial: Some(9),
            error_name: Some(String::from("org.freedesktop.systemd1.NoSuchUnit")),
            first_text: Some(String::from("no room"))
        })
    );
}

/// Truncated and foreign replies are refused,
/// not guessed.
#[test]
fn damaged_replies_are_refused() {
    let whole: Vec<u8> = hand_message(METHOD_RETURN, 2, &[serial_field(1)], &[]);
    assert!(matches!(decode_reply(&whole[..10]), Err(CallError::Malformed(_))));
    let mut big_endian: Vec<u8> = whole.clone();
    big_endian[0] = b'B';
    assert!(matches!(decode_reply(&big_endian), Err(CallError::Malformed(_))));
    let mut lies: Vec<u8> = whole.clone();
    lies[4..8].copy_from_slice(&1000u32.to_le_bytes());
    assert!(matches!(decode_reply(&lies), Err(CallError::Malformed(_))));
    let mut long_fields: Vec<u8> = whole;
    long_fields[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(decode_reply(&long_fields), Err(CallError::Malformed(_))));
}

/// Every prefix and every single-byte change of well-formed replies is answered with a result,
/// never a panic.
#[test]
fn the_reader_is_total_over_damaged_input() {
    let mut body: Vec<u8> = Vec::new();
    body.extend(3u32.to_le_bytes());
    body.extend(b"abc\0");
    let samples: Vec<Vec<u8>> = vec![
        hand_message(METHOD_RETURN, 2, &[serial_field(1)], &[]),
        hand_message(ERROR, 3, &[hand_field(FIELD_ERROR_NAME, b's', "a.b.C"), serial_field(2), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]], &body),
    ];
    for sample in samples {
        for length in 0..=sample.len() {
            let _ = decode_reply(&sample[..length]);
        }
        for at in 0..sample.len() {
            for value in [0u8, 1, 0x7f, 0x80, 0xff] {
                let mut changed: Vec<u8> = sample.clone();
                changed[at] = value;
                let _ = decode_reply(&changed);
            }
        }
    }
}

/// The session bus path is the address variable's `unix:path=` entry,
/// else the runtime directory's `bus`.
#[test]
fn the_session_bus_path_is_found() {
    let runtime: &Path = Path::new("/run/user/1000");
    assert_eq!(session_bus_path(Some("unix:path=/run/user/1000/bus"), None), Some(std::path::PathBuf::from("/run/user/1000/bus")));
    assert_eq!(
        session_bus_path(Some("unix:abstract=/tmp/x,guid=1;unix:guid=2,path=/tmp/other"), None),
        Some(std::path::PathBuf::from("/tmp/other"))
    );
    assert_eq!(session_bus_path(Some("unix:abstract=/tmp/x"), Some(runtime)), Some(runtime.join("bus")));
    assert_eq!(session_bus_path(Some("unix:path=/a%20b"), Some(runtime)), Some(runtime.join("bus")));
    assert_eq!(session_bus_path(None, Some(runtime)), Some(runtime.join("bus")));
    assert_eq!(session_bus_path(None, None), None);
}

/// What a peer that plays the bus does with each call,
/// in order.
struct Script {
    answers: Vec<Vec<Vec<u8>>>,
}

/// A peer on the bus's side:
/// the line-based authentication,
/// then for each call the messages the script gives (a signal first,
///  to show it is skipped).
fn serve_bus(listener: UnixListener, script: Script, seen: std::sync::mpsc::Sender<(String, Vec<Vec<u8>>)>) {
    std::thread::spawn(move || {
        let (mut stream, first) = accept_and_authenticate(&listener);
        let mut calls: Vec<Vec<u8>> = Vec::new();
        for answers in script.answers {
            let call = read_call(&mut stream);
            let serial = serial_of(&call);
            calls.push(call);
            for answer in answers {
                stream.write_all(&answering(answer, serial)).expect("answer");
            }
        }
        let _sent = seen.send((first, calls));
    });
}

/// Accepts a client and takes it through the line-based authentication;
/// its first line is given back.
fn accept_and_authenticate(listener: &UnixListener) -> (UnixStream, String) {
    let (mut stream, _) = listener.accept().expect("a client");
    let mut one: [u8; 1] = [0];
    let mut first: Vec<u8> = Vec::new();
    while !first.ends_with(b"\r\n") {
        stream.read_exact(&mut one).expect("auth bytes");
        first.push(one[0]);
    }
    stream.write_all(b"OK 00112233445566778899aabbccddeeff\r\n").expect("ok");
    // zbus pipelines `NEGOTIATE_UNIX_FD` and `BEGIN` after the answer; the peer agrees to the first and stops at the second.
    loop {
        let mut line: Vec<u8> = Vec::new();
        while !line.ends_with(b"\r\n") {
            stream.read_exact(&mut one).expect("begin bytes");
            line.push(one[0]);
        }
        if line == b"NEGOTIATE_UNIX_FD\r\n" {
            stream.write_all(b"AGREE_UNIX_FD\r\n").expect("agree");
        } else {
            break;
        }
    }
    return (stream, String::from_utf8_lossy(&first).into_owned());
}

/// The serial number of a call:
/// zbus counts serials for the whole process,
/// so a peer must answer the number it was given.
fn serial_of(call: &[u8]) -> u32 {
    return u32::from_le_bytes([call[8], call[9], call[10], call[11]]);
}

/// A prebuilt answer with its `reply_serial` field set to `serial`.
fn answering(mut answer: Vec<u8>, serial: u32) -> Vec<u8> {
    if let Some(at) = answer.windows(4).position(|window| { return window == [5, 1, b'u', 0]; }) {
        answer[at + 4..at + 8].copy_from_slice(&serial.to_le_bytes());
    }
    return answer;
}

/// Reads one whole method call from the client.
fn read_call(stream: &mut UnixStream) -> Vec<u8> {
    let mut header: Vec<u8> = vec![0; 16];
    stream.read_exact(&mut header).expect("header");
    let body: usize = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let fields: usize = u32::from_le_bytes([header[12], header[13], header[14], header[15]]) as usize;
    let mut rest: Vec<u8> = vec![0; (16 + fields).div_ceil(8) * 8 + body - 16];
    stream.read_exact(&mut rest).expect("rest");
    header.extend(rest);
    return header;
}

/// A signal the bus sends right after `Hello`,
/// which a client must skip.
fn name_acquired() -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(5u32.to_le_bytes());
    body.extend(b":1.42\0");
    // A real signal carries a path, an interface and a member; zbus refuses one without them, the hand client skipped any signal.
    return hand_message(
        4,
        1,
        &[
            hand_field(1, b'o', "/org/freedesktop/DBus"),
            hand_field(2, b's', "org.freedesktop.DBus"),
            hand_field(FIELD_MEMBER, b's', "NameAcquired"),
            vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0],
        ],
        &body,
    );
}

fn string_return(serial: u32, text: &str) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
    body.extend(text.as_bytes());
    body.push(0);
    return hand_message(METHOD_RETURN, serial + 10, &[serial_field(serial), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]], &body);
}

/// A return whose body is one object path.
fn path_return(serial: u32, text: &str) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
    body.extend(text.as_bytes());
    body.push(0);
    return hand_message(METHOD_RETURN, serial + 10, &[serial_field(serial), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b'o', 0]], &body);
}

/// A return whose body is one variant holding a string (or,
/// with `inner` other than `s`,
/// a value of that type).
fn variant_return(serial: u32, inner: u8, text: &str) -> Vec<u8> {
    // The variant's own signature (length, type, end), one byte of padding to the four-byte boundary, then the value.
    let mut body: Vec<u8> = vec![1, inner, 0, 0];
    body.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
    body.extend(text.as_bytes());
    body.push(0);
    return hand_message(METHOD_RETURN, serial + 10, &[serial_field(serial), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b'v', 0]], &body);
}

/// An error reply for the call `serial`,
/// with the manager's error name and a text.
fn error_return(serial: u32, name: &str, text: &str) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
    body.extend(text.as_bytes());
    body.push(0);
    return hand_message(ERROR, serial + 10, &[hand_field(FIELD_ERROR_NAME, b's', name), serial_field(serial), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]], &body);
}

/// What:
///  The body of a list of address pairs (`a(ss)`):
/// the byte length of the entries,
/// padding to the eight-byte boundary one pair starts on,
/// then each pair as its two strings,
/// each on a four-byte boundary.
/// Why:
///   A socket unit's `Listen` is one pair per address,
/// and a control has to be able to hand the client a reply the manager could send.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// addressesBody(['/run/user/1000/x.sock']) // bytes of an a(ss) body
/// ```
fn addresses_body(addresses: &[&str]) -> Vec<u8> {
    let mut entries: Vec<u8> = Vec::new();
    for address in addresses {
        while entries.len() % 8 != 0 {
            entries.push(0);
        }
        for text in ["Stream", *address] {
            while entries.len() % 4 != 0 {
                entries.push(0);
            }
            entries.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
            entries.extend(text.as_bytes());
            entries.push(0);
        }
    }
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(entries.len()).expect("short").to_le_bytes());
    while body.len() % 8 != 0 {
        body.push(0);
    }
    body.extend(entries);
    return body;
}

/// A return whose body is one variant holding a list of address pairs.
fn addresses_return(serial: u32, addresses: &[&str]) -> Vec<u8> {
    let mut body: Vec<u8> = vec![5, b'a', b'(', b's', b's', b')', 0, 0];
    body.extend(addresses_body(addresses));
    return hand_message(METHOD_RETURN, serial + 10, &[serial_field(serial), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b'v', 0]], &body);
}

/// What:
///  The addresses of a unit are read from its object,
/// and a unit the manager does not hold has none.
/// Why:
///   The address the manager made is the one fact that says whether a client given another runtime directory can
///  reach the service,
/// so the reading of `Listen` is what the refusal rests on.
#[test]
fn the_addresses_of_a_unit_are_read_from_its_object_and_an_unloaded_unit_has_none() {
    let root = short_directory("listen");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let (sender, receiver) = std::sync::mpsc::channel();
    let script: Script = Script {
        answers: vec![
            vec![string_return(1, ":1.42")],
            vec![path_return(2, "/org/freedesktop/systemd1/unit/cli_2dgit_40a_2esocket")],
            vec![addresses_return(3, &["/run/user/1000/monochromatic-git-policy-cli/a.sock"])],
            vec![error_return(4, NO_SUCH_UNIT, "not loaded")],
        ],
    };
    serve_bus(listener, script, sender);
    let mut bus: Bus = Bus::connect(socket.as_path(), Duration::from_secs(10)).expect("connected");
    assert_eq!(bus.listen_addresses("cli-git@a.socket"), Ok(Some(vec![String::from("/run/user/1000/monochromatic-git-policy-cli/a.sock")])));
    assert_eq!(bus.listen_addresses("cli-git@b.socket"), Ok(None), "a unit the manager does not hold has no address");
    let (_auth, calls) = receiver.recv().expect("seen");
    // The property asked for is `Listen`, of the socket interface: the one property that says where a service answers.
    let interface: &[u8] = b"org.freedesktop.systemd1.Socket";
    let asked: bool = calls[2].windows(6).any(|window| { return window == b"Listen"; });
    let named: bool = calls[2].windows(interface.len()).any(|window| { return window == interface; });
    assert!(asked && named, "the address question names the socket interface's Listen");
    remove(root.as_path());
}

/// An object path and a variant holding a string are read as the first text of a reply;
/// a variant holding another type,
/// or
///  one cut short,
/// is not.
#[test]
fn an_object_path_and_a_string_variant_are_read() {
    assert_eq!(decode_reply(&path_return(4, "/org/freedesktop/systemd1/unit/a_2eservice")).expect("read").first_text.as_deref(), Some("/org/freedesktop/systemd1/unit/a_2eservice"));
    let variant: Vec<u8> = variant_return(4, b's', "deactivating");
    assert_eq!(decode_reply(&variant).expect("read").first_text.as_deref(), Some("deactivating"));
    assert_eq!(decode_reply(&variant_return(4, b'u', "xxxx")).expect("read").first_text, None);
    for length in 0..variant.len() {
        let _ = decode_reply(&variant[..length]);
    }
    for at in 0..variant.len() {
        for value in [0u8, 1, 0x7f, 0x80, 0xff] {
            let mut changed: Vec<u8> = variant.clone();
            changed[at] = value;
            let _ = decode_reply(&changed);
        }
    }
}

/// The text of a reply's body string,
/// for the controls that damage the end of a string.
const STATE_TEXT: &str = "deactivating";

/// The first text of a decoded reply message.
fn first_text(message: &[u8]) -> Option<String> {
    return decode_reply(message).expect("a message").first_text;
}

/// A signature field for a reply whose body is a string,
/// with the given end byte.
fn string_signature(end: u8) -> Vec<u8> {
    return vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', end];
}

/// A string reply whose byte after the text is not the end byte is no text;
/// the same reply with it is read.
#[test]
fn a_body_string_without_its_end_byte_is_not_read() {
    let intact: Vec<u8> = string_return(4, STATE_TEXT);
    assert_eq!(first_text(&intact).as_deref(), Some(STATE_TEXT));
    let mut unterminated: Vec<u8> = intact;
    let end: usize = unterminated.len() - 1;
    unterminated[end] = b'X';
    assert_eq!(first_text(&unterminated), None);
}

/// A string whose end byte lies past the declared body is no text,
/// even when a byte that would end it follows the message.
#[test]
fn a_body_string_whose_end_byte_lies_outside_the_declared_body_is_not_read() {
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(STATE_TEXT.len()).expect("short").to_le_bytes());
    body.extend(STATE_TEXT.as_bytes());
    let mut outside: Vec<u8> = hand_message(METHOD_RETURN, 14, &[serial_field(4), string_signature(0)], &body);
    outside.push(0);
    assert_eq!(first_text(&outside), None);
}

/// A variant holding a string is read;
/// one whose string lacks its end byte,
/// or whose own signature lacks its end byte,
/// is not.
#[test]
fn a_variant_without_an_end_byte_is_not_read() {
    let variant: Vec<u8> = variant_return(4, b's', STATE_TEXT);
    assert_eq!(first_text(&variant).as_deref(), Some(STATE_TEXT));
    let mut value_unterminated: Vec<u8> = variant.clone();
    let value_end: usize = value_unterminated.len() - 1;
    value_unterminated[value_end] = b'X';
    assert_eq!(first_text(&value_unterminated), None, "a variant string whose end byte is another byte");
    // The variant's own signature is its length, its type and its end byte, at the start of the body.
    let body_length: usize = 4 + 4 + STATE_TEXT.len() + 1;
    let body_start: usize = variant.len() - body_length;
    let mut signature_unterminated: Vec<u8> = variant;
    signature_unterminated[body_start + 2] = b'X';
    assert_eq!(first_text(&signature_unterminated), None, "a variant signature whose end byte is another byte");
}

/// A reply whose own signature field lacks its end byte is not interpreted by that signature.
#[test]
fn a_signature_field_without_its_end_byte_is_not_obeyed() {
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(STATE_TEXT.len()).expect("short").to_le_bytes());
    body.extend(STATE_TEXT.as_bytes());
    body.push(0);
    let ended: Vec<u8> = hand_message(METHOD_RETURN, 14, &[serial_field(4), string_signature(0)], &body);
    assert_eq!(first_text(&ended).as_deref(), Some(STATE_TEXT));
    let unended: Vec<u8> = hand_message(METHOD_RETURN, 14, &[serial_field(4), string_signature(b'X')], &body);
    assert_eq!(first_text(&unended), None);
}

/// The state query asks `GetUnit`,
/// then reads `ActiveState` from the unit's object,
/// and a unit the manager does not hold has no
///  state.
#[test]
fn the_state_of_a_unit_is_read_from_its_object_and_an_unloaded_unit_has_none() {
    let root = short_directory("state");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let (sender, receiver) = std::sync::mpsc::channel();
    let script: Script = Script {
        answers: vec![
            vec![string_return(1, ":1.42")],
            vec![path_return(2, "/org/freedesktop/systemd1/unit/cli_2dgit_2eservice")],
            vec![variant_return(3, b's', "deactivating")],
            vec![error_return(4, NO_SUCH_UNIT, "not loaded")],
        ],
    };
    serve_bus(listener, script, sender);
    let mut bus: Bus = Bus::connect(socket.as_path(), Duration::from_secs(10)).expect("connected");
    assert_eq!(bus.active_state("cli-git.service"), Ok(Some(String::from("deactivating"))));
    assert_eq!(bus.active_state("cli-git.socket"), Ok(None));
    let (_auth, calls) = receiver.recv().expect("seen");
    let property: Target<'_> = Target {
        destination: MANAGER_NAME,
        path: "/org/freedesktop/systemd1/unit/cli_2dgit_2eservice",
        interface: "org.freedesktop.DBus.Properties",
        member: "Get",
    };
    // Layouts are zbus's own now; the members called, in order, are what the exchange must show.
    let _unused = (&property, &GET_UNIT);
    let members: Vec<bool> = [b"GetUnit".as_slice(), b"Get".as_slice(), b"GetUnit".as_slice()].iter().enumerate().map(|(at, member)| { return calls[at + 1].windows(member.len()).any(|window| { return window == *member; }); }).collect();
    assert_eq!(members, vec![true, true, true]);
    remove(root.as_path());
}

/// A whole exchange:
/// the identity is sent as the hexadecimal of the user number's digits,
/// `Hello` comes first,
/// a signal in
///  between is skipped,
/// and the reload and the start arrive intact and succeed.
#[test]
fn a_session_says_hello_then_reloads_and_starts() {
    let root = short_directory("exchange");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let (sender, receiver) = std::sync::mpsc::channel();
    let script: Script = Script {
        answers: vec![
            vec![string_return(1, ":1.42"), name_acquired()],
            vec![hand_message(METHOD_RETURN, 12, &[serial_field(2)], &[])],
            vec![hand_message(METHOD_RETURN, 13, &[serial_field(3)], &[])],
        ],
    };
    serve_bus(listener, script, sender);
    let mut bus: Bus = Bus::connect(socket.as_path(), Duration::from_secs(10)).expect("connected");
    bus.reload().expect("reloaded");
    bus.start_unit("cli-git.socket").expect("started");
    let (auth, calls) = receiver.recv().expect("seen");
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let user: u32 = unsafe { libc::geteuid() };
    let digits: String = user.to_string().bytes().map(|byte| { return format!("{byte:02x}"); }).collect();
    assert_eq!(auth, format!("\0AUTH EXTERNAL {digits}\r\n"));
    // The hand-written layout of each call belonged to the hand client; zbus writes its own (sender, flags, serial).
    assert_eq!(calls.len(), 3);
    remove(root.as_path());
}

/// A peer that answers `Hello`,
/// reads the next call,
/// answers it after `delay` (never,
/// when it has no `answer`) and then holds the
///  connection open for `hold`.
fn serve_after_hello(listener: UnixListener, delay: Duration, answer: Option<Vec<u8>>, hold: Duration) {
    std::thread::spawn(move || {
        let (mut stream, _) = accept_and_authenticate(&listener);
        let hello = read_call(&mut stream);
        stream.write_all(&answering(string_return(1, ":1.42"), serial_of(&hello))).expect("the answer to hello");
        let call = read_call(&mut stream);
        std::thread::sleep(delay);
        if let Some(bytes) = answer {
            let _written = stream.write_all(&answering(bytes, serial_of(&call)));
        }
        std::thread::sleep(hold);
    });
}

/// A call that gets no answer within the time it was given fails as a timeout within that time,
/// whatever time the connection
///  was given;
/// a silent bus is not a refused or a broken connection.
#[test]
fn a_call_is_bounded_by_the_time_it_was_given_and_a_timeout_is_told_apart() {
    let root = short_directory("within-bound");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let wait: Duration = Duration::from_millis(300);
    let patience: Duration = Duration::from_secs(10);
    // The bus holds the connection for ten minutes without answering: a call that is not bounded by the time it was given
    // hangs, and the per-test limit reports it; a call that is bounded ends as a timeout, which only the bound produces.
    serve_after_hello(listener, patience, None, Duration::from_secs(600));
    let mut bus: Bus = Bus::connect(socket.as_path(), patience).expect("connected");
    bus.within(wait);
    assert_eq!(bus.reload(), Err(CallError::TimedOut));
    remove(root.as_path());
}

/// The time given to the calls that follow replaces what was left of the connection's own,
/// so a phase that was slow does not
///  make the next one fail.
#[test]
fn a_new_time_replaces_what_was_left_of_the_connections() {
    let root = short_directory("within-extends");
    let given: Duration = Duration::from_millis(200);
    let spent: Duration = given * 2;
    let patience: Duration = Duration::from_secs(10);
    // The control: without a new time, a call made after the connection's time is spent times out.
    // The connection is given `patience` and then its own time with `within`, because the time of a connect also bounds the
    //  handshake and `Hello`, and under a loaded suite those two took longer than 200 ms, so the control failed as
    //  "connected: TimedOut" (`a_new_time_replaces_what_was_left_of_the_connections`, the suite of 2026-11-10).
    let control_socket = root.join("control");
    serve_after_hello(UnixListener::bind(&control_socket).expect("listen"), Duration::ZERO, Some(hand_message(METHOD_RETURN, 12, &[serial_field(2)], &[])), Duration::from_secs(1));
    let mut control: Bus = Bus::connect(control_socket.as_path(), patience).expect("connected");
    control.within(given);
    std::thread::sleep(spent);
    assert_eq!(control.reload(), Err(CallError::TimedOut), "the connection's own time is spent");
    // With a new time, the same call after the same delay succeeds.
    let socket = root.join("bus");
    serve_after_hello(UnixListener::bind(&socket).expect("listen"), Duration::ZERO, Some(hand_message(METHOD_RETURN, 12, &[serial_field(2)], &[])), Duration::from_secs(1));
    let mut bus: Bus = Bus::connect(socket.as_path(), patience).expect("connected");
    bus.within(given);
    std::thread::sleep(spent);
    bus.within(patience);
    assert_eq!(bus.reload(), Ok(()));
    remove(root.as_path());
}

/// An error reply comes back with the manager's name and text.
#[test]
fn an_error_reply_names_the_error() {
    let root = short_directory("error");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut body: Vec<u8> = Vec::new();
    body.extend(9u32.to_le_bytes());
    body.extend(b"not found\0");
    let error: Vec<u8> = hand_message(
        ERROR,
        12,
        &[hand_field(FIELD_ERROR_NAME, b's', "org.freedesktop.systemd1.NoSuchUnit"), serial_field(2), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]],
        &body,
    );
    serve_bus(listener, Script { answers: vec![vec![string_return(1, ":1.42")], vec![error]] }, sender);
    let mut bus: Bus = Bus::connect(socket.as_path(), Duration::from_secs(10)).expect("connected");
    assert_eq!(
        bus.start_unit("x.socket"),
        Err(CallError::Manager { name: String::from("org.freedesktop.systemd1.NoSuchUnit"), text: String::from("not found") })
    );
    remove(root.as_path());
}

/// A refused authentication,
/// a socket nobody listens on,
/// and a bus that never answers are told apart,
/// the last within
///  the bound.
#[test]
fn refusals_missing_sockets_and_silence_are_told_apart() {
    let root = short_directory("refused");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("a client");
        let mut one: [u8; 1] = [0];
        let mut seen: Vec<u8> = Vec::new();
        while !seen.ends_with(b"\r\n") {
            stream.read_exact(&mut one).expect("bytes");
            seen.push(one[0]);
        }
        stream.write_all(b"REJECTED EXTERNAL\r\n").expect("reject");
    });
    assert!(matches!(Bus::connect(socket.as_path(), Duration::from_secs(10)), Err(CallError::Authentication(_))));
    assert!(matches!(Bus::connect(root.join("nobody").as_path(), Duration::from_secs(10)), Err(CallError::Connection(_))));
    let silent = root.join("silent");
    let quiet_listener = UnixListener::bind(&silent).expect("listen");
    // The silent peer holds its end until the connect has ended, so only the bound of the connect can end it.
    let (release, held) = std::sync::mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let (stream, _) = quiet_listener.accept().expect("a client");
        let _released = held.recv();
        drop(stream);
    });
    let wait: Duration = Duration::from_millis(300);
    assert!(Bus::connect(silent.as_path(), wait).is_err());
    release.send(()).expect("the holder lets go");
    holder.join().expect("holder");
    remove(root.as_path());
}

/// Against the real user manager,
/// once by hand:
/// a uniquely named runtime-only socket unit is written,
/// started with
///  `StartUnit` with no reload (the manager finds a new file by itself;
/// the socket appears),
/// stopped with `StopUnit`,
/// and its files removed.
/// Run it with
///  `--run-ignored only`;
/// it touches the account's real service manager,
/// so it is not part of the ordinary run.
#[test]
#[ignore = "touches the real user service manager; run by hand"]
fn the_real_user_manager_loads_and_starts_a_runtime_only_probe_unit() {
    let runtime = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").expect("a runtime directory"));
    let bus_path = session_bus_path(std::env::var("DBUS_SESSION_BUS_ADDRESS").ok().as_deref(), Some(runtime.as_path())).expect("a session bus");
    let name = format!("cli-git-probe-{}", std::process::id());
    let units = runtime.join("systemd").join("user");
    std::fs::create_dir_all(&units).expect("unit directory");
    let socket_path = runtime.join(format!("{name}.sock"));
    std::fs::write(units.join(format!("{name}.socket")), format!("[Socket]\nListenStream={}\n", socket_path.display())).expect("socket unit");
    std::fs::write(units.join(format!("{name}.service")), "[Service]\nExecStart=/usr/bin/true\n").expect("service unit");
    let stop = Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "StopUnit" };
    let outcome = (|| -> Result<(), CallError> {
        let mut bus = Bus::connect(bus_path.as_path(), EXCHANGE_WAIT)?;
        bus.start_unit(format!("{name}.socket").as_str())?;
        // The reply to a start means the job is queued, so the socket may appear a moment later.
        let began = Instant::now();
        let mut listening = UnixStream::connect(&socket_path).is_ok();
        while !listening && began.elapsed() < Duration::from_secs(60) {
            std::thread::sleep(Duration::from_millis(50));
            listening = UnixStream::connect(&socket_path).is_ok();
        }
        let stopped = bus.stop_unit(format!("{name}.socket").as_str());
        let _unused_target = &stop;
        assert!(listening, "the manager made the socket listen");
        return stopped.map(drop);
    })();
    let _removed_socket = std::fs::remove_file(units.join(format!("{name}.socket")));
    let _removed_service = std::fs::remove_file(units.join(format!("{name}.service")));
    assert_eq!(outcome, Ok(()));
}

/// A stop that spares the units requiring the stopped one names the job mode that says so,
/// and the plain stop keeps `replace`.
#[test]
fn a_stop_that_spares_the_dependent_units_names_the_mode_that_ignores_dependencies() {
    let root = short_directory("stop-alone");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let (sender, receiver) = std::sync::mpsc::channel();
    let script: Script = Script {
        answers: vec![
            vec![string_return(1, ":1.42"), name_acquired()],
            vec![hand_message(METHOD_RETURN, 12, &[serial_field(2)], &[])],
            vec![hand_message(METHOD_RETURN, 13, &[serial_field(3)], &[])],
        ],
    };
    serve_bus(listener, script, sender);
    let mut bus: Bus = Bus::connect(socket.as_path(), Duration::from_secs(10)).expect("connected");
    bus.stop_unit("a.socket").expect("stopped");
    bus.stop_unit_alone("b.socket").expect("stopped alone");
    let (_auth, calls) = receiver.recv().expect("seen");
    // The layout of each call is zbus's own now; what the exchange must show is the member, the unit and the job mode of each call.
    let shows = |call: &[u8], words: [&str; 3]| -> bool { return words.iter().all(|word| { return call.windows(word.len()).any(|window| { return window == word.as_bytes(); }); }); };
    assert!(shows(calls[1].as_slice(), ["StopUnit", "a.socket", "replace"]));
    assert!(shows(calls[2].as_slice(), ["StopUnit", "b.socket", "ignore-dependencies"]));
    assert!(!shows(calls[1].as_slice(), ["StopUnit", "a.socket", "ignore-dependencies"]), "the plain stop must keep `replace`");
    remove(root.as_path());
}

/// A bus that accepts the connection and then says nothing at all ends the whole exchange,
/// handshake included,
/// at the deadline.
#[test]
fn a_bus_that_accepts_and_never_answers_ends_at_the_deadline() {
    let root = short_directory("never-answers");
    let path = root.join("bus");
    let listener = std::os::unix::net::UnixListener::bind(&path).expect("bound");
    let holder = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accepted");
        std::thread::sleep(Duration::from_secs(4));
        drop(stream);
    });
    let wait = Duration::from_secs(1);
    let began = Instant::now();
    let result = Bus::connect(path.as_path(), wait);
    let elapsed = began.elapsed();
    assert!(matches!(result, Err(CallError::TimedOut)), "expected a timeout, got {:?}", result.as_ref().err());
    assert!(elapsed >= wait && elapsed < wait + Duration::from_millis(500), "ended after {elapsed:?} for a deadline of {wait:?}");
    holder.join().expect("holder");
}

/// The client speaks through `zbus` (the owner,
/// 2026-11-07:
/// "zbus 6,
/// and I suppose the lack of a blocking api can be worked
///  around.")
/// and no longer writes the wire itself:
/// no socket of its own,
/// no hand-written call or reply.
#[test]
fn the_client_speaks_through_zbus_and_no_longer_writes_the_wire_itself() {
    let source: &str = include_str!("systemd_dbus.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    for hand_written in ["UnixStream", "encode_call(", "decode_reply(", "read_exact"] {
        assert!(!code.contains(hand_written), "the client still writes the wire itself: `{hand_written}`");
    }
    assert!(code.contains("zbus::"), "the client does not use zbus");
}
