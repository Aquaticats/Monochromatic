//! What:
//!  The clone-local sweep nonce:
//!  its text,
//!  its parse,
//!  and the reading that separates a nudge from an inherited value and from a
//!  malformed one.
//! Why:
//!  The design lets a waiter change
//!  `org.monochromatic:git-worktree-clone-mount-sweep-nonce` as often as it
//!  needs on the verified managed clone it waits for,
//!  and says the nudge is not a mount instruction
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  So this value is only a nonce:
//!  it names no operation,
//!  no dataset and no path,
//!  and a reader cannot mistake it for a request even when a caller writes an
//!  intent's text into it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = readSweepNonce(properties, cloneName); // { kind: 'nudge', nonce } | ...
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_property_names::{NONCE_HEX_DIGITS, SWEEP_NONCE_PROPERTY};
use crate::worktree_clone_mount_protocol::local_user_property::{PropertyReading, PropertyRefusal, read_local_property};
use crate::worktree_clone_mount_protocol::request_nonce::{NonceRefusal, RequestNonce};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// What:
///  Why a value is not a sweep nonce.
/// Why:
///   The change was not a nudge the handler can read,
///  and the caller needs to know which shape was wrong without the message
///  suggesting the change was a mount request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NudgeRefusal = { kind: 'nonce'; refusal } | { kind: 'property'; refusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NudgeRefusal {
    /// The value is not one 32-hex-digit nonce.
    Nonce(NonceRefusal),
    /// The property itself could not be read.
    Property(PropertyRefusal),
}

impl NudgeRefusal {
    /// The refusal in words,
    /// naming the property and saying that no request was made.
    pub fn message(&self) -> String {
        let reason: String = match self {
            NudgeRefusal::Nonce(refusal) => refusal.message(),
            NudgeRefusal::Property(refusal) => refusal.message(SWEEP_NONCE_PROPERTY),
        };
        return format!(
            "{reason} The sweep nonce is a nudge and never a mount instruction, so no mount or unmount was requested and nothing was done on its behalf. Ways forward: write exactly {NONCE_HEX_DIGITS} lowercase hexadecimal digits as the value of `{SWEEP_NONCE_PROPERTY}`, and let the durable intent property carry any operation."
        );
    }
}

/// What:
///  What one clone's sweep-nonce property says.
/// Why:
///   Only a local value is a nudge;
///  an inherited one belongs to an ancestor and is ignored,
///  and a malformed one is named rather than treated as a nudge or as
///  silence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NudgeReading = { kind: 'absent' } | { kind: 'inherited'; from } | { kind: 'nudge'; nonce } | { kind: 'refused'; refusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NudgeReading {
    /// The clone carries no sweep nonce of its own.
    Absent,
    /// The value is inherited from another dataset and nudges nothing.
    Inherited {
        /// The dataset the property is set on.
        from: String,
    },
    /// The clone carries one readable nudge.
    Nudge(RequestNonce),
    /// The value is not a nudge and authorizes nothing.
    Refused(NudgeRefusal),
}

/// What:
///  The text one nonce is written as in the sweep property.
/// Why:
///   The value is the nonce alone,
///  so a change can never be read as an operation even if a caller wanted it
///  to be one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = sweepNonceText(nonce); // exactly the 32 digits
/// ```
pub fn sweep_nonce_text(nonce: &RequestNonce) -> String {
    return String::from(nonce.as_str());
}

/// What:
///  Read one sweep-nonce value,
///  bounding its bytes before parsing.
/// Why:
///   The nudge shares the nonce's bound and alphabet,
///  and a value longer than a nonce is refused before any byte is examined
///  as a digit.
///
/// # Errors
/// The value is not 32 bytes,
/// or one byte is not a lowercase hexadecimal digit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nonce = parseSweepNonce(text); // throws NudgeRefusal
/// ```
pub fn parse_sweep_nonce(text: &str) -> Result<RequestNonce, NudgeRefusal> {
    if text.len() > NONCE_HEX_DIGITS {
        return Err(NudgeRefusal::Nonce(NonceRefusal::Length { found: text.len() }));
    }
    return RequestNonce::parse(text).map_err(NudgeRefusal::Nonce);
}

/// What:
///  Read a clone's sweep-nonce property as a nudge,
///  an inherited value,
///  an absence or a refusal.
/// Why:
///   The handler may act on a local nudge and on nothing else,
///  and the same property reader the intent uses keeps the two decisions
///  consistent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readSweepNonce(reply.properties, cloneName);
/// ```
pub fn read_sweep_nonce(properties: Option<&NvList>, clone: &str) -> NudgeReading {
    let reading: PropertyReading<'_> = read_local_property(properties, clone, SWEEP_NONCE_PROPERTY);
    return match reading {
        PropertyReading::Absent => NudgeReading::Absent,
        PropertyReading::Inherited { from } => NudgeReading::Inherited { from },
        PropertyReading::Local { value } => match parse_sweep_nonce(value) {
            Ok(nonce) => NudgeReading::Nudge(nonce),
            Err(refusal) => NudgeReading::Refused(refusal),
        },
        PropertyReading::Malformed(refusal) => NudgeReading::Refused(NudgeRefusal::Property(refusal)),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_sweep_nonce_tests.rs"]
mod tests;
