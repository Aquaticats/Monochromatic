//! What:
//!  The two clone-local properties a forced removal writes before its intent,
//!  what each value says,
//!  and how a reader checks one.
//! Why:
//!  The handler proves the wrapper that asked is alive and is the process the
//!  property names,
//!  so the request carries the wrapper's number,
//!  start time,
//!  recorded parent and parent's start time,
//!  the boot,
//!  and the intent's own nonce
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The caller and its protected set",
//!  and V3).
//!  The force property carries the same nonce and nothing else,
//!  and the design fixes the order:
//!  "the caller property,
//!  the force property and then the intent,
//!  all with one nonce",
//!  so a matching nonce proves the three belong to one request.
//!  The byte length is bounded before any field is split,
//!  because a value of another account's choosing would otherwise decide how
//!  long one split runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const value = callerPropertyValue({ nonce, pid, started, parent, parentStarted, boot });
//! const identity = parseCallerProperty(value);
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_property_names::{
    FIELD_SEPARATOR,
    NONCE_HEX_DIGITS,
    VALUE_VERSION,
};
use crate::worktree_clone_mount_protocol::request_nonce::{
    NonceRefusal,
    RequestNonce,
};

/// The longest caller value the grammar admits,
/// in bytes:
/// `v1:`,
/// the nonce,
/// a ten-digit number each for the four numbers,
/// and the boot id's 36 bytes.
pub const MAXIMUM_CALLER_PROPERTY_BYTES: usize = VALUE_VERSION.len()
    + 1
    + NONCE_HEX_DIGITS
    + (1 + 10)
    + (1 + 20)
    + (1 + 10)
    + (1 + 20)
    + (1 + 36);

/// The longest force value the grammar admits,
/// in bytes:
/// `v1:` and the nonce.
pub const MAXIMUM_FORCE_PROPERTY_BYTES: usize = VALUE_VERSION.len() + 1 + NONCE_HEX_DIGITS;

/// How many bytes the kernel's boot id occupies,
/// as `/proc/sys/kernel/random/boot_id` prints it:
/// eight,
/// four,
/// four,
/// four and twelve hexadecimal digits with four dashes.
pub const BOOT_ID_BYTES: usize = 36;

/// What:
///  Why a caller property is not one this arm can prove.
/// Why:
///  Each cause is separate because the report names what it could not read,
///  and a malformed value "sets nothing":
/// the handler declines the force word
///  rather than reading a field it could not parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CallerPropertyRefusal = { kind: 'too-long' | 'version' | 'fields' | 'number' | 'nonce' | 'boot'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallerPropertyRefusal {
    /// The value is longer than the grammar admits,
    /// so it was not split at all.
    TooLong {
        /// How many bytes the value holds.
        found: usize,
    },
    /// The value does not begin with the version word this grammar writes.
    Version {
        /// The first field,
        /// cut to the version word's own length.
        found: String,
    },
    /// The value holds another number of fields than the grammar writes.
    Fields {
        /// How many fields were found.
        found: usize,
    },
    /// A field that must be a decimal number is not one.
    Number {
        /// The field's own name,
        /// as the design's value spells it.
        field: &'static str,
    },
    /// The nonce field is not 32 lowercase hexadecimal digits.
    Nonce(NonceRefusal),
    /// The boot field is not the kernel's boot id.
    Boot,
}

/// What:
///  Who asked for a forced removal,
///  proved by the handler against the live process.
/// Why:
///  The wrapper is the `git` binary here,
///  so the command the person or the harness ran is that process,
///  and these seven values are what prove it still is
///  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
///  F27).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CallerIdentity = { nonce; pid; started; parent; parentStarted; boot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerIdentity {
    /// The intent's nonce,
    /// which the force property and the intent carry too.
    pub nonce: RequestNonce,
    /// The wrapper's process number.
    pub pid: i32,
    /// The wrapper's start time,
    /// which proves at the next read that the number still names it.
    pub started: u64,
    /// The parent the kernel recorded for the wrapper when the value was written.
    pub parent: i32,
    /// That parent's start time.
    pub parent_started: u64,
    /// The boot the writing ran in.
    pub boot: String,
}

/// What:
///  The caller property's value for one identity.
/// Why:
///  The handler proves the wrapper's number,
///  start time and recorded parent before it walks the chain,
///  so the value carries the parent's identity as well as the wrapper's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${version}:${nonce}:${pid}:${started}:${parent}:${parentStarted}:${boot}`
/// ```
pub fn caller_property_value(identity: &CallerIdentity) -> String {
    return format!(
        "{VALUE_VERSION}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}",
        identity.nonce.as_str(),
        identity.pid,
        identity.started,
        identity.parent,
        identity.parent_started,
        identity.boot
    );
}

