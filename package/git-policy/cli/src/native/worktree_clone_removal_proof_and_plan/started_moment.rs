//! What:
//!  The moment a removal started,
//!  as the destruction plan's `started=` field writes it:
//!  an ISO 8601 second with the machine's own UTC offset.
//! Why:
//!  The plan keeps the moment for `doctor` and for a person and decides nothing by it,
//!  because the design sets no expiry
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Field by field":
//! `started` "is never a proof").
//!  Writing the offset rather than `Z` is what the design's example holds,
//!  so a person reading two records from two machines sees the same local hour the
//!  wrapper printed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = Temporal.Now.zonedDateTimeISO().toPlainDateTime().toString() + offset;
//! ```

/// The bytes of one calendar date,
/// `YYYY-MM-DD`,
/// which takes its two dashes.
const DATE_BYTES: usize = 10;

/// The bytes of one time of day,
/// `HH:MM:SS`,
/// which takes its two colons.
const TIME_BYTES: usize = 8;

/// The bytes of the `T` between the date and the time.
const DATE_TIME_MARKER: usize = 1;

/// The bytes of an offset written as `+hh:mm`,
/// which is a sign and the two separators' characters.
const OFFSET_BYTES: usize = 6;

/// The bytes of one whole stamp with an offset.
pub const STAMP_WITH_OFFSET_BYTES: usize =
    DATE_BYTES + DATE_TIME_MARKER + TIME_BYTES + OFFSET_BYTES;

/// The format this field writes.
/// `%:z` is the offset with its colon,
/// which is the ISO 8601 form.
const STAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%:z";

/// What:
///  This moment,
///  as the plan's `started` field writes it.
/// Why:
///  The plan is about one person's removal,
///  and the machine's own offset is what that person sees on a clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = nowWithOffset();
/// ```
pub fn now_with_offset() -> String {
    return jiff::Zoned::now().strftime(STAMP_FORMAT).to_string();
}

/// What:
///  Whether text is one ISO 8601 second with an offset,
///  in the one spelling this field writes.
/// Why:
///  A reader refuses a field whose shape it does not know rather than reading a moment out of
///  a string nothing guarantees,
///  and the fixed width is what lets the refusal be a shape test and no calendar arithmetic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isStamp = (text) => /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d[+-]\d\d:\d\d$/.test(text);
/// ```
pub fn is_seconds_with_offset(text: &str) -> bool {
    if text.len() != STAMP_WITH_OFFSET_BYTES {
        return false;
    }
    let bytes: &[u8] = text.as_bytes();
    for (place, byte) in bytes.iter().enumerate() {
        let wanted: bool = match place {
            4 | 7 => *byte == b'-',
            10 => *byte == b'T',
            13 | 16 => *byte == b':',
            19 => *byte == b'+' || *byte == b'-',
            22 => *byte == b':',
            _ => byte.is_ascii_digit(),
        };
        if !wanted {
            return false;
        }
    }
    // The digit fields are ranges and not only digit counts, so a month of 13 or an hour of 24 is
    // refused: `jiff` writes this field and a hand-written stamp is still read by a shape.
    return two_digits(bytes, 5) <= 12
        && two_digits(bytes, 8) >= 1
        && two_digits(bytes, 8) <= 31
        && two_digits(bytes, 11) <= 23
        && two_digits(bytes, 14) <= 59
        && two_digits(bytes, 17) <= 59
        && two_digits(bytes, 20) <= 23
        && two_digits(bytes, 23) <= 59;
}

/// The two digits of a stamp at one place.
fn two_digits(
    bytes: &[u8],
    at: usize,
) -> u32 {
    return u32::from(bytes[at] - b'0') * 10 + u32::from(bytes[at + 1] - b'0');
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "started_moment_tests.rs"]
mod tests;
