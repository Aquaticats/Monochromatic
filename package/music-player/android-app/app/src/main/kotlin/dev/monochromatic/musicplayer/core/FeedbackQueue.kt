// What:     `package dev.monochromatic.musicplayer.core` places the feedback queue beside the other pure logic.
// Why:      The queue holds transient message state with no Compose or Android dependency,
//           so the host JVM can verify it.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal const val FEEDBACK_ERROR_DURATION_MS: Long = 4_000L` is the default visible time of an error.
// Why:      The study used Compose's Short snackbar duration, which material3 1.5.0-beta01 maps to 4000 ms
//           (unchanged since 1.5.0-alpha27) before the platform accessibility adjustment.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_ERROR_DURATION_MS = 4000;
// ```
/** Default visible time of one error message in milliseconds, before any platform accessibility adjustment. */
internal const val FEEDBACK_ERROR_DURATION_MS: Long = 4_000L

// What:     `internal const val FEEDBACK_UNDO_DURATION_MS: Long = 4_000L` is the default visible time of Undo.
// Why:      The study showed the Undo message with the same Short duration as the error message.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_UNDO_DURATION_MS = 4000;
// ```
/** Default visible time of one Undo message in milliseconds, before any platform accessibility adjustment. */
internal const val FEEDBACK_UNDO_DURATION_MS: Long = 4_000L

// What:     `internal const val FEEDBACK_NOTICE_DURATION_MS: Long = 4_000L` is the default visible time of a notice.
// Why:     A notice reports progress that the user can watch in the row itself, so it uses the same short time as an
//           error.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_NOTICE_DURATION_MS = 4000;
// ```
/** Default visible time of one notice message in milliseconds, before any platform accessibility adjustment. */
internal const val FEEDBACK_NOTICE_DURATION_MS: Long = 4_000L

// What:     `internal const val FEEDBACK_MAX_DURATION_MS: Long = 60_000L` bounds any caller-supplied visible time.
// Why:      Feedback is transient by D83, and the bound keeps the expiry sum far from Long overflow.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_MAX_DURATION_MS = 60_000;
// ```
/** Upper bound in milliseconds for any visible time a caller supplies. */
internal const val FEEDBACK_MAX_DURATION_MS: Long = 60_000L

// What:     `internal val FEEDBACK_DISPLAY_ORDER: List<FeedbackKind>` lists kinds from top to bottom.
// Why:      D29 places the Undo message above the error message, whatever order they were shown in. The notice
//           kind (File details, re-analysis progress) sits between them: the actionable Undo leads, and a status
//           line never covers an error. Every declared kind is listed once, so none gets an accidental position.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_DISPLAY_ORDER: FeedbackKind[] = ["UNDO", "NOTICE", "ERROR"];
// ```
/** Top-to-bottom display order of feedback kinds; it holds every kind of [FeedbackKind] exactly once. */
internal val FEEDBACK_DISPLAY_ORDER: List<FeedbackKind> =
    listOf(FeedbackKind.UNDO, FeedbackKind.NOTICE, FeedbackKind.ERROR)

// What:     `enum class FeedbackKind` declares the three closed kinds of transient feedback.
// Why:      Replacement, ordering and default duration depend only on which kind a message is.
//
// In TS you'd write (pseudocode):
// ```ts
// type FeedbackKind = "ERROR" | "UNDO" | "NOTICE";
// ```
/** Kinds of transient feedback shown above the player content. */
enum class FeedbackKind(
    // What:     `val defaultDurationMs: Long` stores the visible time used when a show call names none.
    // Why:      Each kind keeps its own accepted duration next to the kind itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // defaultDurationMs: number;
    // ```
    /** Visible time in milliseconds used when a show call does not supply its own duration. */
    val defaultDurationMs: Long,
) {
    // What:     `ERROR` names a failed operation whose outcome text is the message.
    // Why:      Error messages are replaced by the next error and never offer Undo.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // ERROR = "ERROR";
    // ```
    /** A failed operation whose text states the outcome. */
    ERROR(FEEDBACK_ERROR_DURATION_MS),

    // What:     `UNDO` names a successful reversible operation with an Undo action.
    // Why:      The Undo message is a separate capability from the error message and can be visible with it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // UNDO = "UNDO";
    // ```
    /** A successful reversible operation that offers an Undo action. */
    UNDO(FEEDBACK_UNDO_DURATION_MS),

    // What:     `NOTICE` names an operation in progress or a neutral status, with no action.
    // Why:     A progress line such as re-analysing a track is neither an error nor an Undo, and must not replace
    //           either.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // NOTICE = "NOTICE";
    // ```
    /** A neutral status message about an operation in progress, which offers no action. */
    NOTICE(FEEDBACK_NOTICE_DURATION_MS),
}

