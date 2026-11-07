//! What:
//!  The D-Bus wire format the client speaks to the user manager: how a method call is written (header fields, strings,
//!  alignment) and how a reply is read (kind, serial, error name, first text), with the error type of a call.
//! Why:
//!  The connection code in `systemd_dbus.rs` sends and receives bytes through it; the format needs no socket, so it stays
//!  testable by bytes alone and is a file `systemd_dbus.rs` uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = encodeCall(serial, target, args); const reply = decodeReply(readMessage(socket));
//! ```

/// Message type of a method call.
pub(crate) const METHOD_CALL: u8 = 1;

/// Message type of a successful reply; only the controls' fake bus and the reader's controls use it now.
#[cfg(test)]
pub(crate) const METHOD_RETURN: u8 = 2;

/// Message type of an error reply; only the controls' fake bus and the reader's controls use it now.
#[cfg(test)]
pub(crate) const ERROR: u8 = 3;

/// Header field code of the number of the call a reply answers.
pub(crate) const FIELD_REPLY_SERIAL: u8 = 5;

/// Header field codes.
pub(crate) const FIELD_PATH: u8 = 1;
pub(crate) const FIELD_INTERFACE: u8 = 2;
pub(crate) const FIELD_MEMBER: u8 = 3;
pub(crate) const FIELD_ERROR_NAME: u8 = 4;
pub(crate) const FIELD_DESTINATION: u8 = 6;
pub(crate) const FIELD_SIGNATURE: u8 = 8;

/// Why a call did not succeed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallError {
    /// The socket could not be reached or the connection broke.
    Connection(String),
    /// The manager refused the authentication.
    Authentication(String),
    /// The bus did not answer within the time the exchange was given, which says nothing about whether the manager is there.
    TimedOut,
    /// The manager answered with an error.
    Manager {
        /// The error's name, for example `org.freedesktop.systemd1.NoSuchUnit`.
        name: String,
        /// The error's text, when it gave one.
        text: String,
    },
    /// The reply was not a message this client reads.
    Malformed(String),
}

/// Appends values in D-Bus little-endian wire form.
pub(crate) struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn align(&mut self, boundary: usize) {
        while self.bytes.len() % boundary != 0 {
            self.bytes.push(0);
        }
    }

    fn unsigned(&mut self, value: u32) {
        self.align(4);
        self.bytes.extend(value.to_le_bytes());
    }

    fn text(&mut self, value: &str) {
        self.unsigned(u32::try_from(value.len()).unwrap_or(u32::MAX));
        self.bytes.extend(value.as_bytes());
        self.bytes.push(0);
    }

    fn signature(&mut self, value: &str) {
        self.bytes.push(u8::try_from(value.len()).unwrap_or(u8::MAX));
        self.bytes.extend(value.as_bytes());
        self.bytes.push(0);
    }

    /// One header field: its code, then a variant holding a string-like or signature value.
    fn field(&mut self, code: u8, kind: char, value: &str) {
        self.align(8);
        self.bytes.push(code);
        self.signature(&kind.to_string());
        if kind == 'g' {
            self.signature(value);
        } else {
            self.text(value);
        }
    }
}

/// Where a method call goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Target<'a> {
    /// The bus name of the receiver.
    pub destination: &'a str,
    /// The object path.
    pub path: &'a str,
    /// The interface.
    pub interface: &'a str,
    /// The method.
    pub member: &'a str,
}

