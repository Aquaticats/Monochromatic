//! What:
//!  The `discardedCopies` section of `cli-git.config.jsonc`: how long and how much the folder of saved copies of conflicted
//!  files may keep (`maxAgeDays`, `maxSize`).
//! Why:
//!  The settings of the wrapper live in one file with one validation, so a limit is a key there and not a second source in
//!  Git configuration. A value that is no limit (zero, negative, a fraction, text that is not a size) is a configuration
//!  failure that names the key, like every other bad value, and so is never read as "no limit" or as a zero that would remove
//!  every copy. Pruning itself is `discarded_prune.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limits = parseDiscardedCopies({ maxAgeDays: 90, maxSize: '256m' }); // { maxAgeDays: 90, maxBytes: 268435456 }
//! ```

use crate::config_error::ConfigError;
use crate::config_values::{member_keys, member_path, safe_integer, text, wrong_kind};
use crate::discarded_prune::{Limits, parse_size};
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue};

/// The name of the section.
pub const SECTION: &str = "discardedCopies";

/// The key for the age limit, in whole days from 1.
pub const AGE_KEY: &str = "maxAgeDays";

/// The key for the size limit: a whole number of bytes from 1, or text such as `"256m"` (Git's `k`, `m`, `g` suffixes).
pub const SIZE_KEY: &str = "maxSize";

/// What:
///  Read the size limit: a JSON number of bytes, or a string in Git's integer spelling.
/// Why:
///   `"1g"` is how a person writes a gibibyte; a plain number is accepted so that a script can write the exact byte count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = typeof value === 'number' ? safeInteger(value, 1) : parseSize(value);
/// ```
fn size_limit(value: &JsoncValue, path: &str) -> Result<u64, ConfigError> {
    if matches!(value.kind, JsoncKind::Number { .. }) {
        return safe_integer(value, path, 1);
    }
    if !matches!(value.kind, JsoncKind::Text { .. }) {
        return Err(wrong_kind(value, path, "a number of bytes or a size such as \"256m\""));
    }
    let spelled: String = text(value, path)?;
    return match parse_size(spelled.as_str()) {
        Some(size) => Ok(size),
        None => Err(ConfigError::new(
            format!("Configuration key {path} must be a size above zero in bytes or with k, m or g (for example \"256m\"), found {spelled:?}.").as_str(),
        )),
    };
}

/// What:
///  Validate the `discardedCopies` section and apply its keys over the defaults.
/// Why:
///   Both keys are optional; an unknown key is refused with the accepted ones named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDiscardedCopies(value: JsoncValue): Limits;
/// ```
pub fn parse_discarded_copies(value: &JsoncValue) -> Result<Limits, ConfigError> {
    let Some(entries) = value.entries() else {
        return Err(wrong_kind(value, SECTION, "an object"));
    };
    let keys: Vec<String> = member_keys(entries, SECTION)?;
    let mut limits: Limits = Limits::default();
    for (index, key) in keys.iter().enumerate() {
        let path: String = member_path(SECTION, key.as_str());
        let member: &JsoncValue = &entries[index].value;
        if key == AGE_KEY {
            limits.max_age_days = safe_integer(member, path.as_str(), 1)?;
        } else if key == SIZE_KEY {
            limits.max_bytes = size_limit(member, path.as_str())?;
        } else {
            return Err(ConfigError::new(format!("Unknown configuration key: {path}. The accepted keys under {SECTION} are {AGE_KEY}, {SIZE_KEY}.").as_str()));
        }
    }
    return Ok(limits);
}

/// Section controls stay out of the release executable.
#[cfg(test)]
#[path = "config_discarded_tests.rs"]
mod tests;
