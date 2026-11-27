//! What:
//!  Which spelling of a value Git types at startup Git reads silently:
//!  a boolean,
//!  a number with its unit letters,
//!  a color,
//!  a ref name or a word from a list.
//! Why:
//!  Git dies or warns on a value that does not fit its key's type,
//!  and the proof beside this file (`repo_read_config_typed.rs`) accepts only the spellings in here,
//!  leaving every other spelling to Git,
//!  which then words its own failure.
//!  Each check is a plain reading of the text,
//!  because a spelling the checks do not name is a spelling the proof refuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const accepted: boolean = isAccepted(kindOf(key), value);
//! ```

use crate::repo_read_config_value_kinds::{
    ATTRIBUTE_WORDS,
    BRIGHT_COLOR_WORDS,
    FALSE_WORDS,
    Kind,
    MAXIMUM_ABBREVIATION,
    MINIMUM_ABBREVIATION,
    PLAIN_COLOR_WORDS,
    TRUE_WORDS,
};

/// The most digits of a number the proof accepts,
/// so no accepted spelling can overflow the integer Git parses into.
const DIGIT_LIMIT: usize = 9;

/// The suffixes Git multiplies a number by,
/// with their factors.
const UNIT_FACTORS: &[(u8, i64)] = &[(b'k', 1 << 10), (b'm', 1 << 20), (b'g', 1 << 30)];

/// The decimal number a spelling stands for,
/// or `None` for a spelling the proof does not read as one.
///  A sign,
/// 1 to 9 digits with no leading zero (Git reads a leading zero as octal),
/// and one optional unit suffix.
pub(super) fn number_of(text: &str) -> Option<i64> {
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
    if digits.is_empty()
        || digits.len() > DIGIT_LIMIT
        || !digits.iter().all(u8::is_ascii_digit)
        || (digits.len() > 1 && digits[0] == b'0')
    {
        return None;
    }
    let magnitude: i64 = std::str::from_utf8(digits)
        .ok()?
        .parse::<i64>()
        .ok()?
        .checked_mul(factor)?;
    return Some(if negative { -magnitude } else { magnitude });
}

/// Whether `text` is a spelling of a boolean Git reads:
/// a word,
/// an empty value,
/// or a number.
fn is_boolean_text(text: &str) -> bool {
    let lowered: String = text.to_ascii_lowercase();
    return text.is_empty()
        || TRUE_WORDS.contains(&lowered.as_str())
        || FALSE_WORDS.contains(&lowered.as_str())
        || fits_in_i32(text);
}

/// The factor of the unit letter `last` (`k`,
/// `m`,
/// `g`,
/// in either case),
/// or nothing for any other byte.
fn unit_factor(last: u8) -> Option<i64> {
    for (unit, factor) in UNIT_FACTORS.iter() {
        if *unit == last.to_ascii_lowercase() {
            return Some(*factor);
        }
    }
    return None;
}

/// Whether `text` reads as a whole number that fits in 32 bits,
/// with the unit letters Git allows.
fn fits_in_i32(text: &str) -> bool {
    return match number_of(text) {
        Some(number) => i32::try_from(number).is_ok(),
        None => false,
    };
}

/// Whether `text` reads as a whole number from `low` to `high`,
/// both included.
fn number_within(
    text: &str,
    low: i64,
    high: i64,
) -> bool {
    return match number_of(text) {
        Some(number) => (low..=high).contains(&number),
        None => false,
    };
}

/// Whether `text` reads as a number a kind with these bounds takes.
/// Why the minus is refused:
///  a kind whose lower bound is not negative is an unsigned number to Git,
/// and Git refuses every spelling of an unsigned number that carries a minus sign,
/// `-0` included,
/// because the number reader there stops at the sign before it reads any digit
///  (`git_parse_unsigned` in `parse.c` of the Git under test).
/// The number reader here negates what it parsed,
/// and minus zero is zero,
/// so without this the proof would take a spelling Git refuses.
fn number_within_a_kind(
    text: &str,
    low: i64,
    high: i64,
) -> bool {
    if low >= 0 && text.contains('-') {
        return false;
    }
    return number_within(text, low, high);
}

/// Whether `byte` is an ASCII hexadecimal digit.
fn is_hex_byte(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// Whether `byte` may appear in a component of a ref name this check takes:
/// a letter,
/// a digit,
/// `_`,
/// `-` or `.`.
fn is_ref_component_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.');
}

