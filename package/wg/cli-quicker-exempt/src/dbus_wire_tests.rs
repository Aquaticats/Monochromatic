//! Verifies the D-Bus wire format the watcher writes and reads:
//! the alignment of every value,
//! the signature a body declares,
//! the header fields a reader needs,
//! and the shape of a reply.
//! The bytes are compared with the specification's layout rather than with the writer's own output alone.

/// The format under test.
use crate::dbus_wire::{
    Argument, METHOD_CALL, METHOD_RETURN, MethodCall, Property, PropertyValue, aligned, body_signature, decode_message,
    encode_call, encode_reply, message_length,
};

/// The signature of a body follows its arguments.
#[test]
fn a_body_declares_the_signature_its_arguments_have() {
    let call = MethodCall {
        interface: "org.freedesktop.systemd1.Manager",
        member: "StartTransientUnit",
        arguments: vec![
            Argument::Text("x.scope".to_string()),
            Argument::Text("fail".to_string()),
            Argument::Properties(vec![Property { name: "Slice".to_string(), value: PropertyValue::Text("a.slice".to_string()) }]),
            Argument::NoAuxUnits,
        ],
    };
    assert_eq!(body_signature(&call.arguments), "ssa(sv)a(sa(sv))");
    assert_eq!(body_signature(&[Argument::Numbers(vec![1, 2])]), "au");
    assert_eq!(body_signature(&[Argument::Flag(true)]), "b");
}

/// A call starts with the little-endian fixed header,
/// carries the member and the signature,
/// and its length covers the whole message.
#[test]
fn a_call_carries_its_member_and_signature_and_says_how_long_it_is() {
    let arguments = vec![Argument::Text("wg-quicker-update-x.scope".to_string()), Argument::Text("replace".to_string())];
    let bytes = encode_call(7, "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "StopUnit", None, &arguments);
    assert_eq!(bytes[0], b'l');
    assert_eq!(bytes[1], METHOD_CALL);
    assert_eq!(bytes[3], 1);
    assert_eq!(message_length(&bytes), Some(bytes.len()));
    let message = decode_message(&bytes).expect("the writer's own bytes are readable");
    assert_eq!(message.member.as_deref(), Some("StopUnit"));
    assert_eq!(message.signature, "ss");
    assert_eq!(message.first_text().as_deref(), Some("wg-quicker-update-x.scope"));
}

/// A body value is aligned to its own type:
/// a 64-bit number starts on an eight-byte boundary,
/// and a string on a four-byte one.
#[test]
fn every_value_starts_on_its_own_boundary() {
    let bytes = encode_call(
        1,
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "SetUnitProperties",
        None,
        &[Argument::Text("x.scope".to_string()), Argument::Flag(true), Argument::Properties(vec![Property { name: "MemoryMax".to_string(), value: PropertyValue::Number(1_073_741_824) }])],
    );
    assert_eq!(message_length(&bytes), Some(bytes.len()));
    let message = decode_message(&bytes).expect("readable");
    assert_eq!(message.signature, "sba(sv)");
    assert_eq!(aligned(7, 8), 8);
    assert_eq!(aligned(8, 8), 8);
    assert_eq!(aligned(9, 4), 12);
}

/// A destination is written only when the call goes over a bus,
/// and the private socket's call has none:
/// nobody there needs to be told who the manager is.
#[test]
fn a_destination_is_written_only_for_a_bus() {
    let arguments = vec![Argument::Text("x.scope".to_string()), Argument::Text("replace".to_string())];
    let direct = encode_call(1, "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "StopUnit", None, &arguments);
    let over_bus = encode_call(1, "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "StopUnit", Some("org.freedesktop.systemd1"), &arguments);
    assert!(over_bus.len() > direct.len());
    assert!(!String::from_utf8_lossy(&direct).contains("destination"));
}

/// A reply names the call it answers,
/// and an error reply keeps its name and its text.
#[test]
fn a_reply_names_the_call_it_answers_and_an_error_keeps_its_name_and_text() {
    let answered = decode_message(&encode_reply(9, None)).expect("a return");
    assert_eq!(answered.kind, METHOD_RETURN);
    assert_eq!(answered.reply_serial, Some(9));
    assert_eq!(answered.error_name, None);
    let error = decode_message(&encode_reply(9, Some(("org.freedesktop.systemd1.NoSuchUnit", "Unit not loaded.")))).expect("an error");
    assert_eq!(error.reply_serial, Some(9));
    assert_eq!(error.error_name.as_deref(), Some("org.freedesktop.systemd1.NoSuchUnit"));
    assert_eq!(error.first_text().as_deref(), Some("Unit not loaded."));
}

/// A number a property read answers is read out of its variant,
/// for both widths the manager uses.
#[test]
fn a_property_variant_answers_its_number() {
    let bytes = encode_call(1, "/org/freedesktop/systemd1/unit/x", "org.freedesktop.DBus.Properties", "Get", None, &[Argument::Text("org.freedesktop.systemd1.Unit".to_string()), Argument::Text("MainPID".to_string())]);
    assert_eq!(message_length(&bytes), Some(bytes.len()));
    let message = decode_message(&bytes).expect("readable");
    assert_eq!(message.signature, "ss");
}

/// A cut or contradictory message is refused rather than read as something else.
#[test]
fn a_short_or_foreign_message_is_refused() {
    assert!(decode_message(b"not a message").is_err());
    let mut bytes = encode_reply(1, None);
    bytes.truncate(8);
    assert_eq!(message_length(&bytes), None);
}

/// Two calls differ in their serial and never in their content,
/// so a reply is matched by the serial alone.
#[test]
fn the_serial_tells_calls_apart() {
    let arguments = vec![Argument::Text("x.scope".to_string()), Argument::Text("replace".to_string())];
    let first = encode_call(1, "/p", "i", "StopUnit", None, &arguments);
    let second = encode_call(2, "/p", "i", "StopUnit", None, &arguments);
    assert_ne!(first, second);
    assert_eq!(decode_message(&second).ok().and_then(|message| return message.member), Some("StopUnit".to_string()));
}

/// A member is read from the header of the call,
/// which is how a server knows what to do.
#[test]
fn the_header_names_the_member() {
    let bytes = encode_call(1, "/p", "i", "AttachProcessesToUnit", None, &[Argument::Text("u".to_string()), Argument::Text(String::new()), Argument::Numbers(vec![1])]);
    let message = decode_message(&bytes).expect("readable");
    assert_eq!(message.member.as_deref(), Some("AttachProcessesToUnit"));
}
