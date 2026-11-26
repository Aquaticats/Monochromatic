//! What:
//!  Reading one readiness record:
//!  the byte bound before any line,
//!  the exact key at each line,
//!  the state words,
//!  and the pool,
//!  checkout and sweep values.
//! Why:
//!  An attach is admitted on this file,
//!  so nothing is defaulted and a foreign or half-written file is refused
//!  rather than read as ready
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!  The bound comes first so a huge file is refused without being split.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseReadiness(text); // throws ReadinessParseRefusal
//! ```

use crate::foundation::iso_time::instant_of_iso;
use crate::foundation::random_id::is_canonical_uuid;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    MAXIMUM_CHECKOUT_PATH_BYTES,
    MAXIMUM_POOL_NAME_BYTES,
    MAXIMUM_READINESS_BYTES,
    READINESS_KEYS,
    READINESS_KIND,
    READINESS_VERSION,
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::NO_DOCTOR_LINE_AND_NO_INSTALLED_UNIT;
use crate::worktree_clone_mount_protocol::request_nonce::{
    NonceRefusal,
    RequestNonce,
};

/// What:
///  Why a file is not a readiness record this route wrote.
/// Why:
///   The caller admits an attach on the record,
///  so each deviation has a named cause and the message says nothing was
///  attached on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadinessParseRefusal = { kind: 'too-large' | 'line-count' | 'key' | 'value' | 'nonce' | 'boot' | 'time' | 'state'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadinessParseRefusal {
    /// The file is longer than a record may be.
    TooLarge {
        /// How many bytes were read.
        bytes: usize,
        /// The bound.
        bound: usize,
    },
    /// The file does not have the record's line count.
    LineCount {
        /// How many lines it holds.
        found: usize,
        /// How many a record holds.
        expected: usize,
    },
    /// A line does not begin with the key at that place.
    Key {
        /// The one-based line.
        number: usize,
        /// The key that belongs there.
        expected: &'static str,
        /// The key as read.
        found: String,
    },
    /// A value does not fit its field.
    Value {
        /// The field's key.
        key: &'static str,
        /// What was found.
        shown: String,
    },
    /// A pool or checkout state word is not one of the three.
    State {
        /// The field's key.
        key: &'static str,
        /// The word as read.
        shown: String,
    },
    /// The sweep state word is not one of the four.
    SweepState {
        /// The word as read.
        shown: String,
    },
    /// The nonce is not a nonce.
    Nonce(NonceRefusal),
    /// The boot is not the kernel's identifier.
    Boot {
        /// The value as read.
        value: String,
    },
    /// The time is not ISO 8601 with an offset.
    Time {
        /// The value as read.
        value: String,
    },
}

impl ReadinessParseRefusal {
    /// The refusal in words,
    /// naming the field and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            ReadinessParseRefusal::TooLarge {
                bytes,
                bound,
            } => format!("the file is {bytes} bytes and a readiness record is at most {bound}"),
            ReadinessParseRefusal::LineCount {
                found,
                expected,
            } => {
                format!(
                    "the file holds {found} lines and a readiness record holds exactly {expected}"
                )
            }
            ReadinessParseRefusal::Key {
                number,
                expected,
                found,
            } => {
                format!(
                    "line {number} begins with `{found}` and a readiness record has `{expected}` there"
                )
            }
            ReadinessParseRefusal::Value {
                key,
                shown,
            } => format!("`{key}` holds `{shown}`, which does not fit that field"),
            ReadinessParseRefusal::State {
                key,
                shown,
            } => {
                format!(
                    "`{key}` holds state `{shown}`, and the states are `ready`, `not-ready` and `unknown`"
                )
            }
            ReadinessParseRefusal::SweepState {
                shown,
            } => {
                format!(
                    "the sweep state `{shown}` is none of `finished`, `running`, `failed` and `never`"
                )
            }
            ReadinessParseRefusal::Nonce(refusal) => refusal.message(),
            ReadinessParseRefusal::Boot {
                value,
            } => format!("`{value}` is not the kernel's own boot identifier"),
            ReadinessParseRefusal::Time {
                value,
            } => format!("`time {value}` is not an ISO 8601 time with an offset"),
        };
        return format!(
            "the file in the readiness directory is not a readiness record this route wrote: {reason}. Nothing was attached, mounted or unmounted on its answer. Ways forward: run the handler's `boot-reconcile` in this boot, which plan step 8 builds. {NO_DOCTOR_LINE_AND_NO_INSTALLED_UNIT}; do not hand-edit the file."
        );
    }
}

