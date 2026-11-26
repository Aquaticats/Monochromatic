//! What:
//!  The operands one `git worktree` subcommand was given,
//!  read the way Git's own option parser leaves them.
//! Why:
//!  Two readings of the command path name the worktree a command acts on:
//!  `worktree_move_source::parse_move_source` for `git worktree move`,
//!  and `worktree_remove_plan::parse_removal_target` for `git worktree remove`.
//!  A refusal that read only some of Git's own option spellings would let the
//!  rest reach the slot,
//!  and Git's parser already answers which words are the operands:
//!  a word that starts with `-` and has more than that character is an option,
//!  `--` ends the options and is no operand itself,
//!  a word of one `-` is an operand,
//!  and everything else is an operand in the order the person typed it
//!  (`parse_long_opt` and `parse_options_step` in Git 2.56.1's
//!  `parse-options.c`;
//!  `move` and `remove` declare no option that takes a value,
//!  and this scan does not read a `prune`:
//!  prune's own reader takes the value of `--expire <time>` itself,
//!  so that word is never an operand here).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const operands = operandsAfterOptions(['worktree', 'remove', '-f', '-f', 'one']); // ['one']
//! ```

use std::ffi::OsString;

/// What:
///  The operands among the words after one `git worktree` subcommand,
///  in the order they were typed.
/// Why:
///  The scan is the one rule a caller needs,
///  and reading it once means the move refusal and the removal refusal cannot
///  disagree about which word names the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function operandsAfterOptions(tokens: string[]): string[];
/// ```
pub fn operands_after_options(tokens: &[OsString]) -> Vec<&OsString> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut options_end: bool = false;
    for token in tokens {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end && bytes == b"--" {
            options_end = true;
            continue;
        }
        // A word of one `-` is an operand to Git, which reads it as a name and not as an option.
        if !options_end && bytes.len() > 1 && bytes.starts_with(b"-") {
            continue;
        }
        operands.push(token);
    }
    return operands;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_operands_tests.rs"]
mod tests;
