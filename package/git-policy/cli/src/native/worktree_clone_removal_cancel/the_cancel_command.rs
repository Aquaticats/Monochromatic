//! What:
//!  The guarded `cancel-worktree-removal <slot>` command:
//!  the records' proof,
//!  the removal claim,
//!  the deciding proof under it,
//!  the already-cancelled state,
//!  one reversing request,
//!  and the report a person reads.
//! Why:
//!  The project's answer keeps the command as the manual way back for a removal whose wrapper was
//!  killed after it recorded the unmount or that ran past its deadline,
//!  and requires it to report every snapshot already destroyed
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`).
//!  The design's proof runs before the act,
//!  every deciding read is repeated under the claim,
//!  and the act is one `mount` request and one answer
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The proof" and "The act" sections).
//!  The command never forces an unmount,
//!  never detaches a mount,
//!  never ends a holder,
//!  never mounts by path,
//!  never removes a registration and never destroys a snapshot;
//!  it asks for one attach,
//!  settles the removal's own records,
//!  and reports what cannot be restored.
//!  This release's management grammar admits the command,
//!  and the lines this function returns are printed on standard error,
//!  because the repository's stream rule keeps an empty standard output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = cancelTheRemoval(input);
//! for (const line of outcome.report().lines()) console.error(line);
//! process.exit(0);
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_cancel::cancel_arguments::CancelRequest;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_cancelling_sequence::{
    TheReversalDone,
    TheReversalRequest,
    reverse_the_removal,
};
use crate::worktree_clone_removal_cancel::the_plan_it_leaves::{
    ThePlansFate,
    the_plan_a_cancellation_leaves,
};
use crate::worktree_clone_removal_cancel::the_proof::{
    TheRecordsProof,
    WhatTheRecordsProve,
    prove_the_slot_is_a_removals,
    prove_the_slots_records,
};
use crate::worktree_clone_removal_cancel::the_removal_claim::{
    HeldClaim,
    TheRemovalClaim,
};
use crate::worktree_clone_removal_cancel::the_reversing_request::TheMountRequest;
use crate::worktree_clone_removal_cancel::the_slot_mount::{
    TheSlotMount,
    counts_as_at_its_slot,
    word as slot_word,
};
use crate::worktree_clone_removal_cancel::the_staged_state::{
    StagedSettlement,
    TheStagedFolder,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use std::path::Path;

/// The exit code of a reversal,
/// and of a removal that had already been cancelled.
pub const REVERSED: i32 = 0;

/// What:
///  Everything one run of the command drives.
/// Why:
///  The production answers and the controls are the same seams the rest of the route uses,
///  so a control proves the command's decisions without a pool,
///  a mount table or a claim file,
///  and the release wires the device,
///  this process's table
///  and the repository's claim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Command = { checkout; commonDir; home; request; reads; absence; mounts; claim; mountRequest; staged };
/// ```
pub struct TheCancelCommand<'a> {
    /// The repository's pinned parent.
    pub checkout: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The account's home.
    pub home: &'a Path,
    /// The slot the grammar read.
    pub request: &'a CancelRequest,
    /// The pool reads the proof makes.
    pub reads: &'a dyn RemovalPoolReads,
    /// The reader the clone's absence is proved through.
    pub absence: &'a dyn DatasetReader,
    /// Where the clone stands.
    pub mounts: &'a dyn TheSlotMount,
    /// The per-clone removal claim.
    pub claim: &'a dyn TheRemovalClaim,
    /// The route's own request writer and wait.
    pub mount_request: &'a dyn TheMountRequest,
    /// The staging folder's settlement.
    pub staged: &'a dyn TheStagedFolder,
}

/// What:
///  What one run leaves,
///  as the report names it.
/// Why:
///  A run that reversed a removal and a run that found it already cancelled print different
///  first lines and the same facts,
///  so one report carries both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Report = { alreadyCancelled: boolean; slot; dataset; guid; destroyed; mountState; staged; plans; nonce?; mountId? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheCancelReport {
    /// Whether the removal had already been cancelled,
    /// so no request was written.
    pub already_cancelled: bool,
    /// The slot the command proved.
    pub slot: String,
    /// The clone's dataset name.
    pub dataset: String,
    /// The clone's recorded GUID.
    pub guid: String,
    /// Every snapshot the removal destroyed.
    pub destroyed: Vec<String>,
    /// Where the clone stands after the run,
    /// in words.
    pub mount_state: String,
    /// What became of the staged state.
    pub staged: StagedSettlement,
    /// What became of the plan.
    pub plans: ThePlansFate,
    /// The nonce the reversing request was filed under,
    /// where one was written.
    pub nonce: Option<String>,
    /// The mount the answer observed,
    /// where one was asked for.
    pub mount_id: Option<u64>,
}

