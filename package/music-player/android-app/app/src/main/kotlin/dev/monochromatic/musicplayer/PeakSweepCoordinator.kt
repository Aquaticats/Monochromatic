// ============================================================================
// File summary:
//
// This file holds the *concurrency skeleton* shared by both loudness sweeps: the
// foreground initial index (`PeakSweepService`, many parallel decoders) and the
// background upkeep (`PeakSweepWorker`, a single low-priority decoder). Both used
// to carry their own near-identical loop; this is the one extracted, tested copy.
//
// `sweepTracksInParallel` walks a list of work items across N coroutine workers
// pulling from one lock-free atomic cursor (so every item is claimed exactly
// once, no matter the worker count or interleaving), reports per-outcome tallies,
// flushes the caller's cache on a fixed cadence so an abrupt kill loses at most a
// batch, and honors cooperative cancellation (a stopped service/worker unwinds
// its decoders, and the tail still flushes once under `NonCancellable`).
//
// It is deliberately generic (`<T>`) and free of any Android type: the per-track
// op, the flush, and the progress callback are all injected. That is what lets it
// run as a fast, deterministic JVM unit test (`PeakSweepCoordinatorTest`) with
// fakes, instead of an on-device test that would see an already-warm cache and
// exercise no real parallel decode.
// ============================================================================

// What:     `package dev.monochromatic.musicplayer` puts these declarations in
//           the app's single namespace, the same one the service, the worker, and
//           `SweepOutcome` live in, so they refer to each other without imports.
// Why:      The service and worker call `sweepTracksInParallel`, and the
//           coordinator returns `SweepOutcome` values produced by `measureAndCache`.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword; the directory + module system play this role.
// ```
package dev.monochromatic.musicplayer

// What:     `import kotlinx.coroutines.CoroutineDispatcher` names the type that
//           decides WHICH thread (pool) a coroutine runs on. The caller hands one
//           in: the foreground sweep passes its default-priority pool, the upkeep
//           worker passes its single low-priority thread.
// Why:      The coordinator must launch its workers onto the caller's chosen pool
//           rather than picking a thread policy itself.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { CoroutineDispatcher } from "kotlinx/coroutines"; // ~ which worker pool
// ```
import kotlinx.coroutines.CoroutineDispatcher

// What:     `import kotlinx.coroutines.NonCancellable` names a special coroutine
//           context that makes the work inside it ignore cancellation.
// Why:      The final flush must complete even when the sweep is being cancelled,
//           or the last batch of measurements would be lost on a clean stop.
//
// In TS you'd write (pseudocode):
// ```ts
// import { NonCancellable } from "kotlinx/coroutines"; // run-even-if-aborted
// ```
import kotlinx.coroutines.NonCancellable

// What:     `import kotlinx.coroutines.coroutineScope` names the suspending
//           builder that runs a block and does not return until every child
//           coroutine launched inside it has finished (a structured-concurrency
//           barrier).
// Why:      We launch N workers and must wait for all of them to drain the cursor
//           before reporting the tally; `coroutineScope` is that join.
//
// In TS you'd write (pseudocode):
// ```ts
// import { coroutineScope } from "kotlinx/coroutines"; // ~ await Promise.all of children
// ```
import kotlinx.coroutines.coroutineScope

// What:     `import kotlinx.coroutines.isActive` names the coroutine-scope flag
//           that flips to false the moment the coroutine is cancelled.
// Why:      Each worker loops `while (isActive)` so a cancelled sweep stops
//           claiming new items promptly instead of running to the end.
//
// In TS you'd write (pseudocode):
// ```ts
// import { isActive } from "kotlinx/coroutines"; // ~ !signal.aborted
// ```
import kotlinx.coroutines.isActive

// What:     `import kotlinx.coroutines.launch` names the builder that starts a new
//           child coroutine that runs concurrently with its siblings.
// Why:      We start one `launch` per worker; together they consume the cursor.
//
// In TS you'd write (pseudocode):
// ```ts
// import { launch } from "kotlinx/coroutines"; // ~ spawn a concurrent task
// ```
import kotlinx.coroutines.launch

