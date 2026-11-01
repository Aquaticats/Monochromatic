// The update rate limit, the duplicate skip and the end-of-analysis rule for the analysis notification (D107).

// What:     `package ...core` places the pacer beside the progress model it paces.
// Why:      The decision depends only on the snapshot and a clock reading, so JVM tests cover it with a fake clock.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val ANALYSIS_UPDATE_MIN_INTERVAL_MS: Long = 500L` is the least time between ordinary posts.
// Why:      D107 asks for ordinary updates at least 500 ms apart; the platform sheds updates above about five a
//           second, and two a second is plenty for a bar that moves over minutes.
//
// In TS you'd write (pseudocode):
// ```ts
// export const ANALYSIS_UPDATE_MIN_INTERVAL_MS = 500;
// ```
/** Least number of milliseconds between two ordinary posts of the analysis notification. */
const val ANALYSIS_UPDATE_MIN_INTERVAL_MS: Long = 500L

// What:     `enum class AnalysisEnd` says whether the analysis is still running and how it ended.
// Why:      Finishing and giving up end differently: a finished analysis posts its final value first, while an
//           abandoned one (stopped, or no library to read) only removes the notification.
//
// In TS you'd write (pseudocode):
// ```ts
// type AnalysisEnd = "RUNNING" | "COMPLETED" | "ABANDONED";
// ```
/** How the analysis ended, or that it has not ended yet. */
enum class AnalysisEnd {
    /** The analysis is still running, so the notification stays posted. */
    RUNNING,

    /** Every track was visited, so the final value is posted and then the notification is removed. */
    COMPLETED,

    /** The analysis stopped early or found nothing to read, so the notification is only removed. */
    ABANDONED,
}

// What:     `data class AnalysisSnapshot(...)` is the whole input of one pacing decision.
// Why:      One immutable value keeps the counts, the pause flag and the end state consistent for a decision.
//
// In TS you'd write (pseudocode):
// ```ts
// type AnalysisSnapshot = { done: number; total: number; paused: boolean; end: AnalysisEnd };
// ```
/** The latest analysis state that the notification should reflect. */
data class AnalysisSnapshot(
    /** Tracks visited so far. */
    val done: Int,
    /** Tracks the sweep will visit, or zero while unknown. */
    val total: Int,
    /** Whether the user has paused the analysis. */
    val paused: Boolean,
    /** Whether the analysis is running or how it ended. */
    val end: AnalysisEnd,
)

// What:     `sealed interface AnalysisStep` is what the caller must do after asking the pacer.
// Why:      A closed set of answers forces the caller to handle each one, including the trailing wait that keeps
//           the newest value from being lost when it arrives inside the rate limit window.
//
// In TS you'd write (pseudocode):
// ```ts
// type AnalysisStep = { kind: "post"; content } | { kind: "skip" } | { kind: "wait"; millis } | { kind: "remove" };
// ```
/** The action the caller performs after one pacing decision. */
sealed interface AnalysisStep {
    /** Post [content] now; the pacer has already recorded the post. */
    data class Post(
        /** What the notification must show. */
        val content: AnalysisNotificationContent,
    ) : AnalysisStep

    /** Post nothing: the notification already looks the same. */
    data object Skip : AnalysisStep

    /** Wait [millis] and ask again with the newest snapshot, because the rate limit window is still open. */
    data class Wait(
        /** Milliseconds until the window closes, always positive. */
        val millis: Long,
    ) : AnalysisStep

    /** Remove the notification; the analysis has ended. */
    data object Remove : AnalysisStep
}

// What:     `class AnalysisUpdatePacer(minIntervalMs)` remembers the last post and decides each next one.
// Why:      The rules are: skip a post that looks the same; post a pause or resume change at once, because a
//           tap needs an immediate answer and no progress event arrives while paused; delay any other change
//           until the window since the last post has passed, then post the newest value; always post the final
//           value of a completed analysis; and remove the notification when the analysis ends.
//
// In TS you'd write (pseudocode):
// ```ts
// class AnalysisUpdatePacer { markPosted(content, nowMs): void; next(snapshot, nowMs): AnalysisStep; }
// ```
/**
 * Decides when the analysis notification is posted again. It keeps the last posted content and time, so it
 * belongs to one analysis run and must be used from one coroutine at a time.
 *
 * @param minIntervalMs least milliseconds between two ordinary posts
 */
