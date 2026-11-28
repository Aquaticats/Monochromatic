//! What:
//!  The word the management grammar reads for the settling command,
//!  and the proof that it is the subcommand of the program the add's own
//!  sentences print.
//! Why:
//!  The add's records name the command as one whole invocation,
//!  `git monochromatic-git-policy-cli settle-worktree-clone-record`,
//!  in `worktree_clone_binding_record::add_state_commands::SETTLING_COMMAND_WORDS`,
//!  and the grammar matches one word after the program's own name.
//!  Two constants that must agree are one constant written twice when nothing
//!  checks them,
//!  so this file holds the subcommand and the control beside it reads the add's
//!  constant and refuses a drift instead of trusting a reader to notice it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const word = SETTLE_COMMAND_WORD; // 'settle-worktree-clone-record'
//! // SETTLING_COMMAND_WORDS must end with ` ${word}`
//! ```

/// The management word that runs the settling command,
/// which is the last word of the invocation the add's records print.
pub const SETTLE_COMMAND_WORD: &str = "settle-worktree-clone-record";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_settling_commands_word_tests.rs"]
mod tests;
