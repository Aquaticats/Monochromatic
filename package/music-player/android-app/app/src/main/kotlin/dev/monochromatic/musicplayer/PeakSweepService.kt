// Auto-started foreground service that runs the INITIAL full true-peak index in one continuous
// session, in parallel, on the performance cores. The periodic WorkManager sweep
// (PeakSweepScheduler) is throttled into ~10-minute charging windows and never finished a
// multi-thousand-track library on a real phone; a user-foreground service escapes that windowing.
// `startIfNeeded` starts it after each completed library load or rescan, and only when the sweep ledger
// (files/sweep-ledger.bin) shows tracks that are new, replaced or previously failed. See
// DECISION.peak-sweep-parallelism.md for the on-device benchmark that set the worker count and
// priority.
//
// Decision D107: this service is the process owner of the analysis notification. Its mandatory foreground
// notification IS the progress notification (title, count, determinate bar, Pause and Resume button), posted
// from a coroutine of its own so the sweep workers and the UI thread never wait on the notification service.
// The sweep lives as long as this service, which outlives the activity; the pause gate is process-wide
// (AnalysisControl) because the button is a broadcast receiver.

package dev.monochromatic.musicplayer

import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.SystemClock
import android.util.Log
import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic.AtomicBoolean
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import androidx.core.net.toUri
import dev.monochromatic.musicplayer.core.AnalysisNotificationContent
import dev.monochromatic.musicplayer.core.AnalysisNotificationPublisher
import dev.monochromatic.musicplayer.core.AnalysisUpdatePacer
import dev.monochromatic.musicplayer.core.analysisNotificationContent
import dev.monochromatic.musicplayer.core.SweepLedger
import dev.monochromatic.musicplayer.core.SweepPlan
import dev.monochromatic.musicplayer.core.SweepRecord
import dev.monochromatic.musicplayer.core.TrackSignatures
import dev.monochromatic.musicplayer.core.trackFileNameOf
import dev.monochromatic.musicplayer.core.shouldStartAnalysis
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

/**
 * Foreground service that performs the parallel true-peak analysis of the tracks the analysis has not looked at.
 * Started by [PlaybackService] after every completed library load or rescan that finds such tracks: the whole library
 * on the first run, and the new files whenever the library gains tracks later (M9); self-terminates as soon as those
 * tracks are measured. Its foreground notification
 * is the analysis progress notification of decision D107.
 */
class PeakSweepService : Service() {
    /** Service-scoped coroutine scope for the sweep; cancelled in [onDestroy]. */
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    /** Publisher of the run in progress, or null before the first start command; touched on the main thread. */
    private var publisher: AnalysisNotificationPublisher? = null

    /** Job of the run in progress, so a repeated start command does not begin a second sweep. */
    private var sweepJob: Job? = null

    /** Set by a start command that arrives during a pass, so the run plans and sweeps once more before it stops. */
    @Volatile
    private var rerunRequested: Boolean = false

    /** Whether the next pass tries earlier failures again, set from the start command's intent. */
    @Volatile
    private var retryFailed: Boolean = false

    /** Bound clients are not supported; this service is start-only. */
    override fun onBind(intent: Intent?): IBinder? = null

