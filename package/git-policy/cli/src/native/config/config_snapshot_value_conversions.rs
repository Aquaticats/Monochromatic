//! What:
//!  One configuration value's bytes read the way Git reads them for one type:
//!  its boolean words and the integer reader behind them,
//!  the plain decimal and the plain path a lookup models,
//!  and the text Git prints for each.
//! Why:
//!  A lookup answers only when it can reproduce `git config --type=<type> --get`
//!  exactly,
//!  so the conversions live here with the shapes each one declines;
//!  a shape that is declined answers nothing and the caller starts Git,
//!  which is never a failure
//!  (`crate::config_snapshot`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hooksPath = plainPath(value); const enabled = canonicalBool(value); // undefined means: ask Git
//! ```

/// Largest decimal integer text a lookup models;
/// longer digit strings fall back to Git.
const DECIMAL_DIGITS_LIMIT: usize = 9;
/// What:
///  Whether the bytes of one value are a plain decimal integer the lookups model,
/// and its numeric value.
/// Why:
///   Git parses integers with `strtoimax` base 0 and unit suffixes,
/// so only `0` and digit strings that start with
///  `1` to `9` are decided here;
/// leading zeros (octal),
/// signs,
/// `0x`,
/// spaces and suffixes fall back to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^(0|[1-9][0-9]{0,8})$/.test(text) ? Number(text) : undefined;
/// ```
fn plain_decimal(text: &str) -> Option<u32> {
    if text == "0" {
        return Some(0);
    }
    let first: u8 = *text.as_bytes().first()?;
    if !(b'1'..=b'9').contains(&first) || text.len() > DECIMAL_DIGITS_LIMIT {
        return None;
    }
    if !text.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    return text.parse::<u32>().ok();
}

/// What:
///  Git's boolean word list,
/// case-insensitively:
/// `Some(true)` for true,
/// yes and on,
/// `Some(false)` for false,
/// no
///  and off,
/// `None` for any other text.
/// Why:
///   `git_parse_maybe_bool_text` compares with `strcasecmp`,
/// which is ASCII case folding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ true: true, yes: true, on: true, false: false, no: false, off: false })[text.toLowerCase()];
/// ```
fn boolean_word(text: &str) -> Option<bool> {
    for word in ["true", "yes", "on"] {
        if text.eq_ignore_ascii_case(word) {
            return Some(true);
        }
    }
    for word in ["false", "no", "off"] {
        if text.eq_ignore_ascii_case(word) {
            return Some(false);
        }
    }
    return None;
}

/// What:
///  One byte as a digit of `radix`,
/// or nothing when it is not one.
/// Why:
///   `strtoimax` stops at the first byte that is not a digit of the radix it chose,
/// and the loop that walks the
///  digits needs that answer as a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseInt(String.fromCharCode(byte), radix); // NaN when it is not a digit
/// ```
fn digit_of(
    byte: u8,
    radix: u32,
) -> Option<u32> {
    return (byte as char).to_digit(radix);
}