/// Whether `text` is one of the words Git reads as false,
/// or an empty value,
/// without reading it as a number.
fn is_false_word_or_empty(text: &str) -> bool {
    return text.is_empty() || FALSE_WORDS.contains(&text.to_ascii_lowercase().as_str());
}

/// Whether `text` is a false word,
/// an empty value,
/// or a number that stands for false.
fn is_false_text(text: &str) -> bool {
    return is_false_word_or_empty(text) || number_of(text) == Some(0);
}

/// Whether `lowered` is an attribute word,
/// with or without a `no` or `no-` prefix.
fn is_attribute_word(lowered: &str) -> bool {
    let bare: &str = match lowered.strip_prefix("no-") {
        Some(rest) => rest,
        None => lowered.strip_prefix("no").unwrap_or(lowered),
    };
    return ATTRIBUTE_WORDS.contains(&lowered) || ATTRIBUTE_WORDS.contains(&bare);
}

/// Whether one color word,
/// attribute word,
/// number or `#rrggbb` spelling is one Git reads.
fn is_color_word(word: &str) -> bool {
    let lowered_text: String = word.to_ascii_lowercase();
    let lowered: &str = lowered_text.as_str();
    if PLAIN_COLOR_WORDS.contains(&lowered)
        || BRIGHT_COLOR_WORDS.contains(&lowered)
        || is_attribute_word(lowered)
    {
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

/// Whether `text` is at most two colors and any number of attributes,
/// each a word Git reads.
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

/// Whether `name` is a ref name or pattern of plain components under `refs/`,
/// with at most one wildcard component.
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
    return if wildcards > 1 {
        None
    } else {
        Some(wildcards == 1)
    };
}

/// Whether `text` is `[+]refs/...:refs/...` with plain names and a wildcard on both sides or on neither.
fn is_plain_refspec(text: &str) -> bool {
    let Some((source, destination)) = text.strip_prefix('+').unwrap_or(text).split_once(':') else {
        return false;
    };
    return match (
        is_plain_ref_pattern(source),
        is_plain_ref_pattern(destination),
    ) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    };
}

/// Whether `text` is a comma-separated list of the listed words.
fn is_word_list(
    text: &str,
    words: &[&str],
    empty: bool,
    negatable: bool,
) -> bool {
    if text.is_empty() {
        return empty;
    }
    for item in text.split(',') {
        let word: &str = if negatable {
            item.strip_prefix('-').unwrap_or(item)
        } else {
            item
        };
        if !words.contains(&word) {
            return false;
        }
    }
    return true;
}

/// Whether `value` is a spelling of `kind` that Git accepts silently.
pub(super) fn is_accepted(
    kind: Kind,
    value: Option<&[u8]>,
) -> bool {
    let Some(bytes) = value else {
        // A bare name is a true boolean and nothing else.
        return matches!(
            kind,
            Kind::Boolean | Kind::BooleanOr(_) | Kind::IntegerOrBoolean
        );
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    return match kind {
        Kind::Boolean => is_boolean_text(text),
        Kind::BooleanOr(words) => is_boolean_text(text) || words.contains(&text),
        Kind::IntegerOrBoolean => is_boolean_text(text) || fits_in_i32(text),
        Kind::Integer {
            low,
            high,
        } => number_within_a_kind(text, low, high),
        Kind::Text => true,
        Kind::Pathname => !text.starts_with('~') || text.starts_with("~/"),
        Kind::OneOf {
            words,
            empty,
        } => (text.is_empty() && empty) || words.contains(&text),
        Kind::Words {
            words,
            empty,
            negatable,
        } => is_word_list(text, words, empty, negatable),
        Kind::Color => is_color_text(text),
        Kind::Abbreviation => {
            text == "auto"
                || is_false_word_or_empty(text)
                || number_within(text, MINIMUM_ABBREVIATION, MAXIMUM_ABBREVIATION)
        }
        Kind::CommentText => !text.is_empty() && !text.contains('\n'),
        Kind::TraceTarget => text.is_empty() || text == "0" || text.eq_ignore_ascii_case("false"),
        Kind::OffOrBuiltinDaemon => {
            is_false_text(text) || TRUE_WORDS.contains(&text.to_ascii_lowercase().as_str())
        }
        Kind::Refspec => is_plain_refspec(text),
        Kind::Refused => false,
    };
}
