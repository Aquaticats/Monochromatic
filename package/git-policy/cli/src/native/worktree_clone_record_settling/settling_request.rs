//! What:
//!  The three values the settling command's grammar carries,
//!  read from its arguments:
//!  the slot,
//!  the slot again under `--confirm-removal-of`,
//!  and the last four decimal digits of the record's own `cloneGuid=`
//!  under `--confirm-clone-guid`.
//! Why:
//!  The confirmation is the two flags' values and never a prompt,
//!  because the wrapper's commands are non-interactive
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command").
//!  Two checks need no record and run here,
//!  so a mistyped flag and a blind paste of a printed line refuse before any
//!  file is read:
//!  the same slot twice,
//!  and a value that still carries its placeholder.
//!  The checks against the record itself,
//!  the slot its `slot=` derives and its `cloneGuid=`'s digits,
//!  belong to the read that supplies those values.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const request = parseSettlingArguments(['topic', '--confirm-removal-of', 'topic', '--confirm-clone-guid', '3102']);
//! ```

use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_words::{
    CONFIRM_GUID_FLAG,
    CONFIRM_REMOVAL_FLAG,
    carries_a_placeholder,
};
use std::ffi::OsString;

/// The separator that would end this command's operands,
/// which its grammar has no use for.
const SEPARATOR: &str = "--";

/// What:
///  One parsed settling command:
///  the slot the person means and the two values they typed.
/// Why:
///  The record's own reader and the pool are asked about the slot this names,
///  so a value that names another slot never reaches them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { slot: string; confirmedSlot: string; digits: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlingRequest {
    /// The slot named as the command's one operand.
    pub slot: String,
    /// The slot named again under `--confirm-removal-of`.
    pub confirmed_slot: String,
    /// The digits named under `--confirm-clone-guid`.
    pub digits: String,
}

/// What:
///  One argument as text,
///  refused when it is not UTF-8,
///  which no slot name and no digit can be.
/// Why:
///  A value silently replaced by `?` characters would compare unequal to
///  everything and read as a mistyped slot rather than as unreadable arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = asText(argument); // throws for a byte no UTF-8 run holds
/// ```
fn as_text(argument: &OsString) -> Result<String, SettlingStop> {
    return argument.to_str().map(String::from).ok_or_else(|| {
        return SettlingStop::Usage {
            said: format!(
                "one argument `{}` is not text this command can read",
                argument.to_string_lossy()
            ),
        };
    });
}

/// What:
///  The slot's own characters,
///  refused when a slash or the empty name would send a read out of the record home.
/// Why:
///  The slot is a folder name in the repository's worktree parent,
///  and this command removes files named after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (value === '' || value.includes('/')) throw new Error('usage');
/// ```
fn a_slot_name(value: String) -> Result<String, SettlingStop> {
    if value.is_empty() {
        return Err(SettlingStop::Usage {
            said: String::from("the slot names no characters"),
        });
    }
    if value.contains('/') {
        return Err(SettlingStop::Usage {
            said: format!("the slot `{value}` holds a `/`, and a slot is one folder name"),
        });
    }
    if carries_a_placeholder(value.as_str()) {
        return Err(SettlingStop::PlaceholderLeft {
            flag: String::from("the slot"),
            value,
        });
    }
    return Ok(value);
}

/// What:
///  The settling command's three values,
///  or the sentence a malformed invocation reads.
/// Why:
///  This is the whole grammar:
///  one operand and the two flags,
///  each flag once,
///  each with a value,
///  and nothing else.
///  The wiring round calls this in the management grammar and passes its answer
///  to the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseSettlingArguments(args);
/// ```
pub fn parse_settling_arguments(arguments: &[OsString]) -> Result<SettlingRequest, SettlingStop> {
    let mut slot: Option<String> = None;
    let mut confirmed: Option<String> = None;
    let mut digits: Option<String> = None;
    let mut rest: std::slice::Iter<'_, OsString> = arguments.iter();
    while let Some(argument) = rest.next() {
        let text: String = as_text(argument)?;
        if text == SEPARATOR {
            return Err(SettlingStop::Usage {
                said: String::from("a `--` separator has nothing after it in this command"),
            });
        }
        if text != CONFIRM_REMOVAL_FLAG && text != CONFIRM_GUID_FLAG {
            if text.starts_with("--") {
                return Err(SettlingStop::Usage {
                    said: format!("`{text}` is no option of this command"),
                });
            }
            if slot.is_some() {
                return Err(SettlingStop::Usage {
                    said: format!("`{text}` is a second slot, and the command names one"),
                });
            }
            slot = Some(a_slot_name(text)?);
            continue;
        }
        let Some(value) = rest.next() else {
            return Err(SettlingStop::Usage {
                said: format!("`{text}` is the last argument and carries no value"),
            });
        };
        let typed: String = as_text(value)?;
        if text == CONFIRM_REMOVAL_FLAG {
            if confirmed.is_some() {
                return Err(SettlingStop::Usage {
                    said: format!("`{CONFIRM_REMOVAL_FLAG}` stands twice"),
                });
            }
            confirmed = Some(a_slot_name(typed)?);
        } else {
            if digits.is_some() {
                return Err(SettlingStop::Usage {
                    said: format!("`{CONFIRM_GUID_FLAG}` stands twice"),
                });
            }
            if carries_a_placeholder(typed.as_str()) {
                return Err(SettlingStop::PlaceholderLeft {
                    flag: String::from(CONFIRM_GUID_FLAG),
                    value: typed,
                });
            }
            if typed.is_empty() {
                return Err(SettlingStop::Usage {
                    said: format!("`{CONFIRM_GUID_FLAG}` carries no digits"),
                });
            }
            digits = Some(typed);
        }
    }
    let (Some(given_slot), Some(given_confirmed), Some(given_digits)) = (slot, confirmed, digits)
    else {
        return Err(SettlingStop::Usage {
            said: String::from(
                "the command needs a slot, `--confirm-removal-of` and `--confirm-clone-guid`",
            ),
        });
    };
    if given_slot != given_confirmed {
        return Err(SettlingStop::TheSlotIsNamedTwice {
            asked: given_slot,
            confirmed: given_confirmed,
        });
    }
    return Ok(SettlingRequest {
        slot: given_slot,
        confirmed_slot: given_confirmed,
        digits: given_digits,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_request_tests.rs"]
mod tests;
