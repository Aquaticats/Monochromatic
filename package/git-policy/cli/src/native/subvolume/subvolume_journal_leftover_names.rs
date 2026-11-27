//! What:
//!  The names a journal writer may leave beside the journal,
//! and which of them is its own to remove:
//! the journal's name,
//! then `.tmp`,
//! then either nothing (the name releases up to 0.5.20 wrote) or a dot,
//! a process number,
//! a dot and a clock reading.
//! Why:
//!  A killed run leaves a plain file under such a name,
//! and the next write that lands removes those and nothing else;
//! any other name beside the journal is somebody else's and stays,
//! so the shapes are decided here,
//! apart from the journal itself (`subvolume_journal.rs`) and its durable writes
//! (`subvolume_journal_writes.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const leftover = new RegExp(`^${escape(journalName)}\\.tmp(\\.\\d+\\.\\d+)?$`).test(candidate);
//! ```

/// What follows the journal's file name in the name of a temporary file:
/// releases up to 0.5.20 wrote exactly this suffix,
/// and
///  now a process number and a clock reading follow it.
pub(crate) const TEMPORARY_SUFFIX: &str = ".tmp";

/// What:
///  Whether `candidate` is a name under which a writer of the journal `journal_name` leaves a temporary file:
/// the journal's
///  name,
/// `.tmp`,
/// and either nothing (the name of releases up to 0.5.20) or a dot,
/// a process number,
/// a dot and a clock reading.
/// Why:
///   Any other name beside the journal is not this writer's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftover = new RegExp(`^${escape(journalName)}\\.tmp(\\.\\d+\\.\\d+)?$`).test(candidate);
/// ```
pub(crate) fn is_leftover_name(
    candidate: &[u8],
    journal_name: &[u8],
) -> bool {
    let Some(after_name) = candidate.strip_prefix(journal_name) else {
        return false;
    };
    let Some(rest) = after_name.strip_prefix(TEMPORARY_SUFFIX.as_bytes()) else {
        return false;
    };
    if rest.is_empty() {
        return true;
    }
    let Some(numbers) = rest.strip_prefix(b".") else {
        return false;
    };
    let mut parts = numbers.split(is_dot);
    let process: Option<&[u8]> = parts.next();
    let clock: Option<&[u8]> = parts.next();
    return is_number(process) && is_number(clock) && parts.next().is_none();
}

/// Whether `byte` is a dot,
/// the separator inside the numbers of a temporary name.
fn is_dot(byte: &u8) -> bool {
    return *byte == b'.';
}

/// Whether the part exists and is a non-empty run of ASCII digits.
fn is_number(part: Option<&[u8]>) -> bool {
    return match part {
        Some(digits) => !digits.is_empty() && digits.iter().all(u8::is_ascii_digit),
        None => false,
    };
}
