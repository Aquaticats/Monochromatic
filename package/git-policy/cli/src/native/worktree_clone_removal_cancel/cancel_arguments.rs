//! What:
//!  The grammar of `git monochromatic-git-policy-cli cancel-worktree-removal <slot>`:
//!  one positional slot name,
//!  `--help`,
//!  and every caller mistake with its own sentence.
//! Why:
//!  The project's words fix the command's name and its one argument,
//!  "a guarded `cancel-worktree-removal <slot>`"
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`),
//!  and the design places it as a management subcommand taking exactly one positional slot name
//!  and no option
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The grammar").
//!  A caller mistake is exit `2` and a state that prevents the command is exit `1`,
//!  which is the rule the repository's exit codes set,
//!  so the grammar refuses a wrong word before anything is read,
//!  and a name that is no slot name is one of those mistakes.
//!  Nothing here reads a record or the pool:
//!  the wiring round adds the management action that carries the parsed request,
//!  and this module is what that action calls first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const invocation = parseCancelArguments(args); // { kind: 'help' } | { kind: 'cancel'; slot } | refuses with exit 2
//! ```

use crate::worktree_clone_mount::slot_name_refusal;
use std::ffi::OsString;

/// The management subcommand's word,
/// which the wiring round's grammar table also holds.
pub const CANCEL_COMMAND_WORD: &str = "cancel-worktree-removal";

/// The command's own help,
/// printed on standard output with exit `0`.
const THE_HELP: &str = "\
usage: git monochromatic-git-policy-cli cancel-worktree-removal <slot>

Cancel the removal of one Git worktree clone: prove the slot, the clone and the removal's records under the
clone's removal claim, settle what the removal's destruction plan already destroyed, and ask the handler for
one `mount` request that puts the clone back at its slot.

  <slot>   the worktree's folder name under .claude/worktrees

Reports every snapshot the removal destroyed and cannot restore. Destroys nothing, forces no unmount, ends no
holder, and refuses where the clone's head is gone or a destroy outcome is unknown.";

/// What:
///  One run of the command.
/// Why:
///  `--help` prints the usage and exits `0`,
///  and every other shape is one request to prove and reverse,
///  so no caller can reach the proof with an argument the grammar did not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Invocation = { kind: 'help' } | { kind: 'cancel'; slot: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CancelInvocation {
    /// The usage was asked for.
    Help,
    /// One slot is to be cancelled.
    Cancel(CancelRequest),
}

/// What:
///  The one slot a run names.
/// Why:
///  The slot is the record's key and the folder's name,
///  so the request carries the name the grammar proved and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CancelRequest = { slot: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelRequest {
    /// The slot's own name,
    /// a direct child of the repository's worktree folder.
    pub slot: String,
}

/// What:
///  Every way the words are not this command's.
/// Why:
///  All of them are the caller's,
///  so each exits `2`,
///  and each says what this command takes rather than only that it refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mistake = { kind: 'no-argument' } | { kind: 'two-arguments'; ... } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CancelMistake {
    /// No argument was given.
    NoArgument,
    /// More than one positional was given.
    TooManyArguments {
        /// The first.
        first: String,
        /// The second,
        /// which this command has no place for.
        second: String,
    },
    /// An option was given,
    /// and this command takes none.
    AnOption {
        /// The option as it was typed.
        option: String,
    },
    /// The one argument is not a slot name this route accepts.
    NotASlotName {
        /// The argument as it was typed.
        slot: String,
        /// What about it is not a slot name.
        reason: String,
    },
}

impl CancelMistake {
    /// The sentence a person reads,
    /// which names the mistake and the shape this command takes.
    pub fn message(&self) -> String {
        let shape: &str = "`git monochromatic-git-policy-cli cancel-worktree-removal <slot>` takes exactly one slot name and no option; `--help` prints its usage.";
        return match self {
            CancelMistake::NoArgument => format!("no slot was named, so there is nothing to cancel. {shape}"),
            CancelMistake::TooManyArguments { first, second } => format!(
                "two words were named, `{first}` and `{second}`, and this command cancels one slot. {shape}"
            ),
            CancelMistake::AnOption { option } => format!(
                "`{option}` is an option, and this command takes none: what it does is fixed by the records it proves. {shape}"
            ),
            CancelMistake::NotASlotName { slot, reason } => format!(
                "`{slot}` is not a slot name ({reason}), so no worktree folder under `.claude/worktrees` carries it. {shape}"
            ),
        };
    }
}

/// The usage text the help prints.
pub fn usage() -> String {
    return String::from(THE_HELP);
}

/// What:
///  Read the words after `git monochromatic-git-policy-cli cancel-worktree-removal`.
/// Why:
///  The grammar is one positional slot name or the help,
///  and an empty name is no name,
///  so a person who typed nothing learns the shape before any record is read.
///
/// # Errors
/// No argument,
/// a second positional,
/// an option,
/// a name that is not a slot name,
/// or an argument that is not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invocation = parseCancelArguments(['one']);
/// ```
pub fn parse_cancel_arguments(arguments: &[OsString]) -> Result<CancelInvocation, CancelMistake> {
    let Some(first) = arguments.first() else {
        return Err(CancelMistake::NoArgument);
    };
    let first_text: String = text_of(first)?;
    if first_text == "--help" || first_text == "-h" {
        if arguments.len() > 1 {
            return Err(CancelMistake::TooManyArguments { first: first_text, second: text_of(&arguments[1])? });
        }
        return Ok(CancelInvocation::Help);
    }
    if first_text == "--" {
        // `--` ends option parsing, and this command has no option to end, so the word after it is the slot.
        let Some(raw) = arguments.get(1) else {
            return Err(CancelMistake::NoArgument);
        };
        if let Some(extra) = arguments.get(2) {
            return Err(CancelMistake::TooManyArguments { first: text_of(raw)?, second: text_of(extra)? });
        }
        return slot_invocation(text_of(raw)?.as_str());
    }
    if first_text.starts_with('-') {
        return Err(CancelMistake::AnOption { option: first_text });
    }
    if let Some(second) = arguments.get(1) {
        return Err(CancelMistake::TooManyArguments { first: first_text, second: text_of(second)? });
    }
    return slot_invocation(first_text.as_str());
}

/// One argument as text,
/// or the mistake a name this route cannot read is.
fn text_of(argument: &OsString) -> Result<String, CancelMistake> {
    return argument.clone().into_string().map_err(|refused| {
        return CancelMistake::NotASlotName { slot: refused.to_string_lossy().into_owned(), reason: String::from("it is not UTF-8 text") };
    });
}

/// One name as the request,
/// after the slot-name rule the store,
/// the walk and the per-slot claim share.
fn slot_invocation(slot: &str) -> Result<CancelInvocation, CancelMistake> {
    if let Some(reason) = slot_name_refusal(slot) {
        return Err(CancelMistake::NotASlotName { slot: String::from(slot), reason });
    }
    return Ok(CancelInvocation::Cancel(CancelRequest { slot: String::from(slot) }));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "cancel_arguments_tests.rs"]
mod tests;
