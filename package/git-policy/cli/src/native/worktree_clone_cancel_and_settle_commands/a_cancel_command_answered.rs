//! What:
//!  What `git monochromatic-git-policy-cli cancel-worktree-removal <slot>`
//!  answers through the management grammar:
//!  its usage,
//!  every caller's mistake,
//!  and one run of the command over the release's own seams.
//! Why:
//!  The command was built ahead of the grammar
//!  (`worktree_clone_removal_cancel::mod`),
//!  and this file is the management word that reaches it:
//!  the grammar parses the words
//!  (`worktree_clone_removal_cancel::cancel_arguments`),
//!  and everything else here is the composition the command's own guest arm
//!  scripts,
//!  so the word runs the command rather than a copy of its decisions.
//!  The report goes on standard error,
//!  as the command's module doc says,
//!  because the round that also reaches it from the removal's command path
//!  prints it there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = aCancelCommandAnswered(parseCancelArguments(rest), theRoute(word, ...));
//! process.stdout.write(answer.stdout); process.stderr.write(answer.stderr); process.exit(answer.code);
//! ```

use crate::worktree_clone_cancel_and_settle_commands::the_answer_one_word_prints::AnAnswer;
use crate::worktree_clone_cancel_and_settle_commands::the_mount_request_a_cancellation_files::TheMountRequestFiledWhenAsked;
use crate::worktree_clone_cancel_and_settle_commands::the_place_the_two_commands_run_in::TheRoute;
use crate::worktree_clone_cancel_and_settle_commands::the_pool_a_command_reads::ThePoolACommandReadsWhenAsked;
use crate::worktree_clone_removal_cancel::cancel_arguments::{
    CANCEL_COMMAND_WORD,
    CancelInvocation,
    CancelMistake,
    CancelRequest,
    usage,
};
use crate::worktree_clone_removal_cancel::cancel_refusal::CALLERS_MISTAKE;
use crate::worktree_clone_removal_cancel::the_cancel_command::{
    TheCancelCommand,
    cancel_the_removal,
};
use crate::worktree_clone_removal_cancel::the_removal_claim::LiveTheRemovalClaim;
use crate::worktree_clone_removal_cancel::the_slot_mount::LiveTheSlotMount;
use crate::worktree_clone_removal_cancel::the_staged_state::LiveTheStagedFolder;
use std::path::PathBuf;

/// What:
///  The answer the `cancel-worktree-removal` word prints.
/// Why:
///  The grammar hands in what it parsed and what it could learn about where the
///  command runs,
///  so this function decides nothing about the words and everything about the
///  run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aCancelCommandAnswered(invocation, route);
/// ```
pub(crate) fn a_cancel_command_answered(
    invocation: Result<CancelInvocation, CancelMistake>,
    route: Result<TheRoute, AnAnswer>,
) -> AnAnswer {
    let asked: CancelInvocation = match invocation {
        Ok(asked) => asked,
        Err(mistake) => {
            return AnAnswer::on_standard_error(CALLERS_MISTAKE, mistake.message())
                .line_terminated();
        }
    };
    let request: CancelRequest = match asked {
        CancelInvocation::Help => {
            return AnAnswer::on_standard_output(0, usage()).line_terminated();
        }
        CancelInvocation::Cancel(request) => request,
    };
    let place: TheRoute = match route {
        Ok(place) => place,
        Err(answer) => return answer.line_terminated(),
    };
    let home: &std::path::Path = match place.the_home(CANCEL_COMMAND_WORD) {
        Ok(home) => home,
        Err(answer) => return answer.line_terminated(),
    };
    let checkout: PathBuf = place.checkout.clone();
    let common_dir: PathBuf = place.common_dir.clone();
    let pool: ThePoolACommandReadsWhenAsked = ThePoolACommandReadsWhenAsked::not_opened();
    let claim: LiveTheRemovalClaim<'_> = LiveTheRemovalClaim {
        common_dir: common_dir.as_path(),
    };
    let staged: LiveTheStagedFolder<'_> = LiveTheStagedFolder {
        home,
        common_dir: common_dir.as_path(),
    };
    let mount_request: TheMountRequestFiledWhenAsked<'_> =
        TheMountRequestFiledWhenAsked::for_the_slot(common_dir.as_path(), request.slot.as_str());
    let command: TheCancelCommand<'_> = TheCancelCommand {
        checkout: checkout.as_path(),
        common_dir: common_dir.as_path(),
        home,
        request: &request,
        reads: &pool,
        absence: &pool,
        mounts: &LiveTheSlotMount,
        claim: &claim,
        mount_request: &mount_request,
        staged: &staged,
    };
    let answer: AnAnswer = match cancel_the_removal(&command) {
        Ok(outcome) => {
            let report = outcome.report();
            AnAnswer::on_standard_error(report.exit_code(), report.lines().join("\n"))
                .line_terminated()
        }
        Err(refusal) => {
            AnAnswer::on_standard_error(refusal.exit_code(), refusal.message()).line_terminated()
        }
    };
    return answer;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "a_cancel_command_answered_tests.rs"]
mod tests;