/// What:
///  The force property's value for one request.
/// Why:
///  The word carries the nonce and nothing else,
///  so a handler that finds the three values agreeing knows they are one
///  request's,
///  and a value without the caller property proves nothing about who asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${version}:${nonce}`
/// ```
pub fn force_property_value(nonce: &RequestNonce) -> String {
    return format!("{VALUE_VERSION}{FIELD_SEPARATOR}{}", nonce.as_str());
}

/// What:
///  Read one caller property value.
/// Why:
///  A value that is too long is refused before it is split,
///  a malformed one sets nothing,
///  and every refusal names the field it could not read,
///  so a stale or damaged property declines the force word instead of acting
///  on half a request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = parseCallerProperty(text); // throws a refusal
/// ```
pub fn parse_caller_property(text: &str) -> Result<CallerIdentity, CallerPropertyRefusal> {
    if text.len() > MAXIMUM_CALLER_PROPERTY_BYTES {
        return Err(CallerPropertyRefusal::TooLong {
            found: text.len(),
        });
    }
    let fields: Vec<&str> = text.split(FIELD_SEPARATOR).collect();
    if fields.first() != Some(&VALUE_VERSION) {
        return Err(CallerPropertyRefusal::Version {
            found: String::from(fields.first().copied().unwrap_or("")),
        });
    }
    if fields.len() != 7 {
        return Err(CallerPropertyRefusal::Fields {
            found: fields.len(),
        });
    }
    let nonce: RequestNonce =
        RequestNonce::parse(fields[1]).map_err(CallerPropertyRefusal::Nonce)?;
    let pid: i32 = fields[2].parse().map_err(|_error| {
        return CallerPropertyRefusal::Number {
            field: "pid",
        };
    })?;
    let started: u64 = fields[3].parse().map_err(|_error| {
        return CallerPropertyRefusal::Number {
            field: "starttime",
        };
    })?;
    let parent: i32 = fields[4].parse().map_err(|_error| {
        return CallerPropertyRefusal::Number {
            field: "parent-pid",
        };
    })?;
    let parent_started: u64 = fields[5].parse().map_err(|_error| {
        return CallerPropertyRefusal::Number {
            field: "parent-starttime",
        };
    })?;
    let boot: String = String::from(fields[6]);
    if !is_boot_id(boot.as_str()) {
        return Err(CallerPropertyRefusal::Boot);
    }
    return Ok(CallerIdentity {
        nonce,
        pid,
        started,
        parent,
        parent_started,
        boot,
    });
}

/// What:
///  Why a force value is not one this arm can prove.
/// Why:
///  The word is two fields and its only possible faults are a wrong version and
///  a nonce that is not the grammar's,
///  which are kept apart so a report says which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForceWordRefusal = { kind: 'version' } | { kind: 'nonce'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForceWordRefusal {
    /// The value is longer than the grammar admits.
    TooLong {
        /// How many bytes the value holds.
        found: usize,
    },
    /// The value does not begin with the version word.
    Version {
        /// The first field.
        found: String,
    },
    /// The value holds another number of fields than the grammar writes.
    Fields {
        /// How many fields were found.
        found: usize,
    },
    /// The nonce field is not 32 lowercase hexadecimal digits.
    Nonce(NonceRefusal),
}

/// What:
///  Read one force property value and answer the nonce it carries.
/// Why:
///  The nonce is what the caller property and the intent must carry too,
///  so a handler compares three values rather than trusting one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nonce = parseForceProperty(text);
/// ```
pub fn parse_force_property(text: &str) -> Result<RequestNonce, ForceWordRefusal> {
    if text.len() > MAXIMUM_FORCE_PROPERTY_BYTES {
        return Err(ForceWordRefusal::TooLong {
            found: text.len(),
        });
    }
    let fields: Vec<&str> = text.split(FIELD_SEPARATOR).collect();
    if fields.first() != Some(&VALUE_VERSION) {
        return Err(ForceWordRefusal::Version {
            found: String::from(fields.first().copied().unwrap_or("")),
        });
    }
    if fields.len() != 2 {
        return Err(ForceWordRefusal::Fields {
            found: fields.len(),
        });
    }
    return RequestNonce::parse(fields[1]).map_err(ForceWordRefusal::Nonce);
}

/// What:
///  Whether a text is the kernel's boot id.
/// Why:
///  The value carries the boot so a handler on another boot declines a request
///  whose numbers mean other processes there;
///  a text of the wrong shape is not the kernel's answer and proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(text)
/// ```
pub fn is_boot_id(text: &str) -> bool {
    if text.len() != BOOT_ID_BYTES {
        return false;
    }
    let dashes: [usize; 4] = [8, 13, 18, 23];
    for (index, byte) in text.bytes().enumerate() {
        let dashed: bool = dashes.contains(&index);
        if dashed {
            if byte != b'-' {
                return false;
            }
            continue;
        }
        let hexadecimal: bool = byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
        if !hexadecimal {
            return false;
        }
    }
    return true;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "caller_property_tests.rs"]
mod tests;
