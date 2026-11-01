// What:  unit tests for `validate_element`, the decode-time check on a counting-NFA
//        position's repetition bound.
// Why:     This file groups the element test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("element", () => {
//   // test cases below
// });
// ```

use super::{Element, MAX_DECODED_COUNT, validate_element};
use crate::charset::{ByteSet, singleton};

#[test]
fn class_and_anchors_always_validate() {
    assert!(validate_element(&Element::Class(singleton(b'a'))).is_ok());
    assert!(validate_element(&Element::LineStart).is_ok());
    assert!(validate_element(&Element::LineEnd).is_ok());
    assert!(validate_element(&Element::WordBoundary).is_ok());
}

#[test]
fn well_formed_counted_validates() {
    let counted = Element::Counted { set: singleton(b'x'), min: 2, max: 5 };
    assert!(validate_element(&counted).is_ok());
    // What:    min == max and min == 0 are both fine.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(validate_element(&Element::Counted { set: singleton(b'x'), min: 4, max: 4 }).is_ok());
    assert!(validate_element(&Element::Counted { set: singleton(b'x'), min: 0, max: 3 }).is_ok());
}

#[test]
fn counted_with_empty_set_is_rejected() {
    let counted = Element::Counted { set: ByteSet::empty(), min: 1, max: 3 };
    assert!(validate_element(&counted).is_err());
}

#[test]
fn counted_with_min_greater_than_max_is_rejected() {
    let counted = Element::Counted { set: singleton(b'x'), min: 5, max: 2 };
    assert!(validate_element(&counted).is_err());
}

#[test]
fn counted_bound_at_the_cap_is_allowed_but_beyond_is_rejected() {
    let at_cap = Element::Counted { set: singleton(b'x'), min: 0, max: MAX_DECODED_COUNT };
    assert!(validate_element(&at_cap).is_ok());
    let beyond = Element::Counted { set: singleton(b'x'), min: 0, max: MAX_DECODED_COUNT + 1 };
    assert!(validate_element(&beyond).is_err());
}
