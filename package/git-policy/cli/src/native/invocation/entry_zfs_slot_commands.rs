//! What:
//!  What one forwarded Git command does when it meets a managed clone's slot:
//!  the refusal of a move,
//!  the pass a prune runs over the wrapper's own records,
//!  and the repair report a repair command prints before Git starts.
//! Why:
//!  The design refuses `git worktree move` for a bound slot whether the clone is
//!  mounted or not,
//!  and its commands report the slot,
//!  its dataset and the one next safe action rather than acting on absent files
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The words live in the ZFS folders that own them,
//!  and this file decides only when a command asks for them,
//!  so the command path stays one reading of Git's own words.
//!  Every refusal here changes nothing:
//!  it prints what happened,
//!  what is kept and the next safe command,
//!  and exits with the wrapper's refusal status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = moveRefusalOf(command, commonDir) ?? pruneRefusalOf(command, commonDir, stderr);
//! ```

use crate::action::Action;
use crate::forwarded_command::ResolvedCommand;
use crate::global_arguments::{
    GlobalLayout,
    GlobalOutcome,
    global_layout,
};
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_add_plan::{
    copy_start_directory,
    start_directory,
};
use crate::worktree_clone_binding_record::registration_home::holds_a_record;
use crate::worktree_clone_link_repair::link_report_lines;
use crate::worktree_clone_link_repair::live_link_report::{
    repair_paths,
    the_links_report,
};
use crate::worktree_clone_move_refusal::{
    ManagedMoveRefusal,
    refuse_move_of_a_bound_slot,
};
use crate::worktree_clone_prune_pointer::prune_guard::{
    PruneGuard,
    PruneRefusal,
    guard_the_prune,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The one refusal status a command that changed nothing ends in.
/// Why:
///  Every refusal of this wrapper uses it,
///  so a caller distinguishes a condition from a failure.
pub const REFUSAL_STATUS: i32 = REFUSED_EXIT_CODE;

/// What:
///  The subcommand word of one `git worktree` command,
///  which Git never abbreviates.
/// Why:
///  The move refusal,
///  the prune pass and the repair notice run for their own subcommand only,
///  and `git worktree` declares its subcommands rather than options;
///  each of them runs for every forwarded command,
///  so each reads this word before it reads anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = worktreeSubcommand('worktree', ['prune', '--verbose']); // 'prune'
/// ```
fn worktree_subcommand(command: &ResolvedCommand) -> Option<&[u8]> {
    if command.word() != b"worktree" {
        return None;
    }
    let first: &OsString = command
        .region()
        .iter()
        .find(|token| return !token.as_encoded_bytes().starts_with(b"-"))?;
    return Some(first.as_encoded_bytes());
}

/// What:
///  Whether this prune removes nothing,
///  by Git's own reading of its own options.
/// Why:
///  A dry run must not write,
///  so the pass that restores a pointer stays out of it;
///  Git accepts abbreviations of `--dry-run` and `--no-dry-run` and the short
///  `-n`,
///  and only its parser decides which of them stands last
///  (`worktree_clone_prune_pointer::prune_removes_nothing`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dry = pruneRemovesNothing(['prune', '-nv']);
/// ```
fn prune_removes_nothing(command: &ResolvedCommand) -> bool {
    return crate::worktree_clone_prune_pointer::a_prune_that_removes_nothing(command.region());
}

/// What:
///  The two `git worktree` words this file refuses when a global option the
///  wrapper does not read stands before them.
/// Why:
///  A removal deletes a bound slot's registration and underlay pointer,
///  and a move leaves the record and the registration naming different paths,
///  so both are refused where the operand cannot be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotCommandWord = 'remove' | 'move';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotCommandWord {
    /// `git worktree remove`
    Remove,
    /// `git worktree move`
    Move,
}

impl SlotCommandWord {
    /// The word as the command line spells it.
    fn word(self) -> &'static str {
        return match self {
            SlotCommandWord::Remove => "remove",
            SlotCommandWord::Move => "move",
        };
    }

    /// What the command would have done,
    /// which the refusal says did not happen.
    fn done(self) -> &'static str {
        return match self {
            SlotCommandWord::Remove => "removed",
            SlotCommandWord::Move => "moved",
        };
    }
}

/// What:
///  The word of one `git worktree` subcommand this file refuses,
///  or nothing for every other word.
/// Why:
///  `git worktree` declares its subcommands rather than options,
///  so the word after it says which command Git would run.
fn slot_command_word(bytes: &[u8]) -> Option<SlotCommandWord> {
    return match bytes {
        b"remove" => Some(SlotCommandWord::Remove),
        b"move" => Some(SlotCommandWord::Move),
        _ => None,
    };
}

