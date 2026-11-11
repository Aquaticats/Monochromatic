//! The bytes of what a call carries:
//! the property values and arguments of a method call,
//! their signatures,
//! and the byte layout D-Bus gives each of them.
//! Kept apart from the message header so each file holds one piece of the format.

/// The alignment of a struct,
/// a variant
/// and an unsigned 64-bit number.
use super::EIGHT;
/// The alignment of a string,
/// an array length
/// and a 32-bit number.
use super::FOUR;
/// The sixteen fixed header bytes before the header fields.
use super::FIXED_HEADER;
/// The header field that holds the body's signature.
use super::FIELD_SIGNATURE;
/// The header field that holds the destination of a call.
use super::FIELD_DESTINATION;
/// The header field that holds the interface.
use super::FIELD_INTERFACE;
/// The header field that holds the member name.
use super::FIELD_MEMBER;
/// The header field that holds the object path.
use super::FIELD_PATH;
/// The type of a method call.
use super::METHOD_CALL;
/// The first byte of a little-endian message.
use super::ENDIAN_LITTLE;

/// Where an array's length field is,
/// and where its content starts.
///
/// The two are not adjacent when the element alignment is wider than the length field:
/// the content starts after the padding that aligns its first element,
/// and the length counts only the content.
pub(crate) struct ArrayStart {
    /// Offset of the four length bytes.
    pub(crate) length_at: usize,
    /// Offset of the first content byte.
    pub(crate) content_at: usize,
}

/// The value of one property of a unit,
/// typed as the manager takes it:
/// a 64-bit number (`t`),
/// a string (`s`),
/// a boolean (`b`),
/// or an array of device path and number (`a(st)`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyValue {
    /// A 64-bit number,
    /// such as a CPU quota in microseconds per second.
    Number(
        /// The number.
        u64,
    ),
    /// A string,
    /// such as a unit name or a description.
    Text(
        /// The text.
        String,
    ),
    /// A boolean,
    /// such as whether the scope is delegated.
    Flag(
        /// The boolean.
        bool,
    ),
    /// A list of 32-bit numbers,
    /// such as the pids a transient unit starts with.
    Pids(
        /// The pids.
        Vec<u32>,
    ),
    /// One entry per block device,
    /// keyed by the device path the manager takes.
    Devices(
        /// The device path and its limit.
        Vec<(String, u64)>,
    ),
}

/// One property of a unit,
/// as a variadic `(sv)` entry of `StartTransientUnit` or `SetUnitProperties`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    /// The property's name.
    pub name: String,
    /// The property's value.
    pub value: PropertyValue,
}

/// One argument of a call's body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Argument {
    /// A string.
    Text(
        /// The text.
        String,
    ),
    /// A boolean.
    Flag(
        /// The boolean.
        bool,
    ),
    /// A list of 32-bit numbers,
    /// such as the pids of one adoption.
    Numbers(
        /// The numbers.
        Vec<u32>,
    ),
    /// A list of unit properties.
    Properties(
        /// The properties.
        Vec<Property>,
    ),
    /// The auxiliary units of `StartTransientUnit`,
    /// which this watcher never passes any of.
    NoAuxUnits,
}

/// The signature of one property value.
fn property_signature(value: &PropertyValue) -> &'static str {
    return match value {
        PropertyValue::Number(_) => "t",
        PropertyValue::Text(_) => "s",
        PropertyValue::Flag(_) => "b",
        PropertyValue::Pids(_) => "au",
        PropertyValue::Devices(_) => "a(st)",
    };
}

/// The signature of one argument.
fn argument_signature(argument: &Argument) -> &'static str {
    return match argument {
        Argument::Text(_) => "s",
        Argument::Flag(_) => "b",
        Argument::Numbers(_) => "au",
        Argument::Properties(_) => "a(sv)",
        Argument::NoAuxUnits => "a(sa(sv))",
    };
}

/// The signature of a whole body.
pub fn body_signature(arguments: &[Argument]) -> String {
    let mut signature = String::new();
    for argument in arguments {
        signature.push_str(argument_signature(argument));
    }
    return signature;
}

/// Writes D-Bus values at their aligned offsets.
pub(crate) struct Writer {
    /// The bytes written so far.
    pub(crate) bytes: Vec<u8>,
    /// Where these bytes start in the message,
    /// because every alignment is counted from the message's first byte.
    pub(crate) start: usize,
}

/// Writing values.
impl Writer {
    /// Pads until the next byte lies on `boundary`.
    pub(crate) fn pad(&mut self, boundary: usize) {
        while (self.start + self.bytes.len()) % boundary != 0 {
            self.bytes.push(0);
        }
    }

    /// Writes a 32-bit number.
    pub(crate) fn uint32(&mut self, value: u32) {
        self.pad(FOUR);
        self.bytes.extend(value.to_le_bytes());
    }

    /// Writes a 64-bit number.
    pub(crate) fn number(&mut self, value: u64) {
        self.pad(EIGHT);
        self.bytes.extend(value.to_le_bytes());
    }

    /// Writes a string:
    /// its length,
    /// its bytes,
    /// and the end byte.
    pub(crate) fn text(&mut self, value: &str) {
        let length = u32::try_from(value.len()).unwrap_or(u32::MAX);
        self.uint32(length);
        self.bytes.extend(value.as_bytes());
        self.bytes.push(0);
    }

