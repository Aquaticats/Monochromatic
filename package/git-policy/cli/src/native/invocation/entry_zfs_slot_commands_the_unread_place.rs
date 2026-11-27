//! What:
//!  The refusal of a slot command the wrapper read and could not place:
//!  a removal,
//! a move or a prune behind a `!` alias whose shell's directory Git never answered.
//! Why:
//!  `alias_run_directory` answers nothing when Git cannot be asked where a `!` alias's shell runs,
//!  and the slot's start is then nothing,
//!  so the operand and the records of the command are unread.
//!  The owner's answers of 2026-11-26 refuse every slot command the wrapper cannot prove safe,
//!  and a command nothing placed is one nothing proves
//!  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
//!  The words live in a file of their own so the slot glue stays under the linter's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = unplacedRefusalOf(command, printedSoFar); // undefined leaves it to Git
//! ```

use crate::action::Action;
/// The ending a refusal is,
/// and the one sentence builder the slot glue prints refusals through.
use crate::entry_zfs_slot_commands::{
    refusal_action,
    worktree_subcommand,
};
/// The command Git will run,
/// after alias expansion.
use crate::forwarded_command::ResolvedCommand;

/// What:
///  The three `git worktree` commands whose pass reads a slot,
///  each as the words a refusal of it prints.
/// Why:
///  A refusal says what did not happen,
///  and each command has its own verb for that;
///  one table keeps a sentence from naming the wrong one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ACommandThatReadsASlot = 'remove' | 'move' | 'prune';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ACommandThatReadsASlot {
    /// `git worktree remove`
    Remove,
    /// `git worktree move`
    Move,
    /// `git worktree prune`
    Prune,
}

impl ACommandThatReadsASlot {
    /// The command one resolved slot command names,
    ///  or nothing for every other command.
    fn of(command: &ResolvedCommand) -> Option<Self> {
        return match worktree_subcommand(command)? {
            b"remove" => Some(Self::Remove),
            b"move" => Some(Self::Move),
            b"prune" => Some(Self::Prune),
            _ => None,
        };
    }

    /// The word as the command line spells it.
    fn word(self) -> &'static str {
        return match self {
            Self::Remove => "remove",
            Self::Move => "move",
            Self::Prune => "prune",
        };
    }

    /// What the command would have done,
    ///  which the refusal says did not happen.
    fn done(self) -> &'static str {
        return match self {
            Self::Remove => "removed",
            Self::Move => "moved",
            Self::Prune => "pruned",
        };
    }
}

/// What:
///  Whether this slot command's own words name a place of their own,
///  so the repository whose records its pass reads is not the one the caller's options reach.
/// Why:
///  A prune reads no operand and no directory,
///  so only a command that names a place with its own `-C`,
///  `--git-dir` or `--work-tree` can send its pass to records the wrapper did not read;
///  the removal and the move read an operand from the directory Git runs them in and cannot be placed at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesItsOwnPlace = (command) => command.args.slice(0, command.subcommandIndex).some(namesThePlace);
/// ```
fn names_a_place_of_its_own(command: &ResolvedCommand) -> bool {
    return command
        .arguments
        .get(..command.subcommand_index)
        .unwrap_or(&[])
        .iter()
        .any(|token| {
            return crate::global_arguments::names_the_place_the_command_acts(
                token.as_encoded_bytes(),
            );
        });
}

/// What:
///  The sentence a refusal prints for a slot command the wrapper read and could not place:
///  the command stands behind a `!` alias,
/// and Git could not be asked where that alias's shell runs.
/// Why:
///  The owner's answers of 2026-11-26 refuse every slot command the wrapper cannot prove safe
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`),
///  and a command whose directory could not be read is one nothing proves:
///  its operand may name a bound slot in a repository nothing here read.
///  The sentence names what could not be read and both spellings the wrapper reads,
///  so a person has a command that runs instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = whereNothingCouldBeRead(command);
/// ```
fn the_unread_place_sentence(word: ACommandThatReadsASlot) -> String {
    return format!(
        "`git worktree {word}` is refused: the command stands behind a `!` alias, and Git could not be asked where that alias's shell runs, so nothing proves which worktree or repository the command names and nothing proves whether a slot a binding record names is the one it acts on. Nothing was {done} and Git did not run. Run the command again without the alias, or write it as `git -C <repository> worktree {word} ...`, which the wrapper reads in full.",
        word = word.word(),
        done = word.done()
    );
}

/// What:
///  The refusal of a slot command the wrapper read and could not place:
///  its words are the command Git runs,
/// and the directory Git runs that command in could not be read.
/// Why:
///  A slot command behind a `!` alias is read from the top of the working tree Git changes to before it starts the
///  shell
///  (`git.c:388` to `git.c:407`),
///  and a Git that cannot be asked for that directory leaves the operand and the records unread.
///  Forwarding it in silence is what the alpha.50 review's L2 found:
///  a bound slot's removal went through whenever the ask failed.
///  The shell alias whose value the wrapper cannot read at all is not this case and stays as it is
///  (`doc/planning/cli-git-slot-refusals-and-shell-aliases-they-cannot-read.md`,
///  the choice still before the owner).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = unplacedRefusalOf(command, printedSoFar); // undefined leaves it to Git
/// ```
pub(crate) fn unplaced_refusal_of(
    command: &ResolvedCommand,
    printed_so_far: &str,
) -> Option<Action> {
    let word: ACommandThatReadsASlot = ACommandThatReadsASlot::of(command)?;
    // A prune reads no operand, so the caller's own repository is the one its records belong to unless the command
    //  names a place of its own.
    if word == ACommandThatReadsASlot::Prune && !names_a_place_of_its_own(command) {
        return None;
    }
    return Some(refusal_action(
        the_unread_place_sentence(word),
        printed_so_far,
    ));
}
