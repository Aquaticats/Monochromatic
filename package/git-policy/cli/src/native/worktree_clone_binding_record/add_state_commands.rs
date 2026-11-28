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
//!  (the design writer's K14,
//!  added after the eleventh read).
//!  A binding at `cloned` or `mounted` whose clone is gone is the settling command's,
//!  proved or not.
//!  Beside a standing plan the command is the plan design's,
//!  read from the plan's head word and the clone the pool can find:
//!  a plan at `present` or `refused` whose clone does not answer names the
//!  settling command,
//!  never a removal that refuses it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const command = commandForState({ phase: 'mounted', clone: 'gone-unprovable', planHead: 'present' });
//! // { kind: 'settling' }
//! ```

use super::binding_words::Phase;
use super::registration_home::{
    BINDING_SUFFIX,
    GRAMMAR_VERSION_FOLDER,
    WORKTREE_CLONES_FOLDER,
};
use crate::foundation::wrapper_names_old_and_new::DATA_FOLDER;

/// What:
///  Whether the recorded clone still stands,
///  is gone and proved gone,
///  or is gone and the pool could not prove it,
///  or has a name that now answers another clone.
/// Why:
///  The third arm is the one a refused or partial listing leaves,
///  and it names the same command as the second.
///  The fourth is the state a plan cannot settle at all:
///  the name the plan records answers with another GUID,
///  so the clone the plan recorded has no name to be proved absent by,
///  and no command of this release acts on it
///  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
///  "What 2C costs").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClonePresence = { kind: 'stands' } | { kind: 'gone-proved' } | { kind: 'gone-unprovable'; evidence: string } | { kind: 'name-answers-another'; liveGuid: string };
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
    /// The recorded name answers with another clone's GUID,
    /// so the name was reused and the recorded clone's absence rests on the
    /// GUID alone.
    TheNameAnswersAnotherClone {
        /// The GUID the recorded name answered with,
        /// in the canonical decimal form a record writes.
        live_guid: String,
    },
}

/// What:
///  The word a standing destruction plan records for its own head destroy,
///  which is the plan design's set of five.
/// Why:
///  The command a plan's presence allows is not one word for every plan:
///  a head word of `present` or `refused` beside a clone the pool cannot find
///  is the state the removal refuses and the settling command takes,
///  while every other head word leaves the removal the command
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "Which resume path a removal takes").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanHeadWord = 'present' | 'requested' | 'gone' | 'refused' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanHeadWord {
    /// The head destroy was never requested and the clone stands.
    Present,
    /// The head destroy was written and may have happened.
    Requested,
    /// The head destroy answered,
    /// so the clone is gone.
    Gone,
    /// The pool refused the head destroy,
    /// so the clone stands.
    Refused,
    /// The head destroy's outcome cannot be attributed.
    Unknown,
}

impl PlanHeadWord {
    /// The word the plan's record spells,
    /// which a control names beside the command it expects.
    pub fn word(self) -> &'static str {
        return match self {
            PlanHeadWord::Present => "present",
            PlanHeadWord::Requested => "requested",
            PlanHeadWord::Gone => "gone",
            PlanHeadWord::Refused => "refused",
            PlanHeadWord::Unknown => "unknown",
        };
    }

    /// The head word one stored word names,
    /// or nothing when the word is outside the plan design's five.
    pub fn parse(word: &str) -> Option<PlanHeadWord> {
        return match word {
            "present" => Some(PlanHeadWord::Present),
            "requested" => Some(PlanHeadWord::Requested),
            "gone" => Some(PlanHeadWord::Gone),
            "refused" => Some(PlanHeadWord::Refused),
            "unknown" => Some(PlanHeadWord::Unknown),
            _ => None,
        };
    }
}

