//! What:
//!     Builds every rule of a ruleset across worker threads,
//!  handing results back
//!          in input order.
//! Why:
//!      Rule construction is independent per rule and dominates `RegexSet::new`,
//!  so
//!          spreading it over the machine's cores shortens both the forbidden-strings
//!          build-time baseline compile and runtime compiles of large rule files.
//! Gotcha:
//!   The TS analogy is loose.
//!  These are real OS threads sharing memory,
//!  like
//!          `worker_threads` that can read the caller's arrays directly without copying,
//!          not `async` tasks taking turns on one event loop.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module parallel: build_rules(patterns, stopOnError) => outcomes sorted by index.
//! ```

/// What:
///     Imports `NonZeroUsize`,
///  an unsigned integer type the compiler guarantees is
///          never zero.
/// Why:
///      `available_parallelism` reports the core count as a `NonZeroUsize`,
///  and the
///          code below turns it back into a plain `usize` with `NonZeroUsize::get`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No 1:1 equivalent: TS has one `number` type with no "never zero" variant.
/// ```
use std::num::NonZeroUsize;

/// What:
///     Imports `AtomicUsize`,
///  an integer that many threads may read and update at
///          once without a lock,
///  and `Ordering`,
///  the consistency level for each read or
///          write.
/// Why:
///      The workers share a "next rule to build" counter and a "lowest failing rule"
///          marker;
///  atomics keep both correct when threads touch them simultaneously.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Closest analogue: Atomics.add / Atomics.load over a SharedArrayBuffer Int32Array.
/// ```
use std::sync::atomic::{AtomicUsize, Ordering};

/// What:
///     Imports `OnceLock<T>`,
///  a slot that starts empty and is written at most once,
///  safely
///          across threads.
/// Why:
///      Caches the process's core count after the first lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No 1:1 equivalent: a module-level `let cached: T | undefined` filled on first use.
/// ```
use std::sync::OnceLock;

/// What:
///     Imports the standard library thread module (spawning,
///  scopes,
///  core count).
/// Why:
///      The workers run on OS threads created from this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Worker } from "node:worker_threads";
/// ```
use std::thread;

/// What:
///     Imports one built rule's record and the per-rule builder.
/// Why:
///      Each worker turns a parsed pattern into a `BuiltRule` with `build_rule`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BuiltRule, build_rule } from "crate/build";
/// ```
use crate::build::{BuiltRule, build_rule};

/// What:
///     Imports the error type.
/// Why:
///      A rule that fails to parse or build reports a `CompileError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError } from "crate/error";
/// ```
use crate::error::CompileError;

/// What:
///     Imports the parser entry point.
/// Why:
///      Each worker parses its pattern text before building it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse } from "crate/parse";
/// ```
use crate::parse::parse;

/// What:
///     Stack size,
///  in bytes,
///  for each worker thread:
///  64 MiB of address space.
/// Why:
///      Parsing recurses once per group nesting level.
///  A thread spawned with Rust's
///          default 2 MiB stack overflows on patterns (three thousand nested groups,
///  measured
///          in a release build) that compile on a main thread,
///  where rules were built before
///          this module existed.
///  A main thread's stack comes from `ulimit -s`,
///  commonly 8 or
///          16 MiB,
///  so workers reserve more than that.
///  The operating system commits stack
///          pages only when touched,
///  so unused reservation costs no memory.
/// Gotcha:
///   A stack overflow aborts the whole process;
///  it is not a catchable panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORKER_STACK_BYTES: number = 64 * 1024 * 1024;
/// ```
const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

/// What:
///     One rule's result paired with its input index:
///  `(usize, Result<...>)` is a
///          two-element tuple.
///  `usize` is the platform-width unsigned integer (siblings:
///          `u32`,
///  `u64`,
///  `i64`).
///  `Result<BuiltRule, CompileError>` is either `Ok` with the
///          built rule or `Err` with the reason it failed.
/// Why:
///      Workers finish rules in any order,
///  so each result carries its index for the
///          final sort.
///  `usize` (not `u32`/`u64`) because slice indexing requires it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleOutcome = [index: number, result: BuiltRule | CompileError];
/// ```
pub(crate) type RuleOutcome = (usize, Result<BuiltRule, CompileError>);