impl TheCancelReport {
    /// What:
    ///  The report a person reads on standard error.
    /// Why:
    ///  Each line is a fact the design asks the report to name:
    ///  the slot,
    ///  the dataset and GUID,
    ///  the destroyed set and that it cannot be restored,
    ///  the mount state,
    ///  any retained staged state,
    ///  and the next step.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// console.error(report.lines().join('\n'));
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::with_capacity(8);
        if self.already_cancelled {
            lines.push(format!(
                "the removal of `{slot}` was already cancelled: `{dataset}` (GUID `{guid}`) stands where its removal left it and its own request asks for the mount.",
                slot = self.slot,
                dataset = self.dataset,
                guid = self.guid
            ));
        } else {
            lines.push(format!(
                "cancelled the removal of `{slot}` (`{dataset}`, GUID `{guid}`).",
                slot = self.slot,
                dataset = self.dataset,
                guid = self.guid
            ));
        }
        if self.destroyed.is_empty() {
            lines.push(String::from("No snapshot was destroyed."));
        } else {
            let mut names: Vec<String> = Vec::with_capacity(self.destroyed.len());
            for name in &self.destroyed {
                names.push(format!("- `{name}`"));
            }
            lines.push(format!(
                "The removal destroyed {} snapshot(s), which do not come back:\n{}",
                self.destroyed.len(),
                names.join("\n")
            ));
        }
        match &self.plans {
            ThePlansFate::NoPlan => lines.push(String::from("No destruction plan stood for this clone.")),
            ThePlansFate::Removed { path } => lines.push(format!(
                "The destruction plan at {} witnessed nothing and left, so the removal's records are gone.",
                path.display()
            )),
            ThePlansFate::Kept { path, .. } => lines.push(format!(
                "The destruction plan at {} keeps the removal's own outcomes, and lives until this clone's next removal reads it, carries the loss forward and ends it.",
                path.display()
            )),
        }
        lines.push(format!("The clone is {}.", self.mount_state));
        match &self.staged {
            StagedSettlement::NoStagedFolder => lines.push(String::from("No staging folder, record or claim stood for this clone.")),
            StagedSettlement::Removed { record, claim } => lines.push(format!(
                "The removal's staging folder is gone, and its record {} and its claim {}.",
                if *record { "left with it" } else { "was not there" },
                if *claim { "was released" } else { "was not there" }
            )),
            StagedSettlement::ReleasedNoFolder => lines.push(String::from("The staging claim named a folder that was never made, and both left.")),
            StagedSettlement::Retained { step, said } => lines.push(format!(
                "The staging folder, its record and its claim are kept and reported ({step}: {said})."
            )),
        }
        match &self.nonce {
            Some(nonce) => lines.push(format!(
                "The clone's request asks for the mount under nonce `{nonce}`, and the handler observed mount {}.",
                self.mount_id.map_or_else(|| return String::from("none"), |mount| return mount.to_string())
            )),
            None => lines.push(String::from("No new request was written, because the clone's own request already asks for the mount.")),
        }
        lines.push(String::from(
            "`git monochromatic-git-policy-cli doctor` reads the wrapper's records; its ZFS removal lines are plan step 18's, and this release does not print them yet.",
        ));
        return lines;
    }

    /// The status the command exits with.
    pub fn exit_code(&self) -> i32 {
        return REVERSED;
    }
}

/// What:
///  What one run of the command answers.
/// Why:
///  The two arms differ in whether a request was written,
///  and both are a successful run,
///  so one value carries the report and the status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'reversed' | 'already-cancelled'; report };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheCancellation {
    /// A request was written and answered.
    Reversed {
        /// The report.
        report: TheCancelReport,
    },
    /// The removal had already been cancelled,
    /// so no request was written.
    AlreadyCancelled {
        /// The report.
        report: TheCancelReport,
    },
}

impl TheCancellation {
    /// The report either arm prints.
    pub fn report(&self) -> &TheCancelReport {
        return match self {
            TheCancellation::Reversed {
                report,
            } => report,
            TheCancellation::AlreadyCancelled {
                report,
            } => report,
        };
    }
}