/// What:
///  One `git worktree remove` or `git worktree move` behind a global option this
///  build does not read,
///  with that option,
///  and with the token after it that names the repository when one does.
/// Why:
///  The refusal names the option that stopped the reading,
///  and `-C`,
///  `--git-dir` or `--work-tree` after it names a repository the
///  wrapper never read,
///  so nothing proves which repository the command names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnreadGlobal = { option: string; word: 'remove' | 'move'; repositoryNamedBy?: string };
/// ```
struct UnreadGlobal<'a> {
    /// The global option the boundary scan stopped at.
    option: &'a OsString,
    /// The `git worktree` word the words after it name.
    word: SlotCommandWord,
    /// The token after the option that names the repository Git runs in,
    /// when one does.
    repository_named_by: Option<&'a OsString>,
}

/// What:
///  Whether one token of the unread region names the repository Git runs in.
/// Why:
///  Git reads these globals before any command,
///  so one of them after the option the scan could not read names a repository
///  the wrapper never read,
///  and the set is the separated spellings `global_arguments::VALUE_OPTIONS`
///  holds plus their attached `=` forms.
///
/// What:
///  The command one line would run behind a global option this build does not
///  read,
///  or nothing when the reading reached the command word.
/// Why:
///  A global option a newer Git adds stops the boundary scan before the
///  command word,
///  so the resolved command names no subcommand,
///  and the words after that option are what says which command would run.
///  The scan is the wrapper's copy of Git 2.56's global table
///  (`global_arguments::scan_globals`),
///  the same set `copy_start_directory` reads,
///  so the token named here is the one that leaves the operand unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreadGlobalOf(command): UnreadGlobal | undefined;
/// ```
fn unread_global_of(command: &ResolvedCommand) -> Option<UnreadGlobal<'_>> {
    let layout: GlobalLayout = global_layout(command.arguments.as_slice());
    if layout.outcome != GlobalOutcome::InvalidOption {
        return None;
    }
    let option: &OsString = command.arguments.get(layout.prefix_len)?;
    let mut repository_named_by: Option<&OsString> = None;
    // Git stops its option loop at the command word, so the first `worktree` after the option it cannot read starts the
    //  command whose words follow it.
    for (index, token) in command
        .arguments
        .iter()
        .enumerate()
        .skip(layout.prefix_len + 1)
    {
        if token.as_encoded_bytes() != b"worktree" {
            // A repository-naming global here names one the reading stopped before, so the wrapper cannot follow it.
            if repository_named_by.is_none()
                && crate::global_arguments::names_the_place_the_command_acts(token.as_encoded_bytes())
            {
                repository_named_by = Some(token);
            }
            continue;
        }
        let subcommand: &OsString = command.arguments.get(index + 1)?;
        let word: SlotCommandWord = slot_command_word(subcommand.as_encoded_bytes())?;
        return Some(UnreadGlobal {
            option,
            word,
            repository_named_by,
        });
    }
    return None;
}

/// What:
///  Whether this command is a `git worktree remove` or `git worktree move`
///  behind a global option the wrapper does not read.
/// Why:
///  The planner asks before it lets a command with no index write through,
///  because that is the plain forward a command the boundary scan cannot place
///  would otherwise take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hidden = isASlotCommandBehindAnUnreadGlobal(command);
/// ```
pub fn is_a_slot_command_behind_an_unread_global(command: &ResolvedCommand) -> bool {
    return unread_global_of(command).is_some();
}

/// What:
///  The sentence one refusal of a removal or a move behind a global option this
///  build does not read prints in a repository that holds slot records.
/// Why:
///  It names the option it could not read and the two spellings the wrapper
///  reads,
///  so a person has a command that runs instead of a refusal
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
fn unread_global_sentence(
    option: &OsString,
    word: SlotCommandWord,
) -> String {
    let typed: std::borrow::Cow<'_, str> = option.to_string_lossy();
    let what: String = crate::global_arguments::unread_global_words(option);
    return format!(
        "`git worktree {word}` is refused: {what}, so nothing proves which worktree the command names, and this repository holds the records of managed-clone slots. Nothing was {done} and Git did not run. Run the command again without `{typed}`, or write it as `git -C <repository> worktree {word} ...`: the options the wrapper reads leave the operand in view, so a slot a binding record proves is a whole clone is protected either way.",
        word = word.word(),
        done = word.done()
    );
}

