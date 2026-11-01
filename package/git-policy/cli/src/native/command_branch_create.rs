//! What:
//!  Branch-creation facts of `git branch`,
//!  `git checkout` and `git switch` regions,
//!       read as Git 2.56.0 reads them.
//! Why:
//!  The branch-worktree policy rejects creating a branch in the current worktree.
//!  It
//!      needs the explicit forms and the name Git may turn into a new local branch by
//!      matching a remote branch,
//!  for every spelling Git accepts.
//! Gotcha:
//!  The facts read option names and argument counts only.
//!  Where Git later refuses
//!         for an option value,
//!  a combination of options,
//!  configuration or repository
//!         state,
//!  the answer stays "creates":
//!  it errs toward the policy seeing the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseBranchCreationRegion({ subcommand: 'switch', postSubcommandArgs: ['-c', 'topic'] }).createsBranch
//! ```

/// What:
///  Bring the tables,
///  the mode and target decisions,
///  the shared hatch spelling,
///  the
///       tokenizer and its questions into this file.
/// Why:
///   This module only selects the command's table and combines the decisions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions } from './command_options.ts';
/// ```
use super::command_branch_mode::branch_creates;
use super::command_branch_table::BRANCH_TABLE;
use super::command_branch_target::{
    checkout_target, creates_explicitly, guess_allowed, switch_target,
};
use super::command_checkout_table::{CHECKOUT_TABLE, SWITCH_TABLE};
use super::command_options::{DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, parse_options};
use super::command_options_query::{WrapperFlags, positional_tokens, split_wrapper_flags};
use super::escape_hatch::BRANCH_WORKTREE_ESCAPE_HATCH;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The guarded subcommands.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Each has its own option table and its own rule for what a positional name means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BranchCreationSubcommand = 'branch' | 'checkout' | 'switch';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchCreationCommand {
    /// `git branch`.
    Branch,
    /// `git checkout`.
    Checkout,
    /// `git switch`.
    Switch,
}

/// What:
///  Facts of one guarded region.
///  `Option<usize>` is "a token index or nothing".
/// Why:
///   `implicit_creation_target` names the argument a repository probe must check
///       against remote branches;
///  this module never runs that probe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BranchCreationRegion = { createsBranch: boolean; hasEscapeHatch: boolean;
///   implicitCreationTarget: string | typeof NO_IMPLICIT_CREATION_TARGET };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchCreationRegion {
    /// The options ask Git to create,
    ///  reset or copy a branch.
    pub creates_branch: bool,
    /// Region token index of the name Git would create by matching one remote branch.
    pub implicit_creation_target: Option<usize>,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-worktree-branch`.
    pub wrapper: WrapperFlags,
}

/// What:
///  The guarded command a subcommand word names,
///  or nothing.
///  `&[u8]` borrows the word.
/// Why:
///   Callers hold the subcommand token as raw bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guarded = ['branch', 'checkout', 'switch'].includes(word) ? word : undefined;
/// ```
pub fn branch_creation_command(word: &[u8]) -> Option<BranchCreationCommand> {
    if word == b"branch" {
        // `Some(x)` is the "present" case of `Option`.
        return Some(BranchCreationCommand::Branch);
    }
    if word == b"checkout" {
        return Some(BranchCreationCommand::Checkout);
    }
    if word == b"switch" {
        return Some(BranchCreationCommand::Switch);
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  Parse the region after a guarded subcommand.
///  `Result<A, B>` is "either success `A`
///       or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal;
///  Git then creates nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseBranchCreationRegion({ subcommand, postSubcommandArgs }): BranchCreationRegion; // throws
/// ```
pub fn parse_branch_creation_region(
    command: BranchCreationCommand,
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<BranchCreationRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(BRANCH_WORKTREE_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    // `match` picks one arm per variant; the compiler refuses a forgotten variant.
    let table: &[OptionSpec] = match command {
        BranchCreationCommand::Branch => BRANCH_TABLE,
        BranchCreationCommand::Checkout => CHECKOUT_TABLE,
        BranchCreationCommand::Switch => SWITCH_TABLE,
    };
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, table, DEFAULT_MODE, flags.as_slice())?;
    let wrapper: WrapperFlags = split_wrapper_flags(parsed.wrapper.as_slice());
    if command == BranchCreationCommand::Branch {
        let names: usize = positional_tokens(&parsed, region.len()).len();
        // `Ok(x)` is the success case of `Result`.
        return Ok(BranchCreationRegion {
            creates_branch: branch_creates(&parsed, names),
            implicit_creation_target: None,
            wrapper,
        });
    }
    let creates_branch: bool = creates_explicitly(&parsed);
    let mut implicit_creation_target: Option<usize> = None;
    if !creates_branch && guess_allowed(&parsed) {
        implicit_creation_target = if command == BranchCreationCommand::Checkout {
            checkout_target(&parsed, region)
        } else {
            switch_target(&parsed, region)
        };
    }
    return Ok(BranchCreationRegion {
        creates_branch,
        implicit_creation_target,
        wrapper,
    });
}

/// `git branch` creation,
///  escape hatch positions and refusals.
#[cfg(test)]
#[path = "command_branch_create_tests.rs"]
mod tests;

/// Explicit creation and the guess candidate of `git checkout` and `git switch`.
#[cfg(test)]
#[path = "command_branch_create_target_tests.rs"]
mod target_tests;

/// Real Git 2.56.0 controls:
///  the three tables and a differential on `git branch`.
#[cfg(test)]
#[path = "command_branch_create_git_tests.rs"]
mod git_tests;

/// Real Git 2.56.0 controls:
///  creation by `git checkout` and `git switch`.
#[cfg(test)]
#[path = "command_branch_create_guess_tests.rs"]
mod guess_tests;
