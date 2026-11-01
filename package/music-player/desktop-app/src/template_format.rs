//! Numbers and duration text as JavaScript writes and reads them.
//!
//! The template functions `tf` and `tc` depend on three JavaScript rules:
//! `String(value)` prints a number,
//!  `Number(text)` reads one,
//!  and `Math.floor`
//! rounds.
//!  Rust's float printing never uses an exponent and prints `-0` as `-0`,
//! so the printing rule is rebuilt from the shortest round-trip digits that
//! `format!("{:e}")` produces.
//!
//! Known divergence:
//!  hexadecimal,
//!  octal,
//!  and binary literals longer than 128 bits
//! are accumulated in `f64`,
//!  so their last bit can differ from JavaScript's exact
//! rounding.
//!  Shorter literals convert exactly.

/// What:
///  `const MAX_PLAIN_POINT: i32 = 21;` is the largest decimal point position that prints as plain digits.
/// Why:
///  JavaScript switches to an exponent form beyond this position,
///  so the threshold is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_PLAIN_POINT = 21;
/// ```
const MAX_PLAIN_POINT: i32 = 21;

/// What:
///  `const MIN_PLAIN_POINT: i32 = -6;` is the point position below which an exponent is used.
/// Why:
///  JavaScript prints very small numbers with an exponent once the point moves this far left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MIN_PLAIN_POINT = -6;
/// ```
const MIN_PLAIN_POINT: i32 = -6;

/// What:
///  `pub fn number_text(value: f64) -> String` prints a number the way JavaScript's `String()` does.
/// Why:
///  Track lengths and time parts must print with JavaScript's plain or exponent layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = String(value);
/// ```
pub fn number_text(value: f64) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return String::from(if value > 0.0 { "Infinity" } else { "-Infinity" });
    }
    if value == 0.0 {
        return String::from("0");
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let scientific = format!("{:e}", value.abs());
    let Some((mantissa, exponent_text)) = scientific.split_once('e') else {
        unreachable!("a finite nonzero f64 always has an exponent in its scientific form");
    };
    let Ok(exponent) = exponent_text.parse::<i32>() else {
        unreachable!("the exponent of a scientific f64 is always an integer");
    };
    let joined: String = mantissa.chars().filter(|symbol| return *symbol != '.').collect();
    let digits = joined.trim_end_matches('0');
    return format!("{sign}{}", layout_digits(digits, exponent + 1));
}

/// What:
///  `fn layout_digits(digits: &str, point_position: i32) -> String` places the decimal point.
/// Why:
///  The value equals `0.digits` times ten to the `point_position`,
///  and the thresholds decide plain or exponent text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function layout(digits: string, n: number): string { ... }
/// ```
fn layout_digits(digits: &str, point_position: i32) -> String {
    let count = i32::try_from(digits.len()).unwrap_or(i32::MAX);
    if count <= point_position && (..=MAX_PLAIN_POINT).contains(&point_position) {
        return format!("{digits}{}", "0".repeat(usize::try_from(point_position - count).unwrap_or(0)));
    }
    if (1..=MAX_PLAIN_POINT).contains(&point_position) {
        let split = usize::try_from(point_position).unwrap_or(0);
        return format!("{}.{}", &digits[..split], &digits[split..]);
    }
    if (MIN_PLAIN_POINT + 1..=0).contains(&point_position) {
        return format!("0.{}{digits}", "0".repeat(usize::try_from(-point_position).unwrap_or(0)));
    }
    let exponent = point_position - 1;
    let mantissa = if count == 1 { digits.to_string() } else { format!("{}.{}", &digits[..1], &digits[1..]) };
    let exponent_sign = if exponent < 0 { "-" } else { "+" };
    return format!("{mantissa}e{exponent_sign}{}", exponent.abs());
}

