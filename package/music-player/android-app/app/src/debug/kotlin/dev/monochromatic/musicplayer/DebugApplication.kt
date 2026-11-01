package dev.monochromatic.musicplayer

import android.os.StrictMode
import android.util.Log

/** Application class of debug builds: turns on StrictMode logging for the main thread and for leaked closables. */
class DebugApplication : MusicPlayerApplication() {
    /** Installs the StrictMode policies before any activity or service starts, then starts the warm-up. */
    override fun onCreate() {
        StrictMode.setThreadPolicy(StrictMode.ThreadPolicy.Builder().detectAll().penaltyLog().build())
        StrictMode.setVmPolicy(StrictMode.VmPolicy.Builder().detectLeakedClosableObjects().penaltyLog().build())
        super.onCreate()
        Log.i("DebugApplication", "StrictMode logging is on")
    }
}
