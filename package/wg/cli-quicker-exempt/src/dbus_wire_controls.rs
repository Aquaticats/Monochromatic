//! The messages the controls' fake manager and fake bus send,
//! built with the same writer the program uses.
//! A reader is tested against bytes a server wrote,
//! so it is not only tested against its own writer's output.

/// The bytes of one value.
pub(crate) use super::marshaller::Writer;
/// One header field of a message.
pub(crate) use super::marshaller::header_field;
/// One argument of a message body.
use super::Argument;
/// The signature a body of these arguments declares.
use super::body_signature;
/// The type of an error reply.
use super::ERROR;
/// The header field that holds the error name.
use super::FIELD_ERROR_NAME;
/// The header field that holds the reply serial.
use super::FIELD_REPLY_SERIAL;
/// The header field that holds the body's signature.
use super::FIELD_SIGNATURE;
/// The header field that holds the interface.
use super::FIELD_INTERFACE;
/// The header field that holds the member name.
use super::FIELD_MEMBER;
/// The header field that holds the object path.
use super::FIELD_PATH;
/// The type of a successful reply.
use super::METHOD_RETURN;
/// The type of a signal.
use super::SIGNAL;
/// The bus daemon's interface.
use super::BUS_INTERFACE;
/// The bus daemon's object path.
use super::BUS_PATH;
/// The first byte of a little-endian message.
use super::ENDIAN_LITTLE;
/// The sixteen fixed header bytes.
use super::FIXED_HEADER;
/// The alignment of a struct,
/// a variant
/// and an unsigned 64-bit number.
use super::EIGHT;

/// A return whose body is one string,
/// which a fake bus answers `GetNameOwner` and `Hello` with.
#[cfg(test)]
pub fn encode_text_return(reply_serial: u32, text: &str) -> Vec<u8> {
    return encode_return_with_text(reply_serial, "s", text);
}

/// A return whose body is one 32-bit number,
/// which a fake bus answers `GetConnectionUnixProcessID` with.
#[cfg(test)]
pub fn encode_number_return(reply_serial: u32, number: u32) -> Vec<u8> {
    return encode_return_with_number(reply_serial, "u", number);
}

/// An error reply with a name and a text.
#[cfg(test)]
pub fn encode_error(reply_serial: u32, name: &str, text: &str) -> Vec<u8> {
    let mut fields = Writer { bytes: vec![0; FIXED_HEADER], start: 0 };
    fields.pad(EIGHT);
    fields.bytes.push(FIELD_REPLY_SERIAL);
    fields.signature("u");
    fields.uint32(reply_serial);
    header_field(&mut fields, FIELD_ERROR_NAME, "s", name);
    header_field(&mut fields, FIELD_SIGNATURE, "g", "s");
    let fields_length = u32::try_from(fields.bytes.len().saturating_sub(FIXED_HEADER)).unwrap_or(u32::MAX);
    fields.pad(EIGHT);
    let body_start = fields.bytes.len();
    let mut body = Writer { bytes: Vec::new(), start: body_start };
    body.text(text);
    return finish_message(fields.bytes, ERROR, reply_serial, fields_length, body.bytes);
}

/// A return whose body is one string.
#[cfg(test)]
fn encode_return_with_text(reply_serial: u32, signature: &str, text: &str) -> Vec<u8> {
    let mut fields = Writer { bytes: vec![0; FIXED_HEADER], start: 0 };
    fields.pad(EIGHT);
    fields.bytes.push(FIELD_REPLY_SERIAL);
    fields.signature("u");
    fields.uint32(reply_serial);
    header_field(&mut fields, FIELD_SIGNATURE, "g", signature);
    let fields_length = u32::try_from(fields.bytes.len().saturating_sub(FIXED_HEADER)).unwrap_or(u32::MAX);
    fields.pad(EIGHT);
    let body_start = fields.bytes.len();
    let mut body = Writer { bytes: Vec::new(), start: body_start };
    body.text(text);
    return finish_message(fields.bytes, METHOD_RETURN, reply_serial, fields_length, body.bytes);
}

