//! Controls for diagnostic line formatting.

use super::*;

/// A diagnostic is one line,
///  whatever its message holds.
#[test]
fn a_diagnostic_is_one_line() {
    assert_eq!(
        diagnostic_line(
            "debug",
            "2026-10-06T21:40:39.524Z",
            "recovery",
            "owner 7 alive"
        ),
        "[debug] [2026-10-06T21:40:39.524Z] [cli-git] [recovery] owner 7 alive\n"
    );
    assert_eq!(
        diagnostic_line("warn", "t", "capture", "a\nb\r\nc\u{1b}[31m"),
        "[warn] [t] [cli-git] [capture] a b  c [31m\n"
    );
}

/// Only the exact values switch the levels.
#[test]
fn the_switches_need_exact_values() {
    assert!(is_true(std::ffi::OsString::from("true")));
    assert!(!is_true(std::ffi::OsString::from("TRUE")));
    assert!(!is_true(std::ffi::OsString::from("1")));
    assert!(is_false(std::ffi::OsString::from("false")));
    assert!(!is_false(std::ffi::OsString::from("0")));
}
