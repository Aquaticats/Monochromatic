//! What:
//!  One plan through the finish pass:
//!  the head's word,
//!  the slot's claim,
//!  the gate and the pool's own proof of absence,
//!  the registration,
//!  the binding,
//!  and the plan last.
//! Why:
//!  The order is the design's,
//!  and every step decides on a live read and never on a record's word
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration").
//!  A plan whose head is not `gone` is left without a word,
//!  because a kept plan stands for a clone's whole life
//!  and every forwarded command would otherwise repeat the same line.
//!  A live claim owner is left the same way:
//!  that command is finishing the removal.
//!  The plan is never written to,
//!  and the claim is released after the plan has gone,
//!  which is the list's step 14.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! finishOnePlan(pass, plan, planPath, lines);
//! ```

use crate::diagnostic_log::debug;
use crate::worktree_clone_binding_record::registration_home::slot_of;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_finish_pass::pass_done_lines::{the_plan_was_removed_after_a_reuse, the_plan_was_removed_beside_a_new_binding, the_removal_was_finished, the_settlement_of};
use crate::worktree_clone_removal_finish_pass::pass_refusal_lines::{
    the_binding_could_not_be_read, the_binding_disagrees, the_claim_could_not_be_taken, the_clone_stands, the_gate_refused, the_name_was_reused_with_a_registration, the_plan_could_not_be_removed,
    the_pool_could_not_prove, the_registration_could_not_be_read, the_registration_could_not_be_settled, the_slot_is_not_a_name,
};
use crate::worktree_clone_removal_finish_pass::the_pass_seams::FinishPass;
use crate::worktree_clone_removal_finish_pass::the_plans_binding::{TheBindingEnd, finish_the_plans_binding};
use crate::worktree_clone_removal_finish_pass::the_plans_file::remove_the_plan;
use crate::worktree_clone_removal_finish_pass::the_plans_registration::{RegistrationStanding, settle_the_plans_registration, the_registration_standing_for};
use crate::worktree_clone_removal_finish_pass::the_slots_claim::{TheClaim, give_the_claim_back, take_the_slots_claim};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{CloneAbsence, the_clone_absence};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::zfs_compatibility_gate::Verdict;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "removalFinishPass";

/// What:
///  One plan taken as far as its state allows,
///  with what it printed and what it left.
/// Why:
///  The claim is the slot's,
///  so it is taken here and released after the plan's own step,
///  and a live owner ends this plan's part with nothing touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finishOnePlan(pass, plan, planPath, lines);
/// ```
pub fn finish_one_plan(pass: &FinishPass<'_>, plan: &DestructionPlan, plan_path: &Path, lines: &mut Vec<String>) {
    if plan.head != HeadWord::Gone {
        debug(TAG, format!("the plan {} keeps its head as `{}`, so a removal or the settling command takes it", plan_path.display(), plan.head.word()).as_str());
        return;
    }
    let slot: String = match slot_of(plan.facts.target.as_path()) {
        Ok(found) => found,
        Err(said) => {
            lines.push(the_slot_is_not_a_name(plan_path, plan.facts.target.display().to_string().as_str(), said.as_str()));
            return;
        }
    };
    let mut claim = match take_the_slots_claim(pass.common_dir, slot.as_str()) {
        TheClaim::Held(held) => held,
        TheClaim::Busy(owner) => {
            debug(TAG, format!("the plan {} names the slot `{slot}`, whose claim {} holds with the process {}, so that removal finishes it", plan_path.display(), owner.path.display(), owner.pid).as_str());
            return;
        }
        TheClaim::Refused { said } => {
            lines.push(the_claim_could_not_be_taken(plan_path, said.as_str()));
            return;
        }
    };
    finish_under_the_claim(pass, plan, plan_path, slot.as_str(), lines);
    // The claim is the list's step 14 and is given back after the plan's own step; a release that
    // failed leaves a claim naming this process, which stops counting as alive when it ends.
    if let Err(said) = give_the_claim_back(&mut claim) {
        debug(TAG, format!("the claim for the slot `{slot}` could not be released ({said})").as_str());
    }
}

