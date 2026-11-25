//! What:
//!  The words the settling command reads and prints:
//!  its two flags,
//!  the placeholders a printed line carries,
//!  and the last digits of a clone's GUID that one of them asks for.
//! Why:
//!  The design fixes the grammar and the two values a person copies
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command" and its veto entry V21),
//!  and a sentence that prints the digits instead of a placeholder would let a
//!  person confirm without reading anything.
//!  The program's words are the binding folder's one constant,
//!  so the line a refusal prints and the command this module parses cannot
//!  disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const usage = `${SETTLING_COMMAND_WORDS} <slot> ${CONFIRM_REMOVAL_FLAG} <slot> ${CONFIRM_GUID_FLAG} <digits>`;
//! ```

use crate::worktree_clone_binding_record::add_state_commands::SETTLING_COMMAND_WORDS;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;

/// The flag that names the slot a second time.
pub const CONFIRM_REMOVAL_FLAG: &str = "--confirm-removal-of";

/// The flag that names the last digits of the record's own `cloneGuid=`.
pub const CONFIRM_GUID_FLAG: &str = "--confirm-clone-guid";

/// How many of the GUID's decimal digits the second flag asks for.
///
/// The design fixes four in its veto entry V21,
/// so this is its number and not a measured bound.
pub const GUID_DIGITS_SHOWN: usize = 4;

/// The character a printed line wraps a value in,
/// so a paste that leaves it in fails.
pub const PLACEHOLDER_MARK: char = '<';

/// The whole invocation the usage sentence prints,
/// with both values shown as placeholders.
pub fn the_usage_line() -> String {
    return format!("{SETTLING_COMMAND_WORDS} <slot> {CONFIRM_REMOVAL_FLAG} <slot> {CONFIRM_GUID_FLAG} <the last {GUID_DIGITS_SHOWN} decimal digits of cloneGuid=>");
}

/// What:
///  The digits the second flag must carry for one recorded clone.
/// Why:
///  The GUID is a datum the slot's name does not supply,
///  so a typo that finds another slot's record still refuses on these digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const digits = lastFourDecimalDigits(guid); // '3102' for 4821193048571903102
/// ```
pub fn the_last_four_digits(guid: DatasetGuid) -> String {
    let whole: String = guid.file_name();
    let digits: &str = match whole.len() > GUID_DIGITS_SHOWN {
        true => &whole[whole.len() - GUID_DIGITS_SHOWN..],
        false => whole.as_str(),
    };
    return String::from(digits);
}

/// What:
///  Whether a value a person typed still carries a printed placeholder.
/// Why:
///  The line a refusal prints names `<slot>` and `<the last four decimal digits of cloneGuid=>`,
///  so a blind paste of that line arrives here rather than at a comparison that
///  would report a mere difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pasted = carriesAPlaceholder(value);
/// ```
pub fn carries_a_placeholder(value: &str) -> bool {
    return value.contains(PLACEHOLDER_MARK);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_words_tests.rs"]
mod tests;
