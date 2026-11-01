//! What:
//!      Decimal digit-string arithmetic for unbounded JSON number exponents.
//!           A JSON number exponent may carry more digits than any machine integer holds,
//!  so this
//!           module adds and subtracts decimal magnitudes as byte strings (`&str`,
//!  a borrowed
//!           read-only view of UTF-8 text,
//!  not an owned `String` or a fixed `[u8; N]` array).
//! Why:
//!       The exact-number identity must shift an exponent by a digit count without ever
//!           widening it into `i64`,
//!  `i128` or `isize`,
//!  which would silently reject or wrap a
//!           legal JSON literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module numberScale: addDigits(a, b), subtractDigits(a, b), signedSum(...) over decimal strings.
//! ```

/// What:
///      Report whether one byte is an ASCII decimal digit.
/// Why:
///       JSON number grammar admits only ASCII digits,
///  so Unicode numeral characters must be
///           rejected rather than normalized.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isDigit(byte: number): boolean { return byte >= 48 && byte <= 57; }
/// ```
pub(crate) fn is_digit(byte: u8) -> bool {
    // What:     `is_ascii_digit` is the standard-library byte classification.
    // Why:      It rejects non-ASCII numerals without a regular expression.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return byte >= 48 && byte <= 57;
    // ```
    return byte.is_ascii_digit();
}

/// What:
///      Add two non-negative decimal digit strings and return the owned sum.
/// Why:
///       Column addition keeps arbitrary exponent widths exact,
///  the way `bigint` addition does
///           in TypeScript.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addDigits(a: string, b: string): string { /* column addition */ }
/// ```
pub(crate) fn add_digits(left: &str, right: &str) -> String {
    // What:     `as_bytes` borrows each string's UTF-8 bytes without copying or transferring
    //           ownership.
    // Why:      Validated decimal digits are one byte each, so byte indexing is character indexing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const leftBytes = [...a].map((ch) => ch.charCodeAt(0));
    // ```
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    // What:     `Vec<u8>` is an owned growable byte array, unlike a borrowed `&[u8]` slice or a
    //           fixed `[u8; N]` array.
    // Why:      The sum is built from the least significant digit upward, then reversed once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reversed: number[] = [];
    // ```
    let mut reversed: Vec<u8> = Vec::new();
    let mut left_cursor = left_bytes.len();
    let mut right_cursor = right_bytes.len();
    let mut carry: u8 = 0;
    while left_cursor > 0 || right_cursor > 0 || carry > 0 {
        let mut sum: u8 = carry;
        if left_cursor > 0 {
            left_cursor -= 1;
            sum += left_bytes[left_cursor] - b'0';
        }
        if right_cursor > 0 {
            right_cursor -= 1;
            sum += right_bytes[right_cursor] - b'0';
        }
        reversed.push(b'0' + sum % 10);
        carry = sum / 10;
    }
    reversed.reverse();
    // What:     `String::from_utf8` validates the bytes and returns a `Result`; `expect` unwraps it
    //           or panics with this message.
    // Why:      Only ASCII digits were pushed, so a failure would mean a bug in this module.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return String.fromCharCode(...reversed);
    // ```
    return String::from_utf8(reversed).expect("decimal digit sum is ASCII");
}

/// What:
///      Subtract a smaller non-negative decimal string from a larger one.
/// Why:
///       Opposing exponent signs combine by magnitude difference,
///  again without a fixed-width
///           integer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function subtractDigits(a: string, b: string): string { /* requires a >= b */ }
/// ```
pub(crate) fn subtract_digits(left: &str, right: &str) -> String {
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    let mut reversed: Vec<u8> = Vec::new();
    let mut left_cursor = left_bytes.len();
    let mut right_cursor = right_bytes.len();
    let mut borrow: i16 = 0;
    while left_cursor > 0 {
        left_cursor -= 1;
        let mut digit: i16 = i16::from(left_bytes[left_cursor] - b'0') - borrow;
        if right_cursor > 0 {
            right_cursor -= 1;
            digit -= i16::from(right_bytes[right_cursor] - b'0');
        }
        if digit < 0 {
            digit += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        reversed.push(b'0' + u8::try_from(digit).expect("one decimal digit"));
    }
    while reversed.len() > 1 && reversed.last() == Some(&b'0') {
        reversed.pop();
    }
    reversed.reverse();
    return String::from_utf8(reversed).expect("decimal digit difference is ASCII");
}

/// What:
///      Add two signed decimal magnitudes and normalize a zero result.
/// Why:
///       An exponent shifts by the count of trailing zeros and fraction digits,
///  and that shift
///           can be positive or negative.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function signedSum(aNegative: boolean, a: string, bNegative: boolean, b: string): string;
/// ```
pub(crate) fn signed_sum(
    left_negative: bool,
    left: &str,
    right_negative: bool,
    right: &str,
) -> String {
    // What:     `trim_start_matches` borrows a shorter view of the same string.
    // Why:      Exponent digits may carry leading zeros that must not affect magnitude comparison.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const aTrimmed = a.replace(/^0+/, '');
    // ```
    let left_trimmed = left.trim_start_matches('0');
    let right_trimmed = right.trim_start_matches('0');
    let left_digits = if left_trimmed.is_empty() { "0" } else { left_trimmed };
    let right_digits = if right_trimmed.is_empty() { "0" } else { right_trimmed };
    let result_negative;
    let magnitude;
    if left_negative == right_negative {
        magnitude = add_digits(left_digits, right_digits);
        result_negative = left_negative;
    } else {
        // What:     Length-then-lexical comparison orders two zero-free decimal magnitudes.
        // Why:      The larger operand must be the minuend so the difference stays non-negative.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const leftLarger = a.length > b.length || (a.length === b.length && a >= b);
        // ```
        let left_larger = left_digits.len() > right_digits.len()
            || (left_digits.len() == right_digits.len() && left_digits >= right_digits);
        if left_larger {
            magnitude = subtract_digits(left_digits, right_digits);
            result_negative = left_negative;
        } else {
            magnitude = subtract_digits(right_digits, left_digits);
            result_negative = right_negative;
        }
    }
    if magnitude == "0" {
        return String::from("0");
    }
    if result_negative {
        return format!("-{magnitude}");
    }
    return magnitude;
}
