//! What:
//!  Reads the major version out of Mozilla's `FIREFOX_ESR` value, for example `153.6.0esr` gives 153.
//! Why:
//!  The decision record pins the ESR major from Mozilla product-details instead of browserslist's constant,
//!  and a value of any other shape must fail with a diagnostic instead of guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const major = Number(/^(\d+)\.\d+\.\d+esr$/.exec(raw)?.[1]);
//! ```

use crate::error::{BrowserslistError, Outcome};
use crate::js_number::parse_int_prefix;

/// Check that a text is a non-empty run of ASCII digits.
fn is_digits(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
}

/// What:
///  Return the major version of a `<major>.<minor>.<patch>esr` value, or a diagnostic naming the bad value.
/// Why:
///  The lock and the update command share one check, so a hand-edited lock fails like a bad download does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseEsrMajor(raw: string): number; // throws when raw is not <major>.<minor>.<patch>esr
/// ```
pub fn parse_esr_major(raw: &str) -> Outcome<u64> {
    let failure = || {
        return BrowserslistError::Data {
            file: "Firefox ESR value".to_string(),
            offset: None,
            message: format!(
                "{raw:?} is not shaped <major>.<minor>.<patch>esr with decimal numbers (for example \"153.6.0esr\"); \
                 Mozilla product-details may have changed its format, so check \
                 https://product-details.mozilla.org/1.0/firefox_versions.json before editing the lock"
            ),
        };
    };
    let Some(numbers) = raw.strip_suffix("esr") else {
        return Err(failure());
    };
    let parts: Vec<&str> = numbers.split('.').collect();
    if parts.len() != 3 || !parts.iter().all(|part| is_digits(part)) {
        return Err(failure());
    }
    return parse_int_prefix(parts[0]).ok_or_else(failure);
}

#[cfg(test)]
#[path = "firefox_esr_tests.rs"]
mod tests;
