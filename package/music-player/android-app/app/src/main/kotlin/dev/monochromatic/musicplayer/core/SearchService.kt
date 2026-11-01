// The Search service: the index behind a read-write lock, a worker thread that answers queries, a warm-up thread, and
// the refresh signal. A scan that adds tracks appends batches while the open Search page keeps answering keystrokes
// (D111, D112). The write lock is taken only for the push; batch preparation and growth run before it; the writer polls
// with `tryLock` so it never queues ahead of readers; and queries are coalesced so a burst of keystrokes costs one
// ranking. The service has no Android types, so the JVM tests run it with real threads.

// What:     `package dev.monochromatic.musicplayer.core` places the service beside the index it owns.
// Why:      The tests drive it with plain threads and latches, and the app wraps it for the main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.locks.LockSupport
import java.util.concurrent.locks.ReentrantLock
import java.util.concurrent.locks.ReentrantReadWriteLock

/** How long the appender sleeps between attempts to take the write lock, so readers keep entering meanwhile. */
private const val WRITE_POLL_NANOS: Long = 200_000L

/** How long the appender polls before it waits for the lock, so a stream of readers cannot starve it. */
private const val WRITE_PATIENCE_NANOS: Long = 50_000_000L

/** The fewest time between two refresh answers: a scan appends many batches a second and one repaint is enough. */
const val SEARCH_REFRESH_INTERVAL_MILLIS: Long = 120L

/** How long the worker sits idle before it refreshes single-character answers, which holds the read lock. */
const val SEARCH_WARM_DELAY_MILLIS: Long = 250L

/** Nanoseconds in a millisecond, for reporting latencies. */
private const val NANOS_PER_MILLI: Double = 1_000_000.0

/** Wait of an idle worker with no refresh due, which is effectively forever. */
private const val IDLE_WAIT_NANOS: Long = 3_600L * 1_000_000_000L

/** Append holds of the write lock above this many nanoseconds are logged as slow. */
private const val SLOW_PUSH_NANOS: Long = 5_000_000L

