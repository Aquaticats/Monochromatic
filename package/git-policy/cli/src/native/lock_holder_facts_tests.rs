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
    assert!(facts.contains(" in "), "{facts}");
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

/// Only the program and the subcommand are shown: global options and their values, and every later argument, are not.
#[test]
fn only_the_program_and_subcommand_are_shown() {
    let secret: &str = "http.extraHeader=Authorization: secret-value";
    let words: Vec<&str> = vec!["/usr/bin/git", "-C", "/some/dir", "-c", secret, "--no-pager", "push", "--force", "origin"];
    let shown: String = program_and_subcommand(words.as_slice()).expect("a program");
    assert_eq!(shown, "git push");
    assert!(!shown.contains("secret") && !shown.contains("origin"));
    assert_eq!(program_and_subcommand(&["git"]), Some(String::from("git")));
    assert_eq!(program_and_subcommand(&[]), None);
}

/// The holder's record shows its program and working directory, never the arguments of this test process.
#[cfg(target_os = "linux")]
#[test]
fn the_holders_record_names_its_directory_and_no_arguments() {
    let shown: String = holder_command(i64::from(std::process::id())).expect("this process has a record");
    let directory = std::env::current_dir().expect("directory");
    assert!(shown.ends_with(&format!(" in {}", directory.display())), "{shown}");
    let before_directory: &str = shown.rsplit_once(" in ").map_or("", |(front, _)| {
        return front;
    });
    assert!(before_directory.split(' ').count() <= 2, "a program and at most one subcommand: {shown}");
}
