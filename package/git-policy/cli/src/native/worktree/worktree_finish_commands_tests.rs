//! What:
//!  The two commands and the sentence a message prints for a worktree an interrupted add left:
//! the hatch the finish command
//!  names,
//! the quoting of the path,
//! and what the sentence says the reset discards.
//! Why:
//!  The controls in `src/native/binary/binary_printed_remedies_tests.rs` run the printed text end to end;
//! these pin the words and the quoting that
//!  make it run,
//! so a change to either is a change to what a person pastes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(finishAndRemove('/tmp/half')).toEqual(['git -C /tmp/half reset --no-enforce-worktree --hard', ...]);
//! ```

use super::{finish_and_remove, finish_or_remove_sentence};
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use std::path::Path;

/// A path a shell reads back unquoted is printed as it is,
/// and the finish command carries the guard's hatch directly after
///  the subcommand,
/// which is the only place the wrapper reads it.
#[test]
fn a_plain_path_is_printed_bare_with_the_hatch_after_the_subcommand() {
    let (finish, remove): (String, String) = finish_and_remove(Path::new("/home/person/half-added"));
    assert_eq!(
        finish,
        format!("git -C /home/person/half-added reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard"),
        "the hatch is where the wrapper reads it, and the path is bare"
    );
    assert_eq!(remove, "git worktree remove --force /home/person/half-added");
}

/// A path holding a space and a quote is quoted,
/// so the printed command passes one argument and the shell runs nothing else.
#[test]
fn a_path_that_needs_quoting_is_quoted() {
    let (finish, _remove): (String, String) = finish_and_remove(Path::new("/home/person/a b'c"));
    assert_eq!(
        finish,
        format!("git -C '/home/person/a b'\\''c' reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard"),
        "a space and a quote are read back as one word"
    );
}

/// The sentence names both commands and says what the reset discards,
/// so the person is not told to lose work without warning.
#[test]
fn the_sentence_says_what_the_reset_discards() {
    let sentence: String = finish_or_remove_sentence(Path::new("/home/person/half-added"));
    assert!(
        sentence.contains("discarding whatever that worktree holds"),
        "the sentence warns about what the reset discards: {sentence}"
    );
    assert!(
        sentence.contains(WORKTREE_ENFORCEMENT_ESCAPE_HATCH),
        "the sentence names the hatch that makes the reset run: {sentence}"
    );
    assert!(
        sentence.contains("git worktree remove --force /home/person/half-added"),
        "the sentence names the command that removes the worktree instead: {sentence}"
    );
}
