//! What:
//!  Process planned files on a bounded number of worker threads,
//!  in a deterministic result order.
//! Why:
//!  `--concurrency` caps how many files are linted at once.
//!  A panic while processing one file
//! becomes that file's processing finding instead of ending the run,
//!  and files that need the
//! Cargo-backed semantic engine stay on the calling thread,
//!  which owns it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcomes = await mapWithConcurrency(plans, limit, plan => processFile(plan));
//! ```

/// Import per-file processing,
///  its outcome and the plan model.
use crate::{
    run_failure::{file_start, panic_text, processing_failure},
    run_file::{FileOutcome, SourceOutcome, Writer, process_file_with, process_source},
    run_lfs::LfsRepos,
    run_plan::FilePlan,
    run_write::write_atomically,
    rust_file_engine::RustFileEngine,
};
/// Import unwind containment;
///  `AssertUnwindSafe` states that a caught panic leaves no state we reuse unsafely.
/// Import a shared counter workers advance to claim the next file.
/// Import scoped threads,
///  which may borrow data owned by the spawning function.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::{AtomicUsize, Ordering},
    thread::{Builder, Scope, ScopedJoinHandle},
};

/// What:
///  Stack size for every thread that lints,
///  in bytes:
///  the invocation thread and the workers.
/// Why:
///  Parsers recurse on nested input,
///  and a stack overflow cannot be caught.
///  Spawned threads
/// default to 2 MiB,
///  and a main thread's stack is set by the platform:
///  8 MiB is the common Linux
/// default,
///  while the MSVC linker reserves 1 MB unless told otherwise.
///  This value is the Linux main
/// thread's,
///  given to every lint thread explicitly so no platform default decides it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORKER_STACK_BYTES = 8 * 1024 * 1024;
/// ```
pub(crate) const WORKER_STACK_BYTES: usize = 8 * 1024 * 1024;

/// What:
///  A thread builder with the lint stack,
///  for the invocation thread and for every worker.
/// Why:
///  One constructor keeps every thread that lints on the same explicit stack size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lintThread(): ThreadOptions { return { stackBytes: WORKER_STACK_BYTES }; }
/// ```
pub(crate) fn lint_thread() -> Builder {
    return Builder::new().stack_size(WORKER_STACK_BYTES);
}

/// What:
///  State every worker reads while claiming and processing files.
/// Why:
///  Scoped threads borrow this for the duration of the run;
///  nothing in it is mutated except
/// the atomic claim counter.
/// `'run` is a lifetime:
///  the borrowed plans and repositories outlive all workers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared = { plans: FilePlan[]; claimed: number[]; fix: boolean; lfs: LfsRepos };
/// ```
struct Shared<'run> {
    /// Every plan of the run,
    ///  addressed by index.
    plans: &'run [FilePlan],
    /// Indexes of plans workers may process,
    ///  in claim order.
    queue: &'run [usize],
    /// Position in `queue` of the next unclaimed entry.
    next: AtomicUsize,
    /// Whether fixes are applied and written.
    fix: bool,
    /// Shared LFS repository facts.
    lfs: &'run LfsRepos,
}

/// What:
///  The outcome reported for a file whose processing panicked.
/// Why:
///  A parser or rule panic means the file was not verified;
///  it must fail the run with its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function panicked(plan: FilePlan, payload: unknown): FileOutcome;
/// ```
fn panicked(plan: &FilePlan, payload: &(dyn std::any::Any + Send)) -> FileOutcome {
    return FileOutcome {
        findings: vec![processing_failure(
            plan.display.as_str(),
            file_start(),
            format!(
                "Processing panicked: {}. This file was not verified and was not rewritten.",
                panic_text(payload)
            ),
        )],
        notes: Vec::<String>::new(),
        written: false,
    };
}