/// What:
///  The bytes of a method call with string arguments.
/// Why:
///   Kept apart from the socket so a test can read them back and compare them with the specification's layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// encodeCall(serial, START_UNIT, ['cli-git.socket', 'replace'])
/// ```
pub fn encode_call(serial: u32, target: &Target<'_>, arguments: &[&str]) -> Vec<u8> {
    let mut body: Writer = Writer { bytes: Vec::new() };
    for argument in arguments {
        body.text(argument);
    }
    let mut fields: Writer = Writer { bytes: vec![0; 16] };
    fields.field(FIELD_PATH, 'o', target.path);
    fields.field(FIELD_DESTINATION, 's', target.destination);
    fields.field(FIELD_INTERFACE, 's', target.interface);
    fields.field(FIELD_MEMBER, 's', target.member);
    if !arguments.is_empty() {
        fields.field(FIELD_SIGNATURE, 'g', &"s".repeat(arguments.len()));
    }
    // The first sixteen bytes are the fixed header and the array length, filled in below.
    let array_length: u32 = u32::try_from(fields.bytes.len() - 16).unwrap_or(u32::MAX);
    fields.align(8);
    let mut message: Vec<u8> = fields.bytes;
    message[0] = b'l';
    message[1] = METHOD_CALL;
    message[2] = 0;
    message[3] = 1;
    message[4..8].copy_from_slice(&u32::try_from(body.bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    message[8..12].copy_from_slice(&serial.to_le_bytes());
    message[12..16].copy_from_slice(&array_length.to_le_bytes());
    message.extend(body.bytes);
    return message;
}

/// A reply: its type, the error name its header carries, and the first string of its body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reply {
    /// Message type: 2 for a return, 3 for an error, 4 for a signal.
    pub kind: u8,
    /// The number of the call this message answers.
    pub reply_serial: Option<u32>,
    /// The error name of an error reply.
    pub error_name: Option<String>,
    /// The first string argument of the body, which an error reply uses for its text.
    pub first_text: Option<String>,
}

pub(crate) fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    return bytes.get(at..at.checked_add(4)?).map(u32_of_four_bytes);
}

/// What:
///  The little-endian number in four bytes.
/// Why:
///  `read_u32` has already cut the slice to four bytes, so the indexing cannot go out of range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const u32OfFourBytes = (slice: Uint8Array) => new DataView(slice.buffer, slice.byteOffset).getUint32(0, true);
/// ```
fn u32_of_four_bytes(slice: &[u8]) -> u32 {
    return u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]);
}

/// What:
///  The text of a string read together with the offset after it, without the offset.
/// Why:
///  A reply keeps the text and has no use for where it ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const textOf = ([text]: [string, number]) => text;
/// ```
fn text_of(read: (String, usize)) -> String {
    return read.0;
}

/// The string at `at` and the offset after its end byte, or nothing when it is cut short or the byte after the text is not
///  the end byte a D-Bus string carries. Callers pass only the bytes of the region the string must lie in, so an end byte
///  beyond that region is not found.
pub(crate) fn read_text(bytes: &[u8], at: usize) -> Option<(String, usize)> {
    let length: usize = usize::try_from(read_u32(bytes, at)?).ok()?;
    let start: usize = at.checked_add(4)?;
    let end: usize = start.checked_add(length)?;
    let text: &[u8] = bytes.get(start..end)?;
    if *bytes.get(end)? != 0 {
        return None;
    }
    return Some((String::from_utf8_lossy(text).into_owned(), end.saturating_add(1)));
}

/// The string a variant at `at` holds, or nothing when it holds another type, is cut short, or has a signature that does
///  not end with its end byte.
fn read_variant_text(bytes: &[u8], at: usize) -> Option<String> {
    let length: usize = usize::from(*bytes.get(at)?);
    let inner_start: usize = at.checked_add(1)?;
    let inner_end: usize = inner_start.checked_add(length)?;
    let inner: &[u8] = bytes.get(inner_start..inner_end)?;
    if inner != b"s" || *bytes.get(inner_end)? != 0 {
        return None;
    }
    // The signature's end byte is checked; the string follows on a four-byte boundary.
    return read_text(bytes, aligned(inner_end.checked_add(1)?, 4)).map(text_of);
}

pub(crate) fn aligned(offset: usize, boundary: usize) -> usize {
    return match offset.checked_add(boundary - 1) {
        Some(sum) => sum / boundary * boundary,
        None => usize::MAX,
    };
}

