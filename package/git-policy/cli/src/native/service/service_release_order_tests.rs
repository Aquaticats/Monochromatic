//! Controls of the order two release texts are read in.

use super::*;
use std::cmp::Ordering;

/// A later patch, minor or major number is the newer release.
#[test]
fn the_numbers_decide_first() {
    assert_eq!(compare("0.6.1-alpha.13", "0.6.1-alpha.14"), Some(Ordering::Less));
    assert_eq!(compare("0.6.2", "0.6.1-alpha.99"), Some(Ordering::Greater));
    assert_eq!(compare("0.7.0", "0.6.9"), Some(Ordering::Greater));
    assert_eq!(compare("1.0.0", "0.99.99"), Some(Ordering::Greater));
    assert_eq!(compare("0.6.1-alpha.13", "0.6.1-alpha.13"), Some(Ordering::Equal));
}

/// A pre-release's number is a number, and a final release is newer than any pre-release of its own numbers.
#[test]
fn a_pre_release_is_read_as_a_number_and_comes_before_the_release() {
    assert_eq!(compare("0.6.1-alpha.9", "0.6.1-alpha.13"), Some(Ordering::Less), "nine before thirteen");
    assert_eq!(compare("0.6.1", "0.6.1-alpha.13"), Some(Ordering::Greater));
    assert_eq!(compare("0.6.1-alpha.13", "0.6.1"), Some(Ordering::Less));
    assert_eq!(compare("0.6.1-beta.2", "0.6.1-alpha.99"), Some(Ordering::Greater), "the names are compared first");
}

/// What:
///  A text this program does not ship is not ordered at all.
/// Why:
///  A service whose release cannot be read is not proven newer, so the caller keeps the old answer and asks it to leave.
#[test]
fn a_text_that_is_not_a_release_is_not_ordered() {
    for bad in ["", "nonsense", "0.6", "0.6.1.2", "0.6.x", "-alpha.1", "0.6.1-alpha", "0.6.1-alpha.x", "0.6.1-"] {
        assert_eq!(compare(bad, "0.6.1-alpha.13"), None, "{bad}");
        assert_eq!(compare("0.6.1-alpha.13", bad), None, "{bad}");
    }
}