/// What:
///  Process one plan through a given writer,
///  converting a panic into a processing finding.
/// Why:
///  `catch_unwind` needs a callable;
///  the closure only forwards to the named `process_file_with`.
/// The writer parameter lets a test raise a real panic inside the contained region.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function containedWith(plan, fix, lfs, engine, writer): FileOutcome { try { return processFileWith(...); } catch (error) { return panicked(plan, error); } }
/// ```
pub fn contained_with(
    plan: &FilePlan,
    fix: bool,
    lfs: &LfsRepos,
    engine: Option<&mut RustFileEngine>,
    writer: Writer,
) -> FileOutcome {
    let attempt: Result<FileOutcome, Box<dyn std::any::Any + Send>> = catch_unwind(
        AssertUnwindSafe(
            #[cfg_attr(
                dylint_lib = "monochromatic_lint_closure",
                expect(
                    avoidable_closure,
                    reason = "catch_unwind accepts only a callable without parameters, so the plan, the fix flag, the LFS map, the engine and the writer must be moved into a closure"
                )
            )]
            || {
                return process_file_with(plan, fix, lfs, engine, writer);
            },
        ),
    );
    match attempt {
        Ok(outcome) => return outcome,
        Err(payload) => return panicked(plan, payload.as_ref()),
    }
}

/// What:
///  Process one plan with the production atomic writer,
///  with panic containment.
/// Why:
///  Every file the run processes goes through this one entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contained(plan, fix, lfs, engine): FileOutcome;
/// ```
pub fn contained(
    plan: &FilePlan,
    fix: bool,
    lfs: &LfsRepos,
    engine: Option<&mut RustFileEngine>,
) -> FileOutcome {
    return contained_with(plan, fix, lfs, engine, write_atomically);
}

/// What:
///  Check or fix one in-memory source,
///  converting a panic into a processing finding.
/// Why:
///  Standard-input mode has no file to read or write but needs the same containment;
///  on a
/// panic the source is reported as unverified and is returned unchanged by the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function containedSource(plan, source, fix, lfs, engine): SourceOutcome;
/// ```
pub fn contained_source(
    plan: &FilePlan,
    source: &str,
    fix: bool,
    lfs: &LfsRepos,
    engine: Option<&mut RustFileEngine>,
) -> SourceOutcome {
    let attempt: Result<SourceOutcome, Box<dyn std::any::Any + Send>> = catch_unwind(
        AssertUnwindSafe(
            #[cfg_attr(
                dylint_lib = "monochromatic_lint_closure",
                expect(
                    avoidable_closure,
                    reason = "catch_unwind accepts only a callable without parameters, so the plan, the source, the fix flag, the LFS map and the engine must be moved into a closure"
                )
            )]
            || {
                return process_source(plan, source, fix, lfs, engine);
            },
        ),
    );
    match attempt {
        Ok(outcome) => return outcome,
        Err(payload) => {
            let failed: FileOutcome = panicked(plan, payload.as_ref());
            return SourceOutcome {
                findings: failed.findings,
                fixed: None,
                notes: failed.notes,
            };
        }
    }
}

/// What:
///  One worker's loop:
///  claim the next queued plan until none remain.
/// Why:
///  `fetch_add` hands each queue position to exactly one worker without a lock.
/// The result pairs each outcome with its plan index so the caller can restore plan order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function work(shared: Shared): [number, FileOutcome][];
/// ```
fn work(shared: &Shared<'_>) -> Vec<(usize, FileOutcome)> {
    let mut done: Vec<(usize, FileOutcome)> = Vec::<(usize, FileOutcome)>::new();
    loop {
        let position: usize = shared.next.fetch_add(1, Ordering::Relaxed);
        let Some(index): Option<&usize> = shared.queue.get(position) else {
            return done;
        };
        let plan: &FilePlan = &shared.plans[*index];
        done.push((*index, contained(plan, shared.fix, shared.lfs, None)));
    }
}

