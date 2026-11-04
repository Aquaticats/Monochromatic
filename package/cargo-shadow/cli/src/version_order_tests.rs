//! The unit tests of the version ordering,
//! ported from `version-order.unit.test.ts`.

use std::cmp::Ordering;

use crate::version_order::{caret_compatible, compare_texts};

#[test]
fn ordering_follows_semver_including_prereleases() {
    assert_eq!(compare_texts("1.2.3", "1.2.3"), Ordering::Equal);
    assert_eq!(compare_texts("1.10.0", "1.9.0"), Ordering::Greater);
    assert_eq!(compare_texts("1.0.0-rc.1", "1.0.0"), Ordering::Less);
    assert_eq!(compare_texts("1.0.0-alpha", "1.0.0-alpha.1"), Ordering::Less);
    assert_eq!(compare_texts("1.0.0-alpha.2", "1.0.0-alpha.10"), Ordering::Less);
    assert_eq!(compare_texts("1.0.0-1", "1.0.0-alpha"), Ordering::Less);
}

#[test]
fn the_caret_class_is_major_then_minor_then_patch() {
    assert!(caret_compatible("1.4.0", "1.9.2"));
    assert!(!caret_compatible("1.4.0", "2.0.0"));
    assert!(caret_compatible("0.4.2", "0.4.9"));
    assert!(!caret_compatible("0.4.2", "0.5.0"));
    assert!(caret_compatible("0.0.3", "0.0.3"));
    assert!(!caret_compatible("0.0.3", "0.0.4"));
}

#[test]
fn text_that_is_not_a_version_is_never_compatible() {
    assert!(!caret_compatible("1.0.0", "nope"));
    assert!(!caret_compatible("nope", "1.0.0"));
}
