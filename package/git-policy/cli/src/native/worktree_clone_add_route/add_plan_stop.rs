//! What:
//!  The stop one standing removal plan leaves the add,
//!  with the one command that ends the plan written out.
//! Why:
//!  A person who meets a standing plan must read the command that ends it,
//!  not one sentence for every state
//!  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
//!  choice 2C;
//!  `doc/planning/cli-git-zfs-add-binding-record-design.md`, `V24`).
//!  Two facts decide that command:
//!  the plan's head word,
//!  and whether a child of the clone container still carries the clone the
//!  plan records.
//!  So the stop is built here, after the add has opened its device,
//!  and both of the add's plan reads call this one function,
//!  because the same state must not print two different sentences.
//!  The command's own words are never repeated here:
//!  the command list gives them (`command_for`),
//!  the cancel command gives its word
//!  (`worktree_clone_removal_cancel::cancel_arguments::CANCEL_COMMAND_WORD`),
//!  and the plan's sentence only says what the plan and the pool answered.
//!  A plan this reader cannot read keeps the add's refusal,
//!  and that refusal still names no command,
//!  because nothing about such a plan is proved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stop = refuseWhileAPlanStands(route, probe);
//! ```

use super::add_plan_clone_presence::the_presence_of_the_plans_clone;
use super::add_route_gate::RouteFacts;
use super::add_route_steps::StepRefusal;
use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;
use crate::worktree_clone_binding_record::add_state_commands::{ClonePresence, NextCommand, PlanHeadWord, StateFacts, command_for};
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::plan_folder_read::{APlanThatNamesTheClone, the_plan_naming_the_clone};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_removal_cancel::cancel_arguments::CANCEL_COMMAND_WORD;

/// What:
///  Refuse while a destruction plan in the folder names this add's clone,
///  and while a file there cannot be read,
///  with the command the plan's head word and the pool leave the person.
/// Why:
///  This is the add's one call for a standing plan,
///  made before its first mutation and for nothing else:
///  it acts on no plan and removes none,
///  and the sentence it prints is the command list's own line for the state the
///  plan and the pool answer
///  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
///  choice 2C).
///  The read stands after the device opens because the presence is a pool
///  answer, so a plan that stops the add no longer stops it before the device
///  opens.
///
/// # Errors
/// The plan's stop sentence,
/// which names the plan,
/// what the plan and the pool answered,
/// and the one command that ends the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stop = refuseWhileAPlanStands(route, probe); // void when no plan names the clone
/// ```
pub fn refuse_while_a_plan_stands(route: &RouteFacts, probe: &dyn DatasetReader) -> Result<(), StepRefusal> {
    let found: APlanThatNamesTheClone = match the_plan_naming_the_clone(route.common_dir.as_path(), None, route.clone_dataset().as_str()) {
        Ok(None) => return Ok(()),
        Ok(Some(found)) => found,
        Err(unreadable) => return Err(StepRefusal::refused(unreadable.sentence)),
    };
    let presence: ClonePresence = the_presence_of_the_plans_clone(probe, &found);
    return Err(StepRefusal::refused(the_sentence_beside_a_plan(&found, &presence, route.slot.as_str())));
}

/// What:
///  The sentence a person reads beside one standing plan:
///  the plan,
///  what its record and the pool answer,
///  the command for that pair of facts,
///  and the cancel command where the plan witnesses nothing destroyed.
/// Why:
///  The plan design decides the command from those two facts,
///  and the cancel command is the way out for a person who wants the worktree
///  kept,
///  so a plan whose tuples are all `planned` and whose head still stands must
///  name it beside the removal
///  (`doc/planning/cli-git-zfs-add-binding-record-design.md`, `V24`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = theSentenceBesideAPlan(plan, presence, 'topic');
/// ```
pub fn the_sentence_beside_a_plan(plan: &APlanThatNamesTheClone, presence: &ClonePresence, slot: &str) -> String {
    // What:     `StateFacts { .. }` is the record of the facts the command list
    //           reads. Beside a plan, only `plan_head` and `clone` are read,
    //           so the other fields hold the add's state before its first
    //           mutation and no record's.
    // Why:      One function names the command for every state, and the add's
    //           stop must not carry a second copy of that list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const facts = { phase: 'claimed', clone: presence, registrationStands: false, planHead: plan.head, checkoutBegun: false, holdsCloneGuid: true };
    // ```
    let facts: StateFacts = StateFacts {
        phase: Phase::Claimed,
        clone: Some(presence.clone()),
        registration_stands: false,
        plan_head: Some(plan.head),
        checkout_begun: false,
        holds_clone_guid: true,
    };
    let command: NextCommand = command_for(&facts);
    let mut sentence: String = format!(
        "a removal's plan at {} names the clone this add would fill, so the add stops before its first mutation and changes nothing. The plan records its head as `{}` for the clone {}, and {}. Ways forward: {}.",
        plan.plan.display(),
        plan.head.word(),
        plan.clone_guid.file_name(),
        what_the_pool_answered(presence),
        command.printed_line(slot)
    );
    if names_the_cancel_command(plan, presence) {
        sentence.push_str(format!(" A person who wants the worktree kept runs `git {word} {cancel} {slot}`, because the plan records no destroy.", word = COMMAND_WORD.current, cancel = CANCEL_COMMAND_WORD).as_str());
    }
    return sentence;
}

/// What:
///  Whether the plan leaves the cancel command as a way out:
///  its every attempt line still says `planned`,
///  its head word is one that left the clone standing,
///  and the pool's answer leaves a command for the state at all.
/// Why:
///  The plan design names the cancel command where the plan witnesses nothing
///  destroyed,
///  and the cancel command's own rule refuses a head word of `gone` or an
///  outcome it cannot attribute,
///  so naming it beside those plans would send a person to a refusal.
///  A recorded name another clone now answers is the same mistake in another
///  form:
///  the worktree that clone would be kept for is not the dataset the name holds,
///  so no clause says the worktree can be kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = plan.everyTuplePlanned && (plan.head === 'present' || plan.head === 'refused') && presence.kind !== 'name-answers-another';
/// ```
fn names_the_cancel_command(plan: &APlanThatNamesTheClone, presence: &ClonePresence) -> bool {
    if matches!(presence, ClonePresence::TheNameAnswersAnotherClone { .. }) {
        return false;
    }
    return plan.every_tuple_planned && matches!(plan.head, PlanHeadWord::Present | PlanHeadWord::Refused);
}

/// What:
///  The pool's answer about the plan's clone, in one clause of a sentence.
/// Why:
///  The command that follows depends on this answer,
///  so the person must read which answer the add was given,
///  including the pool's own words when it could not prove the clone gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clause = whatThePoolAnswered(presence);
/// ```
fn what_the_pool_answered(presence: &ClonePresence) -> String {
    return match presence {
        ClonePresence::Stands => String::from("a child of the clone container carries that GUID, so the clone stands"),
        ClonePresence::GoneProved => String::from("no child of the clone container carries that GUID and the name is free, so the clone is proved gone"),
        ClonePresence::GoneUnprovable { evidence } => format!("the pool cannot prove that clone gone ({evidence})"),
        ClonePresence::TheNameAnswersAnotherClone { live_guid } => format!("the name it records answers another clone, GUID {live_guid}, so nothing proves the clone it recorded gone"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_plan_stop_tests.rs"]
mod tests;
