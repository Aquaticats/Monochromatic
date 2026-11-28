//! What:
//!  That the word the grammar reads is the subcommand of the invocation the
//!  add's own records print,
//!  and that it is the word the design names.
//! Why:
//!  The word is one constant in two folders,
//!  so a drift between them is a person pasting a command that refuses,
//!  and this control fails where the drift is made rather than in a run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(SETTLING_COMMAND_WORDS.endsWith(` ${SETTLE_COMMAND_WORD}`)).toBe(true);
//! ```

use crate::worktree_clone_binding_record::add_state_commands::SETTLING_COMMAND_WORDS;

/// What:
///  Whether the program the add's records print ends with this grammar's word,
///  read beside the word itself.
/// Why:
///  The grammar matches one word,
///  the add prints a whole invocation,
///  and a person who copies it must reach this grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(SETTLING_COMMAND_WORDS.endsWith(' settle-worktree-clone-record')).toBe(true);
/// ```
#[test]
fn the_adds_printed_invocation_names_the_word_the_grammar_reads() {
    let suffix: String = format!(" {}", super::SETTLE_COMMAND_WORD);
    assert!(
        SETTLING_COMMAND_WORDS.ends_with(suffix.as_str()),
        "the add's records print `{SETTLING_COMMAND_WORDS}`, which ends with no word this grammar reads: the subcommand is `{}`",
        super::SETTLE_COMMAND_WORD
    );
}

/// What:
///  That the word is the one the design names.
/// Why:
///  The design's verdict entry V21 fixes the command's name,
///  and a reader of the grammar learns the name from this constant alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(SETTLE_COMMAND_WORD).toBe('settle-worktree-clone-record');
/// ```
#[test]
fn the_word_is_the_one_the_design_names() {
    assert_eq!(
        super::SETTLE_COMMAND_WORD,
        "settle-worktree-clone-record",
        "the grammar reads the word the design names"
    );
}
