//! What:
//!  The proof that every configuration value Git types at startup, in every scope the readers look at, is a value
//!  Git accepts without a message.
//! Why:
//!  Git reads many keys for every command (`core.fileMode`, `core.compression`, `advice.*`, `diff.context`, and so
//!  on) and dies (`fatal: bad boolean config value 'maybe' for 'core.filemode'`), or warns, when a value does not
//!  fit the key's type.
//!  The readers never look at most of those keys, so without this proof they would answer where Git fails.
//!  The proof accepts only spellings it can show Git accepts (a conservative subset) and leaves every other
//!  spelling of a typed key to Git, which then words its own failure.
//!  The key table was built from the keys real Git 2.56.0 rejected or warned about when set to malformed values
//!  (`doc/planning/cli-git-in-process-reads.md`, "Typed configuration values"), and
//!  `every_documented_key_with_a_malformed_value_is_declined_or_equals_git` keeps it honest against the Git that
//!  runs the tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ok: boolean | undefined = everyTypedValueIsAccepted({ entries, scope }); // undefined: ask Git
//! ```

use crate::repo_read_config_scan::{ScopeEnvironment, include_target, is_include_key, last_variable, read_following};
use crate::repo_read_config_text::{ConfigEntry, parse_config_text};
use std::ffi::OsString;
use std::path::Path;
use crate::repo_read_config_value_kinds::{ATTRIBUTE_WORDS, BRIGHT_COLOR_WORDS, FALSE_WORDS, Kind, MAXIMUM_ABBREVIATION, MINIMUM_ABBREVIATION, PLAIN_COLOR_WORDS, TRUE_WORDS, kind_of};

/// How many nested includes are followed before the proof gives up; Git's own limit.
const INCLUDE_DEPTH_LIMIT: usize = 10;

/// How many environment-supplied entries are checked before the proof gives up.
const ENV_ENTRY_LIMIT: u64 = 1024;

/// The most digits of a number the proof accepts, so no accepted spelling can overflow the integer Git parses into.
const DIGIT_LIMIT: usize = 9;

/// The suffixes Git multiplies a number by, with their factors.
const UNIT_FACTORS: [(u8, i64); 3] = [(b'k', 1 << 10), (b'm', 1 << 20), (b'g', 1 << 30)];

