// Runs one pass of work per step of a changing value, without a step ever cancelling a pass that is running. The page
// measure refresh uses it so the analysis sweep reporting progress ten times a second no longer throws away the batch
// being read (a cold 10,000 track folder measured 38 batches in the 26 seconds of the sweep, and the rest after it).
// This file is plain coroutine code with no Android types, so JVM tests cover it directly.

// What:     `package ...core` places the helper beside the other platform-independent list and measure logic.
// Why:      The helper has no Android types, so the JVM tests reach it without a device.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.conflate

// What:     `suspend fun runPassPerStep(steps: Flow<Int>, pass: suspend () -> Unit)` runs [pass] once for the first
//           step and once more after every pass that finished while newer steps had arrived.
// Why:      A cancel-and-restart per step (what `collectLatest` or an effect keyed on the step does) restarts the pass
//           from the top and discards the batch in flight, so a pass that takes longer than the gap between steps never
//           gets to publish. Here a running pass always finishes, the steps that arrive meanwhile merge into one, and
//           that one starts the next pass, so no step is lost and none starts a second pass at the same time.
//
// In TS you'd write (pseudocode):
// ```ts
// async function runPassPerStep(steps: AsyncIterable<number>, pass: () => Promise<void>): Promise<void> {
//   // Sequential for-await: while `pass` runs the producer keeps only the newest step (conflated).
//   for await (const _step of conflate(steps)) await pass();
// }
// ```
/**
 * Collects [steps] and runs [pass] for each one, one pass at a time, never cancelling a pass that has started.
 *
 * Steps that arrive while a pass runs are conflated into one, so a pass runs again at most once after each pass, and
 * only when some step arrived during it. The call returns when [steps] completes and the last pass has finished, and
 * it throws what [pass] throws; cancelling the caller cancels the running pass.
 *
 * @param steps values whose every emission asks for a pass, such as the count of tracks the sweep has finished
 * @param pass work that one step asks for, which can take much longer than the gap between steps
 */
suspend fun runPassPerStep(steps: Flow<Int>, pass: suspend () -> Unit) {
    steps.conflate().collect { _ -> pass() }
}
