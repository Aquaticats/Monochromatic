// What:     `package dev.monochromatic.musicplayer` puts this class beside `BrainPlayer`, which uses it.
// Why:      The rule is plain Kotlin with no Android types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `class CoalescedTrigger(schedule, action)` runs `action` once for any number of `request()` calls
//           made before the scheduled task runs. `schedule` is a function that takes a task and queues it for
//           later (a looper post).
// Why:      The brain announces a change several times per track change (load, artist tag, position), and each
//           announcement used to queue its own full media session refresh. The refresh reads the brain when it runs, so
//           one refresh after a burst shows the same final state.
// Gotcha:   The latch clears before `action` runs, so a request made by the action itself schedules another run.
//
// In TS you'd write (pseudocode):
// ```ts
// class CoalescedTrigger {
//   private pending = false;
//   constructor(private schedule: (task: () => void) => void, private action: () => void) {}
//   request(): void {
//     if (this.pending) return;
//     this.pending = true;
//     this.schedule(() => { this.pending = false; this.action(); });
//   }
// }
// ```
/** Runs [action] once through [schedule] for each burst of [request] calls. */
class CoalescedTrigger(
    /** Queues a task to run later, on the thread that owns the state [action] reads. */
    private val schedule: (() -> Unit) -> Unit,
    /** Work to run once per burst. */
    private val action: () -> Unit,
) {
    /** Whether a scheduled run has not started yet. */
    private var pending: Boolean = false

    /** Asks for [action] to run; does nothing while a run is already scheduled. */
    fun request() {
        if (pending) {
            return
        }
        pending = true
        schedule {
            pending = false
            action()
        }
    }
}