// What:     `import kotlinx.coroutines.withContext` switches the coroutine context
//           for the duration of a block (here, into `NonCancellable`).
// Why:      The final flush is wrapped in `withContext(NonCancellable) { ... }` so
//           it survives cancellation.
//
// In TS you'd write (pseudocode):
// ```ts
// import { withContext } from "kotlinx/coroutines";
// ```
import kotlinx.coroutines.withContext

// What:     `import java.util.concurrent.atomic.AtomicInteger` names a thread-safe
//           integer whose updates (`getAndIncrement`, `incrementAndGet`) are
//           atomic: concurrent callers never tear or lose an update.
// Why:      The cursor and every tally counter are shared across workers, so they
//           must be atomic, not plain `Int`s.
//
// In TS you'd write (pseudocode):
// ```ts
// import { AtomicInteger } from "java/util/concurrent/atomic"; // JS has no real one (single-threaded)
// ```
import java.util.concurrent.atomic.AtomicInteger
import dev.monochromatic.musicplayer.core.SweepPauseGate

// What:     `suspend fun <T> sweepTracksInParallel(...)` declares the coordinator.
//           - `suspend` because it awaits the workers and the injected suspend ops.
//           - `<T>` is a type parameter: the item type is the caller's (a `Track`
//             in production, an `Int` in the test); the coordinator never inspects
//             it, only passes it to `process`.
//           - the trailing lambda params (`process`, `onFlush`, `onProgress`) are
//             the seams the test replaces with fakes.
// Why:      One tested loop both sweeps share, instead of two hand-written copies.
//
// In TS you'd write (pseudocode):
// ```ts
// async function sweepTracksInParallel<T>(opts: {...}): Promise<SweepTally> { ... }
// ```
/**
 * Runs [process] over the first [maxTracks] of [items] across [workers] coroutines on [dispatcher],
 * a lock-free [AtomicInteger] cursor handing each index to whichever worker is free so every visited
 * item is claimed exactly once. Calls [onFlush] after every [flushBatch] [SweepOutcome.MEASURED]
 * results and once more at the end (even under cancellation, via [NonCancellable]), so an abrupt stop
 * loses at most one batch. Fires [onProgress] every [notifyEvery] visited items when [notifyEvery] is
 * positive. Cooperative cancellation of the calling scope stops the workers promptly. Pass
 * [workers] of 1 for a serial sweep. Returns the [SweepTally].
 *
 * When [pauseGate] is given, every worker passes it before each claim: while it is paused no new track
 * is claimed (a decode already running finishes), resuming continues the same sweep, and cancelling the
 * scope while paused ends the workers instead of hanging them. Null means the sweep cannot be paused.
 *
 * @example
 * ```kotlin
 * val tally = sweepTracksInParallel(
 *     items = tracks, workers = 4, dispatcher = pool, flushBatch = 32, notifyEvery = 16,
 *     maxTracks = Int.MAX_VALUE,
 *     process = { measureAndCache(ctx, it.uri.toUri(), pool) },
 *     onFlush = { PeakCacheStore.flush(ctx) },
 *     onProgress = { done, total -> postProgress(done, total) },
 * )
 * ```
 */