/// A return whose body is one 32-bit number.
#[cfg(test)]
fn encode_return_with_number(reply_serial: u32, signature: &str, number: u32) -> Vec<u8> {
    let mut fields = Writer { bytes: vec![0; FIXED_HEADER], start: 0 };
    fields.pad(EIGHT);
    fields.bytes.push(FIELD_REPLY_SERIAL);
    fields.signature("u");
    fields.uint32(reply_serial);
    header_field(&mut fields, FIELD_SIGNATURE, "g", signature);
    let fields_length = u32::try_from(fields.bytes.len().saturating_sub(FIXED_HEADER)).unwrap_or(u32::MAX);
    fields.pad(EIGHT);
    let body_start = fields.bytes.len();
    let mut body = Writer { bytes: Vec::new(), start: body_start };
    body.uint32(number);
    return finish_message(fields.bytes, METHOD_RETURN, reply_serial, fields_length, body.bytes);
}

/// Joins header and body into one message.
#[cfg(test)]
fn finish_message(mut header: Vec<u8>, kind: u8, serial: u32, fields_length: u32, body: Vec<u8>) -> Vec<u8> {
    header[0] = ENDIAN_LITTLE;
    header[1] = kind;
    header[2] = 0;
    header[3] = 1;
    header[4..8].copy_from_slice(&u32::try_from(body.len()).unwrap_or(u32::MAX).to_le_bytes());
    header[8..12].copy_from_slice(&serial.to_le_bytes());
    header[12..16].copy_from_slice(&fields_length.to_le_bytes());
    header.extend(body);
    return header;
}

/// A signal a fake bus sends in the controls,
/// with its member and one string argument.
#[cfg(test)]
pub fn encode_signal(serial: u32, member: &str, arguments: &[Argument]) -> Vec<u8> {
    let mut fields = Writer { bytes: vec![0; FIXED_HEADER], start: 0 };
    header_field(&mut fields, FIELD_PATH, "o", BUS_PATH);
    header_field(&mut fields, FIELD_INTERFACE, "s", BUS_INTERFACE);
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
    message[1] = SIGNAL;
    message[2] = 0;
    message[3] = 1;
    message[4..8].copy_from_slice(&u32::try_from(body.bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    message[8..12].copy_from_slice(&serial.to_le_bytes());
    message[12..16].copy_from_slice(&fields_length.to_le_bytes());
    message.extend(body.bytes);
    return message;
}

/// Replies a fake manager sends in the controls.
/// A control that answers a call needs the same layout the manager writes,
/// so the reader is exercised against messages it must read rather than against its own writer alone.
#[cfg(test)]
pub fn encode_reply(reply_serial: u32, error: Option<(&str, &str)>) -> Vec<u8> {
    let mut fields = Writer { bytes: vec![0; FIXED_HEADER], start: 0 };
    fields.pad(EIGHT);
    fields.bytes.push(FIELD_REPLY_SERIAL);
    fields.signature("u");
    fields.uint32(reply_serial);
    if let Some((name, _text)) = error {
        header_field(&mut fields, FIELD_ERROR_NAME, "s", name);
        header_field(&mut fields, FIELD_SIGNATURE, "g", "s");
    }
    let fields_length = u32::try_from(fields.bytes.len().saturating_sub(FIXED_HEADER)).unwrap_or(u32::MAX);
    fields.pad(EIGHT);
    let body_start = fields.bytes.len();
    let mut body = Writer { bytes: Vec::new(), start: body_start };
    if let Some((_name, text)) = error {
        body.text(text);
    }
    let mut message = fields.bytes;
    message[0] = ENDIAN_LITTLE;
    message[1] = if error.is_some() { ERROR } else { METHOD_RETURN };
    message[2] = 0;
    message[3] = 1;
    message[4..8].copy_from_slice(&u32::try_from(body.bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    message[8..12].copy_from_slice(&reply_serial.to_le_bytes());
    message[12..16].copy_from_slice(&fields_length.to_le_bytes());
    message.extend(body.bytes);
    return message;
}
