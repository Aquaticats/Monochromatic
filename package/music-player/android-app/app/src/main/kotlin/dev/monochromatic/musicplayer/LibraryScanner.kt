// Runs one library scan through the ingest: the scan's batches are published to the player and to Search while it runs,
// and the finished library comes back ready to install. The source (MediaStore or a picked folder) and the way a
// published library is applied are parameters, so the load, the picked-folder reload and the tests share one path.

// What:     `package dev.monochromatic.musicplayer` places the scanner beside the service that owns it.
// Why:      It ties the pure ingest to the controller, the Search host and the main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.util.Log
import dev.monochromatic.musicplayer.core.BuiltLibrary
import dev.monochromatic.musicplayer.core.FinalLibrary
import dev.monochromatic.musicplayer.core.LibraryFeed
import dev.monochromatic.musicplayer.core.LibraryIngest
import java.util.concurrent.Executors
import kotlin.coroutines.cancellation.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext

/** Log tag of the scanner. */
private const val SCANNER_LOG_TAG: String = "MusicPlayerScan"

// What:     `class ScanOutcome(...)` is what a finished scan hands back.
// Why:      Normally the ingest has the whole library built; if a source reported fewer tracks than it returned, the
//           returned list is the truth and the caller installs it the plain way.
//
// In TS you'd write (pseudocode):
// ```ts
// type ScanOutcome = { ingest: LibraryIngest; final: FinalLibrary | null; returned: Track[] };
// ```
/** The result of one scan: the ingest and its complete library, or the source's own list when they disagreed. */
class ScanOutcome(
    /** The ingest, whose [LibraryIngest.complete] the caller calls after installing [final]. */
    val ingest: LibraryIngest,
    /** The complete built library, or null when the ingest did not carry every track the source returned. */
    val final: FinalLibrary?,
    /** The sorted list the source returned, used when [final] is null. */
    val returned: List<Track>,
)

// What:     `class LibraryScanner(...)` publishes a scan to the player and to Search while it runs.
// Why:      The player must not wait for a scan batch, and Search must see every track once the player shows it (D111).
//
// In TS you'd write (pseudocode):
// ```ts
// const outcome = await scanner.scan(source, (built) => controller.reconcileBuilt(built, false));
// ```
/** Runs scans through [LibraryIngest] for one controller and one Search host. */
class LibraryScanner(
    /** The controller whose library the scan fills; read for the playing URI. */
    private val controller: PlayerController,
    /** The Search host that the scan feeds. */
    private val searchHost: SearchHost,
) {
    /** Marks the scan that may still install; a newer scan replaces it, which drops the older one's installs. */
    @Volatile
    private var current: Any? = null

    /** Drops the installs of the running scan, because the service is going away. */
    fun supersede() {
        current = null
    }

    /**
     * Runs [source] and publishes what it finds through [publish] on the main thread, at most as often as the ingest
     * allows, then returns the complete library off the main thread.
     *
     * [source] receives the callback to report each batch to and returns the source's own sorted list at the end.
     */
    suspend fun scan(
        source: suspend (suspend (List<Track>) -> Unit) -> List<Track>,
        publish: (BuiltLibrary) -> Unit,
    ): ScanOutcome {
        /** Identity of this scan, which only the newest scan holds. */
        val token = Any()
        current = token
        /** Number of the empty library in the Search service that this scan fills. */
        val generation: Long = searchHost.service.replaceLibrary(emptyList(), null)
        /** Feed that appends the published tracks to that library. */
        val feed = LibraryFeed(searchHost.service, generation)
        /** The single thread on which the ingest sorts, builds and publishes. */
        val executor = Executors.newSingleThreadScheduledExecutor { task ->
            Thread(task, "library-ingest").also { thread -> thread.isDaemon = true }
        }
        /** Ingest that turns the scan's batches into published libraries. */
        val ingest = LibraryIngest(
            feed = feed,
            install = { built -> installOnMain(token, built, publish) },
            executor = executor,
            playingUri = { controller.loadedUriSnapshot() },
            logger = { line -> Log.i(SCANNER_LOG_TAG, line) },
        )
        /** Whether the ingest was handed to the caller, which then owns ending it. */
        var handedOver = false
        try {
            /** The source's own sorted list, returned when it finishes. */
            val returned: List<Track> = source { added -> ingest.accept(added) }
            ensureCurrent(token)
            if (ingest.found != returned.size) {
                Log.w(SCANNER_LOG_TAG, "scan reported ${ingest.found} tracks but returned ${returned.size}")
                handedOver = true
                ingest.cancel()
                return ScanOutcome(ingest, null, returned)
            }
            /** The complete library, built off the main thread. */
            val final: FinalLibrary = withContext(Dispatchers.Default) { ingest.finish() }
            ensureCurrent(token)
            handedOver = true
            return ScanOutcome(ingest, final, returned)
        } finally {
            if (!handedOver) {
                ingest.cancel()
            }
        }
    }

    /** Throws when a newer scan replaced the scan that holds [token], which ends the older one. */
    private fun ensureCurrent(token: Any) {
        if (current !== token) {
            throw CancellationException("The scan was superseded by a newer one.")
        }
    }

    /** Applies a published library on the main thread and waits for it; false when the scan was superseded. */
    private fun installOnMain(token: Any, built: BuiltLibrary, publish: (BuiltLibrary) -> Unit): Boolean =
        runBlocking(Dispatchers.Main) {
            if (current === token) {
                publish(built)
                true
            } else {
                false
            }
        }
}
