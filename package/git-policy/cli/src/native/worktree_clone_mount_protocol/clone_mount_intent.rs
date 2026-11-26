//! What:
//!  The durable mount intent of one clone:
//!  its grammar,
//!  its text,
//!  and the reading that separates a local request from an inherited value
//!  and from a malformed one.
//! Why:
//!  The design accepts exactly `v1:<nonce>:mount` or `v1:<nonce>:unmount`,
//!  with a nonce of 32 lowercase ASCII hex digits,
//!  bounds the byte length before parsing,
//!  and ignores inherited intent
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  No request supplies a path,
//!  a dataset name or mount flags:
//!  this grammar is the whole request.
//!  A refusal here authorizes nothing,
//!  so the caller can report it before any filesystem operation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const intent = readIntent(properties, cloneName); // { kind: 'request', operation, nonce } | ...
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_property_names::{
    DESIRED_STATE_PROPERTY,
    FIELD_SEPARATOR,
    MAXIMUM_INTENT_BYTES,
    MOUNT_WORD,
    UNMOUNT_WORD,
    VALUE_VERSION,
};
use crate::worktree_clone_mount_protocol::local_user_property::{
    PropertyReading,
    PropertyRefusal,
    read_local_property,
};
use crate::worktree_clone_mount_protocol::request_nonce::{
    NonceRefusal,
    RequestNonce,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// What:
///  Which mount operation an intent asks for.
/// Why:
///   The two words are a closed set,
///  so a value naming anything else is refused rather than guessed,
///  and the operation is the only thing besides the nonce the grammar
///  carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountOperation = 'mount' | 'unmount';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountOperation {
    /// Attach the clone's mount.
    Mount,
    /// Unmount the clone's mount.
    Unmount,
}

impl MountOperation {
    /// The word this operation is written as.
    pub fn word(self) -> &'static str {
        return match self {
            MountOperation::Mount => MOUNT_WORD,
            MountOperation::Unmount => UNMOUNT_WORD,
        };
    }

    /// Read one operation word,
    /// or nothing when it is not one of the two.
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            MOUNT_WORD => Some(MountOperation::Mount),
            UNMOUNT_WORD => Some(MountOperation::Unmount),
            _ => None,
        };
    }
}

/// What:
///  One validated request:
///  the operation and the nonce that matches its receipt.
/// Why:
///   The two fields are what a receipt is matched on,
///  and keeping them in one value means a caller cannot pass the operation of
///  one request with the nonce of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountIntent = { operation: MountOperation; nonce: RequestNonce };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountIntent {
    /// What the request asks for.
    pub operation: MountOperation,
    /// The nonce every receipt for it carries.
    pub nonce: RequestNonce,
}

/// What:
///  Why an intent value could not be read as a request.
/// Why:
///   The bound is checked before any field is split,
///  so an oversized value has its own cause instead of being reported as a
///  long nonce;
///  every other cause names the word or field a person needs to change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IntentRefusal = { kind: 'too-long' | 'field-count' | 'version' | 'operation' | 'nonce' | 'property'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntentRefusal {
    /// The value is longer than the grammar's bound.
    TooLong {
        /// How many bytes the value holds.
        bytes: usize,
        /// The bound.
        bound: usize,
    },
    /// The value does not have the grammar's three fields.
    FieldCount {
        /// How many fields the separator split the value into.
        fields: usize,
    },
    /// The first field is not the version this build reads.
    WrongVersion {
        /// The field as read.
        word: String,
    },
    /// The third field is neither operation word.
    UnknownOperation {
        /// The field as read.
        word: String,
    },
    /// The second field is not a nonce.
    Nonce(NonceRefusal),
    /// The property itself could not be read,
    /// so no value reached the grammar.
    Property(PropertyRefusal),
}

impl IntentRefusal {
    /// The refusal in words,
    /// naming the property,
    /// what was read and the way forward.
    pub fn message(&self) -> String {
        let named: String = match self {
            IntentRefusal::TooLong {
                bytes,
                bound,
            } => format!(
                "the mount intent under {DESIRED_STATE_PROPERTY} is {bytes} bytes long, and the longest intent this route reads is {bound} bytes (`{VALUE_VERSION}:<32 lowercase hex digits>:{UNMOUNT_WORD}`). Nothing was parsed, mounted or unmounted. Ways forward: write the intent in the designed form, or inherit nothing and leave the property unset."
            ),
            IntentRefusal::FieldCount {
                fields,
            } => format!(
                "the mount intent under {DESIRED_STATE_PROPERTY} has {fields} `{FIELD_SEPARATOR}`-separated fields, and the grammar has exactly three: `{VALUE_VERSION}`, the 32-hex-digit nonce, and `{MOUNT_WORD}` or `{UNMOUNT_WORD}`. Nothing was parsed, mounted or unmounted. Ways forward: write the intent again in the designed form, with no extra field."
            ),
            IntentRefusal::WrongVersion {
                word,
            } => format!(
                "the mount intent under {DESIRED_STATE_PROPERTY} begins with `{word}`, and this build reads only `{VALUE_VERSION}`. Nothing was parsed, mounted or unmounted. Ways forward: write the intent with the `{VALUE_VERSION}` version word; a version this build does not know is never read as an older or newer one."
            ),
            IntentRefusal::UnknownOperation {
                word,
            } => format!(
                "the mount intent under {DESIRED_STATE_PROPERTY} asks for `{word}`, and the only operations are `{MOUNT_WORD}` and `{UNMOUNT_WORD}`. Nothing was parsed, mounted or unmounted. Ways forward: write one of the two operation words."
            ),
            IntentRefusal::Nonce(refusal) => format!(
                "the intent under {DESIRED_STATE_PROPERTY} was not read: {}",
                refusal.message()
            ),
            IntentRefusal::Property(refusal) => refusal.message(DESIRED_STATE_PROPERTY),
        };
        return named;
    }
}