/// What:
///  The sentence one refusal prints where a global option the wrapper does not
///  read stands before the words that name the repository.
/// Why:
///  The reading stops at that option,
///  so `-C`,
///  `--git-dir` or `--work-tree` after it names a repository the
///  wrapper never read,
///  and nothing proves whether that repository holds slot records:
///  the owner's answer of 2026-11-26 refuses that spelling too
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
fn unknown_repository_sentence(
    option: &OsString,
    naming: &OsString,
    word: SlotCommandWord,
) -> String {
    let typed: std::borrow::Cow<'_, str> = option.to_string_lossy();
    let naming_typed: std::borrow::Cow<'_, str> = naming.to_string_lossy();
    let what: String = crate::global_arguments::unread_global_words(option);
    return format!(
        "`git worktree {word}` is refused: {what}, and `{naming_typed}` after it names the repository, so nothing proves which repository the command names, and nothing proves whether that repository holds the records of managed-clone slots. Nothing was {done} and Git did not run. Run the command again without `{typed}`, or write it as `git -C <repository> worktree {word} ...` with no global option this wrapper does not read: the wrapper then reads that repository's records, and a slot a binding record proves is a whole clone is protected.",
        word = word.word(),
        done = word.done()
    );
}

/// What:
///  The action one refusal of a slot command ends in:
///  the words printed before it,
///  then its sentence,
///  on standard error,
///  and the wrapper's refusal status.
/// Why:
///  A person who turned a policy down to `warn` in order to be told reads what that policy said before any
///  refusal of this file,
///  exactly as a removal's refusal prints the policy stage's lines ahead of its own (#6624);
///  a refusal built from its sentence alone dropped them.
///  Both arms are the same shape,
///  and a second copy of it could report one arm's status as a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = refusalAction(sentence, printedSoFar);
/// ```
fn refusal_action(
    sentence: String,
    printed_so_far: &str,
) -> Action {
    return Action::Exit {
        code: REFUSAL_STATUS,
        stdout: String::new(),
        // What the policy stage printed leads the refusal, in the order a removal prints it.
        stderr: format!("{printed_so_far}{sentence}\n"),
    };
}

/// What:
///  The refusal of one `git worktree remove` or `git worktree move` behind a
///  global option this build does not read,
///  in a repository that holds slot records or where the words name a repository
///  the wrapper never read.
/// Why:
///  The owner's answer of 2026-11-26:
///  a repository that holds slot records refuses the command rather than leave
///  it to Git,
///  whose removal would delete a bound slot's registration and underlay pointer
///  and whose move would leave the record naming the old path;
///  and a spelling whose repository the wrapper cannot name is refused as well,
///  because nothing proves whether it holds such records.
///  A repository the wrapper can name and that holds no slot record is forwarded
///  as before,
///  and every arm here reads neither a pool nor a mount table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = unreadGlobalRefusalOf(command, commonDir, printedSoFar); // undefined leaves the command to Git
/// ```
pub fn unread_global_refusal_of(
    command: &ResolvedCommand,
    common_dir: Option<&Path>,
    printed_so_far: &str,
) -> Option<Action> {
    let unread: UnreadGlobal<'_> = unread_global_of(command)?;
    // A repository named after the option the reading stopped at is one the wrapper never read, so no record of it can
    //  prove the command safe, whatever repository this process stands in.
    if let Some(naming) = unread.repository_named_by {
        return Some(refusal_action(
            unknown_repository_sentence(unread.option, naming, unread.word),
            printed_so_far,
        ));
    }
    let common: &Path = common_dir?;
    // A repository whose record folder holds no record holds no bound slot, so the command costs no refusal and
    //  reaches Git as before (`SPEC.md`, "Removal of a ZFS worktree clone").
    if !holds_a_record(common) {
        return None;
    }
    return Some(refusal_action(
        unread_global_sentence(unread.option, unread.word),
        printed_so_far,
    ));
}

/// What:
///  The refusal of one `git worktree move` of a bound managed clone,
///  as the action the command ends in.
/// Why:
///  The mounted and the unmounted slot are the same code path,
///  and the kernel protects only the mounted one,
///  so the wrapper is the whole protection for the other.
///  Every forwarded command asks,
///  so the move word is read before the working directory is.
///  The start directory is read over Git's whole global table rather than the
///  ladder's short list,
///  because a person's `-c` or `--no-optional-locks` does not change where the
///  operand lies and must not hide it from the refusal
///  (`SPEC.md`,
///  "Removal of a ZFS worktree clone").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = moveRefusalOf(command, commonDir, current, printedSoFar); // undefined leaves the move to Git
/// ```
pub fn move_refusal_of(
    command: &ResolvedCommand,
    common_dir: Option<&Path>,
    current: Option<&Path>,
    printed_so_far: &str,
) -> Option<Action> {
    // Every forwarded command asks, so the move word is read before the working directory is.
    if worktree_subcommand(command) != Some(b"move") {
        return None;
    }
    let common: &Path = common_dir?;
    let start: Option<PathBuf> =
        copy_start_directory(&command.arguments[..command.subcommand_index], current?);
    let source = crate::worktree_move_source::parse_move_source(
        command.arguments.as_slice(),
        command.subcommand_index,
        start.as_deref(),
        Some(common),
    )?;
    let refusal: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common, source.source.as_path())?;
    return Some(refusal_action(refusal.sentence, printed_so_far));
}