/// What:
///     `fn(&str) -> Result<BuiltRule, CompileError>` is a function-pointer type:
///  any plain
///          named function that takes a borrowed pattern string and returns a built rule or
///          an error.
///  Siblings:
///  a generic closure parameter `impl Fn(&str) -> ...` and a boxed
///          closure `Box<dyn Fn(&str) -> ...>`.
/// Why:
///      Production passes [`build_pattern`];
///  tests pass a builder that panics,
///  to prove a
///          worker panic reaches the caller.
///  A function pointer (not a closure type) needs no
///          generic parameter and can be copied into every worker thread as-is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleBuilder = (pattern: string) => BuiltRule | CompileError;
/// ```
pub(crate) type RuleBuilder = fn(&str) -> Result<BuiltRule, CompileError>;

/// Parses and builds one pattern into a rule.
///
/// What:
///     Runs the parser,
///  then the rule builder on the parsed node.
/// Why:
///      The production [`RuleBuilder`] every worker runs per claimed index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_pattern(pattern: string): BuiltRule | CompileError {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn build_pattern(pattern: &str) -> Result<BuiltRule, CompileError> {
    // What:    `parse(...)` returns `Result<Node, CompileError>`; `.and_then(build_rule)` runs
    //          `build_rule` on the parsed node only when parsing succeeded, else passes the
    //          parse error through unchanged.
    // Why:     One result per rule, whichever stage failed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return build_rule(parse(pattern)); } catch (error) { return error; }
    // ```
    return parse(pattern).and_then(build_rule)
}

/// What:
///     A process-wide slot holding the core count once computed.
///  `static` means one
///          value for the whole program;
///  `OnceLock<usize>` starts empty and is filled at
///          most once,
///  even when threads race to fill it.
/// Why:
///      On Linux `available_parallelism` re-reads cgroup files on every call,
///  measured
///          at 64 microseconds,
///  which more than doubled a single-rule `RegexSet::new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let CORE_COUNT: number | undefined; // module-level cache
/// ```
static CORE_COUNT: OnceLock<usize> = OnceLock::new();

/// Returns how many cores this process may use,
///  computed once per process.
///
/// What:
///     Fills [`CORE_COUNT`] on the first call and reads it afterwards.
/// Why:
///      Keeps the per-compile cost of choosing a worker count to one memory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function core_count(): number {
///   CORE_COUNT ??= os.availableParallelism?.() ?? 1;
///   return CORE_COUNT;
/// }
/// ```
fn core_count() -> usize {
    // What:    `get_or_init(|| ...)` runs the closure only if the slot is empty, stores its
    //          result, and returns a reference `&usize`; the leading `*` copies the number out
    //          of that reference. `thread::available_parallelism()` returns
    //          `Result<NonZeroUsize, io::Error>`, and `.map_or(1, NonZeroUsize::get)` yields
    //          the count on success or `1` when the platform cannot report it.
    // Why:     An unknown core count degrades to building on the calling thread, which is
    //          the pre-threading behavior.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // CORE_COUNT ??= os.availableParallelism?.() ?? 1;
    // return CORE_COUNT;
    // ```
    return *CORE_COUNT.get_or_init(|| return thread::available_parallelism().map_or(1, NonZeroUsize::get))
}

/// Builds every pattern into a rule on as many threads as the machine has cores.
///
/// What:
///     Picks the worker count (the core count,
///  capped at the pattern count) and
///          delegates to [`build_rules_with`].
///  With `stop_on_error`,
///  rules after the
///          lowest-index failure may be skipped;
///  without it,
///  every rule is built.
/// Why:
///      The single entry point both `RegexSet::new` (strict) and
///          `RegexSet::compile_lenient` (keep what compiles) share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_rules(patterns: string[], stop_on_error: boolean): RuleOutcome[] {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn build_rules(patterns: &[&str], stop_on_error: bool) -> Vec<RuleOutcome> {
    // What:    The cached core count.
    // Why:     The upper bound on useful workers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cores = core_count();
    // ```
    let cores = core_count();
    // What:    The smaller of the core count and the pattern count.
    // Why:     A worker with no rule to claim would only cost a thread spawn; a single
    //          pattern therefore builds on the calling thread with no spawn at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const worker_count = Math.min(cores, patterns.length);
    // ```
    let worker_count = cores.min(patterns.len());
    return build_rules_with(patterns, stop_on_error, worker_count, WORKER_STACK_BYTES, build_pattern)
}