/// What:
///  `pub fn as_number(text: &str) -> Option<f64>` reads a finite number the way JavaScript's `Number(text.trim())` does.
/// Why:
///  `tf` and the `tc` cut length need seconds and lengths,
///  and empty,
///  NaN,
///  and infinite text are not numbers for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function asNumber(text: string): number | undefined {
///   const n = Number(text.trim());
///   return text.trim() === "" || !Number.isFinite(n) ? undefined : n;
/// }
/// ```
pub fn as_number(text: &str) -> Option<f64> {
    let trimmed = trim_js_whitespace(text);
    if trimmed.is_empty() {
        return None;
    }
    let number = number_from_text(trimmed)?;
    if number.is_finite() {
        return Some(number);
    }
    return None;
}

/// What:
///  `fn trim_js_whitespace(text: &str) -> &str` removes the characters JavaScript's `trim` removes.
/// Why:
///  Rust's `trim` uses a different white-space set (it keeps U+FEFF and removes U+0085),
///  so the set is listed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trimmed = text.trim();
/// ```
fn trim_js_whitespace(text: &str) -> &str {
    return text.trim_matches(is_js_whitespace);
}

/// What:
///  `fn is_js_whitespace(symbol: char) -> bool` tells whether JavaScript's `trim` removes a character.
/// Why:
///  The list is the ASCII white space,
///  the Unicode spaces,
///  the line separators,
///  and the byte order mark.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSpace = (c: string) => /\s/.test(c);
/// ```
fn is_js_whitespace(symbol: char) -> bool {
    return matches!(
        u32::from(symbol),
        0x09..=0x0D | 0x20 | 0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF
    );
}

/// What:
///  `fn number_from_text(text: &str) -> Option<f64>` reads a number in hexadecimal,
///  octal,
///  binary,
///  or decimal spelling.
/// Why:
///  A duration may be written in any of JavaScript's number spellings,
///  and nothing else reads as a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const n = Number(text);
/// ```
fn number_from_text(text: &str) -> Option<f64> {
    if let Some((radix, digits)) = radix_prefix(text) {
        return radix_value(digits, radix);
    }
    return decimal_value(text);
}

/// What:
///  `fn radix_prefix(text: &str) -> Option<(u32, &str)>` splits a `0x`,
///  `0o`,
///  or `0b` prefix from its digits.
/// Why:
///  JavaScript reads these spellings only without a sign,
///  and the prefix letter may be either case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const match = /^0([xXoObB])(.*)$/.exec(text);
/// ```
fn radix_prefix(text: &str) -> Option<(u32, &str)> {
    let bytes = text.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'0' {
        return None;
    }
    let radix = match bytes[1] {
        b'x' | b'X' => 16,
        b'o' | b'O' => 8,
        b'b' | b'B' => 2,
        _ => return None,
    };
    return Some((radix, &text[2..]));
}

/// What:
///  `fn radix_value(digits: &str, radix: u32) -> Option<f64>` reads digits in one radix.
/// Why:
///  The exact conversion covers every literal up to 128 bits,
///  and longer ones fall back to `f64` accumulation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const n = parseInt(digits, radix);
/// ```
fn radix_value(digits: &str, radix: u32) -> Option<f64> {
    if digits.is_empty() || !digits.chars().all(|symbol| return symbol.is_digit(radix)) {
        return None;
    }
    if let Ok(exact) = u128::from_str_radix(digits, radix) {
        return Some(exact as f64);
    }
    return Some(digits.chars().fold(0.0, |total, symbol| {
        return total * f64::from(radix) + f64::from(symbol.to_digit(radix).unwrap_or(0));
    }));
}

/// What:
///  `fn decimal_value(text: &str) -> Option<f64>` reads a signed decimal literal,
///  including `Infinity`.
/// Why:
///  The grammar is checked by hand first,
///  because Rust's parser also accepts `inf`,
///  `nan`,
///  and `infinity` in any case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decimal = /^[+-]?(Infinity|(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?)$/;
/// ```
fn decimal_value(text: &str) -> Option<f64> {
    if !is_decimal_literal(text) {
        return None;
    }
    if text.ends_with("Infinity") {
        return Some(if text.starts_with('-') { f64::NEG_INFINITY } else { f64::INFINITY });
    }
    return text.parse::<f64>().ok();
}

