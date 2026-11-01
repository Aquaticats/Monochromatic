//! What:
//!  Remove every wrapper control from one invocation,
//!  before and after the subcommand,
//!       and say how the command's own arguments were read.
//! Why:
//!  Detecting a control and removing it must be one act.
//!  This module is the only place
//!      that removes wrapper tokens:
//!  positions come from Git's own option table of the
//!      command where one is ported,
//!  and are deleted by position,
//!  so a control never
//!      reaches Git and a message,
//!  value or path that spells a control is never removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { args, controls, reading } = stripWrapperControls(rawArgs);
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Each command module reads its region with Git's table and reports where wrapper
///       tokens sat;
///  this module only collects those positions and deletes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseAddRegion } from './command_add.ts';
/// import { withoutTokens } from './command_options_query.ts';
/// ```
use super::command_add::parse_add_region;
use super::command_branch_create::{branch_creation_command, parse_branch_creation_region};
use super::command_clean::parse_clean_region;
use super::command_commit::parse_commit_region;
use super::command_config::parse_config_region;
use super::command_options::{OptionError, WrapperOccurrence};
use super::command_options_query::{WrapperFlags, without_tokens};
use super::command_push::parse_push_region;
use super::command_reset::parse_reset_region;
use super::command_stash::parse_stash_region;
use super::command_status::parse_status_region;
use super::command_worktree::changes_worktree_registrations;
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
use super::policy_registry::PolicyId;
use super::wrapper_controls::{
    CONTROL_SPELLINGS, ControlMeaning, Controls, control_flags, control_meaning, no_controls,
    record_control, strip_global_controls,
};
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments may hold bytes that are not UTF-8 and are forwarded unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  How the tokens after the subcommand were read.
///  An `enum` is a closed set of named
///       alternatives;
///  `Refused` carries Git's refusal.
///  `#[derive(...)]` asks the compiler
///       to generate copying,
///  debug printing and `==`.
/// Why:
///   The caller must know whether controls were found with Git's own grammar,
///  only at
///       the start of the region,
///  or not at all because Git refuses the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegionReading = { kind: 'no-command' | 'table' | 'leading-only' } | { kind: 'refused'; error: OptionError };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionReading {
    /// Git runs no subcommand,
    ///  so there is no region.
    NoCommand,
    /// The command's Git 2.56.0 option table read the whole region.
    Table,
    /// No table is ported for this command:
    ///  only the tokens directly after the subcommand
    /// word,
    ///  which no option can claim as its value,
    ///  were examined.
    LeadingOnly,
    /// Git 2.56.0 itself refuses the region;
    ///  only leading tokens were examined.
    Refused(OptionError),
}

/// What:
///  One invocation with its wrapper controls taken out.
///  A `struct` is a record with
///       named fields;
///  `Vec<OsString>` is an owned argument list.
/// Why:
///   Every later stage reads these arguments,
///  which hold nothing Git does not know,
///       and reads the controls' effect from `controls`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StrippedInvocation = { args: string[]; layout: GlobalLayout; controls: Controls; reading: RegionReading };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrippedInvocation {
    /// The arguments Git may receive:
    ///  no wrapper control in option position remains.
    pub arguments: Vec<OsString>,
    /// Where the subcommand of `arguments` sits and what Git does with the prefix.
    pub layout: GlobalLayout,
    /// What the removed controls asked for.
    pub controls: Controls,
    /// How the command region was read.
    pub reading: RegionReading,
}

/// What:
///  What a command's own escape hatch means,
///  the hatch being the first entry of the
///       flag list its module builds.
/// Why:
///   `git commit` has a hatch for a fixed transform,
///  not for a policy;
///  the other
///       commands' hatches escape a policy;
///  some commands have none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnHatch = { kind: 'none' | 'commit-only' } | { kind: 'policy'; policy: PolicyId };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnHatch {
    /// The command module declares no hatch of its own.
    None,
    /// `--no-enforce-only` of `git commit`.
    CommitOnly,
    /// A hatch that escapes this policy.
    Policy(PolicyId),
}

/// What:
///  Wrapper tokens one table reading found,
///  and what the command's own hatch means.
/// Why:
///   The positions are deleted and the meanings recorded by one shared function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TableControls = { flags: WrapperFlags; own: OwnHatch };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct TableControls {
    /// Own-hatch positions and every other control,
    ///  by region token index.
    flags: WrapperFlags,
    /// The meaning of the own-hatch positions.
    own: OwnHatch,
}

/// What:
///  Wrap the occurrences of a module that declares no hatch of its own.
///       `Vec::<usize>::new()` is an empty owned list (`usize` is the list index type).
/// Why:
///   `push`,
///  `status` and `config` report every wrapper token with the caller's own
///       list index,
///  so nothing is an own hatch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutOwnHatch = (other: WrapperOccurrence[]): TableControls => ({ flags: { escape: [], other }, own: 'none' });
/// ```
fn without_own_hatch(other: Vec<WrapperOccurrence>) -> TableControls {
    return TableControls {
        flags: WrapperFlags {
            escape: Vec::<usize>::new(),
            other,
        },
        own: OwnHatch::None,
    };
}