/// Builds every pattern into a rule with explicit workers,
///  stacks,
///  and builder,
///  sorted by index.
///
/// What:
///     Runs the claim-and-build loop on the calling thread for one worker or fewer,
///          else on spawned workers with `stack_bytes` stacks running `build`,
///  then sorts the
///          outcomes by index.
/// Why:
///      Separated from [`build_rules`] so tests can force the threaded path on any
///          machine,
///  force every spawn to fail with an impossible stack size,
///  and inject a
///          panicking builder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build_rules_with(patterns: string[], stop_on_error: boolean, worker_count: number, stack_bytes: number, build: RuleBuilder): RuleOutcome[] {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn build_rules_with(
    patterns: &[&str],
    stop_on_error: bool,
    worker_count: usize,
    stack_bytes: usize,
    build: RuleBuilder,
) -> Vec<RuleOutcome> {
    // What:    `AtomicUsize::new(0)` creates the shared "next index to claim" counter.
    // Why:     Every worker claims rules from this one counter, so each index is built
    //          exactly once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const next = new Int32Array(new SharedArrayBuffer(4)); // starts at 0
    // ```
    let next = AtomicUsize::new(0);
    // What:    The shared lowest failing index, starting at `usize::MAX` (no failure yet).
    // Why:     Strict builds skip indices above it; `usize::MAX` compares greater than
    //          every real index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let first_error = Number.MAX_SAFE_INTEGER;
    // ```
    let first_error = AtomicUsize::new(usize::MAX);
    // What:    `let mut outcomes` is a binding the code below may modify (`mut`).
    //          `&next` and `&first_error` lend the atomics read-only; atomics update through
    //          a shared borrow, which is what lets several threads hold them at once.
    // Why:     One worker or fewer means no threads are worth spawning.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let outcomes = worker_count <= 1
    //   ? drain(patterns, stop_on_error, next, first_error, build)
    //   : drain_on_workers(patterns, stop_on_error, next, first_error, worker_count, stack_bytes, build);
    // ```
    let mut outcomes = if worker_count <= 1 {
        drain(patterns, stop_on_error, &next, &first_error, build)
    } else {
        drain_on_workers(patterns, stop_on_error, &next, &first_error, worker_count, stack_bytes, build)
    };
    // What:    `sort_unstable_by_key` sorts in place by the tuple's first element (`.0`,
    //          the index). `|outcome| return outcome.0` is a closure (an arrow function).
    //          "Unstable" means equal keys may swap, which cannot happen with unique
    //          indices.
    // Why:     Callers push rules into dense, index-aligned vectors and report the
    //          lowest-index error first, exactly as the one-at-a-time loop did.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // outcomes.sort((a, b) => a[0] - b[0]);
    // ```
    outcomes.sort_unstable_by_key(|outcome| return outcome.0);
    return outcomes
}

