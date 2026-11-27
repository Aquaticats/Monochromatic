//! What:
//!  The fields of one native journal entry:
//!  the sender's identifier,
//!  the check on a field name,
//!  one field written as `NAME=value` or in the length-prefixed form,
//!  and the two functions that build an entry.
//! Why:
//!  A message that starts a field of its own,
//!  or a value whose newline would,
//!  is how an entry came to carry a field nobody wrote;
//!  the length-prefixed form carries any bytes,
//!  so a value reads back in the journal as it was sent,
//!  and a field whose name journald would refuse is left out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! `PRIORITY=${n}\nSYSLOG_IDENTIFIER=${identifier}\nMESSAGE=${message}\n${fields.map(([k, v]) => `${k}=${v}\n`).join('')}`
//! ```

use crate::journald_native_the_bounds_of_one_entry::{
    FIELD_LIMIT,
    FIELD_NAME_LIMIT,
    MESSAGE_LIMIT,
};
use crate::journald_native_the_words_of_one_entry::{
    Priority,
    cut,
};
/// The `SYSLOG_IDENTIFIER` of every entry,
/// the name `journalctl` shows after the host.
pub const IDENTIFIER: &str = "monochromatic-git-policy-cli-service";

/// What:
///  Whether a field name is one journald accepts from a client:
/// 1 to 64 upper-case letters,
/// digits and underscores,
/// not
///  starting with an underscore (the journal's own fields) or a digit.
/// Why:
///   This is systemd's `journal_field_valid` called with `allow_protected` false (`src/libsystemd/sd-journal/journal-file.c`,
///  read from the `main` branch;
/// its comments say names longer than 64 are refused and a digit may not come first),
/// which
///  `manager_process_entry` in `src/journal/journald-native.c` applies to both forms of a field.
/// A refused name is dropped by
///  journald without a word,
/// so the entry is built without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[A-Z][A-Z0-9_]{0,63}$/.test(name)
/// ```
pub fn valid_field_name(name: &str) -> bool {
    let bytes: &[u8] = name.as_bytes();
    return bytes.len() <= FIELD_NAME_LIMIT
        && bytes.first().is_some_and(u8::is_ascii_uppercase)
        && bytes.iter().all(is_field_name_byte);
}

/// What:
///  Whether a byte may appear in a field name after the first:
/// an uppercase ASCII letter,
/// a digit or an underscore.
/// Why:
///  `valid_field_name` checks every byte,
/// as the journal's native protocol allows no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isFieldNameByte = (byte: number) => /[A-Z0-9_]/.test(String.fromCharCode(byte));
/// ```
pub(crate) fn is_field_name_byte(byte: &u8) -> bool {
    return byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_';
}

/// Appends one field:
/// `NAME=value\n` when the value has no newline,
/// otherwise the length-prefixed form,
/// `NAME\n`,
/// the byte
/// length as eight little-endian bytes,
/// the value and `\n`.
pub(crate) fn push_field(
    entry: &mut Vec<u8>,
    name: &str,
    value: &str,
) {
    if value.contains('\n') {
        entry.extend_from_slice(name.as_bytes());
        entry.push(b'\n');
        entry.extend_from_slice(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_le_bytes());
        entry.extend_from_slice(value.as_bytes());
    } else {
        entry.extend_from_slice(name.as_bytes());
        entry.push(b'=');
        entry.extend_from_slice(value.as_bytes());
    }
    entry.push(b'\n');
}

/// What:
///  The bytes of one native entry of the service:
/// see `datagram_as`.
/// Why:
///   Every entry of the service carries the service's identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// datagramAs('cli-git-service', priority, message, fields)
/// ```
pub fn datagram(
    priority: Priority,
    message: &str,
    fields: &[(&str, String)],
) -> Vec<u8> {
    return datagram_as(IDENTIFIER, priority, message, fields);
}

/// What:
///  The bytes of one native entry:
/// `PRIORITY`,
/// `SYSLOG_IDENTIFIER`,
/// `MESSAGE`,
/// then each field,
/// each value as written (cut
///  to its limit),
/// in the length-prefixed form when it holds a newline.
/// Why:
///   A newline cannot be written in the simple `NAME=value` form without starting a field,
/// and the length-prefixed form
///  carries any bytes,
/// so a value reads back in the journal as it was sent.
/// A field whose name journald would refuse is left
///  out.
/// The identifier names the sender:
/// the service,
/// or the wrapper a person ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PRIORITY=${n}\nSYSLOG_IDENTIFIER=${identifier}\nMESSAGE=${message}\n${fields.map(([k, v]) => `${k}=${v}\n`).join('')}`
/// ```
pub fn datagram_as(
    identifier: &str,
    priority: Priority,
    message: &str,
    fields: &[(&str, String)],
) -> Vec<u8> {
    let mut entry: Vec<u8> = Vec::new();
    push_field(
        &mut entry,
        "PRIORITY",
        priority.number().to_string().as_str(),
    );
    push_field(&mut entry, "SYSLOG_IDENTIFIER", identifier);
    push_field(&mut entry, "MESSAGE", cut(message, MESSAGE_LIMIT).as_str());
    for (name, value) in fields {
        if valid_field_name(name) {
            push_field(&mut entry, name, cut(value, FIELD_LIMIT).as_str());
        }
    }
    return entry;
}