// What:     `class SearchAnswer(...)` is one answer delivered to the page.
// Why:      The page needs the sequence to drop answers to an older query, the refresh flag to update in place
//           instead of scrolling to the top, and the completeness flag because a partial scan cannot establish
//           "no match" (D69).
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchAnswer = { sequence: number; query: string; state: SearchState; knownTracks: number;
//   complete: boolean; refresh: boolean };
// ```
/** One answer to a query, with what the page needs to show it truthfully. */
class SearchAnswer(
    /** Sequence number [SearchService.query] returned for the query this answers. */
    val sequence: Long,
    /** The query text. */
    val query: String,
    /** The ranked state. */
    val state: SearchState,
    /** How many tracks the index held when it answered. */
    val knownTracks: Int,
    /** Whether the scan that feeds the index has finished. */
    val complete: Boolean,
    /** True when this re-answers the same query because tracks were appended. */
    val refresh: Boolean,
    /** Number of the library the index described when it answered. */
    val generation: Long,
    /** Time the ranking took, in milliseconds. */
    val elapsedMillis: Double,
)

/** A message to the query worker. */
private sealed interface Request {
    /** A new query with its sequence number. */
    class Query(val sequence: Long, val text: String) : Request

    /** Tracks were appended, paths were replaced, or the scan finished. */
    data object Refresh : Request

    /** Stop the worker. */
    data object Stop : Request
}

// What:     `class SearchService(...)` owns the index and the threads that read and extend it.
// Why:      The window thread must not wait for ranking, and a scan that adds tracks must not make a keystroke wait.
//
// In TS you'd write (pseudocode):
// ```ts
// const search = new SearchService((answer) => render(answer));
// search.query("cam");                  // answered on a worker, delivered to the callback
// search.append(generation, paths, uris) // push under a short lock, then re-answer the latest query
// ```
/**
 * The shared index, its worker threads and the refresh signal.
 *
 * @param deliver Receives each answer on the worker thread; the app forwards it to the main thread.
 * @param logger Receives one line per answer, per slow push and per failure.
 */
class SearchService(
    private val deliver: (SearchAnswer) -> Unit,
    private val logger: (String) -> Unit = {},
    private val refreshIntervalMillis: Long = SEARCH_REFRESH_INTERVAL_MILLIS,
    private val warmDelayMillis: Long = SEARCH_WARM_DELAY_MILLIS,
) : AutoCloseable {
    /** The index and the lock around it: searches read, pushes write. */
    private val lock = ReentrantReadWriteLock()

    /** The index; replaced under the write lock when a new library starts. */
    private var index: SearchIndex = SearchIndex(0L)

    /** Serializes appenders, so batch positions stay contiguous and their pre-grown storage stays valid. */
    private val appendOrder = ReentrantLock()

    /** Number of the current library. */
    private val libraryNumber = AtomicLong(0L)

    /** Source of query sequence numbers. */
    private val sequence = AtomicLong(0L)

    /** Whether the scan that feeds the index has finished. */
    private val complete = AtomicBoolean(false)

    /** Longest time an append held the write lock, in nanoseconds, since the service started. */
    private val longestPush = AtomicLong(0L)

    /** Total time appends held the write lock, in nanoseconds. */
    private val totalPush = AtomicLong(0L)

    /** Number of appends that held the write lock. */
    private val pushCount = AtomicLong(0L)

    /** Queue to the query worker. */
    private val requests = LinkedBlockingQueue<Request>()

    /** Queue to the warm-up thread; any message restarts its quiet wait. */
    private val warmSignals = LinkedBlockingQueue<Request>()

    /** The query worker thread. */
    private val worker: Thread = Thread({ work() }, "search-worker").also { thread ->
        thread.isDaemon = true
        thread.start()
    }

    /** The warm-up thread. */
    private val warmer: Thread = Thread({ warm() }, "search-warmer").also { thread ->
        thread.isDaemon = true
        thread.start()
    }

    /** Number of the library the service describes now; a new library starts a new number. */
    val generation: Long get() = libraryNumber.get()

    /** Longest write-lock hold of an append in milliseconds, which D111 bounds. */
    val longestPushMillis: Double get() = longestPush.get() / NANOS_PER_MILLI

    /** Sequence number the next query will get. */
    val nextSequence: Long get() = sequence.get() + 1

    /** Mean write-lock hold of an append in milliseconds. */
    val meanPushMillis: Double get() = totalPush.get() / NANOS_PER_MILLI / maxOf(1L, pushCount.get())

    /** Number of tracks the index holds now. */
    val trackCount: Int get() = readIndex { current -> current.trackCount }

    // What:     `fun query(text: String): Long` asks for an answer to a query on the worker thread.
    // Why:      Ranking never runs on the caller's thread, and only the newest query matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const sequence = search.query("cam");
    // ```
    /** Queues [text] for the worker and returns its sequence number; the answer arrives through the callback. */
    fun query(text: String): Long {
        /** Sequence number of this query. */
        val number: Long = sequence.incrementAndGet()
        requests.put(Request.Query(number, text))
        return number
    }

    // What:     `fun append(expectedGeneration: Long, paths: List<String>, uris: List<String>?): Boolean` adds tracks.
    // Why:      This is the scan's push: normalize and key outside the lock, enlarge storage outside it, then hold the
    //           write lock only for the copy. A feeder of a replaced library is refused.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // search.append(generation, ["Artist/Song.flac"], ["content://..."]);
    // ```
    /** Appends tracks to library [expectedGeneration], or returns false when that library was replaced. */
    fun append(expectedGeneration: Long, paths: List<String>, uris: List<String>?): Boolean {
        appendOrder.lock()
        try {
            if (libraryNumber.get() != expectedGeneration) {
                logger("search append refused: library $expectedGeneration was replaced by ${libraryNumber.get()}")
                return false
            }
            /** Start of the chunk being appended. */
            var from = 0
            while (from < paths.size) {
                /** End of this chunk. */
                val to: Int = minOf(paths.size, from + MAX_BATCH_TRACKS)
                /** Batch prepared without any lock. */
                val batch: PreparedBatch = prepareBatch(paths.subList(from, to), uris?.subList(from, to))
                /** Enlarged storage made without any lock; the appender is the only writer. */
                val room: AppendRoom = readIndex { current -> current.prepareRoom(batch) }
                pushUnderWriteLock { current -> current.append(batch, room) }
                from = to
            }
            signalGrew()
            return true
        } finally {
            appendOrder.unlock()
        }
    }

    // What:     `fun replaceLibrary(paths: List<String>, uris: List<String>?): Long` starts a new library.
    // Why:      A library that cannot be expressed as an append (tracks were removed, another folder was picked) is
    //           indexed on the side and swapped in, so the old one keeps answering until the new one is ready.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const generation = search.replaceLibrary(paths, uris);
    // ```
    /** Indexes [paths] as a new library, swaps it in and returns its number. */
    fun replaceLibrary(paths: List<String>, uris: List<String>?): Long {
        appendOrder.lock()
        try {
            /** Number of the library being built. */
            val number: Long = libraryNumber.get() + 1
            /** The new index, built on the side. */
            val fresh = SearchIndex(number)
            fresh.appendPaths(paths, uris)
            lock.writeLock().lock()
            try {
                index = fresh
                libraryNumber.set(number)
            } finally {
                lock.writeLock().unlock()
            }
            complete.set(false)
            logger("search library replaced generation=$number tracks=${paths.size}")
            signalGrew()
            return number
        } finally {
            appendOrder.unlock()
        }
    }

    // What:     `fun replacePaths(expectedGeneration: Long, newPaths: List<String>): Boolean` swaps display paths.
    // Why:      A scan can shrink the prefix every path shares, which changes the paths the pages show and the folder
    //           table, not the titles, so the buckets and their cached answers need no rebuild.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // search.replacePaths(generation, relativeNames);
    // ```
    /** Replaces the display path of every track in library [expectedGeneration]; false when it was replaced. */
    fun replacePaths(expectedGeneration: Long, newPaths: List<String>): Boolean {
        appendOrder.lock()
        try {
            if (libraryNumber.get() != expectedGeneration) {
                return false
            }
            /** New table prepared without any lock, or null when a title would change. */
            val replacement: PathReplacement? = readIndex { current -> current.prepareReplacement(newPaths) }
            if (replacement == null) {
                logger("search paths changed a title; indexing the library again")
                /** URIs of the tracks the index holds, to index them again under the new paths. */
                val held: List<String> = readIndex { current ->
                    current.uris.take(current.trackCount).map { uri -> uri.orEmpty() }
                }
                replaceLibrary(newPaths, held)
                complete.set(false)
                return true
            }
            pushUnderWriteLock { current -> current.applyReplacement(replacement) }
            signalGrew()
            return true
        } finally {
            appendOrder.unlock()
        }
    }

    /** Records whether the scan that feeds the index has finished, and re-answers the open query. */
    fun setComplete(finished: Boolean) {
        complete.set(finished)
        signalGrew()
    }

    /** Stops both threads and waits for them. */
    override fun close() {
        requests.put(Request.Stop)
        warmSignals.put(Request.Stop)
        worker.join()
        warmer.join()
    }

    /** Runs [action] on the index under the read lock. */
    private fun <T> readIndex(action: (SearchIndex) -> T): T {
        lock.readLock().lock()
        try {
            return action(index)
        } finally {
            lock.readLock().unlock()
        }
    }

    /** Tells the worker to re-answer and the warmer to start its quiet wait. */
    private fun signalGrew() {
        requests.put(Request.Refresh)
        warmSignals.put(Request.Refresh)
    }

    /** Takes the write lock without queueing ahead of readers for long, runs [action], and records the hold. */
    private fun pushUnderWriteLock(action: (SearchIndex) -> Unit) {
        /** When the wait for the lock began. */
        val waitStart: Long = System.nanoTime()
        while (!lock.writeLock().tryLock()) {
            if (System.nanoTime() - waitStart > WRITE_PATIENCE_NANOS) {
                lock.writeLock().lock()
                break
            }
            LockSupport.parkNanos(WRITE_POLL_NANOS)
        }
        /** When the hold began. */
        val held: Long = System.nanoTime()
        try {
            action(index)
        } finally {
            lock.writeLock().unlock()
        }
        /** How long the lock was held. */
        val holdNanos: Long = System.nanoTime() - held
        longestPush.accumulateAndGet(holdNanos) { current, candidate -> maxOf(current, candidate) }
        totalPush.addAndGet(holdNanos)
        pushCount.incrementAndGet()
        if (holdNanos > SLOW_PUSH_NANOS) {
            logger("search push held the write lock ${holdNanos / NANOS_PER_MILLI} ms")
        }
    }

    /** Ranks one query under the read lock and packages the answer. */
    private fun answerFor(number: Long, text: String, refresh: Boolean): SearchAnswer {
        /** When ranking began. */
        val started: Long = System.nanoTime()
        lock.readLock().lock()
        try {
            /** The index this answer reads. */
            val current: SearchIndex = index
            /** The ranked state. */
            val state: SearchState = current.search(text)
            /** The answer with its timing. */
            val answer = SearchAnswer(
                sequence = number,
                query = text,
                state = state,
                knownTracks = current.trackCount,
                complete = complete.get(),
                refresh = refresh,
                generation = current.generation,
                elapsedMillis = (System.nanoTime() - started) / NANOS_PER_MILLI,
            )
            logger(
                "search answered seq=$number refresh=$refresh known=${answer.knownTracks} " +
                    "complete=${answer.complete} ms=${answer.elapsedMillis}",
            )
            return answer
        } finally {
            lock.readLock().unlock()
        }
    }

    /** Ranks a query and delivers the answer, delivering the unavailable state when ranking fails. */
    private fun answerAndDeliver(number: Long, text: String, refresh: Boolean) {
        try {
            deliver(answerFor(number, text, refresh))
        } catch (expectedFailure: RuntimeException) {
            logger("search failed seq=$number: $expectedFailure")
            deliver(
                SearchAnswer(number, text, SearchState.Unavailable, 0, complete.get(), refresh, generation, 0.0),
            )
        }
    }

    // What:     `private fun work()` is the query worker's loop.
    // Why:      It coalesces requests, answers the newest query, and spaces refreshes so a scan's many batches cost
    //           one repaint per interval.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { const inbox = await drain(requests); /* newest query wins; refresh at most every interval */ }
    // ```
    /** Answers new queries at once and re-answers the current query after appends, at most once per interval. */
    private fun work() {
        /** The newest query, which refreshes re-answer. */
        var current: Request.Query? = null
        /** When the next refresh is due, or null when none is. */
        var refreshDue: Long? = null
        /** When the last answer was delivered. */
        var lastAnswer: Long = System.nanoTime()
        while (true) {
            /** How long to wait for the next message. */
            val waitNanos: Long = refreshDue?.let { due -> maxOf(0L, due - System.nanoTime()) } ?: IDLE_WAIT_NANOS
            /** Messages received, the first after waiting and the rest already queued. */
            val inbox = ArrayList<Request>()
            requests.poll(waitNanos, TimeUnit.NANOSECONDS)?.let { first ->
                inbox.add(first)
                requests.drainTo(inbox)
            }
            /** The newest query in the inbox, or null. */
            var newest: Request.Query? = null
            for (request in inbox) {
                when (request) {
                    is Request.Query -> newest = request
                    Request.Refresh -> if (refreshDue == null) {
                        refreshDue = lastAnswer + refreshIntervalMillis * NANOS_PER_MILLI.toLong()
                    }
                    Request.Stop -> return
                }
            }
            if (newest != null) {
                answerAndDeliver(newest.sequence, newest.text, refresh = false)
                lastAnswer = System.nanoTime()
                current = newest
                refreshDue = null
            } else if (refreshDue != null && System.nanoTime() >= refreshDue) {
                refreshDue = null
                current?.let { open -> answerAndDeliver(open.sequence, open.text, refresh = true) }
                lastAnswer = System.nanoTime()
            }
        }
    }

    /** Refreshes the cached answer of the largest first-character bucket that is behind; false when none is. */
    private fun warmOne(): Boolean {
        return readIndex { current ->
            /** First characters whose cached answer is behind. */
            val behind: List<Char> = current.unitsBehind()
            if (behind.isEmpty()) {
                false
            } else {
                current.warm(behind.first())
                true
            }
        }
    }

    // What:     `private fun warm()` is the warm-up thread's loop.
    // Why:      After the library has been quiet it refreshes every stale single-character answer, one character per
    //           read-lock hold, so the first keystroke of a common letter is a cache read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { await quiet(250); while (await warmOne()) { if (signalled()) break; } }
    // ```
    /** Waits for a quiet period, then warms stale characters until the library changes again. */
    private fun warm() {
        // Validating the ASCII collation shortcut costs about a tenth of a second the first time it is read, so
        // it is paid here and not on the first keystroke.
        logger("search ascii collation shortcut in use: ${AsciiCollation.prepare()}")
        /** Whether a signal arrived since the last warm-up. */
        var pending = false
        while (true) {
            if (!pending) {
                if (warmSignals.take() == Request.Stop) {
                    return
                }
                pending = true
            }
            /** A message that arrived during the quiet wait, which restarts it. */
            val interrupted: Request? = warmSignals.poll(warmDelayMillis, TimeUnit.MILLISECONDS)
            if (interrupted == Request.Stop) {
                return
            }
            if (interrupted != null) {
                continue
            }
            pending = false
            while (!pending) {
                /** A message that arrived while warming. */
                val signal: Request? = warmSignals.poll()
                if (signal == Request.Stop) {
                    return
                }
                if (signal != null) {
                    pending = true
                } else if (!warmOne()) {
                    break
                }
            }
        }
    }
}
