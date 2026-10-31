//! What:
//!  Tests for the shared invariants,
//! with negative controls.
//! Why:
//!  A property check that cannot fail proves nothing,
//!  so each invariant is run once on healthy input and once on deliberately broken input that must make it panic.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('invariants', () => { /* healthy runs and negative controls */ });
//! ```

use monochromatic_xml_edit::parse_xml;

use crate::invariants::{
    assert_append_child, assert_outside_unchanged, assert_parse_or_reject, assert_remove_element, assert_self_consistent,
    assert_set_attribute, assert_usable_rejection, has_unrepresentable,
};

/// A small document with a comment,
/// CDATA and nesting.
const SAMPLE: &str = "<a k=\"1\">\n  <!-- c -->\n  <b><![CDATA[x]]></b>\n  <c/>\n</a>";

/// Check the healthy path of every edit invariant over every element of a sample.
#[test]
fn edit_invariants_hold_on_a_sample() {
    let document = parse_xml(SAMPLE).expect("sample parses");
    for id in document.element_ids() {
        assert_set_attribute(&document, id, "k", "v\"&<>\n\t'");
        assert_set_attribute(&document, id, "fresh", "");
        assert_set_attribute(&document, id, "bad", "bell\u{7}");
        assert_append_child(&document, id);
        if id != document.root() {
            assert_remove_element(&document, id);
        }
    }
    assert_remove_element(&document, document.root());
    assert_self_consistent(&document);
}

/// Check parse-or-reject accepts valid input and diagnoses invalid input.
#[test]
fn parse_or_reject_covers_both_outcomes() {
    assert!(assert_parse_or_reject(SAMPLE).is_some());
    assert!(assert_parse_or_reject("<a>").is_none());
    assert!(assert_parse_or_reject("").is_none());
    assert!(has_unrepresentable("a\u{1}") && has_unrepresentable("\u{FFFF}") && !has_unrepresentable("\t\r\n\u{e9}"));
}

/// Negative control:
/// bytes changed outside the span must make the preservation check panic.
#[test]
#[should_panic(expected = "bytes after the edited element changed")]
fn preservation_check_detects_a_change_after_the_span() {
    assert_outside_unchanged("<a/><!--x-->", "<a/><!--y-->", 0, 4);
}

/// Negative control:
/// bytes changed before the span must make the preservation check panic.
#[test]
#[should_panic(expected = "bytes before the edited element changed")]
fn preservation_check_detects_a_change_before_the_span() {
    assert_outside_unchanged("<!--x--><a/>", "<!--y--><a/>", 8, 12);
}

/// Negative control:
/// a rejection without a message or with a zero position must panic.
#[test]
#[should_panic(expected = "had no message")]
fn rejection_check_detects_an_empty_message() {
    assert_usable_rejection("<a>", 0, 1, 1, "");
}

/// Negative control:
/// an offset past the end of the source must panic.
#[test]
#[should_panic(expected = "past the end")]
fn rejection_check_detects_an_offset_past_the_end() {
    assert_usable_rejection("<a>", 9, 1, 1, "m");
}

/// Negative control:
/// a zero line must panic.
#[test]
#[should_panic(expected = "not one-based")]
fn rejection_check_detects_a_zero_line() {
    assert_usable_rejection("<a>", 0, 0, 1, "m");
}
