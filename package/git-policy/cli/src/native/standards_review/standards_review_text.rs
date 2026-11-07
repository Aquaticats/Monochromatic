//! What:
//!  The few text behaviors of the standards-review log that JavaScript defines and Rust does not share:
//!  UTF-16 lengths and prefixes,
//! `Number` to text,
//! `toFixed(1)` of a duration,
//! and Git's `%s` subject.
//! Why:
//!  The root `mise.toml` task `shadow-review` measures and cuts text in UTF-16 code units and prints numbers the way
//!  ECMAScript does,
//! and its log is compared entry by entry with this port's,
//! so the same text must give the same
//!  counts and the same bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! text.length; text.slice(0, limit); String(number); (milliseconds / 1000).toFixed(1); // git log --format=%s
//! ```

/// What:
///  The length of `text` in UTF-16 code units,
/// which is what JavaScript's `.length` counts.
///  `usize` is the unsigned size every length uses (siblings `u32`,
/// `u64`).
/// Why:
///   The log prints `luna input N chars` with that count,
/// and a Rust `.len()` would count UTF-8 bytes instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.length
/// ```
pub fn utf16_length(text: &str) -> usize {
    return text.chars().map(char::len_utf16).sum();
}

/// What:
///  The first `limit` UTF-16 code units of `text`,
/// as JavaScript's `text.slice(0, limit)` keeps them.
///  A cut that falls inside a surrogate pair drops the first half,
/// because a Rust `String` cannot hold half a pair.
/// Why:
///   The model input and the failure excerpt are cut by code units;
/// a character that a cut would split is the only
///   place where the result differs from the script's,
/// and the script would send a lone surrogate there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.slice(0, limit)
/// ```
pub fn utf16_prefix(text: &str, limit: usize) -> String {
    let mut kept: String = String::new();
    let mut used: usize = 0;
    for character in text.chars() {
        let width: usize = character.len_utf16();
        if used + width > limit {
            break;
        }
        kept.push(character);
        used += width;
    }
    return kept;
}

/// What:
///  `value` as ECMAScript's `String(value)` prints a finite number:
/// shortest digits,
/// no exponent between 1e-6 and
///  1e21,
/// otherwise `1e-7` or `1.5e+21` style.
/// Why:
///   Rust prints `0.0000001` where JavaScript prints `1e-7`,
/// and the log carries the gateway's probabilities as
///   `Number` text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// String(value)
/// ```
pub fn js_number_text(value: f64) -> String {
    if value == 0.0 {
        return String::from("0");
    }
    let sign: &str = if value < 0.0 { "-" } else { "" };
    // Rust's `{:e}` prints the shortest digits that round-trip, as `d.ddde-5`.
    let scientific: String = format!("{:e}", value.abs());
    let (mantissa, exponent_text) = scientific.split_once('e').unwrap_or((scientific.as_str(), "0"));
    let digits: String = mantissa.replace('.', "");
    let exponent: i32 = exponent_text.parse::<i32>().unwrap_or(0);
    let count: i32 = i32::try_from(digits.len()).unwrap_or(i32::MAX);
    // ECMAScript's `n`: the decimal point sits after the first `n` digits.
    let point: i32 = exponent + 1;
    let body: String = if count <= point && point <= 21 {
        format!("{digits}{}", "0".repeat(usize::try_from(point - count).unwrap_or(0)))
    } else if 0 < point && point <= 21 {
        let split: usize = usize::try_from(point).unwrap_or(0);
        format!("{}.{}", &digits[..split], &digits[split..])
    } else if -6 < point && point <= 0 {
        format!("0.{}{digits}", "0".repeat(usize::try_from(-point).unwrap_or(0)))
    } else {
        let power: i32 = point - 1;
        let power_sign: &str = if power < 0 { "-" } else { "+" };
        let lead: &str = &digits[..1];
        let rest: &str = &digits[1..];
        if rest.is_empty() {
            format!("{lead}e{power_sign}{}", power.abs())
        } else {
            format!("{lead}.{rest}e{power_sign}{}", power.abs())
        }
    };
    return format!("{sign}{body}");
}

/// What:
///  `(milliseconds / 1000).toFixed(1)`:
/// seconds with one decimal,
/// a tie rounded up as JavaScript rounds it.
/// Why:
///   The log prints the review's wall-clock cost that way.
/// Only a value that is exactly a quarter second ties
///   (250 ms past a half second),
/// and Rust rounds a tie to even where JavaScript takes the larger neighbor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (milliseconds / 1000).toFixed(1)
/// ```
pub fn seconds_with_tenth(milliseconds: u64) -> String {
    const TIE_PERIOD_MS: u64 = 500;
    const TIE_REMAINDER_MS: u64 = 250;
    const TENTH_MS: u64 = 100;
    if milliseconds % TIE_PERIOD_MS == TIE_REMAINDER_MS {
        let tenths: u64 = milliseconds / TENTH_MS + 1;
        return format!("{}.{}", tenths / 10, tenths % 10);
    }
    let seconds: f64 = milliseconds as f64 / 1000.0;
    return format!("{seconds:.1}");
}

/// What:
///  The subject Git's `%s` prints for a commit message,
/// trimmed as the script trims it:
/// blank lines skipped,
/// the
///  first paragraph's lines joined by one space,
/// each line without trailing whitespace.
/// Why:
///   The landing holds the commit message,
/// so the subject needs no `git log` start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('git', ['log', '-1', '--format=%s', sha]).trim()
/// ```
pub fn commit_subject(message: &str) -> String {
    let mut paragraph: Vec<&str> = Vec::new();
    for line in message.split('\n') {
        let trimmed: &str = line.trim_end();
        if trimmed.trim_start().is_empty() {
            if paragraph.is_empty() {
                continue;
            }
            break;
        }
        paragraph.push(trimmed);
    }
    return paragraph.join(" ").trim().to_string();
}

/// What:
///  `text` on one line:
/// every control character and Unicode line or paragraph separator becomes a visible escape
///  (`\n`,
/// `\r`,
/// `\t`,
/// otherwise `\u{hex}`),
/// and a backslash stays itself.
/// Why:
///   The log is line-structured and a reader parses it by lines.
/// A response body,
/// a transport error or a model answer
///   that carried a line break could otherwise start a fake entry.
/// The script wrote such text unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/[\u0000-\u001f\u007f-\u009f  ]/g, (c) => escapeVisible(c))
/// ```
pub fn on_one_line(text: &str) -> String {
    let mut line: String = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\n' => line.push_str("\\n"),
            '\r' => line.push_str("\\r"),
            '\t' => line.push_str("\\t"),
            other if other.is_control() || other == '\u{2028}' || other == '\u{2029}' => {
                line.push_str(&format!("\\u{{{:x}}}", u32::from(other)));
            }
            other => line.push(other),
        }
    }
    return line;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_text_tests.rs"]
mod tests;