/// Runs the claim-and-build loop on spawned worker threads and gathers their outcomes.
///
/// What:
///     Spawns up to `worker_count` workers with `stack_bytes` stacks inside a thread
///          scope,
///  joins them,
///  and concatenates their outcomes.
///  When no worker can be
///          spawned,
///  the calling thread runs the loop itself.
/// Why:
///      A scope guarantees every worker finishes before this function returns,
///  so the
///          workers may borrow the caller's pattern slice and counters directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function drain_on_workers(patterns: string[], stop_on_error: boolean, next: Counter, first_error: Counter, worker_count: number, stack_bytes: number, build: RuleBuilder): Promise<RuleOutcome[]> {
///   const batches = await Promise.all(range(worker_count).map(() => runWorker(drain)));
///   return batches.flat();
/// }
/// ```
fn drain_on_workers(
    patterns: &[&str],
    stop_on_error: bool,
    next: &AtomicUsize,
    first_error: &AtomicUsize,
    worker_count: usize,
    stack_bytes: usize,
    build: RuleBuilder,
) -> Vec<RuleOutcome> {
    // What:    `thread::scope(|scope| { ... })` runs the closure and waits for every thread
    //          spawned through `scope` to finish before returning the closure's value.
    // Why:     The workers borrow `patterns`, `next`, and `first_error` from this stack
    //          frame; the scope proves to the compiler those borrows cannot outlive it.
    // Gotcha:  Unlike `Promise.all`, nothing here is async: the calling thread blocks
    //          until all workers are done.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return await Promise.all(workers).then((batches) => batches.flat());
    // ```
    return thread::scope(|scope| {
        // What:    `Vec::new()` creates an empty growable array of join handles (a handle
        //          is how the caller waits for one thread and receives its return value).
        // Why:     Each spawned worker's handle is kept so its outcomes can be collected.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const handles: Promise<RuleOutcome[]>[] = [];
        // ```
        let mut handles = Vec::new();
        for _ in 0..worker_count {
            // What:    `thread::Builder::new().stack_size(...)` configures a thread with a
            //          larger stack; `.spawn_scoped(scope, || ...)` starts it inside the
            //          scope running the closure, returning `Ok(handle)` or `Err(io error)`
            //          when the OS refuses to create the thread.
            // Why:     The explicit stack keeps deeply nested patterns compiling as they did
            //          on the main thread.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const spawned = tryStartWorker(() => drain(patterns, stop_on_error, next, first_error, build));
            // ```
            let spawned = thread::Builder::new()
                .stack_size(stack_bytes)
                .spawn_scoped(scope, || return drain(patterns, stop_on_error, next, first_error, build));
            // What:    `match spawned { Ok(handle) => ..., Err(error) => ... }` branches on
            //          whether the thread started.
            // Why:     A refused spawn leaves fewer workers, not a failure: the remaining
            //          workers (or the calling thread, below) still claim every rule.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (spawned.ok) handles.push(spawned.handle);
            // else logger.warn("rule worker spawn failed", { cause: spawned.error });
            // ```
            match spawned {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    tracing::warn!(cause = %error, "rule worker spawn failed; continuing with fewer workers");
                }
            }
        }
        // What:    With zero spawned workers, the calling thread drains the queue itself;
        //          otherwise start from an empty list.
        // Why:     Every rule must still be built even when the OS refuses all threads.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const outcomes = handles.length === 0 ? drain(patterns, stop_on_error, next, first_error, build) : [];
        // ```
        let mut outcomes = if handles.is_empty() {
            drain(patterns, stop_on_error, next, first_error, build)
        } else {
            Vec::new()
        };
        for handle in handles {
            // What:    `handle.join()` waits for the worker and returns `Ok(batch)` with its
            //          outcomes, or `Err(payload)` when the worker panicked.
            //          `std::panic::resume_unwind(payload)` re-raises that same panic on this
            //          thread.
            // Why:     A panicking build must reach the caller's `catch_unwind` boundary with
            //          its original payload, as it did when rules built on the calling thread.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { outcomes.push(...(await handle)); } catch (error) { throw error; }
            // ```
            match handle.join() {
                Ok(batch) => outcomes.extend(batch),
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
        return outcomes
    })
}

/// Claims rule indices from the shared counter and builds each until none remain.
///
/// What:
///     Repeatedly takes the next unclaimed index,
///  runs `build` on that pattern,
///  and
///          records the outcome;
///  in strict mode it stops once its index is past the lowest
///          failure seen so far.
/// Why:
///      The same loop serves the calling thread and every worker.
///  An index below the
///          lowest real failure is never skipped,
///  so that failure and every rule before it
///          are always built,
///  which is what strict callers report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function drain(patterns: string[], stop_on_error: boolean, next: Counter, first_error: Counter, build: RuleBuilder): RuleOutcome[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn drain(
    patterns: &[&str],
    stop_on_error: bool,
    next: &AtomicUsize,
    first_error: &AtomicUsize,
    build: RuleBuilder,
) -> Vec<RuleOutcome> {
    // What:    An empty growable list of this worker's outcomes.
    // Why:     Each worker keeps its own list, so no lock is needed while building.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcomes: RuleOutcome[] = [];
    // ```
    let mut outcomes = Vec::new();
    loop {
        // What:    `fetch_add(1, Ordering::SeqCst)` adds one to the counter and returns the
        //          value from before the add, as one indivisible step. `SeqCst` is the
        //          strictest consistency level.
        // Why:     Two workers can never claim the same index.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const index = Atomics.add(next, 0, 1);
        // ```
        let index = next.fetch_add(1, Ordering::SeqCst);
        if index >= patterns.len() {
            return outcomes;
        }
        if stop_on_error && index > first_error.load(Ordering::SeqCst) {
            return outcomes;
        }
        // What:    Calls the injected builder on the claimed pattern.
        // Why:     One outcome per rule, whichever stage failed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const result = build(patterns[index]);
        // ```
        let result = build(patterns[index]);
        if stop_on_error && result.is_err() {
            // What:    `fetch_min(index, ...)` lowers the shared marker to `index` when
            //          `index` is smaller, as one indivisible step.
            // Why:     Other workers stop claiming rules past the earliest known failure.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // first_error = Math.min(first_error, index);
            // ```
            first_error.fetch_min(index, Ordering::SeqCst);
        }
        outcomes.push((index, result));
    }
}

/// What:
///     Unit tests for the threaded rule builder,
///  in a sidecar (max-lines exempt).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "parallel_tests.rs"]
mod tests;
