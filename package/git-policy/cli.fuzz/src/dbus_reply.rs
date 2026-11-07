//! What:
//! Raw and generated D-Bus reply messages through the reply reader of the session bus client.
//! Why:
//! The bytes come from a socket another process writes;
//! the reader must give a result for every input,
//!      never panic or read past the message,
//! and must read back what a well-formed reply says.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => decodeReply(bytes)).not.toThrow(); expect(decodeReply(build(fields)).errorName).toBe(fields.errorName);
//! ```

/// The reader under test.
use git_policy_cli::systemd_dbus::{Reply, decode_reply};

/// Message type of a successful reply.
const METHOD_RETURN: u8 = 2;

/// Message type of an error reply.
const ERROR: u8 = 3;

/// Header field code of the number of the call a reply answers.
const FIELD_REPLY_SERIAL: u8 = 5;

/// Header field code of the error name.
const FIELD_ERROR_NAME: u8 = 4;

/// Header field code of the body signature.
const FIELD_SIGNATURE: u8 = 8;

/// What:
/// Reads arbitrary bytes as a reply.
/// Why:
///  Totality is the whole property:
/// any input gives a result,
/// and a result that names a number or a text
///       names one the bytes could hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeReply(bytes); // never throws
/// ```
pub fn check_raw_reply(bytes: &[u8]) {
    if let Ok(reply) = decode_reply(bytes) {
        if let Some(name) = reply.error_name.as_ref() {
            assert!(name.len() <= bytes.len(), "an error name longer than the message");
        }
        if let Some(text) = reply.first_text.as_ref() {
            assert!(text.len() <= bytes.len(), "a text longer than the message");
        }
    }
}

/// Pads to the next multiple of `boundary`.
fn pad(bytes: &mut Vec<u8>, boundary: usize) {
    while bytes.len() % boundary != 0 {
        bytes.push(0);
    }
}

/// A well-formed error or return reply built from fuzz bytes,
/// with the reply it must be read as.
pub fn generated_reply(data: &[u8]) -> (Vec<u8>, Reply) {
    let serial: u32 = u32::from(data.first().copied().unwrap_or(1));
    let is_error: bool = data.get(1).copied().unwrap_or(0) % 2 == 1;
    let name: String = data.iter().skip(2).take(12).map(|byte| { return char::from(b'a' + byte % 26); }).collect();
    let text: String = data.iter().skip(14).take(24).map(|byte| { return char::from(b' ' + byte % 90); }).collect();
    let mut array: Vec<u8> = Vec::new();
    if is_error {
        let named: String = format!("x.{name}");
        pad(&mut array, 8);
        array.extend([FIELD_ERROR_NAME, 1, b's', 0]);
        array.extend(u32::try_from(named.len()).unwrap_or(0).to_le_bytes());
        array.extend(named.as_bytes());
        array.push(0);
    }
    pad(&mut array, 8);
    array.extend([FIELD_REPLY_SERIAL, 1, b'u', 0]);
    array.extend(serial.to_le_bytes());
    let mut body: Vec<u8> = Vec::new();
    if is_error {
        pad(&mut array, 8);
        array.extend([FIELD_SIGNATURE, 1, b'g', 0, 1, b's', 0]);
        body.extend(u32::try_from(text.len()).unwrap_or(0).to_le_bytes());
        body.extend(text.as_bytes());
        body.push(0);
    }
    let mut message: Vec<u8> = vec![b'l', if is_error { ERROR } else { METHOD_RETURN }, 0, 1];
    message.extend(u32::try_from(body.len()).unwrap_or(0).to_le_bytes());
    message.extend(7u32.to_le_bytes());
    message.extend(u32::try_from(array.len()).unwrap_or(0).to_le_bytes());
    message.extend(array);
    pad(&mut message, 8);
    message.extend(body);
    let expected: Reply = Reply {
        kind: if is_error { ERROR } else { METHOD_RETURN },
        reply_serial: Some(serial),
        error_name: is_error.then(|| { return format!("x.{name}"); }),
        first_text: is_error.then_some(text),
    };
    return (message, expected);
}

/// What:
/// Checks that a built reply reads back as it was built,
/// and that damaging it never panics.
/// Why:
///  The oracle is the generator's own fields,
/// not the reader's output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(decodeReply(built)).toEqual(expected); for (const prefix of prefixes(built)) decodeReply(prefix);
/// ```
pub fn check_generated_reply(data: &[u8]) {
    let (message, expected) = generated_reply(data);
    assert_eq!(decode_reply(&message), Ok(expected));
    for length in 0..message.len() {
        check_raw_reply(&message[..length]);
    }
    let at: usize = usize::from(data.get(2).copied().unwrap_or(0)) % message.len();
    let mut changed: Vec<u8> = message;
    changed[at] = data.get(3).copied().unwrap_or(0xff);
    check_raw_reply(&changed);
}

/// Generator controls stay out of the release executable.
#[cfg(test)]
#[path = "dbus_reply_tests.rs"]
mod tests;
