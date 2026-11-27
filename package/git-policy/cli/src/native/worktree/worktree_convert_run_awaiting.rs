//! What:
//!  The run of `git monochromatic-git-policy-cli convert-worktree-to-subvolume` on the runtime:
//! what the run needs besides the line sink,
//! the exit statuses it can end with,
//! every worktree as a task of the awaited ordered runner,
//! and the summary it says last.
//! Why:
//!  A caller that is itself asynchronous cannot enter the runtime,
//! so it awaits this face;
//! the lines,
//! the order and the exit status are the synchronous face's.
//! Finding the targets and their lanes (a scan of the linked worktrees) runs on the task that
//!  awaits this face,
//! not on the blocking pool,
//! because a job of the pool must own what it reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await executeRunAwaiting({ platform, arguments, cwd, interrupt, host }, say);
//! ```

use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::diagnostic_log::debug;
use crate::parallel_budget::HostResources;
use crate::parallel_runner_on_the_runtime_awaiting::{
    OrderedRunOfAwaitingOwned,
    run_ordered_awaiting_on_the_runtime,
};
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::Platform;
use crate::worktree_convert::ConvertOptions;
use crate::worktree_convert_arguments::{
    ConvertArguments,
    RISK_NOTICE,
};
use crate::worktree_convert_budget::conversion_workers;
use crate::worktree_convert_each_worktree::ConvertEachWorktree;
use crate::worktree_convert_report::ConversionReport;
use crate::worktree_convert_targets::{
    FoundTargets,
    Targets,
    find_the_targets_and_lanes,
};
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;

const TAG: &str = "convertWorktrees";

/// The exit status of a refusal:
/// a condition stood in the way of a named worktree and nothing was changed.
pub const REFUSED_EXIT_CODE: i32 = 1;

/// The exit status of a failure after the journal began.
pub const FAILED_EXIT_CODE: i32 = 2;

/// What:
///  Everything one run of the command needs besides the line sink.
/// Why:
///   The platform,
/// the interrupt and the host's resources are injected so the controls run the real command on any filesystem
///  and with a host of any size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertRun = { platform: Platform; arguments: ConvertArguments; cwd: string; interrupt: Interrupt; host: HostResources };
/// ```
pub struct ConvertRun<'run> {
    /// The filesystem operations;
    /// shared by every worktree converted at once,
    /// and owned because a conversion is a job of the blocking pool,
    /// which must be `'static`.
    ///  A `LinuxPlatform` given here must have been settled
    /// (`with_its_conversion_environment_settled`),
    /// because a conversion runs inside the runtime,
    /// where the bridge that starts the Git child the facts are learned from refuses.
    pub platform: Arc<dyn Platform + Send + Sync>,
    /// The parsed options.
    pub arguments: &'run ConvertArguments,
    /// The directory relative paths and `--all` are resolved from.
    pub cwd: &'run Path,
    /// The simulated kill;
    /// shared by every worktree converted at once,
    /// and owned for the reason the platform is.
    pub interrupt: Arc<Interrupt>,
    /// The CPUs and memory the budget is derived from.
    pub host: &'run HostResources,
}