/// What:
///  Read the region with the command's Git table,
///  when one is ported.
///  `&[u8]` borrows
///       the subcommand word's bytes;
///  `&[&[u8]]` borrows the control spellings.
///       `Result<Option<T>, E>` is "Git's refusal,
///  or a reading,
///  or no table for this word".
/// Why:
///   Only Git's grammar knows which token is an option and which is an option's value.
/// Gotcha:
///  A trailing `?` returns Git's refusal to our caller,
///  or unwraps the reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTable(word: string, region: string[], flags: string[]): TableControls | undefined; // throws OptionError
/// ```
fn read_table(
    word: &[u8],
    region: &[OsString],
    flags: &[&[u8]],
) -> Result<Option<TableControls>, OptionError> {
    if word == b"commit" {
        // `Ok(Some(x))` is success carrying a present value.
        return Ok(Some(TableControls {
            flags: parse_commit_region(region, flags)?.wrapper,
            own: OwnHatch::CommitOnly,
        }));
    }
    if word == b"add" {
        return Ok(Some(TableControls {
            flags: parse_add_region(region, flags)?.wrapper,
            own: OwnHatch::Policy(PolicyId::AddExplicit),
        }));
    }
    if word == b"reset" {
        return Ok(Some(TableControls {
            flags: parse_reset_region(region, flags)?.wrapper,
            own: OwnHatch::Policy(PolicyId::LinkedWorktreeOnly),
        }));
    }
    if word == b"clean" {
        return Ok(Some(TableControls {
            flags: parse_clean_region(region, flags)?.wrapper,
            own: OwnHatch::Policy(PolicyId::LinkedWorktreeOnly),
        }));
    }
    if word == b"stash" {
        return Ok(Some(TableControls {
            flags: parse_stash_region(region, flags)?.wrapper,
            own: OwnHatch::Policy(PolicyId::LinkedWorktreeOnly),
        }));
    }
    // `if let Some(command) = ...` runs only for `branch`, `checkout` and `switch`.
    if let Some(command) = branch_creation_command(word) {
        return Ok(Some(TableControls {
            flags: parse_branch_creation_region(command, region, flags)?.wrapper,
            own: OwnHatch::Policy(PolicyId::BranchWorktreeOnly),
        }));
    }
    if word == b"push" {
        return Ok(Some(without_own_hatch(
            parse_push_region(region, flags)?.wrapper,
        )));
    }
    if word == b"status" {
        return Ok(Some(without_own_hatch(
            parse_status_region(region, flags)?.wrapper,
        )));
    }
    if word == b"config" {
        return Ok(Some(without_own_hatch(
            parse_config_region(region, flags)?.wrapper,
        )));
    }
    // `Ok(None)` is success carrying "no table for this command".
    return Ok(None);
}

/// What:
///  Record what a table reading found and return the region positions to delete.
///       `&mut Controls` lends the record for writing.
/// Why:
///   An own hatch and a general control are deleted the same way;
///  only their recorded
///       effect differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordTableControls(found: TableControls, controls: Controls): number[];
/// ```
fn record_table_controls(found: &TableControls, controls: &mut Controls) -> Vec<usize> {
    // `.clone()` copies the own-hatch positions into the list this function returns.
    let mut positions: Vec<usize> = found.flags.escape.clone();
    if !found.flags.escape.is_empty() {
        // `match` picks one arm per variant; the compiler refuses a forgotten variant.
        match found.own {
            OwnHatch::None => {}
            OwnHatch::CommitOnly => controls.commit_only_escaped = true,
            OwnHatch::Policy(policy) => record_control(controls, ControlMeaning::Escape(policy)),
        }
    }
    // `for occurrence in &found.flags.other` borrows each general control in argument order.
    for occurrence in &found.flags.other {
        record_control(controls, CONTROL_SPELLINGS[occurrence.flag].meaning);
        positions.push(occurrence.token);
    }
    return positions;
}

/// What:
///  Record and return the positions of the controls directly after the subcommand.
/// Why:
///   The first token after the subcommand word,
///  and each token after a control,
///  cannot
///       be the value of an option,
///  whatever the command's grammar is.
///  Further on,
///  only a
///       ported table can tell an option from a value,
///  so the scan stops at the first
///       token that is not a control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leadingControls(region: string[], controls: Controls): number[];
/// ```
fn leading_controls(region: &[OsString], controls: &mut Controls) -> Vec<usize> {
    let mut positions: Vec<usize> = Vec::<usize>::new();
    // `.iter().enumerate()` yields `(index, token)` pairs in order.
    for (index, token) in region.iter().enumerate() {
        // `.as_encoded_bytes()` lends the raw bytes; `let Some(x) = ... else` leaves the loop.
        let Some(meaning) = control_meaning(token.as_encoded_bytes()) else {
            break;
        };
        record_control(controls, meaning);
        positions.push(index);
    }
    return positions;
}

