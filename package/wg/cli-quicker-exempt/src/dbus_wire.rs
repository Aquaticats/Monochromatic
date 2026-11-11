//! The D-Bus wire format of the method calls the watcher makes,
//! and the reader of the answers.
//! The watcher reaches a user's service manager over that manager's own private socket
//! (`$XDG_RUNTIME_DIR/systemd/private`),
//! and a private socket is not a bus:
//! no daemon answers there,
//! the manager itself does,
//! and no process can take a name that would redirect the call.
//! `busctl` cannot be used for this,
//! because it is a bus client and asks the bus daemon first
//! (`Call failed: Invalid request descriptor` against a private socket).
//! So the call is written here,
//! as the specification lays it out,
//! and the reader takes the reply apart.
//! Nothing here touches a socket,
//! so the layout is testable by bytes alone.

/// Everything a call carries,
/// in the marshaller of its own file.
#[path = "dbus_wire_marshaller.rs"]
pub mod marshaller;
/// The arguments and properties a call carries.
pub use marshaller::{Argument, Property, PropertyValue, encode_call};
/// The signature a body of these arguments declares.
#[cfg(test)]
pub use marshaller::body_signature;

/// The messages a control's fake manager and fake bus answer with.
#[cfg(test)]
#[path = "dbus_wire_controls.rs"]
pub mod controls;
/// The messages a control's fakes build.
#[cfg(test)]
pub use controls::{encode_error, encode_number_return, encode_reply, encode_signal, encode_text_return};

/// Start of a little-endian message.
pub const ENDIAN_LITTLE: u8 = b'l';
/// A message that asks the receiver to do something.
pub const METHOD_CALL: u8 = 1;
/// A message that answers a call.
pub const METHOD_RETURN: u8 = 2;
/// A message that answers a call with an error name and text.
pub const ERROR: u8 = 3;
/// A message the receiver sends on its own.
pub const SIGNAL: u8 = 4;
/// Header field holding the object path.
pub const FIELD_PATH: u8 = 1;
/// Header field holding the interface.
pub const FIELD_INTERFACE: u8 = 2;
/// Header field holding the member name.
pub const FIELD_MEMBER: u8 = 3;
/// Header field holding the error name of an error reply.
pub const FIELD_ERROR_NAME: u8 = 4;
/// Header field holding the serial of the call a reply answers.
pub const FIELD_REPLY_SERIAL: u8 = 5;
/// Header field holding the destination of a call.
pub const FIELD_DESTINATION: u8 = 6;
/// Header field holding the body's signature.
pub const FIELD_SIGNATURE: u8 = 8;
/// The browser and the bus daemon answer on this object path.
pub const BUS_PATH: &str = "/org/freedesktop/DBus";
/// The bus daemon's own interface.
pub const BUS_INTERFACE: &str = "org.freedesktop.DBus";
/// The bus daemon's bus name.
pub const BUS_NAME: &str = "org.freedesktop.DBus";
/// The manager's object path.
pub const MANAGER_PATH: &str = "/org/freedesktop/systemd1";
/// The manager's interface.
pub const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";
/// The sixteen fixed header bytes before the header fields.
const FIXED_HEADER: usize = 16;
/// The alignment of a struct,
/// a variant
/// and an unsigned 64-bit number.
const EIGHT: usize = 8;
/// The alignment of a string,
/// an array length
/// and a 32-bit number.
const FOUR: usize = 4;

/// Why a message could not be written or read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WireError {
    /// The bytes are not a message this client understands,
    /// named by what is wrong.
    Malformed(
        /// What is wrong with the bytes.
        String,
    ),
}

/// One method call of the user's service manager.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodCall {
    /// The interface that holds the method.
    pub interface: &'static str,
    /// The method.
    pub member: &'static str,
    /// Its arguments,
    /// whose order fixes the body's signature.
    pub arguments: Vec<Argument>,
}

/// Reads four bytes as a little-endian number.
fn number_at(bytes: &[u8], at: usize) -> Option<u32> {
    let slice = bytes.get(at..at.checked_add(FOUR)?)?;
    return Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]));
}

/// Rounds `offset` up to `boundary`.
pub fn aligned(offset: usize, boundary: usize) -> usize {
    let Some(sum) = offset.checked_add(boundary - 1) else {
        return usize::MAX;
    };
    return sum / boundary * boundary;
}