class AnalysisUpdatePacer(private val minIntervalMs: Long = ANALYSIS_UPDATE_MIN_INTERVAL_MS) {
    /** Content of the last post, or null before the first post. */
    private var lastContent: AnalysisNotificationContent? = null

    /** Clock reading of the last post in milliseconds. */
    private var lastPostMs: Long = 0L

    // What:     `fun markPosted(...)` records a post that the pacer did not decide.
    // Why:      The foreground service must post its first frame itself while it starts, so the pacer learns of it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // markPosted(content, nowMs) { this.lastContent = content; this.lastPostMs = nowMs; }
    // ```
    /**
     * Records that [content] was posted at [nowMs] by someone other than this pacer.
     *
     * @param content what the posted notification shows
     * @param nowMs monotonic clock reading at the post
     */
    fun markPosted(content: AnalysisNotificationContent, nowMs: Long) {
        lastContent = content
        lastPostMs = nowMs
    }

    // What:     `fun next(snapshot, nowMs): AnalysisStep` answers what to do for the latest snapshot.
    // Why:      Asking again after a `Wait` with a newer snapshot is what makes the last value always arrive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // next(snapshot, nowMs): AnalysisStep { /* end, skip, pause, window */ }
    // ```
    /**
     * Returns the next action for [snapshot]. A returned [AnalysisStep.Post] is recorded before it is returned.
     *
     * @param snapshot latest analysis state
     * @param nowMs monotonic clock reading
     * @return the step the caller performs
     */
    fun next(snapshot: AnalysisSnapshot, nowMs: Long): AnalysisStep {
        if (snapshot.end == AnalysisEnd.ABANDONED) {
            return AnalysisStep.Remove
        }
        if (snapshot.end == AnalysisEnd.COMPLETED) {
            return nextAfterCompletion(snapshot, nowMs)
        }
        /** What the notification would show for this snapshot. */
        val content: AnalysisNotificationContent = analysisNotificationContent(
            done = snapshot.done,
            total = snapshot.total,
            paused = snapshot.paused,
        )
        /** Content of the last post, or null when nothing was posted yet. */
        val previous: AnalysisNotificationContent = lastContent ?: return post(content, nowMs)
        if (content == previous) {
            return AnalysisStep.Skip
        }
        if (content.paused != previous.paused) {
            return post(content, nowMs)
        }
        /** Milliseconds left in the window that follows the last post. */
        val remaining: Long = lastPostMs + minIntervalMs - nowMs
        if (remaining > 0L) {
            return AnalysisStep.Wait(millis = remaining)
        }
        return post(content, nowMs)
    }

    // What:     `private fun nextAfterCompletion(...)` posts the final value once, then removes the notification.
    // Why:      The last visible value should be the complete one, and the rate limit never delays it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function nextAfterCompletion(snapshot, nowMs) { return finalDiffers ? post(final) : remove; }
    // ```
    /** Returns the final post when the last post differs from the complete value, and the removal otherwise. */
    private fun nextAfterCompletion(snapshot: AnalysisSnapshot, nowMs: Long): AnalysisStep {
        if (snapshot.total <= 0) {
            return AnalysisStep.Remove
        }
        /** The complete value: every track visited, and not paused because the analysis is over. */
        val finalContent: AnalysisNotificationContent = analysisNotificationContent(
            done = snapshot.total,
            total = snapshot.total,
            paused = false,
        )
        if (finalContent == lastContent) {
            return AnalysisStep.Remove
        }
        return post(finalContent, nowMs)
    }

    // What:     `private fun post(...)` records a post and wraps it as a step.
    // Why:      Recording inside the decision keeps the pacer consistent with what the caller is about to do.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function post(content, nowMs): AnalysisStep {
    //   this.markPosted(content, nowMs);
    //   return { kind: "post", content };
    // }
    // ```
    /** Records [content] as posted at [nowMs] and returns the post step. */
    private fun post(content: AnalysisNotificationContent, nowMs: Long): AnalysisStep {
        markPosted(content = content, nowMs = nowMs)
        return AnalysisStep.Post(content = content)
    }
}