/// What:
///  `execute_run` on the runtime:
/// every worktree is a task of the awaited ordered runner,
/// and each conversion is one job of the blocking pool inside its task.
/// Why:
///   A caller that is itself asynchronous cannot enter the runtime,
/// so it awaits this face;
/// the lines,
/// the order and the exit status are the synchronous face's.
/// One limit,
/// said rather than hidden:
/// finding the targets and their lanes (a scan of the linked worktrees)
/// runs on the task that awaits this face,
/// not on the blocking pool,
/// because a job of the pool must own what it reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await executeRunAwaiting({ platform, arguments, cwd, interrupt, host }, say);
/// ```
pub async fn execute_run_awaiting(
    run: &ConvertRun<'_>,
    say: &mut dyn FnMut(&str),
) -> i32 {
    let ConvertRun {
        platform,
        arguments,
        cwd,
        interrupt,
        host,
    } = run;
    // Finding the targets and grouping their lanes is storage and Git work (`canonicalize`, a scan of the linked
    //  worktrees), so it is one job of the blocking pool: a path whose lookup stalls would otherwise hold the thread that
    //  polls this runtime before the future's first await, where no timeout could expire.
    let (arguments_owned, cwd_owned): (ConvertArguments, PathBuf) =
        ((*arguments).clone(), cwd.to_path_buf());
    let located: FoundTargets = match run_blocking_job_within_the_budget(move || {
        return find_the_targets_and_lanes(&arguments_owned, cwd_owned.as_path());
    })
    .await
    {
        Ok(found) => found,
        Err(refused) => {
            say(format!(
                "git monochromatic-git-policy-cli convert-worktree-to-subvolume: refused: {refused}"
            )
            .as_str());
            say("Nothing was changed.");
            return REFUSED_EXIT_CODE;
        }
    };
    let FoundTargets {
        found: targets_or_refusal,
        lanes,
    } = located;
    let found: Targets = match targets_or_refusal {
        Ok(covered) => covered,
        Err(reason) => {
            say(format!(
                "git monochromatic-git-policy-cli convert-worktree-to-subvolume: refused: {reason}"
            )
            .as_str());
            say("Nothing was changed.");
            return REFUSED_EXIT_CODE;
        }
    };
    // `--all` covers every linked worktree inside the main checkout; a registration outside it or one Git would prune is
    //  said with its reason, so a run never looks at fewer worktrees than are registered without saying so.
    for reason in &found.left_out {
        say(format!("  left out of --all: {reason}").as_str());
    }
    let left_out: usize = found.left_out.len();
    let list: Arc<[PathBuf]> = Arc::from(found.covered);
    let options: ConvertOptions = ConvertOptions {
        dry_run: arguments.dry_run,
        abandon: arguments.abandon,
        command_line_attributes: arguments.command_line_attributes,
    };
    // Said once, before the first worktree is touched, when the run can delete an old tree.
    if !arguments.dry_run && !arguments.abandon && !list.is_empty() {
        say(RISK_NOTICE);
    }
    let workers: usize = conversion_workers(lanes.len(), arguments.jobs, host);
    debug(
        TAG,
        format!(
            "{} worktree(s) in {} lane(s) on {workers} worker(s) (host {host:?}, cap {:?})",
            list.len(),
            lanes.len(),
            arguments.jobs
        )
        .as_str(),
    );
    // Each worktree has its own administrative directory, index, claim and journal, so the conversions share nothing but the
    //  filesystem. Lines and results come back to the calling task, which says them in input order.
    let mut report: ConversionReport<'_> = ConversionReport {
        say,
        list: list.as_ref(),
        dry_run: arguments.dry_run,
        refused: 0,
        already_subvolume: 0,
        undone: 0,
        converted: 0,
        would_convert: 0,
        failed: 0,
    };
    let ran = run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::clone(&list),
        lanes: Arc::from(lanes),
        workers,
        work: Arc::new(ConvertEachWorktree {
            platform: Arc::clone(platform),
            options,
            interrupt: Arc::clone(interrupt),
        }),
        on_event: &mut report,
    })
    .await;
    let (refused, already_subvolume, undone, converted, would_convert, failed): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ) = (
        report.refused,
        report.already_subvolume,
        report.undone,
        report.converted,
        report.would_convert,
        report.failed,
    );
    let stopped: Option<String> = match ran {
        Ok(()) => None,
        // The worktrees the run did not reach were not touched, and what ended the run is said before the summary.
        Err(ended) => Some(ended.to_string()),
    };
    if let Some(reason) = stopped.as_deref() {
        (report.say)(
            format!("  the run stopped before every worktree was done: {reason}").as_str(),
        );
    }
    // A worktree that is a subvolume already costs nothing and was skipped, as the line beside its name says, so it counts in
    //  the summary's skips; a refusal is counted apart because a refusal of a named worktree alone decides the status.
    let skipped: usize = refused + already_subvolume;
    let failures: String = if failed > 0 {
        format!(", {failed} failed")
    } else {
        String::new()
    };
    let undone_text: String = if undone > 0 {
        format!(", {undone} undone")
    } else {
        String::new()
    };
    let left_text: String = if left_out > 0 {
        format!("; {left_out} registered worktree(s) left out")
    } else {
        String::new()
    };
    // A dry run converts nothing, so its summary counts what it listed as a conversion it would make; the other counters
    // would say "0 converted" beside lines that say "would convert".
    let counted: String = if arguments.dry_run {
        format!("{} would convert or complete", would_convert + converted)
    } else {
        format!("{converted} converted or completed")
    };
    (report.say)(format!("{} worktree(s) looked at, {counted}, {skipped} skipped{undone_text}{failures}{left_text}.", list.len()).as_str());
    if failed > 0 || stopped.is_some() {
        return FAILED_EXIT_CODE;
    }
    return if refused > 0 && !arguments.all && !arguments.dry_run {
        REFUSED_EXIT_CODE
    } else {
        0
    };
}
