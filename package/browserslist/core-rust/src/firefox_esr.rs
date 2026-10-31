//! What:
//!  Reads the major version out of Mozilla's `FIREFOX_ESR` value,
//! for example `153.6.0esr` gives 153.
//! Why:
//!  The decision record pins the ESR major from Mozilla product-details instead of browserslist's constant,
//!  and a value of any other shape must fail with a diagnostic instead of guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const major = Number(/^(\d+)\.\d+\.\d+esr$/.exec(raw)?.[1]);
//! ```

/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `parse_int_prefix` from `crate::js_number::parse_int_prefix`.
/// Why:
///  Version comparison needs JavaScript's number-prefix reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseIntPrefix } from 'crate/js_number';
/// ```
use crate::js_number::parse_int_prefix;

/// Whether a byte is an ASCII digit.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Check that a text is a non-empty run of ASCII digits.
fn is_digits(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(is_digit_byte);
}

/// Check that every part is a non-empty run of ASCII digits.
fn all_digits(parts: &[&str]) -> bool {
    for part in parts {
        if !is_digits(part) {
            return false;
        }
    }
    return true;
}

/// The error for a value that is not shaped `<major>.<minor>.<patch>esr`.
fn shape_error(raw: &str) -> BrowserslistError {
    return BrowserslistError::Data {
        file: "Firefox ESR value".to_string(),
        offset: None,
        message: format!(
            "{raw:?} is not shaped <major>.<minor>.<patch>esr with decimal numbers (for example \"153.6.0esr\"); \
             Mozilla product-details may have changed its format, so check \
             https://product-details.mozilla.org/1.0/firefox_versions.json before editing the lock"
        ),
    };
}

/// What:
///  Return the major version of a `<major>.<minor>.<patch>esr` value,
/// or a diagnostic naming the bad value.
/// Why:
///  The lock and the update command share one check,
/// so a hand-edited lock fails like a bad download does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseEsrMajor(raw: string): number; // throws when raw is not <major>.<minor>.<patch>esr
/// ```
pub fn parse_esr_major(raw: &str) -> Outcome<u64> {
    let Some(numbers) = raw.strip_suffix("esr") else {
        return Err(shape_error(raw));
    };
    let parts: Vec<&str> = numbers.split('.').collect();
    if parts.len() != 3 || !all_digits(&parts) {
        return Err(shape_error(raw));
    }
    let Some(major) = parse_int_prefix(parts[0]) else {
        return Err(shape_error(raw));
    };
    return Ok(major);
}

/// What:
///  The unit tests of `firefox_esr.rs`,
/// kept in the sidecar file `firefox_esr_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './firefox_esr.test';
/// ```
#[cfg(test)]
#[path = "firefox_esr_tests.rs"]
mod tests;
