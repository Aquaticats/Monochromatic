//! Controls for what a waiting notice says about the lock holder.

use super::*;
use crate::test_support::{fixture, remove};

/// The holder here is this test process: it has a command line, and its lock directory has an age.
#[test]
fn a_live_holder_is_named_with_its_command_and_how_long_it_held_the_lock() {
    let root = fixture("holder-facts-live");
    let pid: i64 = i64::from(std::process::id());
    let deadline: Duration = Duration::from_secs(7);
    let facts: String = holder_facts(pid, root.as_path(), deadline);
    assert!(facts.contains("The holder has held the lock for "), "{facts}");
    assert!(facts.contains(" seconds and runs `"), "{facts}");
    assert!(facts.contains(&format!("Its push is stopped after {} seconds", deadline.as_secs())), "{facts}");
    assert!(!facts.contains('\n'), "the notice stays one line: {facts}");
    remove(root.as_path());
}

/// A holder that has gone and a lock that cannot be read leave only the limit, never an invented command or age.
#[test]
fn an_unreadable_holder_and_lock_leave_only_the_limit() {
    let root = fixture("holder-facts-gone");
    let missing = root.join("no-such-lock");
    let deadline: Duration = Duration::from_secs(3);
    let facts: String = holder_facts(i64::from(i32::MAX), missing.as_path(), deadline);
    assert!(!facts.contains("The holder"), "{facts}");
    assert!(facts.starts_with(" Its push is stopped after "), "{facts}");
    remove(root.as_path());
}

/// A command line is cut to a readable length and its separators become spaces.
#[cfg(target_os = "linux")]
#[test]
fn a_command_line_is_one_readable_line_of_bounded_length() {
    let command: Option<String> = holder_command(i64::from(std::process::id()));
    let shown: String = command.expect("this process has a command line");
    assert!(shown.chars().count() <= MAX_COMMAND_CHARACTERS, "{shown}");
    assert!(!shown.contains('\0') && !shown.contains('\n'), "{shown}");
}
