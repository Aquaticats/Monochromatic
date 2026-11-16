//! A live probe of the manager channel against the real user manager.
//! It is ignored by default:
//! it creates throwaway transient units in a throwaway slice,
//! stops them again,
//! and touches nothing else.
//! Its purpose is to prove each part of a message against the manager itself,
//! one argument at a time.

/// The channel under probe.
use crate::dbus_wire::{Argument, MANAGER_INTERFACE, MethodCall, Property, PropertyValue};
/// The manager and its endpoint.
use crate::manager_socket::{ManagerClient, ManagerEndpoint};

/// One part of the message,
/// answered by the manager.
fn probe(client: &ManagerClient, endpoint: &ManagerEndpoint, what: &str, call: &MethodCall) {
    match client.call(endpoint, call) {
        Ok(_reply) => println!("{what}: ok"),
        Err(error) => println!("{what}: {error}"),
    }
}

/// Probes a plain call and then each property of a transient scope in turn.
#[test]
#[ignore = "creates throwaway transient units on the real user manager"]
fn the_manager_answers_each_part_of_a_call() {
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    let endpoint = ManagerEndpoint::resolve(uid).expect("the user's manager answers");
    println!("endpoint {:?}", endpoint);
    let client = ManagerClient::new();
    probe(&client, &endpoint, "ListUnits", &MethodCall { interface: MANAGER_INTERFACE, member: "ListUnits", arguments: vec![] });
    probe(&client, &endpoint, "GetUnit", &MethodCall { interface: MANAGER_INTERFACE, member: "GetUnit", arguments: vec![Argument::Text("app.slice".to_string())] });
    probe(
        &client,
        &endpoint,
        "StopUnit of a unit that does not exist",
        &MethodCall { interface: MANAGER_INTERFACE, member: "StopUnit", arguments: vec![Argument::Text("wg-probe-nonexistent.scope".to_string()), Argument::Text("replace".to_string())] },
    );
    let slice = "app-wgprobe\\x2dshared.slice";
    probe(
        &client,
        &endpoint,
        "slice with description",
        &MethodCall {
            interface: MANAGER_INTERFACE,
            member: "StartTransientUnit",
            arguments: vec![
                Argument::Text(slice.to_string()),
                Argument::Text("fail".to_string()),
                Argument::Properties(vec![Property { name: "Description".to_string(), value: PropertyValue::Text("wg probe".to_string()) }]),
                Argument::NoAuxUnits,
            ],
        },
    );
    let mut child = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start a child to adopt");
    let pid = i32::try_from(child.id()).expect("pid fits");
    let scope = format!("wg-probe-{pid}.scope");
    let cases: Vec<(&str, Vec<Property>)> = vec![
        ("scope with Slice only", vec![Property { name: "Slice".to_string(), value: PropertyValue::Text(slice.to_string()) }]),
        ("scope with PIDs only", vec![Property { name: "PIDs".to_string(), value: PropertyValue::Pids(vec![u32::try_from(pid).unwrap_or(0)]) }]),
        (
            "scope with Slice and PIDs",
            vec![
                Property { name: "Slice".to_string(), value: PropertyValue::Text(slice.to_string()) },
                Property { name: "PIDs".to_string(), value: PropertyValue::Pids(vec![u32::try_from(pid).unwrap_or(0)]) },
            ],
        ),
        (
            "scope with Slice, PIDs, Description",
            vec![
                Property { name: "Slice".to_string(), value: PropertyValue::Text(slice.to_string()) },
                Property { name: "PIDs".to_string(), value: PropertyValue::Pids(vec![u32::try_from(pid).unwrap_or(0)]) },
                Property { name: "Description".to_string(), value: PropertyValue::Text("wg probe scope".to_string()) },
            ],
        ),
        (
            "scope with Slice, PIDs, Description, Delegate, CollectMode",
            vec![
                Property { name: "Slice".to_string(), value: PropertyValue::Text(slice.to_string()) },
                Property { name: "PIDs".to_string(), value: PropertyValue::Pids(vec![u32::try_from(pid).unwrap_or(0)]) },
                Property { name: "Description".to_string(), value: PropertyValue::Text("wg probe scope".to_string()) },
                Property { name: "Delegate".to_string(), value: PropertyValue::Flag(true) },
                Property { name: "CollectMode".to_string(), value: PropertyValue::Text("inactive-or-failed".to_string()) },
            ],
        ),
        (
            "scope with a 64-bit number property",
            vec![
                Property { name: "Slice".to_string(), value: PropertyValue::Text(slice.to_string()) },
                Property { name: "PIDs".to_string(), value: PropertyValue::Pids(vec![u32::try_from(pid).unwrap_or(0)]) },
                Property { name: "MemoryMax".to_string(), value: PropertyValue::Number(1_073_741_824) },
            ],
        ),
    ];
    for (index, (what, properties)) in cases.iter().enumerate() {
        probe(
            &client,
            &endpoint,
            what,
            &MethodCall {
                interface: MANAGER_INTERFACE,
                member: "StartTransientUnit",
                arguments: vec![Argument::Text(format!("{index}-{scope}")), Argument::Text("fail".to_string()), Argument::Properties(properties.clone()), Argument::NoAuxUnits],
            },
        );
    }
    let _killed = child.kill();
    let _reaped = child.wait();
    let _stopped = std::process::Command::new("systemctl").args(["--user", "stop", slice]).status();
}