/// What:
///  The pass before one `git worktree prune`:
///  its notices go to standard error,
///  and a slot nothing restores ends the command as a refusal.
/// Why:
///  An explicit prune drops a registration whose pointer is gone at once,
///  so the pointer is put back first,
///  and a prune that would lose a registration nothing can restore stops
///  before Git starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = beforeAPrune(command, commonDir, stderr); // undefined lets the prune run
/// ```
pub fn before_a_prune(
    command: &ResolvedCommand,
    common_dir: Option<&Path>,
    stderr: &mut String,
) -> Option<Action> {
    let common: &Path = common_dir?;
    if worktree_subcommand(command) != Some(b"prune") || prune_removes_nothing(command) {
        return None;
    }
    let guard: PruneGuard = guard_the_prune(common, &LiveMountedDatasets);
    for line in guard.lines.iter() {
        stderr.push_str(line.as_str());
        stderr.push('\n');
    }
    if guard.refused.is_empty() {
        return None;
    }
    let mut refusal: String = String::new();
    for refused in guard.refused.iter() {
        match refused {
            PruneRefusal::Record {
                message,
                ..
            } => {
                refusal.push_str(message.as_str());
                refusal.push('\n');
            }
            PruneRefusal::Link(facts) => {
                for line in link_report_lines(facts) {
                    refusal.push_str(line.as_str());
                    refusal.push('\n');
                }
            }
        }
    }
    refusal.push_str(&format!(
        "nothing was pruned: Git would drop the registration of a managed clone and the clone dataset would then be reachable by nobody. Restore the pointer from the record, then run the prune again; {}.\n",
        crate::worktree_clone_binding_record::NO_REMOVAL_WORDS
    ));
    // `stderr` holds what the policy stage printed and the lines the guard wrote, so the refusal is appended to them
    //  rather than built beside them (#6624).
    stderr.push_str(refusal.as_str());
    return Some(Action::Exit {
        code: REFUSAL_STATUS,
        stdout: String::new(),
        stderr: std::mem::take(stderr),
    });
}

/// What:
///  The report one `git worktree repair` prints before Git runs.
/// Why:
///  Git `2.56.1` rebuilds a missing pointer and still exits 1,
///  and a registration Git already pruned loses HEAD,
///  the index and the lock,
///  so a person reads what the repair can and cannot restore before they act.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeARepair(command, commonDir, start, stderr);
/// ```
pub fn before_a_repair(
    command: &ResolvedCommand,
    common_dir: Option<&Path>,
    stderr: &mut String,
    current: Option<&Path>,
) {
    let Some(common) = common_dir else {
        return;
    };
    // Every forwarded command asks, so the repair word is read before the working directory is.
    let repair: bool = command.word() == b"worktree"
        && command
            .arguments
            .get(command.subcommand_index + 1)
            .is_some_and(|token| return token.as_encoded_bytes() == b"repair");
    if !repair {
        return;
    }
    let Some(started_in) = current else {
        return;
    };
    let Some(start) = start_directory(&command.arguments[..command.subcommand_index], started_in)
    else {
        return;
    };
    let paths: Vec<PathBuf> = repair_paths(
        command.arguments.as_slice(),
        command.subcommand_index,
        start.as_path(),
        Some(common),
    );
    for path in paths {
        let Some(report) = the_links_report(common, path.as_path(), &LiveMountedDatasets) else {
            continue;
        };
        for line in report.lines() {
            stderr.push_str(line.as_str());
            stderr.push('\n');
        }
    }
}

/// What:
///  Every ending and notice one forwarded command's words ask of the slots a
///  binding record names,
///  over one common directory:
///  the move refusal,
///  the repair report and the prune pass.
/// Why:
///  A command that stands in a worktree reaches these through the worktree the
///  identity names,
///  and a command that stands in a Git directory has no worktree while Git
///  still names its repository,
///  so both callers name the common directory once and this file runs the
///  checks in one order.
///  The removal's refusal is not here because it reads the pool,
///  which only the run that would have carried the removal out may do
///  (`removal_refusal_of`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = theSlotEndings(command, commonDir, stderr, current);
/// ```
pub fn the_slot_endings(
    command: &ResolvedCommand,
    common_dir: &Path,
    stderr: &mut String,
    current: Option<&Path>,
) -> Option<Action> {
    if let Some(ending) = move_refusal_of(command, Some(common_dir), current, stderr.as_str()) {
        return Some(ending);
    }
    before_a_repair(command, Some(common_dir), stderr, current);
    return before_a_prune(command, Some(common_dir), stderr);
}

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
///  the command stands behind a `!` alias, and Git could not be asked where that alias's shell runs.
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
///  its words are the command Git runs, and the directory Git runs that command in could not be read.
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "entry_zfs_slot_commands_tests.rs"]
mod tests;
