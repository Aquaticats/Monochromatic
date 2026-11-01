//! Controls for the minimal D-Bus client: the layout of a call against the specification worked out by hand, the reading
//!  of replies built by hand, and whole exchanges against a peer that speaks the bus's side.

use super::*;
use crate::test_support::remove;

/// A directory whose path is short enough for a socket, in the agent scratch directory.
fn short_directory(name: &str) -> std::path::PathBuf {
    let scratch = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    let directory = scratch.join(format!("dbus-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("scratch directory");
    return directory;
}
use std::os::unix::net::{UnixListener, UnixStream};

/// A string-like header field written out byte by byte: code, a one-character signature, the length, the text and its end.
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

/// A message built by hand: the fixed header, the header fields each starting on an eight-byte boundary, and the body.
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

/// A call without arguments has the layout the specification gives: sixteen bytes of fixed header and array length, then
///  four fields on eight-byte boundaries, the whole header padded to eight, and no body.
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

/// A call with two strings carries their signature and the strings, each on a four-byte boundary, in the body.
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

/// A return reply whose only field is the number of the call it answers is read, and an error reply gives its name and text.
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

/// Truncated and foreign replies are refused, not guessed.
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

/// Every prefix and every single-byte change of well-formed replies is answered with a result, never a panic.
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

/// The session bus path is the address variable's `unix:path=` entry, else the runtime directory's `bus`.
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

/// What a peer that plays the bus does with each call, in order.
struct Script {
    answers: Vec<Vec<Vec<u8>>>,
}

/// A peer on the bus's side: the line-based authentication, then for each call the messages the script gives (a signal first,
///  to show it is skipped).
fn serve_bus(listener: UnixListener, script: Script, seen: std::sync::mpsc::Sender<(String, Vec<Vec<u8>>)>) {
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("a client");
        let mut one: [u8; 1] = [0];
        let mut first: Vec<u8> = Vec::new();
        while !first.ends_with(b"\r\n") {
            stream.read_exact(&mut one).expect("auth bytes");
            first.push(one[0]);
        }
        stream.write_all(b"OK 00112233445566778899aabbccddeeff\r\n").expect("ok");
        let mut begin: Vec<u8> = Vec::new();
        while !begin.ends_with(b"\r\n") {
            stream.read_exact(&mut one).expect("begin bytes");
            begin.push(one[0]);
        }
        let mut calls: Vec<Vec<u8>> = Vec::new();
        for answers in script.answers {
            let mut header: Vec<u8> = vec![0; 16];
            stream.read_exact(&mut header).expect("header");
            let body: usize = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
            let fields: usize = u32::from_le_bytes([header[12], header[13], header[14], header[15]]) as usize;
            let mut rest: Vec<u8> = vec![0; (16 + fields).div_ceil(8) * 8 + body - 16];
            stream.read_exact(&mut rest).expect("rest");
            header.extend(rest);
            calls.push(header);
            for answer in answers {
                stream.write_all(&answer).expect("answer");
            }
        }
        let _sent = seen.send((String::from_utf8_lossy(&first).into_owned(), calls));
    });
}

/// A signal the bus sends right after `Hello`, which a client must skip.
fn name_acquired() -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(5u32.to_le_bytes());
    body.extend(b":1.42\0");
    return hand_message(4, 1, &[hand_field(FIELD_MEMBER, b's', "NameAcquired"), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]], &body);
}

fn string_return(serial: u32, text: &str) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
    body.extend(text.as_bytes());
    body.push(0);
    return hand_message(METHOD_RETURN, serial + 10, &[serial_field(serial), vec![FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]], &body);
}

/// A whole exchange: the identity is sent as the hexadecimal of the user number's digits, `Hello` comes first, a signal in
///  between is skipped, and the reload and the start arrive intact and succeed.
#[test]
fn a_session_says_hello_then_reloads_and_starts() {
    let root = short_directory("exchange");
    let socket = root.join("bus");
    let listener = UnixListener::bind(&socket).expect("listen");
    let (sender, receiver) = std::sync::mpsc::channel();
    let script: Script = Script {
        answers: vec![
            vec![name_acquired(), string_return(1, ":1.42")],
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
    assert_eq!(calls, vec![encode_call(1, &HELLO, &[]), encode_call(2, &RELOAD, &[]), encode_call(3, &START_UNIT, &["cli-git.socket", "replace"])]);
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

/// A refused authentication, a socket nobody listens on, and a bus that never answers are told apart, the last within
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
    let holder = std::thread::spawn(move || {
        let (stream, _) = quiet_listener.accept().expect("a client");
        std::thread::sleep(Duration::from_secs(2));
        drop(stream);
    });
    let began: Instant = Instant::now();
    let wait: Duration = Duration::from_millis(300);
    assert!(Bus::connect(silent.as_path(), wait).is_err());
    assert!(began.elapsed() < wait * 5, "the bound held");
    holder.join().expect("holder");
    remove(root.as_path());
}

/// Against the real user manager, once by hand: a uniquely named runtime-only socket unit is written, started with
///  `StartUnit` with no reload (the manager finds a new file by itself; the socket appears), stopped with `StopUnit`, and its files removed. Run it with
///  `--run-ignored only`; it touches the account's real service manager, so it is not part of the ordinary run.
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
        assert!(UnixStream::connect(&socket_path).is_ok(), "the manager made the socket listen");
        bus.call(&stop, &[format!("{name}.socket").as_str(), "replace"])?;
        return Ok(());
    })();
    let _removed_socket = std::fs::remove_file(units.join(format!("{name}.socket")));
    let _removed_service = std::fs::remove_file(units.join(format!("{name}.service")));
    assert_eq!(outcome, Ok(()));
}
