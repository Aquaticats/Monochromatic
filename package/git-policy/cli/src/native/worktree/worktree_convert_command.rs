//! What:
//!  `git monochromatic-git-policy-cli convert-worktree-to-subvolume`:
//! parse the options,
//! find the worktrees to convert,
//! run the conversion on each,
//!  and turn the results into lines and an exit status.
//! Why:
//!  A conversion takes seconds to minutes per worktree and is bound by the filesystem's metadata work,
//! not by any one
//!  worktree's state,
//! so the worktrees are converted side by side within a budget the host allows (`parallel_budget.rs`);
//!  progress lines go to standard error as they happen for the first unfinished worktree,
//! and every worktree's lines come in
//!  input order (`parallel_runner.rs`).
//! Each conversion is one job of the process's blocking pool,
// awaited by a task of the runtime
//!  (`parallel_runner_on_the_runtime_awaiting.rs`),
//! so the process's one bound covers it and the lines it says while it works go to the
//!  consumer as they are said.
//! Exit status 0 means every worktree is converted,
//! already a subvolume,
//! previewed,
//! undone by `--abandon`,
//! or
//!  skipped under `--all`;
//! 1 means a named worktree was refused before anything changed;
//! 2 means a step failed after the
//!  journal began,
//! with the way forward.
//! One worktree's failure never stops another:
//! each has its own journal and claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await executeConvert({ platform, args, cwd, interrupt, say });
//! ```

use crate::action::Action;
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::subvolume_command::main_checkout_root;
use crate::diagnostic_log::debug;
use crate::lazy_runtime::run_on_runtime;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::{Event, OrderedEvents};
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;
use crate::parallel_runner_on_the_runtime_awaiting::{OrderedAwaitingWork, OrderedRunOfAwaitingOwned, run_ordered_awaiting_on_the_runtime};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_index_files::{LinkedWorktreeScan, scan_linked_worktrees};
use crate::subvolume_platform::{LinuxPlatform, Platform};
use crate::worktree_convert_arguments::{CONVERT_HELP, ConvertArguments, RISK_NOTICE};
use crate::worktree_convert::{ConvertOptions, Verdict, convert_worktree};
use std::collections::HashMap;
use std::ffi::OsString;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

/// The tag of the diagnostic lines of this command.
const TAG: &str = "convertWorktrees";

/// The exit status of a refusal:
/// a condition stood in the way of a named worktree and nothing was changed.
pub const REFUSED_EXIT_CODE: i32 = 1;

/// The exit status of a failure after the journal began.
pub const FAILED_EXIT_CODE: i32 = 2;

/// What:
///  The worktrees a run covers,
/// and the registrations of `--all` it leaves out with why.
/// Why:
///   A run that covers fewer worktrees than are registered says which ones it left out,
/// so no worktree is passed over without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Targets = { covered: string[]; leftOut: string[] };
/// ```
struct Targets {
    /// The worktrees to convert,
    /// in input order.
    covered: Vec<PathBuf>,
    /// One sentence per registered worktree that is not covered.
    left_out: Vec<String>,
}

/// The worktrees a run covers:
/// every linked worktree inside the main checkout,
/// or every named path resolved from `cwd`.
fn targets(arguments: &ConvertArguments, cwd: &Path) -> Result<Targets, String> {
    if arguments.all {
        let root: PathBuf = main_checkout_root(cwd)?;
        let scan: LinkedWorktreeScan = scan_linked_worktrees(root.as_path())?;
        let mut covered: Vec<PathBuf> = Vec::new();
        for linked in scan.inside {
            covered.push(linked.worktree);
        }
        let mut left_out: Vec<String> = Vec::new();
        for left in scan.left_out {
            left_out.push(left.reason);
        }
        return Ok(Targets { covered, left_out });
    }
    let mut named: Vec<PathBuf> = Vec::new();
    for word in &arguments.paths {
        named.push(cwd.join(word));
    }
    return Ok(Targets { covered: named, left_out: Vec::new() });
}

/// What:
///  The bytes one conversion holds at its peak,
/// rounded up for a worktree of about two hundred thousand files.
/// Why:
///   The budget divides the free memory by it.
/// A conversion holds the index entries and the tree-walk state of one worktree and
///  streams the file data through the kernel's clone call,
/// so its peak follows the number of files and not their size:
/// measured
///  on 2026-11-03 at 7484 KiB resident for a worktree of 16 thousand files (about 0.45 KiB per file),
/// which is about 90 MiB at
///  two hundred thousand files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONVERSION_MEMORY = 128 * MiB;
/// ```
pub const CONVERSION_MEMORY: u64 = 128 << 20;