/// What:
///  The live facts a reader has before it names a command.
/// Why:
///  The answers come from the record's phase,
///  the plan folder and the live reads,
///  and the command follows from them alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateFacts = { phase: Phase; clone?: ClonePresence; registrationStands: boolean; plan?: PlanHeadWord; checkoutBegun: boolean; holdsCloneGuid: boolean };
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
    /// The head word of a destruction plan that stands for this clone,
    /// and nothing when no plan stands.
    pub plan_head: Option<PlanHeadWord>,
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
    /// No command acts on the state:
    /// the plan's head word is `gone` and the pool cannot prove the clone's absence,
    /// so the finish pass removes the plan once the pool answers.
    TheFinishPass,
    /// No command of this release acts on the state:
    /// the plan's recorded name answers another clone's GUID,
    /// so a person reads the two records before acting.
    APersonReadsTheRecords,
}

/// The settling command's program and subcommand,
/// which no management grammar admits in this release
/// (`SPEC.md`,
/// "The settling command,
/// built and reachable from no command").
pub const SETTLING_COMMAND_WORDS: &str =
    "git monochromatic-git-policy-cli settle-worktree-clone-record";

/// The words every refusal and report prints where a removal would be the way
/// out.
/// Why:
///  No command of this release removes a clone,
///  so naming one as the way out would send a person to a command that does not
///  exist;
///  this file is where the project states the one command a state names,
///  and it says here that this one is not built
///  (`SPEC.md`,
///  "Removal of a ZFS worktree clone").
pub const NO_REMOVAL_WORDS: &str =
    "this release has no removal for a clone, so the clone and its record stay until it does";

/// The words that say a command the project designed for a state cannot be run
/// in this release.
/// Why:
///  Two commands a state names are built ahead of the grammar that would reach
///  them,
///  so a sentence that tells a person to run one sends them to a command that
///  answers with the usage text and exit 2
///  (`SPEC.md`,
///  "The settling command,
/// built and reachable from no command",
///  and "The cancel command,
/// built and reachable from no command").
///  The round that adds either to the grammar deletes this claim from one place,
///  and every sentence that prints it follows.
pub const NOT_ADMITTED_YET: &str = "no management grammar admits it in this release";

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
    if let Some(head) = facts.plan_head {
        return command_beside_a_standing_plan(facts, head);
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
            Some(
                ClonePresence::GoneUnprovable {
                    ..
                }
                | ClonePresence::TheNameAnswersAnotherClone {
                    ..
                },
            ) => NextCommand::TheSettlingCommand,
            None => NextCommand::TheAddAgain,
        };
    }
    if facts.phase == Phase::Filled {
        return match facts.clone.as_ref() {
            Some(ClonePresence::Stands) | None => NextCommand::None,
            Some(
                ClonePresence::GoneProved
                | ClonePresence::GoneUnprovable {
                    ..
                }
                | ClonePresence::TheNameAnswersAnotherClone {
                    ..
                },
            ) => NextCommand::TheSettlingCommand,
        };
    }
    if matches!(facts.phase, Phase::Cloned | Phase::Mounted) {
        return match facts.clone.as_ref() {
            Some(ClonePresence::Stands) => NextCommand::None,
            // A clone the phase records and the pool cannot answer for is the settling command's,
            // proved gone or not: the refusal names the same command either way.
            Some(
                ClonePresence::GoneProved
                | ClonePresence::GoneUnprovable {
                    ..
                }
                | ClonePresence::TheNameAnswersAnotherClone {
                    ..
                },
            ) => NextCommand::TheSettlingCommand,
            None => NextCommand::TheAddAgain,
        };
    }
    return NextCommand::TheAddAgain;
}

