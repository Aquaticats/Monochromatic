//! What:
//!  Controls of the disposable runner: every process a fixture starts is bounded in time.
//! Why:
//!  "Nothing should ever wait forever or indefinitely" binds the test fixtures too; a hung Git must end the test with a message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a runner kills a process that outlives its limit', ...);
//! ```

use crate::model_fixture::run_bounded;
use std::process::Command;
use std::time::Duration;

/// A process that outlives its limit is killed and reported, not waited for.
#[test]
fn a_process_that_outlives_its_limit_is_killed_and_reported() {
    let mut command: Command = Command::new("/usr/bin/sleep");
    command.arg("30");
    let started = std::time::Instant::now();
    let outcome = run_bounded(&mut command, Duration::from_millis(300));
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(outcome.expect_err("timed out").contains("timed out"));
}

/// A process that ends in time returns its output.
#[test]
fn a_process_that_ends_in_time_returns_its_output() {
    let mut command: Command = Command::new("/usr/bin/echo");
    command.arg("done");
    let output = run_bounded(&mut command, Duration::from_secs(30)).expect("ends");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
}
