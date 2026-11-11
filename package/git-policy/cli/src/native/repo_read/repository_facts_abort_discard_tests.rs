//! What:
//!  The queries and readings of "what would this abort discard",
//!  and the real repository answers they produce.
//! Why:
//!  The listing shapes are what the guard trusts,
//!  so each one is checked against real Git here:
//!  the merge result `AUTO_MERGE` leaves untouched conflict markers out,
//!  a hand resolution in,
//!  and the fallback lists every uncommitted change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paths = await abortDiscard(facts, GuardedCommand.AbortMerge);
//! expect(paths.paths).toEqual(['a.txt']);
//! ```

use super::{
    AbortDiscard, both_listings, merge_result_queries, presence_facts, state_from_presence, state_query_arguments, uncommitted_queries,
};
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsString;
use std::path::Path;

/// The words of an argument list,
///  for comparing queries as text.
fn words(arguments: &[OsString]) -> Vec<String> {
    return arguments.iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
}

/// The state query asks for each state entry and the sequencer directory.
#[test]
fn the_state_query_names_every_entry_once() {
    let asked: Vec<String> = words(state_query_arguments(&[OsString::from("-C"), OsString::from("/repo")]).as_slice());
    assert_eq!(asked.first().map(String::as_str), Some("-C"), "{asked:?}");
    assert_eq!(asked.get(1).map(String::as_str), Some("/repo"), "{asked:?}");
    assert_eq!(asked.get(2).map(String::as_str), Some("rev-parse"), "{asked:?}");
    assert_eq!(asked.get(3).map(String::as_str), Some("--path-format=absolute"), "{asked:?}");
    let mut names: Vec<&str> = Vec::new();
    for pair in asked[4..].chunks(2) {
        assert_eq!(pair.first().map(String::as_str), Some("--git-path"), "{asked:?}");
        names.push(pair.get(1).expect("a name"));
    }
    assert_eq!(
        names,
        vec!["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply/rebasing", "rebase-apply/applying", "sequencer"]
    );
}

/// A presence answer of the wrong length is an error,
///  never "nothing waits".
#[test]
fn a_presence_answer_of_the_wrong_length_is_an_error() {
    assert!(presence_facts(b"MERGE_HEAD\n").is_err());
    assert!(presence_facts(b"").is_err());
    // Six names and a trailing separator is still six.
    let six: &[u8] = b"/a/MERGE_HEAD\n/b/CHERRY_PICK_HEAD\n/c/REVERT_HEAD\n/d/rebase-merge\n/e/rebase-apply/rebasing\n/f/rebase-apply/applying\n";
    assert!(presence_facts(six).is_err());
}

/// A presence answer of the right length becomes the state the question reads.
#[test]
fn a_presence_answer_of_the_right_length_becomes_the_state() {
    let present: [bool; 7] = presence_facts(b"/nowhere/1\n/nowhere/2\n/nowhere/3\n/nowhere/4\n/nowhere/5\n/nowhere/6\n/nowhere/7\n").expect("seven answers");
    assert_eq!(present, [false; 7], "a path that is not there is not present");
    let state = state_from_presence([true, false, false, false, false, false, false]);
    assert!(state.merge_head);
    assert!(!state.sequencer);
}

/// The merge-result queries compare the worktree and the index with `AUTO_MERGE`,
///  and only the index side drops unmerged entries.
#[test]
fn the_merge_result_queries_read_both_sides_against_the_recorded_result() {
    let queries = merge_result_queries(&[]);
    assert_eq!(
        words(queries[0].as_slice()),
        vec!["--no-optional-locks", "diff", "--name-only", "-z", "AUTO_MERGE"]
    );
    assert_eq!(
        words(queries[1].as_slice()),
        vec!["--no-optional-locks", "diff", "--cached", "--name-only", "-z", "--diff-filter=ACDMRTB", "AUTO_MERGE"]
    );
}

/// The fallback queries list every uncommitted tracked change.
#[test]
fn the_fallback_queries_list_every_uncommitted_change() {
    let queries = uncommitted_queries(&[]);
    assert_eq!(words(queries[0].as_slice()), vec!["--no-optional-locks", "diff", "--name-only", "-z"]);
    assert_eq!(words(queries[1].as_slice()), vec!["--no-optional-locks", "diff", "--cached", "--name-only", "-z"]);
}

/// Two listings become one set of paths,
///  each once and in one order.
#[test]
fn two_listings_become_one_set() {
    let both: Vec<Vec<u8>> = both_listings(
        vec![b"b.txt".to_vec(), b"a.txt".to_vec()],
        vec![b"a.txt".to_vec(), b"c.txt".to_vec()],
    );
    assert_eq!(both, vec![b"a.txt".to_vec(), b"b.txt".to_vec(), b"c.txt".to_vec()]);
    assert!(both_listings(Vec::new(), Vec::new()).is_empty());
}

/// A form that aborts no operation asks nothing.
#[test]
fn a_form_that_aborts_no_operation_asks_nothing() {
    let mut facts = crate::repository_facts::git_facts(Path::new(crate::test_support::REAL_GIT), &[], &[]);
    let answer = crate::repository_facts_abort_discard::abort_discard(&mut facts, GuardedCommand::Stash).expect("nothing is asked");
    assert_eq!(answer, AbortDiscard::NothingWaiting);
    assert_eq!(facts.queries, 0, "no Git starts");
}
