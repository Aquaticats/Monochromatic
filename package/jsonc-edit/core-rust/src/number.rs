//! What:
//!      The exact mathematical identity of one JSON number literal.
//!           An identity stores a sign,
//!  the significant digits without zero padding,
//!  and a signed
//!           base-ten exponent kept as decimal text (`String`,
//!  an owned growable UTF-8 buffer,
//!  not a
//!           borrowed `&str` or a fixed-width integer).
//! Why:
//!       `1`,
//!  `1.0` and `1e0` must compare equal while `9007199254740992` and
//!           `9007199254740993` stay distinct,
//!  and a JSON exponent may carry more digits than
//!           `i64`,
//!  `i128` or `isize` holds.
//!  Keeping the exponent as text compares magnitudes
//!           without ever materializing `10^exponent`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module number: type JsoncNumberIdentity = { negative: boolean; digits: string; exponent: string };
//! ```

/// What:
///      Import the crate's number failure type.
/// Why:
///       Grammar rejection and identity construction are one operation,
///  so they share one error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncNumberError } from './error';
/// ```
use crate::error::JsoncNumberError;
/// What:
///      Import the digit-string arithmetic and the ASCII digit test.
/// Why:
///       Exponent adjustment must stay exact for digit runs wider than any machine integer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isDigit, signedSum } from './numberScale';
/// ```
use crate::number_scale::{is_digit, signed_sum};

/// What:
///      One number's canonical mathematical value,
///  independent of how it was spelled.
/// Why:
///       Equality and hashing must follow the value,
///  while the caller keeps the original token
///           separately so an unedited literal is emitted exactly as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncNumberIdentity = { negative: boolean; digits: string; exponent: string };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsoncNumberIdentity {
    /// What:
    ///      Sign after every zero spelling collapses to positive zero.
    /// Why:
    ///       `-0` and `0` denote the same mathematical value,
    ///  so the sign cannot distinguish
    ///           them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// negative: boolean;
    /// ```
    negative: bool,
    /// What:
    ///      Significant digits with no leading or trailing zero padding.
    /// Why:
    ///       Private so no caller can build a non-canonical identity that compares unequal to
    ///           an equivalent parsed value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// digits: string; // constructed only by from_token
    /// ```
    digits: String,
    /// What:
    ///      Signed decimal exponent as text,
    ///  never as a fixed-width integer.
    /// Why:
    ///       Arbitrary exponent widths stay representable and comparable.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// exponent: string; // constructed only by from_token
    /// ```
    exponent: String,
}

/// What:
///      The scanned offsets and signs of one number token's grammar parts.
/// Why:
///       Passing one record instead of nine positional arguments keeps the normalization call
///           readable and makes an empty range ("this part was absent") explicit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NumberParts = { negative: boolean; integer: [number, number]; fraction: [number, number]; exponentNegative: boolean; exponent: [number, number] };
/// ```
struct NumberParts {
    /// Whether the token carried a leading minus sign.
    negative: bool,
    /// Byte range of the integer digits;
    ///  empty when the token had none.
    integer: std::ops::Range<usize>,
    /// Byte range of the fraction digits;
    ///  empty when the token had no decimal point.
    fraction: std::ops::Range<usize>,
    /// Whether the exponent carried a minus sign.
    exponent_negative: bool,
    /// Byte range of the exponent digits;
    ///  empty when the token had no exponent marker.
    exponent: std::ops::Range<usize>,
}

/// What:
///      Build the canonical identity from one validated token and its scanned parts.
/// Why:
///       Keeping normalization separate from grammar scanning lets each stay readable and lets
///           tests exercise them independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalize(raw: string, parts: NumberParts): JsoncNumberIdentity;
/// ```
fn normalize(raw: &str, parts: &NumberParts) -> JsoncNumberIdentity {
    // What:     `format!` allocates one string holding the integer digits followed by the fraction
    //           digits.
    // Why:      Zero stripping and the exponent shift both need the digits in source order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const combined = `${integerDigits}${fractionDigits}`;
    // ```
    let combined = format!(
        "{}{}",
        &raw[parts.integer.clone()],
        &raw[parts.fraction.clone()]
    );
    let without_leading = combined.trim_start_matches('0');
    if without_leading.is_empty() {
        // What:     `JsoncNumberIdentity { .. }` is the canonical zero value.
        // Why:      `-0`, `0.0` and `0e1000` all denote zero, so they must compare equal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { negative: false, digits: '0', exponent: '0' };
        // ```
        return JsoncNumberIdentity {
            negative: false,
            digits: String::from("0"),
            exponent: String::from("0"),
        };
    }
    let significant = without_leading.trim_end_matches('0');
    let trailing_zero_count = without_leading.len() - significant.len();
    let fraction_digit_count = parts.fraction.len();
    let shift_positive = trailing_zero_count >= fraction_digit_count;
    let shift_magnitude = if shift_positive {
        trailing_zero_count - fraction_digit_count
    } else {
        fraction_digit_count - trailing_zero_count
    };
    let written_exponent = if parts.exponent.is_empty() {
        "0"
    } else {
        &raw[parts.exponent.clone()]
    };
    let adjusted = signed_sum(
        parts.exponent_negative,
        written_exponent,
        !shift_positive,
        shift_magnitude.to_string().as_str(),
    );
    return JsoncNumberIdentity {
        negative: parts.negative,
        digits: String::from(significant),
        exponent: adjusted,
    };
}