/// What:
///  `fn is_decimal_literal(text: &str) -> bool` tells whether text matches JavaScript's signed decimal grammar.
/// Why:
///  Only this grammar reads as a number,
///  so `1_0`,
///  `0x`,
///  and `1e` are rejected before parsing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decimal = /^[+-]?(Infinity|(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?)$/;
/// ```
fn is_decimal_literal(text: &str) -> bool {
    let unsigned = text.strip_prefix(is_sign).unwrap_or(text);
    if unsigned == "Infinity" {
        return true;
    }
    let (mantissa, exponent) = match unsigned.find(is_exponent_marker) {
        Some(position) => (&unsigned[..position], Some(&unsigned[position + 1..])),
        None => (unsigned, None),
    };
    if !is_decimal_mantissa(mantissa) {
        return false;
    }
    return match exponent {
        Some(digits) => is_signed_digits(digits),
        None => true,
    };
}

/// What:
///  `fn is_decimal_mantissa(mantissa: &str) -> bool` checks the digits before any exponent.
/// Why:
///  The grammar needs at least one digit,
///  with at most one point between the whole and fraction digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mantissa = /^(\d+\.?\d*|\.\d+)$/;
/// ```
fn is_decimal_mantissa(mantissa: &str) -> bool {
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if whole.is_empty() && fraction.is_empty() {
        return false;
    }
    return all_ascii_digits(whole) && all_ascii_digits(fraction);
}

/// What:
///  `fn is_signed_digits(text: &str) -> bool` checks an exponent's optional sign and its digits.
/// Why:
///  An exponent with no digit after its sign is not a number in JavaScript.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exponent = /^[+-]?\d+$/;
/// ```
fn is_signed_digits(text: &str) -> bool {
    let digits = text.strip_prefix(is_sign).unwrap_or(text);
    return !digits.is_empty() && all_ascii_digits(digits);
}

/// What:
///  `fn all_ascii_digits(part: &str) -> bool` tells whether every byte is an ASCII digit;
///  an empty part passes.
/// Why:
///  The grammar checks the whole and fraction parts separately,
///  and either may be empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allDigits = (part: string) => /^\d*$/.test(part);
/// ```
fn all_ascii_digits(part: &str) -> bool {
    return part.bytes().all(|byte| return byte.is_ascii_digit());
}

/// What:
///  `fn is_sign(symbol: char) -> bool` tells whether a character is `+` or `-`.
/// Why:
///  Signs are optional in front of decimals,
///  exponents,
///  and `Infinity`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSign = (c: string) => c === "+" || c === "-";
/// ```
fn is_sign(symbol: char) -> bool {
    return symbol == '+' || symbol == '-';
}

/// What:
///  `fn is_exponent_marker(symbol: char) -> bool` tells whether a character starts a decimal exponent.
/// Why:
///  JavaScript accepts both `e` and `E` before the exponent digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isMarker = (c: string) => c === "e" || c === "E";
/// ```
fn is_exponent_marker(symbol: char) -> bool {
    return symbol == 'e' || symbol == 'E';
}

/// What:
///  `pub fn unknown_format_letter(format: &str) -> Option<char>` finds the first letter of a format that is neither a time letter nor
///  inside apostrophes.
/// Why:
///  `h`, `m` and `s` are the only time letters, and a stray letter such as the `i` of `m min` used to print as itself, so a
///  typo looked like a working format. Letters between apostrophes are kept text, and every other character is a separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unknownFormatLetter = (format: string) => outsideApostrophes(format).find((c) => /\p{L}/u.test(c) && !"hms".includes(c));
/// ```
pub fn unknown_format_letter(format: &str) -> Option<char> {
    let mut kept = false;
    for letter in format.chars() {
        if letter == '\'' {
            kept = !kept;
            continue;
        }
        if !kept && letter.is_alphabetic() && !matches!(letter, 'h' | 'm' | 's') {
            return Some(letter);
        }
    }
    return None;
}

