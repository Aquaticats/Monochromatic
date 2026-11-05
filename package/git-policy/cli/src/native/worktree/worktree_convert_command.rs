//! What:
//!  `git cli-git convert-worktree-to-subvolume`: parse the options, find the worktrees to convert, run the conversion on each,
//!  and turn the results into lines and an exit status.
//! Why:
//!  A conversion takes seconds to minutes per worktree and is bound by the filesystem's metadata work, not by any one
//!  worktree's state, so the worktrees are converted side by side within a budget the host allows (`parallel_budget.rs`);
//!  progress lines go to standard error as they happen for the first unfinished worktree, and every worktree's lines come in
//!  input order (`parallel_runner.rs`). Exit status 0 means every worktree is converted, already a subvolume, previewed, or
//!  skipped under `--all`; 1 means a named worktree was refused before anything changed; 2 means a step failed after the
//!  journal began, with the way forward. One worktree's failure never stops another: each has its own journal and claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await executeConvert({ platform, args, cwd, interrupt, say });
//! ```

use crate::action::Action;
use crate::subvolume_command::main_checkout_root;
use crate::diagnostic_log::debug;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::line_sink::LineSink;
use crate::parallel_runner::{Event, OrderedEvents, OrderedRun, OrderedWork, run_ordered};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_index_files::linked_worktrees_inside;
use crate::subvolume_platform::{LinuxPlatform, Platform};
use crate::worktree_convert_arguments::{CONVERT_HELP, ConvertArguments, RISK_NOTICE};
use crate::worktree_convert::{ConvertOptions, Verdict, convert_worktree};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The tag of the diagnostic lines of this command.
const TAG: &str = "convertWorktrees";

/// The exit status of a refusal: a condition stood in the way of a named worktree and nothing was changed.
pub const REFUSED_EXIT_CODE: i32 = 1;

/// The exit status of a failure after the journal began.
pub const FAILED_EXIT_CODE: i32 = 2;

/// The worktrees a run covers: every linked worktree inside the main checkout, or the named paths resolved from `cwd`.
fn targets(arguments: &ConvertArguments, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    if arguments.all {
        let root: PathBuf = main_checkout_root(cwd)?;
        let mut inside: Vec<PathBuf> = Vec::new();
        for linked in linked_worktrees_inside(root.as_path()) {
            inside.push(linked.worktree);
        }
        return Ok(inside);
    }
    let mut named: Vec<PathBuf> = Vec::new();
    for word in &arguments.paths {
        named.push(cwd.join(word));
    }
    return Ok(named);
}

/// What:
///  The bytes one conversion holds at its peak, rounded up for a worktree of about two hundred thousand files.
/// Why:
///   The budget divides the free memory by it. A conversion holds the index entries and the tree-walk state of one worktree and
///  streams the file data through the kernel's clone call, so its peak follows the number of files and not their size: measured
///  on 2026-11-03 at 7484 KiB resident for a worktree of 16 thousand files (about 0.45 KiB per file), which is about 90 MiB at
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
///   The conversions are bound by the filesystem's metadata work, which stops scaling past a point. Measured on 2026-11-03 on a
///  btrfs volume (32 worktrees of about ten thousand files each, 28 threads, other sessions running, medians of three or more
///  runs at 1, 2, 4, 8, 16 and 18 workers: 61, 35, 19, 14, 12 and 12 seconds; further runs at 12, 16 and 24 workers: 11 to 30, 11
///  to 16 and 16 to 18 seconds): more than sixteen workers were never faster.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONVERSION_STORAGE_CEILING = 16;
/// ```
pub const CONVERSION_STORAGE_CEILING: usize = 16;

/// The workers a run of `lanes` independent conversions uses on `host`, with the person's `cap`.
pub fn conversion_workers(lanes: usize, cap: Option<usize>, host: &HostResources) -> usize {
    return worker_budget(&BudgetRequest { items: lanes, per_item_memory: CONVERSION_MEMORY, ceiling: Some(CONVERSION_STORAGE_CEILING), cap, host });
}

/// What:
///  Everything one run of the command needs besides the line sink.
/// Why:
///   The platform, the interrupt and the host's resources are injected so the controls run the real command on any filesystem
///  and with a host of any size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertRun = { platform: Platform; arguments: ConvertArguments; cwd: string; interrupt: Interrupt; host: HostResources };
/// ```
pub struct ConvertRun<'run> {
    /// The filesystem operations; shared by every worktree converted at once.
    pub platform: &'run (dyn Platform + Sync),
    /// The parsed options.
    pub arguments: &'run ConvertArguments,
    /// The directory relative paths and `--all` are resolved from.
    pub cwd: &'run Path,
    /// The simulated kill; shared by every worktree converted at once.
    pub interrupt: &'run Interrupt,
    /// The CPUs and memory the budget is derived from.
    pub host: &'run HostResources,
}

