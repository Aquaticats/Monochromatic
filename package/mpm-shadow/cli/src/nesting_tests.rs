//! Unit tests of the depth guard.

use crate::nesting::{MAX_NESTING, must_refuse, nesting_depth};

#[test]
fn no_value_and_unreadable_values_name_no_depth() {
    assert_eq!(nesting_depth(None), 0);
    for text in ["", "x", "-1", "1.5", " 2"] {
        assert_eq!(nesting_depth(Some(text)), 0, "{text:?}");
    }
}

#[test]
fn a_number_names_its_depth() {
    assert_eq!(nesting_depth(Some("1")), 1);
    assert_eq!(nesting_depth(Some(&MAX_NESTING.to_string())), MAX_NESTING);
}

#[test]
fn the_guard_refuses_from_the_maximum_depth_on_and_not_before() {
    assert!(!must_refuse(0));
    assert!(!must_refuse(MAX_NESTING - 1));
    assert!(must_refuse(MAX_NESTING));
    assert!(must_refuse(MAX_NESTING + 1));
}
