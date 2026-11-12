//! What:
//!  What one look at the run found:
//! the totals it added up over every task of this process and of the processes it started,
//! and what each of those tasks had moved.
//! Why:
//!   A task's own figures only grow,
//! and a task that ends takes its figures out of the totals a look adds up,
//! so the judge compares each task with the figures that task showed in the look before,
//! and the totals are left to the report and to the controls that name them.
//! A task is named by the process number and the task number it was read under,
//! because a thread of one process can carry a number another process gave its own thread.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // run_progress_look_figures.ts: export type TaskFigures, taskNumber, type Look.
//! ```

/// What:
///  What one task had moved when a look saw it.
/// Why:
///   A task's own bytes and processor ticks only grow,
/// and a task that ends takes its figures out of the totals a look adds up,
/// so the totals can fall while a task that stayed moved;
/// the figures of the tasks themselves are what says a task grew.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TaskFigures = { task: bigint; bytes: number; ticks: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TaskFigures {
    /// The process and the task the kernel numbers,
    /// packed into one figure.
    pub task: u64,
    /// Every byte this task read and wrote so far.
    pub bytes: u64,
    /// Every processor tick this task used so far.
    pub ticks: u64,
}

/// What:
///  The one figure a task's process number and its task number make together.
/// Why:
///   A task is named by both,
/// because a thread of one process can carry a number another process gave its own thread,
/// and the two figures are put side by side so one number names the task in `Look`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taskNumber = (process: number, task: number) => (BigInt(process) << 32n) | BigInt(task);
/// ```
pub(crate) fn task_number(process: i32, task: i32) -> u64 {
    let process_figure: u64 = u64::try_from(process).unwrap_or(0);
    let task_figure: u64 = u64::try_from(task).unwrap_or(0);
    return (process_figure << 32) | task_figure;
}

/// What:
///  What one look at the run found:
/// the totals it added up over every task of this process and of the processes it started,
/// and what each of those tasks had moved.
/// Why:
///   A task able to run or waiting for the disk is at work now,
/// and its own bytes and processor ticks only grow,
/// so a look
///  says whether anything moved since the one before it.
/// The totals can fall while a task that stayed moved,
/// so the judge reads the tasks themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Look = { bytes: number; ticks: number; working: boolean; members: number; tasks: TaskFigures[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Look {
    /// Every byte the run's tasks read and wrote,
    /// from their `io` files.
    pub bytes: u64,
    /// Every processor tick the run's tasks used,
    /// from their `stat` files.
    pub ticks: u64,
    /// Whether some task is able to run (`R`) or waiting for the disk (`D`).
    pub working: bool,
    /// How many tasks were seen.
    pub members: u64,
    /// What each task the look found had moved,
    /// so the next look can tell a task that moved from one that ended.
    pub tasks: Vec<TaskFigures>,
}