/// What:
///  Start up to `workers` scoped threads running the worker loop,
///  then join them all.
/// Why:
///  A worker that cannot be started,
///  or that dies outside per-file containment,
///  leaves its
/// files unclaimed or unreported;
///  the caller turns every missing outcome into a processing finding.
/// `'scope` is the lifetime of the thread scope and `'env` of the data the threads borrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spawnAndJoin(scope, shared, workers): [number, FileOutcome][];
/// ```
fn spawn_and_join<'scope, 'env>(
    scope: &'scope Scope<'scope, 'env>,
    shared: &'env Shared<'env>,
    workers: usize,
) -> Vec<(usize, FileOutcome)> {
    let mut handles: Vec<ScopedJoinHandle<'scope, Vec<(usize, FileOutcome)>>> = Vec::new();
    for _ in 0..workers {
        let builder: Builder = lint_thread();
        // A spawn failure (resource exhaustion) leaves fewer workers; the rest still drain the queue.
        // The thread API needs a callable; the closure only forwards to the named `work`.
        if let Ok(handle) = builder.spawn_scoped(
            scope,
            #[cfg_attr(
                dylint_lib = "monochromatic_lint_closure",
                expect(
                    avoidable_closure,
                    reason = "spawn_scoped runs the worker closure on the spawned thread after this call returns, so the shared run state must travel inside it"
                )
            )]
            || return work(shared),
        ) {
            handles.push(handle);
        }
    }
    let mut collected: Vec<(usize, FileOutcome)> = Vec::<(usize, FileOutcome)>::new();
    if handles.is_empty() {
        // No thread could be started: drain the queue on the calling thread instead.
        collected.extend(work(shared));
    }
    for handle in handles {
        if let Ok(done) = handle.join() {
            collected.extend(done);
        }
    }
    return collected;
}

/// What:
///  Run the worker loop on scoped threads and collect every claimed outcome.
/// Why:
///  Scoped threads may borrow the run's state and are all joined before this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runWorkers(shared: Shared, workers: number): [number, FileOutcome][];
/// ```
fn run_workers(shared: &Shared<'_>, workers: usize) -> Vec<(usize, FileOutcome)> {
    // The scope API needs a callable; the closure only forwards to the named `spawn_and_join`.
    return std::thread::scope(
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "std::thread::scope hands its callback only the scope reference, so the shared run state and the worker count can reach spawn_and_join only through a closure"
            )
        )]
        |scope| return spawn_and_join(scope, shared, workers),
    );
}

/// What:
///  Process every plan and return outcomes in plan order.
/// Why:
///  Output order must not depend on thread scheduling.
///  With a limit of one,
///  or one file,
/// no worker is started.
///  Semantic-engine plans run on the calling thread after the workers finish,
/// so the limit is never exceeded.
///  The executable calls this on its invocation thread,
///  which has
/// the same stack as a worker (`run_process`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processPlans(plans, fix, lfs, concurrency, engine): FileOutcome[];
/// ```
pub fn process_plans(
    plans: &[FilePlan],
    fix: bool,
    lfs: &LfsRepos,
    concurrency: usize,
    engine: &mut RustFileEngine,
) -> Vec<FileOutcome> {
    let mut general: Vec<usize> = Vec::<usize>::new();
    let mut semantic: Vec<usize> = Vec::<usize>::new();
    for (index, plan) in plans.iter().enumerate() {
        if plan.needs_semantic_engine() {
            semantic.push(index);
        } else {
            general.push(index);
        }
    }
    let mut slots: Vec<Option<FileOutcome>> = Vec::<Option<FileOutcome>>::new();
    for _ in plans {
        slots.push(None);
    }
    let workers: usize = concurrency.min(general.len());
    if workers <= 1 {
        for index in &general {
            slots[*index] = Some(contained(&plans[*index], fix, lfs, None));
        }
    } else {
        let shared: Shared<'_> = Shared {
            plans,
            queue: general.as_slice(),
            next: AtomicUsize::new(0),
            fix,
            lfs,
        };
        for (index, outcome) in run_workers(&shared, workers) {
            slots[index] = Some(outcome);
        }
    }
    for index in &semantic {
        slots[*index] = Some(contained(&plans[*index], fix, lfs, Some(&mut *engine)));
    }
    let mut outcomes: Vec<FileOutcome> = Vec::<FileOutcome>::new();
    for (index, slot) in slots.into_iter().enumerate() {
        match slot {
            Some(outcome) => outcomes.push(outcome),
            None => outcomes.push(FileOutcome {
                findings: vec![processing_failure(
                    plans[index].display.as_str(),
                    file_start(),
                    String::from(
                        "A worker thread stopped before reporting this file. It was not verified.",
                    ),
                )],
                notes: Vec::<String>::new(),
                written: false,
            }),
        }
    }
    return outcomes;
}

/// Ordering,
///  containment and concurrency controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_workers_tests.rs"]
mod tests;