/// Dumps what the manager sends on a direct connection,
/// one message at a time,
/// so the reader is checked against real bytes rather than against the writer.
#[test]
#[ignore = "talks to the real user manager"]
fn the_manager_sends_signals_before_its_reply() {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    let endpoint = ManagerEndpoint::resolve(uid).expect("the user's manager answers");
    let mut stream = UnixStream::connect(&endpoint.socket).expect("connect");
    let _timeout = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    stream.write_all(format!("\0AUTH EXTERNAL {}\r\n", crate::dbus_login::external_token(uid)).as_bytes()).expect("auth");
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    while !line.ends_with(b"\r\n") {
        stream.read_exact(&mut byte).expect("login line");
        line.push(byte[0]);
    }
    stream.write_all(b"BEGIN\r\n").expect("begin");
    let call = crate::dbus_wire::encode_call(
        7,
        crate::dbus_wire::MANAGER_PATH,
        MANAGER_INTERFACE,
        "GetUnit",
        None,
        &[Argument::Text("app.slice".to_string())],
    );
    stream.write_all(&call).expect("send");
    let mut buffer: Vec<u8> = Vec::new();
    let mut seen: usize = 0;
    let mut chunk = [0u8; 65536];
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(6) && seen < 8 {
        let count = match stream.read(&mut chunk) {
            Ok(count) => count,
            Err(error) => {
                println!("read error {error}");
                return;
            }
        };
        if count == 0 {
            println!("closed");
            return;
        }
        buffer.extend_from_slice(chunk.get(..count).unwrap_or_default());
        loop {
            let Some(length) = crate::dbus_wire::message_length(&buffer) else { break };
            if buffer.len() < length {
                break;
            }
            seen = seen.saturating_add(1);
            match crate::dbus_wire::decode_message(buffer.get(..length).unwrap_or_default()) {
                Ok(message) => println!("message {seen}: length {length} kind {} serial {:?} member {:?} signature {:?}", message.kind, message.reply_serial, message.member, message.signature),
                Err(error) => println!("message {seen}: length {length} refused: {error:?}"),
            }
            buffer.drain(..length);
        }
    }
    println!("done after {seen} messages, {} bytes left", buffer.len());
}

/// Reproduces the sequence the policy makes:
/// one call,
/// then another.
#[test]
#[ignore = "talks to the real user manager"]
fn a_call_after_a_big_reply_still_gets_its_own_reply() {
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    let endpoint = ManagerEndpoint::resolve(uid).expect("the user's manager answers");
    let client = ManagerClient::new();
    let first_started = std::time::Instant::now();
    let first = client.call(&endpoint, &MethodCall { interface: MANAGER_INTERFACE, member: "ListUnits", arguments: vec![] });
    println!("ListUnits {:?} after {:?}", first.map(|reply| return reply.kind), first_started.elapsed());
    let second_started = std::time::Instant::now();
    let second = client.call(&endpoint, &MethodCall { interface: MANAGER_INTERFACE, member: "GetUnit", arguments: vec![Argument::Text("app.slice".to_string())] });
    println!("GetUnit {:?} after {:?}", second.map(|reply| return (reply.kind, reply.reply_serial, reply.signature)), second_started.elapsed());
    let child_started = std::time::Instant::now();
    let mut second_child = std::process::Command::new("/usr/bin/sleep").arg("5").spawn().expect("child");
    let pid = i32::try_from(second_child.id()).expect("pid");
    let _killed = second_child.kill();
    let _reaped = second_child.wait();
    println!("(child {pid} ended) after {:?}", child_started.elapsed());
    let third_started = std::time::Instant::now();
    let third = client.call(&endpoint, &MethodCall { interface: MANAGER_INTERFACE, member: "GetUnit", arguments: vec![Argument::Text("app.slice".to_string())] });
    println!("GetUnit again {:?} after {:?}", third.map(|reply| return (reply.kind, reply.reply_serial)), third_started.elapsed());
}