/// The gate,
/// the pool's proof of absence,
/// the registration,
/// the binding
/// and the plan,
/// under a claim this pass holds.
/// Why:
///  Each step reads the live state it decides on,
///  and a refusal leaves every record standing with one line for the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finishUnderTheClaim(pass, plan, planPath, slot, lines);
/// ```
fn finish_under_the_claim(pass: &FinishPass<'_>, plan: &DestructionPlan, plan_path: &Path, slot: &str, lines: &mut Vec<String>) {
    if let Verdict::Refused(refusal) = pass.gate.verdict() {
        lines.push(the_gate_refused(plan_path, format!("{refusal}").as_str()));
        return;
    }
    let reused: Option<DatasetGuid> = match the_clone_absence(pass.probe, plan.facts.clone_dataset.as_str(), plan.facts.clone_guid) {
        Ok(CloneAbsence::Gone) => None,
        Ok(CloneAbsence::There { name, guid }) => {
            lines.push(the_clone_stands(plan_path, name.as_str(), guid.file_name().as_str()));
            return;
        }
        Ok(CloneAbsence::Reused { guid }) => Some(guid),
        Err(stop) => {
            lines.push(the_pool_could_not_prove(plan_path, stop.message().as_str()));
            return;
        }
    };
    let standing: RegistrationStanding = the_registration_standing_for(pass.common_dir, plan);
    if let RegistrationStanding::Unreadable { path, said } = &standing {
        lines.push(the_registration_could_not_be_read(plan_path, path, said.as_str()));
        return;
    }
    if let (Some(live), RegistrationStanding::ThisPlans) = (&reused, &standing) {
        lines.push(the_name_was_reused_with_a_registration(plan_path, plan.facts.clone_dataset.as_str(), plan.facts.clone_guid.file_name().as_str(), live.file_name().as_str()));
        return;
    }
    let settlement: String = match &standing {
        RegistrationStanding::ThisPlans => match settle_the_plans_registration(pass.common_dir, slot, plan, pass.settle) {
            Ok(answer) => the_settlement_of(&answer),
            Err(stop) => {
                let (step, said) = the_settles_words(&stop);
                lines.push(the_registration_could_not_be_settled(plan_path, step.as_str(), said.as_str()));
                return;
            }
        },
        RegistrationStanding::Anothers => String::from("a registration of another identity stands at the plan's registration name, so nothing of this plan's was settled"),
        _ => String::from("no registration of this plan stood"),
    };
    let end: TheBindingEnd = finish_the_plans_binding(pass.common_dir, slot, plan);
    match &end {
        TheBindingEnd::Unreadable { path, said } => {
            lines.push(the_binding_could_not_be_read(plan_path, path, said.as_str()));
            return;
        }
        TheBindingEnd::Differs { path, field, binding, plan: plan_value } => {
            lines.push(the_binding_disagrees(plan_path, path, field, binding.as_str(), plan_value.as_str()));
            return;
        }
        _ => {}
    }
    if let Err(said) = remove_the_plan(pass.common_dir, plan) {
        lines.push(the_plan_could_not_be_removed(plan_path, said.as_str()));
        return;
    }
    match (&end, &reused) {
        (TheBindingEnd::AnotherClones, Some(live)) => lines.push(the_plan_was_removed_after_a_reuse(plan_path, slot, plan.facts.clone_dataset.as_str(), plan.facts.clone_guid.file_name().as_str(), live.file_name().as_str())),
        (TheBindingEnd::AnotherClones, None) => lines.push(the_plan_was_removed_beside_a_new_binding(plan_path, slot)),
        (TheBindingEnd::Removed { files }, _) | (TheBindingEnd::Missing { files }, _) => lines.push(the_removal_was_finished(plan_path, slot, files.as_slice(), settlement.as_str())),
        (TheBindingEnd::Unreadable { .. }, _) | (TheBindingEnd::Differs { .. }, _) => {}
    }
}

/// What:
///  The step and the words of a stop the settle answered.
/// Why:
///  A settle's refusal says which step refused and what it read,
///  and the pass's line prints both;
///  a refusal of another shape is named by its own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [step, said] = theSettlesWords(stop);
/// ```
fn the_settles_words(stop: &RemovalStop) -> (String, String) {
    return match stop {
        RemovalStop::TheSettleRefused { step, said } => (step.clone(), said.clone()),
        other => (String::from("the settle"), other.message()),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pass_on_one_plan_tests.rs"]
mod tests;
