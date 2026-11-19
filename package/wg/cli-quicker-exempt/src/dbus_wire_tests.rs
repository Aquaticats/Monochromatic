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

/// A reader of one body,
/// written from the specification's layout rules,
/// so a value's bytes are checked against the format and not against the writer.
struct SpecReader<'a> {
    /// The bytes being read.
    bytes: &'a [u8],
    /// Where the reader is.
    at: usize,
}

/// Reading,
/// by the specification.
impl<'a> SpecReader<'a> {
    /// Starts at the first byte of a body.
    fn new(bytes: &'a [u8]) -> SpecReader<'a> {
        return SpecReader { bytes, at: 0 };
    }

    /// Skips the padding that puts the next value on `boundary`,
    /// and answers the padding's length.
    fn align(&mut self, boundary: usize) -> usize {
        let padding = (boundary - (self.at % boundary)) % boundary;
        self.at += padding;
        return padding;
    }

    /// Reads a 32-bit number,
    /// aligned to four.
    fn u32(&mut self) -> u32 {
        let _padding = self.align(4);
        let value = u32::from_le_bytes(self.bytes[self.at..self.at + 4].try_into().expect("four bytes"));
        self.at += 4;
        return value;
    }

    /// Reads a boolean,
    /// aligned to four.
    fn flag(&mut self) -> bool {
        return self.u32() != 0;
    }

    /// Reads a string:
    /// its length,
    /// its bytes and the end byte.
    fn string(&mut self) -> String {
        let length = usize::try_from(self.u32()).expect("length fits");
        let text = String::from_utf8_lossy(&self.bytes[self.at..self.at + length]).into_owned();
        self.at += length;
        assert_eq!(self.bytes[self.at], 0, "a string ends with a zero byte");
        self.at += 1;
        return text;
    }

    /// Reads a signature:
    /// its length,
    /// its bytes and the end byte.
    fn signature(&mut self) -> String {
        let length = usize::from(self.bytes[self.at]);
        self.at += 1;
        let text = String::from_utf8_lossy(&self.bytes[self.at..self.at + length]).into_owned();
        self.at += length;
        assert_eq!(self.bytes[self.at], 0, "a signature ends with a zero byte");
        self.at += 1;
        return text;
    }

    /// Reads an unsigned 64-bit number,
    /// aligned to eight.
    fn u64(&mut self) -> u64 {
        let _padding = self.align(8);
        let value = u64::from_le_bytes(self.bytes[self.at..self.at + 8].try_into().expect("eight bytes"));
        self.at += 8;
        return value;
    }

    /// Reads an array whose element alignment is `element_alignment`,
    /// and answers its length and where its content ends.
    ///
    /// The specification puts the length first,
    /// then the padding that aligns the first element,
    /// then the content,
    /// and the length counts the content only.
    fn array(&mut self, element_alignment: usize) -> (u32, usize) {
        let length = self.u32();
        let _padding = self.align(element_alignment);
        let content_start = self.at;
        let content_end = content_start + usize::try_from(length).expect("length fits");
        return (length, content_end);
    }
}

/// An array of eight-byte-aligned elements declares its content's length and not its padding,
/// read against the specification's layout.
///
/// The body is that of `SetUnitProperties`:
/// the unit's name,
/// the runtime flag and the properties,
/// and the property here carries an `a(st)` device list,
/// whose elements start on eight-byte boundaries.
///
/// The scope name's length moves the array's length field between the two four-byte positions of an
/// eight-byte boundary,
/// so every leaf length is checked and not one that happens to align.
#[test]
fn an_array_of_aligned_elements_declares_its_content_length() {
    for leaf_length in 2..=13 {
    let bytes = encode_call(
        7,
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "SetUnitProperties",
        None,
        &[
            Argument::Text(format!("wg-quicker-update-app.slice-{}.scope", "x".repeat(leaf_length - 2))),
            Argument::Flag(true),
            Argument::Properties(vec![
                Property { name: "IOReadBandwidthMax".to_string(), value: PropertyValue::Devices(vec![("/dev/block/8:0".to_string(), 1_000)]) },
            ]),
        ],
    );
    let message = decode_message(&bytes).expect("readable");
    assert_eq!(message.signature, "sba(sv)");
    let mut reader = SpecReader::new(message.body());
    let _name = reader.string();
    assert!(reader.flag());
    let (_properties_length, properties_end) = reader.array(8);
    // One `(sv)` element, on its own eight-byte boundary.
    let _padding = reader.align(8);
    assert_eq!(reader.string(), "IOReadBandwidthMax");
    assert_eq!(reader.signature(), "a(st)");
    let (devices_length, devices_end) = reader.array(8);
    let devices_start = reader.at;
    assert_eq!(usize::try_from(devices_length).expect("length fits"), devices_end - devices_start, "the device array length is its content's length");
    let _element_padding = reader.align(8);
    assert_eq!(reader.string(), "/dev/block/8:0");
    assert_eq!(reader.u64(), 1_000);
    assert_eq!(reader.at, devices_end, "the device array ends where its length says");
    assert_eq!(reader.at, properties_end, "the property array ends where its length says");
    assert_eq!(reader.at, message.body().len(), "the body ends after its last value, for leaf length {leaf_length}");
    }
}