// What:     `data class FeedbackMessage` holds one message's stable id, text, kind and optional action label.
// Why:      The overlay and the queue both read the same immutable value, and invalid text is rejected at construction.
//
// In TS you'd write (pseudocode):
// ```ts
// type FeedbackMessage = { id: string; text: string; kind: FeedbackKind; actionLabel: string | null };
// ```
/** One transient message, identified by a caller-owned id that dismissal and actions use. */
data class FeedbackMessage(
    // What:     `val id: String` is the caller's stable key for this message.
    // Why:      Dismissal and action callbacks name the message by this key, never by position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // id: string;
    // ```
    /** Stable key used to dismiss the message and to report its action. */
    val id: String,
    // What:     `val text: String` is the literal message shown to the user.
    // Why:      The overlay limits the visible lines; the text itself is never shortened by the model.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // text: string;
    // ```
    /** Message text shown to the user; never blank. */
    val text: String,
    // What:     `val kind: FeedbackKind` selects replacement, ordering and default duration.
    // Why:      The kind is the only property that decides which messages can be visible together.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // kind: FeedbackKind;
    // ```
    /** Kind that selects replacement, display order and default visible time. */
    val kind: FeedbackKind,
    // What:     `val actionLabel: String?` is the optional action button text, or null when none is offered.
    // Why:      Undo carries its action here, and any kind may carry one, so the overlay needs only one field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // actionLabel: string | null;
    // ```
    /** Label of the action button, or null when the message offers no action; never blank when present. */
    val actionLabel: String?,
) {
    // What:     `init` checks each field once when the message is constructed.
    // Why:      A blank id cannot be dismissed reliably and a blank text would show an empty message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (id.trim() === "") throw new Error("Feedback message id must not be blank.");
    // ```
    init {
        require(id.isNotBlank()) { "Feedback message id must not be blank." }
        require(text.isNotBlank()) { "Feedback message text must not be blank." }
        require(actionLabel == null || actionLabel.isNotBlank()) {
            "Feedback action label must not be blank when present."
        }
    }
}

// What:     `private data class FeedbackEntry` pairs one message with the clock value and duration it was shown with.
// Why:      Expiry needs the show time and duration, which the public message does not carry.
//
// In TS you'd write (pseudocode):
// ```ts
// type FeedbackEntry = { message: FeedbackMessage; shownAtMs: number; durationMs: number };
// ```
/** One visible message together with the clock value at which it appeared and how long it stays. */
private data class FeedbackEntry(
    // What:     `val message: FeedbackMessage` is the message presented while the entry is visible.
    // Why:      The queue returns messages to the overlay and keeps timing beside them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // message: FeedbackMessage;
    // ```
    /** Message presented while this entry is visible. */
    val message: FeedbackMessage,
    // What:     `val shownAtMs: Long` is the caller's clock value when the entry was shown.
    // Why:      Elapsed time is computed from this value and the clock value the caller passes to expiry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // shownAtMs: number;
    // ```
    /** Caller clock value in milliseconds at which this entry was shown. */
    val shownAtMs: Long,
    // What:     `val durationMs: Long` is how long the entry stays visible.
    // Why:      The wiring step can pass a platform-adjusted duration per show call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // durationMs: number;
    // ```
    /** Visible time of this entry in milliseconds. */
    val durationMs: Long,
) {
    // What:     `fun isExpiredAt(nowMs: Long): Boolean` reports whether the full visible time has elapsed.
    // Why:      Expiry is the single boundary rule the queue applies, so tests can pin it exactly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // isExpiredAt(nowMs: number): boolean { return nowMs - this.shownAtMs >= this.durationMs; }
    // ```
    /** Reports true once the elapsed time since the entry was shown reaches its duration. */
    fun isExpiredAt(nowMs: Long): Boolean {
        // What:     `val elapsedMs: Long` is the time since the entry was shown.
        // Why:      A clock value before the show time gives a negative elapsed time, which never expires.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const elapsedMs = nowMs - this.shownAtMs;
        // ```
        /** Milliseconds between the show time and the supplied clock value, negative if the clock is earlier. */
        val elapsedMs: Long = nowMs - shownAtMs
        return elapsedMs >= durationMs
    }
}

// What:     `class FeedbackQueue` is an immutable set of visible messages with pure transition functions.
// Why:      The player keeps one value and replaces it on each event, so every state is reproducible and testable.
//
// In TS you'd write (pseudocode):
// ```ts
// class FeedbackQueue { private constructor(private readonly entries: FeedbackEntry[]) {} }
// ```
/**
 * Immutable set of visible transient messages; every transition returns a new queue.
 * At most one message per kind and at most one message per id is kept at any time.
 */