/// Read one readiness record's text.
///
/// # Errors
/// The text is over the bound,
/// a line's key or value does not fit,
/// or a state word is outside its vocabulary.
pub fn parse_readiness(text: &str) -> Result<ReadinessRecord, ReadinessParseRefusal> {
    if text.len() > MAXIMUM_READINESS_BYTES {
        return Err(ReadinessParseRefusal::TooLarge {
            bytes: text.len(),
            bound: MAXIMUM_READINESS_BYTES,
        });
    }
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() != READINESS_KEYS.len() {
        return Err(ReadinessParseRefusal::LineCount {
            found: lines.len(),
            expected: READINESS_KEYS.len(),
        });
    }
    if lines[0] != READINESS_VERSION {
        return Err(ReadinessParseRefusal::Key {
            number: 1,
            expected: READINESS_VERSION,
            found: String::from(lines[0]),
        });
    }
    let kind: &str = value_of(&lines, 1, READINESS_KEYS[1])?;
    if kind != READINESS_KIND {
        return Err(ReadinessParseRefusal::Value {
            key: READINESS_KEYS[1],
            shown: String::from(kind),
        });
    }
    let boot: &str = value_of(&lines, 2, READINESS_KEYS[2])?;
    if !is_canonical_uuid(boot) {
        return Err(ReadinessParseRefusal::Boot {
            value: String::from(boot),
        });
    }
    let (pool_state, pool_name): (ReadyState, String) =
        state_and_rest(&lines, 3, READINESS_KEYS[3])?;
    if pool_name.is_empty()
        || pool_name.len() > MAXIMUM_POOL_NAME_BYTES
        || !pool_name.bytes().all(dataset_byte)
    {
        return Err(ReadinessParseRefusal::Value {
            key: READINESS_KEYS[3],
            shown: pool_name,
        });
    }
    let (checkout_state, checkout_path): (ReadyState, String) =
        state_and_rest(&lines, 4, READINESS_KEYS[4])?;
    if checkout_path.len() > MAXIMUM_CHECKOUT_PATH_BYTES
        || !checkout_path.starts_with('/')
        || checkout_path.chars().any(char::is_control)
    {
        return Err(ReadinessParseRefusal::Value {
            key: READINESS_KEYS[4],
            shown: checkout_path,
        });
    }
    let (sweep_state, sweep_nonce): (SweepState, Option<RequestNonce>) = sweep_of(&lines)?;
    let time: &str = value_of(&lines, 6, READINESS_KEYS[6])?;
    if instant_of_iso(time).is_none() {
        return Err(ReadinessParseRefusal::Time {
            value: String::from(time),
        });
    }
    return Ok(ReadinessRecord {
        boot: String::from(boot),
        pool_name,
        pool_state,
        checkout_path,
        checkout_state,
        sweep_nonce,
        sweep_state,
        time: String::from(time),
    });
}

/// The dataset-name alphabet the pool name is held to.
fn dataset_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/' | b'$');
}

/// One state word and the rest of the line after it.
fn state_and_rest(
    lines: &[&str],
    index: usize,
    key: &'static str,
) -> Result<(ReadyState, String), ReadinessParseRefusal> {
    let rest: &str = value_of(lines, index, key)?;
    let Some((word, tail)) = rest.split_once(' ') else {
        return Err(ReadinessParseRefusal::Value {
            key,
            shown: String::from(rest),
        });
    };
    let Some(state) = ReadyState::from_word(word) else {
        return Err(ReadinessParseRefusal::State {
            key,
            shown: String::from(word),
        });
    };
    if tail.is_empty() {
        return Err(ReadinessParseRefusal::Value {
            key,
            shown: String::from(rest),
        });
    }
    return Ok((state, String::from(tail)));
}

/// The sweep's state and nonce.
fn sweep_of(lines: &[&str]) -> Result<(SweepState, Option<RequestNonce>), ReadinessParseRefusal> {
    let rest: &str = value_of(lines, 5, READINESS_KEYS[5])?;
    let Some((word, nonce_word)) = rest.split_once(' ') else {
        return Err(ReadinessParseRefusal::Value {
            key: READINESS_KEYS[5],
            shown: String::from(rest),
        });
    };
    let Some(state) = SweepState::from_word(word) else {
        return Err(ReadinessParseRefusal::SweepState {
            shown: String::from(word),
        });
    };
    if nonce_word == "none" {
        return Ok((state, None));
    }
    let nonce: RequestNonce =
        RequestNonce::parse(nonce_word).map_err(ReadinessParseRefusal::Nonce)?;
    return Ok((state, Some(nonce)));
}

/// The value after the key at one place.
fn value_of<'a>(
    lines: &[&'a str],
    index: usize,
    key: &'static str,
) -> Result<&'a str, ReadinessParseRefusal> {
    let line: &'a str = lines[index];
    let Some((found, value)) = line.split_once(' ') else {
        return Err(ReadinessParseRefusal::Key {
            number: index + 1,
            expected: key,
            found: String::from(line),
        });
    };
    if found != key {
        return Err(ReadinessParseRefusal::Key {
            number: index + 1,
            expected: key,
            found: String::from(found),
        });
    }
    if value.is_empty() {
        return Err(ReadinessParseRefusal::Value {
            key,
            shown: String::from("<empty>"),
        });
    }
    return Ok(value);
}