/// What:
///  The command a state with a standing plan allows,
///  by the plan's head word and the clone the pool can find.
/// Why:
///  The plan design decides the command from two facts:
///  the plan's head word,
///  and whether the pinned parent carries the recorded clone.
///  A clone that answers takes the head-present path,
///  whatever the word says,
///  and that path is the removal's.
///  A proved absence takes the head-gone path when the head is `requested`,
///  `unknown` or `gone`,
///  and is the outside destroy the removal refuses,
///  which names the settling command,
///  when the head is `present` or `refused`.
///  An absence the pool cannot prove names the settling command for every head
///  word but `gone`,
///  whose route is the finish pass.
///  A recorded name that answers another clone's GUID is the one state no
///  command settles:
///  the removal refuses it and so does the settling command,
///  and the finish pass leaves it standing,
///  so a person reads the two records instead.
///  A reader that did not look for the clone,
///  which is the add's smallest plan read,
///  decides by the head word alone and names the removal for every word but
///  `present` and `refused`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const command = commandBesideAStandingPlan(facts, head);
/// ```
fn command_beside_a_standing_plan(
    facts: &StateFacts,
    head: PlanHeadWord,
) -> NextCommand {
    if matches!(facts.clone.as_ref(), Some(ClonePresence::Stands)) {
        return NextCommand::TheRemoval;
    }
    if matches!(
        facts.clone.as_ref(),
        Some(ClonePresence::TheNameAnswersAnotherClone { .. })
    ) {
        return NextCommand::APersonReadsTheRecords;
    }
    if matches!(
        facts.clone.as_ref(),
        Some(ClonePresence::GoneUnprovable { .. })
    ) {
        return match head {
            PlanHeadWord::Gone => NextCommand::TheFinishPass,
            PlanHeadWord::Present
            | PlanHeadWord::Requested
            | PlanHeadWord::Refused
            | PlanHeadWord::Unknown => NextCommand::TheSettlingCommand,
        };
    }
    return match head {
        PlanHeadWord::Present | PlanHeadWord::Refused => NextCommand::TheSettlingCommand,
        PlanHeadWord::Requested | PlanHeadWord::Gone | PlanHeadWord::Unknown => {
            NextCommand::TheRemoval
        }
    };
}

impl NextCommand {
    /// The sentence a refusal or a report prints,
    /// with the slot filled in and the settling command's flags left as placeholders.
    pub fn printed_line(
        self,
        slot: &str,
    ) -> String {
        return match self {
            NextCommand::TheAddAgain => format!("run the add again at `{slot}`"),
            NextCommand::TheSettlingCommand => format!(
                "the command that settles this state is `{SETTLING_COMMAND_WORDS} <slot> --confirm-removal-of <slot> --confirm-clone-guid <the last four decimal digits of cloneGuid=>` with `{slot}` in both slot places, and it is built while {NOT_ADMITTED_YET}, so it cannot be run yet: read the evidence above, and leave the record and its slot as they stand until a release that answers it ships"
            ),
            NextCommand::GitWorktreeRemove => format!(
                "run `git worktree remove {slot}`, and `--force` if Git refuses it because the worktree holds files it did not index"
            ),
            NextCommand::TheRemoval => format!(
                "the removal this slot's destruction plan belongs to resumes from the plan and ends it, and {NO_REMOVAL_WORDS}"
            ),
            NextCommand::TheFinishPass => String::from(
                "wait: the pool cannot prove the clone's absence and the plan's head is `gone`, so the settling command refuses it and the finish pass removes the plan once the pool answers",
            ),
            NextCommand::None => {
                String::from("nothing: the add continues at the step its phase names")
            }
            NextCommand::APersonReadsTheRecords => format!(
                "read the two records before acting: the plan's recorded clone name now answers another clone, so no command of this release settles the state, and `git monochromatic-git-policy-cli doctor` reports neither the plan nor the registration nor the slot yet, so read the plan's own file and the binding record `{slot}{binding}` in `{wrapper}/{clones}/{grammar}` under the Git common directory",
                binding = BINDING_SUFFIX,
                wrapper = DATA_FOLDER.current,
                clones = WORKTREE_CLONES_FOLDER,
                grammar = GRAMMAR_VERSION_FOLDER
            ),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_state_commands_tests.rs"]
mod tests;
