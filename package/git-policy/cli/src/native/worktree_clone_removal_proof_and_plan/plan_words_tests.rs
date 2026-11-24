//! What:
//!  The controls of the plan's words:
//!  the key order,
//!  the five snapshot outcomes,
//!  the five head words,
//!  and the header line the add's reader already knows.
//! Why:
//!  One closed set per word is what lets a reader refuse a later grammar rather than guess,
//!  and the key order is the rule that refuses a repeated key and an out-of-place one at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(keys).toEqual([...]); expect(Outcome.parse('destroyed')).toBe('destroyed');
//! ```

use super::{HeadWord, PLAN_HEADER, PLAN_KEYS_IN_ORDER, SnapshotOutcome};
use crate::worktree_clone_binding_record::plan_folder_read::PLAN_HEADER as THE_ADD_READERS_HEADER;

#[test]
fn the_key_order_is_the_one_the_header_writes_and_holds_no_repeat() {
    assert_eq!(PLAN_KEYS_IN_ORDER.len(), 18);
    let mut sorted: Vec<&str> = PLAN_KEYS_IN_ORDER.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), PLAN_KEYS_IN_ORDER.len(), "no key is written twice");
    assert_eq!(PLAN_KEYS_IN_ORDER[0], "run");
    assert_eq!(PLAN_KEYS_IN_ORDER[PLAN_KEYS_IN_ORDER.len() - 1], "attempts");
}

#[test]
fn the_header_line_is_the_adds_own_constant() {
    assert_eq!(PLAN_HEADER, THE_ADD_READERS_HEADER);
    assert_eq!(PLAN_HEADER, "# monochromatic-git-policy-cli zfs worktree clone destruction plan, version 1");
}

#[test]
fn each_outcome_word_reads_back_as_itself_and_no_other_word_reads() {
    for outcome in [SnapshotOutcome::Planned, SnapshotOutcome::Destroyed, SnapshotOutcome::Refused, SnapshotOutcome::Intact, SnapshotOutcome::Unknown] {
        assert_eq!(SnapshotOutcome::parse(outcome.word()), Some(outcome));
    }
    for refused in ["", "Planned", "gone", "intact ", "destroyed!"] {
        assert_eq!(SnapshotOutcome::parse(refused), None, "`{refused}` is no outcome word");
    }
}

#[test]
fn each_head_word_reads_back_as_itself_and_no_other_word_reads() {
    for head in [HeadWord::Present, HeadWord::Requested, HeadWord::Gone, HeadWord::Refused, HeadWord::Unknown] {
        assert_eq!(HeadWord::parse(head.word()), Some(head));
    }
    for refused in ["", "Present", "absent", "request"] {
        assert_eq!(HeadWord::parse(refused), None, "`{refused}` is no head word");
    }
}