class FeedbackQueue private constructor(
    // What:     `private val entries: List<FeedbackEntry>` holds the visible entries.
    // Why:      A read-only list cannot be mutated by a caller that holds an older queue value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly entries: readonly FeedbackEntry[];
    // ```
    /** Visible entries in the order they were last shown. */
    private val entries: List<FeedbackEntry>,
) {
    // What:     `fun show(...)` returns a queue where the message is the only one of its kind and its id.
    // Why:      A new message replaces the visible one of the same kind, and a repeated id restarts its timer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // show(message, nowMs, durationMs = defaults[message.kind]): FeedbackQueue
    // ```
    /**
     * Returns a queue in which [message] replaces any entry of its kind or with its id and is visible from [nowMs].
     * Throws [IllegalArgumentException] when [durationMs] is outside 1 through [FEEDBACK_MAX_DURATION_MS].
     */
    fun show(
        message: FeedbackMessage,
        nowMs: Long,
        durationMs: Long = message.kind.defaultDurationMs,
    ): FeedbackQueue {
        require(durationMs in 1L..FEEDBACK_MAX_DURATION_MS) {
            "Feedback duration must be between 1 and $FEEDBACK_MAX_DURATION_MS ms, was $durationMs."
        }
        // What:     `val kept: List<FeedbackEntry>` drops every entry the new message supersedes.
        // Why:      Same-kind replacement and id reuse both remove the older entry, so the invariant holds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kept = entries.filter(e => e.message.id !== message.id && e.message.kind !== message.kind);
        // ```
        /** Entries that the new message does not supersede, in their existing order. */
        val kept: List<FeedbackEntry> = entries.filterNot { entry ->
            entry.message.id == message.id || entry.message.kind == message.kind
        }
        return FeedbackQueue(kept + FeedbackEntry(message = message, shownAtMs = nowMs, durationMs = durationMs))
    }

    // What:     `fun dismiss(id: String)` removes the entry with that id and leaves every other entry as it is.
    // Why:      Dismissal by id cannot remove a newer message that replaced the one the user saw.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dismiss(id: string): FeedbackQueue { return new FeedbackQueue(this.entries.filter(e => e.message.id !== id)); }
    // ```
    /** Returns a queue without the message with [id]; an unknown id leaves the visible messages unchanged. */
    fun dismiss(id: String): FeedbackQueue = FeedbackQueue(entries.filterNot { entry -> entry.message.id == id })

    // What:     `fun expire(nowMs: Long)` removes every entry whose visible time has fully elapsed.
    // Why:      The caller supplies the clock, so the queue never reads time and expiry stays deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expire(nowMs: number): FeedbackQueue {
    //   return new FeedbackQueue(this.entries.filter(e => !e.isExpiredAt(nowMs)));
    // }
    // ```
    /** Returns a queue without the entries whose visible time has elapsed at [nowMs]. */
    fun expire(nowMs: Long): FeedbackQueue = FeedbackQueue(entries.filterNot { entry -> entry.isExpiredAt(nowMs) })

    // What:     `fun visible(): List<FeedbackMessage>` returns the messages in top-to-bottom display order.
    // Why:      The overlay draws the list as given, so the display order is decided once here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // visible(): FeedbackMessage[] { return sortBy(this.entries, kindRank).map(e => e.message); }
    // ```
    /** Returns the visible messages in [FEEDBACK_DISPLAY_ORDER], independent of the order they were shown in. */
    fun visible(): List<FeedbackMessage> =
        entries.sortedBy { entry -> FEEDBACK_DISPLAY_ORDER.indexOf(entry.message.kind) }.map { entry -> entry.message }

    // What:     `fun nextExpiryMs(): Long?` returns the earliest clock value at which an entry expires.
    // Why:      The wiring step can wait for one delay instead of polling, and null means nothing needs a timer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // nextExpiryMs(): number | null { ... }
    // ```
    /** Returns the earliest expiry clock value among visible entries, or null when none is visible. */
    fun nextExpiryMs(): Long? = entries.minOfOrNull { entry -> entry.shownAtMs + entry.durationMs }

    // What:     `companion object` holds the empty queue value.
    // Why:      Callers start from one shared empty value rather than constructing their own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // static readonly EMPTY = new FeedbackQueue([]);
    // ```
    /** Holds the shared empty queue. */
    companion object {
        // What:     `val EMPTY: FeedbackQueue` is a queue with no visible messages.
        // Why:      Starting every screen from the same value keeps the initial state identical.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // static readonly EMPTY: FeedbackQueue = new FeedbackQueue([]);
        // ```
        /** Queue with no visible messages. */
        val EMPTY: FeedbackQueue = FeedbackQueue(emptyList())
    }
}
