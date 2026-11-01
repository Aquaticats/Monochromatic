// The Search service as the app holds it: one service per process, its answers as Compose state, and the calls that the
// player makes when its library changes. Search is available from the moment any track is known (D111): the scan feeds
// the service while the Search page, when open, keeps showing and refreshing the answers.

// What:     `package dev.monochromatic.musicplayer` places the host beside the controller and the service that use it.
// Why:      The pure service lives in `core`; this file adds the Android logging and the Compose state around it.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import dev.monochromatic.musicplayer.core.SearchAnswer
import dev.monochromatic.musicplayer.core.SearchService
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

/** Log tag of the Search host. */
private const val SEARCH_LOG_TAG: String = "MusicPlayerSearch"

// What:     `class SearchHost` owns the Search service and exposes the newest answer as observable state.
// Why:      The Search page recomposes when an answer arrives, whether it follows a keystroke or a refresh after a scan
//           batch, and the answer is produced on a worker thread.
//
// In TS you'd write (pseudocode):
// ```ts
// class SearchHost { answer = signal<SearchAnswer | null>(null); 
//   service = new SearchService((a) => this.answer.set(a)); }
// ```
/** Holds the process's Search service and the newest answer it delivered. */
class SearchHost : AutoCloseable {
    /** The newest answer, for the page to read; written on the service's worker thread. */
    var answer: SearchAnswer? by mutableStateOf(null)
        private set

    /** Sequence number of the first query of the current Search visit; answers to older queries are not shown. */
    private var visitStart: Long = 0L

    /** The newest answer of the current visit, or null before the first one; read it in composition. */
    val shownAnswer: SearchAnswer?
        get() = answer?.takeIf { delivered -> delivered.sequence >= visitStart }

    /** The service; its answers land in [answer]. */
    val service: SearchService = SearchService(
        deliver = { delivered -> answer = delivered },
        logger = { line -> Log.i(SEARCH_LOG_TAG, line) },
    )

    /** Builds replacement indexes one after another, off the thread that changed the library. */
    private val builder: ExecutorService = Executors.newSingleThreadExecutor { task ->
        Thread(task, "search-library-builder").also { thread -> thread.isDaemon = true }
    }

    /** Starts a Search visit: answers to queries asked before now are no longer shown (D66). */
    fun beginVisit() {
        visitStart = service.nextSequence
    }

    /** Asks the service to answer [text] on its worker thread; the answer arrives in [answer]. */
    fun ask(text: String) {
        service.query(text)
    }

    /**
     * Indexes a whole new library on a background thread: tracks were removed, another folder was picked, or the list
     * came from a scan that did not stream. [displayPaths] and [uris] are in library order.
     */
    fun libraryReplaced(displayPaths: List<String>, uris: List<String>) {
        builder.execute {
            service.replaceLibrary(displayPaths, uris)
            service.setComplete(true)
        }
    }

    /** Stops the builder and the service's threads. */
    override fun close() {
        builder.shutdownNow()
        service.close()
    }
}
