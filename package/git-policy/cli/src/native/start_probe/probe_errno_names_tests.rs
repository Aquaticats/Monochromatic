//! What:
//!  Controls of the closed errno and reason vocabularies:
//!  each name maps to its number and back,
//!  no number is named twice,
//!  and only the listed reason words are known.
//! Why:
//!  The record names an errno rather than a number,
//!  and a duplicated value would name the wrong call's answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const [name, value] of ERRNO_NAMES) expect(errnoOf(value)).toBe(name);
//! ```

use super::{
    ERRNO_NAMES,
    ErrnoWord,
    REASON_WORDS,
    errno_name,
    errno_of,
    known_reason_word,
};

/// Each listed name maps to its own number and back.
#[test]
fn every_name_round_trips() {
    for &(name, value) in ERRNO_NAMES {
        assert_eq!(errno_of(value), Some(name));
        assert_eq!(errno_name(name), Some(name));
    }
}

/// No number is listed twice,
/// so a call's answer names one word.
#[test]
fn no_number_is_named_twice() {
    for &(name, value) in ERRNO_NAMES {
        let matching: usize = ERRNO_NAMES
            .iter()
            .filter(|(_, other)| return *other == value)
            .count();
        assert_eq!(matching, 1, "{name} shares its number");
    }
}

/// A number this build does not name stays unnamed.
#[test]
fn an_unnamed_number_stays_unnamed() {
    assert_eq!(errno_of(4242), None);
    assert_eq!(errno_name("ENOTLISTED"), None);
}

/// A named number becomes its name,
/// an unnamed number stays itself,
/// and both parse back.
#[test]
fn the_errno_field_names_or_keeps_the_number() {
    assert_eq!(
        ErrnoWord::of_number(Some(libc::EBUSY)),
        ErrnoWord::Named("EBUSY")
    );
    assert_eq!(ErrnoWord::of_number(Some(4242)), ErrnoWord::Number(4242));
    assert_eq!(ErrnoWord::of_number(None), ErrnoWord::None);
    assert_eq!(ErrnoWord::parse("EBUSY"), Some(ErrnoWord::Named("EBUSY")));
    assert_eq!(ErrnoWord::parse("4242"), Some(ErrnoWord::Number(4242)));
    assert_eq!(ErrnoWord::parse("none"), Some(ErrnoWord::None));
    assert_eq!(ErrnoWord::parse("ENOTLISTED"), None);
}

/// Only the listed reason words are known,
/// and an unknown one is no fallback.
#[test]
fn only_the_listed_reason_words_are_known() {
    for &word in REASON_WORDS {
        assert_eq!(known_reason_word(word), Some(word));
    }
    assert_eq!(known_reason_word("no-such-reason"), None);
}