/// Reads a string at `at`,
/// with the offset just after it,
/// or nothing when it is cut short or its end byte is missing.
fn read_text(bytes: &[u8], at: usize) -> Option<(String, usize)> {
    let length = usize::try_from(number_at(bytes, at)?).ok()?;
    let start = at.checked_add(FOUR)?;
    let end = start.checked_add(length)?;
    if *bytes.get(end)? != 0 {
        return None;
    }
    return Some((String::from_utf8_lossy(bytes.get(start..end)?).into_owned(), end.saturating_add(1)));
}

/// One message read off the socket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    /// Its type,
    /// one of the constants above.
    pub kind: u8,
    /// The message's own serial.
    pub serial: u32,
    /// The serial of the call a reply answers.
    pub reply_serial: Option<u32>,
    /// The member of the call,
    /// when the message is a call.
    pub member: Option<String>,
    /// The error name of an error reply.
    pub error_name: Option<String>,
    /// The body's signature.
    pub signature: String,
    /// Where the body starts in the message.
    body_start: usize,
    /// The message bytes.
    bytes: Vec<u8>,
}

/// The size of the whole message at the start of `bytes`,
/// or nothing when the header is cut short.
pub fn message_length(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < FIXED_HEADER || bytes[0] != ENDIAN_LITTLE {
        return None;
    }
    let body_length = usize::try_from(number_at(bytes, 4)?).ok()?;
    let fields_length = usize::try_from(number_at(bytes, 12)?).ok()?;
    let body_start = aligned(FIXED_HEADER.checked_add(fields_length)?, EIGHT);
    return body_start.checked_add(body_length);
}

/// Reads one message.
pub fn decode_message(bytes: &[u8]) -> Result<Message, WireError> {
    let Some(total) = message_length(bytes) else {
        return Err(WireError::Malformed("the message is not a little-endian header".to_string()));
    };
    if bytes.len() < total {
        return Err(WireError::Malformed("the message is shorter than its header says".to_string()));
    }
    let fields_end = usize::try_from(number_at(bytes, 12).unwrap_or(0)).unwrap_or(usize::MAX).saturating_add(FIXED_HEADER);
    let body_start = aligned(fields_end, EIGHT);
    let mut message = Message {
        kind: bytes[1],
        serial: number_at(bytes, 8).unwrap_or(0),
        reply_serial: None,
        member: None,
        error_name: None,
        signature: String::new(),
        body_start,
        bytes: bytes.get(..total).unwrap_or_default().to_vec(),
    };
    let mut at = FIXED_HEADER;
    while at < fields_end {
        at = aligned(at, EIGHT);
        let Some(&code) = message.bytes.get(at) else { break };
        let Some(&signature_length) = message.bytes.get(at.saturating_add(1)) else { break };
        let kind_at = at.saturating_add(2);
        let Some(&value_kind) = message.bytes.get(kind_at) else { break };
        let value_at = kind_at.saturating_add(usize::from(signature_length)).saturating_add(1);
        match value_kind {
            b's' | b'o' => {
                let Some((text, next)) = read_text(&message.bytes, aligned(value_at, FOUR)) else { break };
                if code == FIELD_ERROR_NAME {
                    message.error_name = Some(text.clone());
                }
                if code == FIELD_MEMBER {
                    message.member = Some(text);
                }
                at = next;
            }
            b'g' => {
                let length = usize::from(*message.bytes.get(value_at).unwrap_or(&0));
                let start = value_at.saturating_add(1);
                if code == FIELD_SIGNATURE && message.bytes.get(start.saturating_add(length)) == Some(&0) {
                    message.signature = String::from_utf8_lossy(message.bytes.get(start..start.saturating_add(length)).unwrap_or_default()).into_owned();
                }
                at = start.saturating_add(length).saturating_add(1);
            }
            b'u' => {
                let at_number = aligned(value_at, FOUR);
                if code == FIELD_REPLY_SERIAL {
                    message.reply_serial = number_at(&message.bytes, at_number);
                }
                at = at_number.saturating_add(FOUR);
            }
            _ => break,
        }
    }
    return Ok(message);
}

/// Reading a message's body.
impl Message {
    /// The first string of the body,
    /// which an error reply uses for its text.
    pub fn first_text(&self) -> Option<String> {
        if !self.signature.starts_with('s') && !self.signature.starts_with('o') {
            return None;
        }
        let read = read_text(&self.bytes, aligned(self.body_start, FOUR))?;
        return Some(read.0);
    }

    /// The first 32-bit number of the body.
    pub fn first_number(&self) -> Option<u32> {
        if !self.signature.starts_with('u') {
            return None;
        }
        return number_at(&self.bytes, aligned(self.body_start, FOUR));
    }

}