/// What:
///  Record and return the control positions of a command that has no ported table.
/// Why:
///   `git worktree add` and `git worktree move` take the worktree-copy opt-out,
///  which
///       callers write after the second word.
///  That word is a subcommand,
///  so the token
///       after it cannot be an option's value either,
///  and the same leading scan applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function untabledControls(word: string, region: string[], controls: Controls): number[];
/// ```
fn untabled_controls(word: &[u8], region: &[OsString], controls: &mut Controls) -> Vec<usize> {
    let mut positions: Vec<usize> = leading_controls(region, controls);
    // The first token that is not a control sits right after the leading controls.
    let second_word: usize = positions.len();
    if word == b"worktree" && changes_worktree_registrations(&region[second_word..]) {
        let after: usize = second_word + 1;
        // Positions of the inner scan count from `after`, so each is shifted back.
        for position in leading_controls(&region[after..], controls) {
            positions.push(position + after);
        }
    }
    return positions;
}

/// What:
///  Remove every wrapper control from one invocation.
///  `&[OsString]` borrows the
///       arguments after the program name.
/// Why:
///   Controls before the subcommand go first,
///  because Git's global-option reader
///       would otherwise report them as unknown options and every policy decision built
///       on it would leave the command alone.
///  Then the command's own table,
///  when ported
///       and when Git accepts the region,
///  says where the remaining controls sit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stripWrapperControls(args: string[]): StrippedInvocation;
/// ```
pub fn strip_wrapper_controls(arguments: &[OsString]) -> StrippedInvocation {
    // `mut` allows the scans below to add to the record.
    let mut controls: Controls = no_controls();
    // `&mut controls` lends the record for writing.
    let global_clean: Vec<OsString> = strip_global_controls(arguments, &mut controls);
    // `.as_slice()` lends the owned list as a borrowed view.
    let layout: GlobalLayout = global_layout(global_clean.as_slice());
    if layout.outcome != GlobalOutcome::Command {
        return StrippedInvocation {
            arguments: global_clean,
            layout,
            controls,
            reading: RegionReading::NoCommand,
        };
    }
    let region_start: usize = layout.prefix_len + 1;
    // `&global_clean[n..]` borrows the tokens from index `n` on.
    let region: &[OsString] = &global_clean[region_start..];
    let flags: Vec<&'static [u8]> = control_flags();
    // `.as_encoded_bytes()` lends the raw bytes of the subcommand word.
    let word: &[u8] = global_clean[layout.prefix_len].as_encoded_bytes();
    let table: Result<Option<TableControls>, OptionError> =
        read_table(word, region, flags.as_slice());
    // `match` unpacks the three outcomes into the positions to delete and the reading.
    let (positions, reading): (Vec<usize>, RegionReading) = match table {
        Ok(Some(found)) => (
            record_table_controls(&found, &mut controls),
            RegionReading::Table,
        ),
        Ok(None) => (
            untabled_controls(word, region, &mut controls),
            RegionReading::LeadingOnly,
        ),
        Err(error) => (
            leading_controls(region, &mut controls),
            RegionReading::Refused(error),
        ),
    };
    return StrippedInvocation {
        arguments: without_tokens(global_clean.as_slice(), region_start, positions.as_slice()),
        layout,
        controls,
        reading,
    };
}

/// What:
///  The raw bytes of the command word of an invocation that names a command.
///       `&StrippedInvocation` borrows the invocation;
///  `&[u8]` borrows the word's bytes.
/// Why:
///   Every later decision starts from the command word,
///  read in one place.
/// Gotcha:
///  The caller must have seen that the invocation names a command;
///  without one
///         there is no word and this function panics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandWord(stripped: StrippedInvocation): string { return stripped.args[stripped.layout.prefixLen]; }
/// ```
pub fn command_word(stripped: &StrippedInvocation) -> &[u8] {
    // `.as_encoded_bytes()` lends the raw bytes of the token.
    return stripped.arguments[stripped.layout.prefix_len].as_encoded_bytes();
}

/// What:
///  The tokens after the command word of an invocation that names a command.
///       `&[OsString]` borrows those tokens.
/// Why:
///   The refusal frontier and the push gate both read the command's own options;
///       taking the region from one function keeps the two from reading different tokens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandRegion(stripped: StrippedInvocation): string[] { return stripped.args.slice(stripped.layout.prefixLen + 1); }
/// ```
pub fn command_region(stripped: &StrippedInvocation) -> &[OsString] {
    // `&list[n..]` borrows the tokens from index `n` on.
    return &stripped.arguments[stripped.layout.prefix_len + 1..];
}

/// Position,
///  value and separator controls stay out of the release executable.
#[cfg(test)]
#[path = "wrapper_invocation_tests.rs"]
mod tests;
