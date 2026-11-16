//! What:
//!  Controls for the words a span of time is printed in.
//! Why:
//!  The listing is how a person compares an age with a limit,
//!  so a limit of a second or of a month must read as itself,
//!  and a count of one must not read as a plural.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(spanWords(Duration.ofSeconds(1))).toBe('1 second');
//! ```

use super::{age_words, counted, span_words};
use std::time::Duration;

/// An age reads as the days and hours a person compares with the limit.
#[test]
fn an_age_reads_in_days_and_hours() {
    assert_eq!(age_words(0), "0 minutes");
    assert_eq!(age_words(90 * 1000), "1 minute");
    assert_eq!(age_words(2 * 60 * 60 * 1000), "2 hours 0 minutes");
    assert_eq!(age_words(6 * 24 * 60 * 60 * 1000 + 5 * 60 * 60 * 1000), "6 days 5 hours");
}

/// A limit reads in the largest unit that divides it,
/// so the figure a person reads is the one the run was given.
#[test]
fn a_limit_reads_in_the_unit_it_was_given() {
    assert_eq!(span_words(Duration::from_secs(1)), "1 second");
    assert_eq!(span_words(Duration::from_secs(1_800)), "30 minutes");
    assert_eq!(span_words(Duration::from_secs(60 * 60)), "1 hour");
    assert_eq!(span_words(Duration::from_secs(30 * 24 * 60 * 60)), "30 days");
    assert_eq!(span_words(Duration::from_secs(2 * 24 * 60 * 60 + 60 * 60)), "2 days");
}

/// A count of one takes the singular noun,
/// and every other count the plural.
#[test]
fn a_count_of_one_takes_the_singular() {
    assert_eq!(counted(1, "day"), "day");
    assert_eq!(counted(0, "day"), "days");
    assert_eq!(counted(2, "hour"), "hours");
}
