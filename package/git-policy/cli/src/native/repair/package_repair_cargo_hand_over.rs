//! What:
//!  The hand-over of the filling of a new worktree's build directories to the commit service,
//! and the one line the add answers for it.
//! Why:
//!  `package_repair_cargo.rs` decides what each workspace's build state needs and stays under the
//!  file-size limit;
//! the hand-over reads only the entries that file produced,
//! so it lives in a file of its own beside the repair.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handedOver = async (request, entries) => { const told = await handOverWithinTheBudget(request.git, await commonDirectory(request.destination), request.destination, entries); return repaired(told); };
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::package_repair_prewarm::{PrewarmEntry, hand_over_in_a_job};
use crate::package_repair_report::{Finding, Outcome, Tool};
use crate::package_repair_request::RepairRequest;
use crate::service_work::work_directory;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What:
///  The directory tree in which a person watches what became of work the add handed over:
/// the queue's own states,
/// with the service's journal beside them.
/// Why:
///   The line the add prints for that work names a place,
/// and a line whose place cannot be named would be worse than one that says less.
fn queues_states(common: &Path) -> String {
    return work_directory(common).display().to_string();
}

/// What:
///  One handed-over workspace's contribution to the count of ones whose own copied build state the
/// add moved aside.
/// Why:
///   That count is what separates the line about a clean of state the worktree no longer holds from
/// the line about a clone of the source's build directory.
fn moved_aside(entry: &PrewarmEntry) -> usize {
    return usize::from(entry.staged_in.is_some());
}

/// What:
///  Hand the filling of the worktree's build directories to the commit service,
/// and answer the line the person reads.
/// Why:
///   The record is written before the service is reached,
/// so a wake that cannot reach it leaves the work named rather than lost,
/// and the add never fails for it:
/// the worktree is correct without the filling and only its first build is slower.
/// The writing of a record and the wake that may wait for a service to start are storage and socket
///  work,
/// so they run as a job of the blocking pool and never on the task that polls the three repairs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handedOver = async (request, entries) => { const told = await handOverWithinTheBudget(request.git, await commonDirectory(request.destination), request.destination, entries); return repaired(told); };
/// ```
pub(super) async fn hand_the_filling_over(request: &RepairRequest<'_>, entries: &[PrewarmEntry], common_dir: Option<&Path>) -> Finding {
    let began: Instant = begin_work();
    // Where a person watches what became of the work this add leaves behind: the queue's own states, and the service's journal.
    let the_queues_states: Option<String> = common_dir.map(queues_states);
    let told: Result<Result<(), String>, BlockingJobError> = match common_dir {
        Some(common) => {
            let state: (crate::transaction_git::GitContext, PathBuf, PathBuf, Vec<PrewarmEntry>) =
                (request.git.clone(), common.to_path_buf(), request.destination.to_path_buf(), entries.to_vec());
            run_a_job_or_in_turn(state, hand_over_in_a_job).await
        }
        None => Ok(Err(String::from("git could not name the repository's common directory"))),
    };
    end_work("cargoPrewarmHandOver", began);
    let staged: usize = entries.iter().map(moved_aside).sum();
    let workspace_noun: &str = if entries.len() == 1 { "build directory" } else { "build directories" };
    let them: &str = if entries.len() == 1 { "it" } else { "them" };
    let queue: String = match the_queues_states {
        Some(directory) => format!("`{directory}/{{pending,claimed,done,failed}}` and the service's journal"),
        // No queue directory could be named, so the line says only what happens; a command naming a path that is not there would
        //  be worse than none.
        None => String::from("the commit service's journal"),
    };
    // A workspace whose own copied state the add moved aside is named apart: what waits for the service there is a clean of
    //  state the worktree no longer holds, and a build started at once compiles it itself.
    let moved: String = if staged == 0 {
        String::new()
    } else {
        format!(
            ", and the cleaning of the {staged} {} the copy carried, moved aside before this command answered so that no build trusts {}",
            if staged == 1 { "build state" } else { "build states" },
            if staged == 1 { "it" } else { "them" }
        )
    };
    let detail: String = match told {
        Ok(Ok(())) => format!(
            "Cargo kept dependency builds and handed the filling of {} {workspace_noun} to the commit service{moved}, whose progress shows in {queue}, and which fills or cleans {them} after this command returns",
            entries.len()
        ),
        // The record stays and the service fills it when it next runs for this repository, so this is a line of the report and
        //  never the add's failure: the worktree is whole either way and its first build only compiles more.
        Ok(Err(reason)) => format!(
            "Cargo kept dependency builds and wrote the record that fills {} {workspace_noun}{moved}, which it could not hand to the commit service ({reason}); the worktree's first build compiles what it needs itself, and the record waits in {queue} until a service runs for this repository",
            entries.len()
        ),
        Err(refusal) => format!(
            "Cargo kept dependency builds and could not write the record that fills {} {workspace_noun}{moved} ({refusal}); the worktree's first build compiles what it needs itself, and nothing was left for a service to pick up",
            entries.len()
        ),
    };
    return Finding { tool: Tool::Cargo, outcome: Outcome::Repaired { detail, millis: began.elapsed().as_millis() } };
}
