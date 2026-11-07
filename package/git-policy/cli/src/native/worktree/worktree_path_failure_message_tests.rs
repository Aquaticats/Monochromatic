//! Controls for the message shape of a failed step on a path.

use super::{path_failure, subject_failure};
use std::path::Path;

#[test]
fn a_failure_names_the_path_the_words_and_the_error() {
    let outcome: Result<u8, String> = Err(String::from("denied"));
    let message: String = path_failure(outcome, Path::new("/a/b"), "cannot be examined").expect_err("a failure");
    assert_eq!(message, "/a/b cannot be examined (denied)");
}

#[test]
fn a_success_passes_its_value_through() {
    let outcome: Result<u8, String> = Ok(7);
    let value: u8 = path_failure(outcome, Path::new("/a/b"), "cannot be examined").expect("a value");
    assert_eq!(value, 7);
}

#[test]
fn a_failure_names_a_subject_that_is_not_a_path() {
    let outcome: Result<u8, String> = Err(String::from("denied"));
    let message: String = subject_failure(outcome, &"the snapshot", "cannot be held").expect_err("a failure");
    assert_eq!(message, "the snapshot cannot be held (denied)");
}