suspend fun <T> sweepTracksInParallel(
    items: List<T>,
    workers: Int,
    dispatcher: CoroutineDispatcher,
    flushBatch: Int,
    notifyEvery: Int,
    maxTracks: Int,
    process: suspend (T) -> SweepOutcome,
    onFlush: suspend () -> Unit,
    onProgress: (Int, Int) -> Unit,
    pauseGate: SweepPauseGate? = null,
): SweepTally {
    // What:     `minOf(items.size, maxTracks)` is the highest index the workers
    //           will claim: the whole list, or the cap, whichever is smaller.
    // Why:      The test passes a small cap for a fast slice; the service passes
    //           `Int.MAX_VALUE` to mean "the whole library".
    /** One past the last index any worker will process (the smaller of size and cap). */
    val limit: Int = minOf(items.size, maxTracks)
    // What:     `AtomicInteger(0)` shared cursor; `getAndIncrement` atomically
    //           returns the current value and bumps it, so two workers can never
    //           receive the same index.
    // Why:      Lock-free, load-balanced work distribution: a worker that finishes
    //           a short track grabs the next index immediately.
    /** Shared claim cursor; each `getAndIncrement` hands one unique index to one worker. */
    val cursor = AtomicInteger(0)
    /** Count of items visited (any outcome), the basis for progress and the tally. */
    val processed = AtomicInteger(0)
    /** Running count of fresh measurements, also the flush-cadence clock. */
    val measured = AtomicInteger(0)
    /** Running count of cache hits. */
    val cached = AtomicInteger(0)
    /** Running count of unfingerprintable (skipped) items. */
    val unfingerprintable = AtomicInteger(0)
    /** Running count of failed decodes. */
    val failed = AtomicInteger(0)
    // What:     `try { coroutineScope { ... } } finally { ...final flush... }`.
    //           The `coroutineScope` joins all workers; the `finally` runs on both
    //           normal completion and cancellation.
    // Why:      Guarantee the tail flush regardless of how the sweep ends.
    try {
        coroutineScope {
            // What:     `repeat(workers) { launch(dispatcher) { ... } }` starts
            //           `workers` concurrent worker coroutines on the caller's pool.
            // Why:      Parallel decode for the foreground sweep; `workers == 1`
            //           degenerates to the serial upkeep loop.
            repeat(workers) {
                launch(dispatcher) {
                    // What:     `while (isActive)` keeps claiming until the cursor
                    //           is drained OR the coroutine is cancelled.
                    // Why:      Promptly abandon work when the service/worker stops.
                    while (isActive) {
                        // What:     `pauseGate?.awaitResumed()` suspends here while the user has paused the sweep.
                        // Why:      D84 reads Pause as a temporary pause: no new claim while paused, so a paused
                        //           worker never holds an index, and the same sweep continues on resume. The wait is
                        //           cancellable, so a stopped service does not hang on a paused gate.
                        pauseGate?.awaitResumed()
                        /** This worker's freshly claimed index, or past-the-end when drained. */
                        val index: Int = cursor.getAndIncrement()
                        if (index >= limit) {
                            break
                        }
                        // What:     `when (process(items[index]))` runs the injected
                        //           per-item op and tallies its outcome. For a
                        //           MEASURED result we also advance the flush clock:
                        //           `measured.incrementAndGet()` returns a unique
                        //           value per measurement, so exactly one worker sees
                        //           each multiple of `flushBatch` and flushes, making
                        //           the cadence deterministic under any interleaving.
                        // Why:      Bounded loss on a kill without a per-item disk write.
                        when (process(items[index])) {
                            SweepOutcome.MEASURED ->
                                if (measured.incrementAndGet() % flushBatch == 0) {
                                    onFlush()
                                }
                            SweepOutcome.CACHED -> cached.incrementAndGet()
                            SweepOutcome.UNFINGERPRINTABLE -> unfingerprintable.incrementAndGet()
                            SweepOutcome.FAILED -> failed.incrementAndGet()
                        }
                        /** Visited count after this item, used to pace progress updates. */
                        val done: Int = processed.incrementAndGet()
                        if (notifyEvery > 0 && done % notifyEvery == 0) {
                            onProgress(done, limit)
                        }
                    }
                }
            }
        }
    } finally {
        // What:     `withContext(NonCancellable) { onFlush() }` persists whatever is
        //           in memory once the workers have stopped, ignoring cancellation.
        // Why:      On a clean stop (timeout, service stop) the tail batch is saved
        //           rather than lost; a hard process kill cannot run this, which is
        //           why the periodic flush above is the real kill-resilience.
        withContext(NonCancellable) {
            onFlush()
        }
    }
    // What:     Snapshot every atomic into the immutable `SweepTally`.
    // Why:      Hand the caller (and the test) the final accounting.
    return SweepTally(
        total = items.size,
        processed = processed.get(),
        measured = measured.get(),
        cached = cached.get(),
        unfingerprintable = unfingerprintable.get(),
        failed = failed.get(),
    )
}
