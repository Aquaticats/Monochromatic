// The loop that turns sweep reports into paced notification posts (D107), free of Android types.

// What:     `package ...core` places the publisher beside the pacer and the progress model it drives.
// Why:      The publisher calls injected `post` and `remove` functions, so JVM tests drive it with fakes.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.transformWhile
import kotlinx.coroutines.flow.update

// What:     `class AnalysisNotificationPublisher(...)` owns the latest sweep state and the loop that posts it.
// Why:      Sweep workers must never wait on the notification service, so they only write a conflated state
//           value (a lock-free compare and set); this loop runs on its own coroutine, reads the newest value,
//           asks the pacer, and makes the possibly slow `post` call. A report that lands while the loop is in a
//           rate limit wait replaces the waiting decision, so the newest value is the one posted.
//
// In TS you'd write (pseudocode):
// ```ts
// class AnalysisNotificationPublisher {
//   report(done, total): void;   // cheap, any thread
//   complete(): void; abandon(): void;
//   run(): Promise<void>;        // resolves after the notification is removed
// }
// ```
/**
 * Publishes the analysis notification at a limited rate from the newest sweep state.
 *
 * Call [report], [complete] and [abandon] from any thread; they never block. Run [run] on a coroutine of its own,
 * which returns only after the end of the analysis has been handled.
 *
 * @param paused pause flag of the sweep, mirrored into the notification so its button flips
 * @param pacer rate limit and duplicate rules, used from [run] only
 * @param clockMs monotonic clock in milliseconds
 * @param post shows the given content, and may block on the notification service
 * @param remove takes the notification away
 */
class AnalysisNotificationPublisher(
    private val paused: StateFlow<Boolean>,
    private val pacer: AnalysisUpdatePacer,
    private val clockMs: () -> Long,
    private val post: (AnalysisNotificationContent) -> Unit,
    private val remove: () -> Unit,
) {
    /** Counts and end state of the sweep; the pause flag joins them in [run]. */
    private val state: MutableStateFlow<AnalysisSnapshot> =
        MutableStateFlow(AnalysisSnapshot(done = 0, total = 0, paused = false, end = AnalysisEnd.RUNNING))

    // What:     `fun currentContent(): AnalysisNotificationContent` is what the notification shows right now.
    // Why:      The service must hand a notification to `startForeground` on every start command, and a repeat
    //           start must not reset a running bar to its indeterminate first frame.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // currentContent() { return analysisNotificationContent(state.done, state.total, paused.value); }
    // ```
    /** Returns the content for the newest counts and pause flag, without posting it. */
    fun currentContent(): AnalysisNotificationContent {
        /** Newest counts. */
        val latest: AnalysisSnapshot = state.value
        return analysisNotificationContent(done = latest.done, total = latest.total, paused = paused.value)
    }

    // What:     `fun report(done, total)` records the newest counts.
    // Why:      Parallel workers report out of order, so within one total the larger done count wins and the
    //           bar never moves back; a different total starts a new count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // report(done, total) {
    //   state = state.total === total ? { ...state, done: max(state.done, done) } : { ...state, done, total };
    // }
    // ```
    /**
     * Records the newest counts without blocking.
     *
     * @param done tracks visited so far
     * @param total tracks the sweep will visit
     */
    fun report(done: Int, total: Int) {
        state.update { current ->
            if (current.total == total) {
                current.copy(done = maxOf(done, current.done))
            } else {
                current.copy(done = done, total = total)
            }
        }
    }

    /**
     * Marks the analysis as finished, so [run] posts the final value and then removes the notification.
     * Has no effect once the analysis has already ended.
     */
    fun complete() {
        end(AnalysisEnd.COMPLETED)
    }

    /**
     * Marks the analysis as stopped early, so [run] only removes the notification. Has no effect once the analysis
     * has already ended, so calling it after [complete] in a cleanup block cannot cancel the final value.
     */
    fun abandon() {
        end(AnalysisEnd.ABANDONED)
    }

    /** Moves a running analysis to [ending]; the first ending wins. */
    private fun end(ending: AnalysisEnd) {
        state.update { current ->
            if (current.end == AnalysisEnd.RUNNING) current.copy(end = ending) else current
        }
    }

    // What:     `suspend fun run()` collects the newest state and applies the pacer's steps until the end.
    // Why:      `collectLatest` cancels the handling of an older state when a newer one arrives, which is what
    //           turns a rate limit wait into "post the newest value when the window closes"; `transformWhile`
    //           ends the flow after the ended state, so the function returns once the removal has happened.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async run() {
    //   for await (const s of latest(state, paused)) { await handle(s); if (s.end !== "RUNNING") return; }
    // }
    // ```
    /**
     * Posts paced updates until the analysis ends and the notification has been removed.
     *
     * @throws kotlinx.coroutines.CancellationException when the caller cancels the loop
     */
    suspend fun run() {
        combine(state, paused) { counts, isPaused -> counts.copy(paused = isPaused) }
            .transformWhile { snapshot ->
                emit(snapshot)
                snapshot.end == AnalysisEnd.RUNNING
            }
            .collectLatest { snapshot -> apply(snapshot) }
    }

    // What:     `private suspend fun apply(snapshot)` repeats the pacer's question until it gives a final answer.
    // Why:      A `Wait` answer means "ask again after the window"; a post of the final value is followed by the
    //           removal, so the loop continues after posting once the analysis has ended.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function apply(s) { for (;;) { const step = pacer.next(s, now()); /* post, skip, wait, remove */ } }
    // ```
    /** Applies pacer steps for [snapshot] until one of them ends the handling. */
    private suspend fun apply(snapshot: AnalysisSnapshot) {
        while (true) {
            /** The pacer's answer for the snapshot at the current time. */
            val step: AnalysisStep = pacer.next(snapshot = snapshot, nowMs = clockMs())
            when (step) {
                is AnalysisStep.Post -> {
                    post(step.content)
                    if (snapshot.end == AnalysisEnd.RUNNING) {
                        return
                    }
                }
                AnalysisStep.Skip -> return
                is AnalysisStep.Wait -> delay(step.millis)
                AnalysisStep.Remove -> {
                    remove()
                    return
                }
            }
        }
    }
}