    /// Writes a signature:
    /// its length,
    /// its bytes,
    /// and the end byte.
    pub(crate) fn signature(&mut self, value: &str) {
        let length = u8::try_from(value.len()).unwrap_or(u8::MAX);
        self.bytes.push(length);
        self.bytes.extend(value.as_bytes());
        self.bytes.push(0);
    }

    /// Writes a boolean value.
    fn flag(&mut self, value: bool) {
        self.pad(FOUR);
        self.bytes.extend(u32::from(value).to_le_bytes());
    }

    /// Starts an array whose element alignment is `element_alignment`,
    /// and returns where its length field is and where its content starts,
    /// so `finish_array` fills the length in once the elements are written.
    ///
    /// The length a D-Bus array field carries is the length of its *content*:
    /// the specification puts the padding that aligns the first element between the length and the content,
    /// and the length does not count it.
    fn start_array(&mut self, element_alignment: usize) -> ArrayStart {
        self.pad(FOUR);
        let length_at = self.bytes.len();
        self.bytes.extend([0; FOUR]);
        self.pad(element_alignment);
        return ArrayStart { length_at, content_at: self.bytes.len() };
    }

    /// Fills in the length of the array that started at `start`,
    /// which is the bytes written after its alignment padding.
    fn finish_array(&mut self, start: ArrayStart) {
        let content_length = u32::try_from(self.bytes.len().saturating_sub(start.content_at)).unwrap_or(u32::MAX);
        self.bytes[start.length_at..start.length_at.saturating_add(FOUR)].copy_from_slice(&content_length.to_le_bytes());
    }

    /// Writes one property value under its own signature,
    /// which the caller wrote just before.
    fn property_value(&mut self, value: &PropertyValue) {
        match value {
            PropertyValue::Number(number) => self.number(*number),
            PropertyValue::Text(text) => self.text(text),
            PropertyValue::Flag(flag) => self.flag(*flag),
            PropertyValue::Pids(pids) => {
                let length_at = self.start_array(FOUR);
                for pid in pids {
                    self.uint32(*pid);
                }
                self.finish_array(length_at);
            }
            PropertyValue::Devices(devices) => {
                let length_at = self.start_array(EIGHT);
                for (path, number) in devices {
                    // Every element of an array starts on the element's own boundary.
                    self.pad(EIGHT);
                    self.text(path);
                    self.number(*number);
                }
                self.finish_array(length_at);
            }
        }
    }

    /// Writes one argument.
    pub(crate) fn argument(&mut self, argument: &Argument) {
        match argument {
            Argument::Text(text) => self.text(text),
            Argument::Flag(flag) => self.flag(*flag),
            Argument::Numbers(numbers) => {
                let length_at = self.start_array(FOUR);
                for number in numbers {
                    self.uint32(*number);
                }
                self.finish_array(length_at);
            }
            Argument::Properties(properties) => {
                let length_at = self.start_array(EIGHT);
                for property in properties {
                    // Every element of an array starts on the element's own boundary,
                    // which a variant's value may not have ended on.
                    self.pad(EIGHT);
                    self.text(&property.name);
                    self.signature(property_signature(&property.value));
                    self.property_value(&property.value);
                }
                self.finish_array(length_at);
            }
            Argument::NoAuxUnits => {
                let length_at = self.start_array(EIGHT);
                self.finish_array(length_at);
            }
        }
    }
}


/// Writes one header field:
/// its code
/// and a variant holding the value.
pub(crate) fn header_field(writer: &mut Writer, code: u8, value_signature: &str, value: &str) {
    writer.pad(EIGHT);
    writer.bytes.push(code);
    writer.signature(value_signature);
    if value_signature == "g" {
        writer.signature(value);
    } else {
        writer.text(value);
    }
}

/// The bytes of a method call,
/// with `destination` set only when the call goes over a bus.
pub fn encode_call(serial: u32, path: &str, interface: &str, member: &str, destination: Option<&str>, arguments: &[Argument]) -> Vec<u8> {
    let mut fields = Writer { bytes: vec![0; FIXED_HEADER], start: 0 };
    header_field(&mut fields, FIELD_PATH, "o", path);
    if let Some(name) = destination {
        header_field(&mut fields, FIELD_DESTINATION, "s", name);
    }
    header_field(&mut fields, FIELD_INTERFACE, "s", interface);
    header_field(&mut fields, FIELD_MEMBER, "s", member);
    let signature = body_signature(arguments);
    if !signature.is_empty() {
        header_field(&mut fields, FIELD_SIGNATURE, "g", &signature);
    }
    let fields_length = u32::try_from(fields.bytes.len().saturating_sub(FIXED_HEADER)).unwrap_or(u32::MAX);
    fields.pad(EIGHT);
    let body_start = fields.bytes.len();
    let mut body = Writer { bytes: Vec::new(), start: body_start };
    for argument in arguments {
        body.argument(argument);
    }
    let mut message = fields.bytes;
    message[0] = ENDIAN_LITTLE;
    message[1] = METHOD_CALL;
    message[2] = 0;
    message[3] = 1;
    message[4..8].copy_from_slice(&u32::try_from(body.bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    message[8..12].copy_from_slice(&serial.to_le_bytes());
    message[12..16].copy_from_slice(&fields_length.to_le_bytes());
    message.extend(body.bytes);
    return message;
}