/// What:
///  One integer as Git's own reader spells it:
/// `strtoimax` with base 0,
/// then an optional `k`,
/// `m` or `g` unit.
/// Why:
///   `git_parse_signed` reads an integer this way and multiplies it by the unit,
/// so `00` is octal zero,
///  `0x0` is zero,
/// `-0` and `+0` are zero,
/// `0k` is zero,
/// `010` is eight,
/// and `1k` is 1024.
///  A text that reader refuses (a trailing space,
/// a digit the radix does not hold,
/// a value past the range of a
///  signed 64-bit integer) is `None`,
/// which means Git reads it as something else,
/// such as a hook path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitInteger('0x10'); // 16
/// ```
fn git_integer(text: &str) -> Option<i64> {
    let bytes: &[u8] = text.as_bytes();
    let mut at: usize = 0;
    // `strtoimax` skips leading whitespace and takes one sign.
    while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        at += 1;
    }
    let negative: bool = match bytes.get(at) {
        Some(b'-') => {
            at += 1;
            true
        }
        Some(b'+') => {
            at += 1;
            false
        }
        _ => false,
    };
    let (radix, first_digit): (u32, usize) =
        match (bytes.get(at), bytes.get(at + 1).map(u8::to_ascii_lowercase)) {
            (Some(b'0'), Some(b'x')) => (16, at + 2),
            (Some(b'0'), _) => (8, at),
            _ => (10, at),
        };
    let mut magnitude: i64 = 0;
    let mut digits: usize = 0;
    let mut cursor: usize = first_digit;
    loop {
        let Some(byte) = bytes.get(cursor) else { break };
        let Some(digit) = digit_of(*byte, radix) else {
            break;
        };
        magnitude = magnitude
            .checked_mul(i64::from(radix))?
            .checked_add(i64::from(digit))?;
        digits += 1;
        cursor += 1;
    }
    if digits == 0 {
        return None;
    }
    let factor: i64 = match bytes.get(cursor..) {
        Some(b"") => 1,
        Some(tail) if tail.eq_ignore_ascii_case(b"k") => 1024,
        Some(tail) if tail.eq_ignore_ascii_case(b"m") => 1024 * 1024,
        Some(tail) if tail.eq_ignore_ascii_case(b"g") => 1024 * 1024 * 1024,
        _ => return None,
    };
    let scaled: i64 = magnitude.checked_mul(factor)?;
    return Some(if negative { -scaled } else { scaled });
}

/// What:
///  `--type=bool` of one value:
/// a valueless key is true,
/// empty text is false,
/// a word from the list is itself,
///  and any other text Git's integer reader accepts is whether the integer is not zero.
/// Why:
///   Git asks its integer reader for everything that is not a boolean word,
/// so the integer spellings
///  (octal,
/// hexadecimal,
/// signs,
/// unit suffixes) are the person's answer too,
/// and a value the reader refuses is
///  the only one left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// canonicalBool(value); // undefined means: ask Git
/// ```
pub fn canonical_bool(value: Option<&[u8]>) -> Option<bool> {
    let Some(bytes) = value else {
        return Some(true);
    };
    let text: &str = std::str::from_utf8(bytes).ok()?;
    if text.is_empty() {
        return Some(false);
    }
    if let Some(word) = boolean_word(text) {
        return Some(word);
    }
    let number: i64 = git_integer(text)?;
    return Some(number != 0);
}

/// What:
///  `--type=bool-or-int` of one value,
/// as the text Git prints.
/// Why:
///   Git prints `true` or `false` for a boolean word,
/// valueless key or empty text,
/// and the decimal number for an
///  integer;
/// the plain decimal subset prints unchanged,
/// and anything else is not modelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// canonicalBoolOrInt(value); // undefined means: ask Git
/// ```
pub fn canonical_bool_or_int(value: Option<&[u8]>) -> Option<String> {
    let Some(bytes) = value else {
        return Some(String::from("true"));
    };
    let text: &str = std::str::from_utf8(bytes).ok()?;
    if text.is_empty() {
        return Some(String::from("false"));
    }
    if let Some(word) = boolean_word(text) {
        return Some(String::from(if word { "true" } else { "false" }));
    }
    let number: u32 = plain_decimal(text)?;
    return Some(number.to_string());
}

/// What:
///  `--type=path` of one value,
/// when Git returns the text unchanged.
/// Why:
///   Git expands a leading `~`,
/// `~user/` and `%(prefix)/`,
/// treats `:(optional)` specially,
/// and refuses a valueless
///  key;
/// those,
/// an empty value and non-UTF-8 bytes are not modelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plainPath(value); // undefined means: ask Git
/// ```
pub fn plain_path(value: Option<&[u8]>) -> Option<String> {
    let text: &str = std::str::from_utf8(value?).ok()?;
    if text.is_empty() || text.starts_with('~') || text.starts_with("%(") || text.starts_with(":(")
    {
        return None;
    }
    return Some(String::from(text));
}