/// What:
///  The most worktrees converted at once however many CPUs and bytes the host has.
/// Why:
///   The conversions are bound by the filesystem's metadata work,
/// which stops scaling past a point.
/// Measured on 2026-11-03 on a
///  btrfs volume (32 worktrees of about ten thousand files each,
/// 28 threads,
/// other sessions running,
/// medians of three or more
///  runs at 1,
/// 2,
/// 4,
/// 8,
/// 16 and 18 workers:
/// 61,
/// 35,
/// 19,
/// 14,
/// 12 and 12 seconds;
/// further runs at 12,
/// 16 and 24 workers:
/// 11 to 30,
/// 11
///  to 16 and 16 to 18 seconds):
/// more than sixteen workers were never faster.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONVERSION_STORAGE_CEILING = 16;
/// ```
pub const CONVERSION_STORAGE_CEILING: usize = 16;

/// The workers a run of `lanes` independent conversions uses on `host`,
/// with the person's `cap`.
pub fn conversion_workers(lanes: usize, cap: Option<usize>, host: &HostResources) -> usize {
    return worker_budget(&BudgetRequest { items: lanes, per_item_memory: CONVERSION_MEMORY, ceiling: Some(CONVERSION_STORAGE_CEILING), cap, host });
}

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
///  Run the command over its targets and return the exit status;
/// every line goes through `say`.
/// Why:
///   The platform and the interrupt are injected so the controls run the real command on any filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await execute(platform, arguments, cwd, interrupt, say);
/// ```
pub fn execute(platform: Arc<dyn Platform + Send + Sync>, arguments: &ConvertArguments, cwd: &Path, interrupt: Arc<Interrupt>, say: &mut dyn FnMut(&str)) -> i32 {
    return execute_run(&ConvertRun { platform, arguments, cwd, interrupt, host: &HostResources::read() }, say);
}

/// What:
///  The lanes of a run:
/// the targets that name one worktree share a lane,
/// so they run one after another,
/// and every other
///  target has a lane of its own.
/// Why:
///   The same worktree named twice (`wt wt`,
/// `wt ./wt`) is converted by the first and found converted by the second,
/// as in
///  a run of one after another;
/// run side by side,
/// the second would meet the claim of the first and be refused as busy with itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lanes = groupBy(list.keys(), (index) => realpath(list[index]) ?? list[index]);
/// ```
fn lanes_of(list: &[PathBuf]) -> Vec<Vec<usize>> {
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    let mut lane_of: HashMap<PathBuf, usize> = HashMap::new();
    for (index, path) in list.iter().enumerate() {
        // A path that cannot be resolved stands for itself, so it still gets a lane of its own.
        let real: PathBuf = match std::fs::canonicalize(path) {
            Ok(resolved) => resolved,
            Err(_unresolved) => path.clone(),
        };
        match lane_of.get(&real) {
            Some(&lane) => lanes[lane].push(index),
            None => {
                lane_of.insert(real, lanes.len());
                lanes.push(vec![index]);
            }
        }
    }
    return lanes;
}

/// What:
///  The conversion of one worktree,
/// run by a worker of the ordered runner.
/// Why:
///   The platform,
/// the options and the interrupt are what the conversion reads;
/// the runner shares them among its worker threads,
///  so they are plain borrowed fields of a value that is `Sync`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConvertEachWorktree implements OrderedAwaitingWork<string, Result<Verdict>> { async run(index, path, say) { return await runBlockingJobWithinTheBudget(() => convertWorktree(platform, path, options, interrupt, say)); } }
/// ```
struct ConvertEachWorktree {
    /// The filesystem the conversions run on.
    platform: Arc<dyn Platform + Send + Sync>,
    /// What the invocation asked of each conversion.
    options: ConvertOptions,
    /// Where an injected interruption stops a conversion.
    interrupt: Arc<Interrupt>,
}

