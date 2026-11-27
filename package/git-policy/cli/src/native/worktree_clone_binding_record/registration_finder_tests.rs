//! Controls for the finder's names:
//!  Git's own sanitiser,
//!  the counters a stale registration makes,
//!  and the names another slot's binding owns.

use crate::worktree_clone_binding_record::registration_finder::{
    MOST_CANDIDATES,
    candidate_names,
    git_sanitized_name,
    owned_by_another_slot,
};

/// Git's sanitiser collapses `..`,
/// replaces a leading dot and a forbidden byte,
/// turns `@{`'s brace into `-`,
/// strips a trailing `.lock` repeatedly and keeps a trailing dot.
#[test]
fn gits_sanitiser_makes_the_names_git_makes() {
    assert_eq!(git_sanitized_name("topic"), "topic");
    assert_eq!(git_sanitized_name(".hidden"), "-hidden");
    assert_eq!(git_sanitized_name("two words"), "two-words");
    assert_eq!(git_sanitized_name("a..b"), "a-b");
    assert_eq!(git_sanitized_name("a@{b"), "a@-b");
    assert_eq!(git_sanitized_name("k.lock.lock"), "k");
    assert_eq!(git_sanitized_name("x."), "x.");
    assert_eq!(git_sanitized_name("a:b"), "a-b");
    assert_eq!(git_sanitized_name("a~b"), "a-b");
}

/// The candidates are the sanitised name and Git's decimal counters.
#[test]
fn the_candidates_are_the_name_and_its_counters() {
    let names: Vec<String> = candidate_names("topic");
    assert_eq!(names.len(), MOST_CANDIDATES);
    assert_eq!(names[0], "topic");
    assert_eq!(names[1], "topic1");
    assert_eq!(
        names[MOST_CANDIDATES - 1],
        format!("topic{}", MOST_CANDIDATES - 1)
    );
    let numbered: Vec<String> = candidate_names("topic1");
    assert_eq!(numbered[0], "topic1");
    assert_eq!(numbered[1], "topic11");
}

/// One slot's candidates never include another's bare name,
/// so `topic1`'s binding owns the entry slot `topic`'s finder must leave.
#[test]
fn another_slots_candidates_own_their_names() {
    let others: Vec<String> = vec![String::from("topic1")];
    assert!(owned_by_another_slot("topic1", "topic", others.as_slice()));
    assert!(!owned_by_another_slot("topic", "topic", others.as_slice()));
    assert!(!owned_by_another_slot(
        "topic1",
        "topic1",
        others.as_slice()
    ));
    assert!(!owned_by_another_slot("topic2", "topic", others.as_slice()));
}
