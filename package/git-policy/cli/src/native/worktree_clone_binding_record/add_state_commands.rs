//! What:
//!  The one command that is true for an add state a resume meets,
//!  and the sentence a refusal prints.
//! Why:
//!  The design states one list so that every reader,
//!  the add's refusal,
//!  the removal and `doctor`,
//!  names the same command for the same state
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "Which command is true for a state").
//!  A clone whose absence the pool cannot prove names the same settling command as one it can:
//!  the pool refusing its listing is evidence a person confirms,
//!  never a reason to leave the slot with no command
//!  (the design writer's K14, added after the eleventh read).
//!  A binding at `cloned` or `mounted` whose clone is gone is the settling command's,
//!  proved or not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const command = commandForState({ phase, clone: 'gone-unprovable', registrationStands: true, planStands: false });
//! // { kind: 'settling' }
//! ```

use super::binding_words::Phase;

/// What:
///  Whether the recorded clone still stands,
///  is gone and proved gone,
///  or is gone and the pool could not prove it.
/// Why:
///  The third arm is the one a refused or partial listing leaves,
///  and it names the same command as the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClonePresence = { kind: 'stands' } | { kind: 'gone-proved' } | { kind: 'gone-unprovable'; evidence: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClonePresence {
    /// The clone answers under the pinned parent.
    Stands,
    /// The pinned parent's children carry neither the GUID nor the name.
    GoneProved,
    /// The pool refused the listing or answered it only in part,
    /// and the evidence is what the refusal said.
    GoneUnprovable {
        /// Why the absence cannot be proved.
        evidence: String,
    },
}

/// What:
///  The live facts a reader has before it names a command.
/// Why:
///  The answers come from the record's phase and the live reads,
///  and the command follows from them alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateFacts = { phase: Phase; clone?: ClonePresence; registrationStands: boolean; planStands: boolean; checkoutBegun: boolean; holdsCloneGuid: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateFacts {
    /// The phase the record names.
    pub phase: Phase,
    /// The clone,
    /// when the phase records one.
    pub clone: Option<ClonePresence>,
    /// Whether the registration the record names still stands.
    pub registration_stands: bool,
    /// Whether a destruction plan stands for this clone.
    pub plan_stands: bool,
    /// Whether the slot holds an entry besides `.git` with nothing mounted there.
    pub checkout_begun: bool,
    /// Whether the record holds a `cloneGuid=`.
    pub holds_clone_guid: bool,
}

/// What:
///  The command a person or the wrapper runs for one state.
/// Why:
///  Each answer prints one line,
///  and the sentences are the design's own words where it states them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Command = { kind: 'add-again' } | { kind: 'settling' } | { kind: 'remove' } | { kind: 'removal' } | { kind: 'none' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NextCommand {
    /// The add again at the slot.
    TheAddAgain,
    /// The settling command,
    /// with both flags shown as placeholders.
    TheSettlingCommand,
    /// `git worktree remove <slot>`.
    GitWorktreeRemove,
    /// The removal the standing plan belongs to.
    TheRemoval,
    /// No command is needed,
    /// because the add continues.
    None,
}

/// The settling command's program and subcommand,
/// which the wrapper's own executable answers.
pub const SETTLING_COMMAND_WORDS: &str = "git monochromatic-git-policy-cli settle-worktree-clone-record";

/// What:
///  The command one state names.
/// Why:
///  This is the one statement every refusal and report cites,
///  so no caller invents its own order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const command = commandFor(facts);
/// ```
pub fn command_for(facts: &StateFacts) -> NextCommand {
    if facts.plan_stands {
        return NextCommand::TheRemoval;
    }
    if facts.phase == Phase::Settling {
        if !facts.holds_clone_guid {
            return NextCommand::TheAddAgain;
        }
        return match facts.clone.as_ref() {
            Some(ClonePresence::Stands) => NextCommand::GitWorktreeRemove,
            Some(ClonePresence::GoneProved) => match facts.registration_stands {
                true => NextCommand::TheAddAgain,
                false => NextCommand::TheSettlingCommand,
            },
            Some(ClonePresence::GoneUnprovable { .. }) => NextCommand::TheSettlingCommand,
            None => NextCommand::TheAddAgain,
        };
    }
    if facts.phase == Phase::Filled {
        return match facts.clone.as_ref() {
            Some(ClonePresence::Stands) | None => NextCommand::None,
            Some(ClonePresence::GoneProved | ClonePresence::GoneUnprovable { .. }) => NextCommand::TheSettlingCommand,
        };
    }
    if matches!(facts.phase, Phase::Cloned | Phase::Mounted) {
        return match facts.clone.as_ref() {
            Some(ClonePresence::Stands) => NextCommand::None,
            // A clone the phase records and the pool cannot answer for is the settling command's,
            // proved gone or not: the refusal names the same command either way.
            Some(ClonePresence::GoneProved | ClonePresence::GoneUnprovable { .. }) => NextCommand::TheSettlingCommand,
            None => NextCommand::TheAddAgain,
        };
    }
    return NextCommand::TheAddAgain;
}

impl NextCommand {
    /// The sentence a refusal or a report prints,
    /// with the slot filled in and the settling command's flags left as placeholders.
    pub fn printed_line(self, slot: &str) -> String {
        return match self {
            NextCommand::TheAddAgain => format!("run the add again at `{slot}`"),
            NextCommand::TheSettlingCommand => format!(
                "run `{SETTLING_COMMAND_WORDS} <slot> --confirm-removal-of <slot> --confirm-clone-guid <the last four decimal digits of cloneGuid=>` with `{slot}` in both slot places, after reading the evidence above"
            ),
            NextCommand::GitWorktreeRemove => format!("run `git worktree remove {slot}`, and `--force` if Git refuses it because the worktree holds files it did not index"),
            NextCommand::TheRemoval => format!("run the removal this slot's destruction plan belongs to; it resumes from the plan and ends it"),
            NextCommand::None => String::from("nothing: the add continues at the step its phase names"),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_state_commands_tests.rs"]
mod tests;
