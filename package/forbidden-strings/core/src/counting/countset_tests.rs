// What:  unit tests for `CountSet`, the per-position bitset of live repetition counts.
// Why:     This file groups the countset test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("countset", () => {
//   // test cases below
// });
// ```

use super::*;

#[test]
fn new_is_empty_with_no_counts() {
    let set = CountSet::new(8);
    assert!(set.is_empty());
    assert!(!set.has_at_least(0));
    assert!(!set.has_at_least(1));
}

#[test]
fn insert_zero_seeds_count_zero_once() {
    let mut set = CountSet::new(4);
    assert!(set.insert_zero(), "first insert reports a change");
    assert!(!set.insert_zero(), "second insert is a no-op");
    assert!(!set.is_empty());
    assert!(set.has_at_least(0));
    assert!(!set.has_at_least(1));
}

#[test]
fn clear_empties_the_set() {
    let mut set = CountSet::new(4);
    set.insert_zero();
    set.clear();
    assert!(set.is_empty());
    assert!(!set.has_at_least(0));
}

#[test]
fn advance_steps_every_count_up_by_one() {
    let mut src = CountSet::new(8);
    src.insert_zero();
    let mut dst = CountSet::new(8);
    dst.copy_advanced_from(&src, 8);
    // What:    The single count moved from 0 to 1.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(dst.has_at_least(1));
    assert!(!dst.has_at_least(2));
}

#[test]
fn advance_preserves_a_spread_of_counts() {
    // What:    Build {0, 2} by advancing twice with a fresh zero each step.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut a = CountSet::new(8);
    // What:    {0}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    a.insert_zero();
    let mut b = CountSet::new(8);
    // What:    {1}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    b.copy_advanced_from(&a, 8);
    // What:    {0,1}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    b.insert_zero();
    let mut c = CountSet::new(8);
    // What:    {1,2}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    c.copy_advanced_from(&b, 8);
    assert!(c.has_at_least(1));
    assert!(c.has_at_least(2));
    assert!(!c.has_at_least(3));
}

#[test]
fn advance_drops_a_count_that_would_exceed_max() {
    // What:    A single count sitting at `max` is dropped when advanced past the bound.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut src = CountSet::new(3);
    src.insert_zero();
    let mut at_max = CountSet::new(3);
    // What:    Advance to 1, 2, 3 (== max), each carrying the value forward.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut step = CountSet::new(3);
    // What:    {1}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    step.copy_advanced_from(&src, 3);
    // What:    {2}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    at_max.copy_advanced_from(&step, 3);
    // What:    {3}.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    step.copy_advanced_from(&at_max, 3);
    assert!(step.has_at_least(3));
    // What:    Advancing the count at max (3) past the bound drops it: the set goes empty.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut overflowed = CountSet::new(3);
    overflowed.copy_advanced_from(&step, 3);
    assert!(overflowed.is_empty(), "count beyond max is dropped");
}

#[test]
fn has_at_least_is_a_lower_bound_test() {
    // What:    Put a single count at 5 (max 8) and probe the threshold around it.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut set = CountSet::new(8);
    set.insert_zero();
    for _ in 0..5 {
        let mut next = CountSet::new(8);
        next.copy_advanced_from(&set, 8);
        set = next;
    }
    assert!(set.has_at_least(5));
    assert!(set.has_at_least(4));
    assert!(!set.has_at_least(6));
}

#[test]
fn counts_can_cross_a_word_boundary() {
    // What:    max above 64 forces a multi-word bitset; a count past bit 63 must still
    //          register.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut set = CountSet::new(80);
    set.insert_zero();
    for _ in 0..70 {
        let mut next = CountSet::new(80);
        next.copy_advanced_from(&set, 80);
        set = next;
    }
    assert!(set.has_at_least(70));
    assert!(!set.has_at_least(71));
}

#[test]
fn has_at_least_scans_higher_words_for_a_low_threshold() {
    // What:    A count parked in a higher word must satisfy a LOW threshold through the
    //          higher-words scan, not just the in-word shift test the boundary case exercises.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut set = CountSet::new(80);
    // What:    count 70 lives in word 1, bit 6.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    set.words[1] = 1 << 6;
    // What:    word 0 holds nothing; only the higher-words scan sees it.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(set.has_at_least(5));
    assert!(set.has_at_least(64));
    assert!(set.has_at_least(70));
    assert!(!set.has_at_least(71));
}

#[test]
fn nwords_sizes_to_address_the_max_plus_one_bit() {
    // What:    Off-by-one or wrong arithmetic here mis-sizes every CountSet of that element.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(nwords(0), 1);
    assert_eq!(nwords(62), 1);
    // What:    bit 64 needs a second word.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert_eq!(nwords(63), 2);
    assert_eq!(nwords(64), 2);
    assert_eq!(nwords(80), 2);
    assert_eq!(nwords(128), 3);
}
