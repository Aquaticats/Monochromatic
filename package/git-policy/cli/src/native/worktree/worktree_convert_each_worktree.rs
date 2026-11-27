//! What:
//!  One worktree's conversion as the ordered runner's worker sees it:
//! a job of the process's blocking pool inside a task of the runtime,
//! with the conversion's own lines going to the run's consumer as it works.
//! Why:
//!  The conversion is storage work,
//! so it never runs on the thread that polls the runtime,
//! and the sink is a value the item owns because the job must be `'static`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ConvertEachWorktree implements OrderedAwaitingWork<string, Result<Verdict>> {
//!   async run(index, path, say) { return await runBlockingJobWithinTheBudget(() => convertWorktree(platform, path, options, interrupt, say)); }
//! }
//! ```

use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;
use crate::parallel_runner_on_the_runtime_awaiting::OrderedAwaitingWork;
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_platform::Platform;
use crate::worktree_convert::{
    ConvertOptions,
    Verdict,
    convert_worktree,
};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

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
pub(crate) struct ConvertEachWorktree {
    /// The filesystem the conversions run on.
    pub(crate) platform: Arc<dyn Platform + Send + Sync>,
    /// What the invocation asked of each conversion.
    pub(crate) options: ConvertOptions,
    /// Where an injected interruption stops a conversion.
    pub(crate) interrupt: Arc<Interrupt>,
}

impl OrderedAwaitingWork<PathBuf, Result<Verdict, MigrateError>> for ConvertEachWorktree {
    fn run<'a>(
        &'a self,
        _index: usize,
        path: &'a PathBuf,
        say: LinesToTheConsumer<Result<Verdict, MigrateError>>,
    ) -> Pin<Box<dyn Future<Output = Result<Verdict, MigrateError>> + Send + 'a>> {
        return Box::pin(async move {
            let platform: Arc<dyn Platform + Send + Sync> = Arc::clone(&self.platform);
            let interrupt: Arc<Interrupt> = Arc::clone(&self.interrupt);
            let options: ConvertOptions = self.options;
            let worktree: PathBuf = path.clone();
            // The sink is a value the item owns, so the job of the pool says the conversion's lines as it works.
            let mut lines: LinesToTheConsumer<Result<Verdict, MigrateError>> = say;
            let converted = run_blocking_job_within_the_budget(move || {
                return convert_worktree(
                    platform.as_ref(),
                    worktree.as_path(),
                    options,
                    interrupt.as_ref(),
                    &mut lines,
                );
            })
            .await;
            return match converted {
                Ok(verdict) => verdict,
                // Whether the worktree was touched is not known here: the step may have ended before it began,
                //  or after it changed something, and the reason says which kind of end this was.
                Err(refused) => Err(MigrateError::Runtime(refused.to_string())),
            };
        });
    }
}
