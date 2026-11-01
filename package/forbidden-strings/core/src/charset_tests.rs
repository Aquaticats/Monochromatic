// What:  unit tests for the `ByteSet` 256-bit alphabet and the shorthand set builders.
// Why:     This file groups the charset test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("charset", () => {
//   // test cases below
// });
// ```

use super::*;

#[test]
fn empty_contains_nothing() {
    let set = ByteSet::empty();
    assert!(set.is_empty());
    for b in 0u8..=u8::MAX {
        assert!(!set.contains(b));
    }
}

#[test]
fn all_bytes_contains_everything() {
    let set = ByteSet::all_bytes();
    assert!(!set.is_empty());
    for b in 0u8..=u8::MAX {
        assert!(set.contains(b));
    }
}

#[test]
fn insert_marks_only_that_byte() {
    let mut set = ByteSet::empty();
    set.insert(b'Q');
    assert!(set.contains(b'Q'));
    assert!(!set.contains(b'P'));
    assert!(!set.contains(b'R'));
    assert_eq!(set.as_singleton(), Some(b'Q'));
}

#[test]
fn insert_range_is_inclusive_on_both_ends() {
    let mut set = ByteSet::empty();
    set.insert_range(b'c', b'f');
    assert!(!set.contains(b'b'));
    assert!(set.contains(b'c'));
    assert!(set.contains(b'd'));
    assert!(set.contains(b'f'));
    assert!(!set.contains(b'g'));
}

#[test]
fn insert_range_handles_byte_extremes() {
    let mut set = ByteSet::empty();
    set.insert_range(0, 255);
    assert!(set.contains(0));
    assert!(set.contains(255));
    assert!(set.contains(128));
}

#[test]
fn union_with_merges_members() {
    let mut a = ByteSet::empty();
    a.insert(b'a');
    let mut b = ByteSet::empty();
    b.insert(b'z');
    a.union_with(&b);
    assert!(a.contains(b'a'));
    assert!(a.contains(b'z'));
    assert!(!a.contains(b'm'));
}

#[test]
fn negate_flips_every_bit() {
    let mut set = ByteSet::empty();
    set.insert(b'x');
    let complement = set.negate();
    assert!(!complement.contains(b'x'));
    assert!(complement.contains(b'y'));
    // What:    Double negation is identity.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(set, complement.negate());
}

#[test]
fn as_singleton_only_for_exactly_one_member() {
    assert_eq!(ByteSet::empty().as_singleton(), None);
    assert_eq!(singleton(b'k').as_singleton(), Some(b'k'));
    let mut two = ByteSet::empty();
    two.insert(b'a');
    two.insert(b'b');
    assert_eq!(two.as_singleton(), None);
}

#[test]
fn singleton_holds_one_byte() {
    let set = singleton(b'_');
    assert!(set.contains(b'_'));
    assert_eq!(set.as_singleton(), Some(b'_'));
}

#[test]
fn dot_set_is_everything_but_newline() {
    let set = dot_set();
    assert!(!set.contains(b'\n'));
    assert!(set.contains(b'a'));
    assert!(set.contains(b'\r'));
    assert!(set.contains(0));
    assert!(set.contains(255));
}

#[test]
fn digit_set_is_ascii_digits() {
    let set = digit_set();
    for b in b'0'..=b'9' {
        assert!(set.contains(b));
    }
    assert!(!set.contains(b'/'));
    assert!(!set.contains(b':'));
    assert!(!set.contains(b'a'));
}

#[test]
fn word_set_matches_is_word_byte() {
    let set = word_set();
    for b in 0u8..=u8::MAX {
        assert_eq!(set.contains(b), is_word_byte(b), "byte {b}");
    }
    assert!(set.contains(b'_'));
    assert!(set.contains(b'A'));
    assert!(set.contains(b'9'));
    assert!(!set.contains(b'-'));
}

#[test]
fn is_word_byte_covers_alnum_and_underscore() {
    assert!(is_word_byte(b'a'));
    assert!(is_word_byte(b'Z'));
    assert!(is_word_byte(b'0'));
    assert!(is_word_byte(b'_'));
    assert!(!is_word_byte(b'-'));
    assert!(!is_word_byte(b' '));
    assert!(!is_word_byte(b'.'));
}

#[test]
fn space_set_is_the_six_ascii_whitespace_bytes() {
    let set = space_set();
    for b in [b' ', b'\t', b'\n', b'\r', 0x0c, 0x0b] {
        assert!(set.contains(b), "whitespace {b}");
    }
    assert!(!set.contains(b'a'));
    assert!(!set.contains(0));
}

#[test]
fn shorthand_negations_are_complements() {
    assert_eq!(digit_set().negate(), digit_set().negate());
    // What:    A negated digit set excludes digits and includes letters.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let non_digit = digit_set().negate();
    assert!(!non_digit.contains(b'5'));
    assert!(non_digit.contains(b'a'));
}
