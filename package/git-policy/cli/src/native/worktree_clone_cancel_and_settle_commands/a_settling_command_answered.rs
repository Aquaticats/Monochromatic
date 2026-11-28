//! What:
//!  What `git monochromatic-git-policy-cli settle-worktree-clone-record`
//!  answers through the management grammar:
//!  every way its words or its two confirming values are wrong,
//!  and one run of the command over the release's own reads.
//! Why:
//!  The command was built ahead of the grammar
//!  (`worktree_clone_record_settling::mod`),
//!  and this file is the management word that reaches it:
//!  the grammar parses the three values
//!  (`worktree_clone_record_settling::settling_request`),
//!  and the run is the command's own six reads over the live pool and this
//!  process's mount table,
//!  which is what its guest arm hands in.
//!  Its report goes on standard output,
//!  where a management command's report belongs and where a shell's pipeline
//!  reads it.
//!  The command has no `--help`:
//!  its grammar is one operand and two flags,
//!  so `--help` is a usage refusal that prints the line a person copies,
//!  and a help text of its own would be a second grammar to keep in step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = aSettlingCommandAnswered(parseSettlingArguments(rest), theRoute(word, ...));
//! ```

use crate::worktree_clone_cancel_and_settle_commands::the_answer_one_word_prints::AnAnswer;
use crate::worktree_clone_cancel_and_settle_commands::the_place_the_two_commands_run_in::TheRoute;
use crate::worktree_clone_cancel_and_settle_commands::the_pool_a_command_reads::ThePoolACommandReadsWhenAsked;
use crate::worktree_clone_cancel_and_settle_commands::the_status_a_settling_stop_ends_in::exit_status_of;
use crate::worktree_clone_record_settling::settling_command::{
    SettlingReads,
    settle_the_records_of_a_slot,
};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_request::SettlingRequest;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::path::PathBuf;

/// What:
///  The answer the `settle-worktree-clone-record` word prints.
/// Why:
///  The grammar hands in what it parsed and what it could learn about where the
///  command runs,
///  so a wrong word is answered before any record is read,
///  and a right one runs the command's own order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aSettlingCommandAnswered(parse_settling_arguments(rest), theRoute(word, ...));
/// ```
pub fn a_settling_command_answered(
    invocation: Result<SettlingRequest, SettlingStop>,
    route: Result<TheRoute, AnAnswer>,
) -> AnAnswer {
    let request: SettlingRequest = match invocation {
        Ok(request) => request,
        Err(stop) => return a_settling_stop(&stop),
    };
    let place: TheRoute = match route {
        Ok(place) => place,
        Err(answer) => return answer.line_terminated(),
    };
    let common_dir: PathBuf = place.common_dir.clone();
    let pool: ThePoolACommandReadsWhenAsked = ThePoolACommandReadsWhenAsked::not_opened();
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let reads: SettlingReads<'_> = SettlingReads {
        probe: &pool,
        mounts: &mounts,
    };
    let answer: AnAnswer =
        match settle_the_records_of_a_slot(common_dir.as_path(), &request, &reads) {
            Ok(done) => AnAnswer::on_standard_output(0, done.lines.join("\n")).line_terminated(),
            Err(stop) => a_settling_stop(&stop),
        };
    return answer;
}

/// What:
///  One stop of the settling command as the answer a person reads.
/// Why:
///  Every stop names what it read,
///  what it kept and the command that goes on,
///  and the status its kind owes,
///  so one function turns all of them into the one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { code: exitStatusOf(stop), stdout: '', stderr: stop.sentence() + '\n' };
/// ```
fn a_settling_stop(stop: &SettlingStop) -> AnAnswer {
    return AnAnswer::on_standard_error(exit_status_of(stop), stop.sentence()).line_terminated();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "a_settling_command_answered_tests.rs"]
mod tests;