/// What:
///      The public behavior of one exact number identity.
/// Why:
///       Construction is restricted to validated tokens,
///  so callers get accessors and questions
///           instead of a way to build a non-canonical identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncNumberIdentity { static fromToken(raw: string): JsoncNumberIdentity }
/// ```
impl JsoncNumberIdentity {
    /// What:
    ///      Validate one complete JSON number token and return its canonical identity.
    /// Why:
    ///       Parsing and normalization must agree on the grammar,
    ///  so a caller cannot store a
    ///           token that the emitter would later be unable to compare.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromToken(raw: string): JsoncNumberIdentity { /* validate, then normalize */ }
    /// ```
    ///
    /// # Errors
    /// Returns the grammar part that the token violates,
    ///  so a caller can report it precisely.
    pub fn from_token(raw: &str) -> Result<JsoncNumberIdentity, JsoncNumberError> {
        // What:     `as_bytes` borrows the token's UTF-8 bytes.
        // Why:      JSON number grammar is ASCII, so byte offsets and character offsets agree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const bytes = [...raw].map((ch) => ch.charCodeAt(0));
        // ```
        let bytes = raw.as_bytes();
        if bytes.is_empty() {
            return Err(JsoncNumberError::Empty);
        }
        let mut cursor: usize = 0;
        let negative = bytes[0] == b'-';
        if negative || bytes[0] == b'+' {
            if bytes[0] == b'+' {
                return Err(JsoncNumberError::LeadingPlus);
            }
            cursor += 1;
        }
        let integer_start = cursor;
        if bytes.get(cursor) == Some(&b'0') {
            cursor += 1;
            if bytes.get(cursor).copied().is_some_and(is_digit) {
                return Err(JsoncNumberError::LeadingZero);
            }
        } else {
            // What:     `copied().unwrap_or(0)` supplies a non-digit sentinel when the token ended.
            // Why:      An absent byte must take the same rejection path as a non-digit byte.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const first = bytes[cursor] ?? 0;
            // ```
            let first = bytes.get(cursor).copied().unwrap_or(0);
            if !is_digit(first) {
                return Err(JsoncNumberError::MissingIntegerDigit);
            }
            cursor += 1;
            while bytes.get(cursor).copied().is_some_and(is_digit) {
                cursor += 1;
            }
        }
        let integer_end = cursor;
        let mut fraction_start = cursor;
        let mut fraction_end = cursor;
        if bytes.get(cursor) == Some(&b'.') {
            cursor += 1;
            fraction_start = cursor;
            while bytes.get(cursor).copied().is_some_and(is_digit) {
                cursor += 1;
            }
            if cursor == fraction_start {
                return Err(JsoncNumberError::MissingFractionDigit);
            }
            fraction_end = cursor;
        }
        let mut exponent_negative = false;
        let mut exponent_start = cursor;
        let mut exponent_end = cursor;
        let exponent_marker = bytes.get(cursor) == Some(&b'e') || bytes.get(cursor) == Some(&b'E');
        if exponent_marker {
            cursor += 1;
            exponent_negative = bytes.get(cursor) == Some(&b'-');
            if exponent_negative || bytes.get(cursor) == Some(&b'+') {
                cursor += 1;
            }
            exponent_start = cursor;
            while bytes.get(cursor).copied().is_some_and(is_digit) {
                cursor += 1;
            }
            if cursor == exponent_start {
                return Err(JsoncNumberError::MissingExponentDigit);
            }
            exponent_end = cursor;
        }
        if cursor != bytes.len() {
            return Err(JsoncNumberError::UnexpectedSuffix);
        }
        let parts = NumberParts {
            negative,
            integer: integer_start..integer_end,
            fraction: fraction_start..fraction_end,
            exponent_negative,
            exponent: exponent_start..exponent_end,
        };
        return Ok(normalize(raw, &parts));
    }

    /// What:
    ///      Report whether this identity denotes zero.
    /// Why:
    ///       Every zero spelling shares one canonical form,
    ///  and callers that normalize output
    ///           need to ask without re-deriving it from the digits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isZero(): boolean { return this.digits === '0'; }
    /// ```
    pub fn is_zero(&self) -> bool {
        return self.digits == "0";
    }

    /// What:
    ///      Report whether this identity is negative.
    /// Why:
    ///       Zero is never negative in this model,
    ///  so the sign is meaningful only for non-zero
    ///           values.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isNegative(): boolean { return this.negative; }
    /// ```
    pub fn is_negative(&self) -> bool {
        return self.negative;
    }

    /// What:
    ///      Read the significant digits without zero padding.
    /// Why:
    ///       Diagnostic output and differential tests compare values without reaching into
    ///           private fields.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get digits(): string { return this.#digits; }
    /// ```
    pub fn digits(&self) -> &str {
        return self.digits.as_str();
    }

    /// What:
    ///      Read the signed decimal exponent as text.
    /// Why:
    ///       The exponent can exceed every fixed-width integer,
    ///  so the accessor returns the same
    ///           representation the identity stores.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get exponent(): string { return this.#exponent; }
    /// ```
    pub fn exponent(&self) -> &str {
        return self.exponent.as_str();
    }
}
