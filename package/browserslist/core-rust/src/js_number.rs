//! What:
//!  Rust ports of the three JavaScript number readers browserslist leans on: `parseInt`, `parseFloat`,
//!  and its own `parseLatestFloat`.
//! Why:
//!  Version comparisons in browserslist read the longest numeric prefix of a string and ignore the rest,
//!  which Rust's strict `str::parse` does not do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! parseInt('4.4.3'); parseFloat('15.2-15.3'); // these already exist in TS; Rust has no equivalent
//! ```

/// What:
///  JavaScript `parseInt(text)` for non-negative decimal text:
///  the leading run of ASCII digits, or `None` where JavaScript yields `NaN`.
///  `u64` (not `u32` or `i64`) holds any digit run browserslist data produces without a sign bit to misuse.
/// Why:
///  Version parts like `4` in `4.4.3` and majors of Node releases are read this way.
/// Gotcha:
///  A run too long for `u64` also gives `None`; real data never gets near it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = parseInt(text); return Number.isNaN(value) ? undefined : value;
/// ```
pub fn parse_int_prefix(text: &str) -> Option<u64> {
    let digits: &str = &text[..text.bytes().take_while(u8::is_ascii_digit).count()];
    if digits.is_empty() {
        return None;
    }
    return digits.parse::<u64>().ok();
}

/// What:
///  JavaScript `parseFloat(text)` for the decimal forms version strings take:
///  an optional sign, digits with an optional fraction, and an optional exponent;
///  anything after that prefix is ignored, and no prefix gives `NaN`.
/// Why:
///  Ray queries such as `ie <= 11` compare versions like `5.5` and `15.2-15.3` this way.
/// Gotcha:
///  JavaScript also reads the word `Infinity`; no version string starts with it, so it yields `NaN` here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseFloat(text)
/// ```
pub fn parse_float(text: &str) -> f64 {
    let bytes = text.as_bytes();
    let mut end = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        end += 1;
    }
    let digits_start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut digit_count = end - digits_start;
    if bytes.get(end) == Some(&b'.') {
        let fraction_start = end + 1;
        let mut fraction_end = fraction_start;
        while bytes.get(fraction_end).is_some_and(u8::is_ascii_digit) {
            fraction_end += 1;
        }
        if fraction_end > fraction_start || digit_count > 0 {
            digit_count += fraction_end - fraction_start;
            end = fraction_end;
        }
    }
    if digit_count == 0 {
        return f64::NAN;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent_end = end + 1;
        if matches!(bytes.get(exponent_end), Some(b'+' | b'-')) {
            exponent_end += 1;
        }
        let exponent_digits = exponent_end;
        while bytes.get(exponent_end).is_some_and(u8::is_ascii_digit) {
            exponent_end += 1;
        }
        if exponent_end > exponent_digits {
            end = exponent_end;
        }
    }
    return text[..end].parse::<f64>().unwrap_or(f64::NAN);
}

/// What:
///  browserslist's `parseLatestFloat(v)`: `parseFloat(v.split('-')[1] || v)`.
///  For a joined range such as `15.2-15.3` it reads the upper end.
/// Why:
///  The `>=` ray form compares against a range's newest member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseFloat(v.split('-')[1] || v)
/// ```
pub fn parse_latest_float(text: &str) -> f64 {
    let after_dash = text.split('-').nth(1).unwrap_or("");
    if after_dash.is_empty() {
        return parse_float(text);
    }
    return parse_float(after_dash);
}

#[cfg(test)]
#[path = "js_number_tests.rs"]
mod tests;