/// The decimal number a spelling stands for, or `None` for a spelling the proof does not read as one.
///  A sign, 1 to 9 digits with no leading zero (Git reads a leading zero as octal), and one optional unit suffix.
fn number_of(text: &str) -> Option<i64> {
    let bytes: &[u8] = text.as_bytes();
    let (negative, rest): (bool, &[u8]) = match bytes.first() {
        Some(b'-') => (true, &bytes[1..]),
        Some(b'+') => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    let (digits, factor): (&[u8], i64) = match rest.split_last() {
        Some((last, head)) => match unit_factor(*last) {
            Some(factor) => (head, factor),
            None => (rest, 1),
        },
        None => (rest, 1),
    };
    if digits.is_empty() || digits.len() > DIGIT_LIMIT || !digits.iter().all(u8::is_ascii_digit) || (digits.len() > 1 && digits[0] == b'0') {
        return None;
    }
    let magnitude: i64 = std::str::from_utf8(digits).ok()?.parse::<i64>().ok()?.checked_mul(factor)?;
    return Some(if negative { -magnitude } else { magnitude });
}

/// Whether `text` is a spelling of a boolean Git reads: a word, an empty value, or a number.
fn is_boolean_text(text: &str) -> bool {
    let lowered: String = text.to_ascii_lowercase();
    return text.is_empty()
        || TRUE_WORDS.contains(&lowered.as_str())
        || FALSE_WORDS.contains(&lowered.as_str())
        || fits_in_i32(text);
}

/// The factor of the unit letter `last` (`k`, `m`, `g`, in either case), or nothing for any other byte.
fn unit_factor(last: u8) -> Option<i64> {
    for (unit, factor) in UNIT_FACTORS.iter() {
        if *unit == last.to_ascii_lowercase() {
            return Some(*factor);
        }
    }
    return None;
}

/// Whether `text` reads as a whole number that fits in 32 bits, with the unit letters Git allows.
fn fits_in_i32(text: &str) -> bool {
    return match number_of(text) {
        Some(number) => i32::try_from(number).is_ok(),
        None => false,
    };
}

/// Whether `text` reads as a whole number from `low` to `high`, both included.
fn number_within(text: &str, low: i64, high: i64) -> bool {
    return match number_of(text) {
        Some(number) => (low..=high).contains(&number),
        None => false,
    };
}

/// Whether `byte` is an ASCII hexadecimal digit.
fn is_hex_byte(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// Whether `byte` may appear in a component of a ref name this check takes: a letter, a digit, `_`, `-` or `.`.
fn is_ref_component_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.');
}

/// Whether `text` is a false word or empty, the spellings Git reads as false without a number.
fn is_false_text(text: &str) -> bool {
    return text.is_empty() || FALSE_WORDS.contains(&text.to_ascii_lowercase().as_str()) || number_of(text) == Some(0);
}

/// Whether `lowered` is an attribute word, with or without a `no` or `no-` prefix.
fn is_attribute_word(lowered: &str) -> bool {
    let bare: &str = match lowered.strip_prefix("no-") {
        Some(rest) => rest,
        None => lowered.strip_prefix("no").unwrap_or(lowered),
    };
    return ATTRIBUTE_WORDS.contains(&lowered) || ATTRIBUTE_WORDS.contains(&bare);
}

/// Whether one color word, attribute word, number or `#rrggbb` spelling is one Git reads.
fn is_color_word(word: &str) -> bool {
    let lowered_text: String = word.to_ascii_lowercase();
    let lowered: &str = lowered_text.as_str();
    if PLAIN_COLOR_WORDS.contains(&lowered) || BRIGHT_COLOR_WORDS.contains(&lowered) || is_attribute_word(lowered) {
        return true;
    }
    if let Some(base) = lowered.strip_prefix("bright")
        && BRIGHT_COLOR_WORDS.contains(&base)
    {
        return true;
    }
    if let Some(hex) = lowered.strip_prefix('#') {
        return (hex.len() == 6 || hex.len() == 3) && hex.bytes().all(is_hex_byte);
    }
    return number_within(word, 0, 255);
}

/// Whether `text` is at most two colors and any number of attributes, each a word Git reads.
fn is_color_text(text: &str) -> bool {
    let words: Vec<&str> = text.split_ascii_whitespace().collect();
    let mut colors: usize = 0;
    for word in &words {
        if !is_attribute_word(word.to_ascii_lowercase().as_str()) {
            colors += 1;
        }
    }
    for word in &words {
        if !is_color_word(word) {
            return false;
        }
    }
    return colors <= 2;
}

/// Whether `name` is a ref name or pattern of plain components under `refs/`, with at most one wildcard component.
fn is_plain_ref_pattern(name: &str) -> Option<bool> {
    let rest: &str = name.strip_prefix("refs/")?;
    let mut wildcards: usize = 0;
    for component in rest.split('/') {
        if component == "*" {
            wildcards += 1;
            continue;
        }
        let plain: bool = !component.is_empty()
            && !component.starts_with('.')
            && !component.contains("..")
            && !component.ends_with(".lock")
            && !component.ends_with('.')
            && component.bytes().all(is_ref_component_byte);
        if !plain {
            return None;
        }
    }
    return if wildcards > 1 { None } else { Some(wildcards == 1) };
}

/// Whether `text` is `[+]refs/...:refs/...` with plain names and a wildcard on both sides or on neither.
fn is_plain_refspec(text: &str) -> bool {
    let Some((source, destination)) = text.strip_prefix('+').unwrap_or(text).split_once(':') else {
        return false;
    };
    return match (is_plain_ref_pattern(source), is_plain_ref_pattern(destination)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    };
}

/// Whether `text` is a comma-separated list of the listed words.
fn is_word_list(text: &str, words: &[&str], empty: bool, negatable: bool) -> bool {
    if text.is_empty() {
        return empty;
    }
    for item in text.split(',') {
        let word: &str = if negatable { item.strip_prefix('-').unwrap_or(item) } else { item };
        if !words.contains(&word) {
            return false;
        }
    }
    return true;
}

/// Whether `value` is a spelling of `kind` that Git accepts silently.
fn is_accepted(kind: Kind, value: Option<&[u8]>) -> bool {
    let Some(bytes) = value else {
        // A bare name is a true boolean and nothing else.
        return matches!(kind, Kind::Boolean | Kind::BooleanOr(_) | Kind::IntegerOrBoolean);
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    return match kind {
        Kind::Boolean => is_boolean_text(text),
        Kind::BooleanOr(words) => is_boolean_text(text) || words.contains(&text),
        Kind::IntegerOrBoolean => is_boolean_text(text) || fits_in_i32(text),
        Kind::Integer { low, high } => number_within(text, low, high),
        Kind::Text => true,
        Kind::Pathname => !text.starts_with('~') || text.starts_with("~/"),
        Kind::OneOf { words, empty } => (text.is_empty() && empty) || words.contains(&text),
        Kind::Words { words, empty, negatable } => is_word_list(text, words, empty, negatable),
        Kind::Color => is_color_text(text),
        Kind::Abbreviation => text == "auto" || is_false_text(text) || number_within(text, MINIMUM_ABBREVIATION, MAXIMUM_ABBREVIATION),
        Kind::CommentText => !text.is_empty() && !text.contains('\n'),
        Kind::TraceTarget => text.is_empty() || text == "0" || text.eq_ignore_ascii_case("false"),
        Kind::OffOrBuiltinDaemon => is_false_text(text) || TRUE_WORDS.contains(&text.to_ascii_lowercase().as_str()),
        Kind::Refspec => is_plain_refspec(text),
        Kind::Refused => false,
    };
}

/// What:
///  Whether Git accepts the value of one entry silently.
/// Why:
///   A key the table does not list is read by no startup code of the commands the readers replace, so it passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const accepted = kindOf(entry.key) === undefined || isAccepted(kindOf(entry.key), entry.value);
/// ```
pub fn entry_is_accepted(key: &str, value: Option<&[u8]>) -> bool {
    return match kind_of(key) {
        Some(kind) => is_accepted(kind, value),
        None => true,
    };
}

/// Whether the entries parsed from `path`, and every file they include, hold only accepted values;
///  `None` when a file cannot be read or parsed.
pub fn entries_are_accepted(entries: &[ConfigEntry], path: &Path, home: Option<&Path>, depth: usize) -> Option<bool> {
    if depth > INCLUDE_DEPTH_LIMIT {
        return None;
    }
    for entry in entries {
        if !entry_is_accepted(entry.key.as_str(), entry.value.as_deref()) {
            return Some(false);
        }
        if is_include_key(entry.key.as_str()) {
            let target = include_target(entry.value.as_deref()?, path, home)?;
            if !file_is_accepted(target.as_path(), home, depth + 1)? {
                return Some(false);
            }
        }
    }
    return Some(true);
}

/// The same proof for the file at `path`; a missing file is an empty file to Git.
fn file_is_accepted(path: &Path, home: Option<&Path>, depth: usize) -> Option<bool> {
    let Some(text) = read_following(path)? else {
        return Some(true);
    };
    let entries: Vec<ConfigEntry> = parse_config_text(text.as_slice())?;
    return entries_are_accepted(entries.as_slice(), path, home, depth);
}

/// What:
///  Whether the system and global files, and everything they include, hold only accepted values.
/// Why:
///   These scopes feed the same startup reads as the repository's own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scopeFiles(env).every((file) => fileIsAccepted(file)); // undefined when unreadable
/// ```
pub fn outer_scopes_are_accepted(scope: &ScopeEnvironment) -> Option<bool> {
    if scope.unresolvable {
        return None;
    }
    for file in scope.files() {
        if !file_is_accepted(file.as_path(), scope.home.as_deref(), 0)? {
            return Some(false);
        }
    }
    return Some(true);
}

/// The key as Git canonicalizes it: lower-case section and name, subsection kept.
fn canonical(key: &str) -> Option<String> {
    let (section, rest) = key.split_once('.')?;
    return Some(match rest.rsplit_once('.') {
        Some((subsection, name)) => format!("{}.{subsection}.{}", section.to_ascii_lowercase(), name.to_ascii_lowercase()),
        None => format!("{}.{}", section.to_ascii_lowercase(), rest.to_ascii_lowercase()),
    });
}

/// What:
///  Whether every `GIT_CONFIG_COUNT` entry among `variables` holds an accepted value.
/// Why:
///   Environment entries reach the same startup reads; a malformed set of entries is not accepted either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// envEntries(variables).every(({ key, value }) => entryIsAccepted(key, value)); // false when malformed
/// ```
pub fn environment_is_accepted(variables: &[(OsString, OsString)]) -> bool {
    let Some(count_text) = last_variable(variables, "GIT_CONFIG_COUNT") else {
        return true;
    };
    let Some(count) = crate::child_environment::parse_config_count(count_text.as_os_str()) else {
        return false;
    };
    if count > ENV_ENTRY_LIMIT {
        return false;
    }
    for index in 0..count {
        let (Some(key), Some(value)) = (last_variable(variables, format!("GIT_CONFIG_KEY_{index}").as_str()), last_variable(variables, format!("GIT_CONFIG_VALUE_{index}").as_str())) else {
            return false;
        };
        let Some(key_text) = key.to_str() else {
            return false;
        };
        let Some(canonical_key) = canonical(key_text) else {
            return false;
        };
        if !entry_is_accepted(canonical_key.as_str(), Some(value.as_encoded_bytes())) {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
#[path = "repo_read_config_typed_tests.rs"]
mod tests;