impl OrderedAwaitingWork<PathBuf, Result<Verdict, MigrateError>> for ConvertEachWorktree {
    fn run<'a>(&'a self, _index: usize, path: &'a PathBuf, say: LinesToTheConsumer<Result<Verdict, MigrateError>>) -> Pin<Box<dyn Future<Output = Result<Verdict, MigrateError>> + Send + 'a>> {
        return Box::pin(async move {
            let platform: Arc<dyn Platform + Send + Sync> = Arc::clone(&self.platform);
            let interrupt: Arc<Interrupt> = Arc::clone(&self.interrupt);
            let options: ConvertOptions = self.options;
            let worktree: PathBuf = path.clone();
            // The sink is a value the item owns, so the job of the pool says the conversion's lines as it works.
            let mut lines: LinesToTheConsumer<Result<Verdict, MigrateError>> = say;
            let converted = run_blocking_job_within_the_budget(move || {
                return convert_worktree(platform.as_ref(), worktree.as_path(), options, interrupt.as_ref(), &mut lines);
            })
            .await;
            return match converted {
                Ok(verdict) => verdict,
                // The worktree was not touched, and the runtime's own reason is why.
                Err(refused) => Err(MigrateError::Runtime(refused.to_string())),
            };
        });
    }
}

/// What:
///  What the person is told as the conversions finish,
/// in input order,
/// and how many of each kind there were.
/// Why:
///   The runner calls the report on the calling thread only,
/// one event at a time,
/// so the counts need no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConversionReport implements OrderedEvents<Result<Verdict>> { refused = 0; converted = 0; failed = 0; }
/// ```
struct ConversionReport<'a> {
    /// Where the lines go.
    say: &'a mut dyn FnMut(&str),
    /// The worktrees,
    /// in the order their output is reported.
    list: &'a [PathBuf],
    /// Whether the run only lists what it would do.
    dry_run: bool,
    /// How many worktrees were refused before anything changed,
    /// which is what ends a run of named worktrees with status 1.
    refused: usize,
    /// How many worktrees needed nothing because the path is a subvolume already.
    already_subvolume: usize,
    /// How many unfinished conversions the run undid because it was asked to (`--abandon`).
    undone: usize,
    /// How many worktrees were converted or completed.
    converted: usize,
    /// How many worktrees a dry run would convert or complete.
    would_convert: usize,
    /// How many conversions failed.
    failed: usize,
}

impl OrderedEvents<Result<Verdict, MigrateError>> for ConversionReport<'_> {
    fn on_event(&mut self, event: Event<'_, Result<Verdict, MigrateError>>) {
        match event {
            Event::Begin(index) => (self.say)(format!("{}", self.list[index].display()).as_str()),
            Event::Line(_index, text) => (self.say)(text),
            Event::End(_index, Ok(Verdict::Converted)) => {
                self.converted += 1;
                (self.say)("  converted into a subvolume");
            }
            Event::End(_index, Ok(Verdict::Resumed)) => {
                self.converted += 1;
                (self.say)(if self.dry_run { "  an unfinished conversion would be completed" } else { "  the unfinished conversion was completed" });
            }
            Event::End(_index, Ok(Verdict::WouldConvert)) => {
                self.would_convert += 1;
                (self.say)("  would convert");
            }
            Event::End(_index, Ok(Verdict::AlreadySubvolume)) => {
                self.already_subvolume += 1;
                (self.say)("  already a subvolume, skipped");
            }
            Event::End(_index, Ok(Verdict::Abandoned)) => {
                self.undone += 1;
                (self.say)("  the unfinished conversion was undone");
            }
            Event::End(_index, Ok(Verdict::Skipped(reasons))) => {
                self.refused += 1;
                for reason in reasons {
                    (self.say)(format!("  skipped: {reason}").as_str());
                }
            }
            Event::End(_index, Err(error)) => {
                self.failed += 1;
                (self.say)(format!("  failed: {error}").as_str());
                (self.say)("  The conversion is journaled. Run the same command again to finish it, or add --abandon to undo it before its swap.");
            }
        }
    }
}

/// `execute` with the host's resources chosen by the caller.
/// The callers of this face are synchronous (the command of the wrapper);
/// so it enters the process's runtime once and waits there,
/// and from inside the runtime it refuses,
/// where `execute_run_awaiting` is the face such a caller uses.
pub fn execute_run(run: &ConvertRun<'_>, say: &mut dyn FnMut(&str)) -> i32 {
    return match run_on_runtime(execute_run_awaiting(run, say)) {
        Ok(code) => code,
        Err(refusal) => {
            say(format!("git monochromatic-git-policy-cli convert-worktree-to-subvolume: could not start the work: {refusal}").as_str());
            return FAILED_EXIT_CODE;
        }
    };
}

