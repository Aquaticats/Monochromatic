//! What:
//!  The refusal of a discarding command that would write over or delete a worktree entry
//!  nothing else holds.
//! Why:
//!  The person decides here: the command did not run,
//!  so the entry is still where they left it,
//!  and the refusal names each entry,
//!  says in one sentence why Git would lose it,
//!  and gives the one command that runs it anyway.
//!  The option is the guard family's own `--no-enforce-worktree`,
//!  written directly after the subcommand because that is where the wrapper looks for it,
//!  and a command run with it runs exactly as real Git runs it,
//!  with no copy kept.
//!  Every entry is named up to the bound the package's other refusals use,
//!  and the rest are counted,
//!  so a worktree holding thousands of them still prints a sentence a person can read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // overwrittenRefusalMessage({ command: 'reset-hard', region: ['--hard', 'other'], paths: ['b.txt'] })
//! ```

/// The command the words of the refusal name,
///  and the bound on how many entries it lists.
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use crate::rule_linked_worktree::GuardedCommand;
/// The bound on the entries one refusal names before it counts the rest.
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
/// The quoting a command printed for a person to run needs.
use crate::shell_quote::shell_quote;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  Everything one refusal names:
///  the form,
///  the words after it and the entries in its way.
///  A `struct` is a record with named fields.
/// Why:
///   The form and its words make the command the person runs again,
///  and the entries are what that command would lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Overwritten<'a> = { command; region; paths };
/// ```
pub struct Overwritten<'a> {
    /// The guarded form that was refused.
    pub command: GuardedCommand,
    /// The tokens after the subcommand,
    ///  as the guard decided about them.
    pub region: &'a [OsString],
    /// The entries the command would write over or delete,
    ///  as Git printed them.
    pub paths: &'a [String],
}

/// What:
///  The word a form is typed under,
///  which the command the refusal prints starts with.
/// Why:
///   The example has to be a command the person can paste,
///  and the wrapper reads its control only after the subcommand word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subcommand = (command) => 'reset' | 'checkout' | /* ... */ 'read-tree';
/// ```
fn subcommand(command: GuardedCommand) -> &'static str {
    return match command {
        GuardedCommand::Stash | GuardedCommand::StashDrop => "stash",
        GuardedCommand::Clean | GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly => "clean",
        GuardedCommand::ResetHard | GuardedCommand::ResetMixed => "reset",
        GuardedCommand::RestoreStaged | GuardedCommand::RestoreWorktree => "restore",
        GuardedCommand::CheckoutPaths | GuardedCommand::CheckoutMove => "checkout",
        GuardedCommand::SwitchForce | GuardedCommand::SwitchMove => "switch",
        GuardedCommand::Remove => "rm",
        GuardedCommand::Move => "mv",
        GuardedCommand::CheckoutIndex => "checkout-index",
        GuardedCommand::ReadTree => "read-tree",
        GuardedCommand::AbortMerge => "merge",
        GuardedCommand::AbortRebase => "rebase",
        GuardedCommand::AbortPick => "cherry-pick",
        GuardedCommand::AbortAm => "am",
    };
}

/// What:
///  Each entry a refusal names,
///  as a word a shell reads back with the bytes Git printed,
///  up to the bound,
///  and the rest counted.
/// Why:
///   A name with a space,
///  a quote or a newline in it is pasted as one argument only when it is quoted,
///  and a worktree holding every file a tree writes would otherwise print a list nobody reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = (paths) => paths.slice(0, 5).map((p) => `\`${p}\``).join(', ') + (rest ? ` and ${rest} more` : '');
/// ```
fn listed(paths: &[String]) -> String {
    let mut named: Vec<String> = Vec::<String>::with_capacity(SHARED_PATHS_NAMED.min(paths.len()));
    for path in paths.iter().take(SHARED_PATHS_NAMED) {
        named.push(format!("`{path}`"));
    }
    let rest: usize = paths.len().saturating_sub(named.len());
    let and_more: String = match rest {
        0 => String::new(),
        _ => format!(" and {rest} more"),
    };
    return format!("{}{and_more}", named.join(", "));
}

/// What:
///  The command the person runs when they mean to lose those entries after all:
///  the same command with the guard's own opt-out directly after the subcommand.
/// Why:
///   A person who has read the list can act on it without looking up a spelling,
///  and the printed command is the one the wrapper lets through untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hatchCommand = (command, region) => `git ${subcommand(command)} --no-enforce-worktree ${region.map(shellQuote).join(' ')}`;
/// ```
fn hatch_command(command: GuardedCommand, region: &[OsString]) -> String {
    let mut words: Vec<String> = vec![String::from("git"), String::from(subcommand(command)), String::from(WORKTREE_ENFORCEMENT_ESCAPE_HATCH)];
    for token in region {
        words.push(shell_quote(token.as_encoded_bytes()));
    }
    return words.join(" ");
}

/// What:
///  The refusal of one discarding command:
///  what it would lose,
///  why that is a loss,
///  that nothing ran,
///  and the one command that runs it anyway.
/// Why:
///   The person has to know three things in one place:
///  which of their files is at stake,
///  why nothing else holds it,
///  and how to proceed if they meant it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overwrittenRefusalMessage(overwritten: Overwritten): string;
/// ```
pub fn overwritten_refusal_message(overwritten: Overwritten<'_>) -> String {
    let them: &str = if overwritten.paths.len() == 1 { "it" } else { "them" };
    let stands: &str = if overwritten.paths.len() == 1 { "stands" } else { "stand" };
    return format!(
        "monochromatic-git-policy-cli: git {} is rejected because it would write over or delete {}, which {stands} \
         nowhere else: Git did not put {them} in this worktree, so the repository holds no copy. Nothing in your \
         worktree was changed. If you have decided to lose {them} on purpose, run `{}`, and the command runs exactly as \
         real Git runs it.",
        subcommand(overwritten.command),
        listed(overwritten.paths),
        hatch_command(overwritten.command, overwritten.region),
    );
}

/// The refusal's words.
#[cfg(test)]
#[path = "rule_linked_worktree_refused_overwrite_messages_tests.rs"]
mod tests;
