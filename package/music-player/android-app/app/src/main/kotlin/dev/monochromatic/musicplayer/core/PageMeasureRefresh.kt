// The page measure refresh as plain coroutine code: which sweep steps start a pass, where a pass starts reading, and
// how a pass walks the page in batches. The host only supplies the sweep steps, the visible track, and the reads, so
// JVM tests cover the whole decision chain without a device or a Compose runtime.

// What:     `package ...core` places the refresh beside the batch choice and the pass scheduler it composes.
// Why:      The refresh uses `measureNextBatch` and `runPassPerStep` without an import, and has no Android types.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports for the track type, the coroutine dispatcher, the context switch, and the step flow.
// Why:      The batch choice scans the whole page, so it runs on the default dispatcher, off the main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "../Track";
// ```
import dev.monochromatic.musicplayer.Track
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.withContext

// What:     `internal fun startPositionFor(tracks: List<Track>, firstVisibleUri: String?): Int` finds where a pass
//           begins reading the page.
// Why:      The tracks on screen are read first. An unknown or absent URI gives a negative position, which the batch
//           choice reads as the start of the page.
//
// In TS you'd write (pseudocode):
// ```ts
// const startPositionFor = (tracks: Track[], firstVisibleUri: string | null): number =>
//   tracks.findIndex((track) => track.uri === firstVisibleUri);
// ```
/** Position in [tracks] of the track whose URI is [firstVisibleUri], or -1 when none is, or when the URI is null. */
internal fun startPositionFor(tracks: List<Track>, firstVisibleUri: String?): Int =
    tracks.indexOfFirst { track -> track.uri == firstVisibleUri }

// What:     `private suspend fun readPageInBatches(...)` reads the page's unmeasured tracks batch by batch until none
//           is left, publishing each batch.
// Why:      A batch lands on screen as soon as it is read, and each track is attempted at most once per pass. The
//           known measures and the start position are read again before each batch, so the rows on screen at that
//           moment are read first.
//
// In TS you'd write (pseudocode):
// ```ts
// async function readPageInBatches(tracks, known, firstVisibleUri, read, publish, log, batchSize): Promise<void> {
//   const attempted = new Set<string>();
//   for (;;) {
//     const batch = nextBatch(tracks, known(), attempted, batchSize, startPositionFor(tracks, firstVisibleUri()));
//     if (batch.length === 0) return log(`page measures are complete attempted=${attempted.size}`);
//     batch.forEach((track) => attempted.add(track.uri));
//     publish(await read(batch));
//   }
// }
// ```
/**
 * Reads the tracks of [tracks] that still lack a measure, [batchSize] at a time, handing each batch to [publish].
 *
 * @param known returns the measures already published, read again before each batch
 * @param firstVisibleUri returns the URI of the first track on screen, or null, read again before each batch
 * @param read reads the measures of one batch of tracks
 * @param publish receives the measures of one batch
 * @param log receives one line when the page has no track left to read
 */
private suspend fun readPageInBatches(
    tracks: List<Track>,
    known: () -> Map<String, TrackMeasure>,
    firstVisibleUri: () -> String?,
    read: suspend (List<Track>) -> Map<String, TrackMeasure>,
    publish: (Map<String, TrackMeasure>) -> Unit,
    log: (String) -> Unit,
    batchSize: Int,
) {
    /** URIs already read in this pass, so a track without a peak is not read twice in one pass. */
    val attempted: MutableSet<String> = mutableSetOf()
    while (true) {
        /** Next tracks to read in this pass, in page order from the position the page is shown at. */
        // The choice scans the whole page, so it runs off the main thread: a page of thousands of tracks would
        // otherwise cost the main thread a full scan per batch.
        val batch: List<Track> = withContext(Dispatchers.Default) {
            measureNextBatch(
                page = tracks,
                known = known(),
                attempted = attempted,
                batchSize = batchSize,
                startAt = startPositionFor(tracks, firstVisibleUri()),
            )
        }
        if (batch.isEmpty()) {
            log("page measures are complete attempted=${attempted.size}")
            return
        }
        attempted += batch.map { track -> track.uri }
        publish(read(batch))
    }
}

// What:     `internal suspend fun pageMeasurePasses(...)` reads the page's measures once for the first sweep step and
//           once more after every pass that finished while newer steps had arrived.
// Why:      A cancel-and-restart per sweep step (what `collectLatest` does) discards the batch in flight and begins
//           again at the top, so a page of thousands of tracks got its measures only after the sweep ended. Here the
//           pass scheduler lets a running pass finish, and the steps that arrive meanwhile merge into one pass.
//
// In TS you'd write (pseudocode):
// ```ts
// async function pageMeasurePasses(steps, tracks, known, firstVisibleUri, read, publish, log, batchSize) {
//   await runPassPerStep(steps, () =>
//     readPageInBatches(tracks, known, firstVisibleUri, read, publish, log, batchSize));
// }
// ```
/**
 * Collects [steps] and, for each pass they ask for, reads the unmeasured tracks of [tracks] batch by batch.
 *
 * A pass that started is never cancelled by a newer step; see [runPassPerStep]. Each batch starts at the first track
 * on screen, which [firstVisibleUri] names at that moment. The call returns when [steps] completes and the last pass
 * has finished, and it throws what [read] or [publish] throws.
 *
 * @param steps values whose every emission asks for a pass, such as the count of tracks the sweep has finished
 * @param tracks library tracks of the selected page, each keyed by the URI that its measure is stored under
 * @param known returns the measures already published, read again before each batch
 * @param firstVisibleUri returns the URI of the first track on screen, or null when the list has not reported one
 * @param read reads the measures of one batch of tracks, such as from the duration cache and the decision cache
 * @param publish receives the measures of one batch
 * @param log receives one line at the end of each pass
 * @param batchSize count of tracks one batch holds
 */
internal suspend fun pageMeasurePasses(
    steps: Flow<Int>,
    tracks: List<Track>,
    known: () -> Map<String, TrackMeasure>,
    firstVisibleUri: () -> String?,
    read: suspend (List<Track>) -> Map<String, TrackMeasure>,
    publish: (Map<String, TrackMeasure>) -> Unit,
    log: (String) -> Unit,
    batchSize: Int = MEASURE_BATCH_SIZE,
) {
    runPassPerStep(steps) {
        readPageInBatches(
            tracks = tracks,
            known = known,
            firstVisibleUri = firstVisibleUri,
            read = read,
            publish = publish,
            log = log,
            batchSize = batchSize,
        )
    }
}
