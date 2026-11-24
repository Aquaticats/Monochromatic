//! What:
//!  The canonical decimal spelling a destruction plan's numbers take:
//!  unsigned 64-bit values with no sign,
//!  no leading zero and no byte that is not a digit,
//!  and the digit count of one value.
//! Why:
//!  The plan's reader refuses a value with two spellings,
//!  because a GUID or a transaction group that could be written two ways would make two
//!  records of one object,
//!  and the merge rule compares attempt numbers as numbers.
//!  The grammar's own digit bound is what refuses a number too wide for its line,
//!  so the parse takes the bound rather than reading an unbounded string.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const isCanonical = (text: string, bound: number) => /^(0|[1-9][0-9]*)$/.test(text) && text.length <= bound;
//! ```

/// Whether `text` is a canonical unsigned decimal with at most `maximum_digits` digits.
///
/// Why:
///  `0` is its own spelling and `00`, `+1` and `007` are not numbers this grammar holds.
pub fn is_canonical_decimal(text: &str, maximum_digits: usize) -> bool {
    if text.is_empty() || text.len() > maximum_digits {
        return false;
    }
    if !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return false;
    }
    return text == "0" || !text.starts_with('0');
}

/// What:
///  One canonical unsigned decimal read as a number.
/// Why:
///  A caller that holds the digits and the number cannot disagree with itself about which
///  value a line carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parseCanonical = (text, bound) => isCanonical(text, bound) ? BigInt(text) : undefined;
/// ```
pub fn parse_canonical_u64(text: &str, maximum_digits: usize) -> Option<u64> {
    if !is_canonical_decimal(text, maximum_digits) {
        return None;
    }
    return text.parse::<u64>().ok();
}

/// How many decimal digits `number` takes,
/// which is one for zero.
pub fn decimal_digits(number: u64) -> usize {
    let mut digits: usize = 1;
    let mut left: u64 = number;
    while left >= 10 {
        left /= 10;
        digits += 1;
    }
    return digits;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "canonical_decimal_tests.rs"]
mod tests;