/// What:
///  Reverse one removal,
///  or report that it was reversed already.
/// Why:
///  The proof runs,
///  the claim is taken,
///  every deciding read is repeated under it,
///  the already-cancelled state is reported without a second request,
///  and the act is one request whose answer is verified.
///  The claim is released on every path out,
///  including the paths that refuse.
///
/// # Errors
/// Every state the proof refuses,
/// a live removal,
/// a handler answer that refuses,
/// a mount that is not the answered one,
/// a retained staged state,
/// or a plan that could not be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = cancelTheRemoval(input);
/// ```
pub fn cancel_the_removal(input: &TheCancelCommand<'_>) -> Result<TheCancellation, CancelRefusal> {
    let proof = TheRecordsProof {
        checkout: input.checkout,
        common_dir: input.common_dir,
        home: input.home,
        slot: input.request.slot.as_str(),
        reads: input.reads,
        absence: input.absence,
    };
    // The records are proved before the claim is taken, so a caller's mistake leaves it untouched.
    let claimed: DatasetGuid = prove_the_slot_is_a_removals(&proof)?;
    // The held claim gives the slot back on every path out, the refusals included, when it is dropped.
    let _held: HeldClaim = input.claim.take(input.request.slot.as_str())?;
    return act_under_the_claim(input, &proof, claimed);
}

/// Everything after the claim is held,
/// which is where every deciding read happens again.
fn act_under_the_claim(
    input: &TheCancelCommand<'_>,
    proof: &TheRecordsProof<'_>,
    claimed: DatasetGuid,
) -> Result<TheCancellation, CancelRefusal> {
    let proved: WhatTheRecordsProve = prove_the_slots_records(proof)?;
    // The claim was taken by the GUID the first proof read, so a record rewritten between the two reads is a
    // disagreement rather than a second removal's act under the first one's claim.
    if proved.recorded.clone_guid != claimed {
        return Err(CancelRefusal::TheRecordsDisagree {
            what: String::from(
                "the binding was rewritten between the records' proof and the one under the claim",
            ),
            first: claimed.file_name(),
            second: proved.recorded.clone_guid.file_name(),
        });
    }
    let recorded: &RecordedBinding = &proved.recorded;
    let where_it_stands = input.mounts.where_the_clone_stands(
        proved.recorded.clone_dataset.as_str(),
        proved.slot_path.as_path(),
    );
    let already: bool =
        proved.intent.operation == MountOperation::Mount && counts_as_at_its_slot(&where_it_stands);
    if already {
        let staged: StagedSettlement = input.staged.settle(proved.recorded.clone_guid);
        if let StagedSettlement::Retained {
            step,
            said,
        } = &staged
        {
            return Err(CancelRefusal::TheStagedStateIsKept {
                step: step.clone(),
                said: said.clone(),
            });
        }
        let plans: ThePlansFate = the_plan_a_cancellation_leaves(
            input.common_dir,
            recorded,
            proved.plan.as_ref(),
            proved.destroyed.as_slice(),
        )?;
        return Ok(TheCancellation::AlreadyCancelled {
            report: TheCancelReport {
                already_cancelled: true,
                slot: input.request.slot.clone(),
                dataset: proved.recorded.clone_dataset.clone(),
                guid: proved.recorded.clone_guid.file_name(),
                destroyed: proved.destroyed.clone(),
                mount_state: slot_word(&where_it_stands),
                staged,
                plans,
                nonce: None,
                mount_id: None,
            },
        });
    }
    let reversal: TheReversalDone = reverse_the_removal(&TheReversalRequest {
        common_dir: input.common_dir,
        recorded,
        slot_path: proved.slot_path.as_path(),
        plan: proved.plan.as_ref(),
        destroyed: proved.destroyed.as_slice(),
        mount_request: input.mount_request,
        staged: input.staged,
    })?;
    if let StagedSettlement::Retained {
        step,
        said,
    } = &reversal.staged
    {
        return Err(CancelRefusal::TheStagedStateIsKept {
            step: step.clone(),
            said: said.clone(),
        });
    }
    let after: CloneMount = input.mounts.where_the_clone_stands(
        proved.recorded.clone_dataset.as_str(),
        proved.slot_path.as_path(),
    );
    if !counts_as_at_its_slot(&after) {
        return Err(CancelRefusal::TheMountStateIsNotTheAnswers {
            said: format!(
                "the handler answered `done` and this process's mount table says the clone is {}.",
                slot_word(&after)
            ),
        });
    }
    return Ok(TheCancellation::Reversed {
        report: TheCancelReport {
            already_cancelled: false,
            slot: input.request.slot.clone(),
            dataset: proved.recorded.clone_dataset.clone(),
            guid: proved.recorded.clone_guid.file_name(),
            destroyed: reversal.destroyed.clone(),
            mount_state: slot_word(&after),
            staged: reversal.staged,
            plans: reversal.plans,
            nonce: Some(reversal.nonce),
            mount_id: Some(reversal.mount_id),
        },
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_cancel_command_tests.rs"]
mod tests;