/// What:
///  `pub fn format_seconds(seconds: f64, format: &str) -> String` fills a format's time letters with hours,
///  minutes,
///  and seconds.
/// Why:
///  Doubled letters pad with a zero,
///  apostrophes keep text as written,
///  and minutes carry the hours only when the format has no hour letter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatSeconds({ seconds, format }: { seconds: number; format: string }): string { ... }
/// ```
pub fn format_seconds(seconds: f64, format: &str) -> String {
    let whole = seconds.floor().max(0.0);
    let hours = (whole / 3600.0).floor();
    // Minutes carry the hours when the format has no hour letter, so 4000 seconds reads 66:40 under m:ss.
    let minutes = if format.contains('h') { ((whole % 3600.0) / 60.0).floor() } else { (whole / 60.0).floor() };
    let secs = whole % 60.0;
    let clock = [('h', hours), ('m', minutes), ('s', secs)];
    let letters: Vec<char> = format.chars().collect();
    let mut output = String::new();
    let mut kept = false;
    let mut index = 0;
    while index < letters.len() {
        let letter = letters[index];
        if letter == '\'' {
            kept = !kept;
            index += 1;
            continue;
        }
        let value = if kept {
            None
        } else {
            clock.iter().find(|entry| return entry.0 == letter).map(|entry| return entry.1)
        };
        let Some(number) = value else {
            output.push(letter);
            index += 1;
            continue;
        };
        let doubled = letters.get(index + 1) == Some(&letter);
        let number_output = number_text(number);
        if doubled {
            output.push_str(&pad_to_two(&number_output));
            index += 2;
        } else {
            output.push_str(&number_output);
            index += 1;
        }
    }
    return output;
}

/// What:
///  `fn pad_to_two(text: &str) -> String` pads text with leading zeros to two characters.
/// Why:
///  A doubled time letter writes its value with at least two digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const padded = text.padStart(2, "0");
/// ```
fn pad_to_two(text: &str) -> String {
    let width = text.chars().count();
    return format!("{}{text}", "0".repeat(2usize.saturating_sub(width)));
}

/// What:
///  `pub fn keep_code_points(text: &str, count: f64) -> String` keeps the first whole number of code points below `count`.
/// Why:
///  The reference counts characters as code points,
///  so a surrogate pair is one character,
///  and a count past the end keeps all of the text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = [...text].slice(0, Math.floor(count)).join("");
/// ```
pub fn keep_code_points(text: &str, count: f64) -> String {
    let total = text.chars().count();
    let whole = count.floor();
    if whole >= total as f64 {
        return text.to_string();
    }
    return text.chars().take(whole as usize).collect();
}

/// What:
///  `pub fn capitalize_first_unit(text: &str) -> String` uppercases the first UTF-16 unit of the text,
///  as `cap` does.
/// Why:
///  JavaScript's `slice(0, 1)` takes one UTF-16 unit,
///  so a first character outside the Basic Multilingual Plane is a lone surrogate there and stays unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capped = text.slice(0, 1).toUpperCase() + text.slice(1);
/// ```
pub fn capitalize_first_unit(text: &str) -> String {
    let Some(first) = text.chars().next() else {
        return String::new();
    };
    if first.len_utf16() > 1 {
        return text.to_string();
    }
    let rest = &text[first.len_utf8()..];
    return format!("{}{rest}", first.to_uppercase());
}

/// What:
///  `pub fn utf16_length(text: &str) -> usize` counts the text's UTF-16 code units.
/// Why:
///  Every position in the template language is a UTF-16 offset,
///  as in JavaScript.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const length = text.length;
/// ```
pub fn utf16_length(text: &str) -> usize {
    return text.encode_utf16().count();
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_format_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_format_tests.rs"]
mod tests;
