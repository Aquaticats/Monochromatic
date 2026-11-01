// The application class of the player. It does one thing: before the first screen needs them, it reads the two
// preference files and loads the native library on a worker thread, so the main thread meets them already in memory.
// StrictMode in the debug build found each of these as a disk read on the main thread: the native library load (about
// 78 ms a time, several times) when the playback service built its engine, and the session and library root files
// (up to 529 ms) when the first-run decision and the activity asked for them.

// What:     `package dev.monochromatic.musicplayer` places the class beside the activity and the service it serves.
// Why:      The manifest names the class by its package-relative name.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.app.Application
import android.util.Log

/** Log tag of the warm-up. */
private const val WARM_UP_TAG: String = "AppWarmUp"

/** Name of the worker thread of the warm-up. */
private const val WARM_UP_THREAD: String = "app-warm-up"

// What:     `open class MusicPlayerApplication : Application()` is the process-wide singleton Android creates first.
// Why:      Its `onCreate` runs before any activity or service of the process, so it is the earliest place to start
//           work that must not be left to the main thread. It is open so the debug build can add StrictMode on top.
//
// In TS you'd write (pseudocode):
// ```ts
// class MusicPlayerApplication extends Application { onCreate() { startThread(warmUp); } }
// ```
/** Application of the player: starts the warm-up of the files and the native library that the main thread needs. */
open class MusicPlayerApplication : Application() {
    /** Starts the warm-up on a daemon thread, so it never delays the first frame and never keeps the process alive. */
    override fun onCreate() {
        super.onCreate()
        Thread({ warmUp() }, WARM_UP_THREAD).apply { isDaemon = true }.start()
    }

    /** Loads the native library and reads the preference files, logging how long each took. */
    private fun warmUp() {
        /** Start of the native library load. */
        val nativeStart: Long = System.nanoTime()
        Class.forName(NativeBridge::class.java.name)
        Log.i(WARM_UP_TAG, "native library loaded ms=${(System.nanoTime() - nativeStart) / NANOS_PER_MILLI}")
        /** Start of the preference reads. */
        val prefsStart: Long = System.nanoTime()
        LibraryRoot.warmUp(this)
        SessionStore.warmUp(this)
        Log.i(WARM_UP_TAG, "preferences read ms=${(System.nanoTime() - prefsStart) / NANOS_PER_MILLI}")
    }

    /** Constants of the warm-up. */
    private companion object {
        /** Nanoseconds in a millisecond. */
        const val NANOS_PER_MILLI: Long = 1_000_000L
    }
}