/// What:
///  `execute_run` on the runtime:
/// every worktree is a task of the awaited ordered runner,
/// and each conversion is one job of the blocking pool inside its task.
/// Why:
///   A caller that is itself asynchronous cannot enter the runtime,
/// so it awaits this face;
/// the lines, the order and the exit status are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await executeRunAwaiting({ platform, arguments, cwd, interrupt, host }, say);
/// ```
pub async fn execute_run_awaiting(run: &ConvertRun<'_>, say: &mut dyn FnMut(&str)) -> i32 {
    let ConvertRun { platform, arguments, cwd, interrupt, host } = run;
    let found: Targets = match targets(arguments, cwd) {
        Ok(found) => found,
        Err(reason) => {
            say(format!("git monochromatic-git-policy-cli convert-worktree-to-subvolume: refused: {reason}").as_str());
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
    let options: ConvertOptions = ConvertOptions { dry_run: arguments.dry_run, abandon: arguments.abandon, command_line_attributes: arguments.command_line_attributes };
    // Said once, before the first worktree is touched, when the run can delete an old tree.
    if !arguments.dry_run && !arguments.abandon && !list.is_empty() {
        say(RISK_NOTICE);
    }
    let lanes: Vec<Vec<usize>> = lanes_of(list.as_ref());
    let workers: usize = conversion_workers(lanes.len(), arguments.jobs, host);
    debug(TAG, format!("{} worktree(s) in {} lane(s) on {workers} worker(s) (host {host:?}, cap {:?})", list.len(), lanes.len(), arguments.jobs).as_str());
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
        work: Arc::new(ConvertEachWorktree { platform: Arc::clone(platform), options, interrupt: Arc::clone(interrupt) }),
        on_event: &mut report,
    })
    .await;
    let (refused, already_subvolume, undone, converted, would_convert, failed): (usize, usize, usize, usize, usize, usize) =
        (report.refused, report.already_subvolume, report.undone, report.converted, report.would_convert, report.failed);
    let stopped: Option<String> = match ran {
        Ok(()) => None,
        // The worktrees the run did not reach were not touched, and what ended the run is said before the summary.
        Err(ended) => Some(ended.to_string()),
    };
    if let Some(reason) = stopped.as_deref() {
        (report.say)(format!("  the run stopped before every worktree was done: {reason}").as_str());
    }
    // A worktree that is a subvolume already costs nothing and was skipped, as the line beside its name says, so it counts in
    //  the summary's skips; a refusal is counted apart because a refusal of a named worktree alone decides the status.
    let skipped: usize = refused + already_subvolume;
    let failures: String = if failed > 0 { format!(", {failed} failed") } else { String::new() };
    let undone_text: String = if undone > 0 { format!(", {undone} undone") } else { String::new() };
    let left_text: String = if left_out > 0 { format!("; {left_out} registered worktree(s) left out") } else { String::new() };
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
    return if refused > 0 && !arguments.all && !arguments.dry_run { REFUSED_EXIT_CODE } else { 0 };
}

/// What:
///  Say one line of progress on standard error.
/// Why:
///   The command prints progress as it goes;
/// a write that fails (a closed pipe) must not stop the conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sayOnStandardError = (text: string) => { try { process.stderr.write(`${text}\n`); } catch {} };
/// ```
fn say_on_standard_error(text: &str) {
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "{text}");
}

/// What:
///  The action of the command:
/// help,
/// a refusal,
/// or the run on the real platform with progress on standard error.
/// Why:
///   The command reads `.git` itself and calls the filesystem,
/// and starts no Git.
/// It still needs to know which Git the wrapper
///  forwards to (`real_git`,
/// resolved by the caller),
/// because where that Git reads its system files from is a fact only that
///  Git reports,
/// and the wrapper's own environment does not name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertAction(arguments: ConvertArguments, cwd: string, environment: [string, string][], realGit?: string): Action;
/// ```
pub fn convert_action(arguments: &ConvertArguments, cwd: &Path, environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    if arguments.help {
        return Action::Exit { code: 0, stdout: String::from(CONVERT_HELP), stderr: String::new() };
    }
    let code: i32 = execute(Arc::new(LinuxPlatform::forwarding_to(environment, real_git)), arguments, cwd, Arc::new(Interrupt::never()), &mut say_on_standard_error);
    return Action::Exit { code, stdout: String::new(), stderr: String::new() };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_command_tests.rs"]
mod tests;

/// The controls for worktrees converted side by side stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_parallel_tests.rs"]
mod parallel_tests;
