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
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};

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
        GuardedCommand::SkipRebase => "rebase",
        GuardedCommand::SkipPick => "cherry-pick",
        GuardedCommand::SkipAm => "am",
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

/// What:
///  The refusal of a form that recurses into a submodule whose own worktree holds uncommitted changes,
///  naming each change and the one command that runs the form anyway.
///  `&[String]` borrows the changes as this worktree spells them.
/// Why:
///  A submodule's worktree holds a person's files as much as the outer one,
///  and a recursive checkout, switch, reset or restore moves each submodule to the commit the target
///  tree records, replacing what that submodule's repository does not hold.
///  The sentence cannot say "Git did not put them in this worktree", which is true of an untracked
///  file: Git did put these there, and it is the person's uncommitted change to them that goes,
///  so the refusal says that instead and names each one with the submodule in its name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// submoduleChangesRefusalMessage({ command: 'checkout-paths', region: ['-f','--recurse-submodules','other'], paths: ['sub/f.txt'] });
/// ```
pub fn submodule_changes_refusal_message(overwritten: Overwritten<'_>) -> String {
    let them: &str = if overwritten.paths.len() == 1 { "it" } else { "them" };
    return format!(
        "monochromatic-git-policy-cli: git {} is rejected because it recurses into a submodule and would replace \
         the uncommitted changes {} inside it, which that submodule's own repository does not hold. Nothing in your \
         worktree was changed. If you have decided to lose {them} on purpose, run `{}`, and the command runs exactly as \
         real Git runs it, moving each submodule to the commit the target records.",
        subcommand(overwritten.command),
        listed(overwritten.paths),
        hatch_command(overwritten.command, overwritten.region),
    );
}

/// What:
///  The refusal of a form whose written paths the guard could not read,
///  naming what could not be read and the command that runs it anyway.
///  `&str` borrows the cause,
///  which names the tree and what Git did.
/// Why:
///  A guard that cannot read what a command would write must not read the failure as
///  "nothing is written":
///  a remote-only branch, a tree whose object is unreadable and a transient failed read all
///  answer with a failed listing,
///  and one of them was the loss this reading exists to prevent.
///  The person reads which name failed, that nothing ran and that the option runs the command as
///  real Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// treeReadingFailedMessage('checkout', ['-f', 'topic'], 'the guard could not read the tree `topic` names ...');
/// ```
pub fn tree_reading_failed_message(command: GuardedCommand, region: &[OsString], cause: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: git {} is rejected because {cause}. Nothing in your worktree was changed. If \
         you have decided to run it anyway, run `{}`, and the command runs exactly as real Git runs it.",
        subcommand(command),
        hatch_command(command, region),
    );
}

/// What:
///  The refusal of a command one of whose words is an option the guard does not read,
///  naming that word and the command that runs it anyway.
///  `&[u8]` borrows the word as the person typed it.
///  A `struct` is a record with named fields.
/// Why:
///  The guard decides from Git 2.56.0's own option table for each subcommand it reads.
///  A word no table holds is one whose value and whose meaning the guard cannot know:
///  it may be a newer Git's option that discards, so reading it as harmless is a guess, and a
///  guess is what let `git rm -f --pathspec-from-file --no-force` delete an edit.
///  The person reads which word the guard could not read, that nothing ran, and that the option
///  runs the command as real Git runs it, where Git's own parser answers the word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unreadOptionMessage({ command: 'rm', region: ['-f', '--pathspec-from-file', '--no-force'], written: '--no-force' });
/// ```
pub struct UnreadOption<'a> {
    /// The guarded form whose region holds the word.
    pub command: GuardedCommand,
    /// The tokens after the subcommand,
    ///  as the guard decided about them.
    pub region: &'a [OsString],
    /// The word no option table of this subcommand holds.
    pub written: &'a [u8],
}

/// What:
///  The sentence that refusal prints.
/// Why:
///  It says which word stopped the guard,
///  why the guard cannot read it,
///  that nothing ran,
///  and the one command that runs it anyway,
///  which is what a person needs to act.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreadOptionMessage(unread: UnreadOption): string;
/// ```
pub fn unread_option_message(unread: UnreadOption<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git {} is rejected because the guard does not read the option `{}` of this \
         command, so it cannot tell what the command would write over or delete. Nothing in your worktree was \
         changed. If you have decided to run it anyway, run `{}`, and the command runs exactly as real Git runs it.",
        subcommand(unread.command),
        String::from_utf8_lossy(unread.written),
        hatch_command(unread.command, unread.region),
    );
}

/// What:
///  The refusal of a `!` shell alias whose own words read a pathspec file,
///  naming the file and the command that runs it anyway.
///  `&[OsString]` borrows the words the shell runs and the command the person types.
/// Why:
///  The guard reads that file to know which paths the command writes,
///  and the shell Git starts reads it again:
///  a pipe has one writer, so the second read waits for one that is gone,
///  and a file changed in between makes Git act on names the guard never judged.
///  The wrapper forwards a `!` alias's value unchanged, because the shell keeps the `GIT_DIR` and
///  `GIT_PREFIX` Git gives it,
///  so it cannot put the bytes it read in place of the file's name.
///  The person reads which file stopped it and how to run the command as real Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathspecUnderAShellAliasMessage({ command: 'restore', region: ['--source=HEAD', '--pathspec-from-file=list'], file: 'list' });
/// ```
pub struct PathspecUnderAShellAlias<'a> {
    /// The guarded form the alias runs.
    pub command: GuardedCommand,
    /// The words the shell runs, after the subcommand.
    pub region: &'a [OsString],
    /// The file the alias reads its pathspecs from.
    pub file: &'a OsStr,
}

/// What:
///  The sentence that refusal prints.
/// Why:
///  It has to name the file Git would read a second time,
///  why the wrapper cannot hand over the bytes it read,
///  that nothing ran,
///  and the one command that runs it anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathspecUnderAShellAliasMessage(refusal): string;
/// ```
pub fn pathspec_under_a_shell_alias_message(refusal: PathspecUnderAShellAlias<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git {} is rejected because the `!` shell alias you ran reads its pathspecs from `{}`, and the wrapper reads that file before Git does: under a shell alias the wrapper cannot put the bytes it read in the file's place, so Git would read them again (a pipe waits for a writer that is gone). Nothing in your worktree was changed. If you have decided to run it anyway, run `{}`, and the command runs exactly as real Git runs it.",
        subcommand(refusal.command),
        refusal.file.to_string_lossy(),
        hatch_command(refusal.command, refusal.region),
    );
}

/// The refusal's words.
#[cfg(test)]
#[path = "rule_linked_worktree_refused_overwrite_messages_tests.rs"]
mod tests;