/// What:
///  Reads a complete reply message.
/// Why:
///   Only the type, the error name and the first string of the body matter; a header field of a kind this reader does not
///  know ends the walk of the header, which is safe because the fields it needs come first in what the manager sends or
///  are looked for before any unknown kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeReply(bytes)
/// ```
pub fn decode_reply(bytes: &[u8]) -> Result<Reply, CallError> {
    if bytes.len() < 16 || bytes[0] != b'l' {
        return Err(malformed("the reply is not a little-endian message"));
    }
    let kind: u8 = bytes[1];
    let Some(body_length_field) = read_u32(bytes, 4) else {
        return Err(malformed("no body length"));
    };
    let body_length: usize = usize::try_from(body_length_field).unwrap_or(usize::MAX);
    let Some(fields_length_field) = read_u32(bytes, 12) else {
        return Err(malformed("no header length"));
    };
    let fields_length: usize = usize::try_from(fields_length_field).unwrap_or(usize::MAX);
    let fields_end: usize = 16usize.saturating_add(fields_length);
    let body_start: usize = aligned(fields_end, 8);
    if bytes.len() < body_start.saturating_add(body_length) || fields_end > bytes.len() {
        return Err(malformed("the reply is shorter than it says"));
    }
    let mut error_name: Option<String> = None;
    let mut reply_serial: Option<u32> = None;
    let mut signature: String = String::new();
    // A header field lies within the declared header and a body value within the declared body, whatever follows them.
    let header: &[u8] = bytes.get(..fields_end).unwrap_or_default();
    let body: &[u8] = bytes.get(..body_start.saturating_add(body_length)).unwrap_or_default();
    let mut at: usize = 16;
    while at < fields_end {
        at = aligned(at, 8);
        let (Some(&code), Some(&signature_length)) = (header.get(at), header.get(at.saturating_add(1))) else { break };
        let kind_at: usize = at.saturating_add(2);
        let Some(&value_kind) = header.get(kind_at) else { break };
        let value_at: usize = kind_at.saturating_add(usize::from(signature_length)).saturating_add(1);
        match (value_kind, code) {
            (b's' | b'o', _) => {
                let Some((text, next)) = read_text(header, aligned(value_at, 4)) else { break };
                if code == FIELD_ERROR_NAME {
                    error_name = Some(text);
                }
                at = next;
            }
            (b'g', _) => {
                let length: usize = usize::from(*header.get(value_at).unwrap_or(&0));
                if code == FIELD_SIGNATURE {
                    let start: usize = value_at.saturating_add(1);
                    // A signature whose end byte is missing is not read, so the body is not interpreted by it.
                    if header.get(start.saturating_add(length)) == Some(&0) {
                        signature = String::from_utf8_lossy(header.get(start..start.saturating_add(length)).unwrap_or_default()).into_owned();
                    }
                }
                at = value_at.saturating_add(length).saturating_add(2);
            }
            (b'u', _) => {
                let number_at: usize = aligned(value_at, 4);
                if code == FIELD_REPLY_SERIAL {
                    reply_serial = read_u32(header, number_at);
                }
                at = number_at.saturating_add(4);
            }
            _ => break,
        }
    }
    // A string and an object path share one wire form; a variant holding a string is its own signature, then that form.
    let first_text: Option<String> = if signature.starts_with('s') || signature.starts_with('o') {
        read_text(body, aligned(body_start, 4)).map(text_of)
    } else if signature == "v" {
        read_variant_text(body, body_start)
    } else {
        None
    };
    return Ok(Reply { kind, reply_serial, error_name, first_text });
}

/// What:
///  A reply that is cut short or wrongly formed, named by what is wrong with it.
/// Why:
///  `decode_reply` refuses at several points with the same kind of error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const malformed = (what: string) => new CallError('malformed', what);
/// ```
fn malformed(what: &str) -> CallError {
    return CallError::Malformed(what.to_string());
}
