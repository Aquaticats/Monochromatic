//! Controls for the wait limit: its value from the environment or the default.

use super::*;
use std::ffi::OsString;


/// The limit follows the variable and falls back to the default for anything unusable.
#[test]
fn the_limit_comes_from_the_variable_or_the_default() {
    assert_eq!(live_wait_limit_from(None), DEFAULT_LIVE_WAIT);
    // The default only has to outlast an ordinary wait: a zero one would give up at once.
    assert!(DEFAULT_LIVE_WAIT > Duration::ZERO);
    assert_eq!(
        live_wait_limit_from(Some(OsStr::new("90"))),
        Duration::from_secs(90)
    );
    for value in ["0", "", "-1", "1.5", "soon"] {
        assert_eq!(
            live_wait_limit_from(Some(OsStr::new(value))),
            DEFAULT_LIVE_WAIT,
            "{value:?}"
        );
    }
}

/// A value that is not valid text is unusable.
#[cfg(unix)]
#[test]
fn a_non_text_limit_falls_back_to_the_default() {
    use std::os::unix::ffi::OsStringExt;
    let value: OsString = OsString::from_vec(vec![0xff, 0xfe]);
    assert_eq!(live_wait_limit_from(Some(value.as_os_str())), DEFAULT_LIVE_WAIT);
}

/// Variable the helper process is started with.
const HELPER_EXPECTED: &str = "LIVE_LOCK_TEST_EXPECTED_SECONDS";

/// Helper process body,
///  started by the test below with the limit variable set:
///  the limit this process runs under is the variable's.
#[test]
#[ignore = "started as a helper process by the environment test"]
fn helper_reads_the_limit_from_its_environment() {
    let expected: u64 = std::env::var(HELPER_EXPECTED)
        .expect("expected")
        .parse()
        .expect("seconds");
    assert_eq!(live_wait_limit(), Duration::from_secs(expected));
}

/// Run the helper with an expectation and, when given, the limit variable;
///  whether the helper's assertion held.
fn helper_agrees(value: Option<&str>, expected: u64) -> bool {
    let mut command: std::process::Command =
        std::process::Command::new(crate::test_support::this_test_binary().expect("test binary"));
    command
        .args([
            "--exact",
            "wait_limit::tests::helper_reads_the_limit_from_its_environment",
            "--ignored",
            "--test-threads=1",
            "--quiet",
        ])
        .env(HELPER_EXPECTED, expected.to_string())
        .env_remove(LIVE_WAIT_VARIABLE);
    if let Some(text) = value {
        command.env(LIVE_WAIT_VARIABLE, text);
    }
    return command.status().expect("start the helper").success();
}

/// The process environment reaches the limit:
///  a child started with the variable runs under it, and one started without it runs under the default.
///  A wrong expectation must fail the helper, which proves the helper really ran its assertion.
#[test]
fn the_process_environment_sets_the_limit() {
    assert!(!helper_agrees(Some("7"), 8), "the helper must fail a wrong expectation");
    assert!(!helper_agrees(None, 7), "the helper must fail a wrong default");
    for (value, expected) in [(Some("7"), 7_u64), (Some("0"), 1800), (Some("junk"), 1800), (None, 1800)] {
        assert!(helper_agrees(value, expected), "{value:?}");
    }
}
