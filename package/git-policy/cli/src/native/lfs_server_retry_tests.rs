//! Controls for which `git-lfs` failures are retried, how long the retries wait and how many there are.

use super::*;

/// An environment with the delay variable set to `value`.
fn with_delays(value: &str) -> Vec<(OsString, OsString)> {
    return vec![(OsString::from(RETRY_DELAYS_VARIABLE), OsString::from(value))];
}

/// The incident's output, the 500 wording and the other 5xx wording are server errors, however much text surrounds them.
#[test]
fn the_server_error_wordings_of_git_lfs_are_retried() {
    for output in [
        "batch response: Fatal error: Server error https://<credentials>@lfs.invalid/objects/batch from HTTP 503",
        "batch response: Fatal error: Server error https://lfs.invalid/objects/batch from HTTP 502\nerror: failed to push some refs",
        "LFS: Fatal error: Server error https://lfs.invalid/o from HTTP 504",
        "batch response: Fatal error: Server error: https://lfs.invalid/objects/batch",
        "batch response: Fatal error: the server says it is overloaded",
        "Uploading LFS objects:   0% (0/3), 0 B | 0 B/s\nbatch response: Fatal error: Server error https://h/x from HTTP 599\n",
    ] {
        assert!(is_server_error(output), "{output}");
    }
}

/// A refused credential, a missing repository, a bad request, a missing or corrupt local object, locked files, a status
///  that is not a server error and plain unknown failures are never retried.
#[test]
fn every_other_failure_is_terminal() {
    for output in [
        "batch response: Authorization error: https://lfs.invalid/objects/batch\nCheck that you have proper access to the repository",
        "batch response: Repository or object not found: https://lfs.invalid/objects/batch\nCheck that it exists",
        "batch response: Client error https://lfs.invalid/objects/batch from HTTP 418",
        "batch response: Client error: https://lfs.invalid/objects/batch",
        "batch response: Unprocessable entity: https://lfs.invalid/objects/batch",
        "Git LFS upload failed:\n  (missing) big.bin (abc)\nhint: Your push was rejected due to missing or corrupt local objects.",
        "Git LFS upload failed:\n  (corrupt) big.bin (abc)",
        "Unable to push locked files:\n* a.bin - someone",
        "batch response: Not Implemented: https://lfs.invalid/objects/batch",
        "batch response: Insufficient server storage: https://lfs.invalid/objects/batch",
        "batch response: Bandwidth limit exceeded: https://lfs.invalid/objects/batch",
        "Post \"https://lfs.invalid/objects/batch\": dial tcp 127.0.0.1:9: connect: connection refused",
        "Server error https://lfs.invalid/o from HTTP 404",
        "Server error https://lfs.invalid/o from HTTP 5",
        "Server error https://lfs.invalid/o from HTTP 5x2",
        "error: unknown",
        "",
    ] {
        assert!(!is_server_error(output), "{output}");
    }
}

/// One permanent failure anywhere in the output decides: the server error beside it would be met again.
#[test]
fn a_permanent_failure_beside_a_server_error_is_terminal() {
    let mixed: &str = "batch response: Fatal error: Server error https://h/x from HTTP 503\n  (missing) big.bin (abc)";
    assert!(!is_server_error(mixed));
    let refused: &str = "Server error https://h/x from HTTP 503\nAuthorization error: https://h/y";
    assert!(!is_server_error(refused));
}

/// The default delays are the two named constants, and each retry gets its own entry of the list.
#[test]
fn the_default_delays_are_the_named_constants() {
    assert_eq!(retry_delays(&[]), RETRY_DELAYS.to_vec());
    assert_eq!(RETRY_DELAYS.len(), 2);
    assert!(RETRY_DELAYS.windows(2).all(|pair| { return pair[0] < pair[1]; }));
    let failing: &str = "Server error https://h/x from HTTP 503";
    for (done, expected) in RETRY_DELAYS.iter().enumerate() {
        assert_eq!(next_delay(failing, done, &RETRY_DELAYS), Some(*expected));
    }
    assert_eq!(next_delay(failing, RETRY_DELAYS.len(), &RETRY_DELAYS), None, "the allowance is spent");
    assert_eq!(next_delay("Authorization error", 0, &RETRY_DELAYS), None);
}

/// A test injects its own short delays, an empty value means none, and a malformed or oversized list stays bounded.
#[test]
fn the_variable_replaces_the_delays_and_stays_bounded() {
    assert_eq!(retry_delays(&with_delays("1,2")), vec![Duration::from_millis(1), Duration::from_millis(2)]);
    assert_eq!(retry_delays(&with_delays(" 0 , 7 ")), vec![Duration::ZERO, Duration::from_millis(7)]);
    assert_eq!(retry_delays(&with_delays("")), Vec::<Duration>::new());
    assert_eq!(retry_delays(&with_delays("1,x")), RETRY_DELAYS.to_vec(), "a malformed list is ignored");
    assert_eq!(retry_delays(&with_delays("-1")), RETRY_DELAYS.to_vec());
    let long: Vec<Duration> = retry_delays(&with_delays("1,1,1,1,1,1,1,1,1,1"));
    assert_eq!(long.len(), MAX_CONFIGURED_RETRIES);
    assert_eq!(retry_delays(&with_delays("99999999")), vec![MAX_CONFIGURED_DELAY]);
}

/// The notice names the wait and the position of the retry.
#[test]
fn the_notice_names_the_wait_and_the_retry() {
    let notice: String = retry_notice(Duration::from_secs(5), 1, 2);
    assert!(notice.contains("5.0 seconds") && notice.contains("retry 2 of 2"), "{notice}");
    assert!(notice.ends_with('\n'));
}
