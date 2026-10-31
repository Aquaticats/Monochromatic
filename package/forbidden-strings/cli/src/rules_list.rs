//! The rules-list grammar: how `--rules`, `FORBIDDEN_STRINGS_RULES`, and `compile-rules --rules` name several rules
//! files in one value.
//!
//! The grammar is one function of the value, with no escape syntax:
//!
//! ```text
//! list  := entry ( "," entry )*
//! entry := path [ "?" ]
//! path  := one or more characters, none of them ","
//! ```
//!
//! - The value is split on every `,`, and nothing is trimmed.
//! - Exactly one trailing `?` marks an entry that may be absent;
//!    a path that itself ends in `?` is therefore nameable only as an optional entry (`name??`).
//! - An empty value, an empty entry (`a,,b`, a leading or trailing comma, a bare `?`), and a path named twice are
//!    errors, never silently dropped.
//! - A value with no `,` and no trailing `?` is one required entry equal to the whole value, which is the meaning
//!    it had before lists existed.
//!    A path holding a `,`, or a required path ending in `?`, cannot be named; that tradeoff was accepted
//!    when the list syntax was chosen.

use std::collections::HashMap;
use std::path::PathBuf;

/// The separator between entries.
const SEPARATOR: char = ',';
/// The suffix marking an entry that may be absent.
const OPTIONAL_MARKER: char = '?';

/// One rules file named by a list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulesEntry {
    /// The file to read, exactly as written (relative paths resolve against the process working directory).
    pub path: PathBuf,
    /// Whether the entry may be absent;
    ///  an absent optional entry is skipped and logged, an absent required entry is an error.
    pub optional: bool,
}

/// Why a rules-list value was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RulesListError {
    /// The whole value is empty.
    Empty,
    /// An entry names no path: an empty entry or a bare `?`.
    EmptyEntry {
        /// 1-based position of the entry in the list.
        position: usize,
    },
    /// The same path is named by two entries, whether or not either is optional.
    DuplicatePath {
        /// 1-based position of the first entry naming the path.
        first: usize,
        /// 1-based position of the repeated entry.
        position: usize,
    },
}

/// Renders the refusal as one line that quotes no path text.
impl std::fmt::Display for RulesListError {
    /// Writes the refusal with the entry position, so a long list stays diagnosable.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => return write!(formatter, "the rules list is empty; name at least one rules file"),
            Self::EmptyEntry { position } => {
                return write!(
                    formatter,
                    "rules list entry {position} names no file (an empty entry between commas, or a bare '?')",
                );
            }
            Self::DuplicatePath { first, position } => {
                return write!(
                    formatter,
                    "rules list entry {position} names the same file as entry {first}",
                );
            }
        }
    }
}

/// Lets the refusal travel through `anyhow` and `?`.
impl std::error::Error for RulesListError {}

/// Parses a rules-list value into its ordered entries.
///
/// A value without `,` or a trailing `?` yields one required entry holding the whole value.
///
/// # Errors
///
/// [`RulesListError::Empty`] for an empty value, [`RulesListError::EmptyEntry`] for an entry without a path, and
/// [`RulesListError::DuplicatePath`] when two entries name the same path.
pub fn parse_rules_list(value: &str) -> Result<Vec<RulesEntry>, RulesListError> {
    if value.is_empty() {
        return Err(RulesListError::Empty);
    }
    let mut entries: Vec<RulesEntry> = Vec::new();
    // Maps each named path to its first position, so duplicate detection stays linear in the value's length.
    let mut first_positions: HashMap<&str, usize> = HashMap::new();
    for (index, raw) in value.split(SEPARATOR).enumerate() {
        let position: usize = index + 1;
        let (name, optional): (&str, bool) = match raw.strip_suffix(OPTIONAL_MARKER) {
            Some(without_marker) => (without_marker, true),
            None => (raw, false),
        };
        if name.is_empty() {
            return Err(RulesListError::EmptyEntry { position });
        }
        if let Some(&first) = first_positions.get(name) {
            return Err(RulesListError::DuplicatePath { first, position });
        }
        first_positions.insert(name, position);
        entries.push(RulesEntry { path: PathBuf::from(name), optional });
    }
    return Ok(entries);
}

/// Registers the grammar and adversarial-boundary tests (sidecar, lint-exempt).
#[cfg(test)]
#[path = "rules_list_tests.rs"]
mod tests;