/// What:
///  Run the command over its targets and return the exit status; every line goes through `say`.
/// Why:
///   The platform and the interrupt are injected so the controls run the real command on any filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await execute(platform, arguments, cwd, interrupt, say);
/// ```
pub fn execute(platform: &(dyn Platform + Sync), arguments: &ConvertArguments, cwd: &Path, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> i32 {
    return execute_run(&ConvertRun { platform, arguments, cwd, interrupt, host: &HostResources::read() }, say);
}

/// What:
///  The lanes of a run: the targets that name one worktree share a lane, so they run one after another, and every other
///  target has a lane of its own.
/// Why:
///   The same worktree named twice (`wt wt`, `wt ./wt`) is converted by the first and found converted by the second, as in
///  a run of one after another; run side by side, the second would meet the claim of the first and be refused as busy with itself.
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
///  The conversion of one worktree, run by a worker of the ordered runner.
/// Why:
///   The platform, the options and the interrupt are what the conversion reads; the runner shares them among its worker threads,
///  so they are plain borrowed fields of a value that is `Sync`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConvertEachWorktree implements OrderedWork<string, Result<Verdict>> { run(index, path, say) { return convertWorktree(platform, path, options, interrupt, say); } }
/// ```
struct ConvertEachWorktree<'a> {
    /// The filesystem the conversions run on.
    platform: &'a (dyn Platform + Sync),
    /// What the invocation asked of each conversion.
    options: ConvertOptions,
    /// Where an injected interruption stops a conversion.
    interrupt: &'a Interrupt,
}

impl OrderedWork<PathBuf, Result<Verdict, MigrateError>> for ConvertEachWorktree<'_> {
    fn run(&self, _index: usize, path: &PathBuf, say: &mut dyn LineSink) -> Result<Verdict, MigrateError> {
        return convert_worktree(self.platform, path.as_path(), self.options, self.interrupt, say);
    }
}

/// What:
///  What the person is told as the conversions finish, in input order, and how many of each kind there were.
/// Why:
///   The runner calls the report on the calling thread only, one event at a time, so the counts need no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConversionReport implements OrderedEvents<Result<Verdict>> { refused = 0; converted = 0; failed = 0; }
/// ```
struct ConversionReport<'a> {
    /// Where the lines go.
    say: &'a mut dyn FnMut(&str),
    /// The worktrees, in the order their output is reported.
    list: &'a [PathBuf],
    /// Whether the run only lists what it would do.
    dry_run: bool,
    /// How many worktrees were skipped.
    refused: usize,
    /// How many worktrees were converted or completed.
    converted: usize,
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
            Event::End(_index, Ok(Verdict::WouldConvert)) => (self.say)("  would convert"),
            Event::End(_index, Ok(Verdict::AlreadySubvolume)) => (self.say)("  already a subvolume, skipped"),
            Event::End(_index, Ok(Verdict::Abandoned)) => (self.say)("  the unfinished conversion was undone"),
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
pub fn execute_run(run: &ConvertRun<'_>, say: &mut dyn FnMut(&str)) -> i32 {
    let ConvertRun { platform, arguments, cwd, interrupt, host } = *run;
    let list: Vec<PathBuf> = match targets(arguments, cwd) {
        Ok(found) => found,
        Err(reason) => {
            say(format!("git cli-git convert-worktree-to-subvolume: refused: {reason}").as_str());
            say("Nothing was changed.");
            return REFUSED_EXIT_CODE;
        }
    };
    let options: ConvertOptions = ConvertOptions { dry_run: arguments.dry_run, abandon: arguments.abandon, command_line_attributes: arguments.command_line_attributes };
    // Said once, before the first worktree is touched, when the run can delete an old tree.
    if !arguments.dry_run && !arguments.abandon && !list.is_empty() {
        say(RISK_NOTICE);
    }
    let lanes: Vec<Vec<usize>> = lanes_of(list.as_slice());
    let workers: usize = conversion_workers(lanes.len(), arguments.jobs, host);
    debug(TAG, format!("{} worktree(s) in {} lane(s) on {workers} worker(s) (host {host:?}, cap {:?})", list.len(), lanes.len(), arguments.jobs).as_str());
    // Each worktree has its own administrative directory, index, claim and journal, so the conversions share nothing but the
    //  filesystem. Lines and results come back to this thread, which says them in input order.
    let mut report: ConversionReport<'_> = ConversionReport { say, list: list.as_slice(), dry_run: arguments.dry_run, refused: 0, converted: 0, failed: 0 };
    run_ordered(OrderedRun {
        items: list.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work: &ConvertEachWorktree { platform, options, interrupt },
        on_event: &mut report,
    });
    let (refused, converted, failed): (usize, usize, usize) = (report.refused, report.converted, report.failed);
    let failures: String = if failed > 0 { format!(", {failed} failed") } else { String::new() };
    (report.say)(format!("{} worktree(s) looked at, {converted} converted or completed, {refused} skipped{failures}.", list.len()).as_str());
    if failed > 0 {
        return FAILED_EXIT_CODE;
    }
    return if refused > 0 && !arguments.all && !arguments.dry_run { REFUSED_EXIT_CODE } else { 0 };
}

/// What:
///  Say one line of progress on standard error.
/// Why:
///   The command prints progress as it goes; a write that fails (a closed pipe) must not stop the conversion.
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
///  The action of the command: help, a refusal, or the run on the real platform with progress on standard error.
/// Why:
///   The command reads `.git` itself and calls the filesystem, and starts no Git. It still needs to know which Git the wrapper
///  forwards to (`real_git`, resolved by the caller), because where that Git reads its system files from is a fact only that
///  Git reports, and the wrapper's own environment does not name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertAction(arguments: ConvertArguments, cwd: string, environment: [string, string][], realGit?: string): Action;
/// ```
pub fn convert_action(arguments: &ConvertArguments, cwd: &Path, environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    if arguments.help {
        return Action::Exit { code: 0, stdout: String::from(CONVERT_HELP), stderr: String::new() };
    }
    let code: i32 = execute(&LinuxPlatform::forwarding_to(environment, real_git), arguments, cwd, &Interrupt::never(), &mut say_on_standard_error);
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