/// What:
///  What one clone's desired-state property says.
/// Why:
///   The distinction between absent,
///  inherited and a request is the design's own:
///  an inherited value is ignored rather than obeyed,
///  and a malformed value is refused rather than treated as absent,
///  because a silent absence would leave an operator believing a request was
///  read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IntentReading = { kind: 'absent' } | { kind: 'inherited'; from } | { kind: 'request'; intent } | { kind: 'refused'; refusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntentReading {
    /// The clone carries no intent of its own.
    Absent,
    /// The value is inherited from another dataset and authorizes nothing.
    Inherited {
        /// The dataset the property is set on.
        from: String,
    },
    /// The clone asks for one operation under one nonce.
    Request(MountIntent),
    /// The value cannot be read as a request and authorizes nothing.
    Refused(IntentRefusal),
}

impl IntentReading {
    /// Whether reading this property asks the route to do work.
    /// Only a request does.
    pub fn asks_for_work(&self) -> bool {
        return matches!(self, IntentReading::Request(_));
    }
}

/// What:
///  The value text one operation and nonce are written as.
/// Why:
///   One writer serves the wrapper's request and every control,
///  so the grammar is spelled once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intentText = (operation, nonce) => `v1:${nonce.asStr()}:${operation}`;
/// ```
pub fn intent_text(
    operation: MountOperation,
    nonce: &RequestNonce,
) -> String {
    return format!(
        "{VALUE_VERSION}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}",
        nonce.as_str(),
        operation.word()
    );
}

/// What:
///  Read one intent value,
///  bounding its bytes before splitting any field.
/// Why:
///   The design asks for the bound before parsing,
///  and it keeps an oversized value from being reported as a malformed nonce
///  or from splitting a million fields.
///
/// # Errors
/// The value is over the bound,
/// does not have three fields,
/// names another version or operation,
/// or carries a malformed nonce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = parseMountIntent(text); // throws IntentRefusal
/// ```
pub fn parse_mount_intent(text: &str) -> Result<MountIntent, IntentRefusal> {
    let bytes: usize = text.len();
    if bytes > MAXIMUM_INTENT_BYTES {
        return Err(IntentRefusal::TooLong {
            bytes,
            bound: MAXIMUM_INTENT_BYTES,
        });
    }
    let fields: Vec<&str> = text.split(FIELD_SEPARATOR).collect();
    let [version, digits, operation_word] = fields.as_slice() else {
        return Err(IntentRefusal::FieldCount {
            fields: fields.len(),
        });
    };
    if *version != VALUE_VERSION {
        return Err(IntentRefusal::WrongVersion {
            word: String::from(*version),
        });
    }
    let operation: MountOperation = MountOperation::from_word(operation_word).ok_or_else(|| {
        return IntentRefusal::UnknownOperation {
            word: String::from(*operation_word),
        };
    })?;
    let nonce: RequestNonce = RequestNonce::parse(digits).map_err(IntentRefusal::Nonce)?;
    return Ok(MountIntent {
        operation,
        nonce,
    });
}

/// What:
///  Read a clone's desired-state property as a request,
///  an inherited value,
///  an absence or a refusal.
/// Why:
///   Every caller of this route must make the same three-way distinction,
///  and the property's source decides local from inherited:
///  a value set on an ancestor is never this clone's request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readIntent(reply.properties, cloneName);
/// ```
pub fn read_intent(
    properties: Option<&NvList>,
    clone: &str,
) -> IntentReading {
    let reading: PropertyReading<'_> =
        read_local_property(properties, clone, DESIRED_STATE_PROPERTY);
    return match reading {
        PropertyReading::Absent => IntentReading::Absent,
        PropertyReading::Inherited {
            from,
        } => IntentReading::Inherited {
            from,
        },
        PropertyReading::Local {
            value,
        } => match parse_mount_intent(value) {
            Ok(intent) => IntentReading::Request(intent),
            Err(refusal) => IntentReading::Refused(refusal),
        },
        PropertyReading::Malformed(refusal) => {
            IntentReading::Refused(IntentRefusal::Property(refusal))
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_intent_tests.rs"]
mod tests;
