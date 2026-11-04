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

/// A leader that exits while a descendant keeps the output pipes open is bounded as a whole: the run reports the timeout within its
/// limit (the descendant sleeps far longer than the limit) and the descendant's group is killed, instead of waiting on the readers.
#[test]
fn a_descendant_that_keeps_the_pipes_open_after_the_leader_exits_is_bounded_and_killed() {
    let mut command: Command = Command::new("/bin/sh");
    command.args(["-c", "sleep 8 & exit 0"]);
    let started = std::time::Instant::now();
    let outcome = run_bounded(&mut command, Duration::from_millis(500));
    assert!(started.elapsed() < Duration::from_secs(5), "the whole operation, output collection included, is bounded: {:?}", started.elapsed());
    assert!(outcome.expect_err("the pipes never closed in time").contains("timed out"));
}