    /**
     * Promotes the service to foreground, runs the sweep, persists the done flag, then stops. A start command
     * that arrives while a sweep runs (the activity was recreated) only refreshes the foreground promotion.
     */
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        ensureAnalysisChannel(this)
        /** Publisher of the run that is already in progress, if there is one. */
        val running: AnalysisNotificationPublisher? = publisher
        if (running != null && sweepJob?.isActive == true) {
            Log.i(TAG, "PeakSweepService start command while a sweep runs; another pass follows it")
            rerunRequested = true
            retryFailed = retryFailed || (intent?.getBooleanExtra(EXTRA_RETRY_FAILED, false) ?: false)
            promoteToForeground(running.currentContent())
            return START_NOT_STICKY
        }
        retryFailed = intent?.getBooleanExtra(EXTRA_RETRY_FAILED, false) ?: false
        /** Frame shown until the library is read and the total is known, an indeterminate bar. */
        val firstFrame: AnalysisNotificationContent = analysisNotificationContent(done = 0, total = 0, paused = false)
        promoteToForeground(firstFrame)
        sweepJob = scope.launch {
            /** Number of passes made in this run; a start command that arrives during a pass makes another one. */
            var pass = 0
            do {
                rerunRequested = false
                if (pass > 0) {
                    promoteToForeground(firstFrame)
                }
                pass += 1
                /** Publisher of this pass, which the notification follows. */
                val newPublisher: AnalysisNotificationPublisher = newPublisherFor(firstFrame)
                publisher = newPublisher
                /** Posting loop, on the IO dispatcher because a post is a binder call. */
                val poster: Job = launch(Dispatchers.IO) { newPublisher.run() }
                /** True only when the pass walked its plan without being cut short. */
                val completed: Boolean = try {
                    runSweep(newPublisher)
                } finally {
                    newPublisher.abandon()
                }
                poster.join()
                AnalysisControl.setActiveTotal(0)
                Log.i(TAG, "PeakSweepService pass $pass finished completed=$completed again=$rerunRequested")
            } while (rerunRequested)
            ServiceCompat.stopForeground(this@PeakSweepService, ServiceCompat.STOP_FOREGROUND_REMOVE)
            stopSelf()
        }
        return START_NOT_STICKY
    }

    /** Builds the publisher of one pass, whose first frame the foreground promotion already posted. */
    private fun newPublisherFor(firstFrame: AnalysisNotificationContent): AnalysisNotificationPublisher {
        /** Pacer that knows the first frame was posted by the foreground promotion. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(content = firstFrame, nowMs = SystemClock.elapsedRealtime())
        return AnalysisNotificationPublisher(
            paused = AnalysisControl.pauseGate.paused,
            pacer = pacer,
            clockMs = SystemClock::elapsedRealtime,
            post = { content -> postContent(content) },
            remove = { removeNotification() },
        )
    }

    /** Loads the library and measures every uncached track across the parallel decode pool. */
    private suspend fun runSweep(newPublisher: AnalysisNotificationPublisher): Boolean {
        AnalysisControl.pauseGate.resume()
        SweepProgress.reset()
        /** The production track enumeration, the exact list the background worker also walks. */
        val library = LibrarySource.load(applicationContext)
        /** Tracks the analysis ran for earlier, or failed on. */
        val ledger: SweepLedger = loadLedger(applicationContext)
        /** What to analyze: new and changed files, and the earlier failures when this start retries them. */
        val plan: SweepPlan = ledger.plan(
            uris = library.map { track -> track.uri },
            signatureOf = TrackSignatures::of,
            retryFailed = retryFailed.also { retryFailed = false },
        )
        /** Signature of each planned file as the scan just read it, kept for the ledger record. */
        val signatures: Map<String, Long> = plan.todo.associateWith { uri -> TrackSignatures.of(uri) }
        /** Library tracks by URI, to name a failed file. */
        val byUri: Map<String, Track> = library.associateBy { track -> track.uri }
        /** The planned tracks in plan order. */
        val tracks: List<Track> = plan.todo.mapNotNull { uri -> byUri[uri] }
        if (tracks.isEmpty()) {
            Log.i(TAG, "PeakSweepService found no new tracks to index library=${library.size}")
            return false
        }
        Log.i(
            TAG,
            "PeakSweepService indexing ${tracks.size} tracks fresh=${plan.fresh} retried=${plan.retried} " +
                "of ${library.size}",
        )
        /** Tracks that were measured or cached, with their signatures; filled from the decode workers. */
        val done = ConcurrentLinkedQueue<Pair<String, Long>>()
        /** Tracks that could not be decoded, with their signatures; filled from the decode workers. */
        val failures = ConcurrentLinkedQueue<Pair<String, Long>>()
        SweepProgress.report(done = 0, total = tracks.size)
        newPublisher.report(done = 0, total = tracks.size)
        AnalysisControl.setActiveTotal(tracks.size)
        /** Outcome tally from the shared parallel coordinator over the foreground decode pool. */
        val tally = try {
            sweepTracksInParallel(
                items = tracks,
                workers = FOREGROUND_SWEEP_WORKERS,
                dispatcher = foregroundSweepDispatcher,
                flushBatch = FLUSH_BATCH,
                notifyEvery = NOTIFY_EVERY,
                maxTracks = Int.MAX_VALUE,
                process = { track ->
                    measureAndCache(applicationContext, track.uri.toUri(), foregroundSweepDispatcher).also { outcome ->
                        /** The track with the signature of the file the scan read. */
                        val entry: Pair<String, Long> = track.uri to (signatures[track.uri] ?: 0L)
                        if (outcome == SweepOutcome.FAILED || outcome == SweepOutcome.UNFINGERPRINTABLE) {
                            failures.add(entry)
                        } else {
                            done.add(entry)
                        }
                    }
                },
                onFlush = { },
                onProgress = { done, total ->
                    SweepProgress.report(done = done, total = total)
                    newPublisher.report(done = done, total = total)
                },
                pauseGate = AnalysisControl.pauseGate,
            )
        } catch (cancellation: CancellationException) {
            SweepProgress.reset()
            throw cancellation
        }
        SweepProgress.report(done = tally.total, total = tally.total)
        newPublisher.report(done = tally.total, total = tally.total)
        newPublisher.complete()
        Log.i(TAG, "PeakSweepService indexed ${tally.processed} of ${tally.total} tracks")
        MeasureInvalidation.report(done.map { (uri, _) -> uri })
        /** The ledger with this sweep's results and the files that failed for the first time. */
        val record: SweepRecord = ledger.record(done = done.toList(), failures = failures.toList())
        saveLedger(applicationContext, record.ledger)
        if (record.newFailures.isNotEmpty()) {
            Log.w(TAG, "PeakSweepService could not decode ${record.newFailures.size} files")
            AnalysisFailures.report(
                context = applicationContext,
                fileNames = record.newFailures.map { uri -> trackFileNameOf(byUri[uri]?.displayPath ?: uri) },
            )
        }
        return true
    }

    /** Promotes the service to foreground with the notification for [content]. */
    private fun promoteToForeground(content: AnalysisNotificationContent) {
        ServiceCompat.startForeground(
            this,
            ANALYSIS_NOTIFICATION_ID,
            buildAnalysisNotification(this, content),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC
            } else {
                0
            },
        )
    }

    /**
     * Posts [content] over the foreground notification. Called from the posting coroutine only; a failure is
     * logged and swallowed so one refused post cannot end the posting loop.
     */
    private fun postContent(content: AnalysisNotificationContent) {
        runCatching {
            getSystemService(NotificationManager::class.java)
                .notify(ANALYSIS_NOTIFICATION_ID, buildAnalysisNotification(this, content))
        }.onFailure { refused ->
            Log.w(TAG, "PeakSweepService could not post the analysis notification", refused)
        }
    }

    /**
     * Takes the notification away: ends the foreground promotion with removal, then cancels any plain copy.
     * Called from the posting coroutine after the final value, so no post can follow it.
     */
    private fun removeNotification() {
        runCatching {
            ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
            getSystemService(NotificationManager::class.java).cancel(ANALYSIS_NOTIFICATION_ID)
        }.onFailure { refused ->
            Log.w(TAG, "PeakSweepService could not remove the analysis notification", refused)
        }
    }

    /** Cancels the sweep scope so a destroyed service does not leak its workers. */
    override fun onDestroy() {
        scope.cancel()
        AnalysisControl.setActiveTotal(0)
        super.onDestroy()
    }

    /**
     * API 34 time-limit signal, which the system sends only to a short foreground service. The sweep is a dataSync
     * service, whose limit arrives through the two-argument overload, so on the platforms measured this one is
     * never called for it and only stays as a safe stop should the service type ever change.
     */
    override fun onTimeout(startId: Int) {
        Log.w(TAG, "PeakSweepService hit the foreground-service time limit; stopping")
        stopSelf()
    }

    /**
     * API 35+ time-limit signal for a dataSync foreground service: stop promptly, because the system raises
     * `ForegroundServiceDidNotStopInTimeException` and ends the app a few seconds after this callback if the
     * service keeps running (measured on an API 37 emulator with the limit shortened to one minute). Measurements
     * persist in batches, so a stop here loses at most one unflushed batch, and the next launch continues from the
     * cache and the sweep ledger only records tracks of a run that finished.
     */
    override fun onTimeout(startId: Int, fgsType: Int) {
        Log.w(TAG, "PeakSweepService hit the foreground-service time limit fgsType=$fgsType; stopping")
        stopSelf()
    }

    /** Static entry points and tuning constants for the sweep service. */
    companion object {
        /** Shared logcat tag with the rest of the sweep code. */
        private const val TAG = "PeakSweep"

        /** Fresh measurements between disk flushes: small enough to bound loss when killed, large
         *  enough that the full-file JSON rewrite stays cheap (the cache is about 120 KB). */
        private const val FLUSH_BATCH = 32

        /** Visit count between progress reports, so the sweep reports without thrashing. */
        private const val NOTIFY_EVERY = 16

        /** File in app storage that remembers the tracks the analysis has looked at. */
        private const val LEDGER_FILE = "sweep-ledger.bin"

        /** Reads the ledger of analysed tracks; a missing or damaged file is the empty ledger, so everything is new. */
        private fun loadLedger(context: Context): SweepLedger {
            /** The stored ledger. */
            val file = File(context.filesDir, LEDGER_FILE)
            if (!file.isFile) {
                return SweepLedger.EMPTY
            }
            return try {
                SweepLedger.fromBytes(file.readBytes())
            } catch (expectedFailure: java.io.IOException) {
                Log.w(TAG, "PeakSweepService could not read the sweep ledger", expectedFailure)
                SweepLedger.EMPTY
            }
        }

        /** Writes the ledger through a temporary file, so a kill mid-write keeps the previous ledger. */
        private fun saveLedger(context: Context, ledger: SweepLedger) {
            /** Final location of the ledger. */
            val file = File(context.filesDir, LEDGER_FILE)
            /** Temporary copy, moved over the final file once whole. */
            val temporary = File(context.filesDir, "$LEDGER_FILE.tmp")
            try {
                temporary.writeBytes(ledger.toBytes())
                Files.move(
                    temporary.toPath(),
                    file.toPath(),
                    StandardCopyOption.REPLACE_EXISTING,
                    StandardCopyOption.ATOMIC_MOVE,
                )
            } catch (expectedFailure: java.io.IOException) {
                Log.w(TAG, "PeakSweepService could not write the sweep ledger", expectedFailure)
            }
        }

        /** Intent extra that tells the service to try the earlier failures again in this run. */
        private const val EXTRA_RETRY_FAILED = "retry-failed"

        /** True until the first check of this process, which is the start that retries failed files. */
        private val processStart = AtomicBoolean(true)

        /**
         * Starts the foreground sweep when a music source can be read and the library holds work for the analysis: new
         * files, files whose size or time changed, and, at the app's start or beside such work, files that failed
         * before (D84, D107, D113). Call it after each completed library load or rescan, off the main thread, with the
         * scan's file signatures recorded; the scans keep them for the process's life, so no check sees them missing.
         *
         * @param context any context of this app
         * @param uris URIs of every track of the library as it is now
         */
        fun startIfNeeded(context: Context, uris: List<String>) {
            /** Whether the library can be listed: music access or a held folder grant. */
            val sourceReadable: Boolean = hasAudioPermission(context) || LibraryRoot.heldRoot(context) != null
            /** What a sweep would analyze now. */
            val plan: SweepPlan = loadLedger(context).plan(
                uris = uris,
                signatureOf = TrackSignatures::of,
                retryFailed = processStart.getAndSet(false),
            )
            if (!shouldStartAnalysis(sourceReadable = sourceReadable, plan = plan)) {
                Log.i(TAG, "PeakSweepService not started readable=$sourceReadable work=${plan.todo.size}")
                return
            }
            Log.i(TAG, "PeakSweepService starting fresh=${plan.fresh} retried=${plan.retried}")
            ContextCompat.startForegroundService(
                context,
                Intent(context, PeakSweepService::class.java).putExtra(EXTRA_RETRY_FAILED, plan.retried > 0),
            )
        }
    }
}
