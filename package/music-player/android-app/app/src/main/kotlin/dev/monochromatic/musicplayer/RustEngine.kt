// What:     `package dev.monochromatic.musicplayer` declares the namespace this file's
//           declarations belong to. This file is compiled with the app's main source set and
//           implements the production native audio engine.
// Why:      Keeps `RustEngine` in the same package as the shared `AudioEngine` interface it
//           implements and the `NativeBridge` it calls into.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword — file path is the module.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.BroadcastReceiver` brings in `BroadcastReceiver`, the base
//           class for objects that receive system broadcasts (here, the headphone-unplug event).
// Why:      The engine subclasses it (anonymously) to react to "audio becoming noisy".
//
// In TS you'd write (pseudocode):
// ```ts
// import { BroadcastReceiver } from "android-framework";
// ```
import android.content.BroadcastReceiver

// What:     `import android.content.Context` brings in Android's `Context` (app-environment
//           handle), with constants like `Context.AUDIO_SERVICE` and `Context.RECEIVER_NOT_EXPORTED`.
// Why:      The constructor takes a `Context`; the engine also looks up system services and
//           registers a receiver through it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Context } from "android-framework";
// ```
import android.content.Context

// What:     `import android.content.Intent` brings in `Intent`, Android's "something happened /
//           please do something" message object; a received broadcast arrives as an `Intent`.
// Why:      `onReceive(context, intent)` inspects the `Intent`'s action.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Intent } from "android-framework";
// ```
import android.content.Intent

// What:     `import android.content.IntentFilter` brings in `IntentFilter`, which declares WHICH
//           broadcast actions a receiver wants.
// Why:      The engine registers `noisyReceiver` with a filter for the becoming-noisy action.
//
// In TS you'd write (pseudocode):
// ```ts
// import { IntentFilter } from "android-framework";
// ```
import android.content.IntentFilter

// What:     `import android.media.AudioAttributes` brings in `AudioAttributes` (usage/content-type
//           metadata) and its nested `AudioAttributes.Builder`.
// Why:      The focus request is built with media-usage attributes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { AudioAttributes } from "android-framework";
// ```
import android.media.AudioAttributes

// What:     `import android.media.AudioFocusRequest` brings in `AudioFocusRequest` and its nested
//           `AudioFocusRequest.Builder`: the object describing a request to OWN audio focus.
// Why:      The engine builds one persistent `focusRequest` and reuses it for every play.
//
// In TS you'd write (pseudocode):
// ```ts
// import { AudioFocusRequest } from "android-framework";
// ```
import android.media.AudioFocusRequest

// What:     `import android.media.AudioManager` brings in `AudioManager`, the system audio service,
//           with focus methods and constants (`AUDIOFOCUS_GAIN`, `AUDIOFOCUS_LOSS`,
//           `ACTION_AUDIO_BECOMING_NOISY`, ...).
// Why:      The engine requests/abandons focus and reacts to focus changes through it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { AudioManager } from "android-framework";
// ```
import android.media.AudioManager

// What:     `import android.net.Uri` brings in Android's parsed `Uri`, with `Uri.parse(string)`.
// Why:      `openDescriptor`/`resolveNormalizationGain` parse the string URI.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Uri } from "android-framework";
// ```
import android.net.Uri

// What:     `import android.os.Handler` brings in `Handler`, which posts/schedules `Runnable`s onto
//           a specific thread's message loop (looper).
// Why:      The engine uses a main-looper `Handler` as its poller and to marshal callbacks onto the
//           main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // Mentally a setTimeout bound to a specific thread's event loop.
// ```
import android.os.Handler

// What:     `import android.os.Looper` brings in `Looper`; `Looper.getMainLooper()` is the main
//           (UI) thread's message loop.
// Why:      The poller `Handler` is bound to the main looper so callbacks land on the UI thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent — JS has a single implicit event loop.
// ```
import android.os.Looper

// What:     `import android.os.ParcelFileDescriptor` brings in `ParcelFileDescriptor` (a `Closeable`
//           open-file-descriptor wrapper exposing `.fd`).
// Why:      The engine opens a descriptor for the track and hands its fd to the native loader.
//
// In TS you'd write (pseudocode):
// ```ts
// // Mentally a file handle with a numeric `.fd` and a `.close()`.
// ```
import android.os.ParcelFileDescriptor

// What:     `import android.util.Log` brings in Android's tagged logging (`Log.i`, `Log.w`).
// Why:      The engine logs load events and warnings.
//
// In TS you'd write (pseudocode):
// ```ts
// // Mentally a tagged console.
// ```
import android.util.Log

// What:     `import androidx.core.content.ContextCompat` brings in the AndroidX compat helper for
//           `Context` operations; below we use its static `registerReceiver(...)` and its
//           `RECEIVER_NOT_EXPORTED` flag constant.
// Why:      `ContextCompat.registerReceiver` applies the API-33+ `RECEIVER_NOT_EXPORTED` export flag
//           on new devices and falls back to plain registration on API 26-32, so the becoming-noisy
//           receiver needs no hand-written `Build.VERSION.SDK_INT` branch and the app's true floor
//           stays at the native AAudio API-26 minimum instead of being pushed up to 33.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ContextCompat } from "androidx-core-content";
// ```
import androidx.core.content.ContextCompat

// What:     `import dev.monochromatic.musicplayer.core.QueuedLoadLedger` brings in the counter of queued native loads.
// Why:      `poll` asks it whether the worker's failure number names the latest load, so a decode failure is
//           reported once and an older load's failure is not.
//
// In TS you'd write (pseudocode):
// ```ts
// import { QueuedLoadLedger } from "./core/QueuedLoadLedger";
// ```
import dev.monochromatic.musicplayer.core.QueuedLoadLedger

// What:     `import dev.monochromatic.musicplayer.core.volumeWhileDucked` brings in the pure duck volume rule.
// Why:      A can-duck focus loss lowers the playing volume instead of pausing (D134), and this rule combines the
//           user volume with the duck flag.
//
// In TS you'd write (pseudocode):
// ```ts
// import { volumeWhileDucked } from "./core/FocusDuck";
// ```
import dev.monochromatic.musicplayer.core.volumeWhileDucked

// What:     `import java.io.File` brings in the JDK `File` type (a path on the local filesystem).
// Why:      `openDescriptor` wraps a bare absolute path in a `File` to open it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Mentally just a filesystem path string.
// ```
import java.io.File

// What:     `import kotlinx.coroutines.CoroutineScope` brings in `CoroutineScope`, the owner of a
//           set of coroutines that can all be cancelled together.
// Why:      `resolveScope` owns the per-track gain resolution so `release()` can cancel it.
//
// In TS you'd write (pseudocode):
// ```ts
// // Mentally an AbortController owning async tasks.
// ```
import kotlinx.coroutines.CoroutineScope

// What:     `import kotlinx.coroutines.Dispatchers` brings in the named thread pools
//           (`Dispatchers.Default` for CPU work).
// Why:      `resolveScope` runs on `Dispatchers.Default`.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent — JS is single-threaded.
// ```
import kotlinx.coroutines.Dispatchers

// What:     `import kotlinx.coroutines.SupervisorJob` brings in `SupervisorJob()`, a job whose
//           children fail INDEPENDENTLY (one failure does not cancel siblings or the scope).
// Why:      The resolve scope uses it so one failed measure does not kill later ones.
//
// In TS you'd write (pseudocode):
// ```ts
// // Mentally: one task's failure must not cancel its siblings.
// ```
import kotlinx.coroutines.SupervisorJob

// What:     `import kotlinx.coroutines.cancel` brings in the `cancel()` extension on `CoroutineScope`.
// Why:      `release()` cancels `resolveScope`.
//
// In TS you'd write (pseudocode):
// ```ts
// // resolveScope.cancel() ~ abortController.abort()
// ```
import kotlinx.coroutines.cancel

// What:     `import kotlinx.coroutines.launch` brings in `launch { ... }`, which starts a coroutine
//           that runs concurrently and returns immediately (fire-and-forget).
// Why:      `load` launches the off-thread gain resolution.
//
// In TS you'd write (pseudocode):
// ```ts
// // scope.launch { body } ~ void (async () => { body })();
// ```
import kotlinx.coroutines.launch

// =============================================================================
// File summary (folds in the old KDoc's domain content; corrects a stale claim)
// =============================================================================
//
// `RustEngine` is the production `AudioEngine`: a thin Kotlin FACADE over the native engine
// (`engine.rs`), which decodes with symphonia/libopus and outputs through AAudio, ALL
// in-process. Playback does not go through platform MediaCodec or ExoPlayer.
//
// The native engine is PULL-based (no native-to-JVM callbacks), so this class translates that
// into the PUSH-style `AudioEngine` contract: a 200 ms main-thread poller reads the native
// playing/ended state and fires `onPlayingChanged`/`onTrackEnded` on transitions.
// `PlayerController` drives this engine entirely on the MAIN thread, which is also where the
// poller runs, so the callbacks land on the thread the controller's Compose state requires.
//
// Loading hands the native side a `content://` (or file) descriptor: the borrowed
// `ParcelFileDescriptor` fd is passed inside a `use {}` block, and the native `load` dups it
// synchronously, so the JVM keeps and closes the original while Rust owns the dup (the
// dup-ownership protocol that avoids the fdsan double-close).
//
// This file owns the Android behaviors the native engine needs around the sample pipeline: audio
// focus via `focusRequest`/`requestFocus`/`onFocusChange`, the becoming-noisy headphone-unplug pause
// via `noisyReceiver`, and true-peak normalization via `resolveNormalizationGain` +
// `NativeBridge.nativeEngineSetNormalizationGain`.

// What:     `class RustEngine(context: Context) : AudioEngine { ... }` declares a class with a
//           primary constructor taking a `Context` (no `val`, so not stored as a field) that
//           IMPLEMENTS the `AudioEngine` interface (the `: AudioEngine` has no `()` because an
//           interface is implemented, not constructed).
// Why:      Provide the native Rust implementation of the `AudioEngine` contract.
// Gotcha:   `: AudioEngine` WITHOUT `()` = implements an interface; a supertype WITH `()` would be
//           a superclass constructor call.
//
// In TS you'd write (pseudocode):
// ```ts
// class RustEngine implements AudioEngine {
//   constructor(context: Context) { /* ...fields below... */ }
// }
// ```
/**
 * Defines rust engine type for this music-player component; the TypeScript-oriented notes above explain its
 * role.
 */
class RustEngine(context: Context) : AudioEngine {
    // What:     `private val appContext: Context = context.applicationContext` declares a private
    //           read-only `Context` holding the long-lived application context.
    // Why:      For the content resolver, held without leaking the (short-lived) activity.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly appContext: Context = context.applicationContext;
    // ```
    /**
     * Defines app context value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val appContext: Context = context.applicationContext

    // What:     `private var handle: Long = NativeBridge.nativeEngineCreate()` declares a private,
    //           REASSIGNABLE `Long` (64-bit) holding the opaque native engine handle, initialised by
    //           creating the native engine. Sibling `Int` (32-bit) is declined: a native pointer
    //           needs 64 bits. It is `var` because `release()` later resets it to `0`.
    // Why:      Opaque native engine handle; `0` only if the worker thread could not be spawned.
    // Gotcha:   `handle` is an opaque pointer, not a number for math; only `== 0L` (invalid) is a
    //           meaningful test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private handle: bigint = NativeBridge.nativeEngineCreate(); // 0n => spawn failed
    // ```
    /**
     * Defines handle value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var handle: Long = NativeBridge.nativeEngineCreate()

    // What:     `private var onPlayingChanged: ((Boolean) -> Unit)? = null` declares a private,
    //           reassignable NULLABLE function-type field (`(Boolean) -> Unit` = takes a boolean,
    //           returns void; trailing `?` = nullable), initial `null`.
    // Why:      Play/pause-state callback, fired by the poller on a transition.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private onPlayingChanged: ((playing: boolean) => void) | null = null;
    // ```
    /**
     * Defines on playing changed value for this music-player component; the TypeScript-oriented notes above
     * explain its source and use.
     */
    private var onPlayingChanged: ((Boolean) -> Unit)? = null

    // What:     `private var onTrackEnded: (() -> Unit)? = null` declares a private, reassignable
    //           nullable no-arg void callback, initial `null`.
    // Why:      Natural-end callback, fired by the poller once per ended track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private onTrackEnded: (() => void) | null = null;
    // ```
    /**
     * Defines on track ended value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private var onTrackEnded: (() -> Unit)? = null

    // What:     `private val resolveScope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)`
    //           declares a private read-only coroutine scope. `SupervisorJob()` builds a supervisor
    //           job; `+` here is the OVERLOADED coroutine-context merge (NOT arithmetic), combining
    //           the job with `Dispatchers.Default`; `CoroutineScope(...)` wraps it.
    // Why:      Off-thread scope for resolving the per-track normalization gain (cache hit or native
    //           measure); cancelled in `release` so a pending measure cannot outlive the engine.
    // Gotcha:   `+` on coroutine contexts is OVERLOADED (context merge), not numeric addition.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const resolveScope = new AbortController(); // owns background gain-resolution tasks
    // ```
    /**
     * Defines resolve scope value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private val resolveScope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    // What:     `private var loadGeneration: Int = 0` declares a private reassignable 32-bit `Int`
    //           counter starting at 0. `Long` is unnecessary (never billions of loads per session).
    // Why:      Bumped each load; a resolved gain is applied only when its load is still current, so a
    //           measure that finishes after the user skipped ahead cannot retag the new track.
    //           Main-thread only, so it needs no `@Volatile`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private loadGeneration = 0;
    // ```
    /**
     * Defines load generation value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private var loadGeneration: Int = 0

    /** Whether no track is loaded, so position and duration read zero; set by `unload`, cleared by `load`. */
    private var unloaded: Boolean = false

    /** Load whose descriptor is still opening, with the calls made meanwhile; main thread only. */
    private var pendingLoad: PendingLoad? = null

    /** Receives a load that could not start, so the controller can tell the user. */
    private var onLoadFailed: ((String, LoadFailure) -> Unit)? = null

    /** Counts the loads queued with the native engine, to tell a decode failure of the latest one from an old one. */
    private val queuedLoads: QueuedLoadLedger = QueuedLoadLedger()

    /** Opens track descriptors on the I/O dispatcher with a time limit, never on the main thread. */
    private val descriptorOpener: TimedDescriptorOpen<ParcelFileDescriptor> = TimedDescriptorOpen(
        dispatcher = Dispatchers.IO,
        timeoutMs = OPEN_TIMEOUT_MS,
        opener = { uri -> openDescriptor(uri) },
        report = { uri, failure -> Log.w(LOG_TAG, "descriptor open threw for $uri", failure) },
    )

    // What:     `private var lastPlaying: Boolean = false` declares a private reassignable boolean,
    //           initial `false`.
    // Why:      Last play state the poller reported, to EDGE-TRIGGER `onPlayingChanged` (fire only on
    //           a change).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private lastPlaying = false;
    // ```
    /**
     * Defines last playing value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private var lastPlaying: Boolean = false

    // What:     `private var endedHandled: Boolean = false` declares a private reassignable boolean,
    //           initial `false`.
    // Why:      Whether the current ended state has already fired `onTrackEnded`; rearms when native
    //           clears it (so a track that ends fires exactly once).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private endedHandled = false;
    // ```
    /**
     * Defines ended handled value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private var endedHandled: Boolean = false

    // What:     `private val poller: Handler = Handler(Looper.getMainLooper())` declares a private
    //           read-only `Handler` bound to the MAIN looper (`Looper.getMainLooper()`), constructed
    //           with no `new`.
    // Why:      Main-looper poller that turns the native pull-state into the engine's push callbacks
    //           (and marshals work onto the main thread).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const poller = new Handler(Looper.getMainLooper());
    // ```
    /**
     * Defines poller value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val poller: Handler = Handler(Looper.getMainLooper())

    // What:     `private val pollTask: Runnable = object : Runnable { override fun run() { ... } }`
    //           declares a private read-only `Runnable`. `object : Runnable { ... }` is an OBJECT
    //           EXPRESSION: an anonymous one-off instance implementing the `Runnable` interface
    //           inline (no `()` after `Runnable` because it is an interface). `run()` is its body.
    // Why:      A SELF-RESCHEDULING poll task: each run polls, then re-posts itself after `POLL_MS`.
    // Gotcha:   `object : Runnable {}` is an anonymous instance created right here, not a type.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pollTask: Runnable = {
    //   run() { this.poll(); this.poller.postDelayed(pollTask, POLL_MS); },
    // };
    // ```
    /**
     * Defines poll task value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val pollTask: Runnable = object : Runnable {
        // What:     `override fun run() { ... }` overrides the `Runnable.run` method (the work the
        //           handler executes).
        // Why:      Define one poll cycle plus its self-reschedule.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // run() { this.poll(); this.poller.postDelayed(this, POLL_MS); }
        // ```
        /**
         * Defines run behavior for this music-player component; the TypeScript-oriented notes above explain its
         * call shape and effects.
         */
        override fun run() {
            // What:     `poll()` calls the engine's private `poll` method (sample native state, fire
            //           callbacks on transitions).
            // Why:      Perform one poll of the native engine.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.poll();
            // ```
            poll()
            // What:     `poller.postDelayed(this, POLL_MS)` re-posts THIS runnable to the handler to run
            //           again after `POLL_MS` milliseconds. `this` here refers to the anonymous
            //           `Runnable` itself.
            // Why:      Keep the poll loop going on a fixed cadence.
            // Gotcha:   `this` is the anonymous `Runnable`, NOT the enclosing `RustEngine`; that is how
            //           the task reschedules itself.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.poller.postDelayed(this, POLL_MS); // `this` = the runnable
            // ```
            poller.postDelayed(this, POLL_MS)
        }
    }

    // What:     `private val audioManager: AudioManager = appContext.getSystemService(Context.AUDIO_SERVICE) as
    //           AudioManager`
    //           declares a private read-only `AudioManager`. `getSystemService(Context.AUDIO_SERVICE)`
    //           returns a generic `Any?`/`Object`, so `as AudioManager` is an UNSAFE CAST narrowing it
    //           to `AudioManager` (it throws `ClassCastException` if the runtime type is wrong, which
    //           it never is for this well-known service).
    // Why:      System audio service, for focus and the music-stream becoming-noisy broadcast.
    // Gotcha:   `as` here is a RUNTIME-CHECKED cast (unlike TS's compile-only `as`); a wrong type
    //           throws rather than silently mis-typing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const audioManager = appContext.getSystemService(Context.AUDIO_SERVICE) as AudioManager;
    // ```
    /**
     * Defines audio manager value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private val audioManager: AudioManager = appContext.getSystemService(Context.AUDIO_SERVICE) as AudioManager

    // What:     `private val focusRequest: AudioFocusRequest = AudioFocusRequest.Builder(...)....build()`
    //           declares a private read-only `AudioFocusRequest`, built by a FLUENT BUILDER CHAIN
    //           (commented step by step below). `AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)`
    //           starts a request for permanent ("gain") focus.
    // Why:      Persistent media focus request (gain, usage=media), reused for every play.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const focusRequest = new AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
    //   .setAudioAttributes(new
    //   AudioAttributes.Builder().setUsage(USAGE_MEDIA).setContentType(CONTENT_TYPE_MUSIC).build())
    //   .setOnAudioFocusChangeListener((change) => this.onFocusChange(change), this.poller)
    //   .setWillPauseWhenDucked(true)
    //   .build();
    // ```
    /**
     * Defines focus request value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private val focusRequest: AudioFocusRequest =
        AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            // What:     `.setAudioAttributes( AudioAttributes.Builder().setUsage(...).setContentType(...).build() )`
            //           sets the request's attributes, built by a NESTED builder chain:
            //           `AudioAttributes.Builder()` -> `.setUsage(AudioAttributes.USAGE_MEDIA)` ->
            //           `.setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)` -> `.build()`.
            // Why:      Declare this as MEDIA/MUSIC audio so the focus system treats it correctly.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setAudioAttributes(
            //   new AudioAttributes.Builder().setUsage(USAGE_MEDIA).setContentType(CONTENT_TYPE_MUSIC).build(),
            // )
            // ```
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                    .build(),
            )
            // What:     `.setOnAudioFocusChangeListener({ change -> onFocusChange(change) }, poller)`
            //           registers the focus-change listener. The FIRST argument is a LAMBDA
            //           `{ change -> onFocusChange(change) }` (named param `change`, forwarding to the
            //           engine's `onFocusChange`); the SECOND argument `poller` is the `Handler` on
            //           which the listener is invoked (so it runs on the main thread).
            // Why:      Route focus changes (loss/gain) to `onFocusChange`, delivered on the main thread.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setOnAudioFocusChangeListener((change) => this.onFocusChange(change), this.poller)
            // ```
            .setOnAudioFocusChangeListener({ change -> onFocusChange(change) }, poller)
            // What:     `.setWillPauseWhenDucked(true)` tells the framework not to duck this app by itself but to
            //           report a transient-duck focus loss to the listener.
            // Why:      Android never ducks AAudio output (`PlaybackActivityMonitor.UNDUCKABLE_PLAYER_TYPES`), which
            //           is what this engine plays through, so the listener lowers the volume itself (D134).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setWillPauseWhenDucked(true)
            // ```
            .setWillPauseWhenDucked(true)
            // What:     `.build()` finalises the builder and returns the `AudioFocusRequest`.
            // Why:      Produce the reusable request object.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .build()
            // ```
            .build()

    // What:     `private var resumeOnFocusGain: Boolean = false` declares a private reassignable
    //           boolean, initial `false`.
    // Why:      Set when a transient focus loss paused mid-play, so a later focus GAIN resumes; a
    //           permanent loss or a user pause clears it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private resumeOnFocusGain = false;
    // ```
    /**
     * Defines resume on focus gain value for this music-player component; the TypeScript-oriented notes above
     * explain its source and use.
     */
    private var resumeOnFocusGain: Boolean = false

    // What:     `private var userVolume: Float = 1.0f` declares the volume the user chose, initial full volume.
    // Why:      A duck lowers the volume the engine plays at but must not forget the user's choice, so `setVolume`
    //           keeps the chosen value here and `applyVolume` combines it with `ducked`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private userVolume = 1.0;
    // ```
    /** Volume the user chose through [setVolume], before any duck. */
    private var userVolume: Float = 1.0f

    // What:     `private var ducked: Boolean = false` declares whether a can-duck focus loss is lowering the volume.
    // Why:      A notification sound or a navigation prompt asks other apps to play quieter, not to stop; the flag is
    //           set while that interruption lasts and cleared when focus returns or is lost for good (D134).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private ducked = false;
    // ```
    /** Whether a short interruption that allows ducking currently lowers the volume. */
    private var ducked: Boolean = false

    // What:     `private val noisyReceiver: BroadcastReceiver = object : BroadcastReceiver() { ... }`
    //           declares a private read-only `BroadcastReceiver` as an OBJECT EXPRESSION: an anonymous
    //           instance EXTENDING `BroadcastReceiver` (note the `()` after the base name = superclass
    //           constructor call, since it is a class, not an interface). `onReceive` is overridden.
    // Why:      Pauses on a headphone unplug (`ACTION_AUDIO_BECOMING_NOISY`) so audio never jumps to
    //           the speaker.
    // Gotcha:   `object : BroadcastReceiver()` (WITH `()`) extends a CLASS; contrast `object : Runnable`
    //           (no `()`) which implements an INTERFACE.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const noisyReceiver: BroadcastReceiver = new (class extends BroadcastReceiver {
    //   onReceive(context: Context | null, intent: Intent | null) { ... }
    // })();
    // ```
    /**
     * Defines noisy receiver value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private val noisyReceiver: BroadcastReceiver = object : BroadcastReceiver() {
        // What:     `override fun onReceive(context: Context?, intent: Intent?) { ... }` overrides the
        //           receiver callback. Both params are NULLABLE (`Context?`, `Intent?`) because the
        //           framework may pass `null`.
        // Why:      Handle the incoming broadcast and pause when it is the becoming-noisy action.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // onReceive(context: Context | null, intent: Intent | null) { ... }
        // ```
        /**
         * Defines on receive behavior for this music-player component; the TypeScript-oriented notes above
         * explain its call shape and effects.
         */
        override fun onReceive(context: Context?, intent: Intent?) {
            // What:     `if (intent?.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) { ... }` is a
            //           guarded check. `intent?.action` is a SAFE CALL: read `.action` only if `intent`
            //           is non-null (else `null`); `==` then compares that nullable string against the
            //           becoming-noisy action constant (`null` never equals the constant, so a null
            //           intent is safely ignored).
            // Why:      Only react to the headphone-unplug broadcast, not any other delivery.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (intent?.action === AudioManager.ACTION_AUDIO_BECOMING_NOISY) { ... }
            // ```
            if (intent?.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) {
                // What:     `resumeOnFocusGain = false` clears the resume flag.
                // Why:      A deliberate unplug-pause should NOT auto-resume on a later focus gain.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.resumeOnFocusGain = false;
                // ```
                resumeOnFocusGain = false
                // What:     `NativeBridge.nativeEnginePause(handle)` calls the native pause via the JNI
                //           bridge, passing the opaque `handle`.
                // Why:      Pause playback so audio does not blast from the phone speaker.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // NativeBridge.nativeEnginePause(this.handle);
                // ```
                NativeBridge.nativeEnginePause(handle)
            }
        }
    }

    // What:     `init { ... }` is an INITIALIZER BLOCK running during construction (after the property
    //           initialisers above).
    // Why:      Validate the native engine spawned, register the becoming-noisy receiver, and start the
    //           poll loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // (inside the constructor body)
    // ```
    init {
        // What:     `if (handle == 0L) { throw IllegalStateException("native engine worker could not be spawned") }`
        //           checks the handle against `0L` (a `Long` zero) and throws if the native engine
        //           could not be created.
        // Why:      A zero handle means the native worker thread failed to spawn; fail construction loudly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.handle === 0n) throw new IllegalStateException("native engine worker could not be spawned");
        // ```
        if (handle == 0L) {
            throw IllegalStateException("native engine worker could not be spawned")
        }
        // What:     `ContextCompat.registerReceiver(appContext, noisyReceiver,
        //           IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY), ContextCompat.RECEIVER_NOT_EXPORTED)`
        //           registers the receiver for the becoming-noisy action. `IntentFilter(...)` constructs
        //           a filter for that one action; `ContextCompat.RECEIVER_NOT_EXPORTED` flags the receiver
        //           as NOT visible to other apps.
        // Why:      Start listening for headphone-unplug so the engine can pause on it. The COMPAT call
        //           (not `appContext.registerReceiver(..., Context.RECEIVER_NOT_EXPORTED)`) is what keeps
        //           the minSdk floor at 26: `Context.RECEIVER_NOT_EXPORTED` is an API-33 field, so naming
        //           it directly would force minSdk 33, whereas `ContextCompat` carries the constant at all
        //           levels and applies the export flag only on API 33+ (a no-op on 26-32, where this
        //           system-protected broadcast registers fine without it).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // ContextCompat.registerReceiver(
        //   appContext,
        //   noisyReceiver,
        //   new IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY),
        //   ContextCompat.RECEIVER_NOT_EXPORTED,
        // );
        // ```
        ContextCompat.registerReceiver(
            appContext,
            noisyReceiver,
            IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY),
            ContextCompat.RECEIVER_NOT_EXPORTED,
        )
        // What:     `poller.postDelayed(pollTask, POLL_MS)` schedules the first poll after `POLL_MS`
        //           milliseconds, kicking off the self-rescheduling loop.
        // Why:      Begin polling the native pull-state to drive the push callbacks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.poller.postDelayed(pollTask, POLL_MS);
        // ```
        poller.postDelayed(pollTask, POLL_MS)
    }

    // What:     `override fun load(uri: String, play: Boolean) { ... }` implements `AudioEngine.load`.
    // Why:      Start loading a track by URI into the native engine. The descriptor open asks the media provider,
    //           which can stall for seconds, so it runs on a background thread with a time limit and the rest
    //           (focus, the native load, the optional seek) continues on the main thread when it answers. Calls
    //           that arrive meanwhile (`play`, `pause`, `seekTo`) are kept on the pending load and applied
    //           when it completes. NOTE: it deliberately does NOT reset `endedHandled` here: the worker clears
    //           the native `ended` flag asynchronously, so an eager reset would let a poll between this load and
    //           that clear see the OLD `ended=true` with `endedHandled=false` and fire `onTrackEnded` a second
    //           time (a skipped track). The falling-edge rearm in `poll()` is the correct place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // load(uri: string, play: boolean): void {
    //   const generation = ++this.loadGeneration;
    //   this.pending = { generation, uri, play, seekSec: null };
    //   this.opener.open(uri, (outcome) => mainThread.post(() => this.finishLoad(generation, outcome)));
    // }
    // ```
    /**
     * Starts loading [uri]; the descriptor opens off the main thread and playback begins when it answers.
     *
     * @param uri track to load
     * @param play whether playback should start once the track is loaded and audio focus is granted
     */
    override fun load(uri: String, play: Boolean) {
        Log.i(LOG_TAG, "RustEngine.load ${uri.substringAfterLast('/')} play=$play")
        NativeBridge.nativeEngineSetNormalizationGain(handle, UNITY_GAIN)
        unloaded = false
        /** Number of this load; a later load makes every answer to this one stale. */
        val generation: Int = ++loadGeneration
        pendingLoad = PendingLoad(generation = generation, uri = uri, play = play)
        // A failure of an older queued load belongs to a replaced request from here on, queued or not.
        queuedLoads.requested()
        descriptorOpener.open(uri) { outcome ->
            poller.post { finishLoad(generation = generation, outcome = outcome) }
        }
    }

    // What:     `private fun finishLoad(generation: Int, outcome: DescriptorOutcome<ParcelFileDescriptor>)`
    //           completes a load on the main thread once the descriptor open answered.
    // Why:      A stale answer (the user already chose another track) only closes its descriptor. A failed or
    //           stalled open stops the old track, so the deck does not keep playing something other than the chosen
    //           track, and tells the controller so it can show a message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private finishLoad(generation: number, outcome: DescriptorOutcome): void {
    //   if (generation !== this.loadGeneration) { outcome.descriptor?.close(); return; }
    //   const load = this.pending; this.pending = null;
    //   if (outcome.kind !== "opened") { nativePause(); this.onLoadFailed?.(load.uri, reason); return; }
    //   /* focus, native load, apply the kept seek, resolve the gain */
    // }
    // ```
    /** Finishes the pending load on the main thread with the outcome of its descriptor open. */
    private fun finishLoad(generation: Int, outcome: DescriptorOutcome<ParcelFileDescriptor>) {
        if (generation != loadGeneration || handle == 0L) {
            Log.i(LOG_TAG, "dropping the answer of a stale load generation=$generation")
            if (outcome is DescriptorOutcome.Opened) {
                outcome.descriptor.close()
            }
            return
        }
        /** The load this answer belongs to, with the calls that arrived while it was opening. */
        val load: PendingLoad = pendingLoad ?: return
        pendingLoad = null
        if (outcome !is DescriptorOutcome.Opened) {
            Log.w(LOG_TAG, "could not open a descriptor for ${load.uri}: $outcome")
            NativeBridge.nativeEnginePause(handle)
            onLoadFailed?.invoke(
                load.uri,
                if (outcome == DescriptorOutcome.TimedOut) LoadFailure.TIMED_OUT else LoadFailure.UNREADABLE,
            )
            return
        }
        /** Whether to start playing: wanted by the caller (as last changed while loading) and granted focus. */
        val startPlaying: Boolean = load.play && requestFocus()
        // The borrowed fd is dup'd synchronously by the native loader; `use {}` closes the original afterward.
        /** Native return code, 0 for success. */
        val result: Int = outcome.descriptor.use { NativeBridge.nativeEngineLoad(handle, it.fd, startPlaying) }
        if (result != 0) {
            Log.w(LOG_TAG, "native load failed (code $result) for ${load.uri}")
            onLoadFailed?.invoke(load.uri, LoadFailure.UNREADABLE)
            return
        }
        // The worker opens and decodes the file after this call returns; a failure there is read in `poll`.
        queuedLoads.queued(load.uri)
        load.seekSec?.let { seconds -> NativeBridge.nativeEngineSeek(handle, seconds) }
        resolveScope.launch {
            /** Normalization gain from the cache or a native measurement, resolved off the main thread. */
            val gain: Float = resolveNormalizationGain(load.uri)
            poller.post {
                if (generation == loadGeneration && handle != 0L) {
                    NativeBridge.nativeEngineSetNormalizationGain(handle, gain)
                }
            }
        }
    }

    // What:     `private suspend fun resolveNormalizationGain(uri: String): Float { ... }` declares a
    //           PRIVATE SUSPEND function returning a `Float`.
    // Why:      Resolve the track's true-peak normalization gain through the native decision service:
    //           `nativeResolveGain` returns a cached decision's gain on a hit, or decodes, caches, and
    //           returns on a miss. The cache and the gain math live in Rust now, so Kotlin computes
    //           nothing. A track that cannot be fingerprinted or whose fd cannot be opened plays at
    //           unity (the callback's clamp still guards against clipping); `nativeResolveGain` itself
    //           never fails (it falls back to the ceiling gain).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private async resolveNormalizationGain(uri: string): Promise<number> { ... }
    // ```
    /**
     * Defines resolve normalization gain behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    private suspend fun resolveNormalizationGain(uri: String): Float {
        // What:     `val parsed: Uri = Uri.parse(uri)` parses text into Android's URI object.
        // Why:      Fingerprinting needs the structured form.
        /** Parsed track URI used by the fingerprint helper. */
        val parsed: Uri = Uri.parse(uri)
        // What:     `val fingerprint: Long? = TrackFingerprint.of(...)` asks for the u64 cache key.
        // Why:      A missing key means the file cannot be stat'd, so return unity gain.
        /** Optional stable u64 cache key for this track. */
        val fingerprint: Long? = TrackFingerprint.of(appContext, parsed)
        if (fingerprint == null) {
            // What:     `return UNITY_GAIN` when there is no fingerprint.
            // Why:      Without a key the native cache cannot memoize; play unattenuated.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (fingerprint == null) return UNITY_GAIN;
            // ```
            return UNITY_GAIN
        }
        // What:     `val descriptor = openDescriptor(uri) ?: return UNITY_GAIN` opens a read-only
        //           fd for the track, returning unity gain if it cannot be opened.
        // Why:      The native resolver decodes through this fd on a cache miss.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const descriptor = openDescriptor(uri); if (!descriptor) return UNITY_GAIN;
        // ```
        /** Read-only descriptor for the track, or null when the provider cannot open it. */
        val descriptor = openDescriptor(uri) ?: return UNITY_GAIN
        // What:     `descriptor.use { pfd -> NativeBridge.nativeResolveGain(TruePeakGain.handle(appContext),
        //           pfd.fd, fingerprint) }` resolves the gain natively and closes the fd after.
        //           `nativeResolveGain` returns the gain directly (a cache hit reuses the stored
        //           decision; a miss decodes, caches, and returns), never negative: it falls back
        //           to the safe ceiling gain on any error. The gain math and the cache now live in
        //           Rust, so Kotlin computes nothing here. `TruePeakGain` owns the one process-wide
        //           service handle, shared with the background sweep.
        // Why:      One native call replaces the old Kotlin cache get/put/flush + gain math.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return descriptor.use((pfd) => nativeResolveGain(TruePeakGain.handle(appContext), pfd.fd, fingerprint));
        // ```
        return descriptor.use { pfd ->
            NativeBridge.nativeResolveGain(TruePeakGain.handle(appContext), pfd.fd, fingerprint)
        }
    }

    // What:     `private fun openDescriptor(uri: String): ParcelFileDescriptor? = try { ... } catch (failure:
    //           Exception) { ... }`
    //           declares a private function returning a NULLABLE `ParcelFileDescriptor?` via an EXPRESSION
    //           BODY that is a TRY EXPRESSION (its value is the `try` block's value, or `null` from the catch).
    // Why:      Open a read-only descriptor for the track: a bare absolute path via
    //           `ParcelFileDescriptor.open`, everything else (the `content://` URIs the library actually
    //           yields) via the content resolver. Returns `null` when it cannot be opened.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private openDescriptor(uri: string): ParcelFileDescriptor | null {
    //   try {
    //     return uri.startsWith("/")
    //       ? ParcelFileDescriptor.open(new File(uri), ParcelFileDescriptor.MODE_READ_ONLY)
    //       : this.appContext.contentResolver.openFileDescriptor(Uri.parse(uri), "r");
    //   } catch (e) {
    //     console.warn(LOG_TAG, `openDescriptor failed for ${uri}`, e);
    //     return null;
    //   }
    // }
    // ```
    /**
     * Defines open descriptor behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun openDescriptor(uri: String): ParcelFileDescriptor? =
        try {
            // What:     `if (uri.startsWith("/")) { ... } else { ... }` is an IF EXPRESSION: it evaluates to
            //           one of the two branch values (which becomes the `try`'s value). `uri.startsWith("/")`
            //           tests whether the URI is a bare absolute filesystem path.
            // Why:      An absolute path is opened directly as a file; anything else is a provider URI.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // uri.startsWith("/") ? <file path branch> : <content resolver branch>
            // ```
            if (uri.startsWith("/")) {
                // What:     `ParcelFileDescriptor.open(File(uri), ParcelFileDescriptor.MODE_READ_ONLY)`
                //           opens a `File(uri)` (constructed from the path, no `new`) in read-only mode and
                //           returns a `ParcelFileDescriptor`. This is the `then`-branch value.
                // Why:      Open a bare absolute path directly.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // ParcelFileDescriptor.open(new File(uri), ParcelFileDescriptor.MODE_READ_ONLY)
                // ```
                ParcelFileDescriptor.open(File(uri), ParcelFileDescriptor.MODE_READ_ONLY)
            } else {
                // What:     `appContext.contentResolver.openFileDescriptor(Uri.parse(uri), "r")` parses the
                //           string to a `Uri` and opens it read-only (`"r"`) through the content resolver,
                //           returning a nullable `ParcelFileDescriptor?`. This is the `else`-branch value.
                // Why:      Open a `content://` (provider) URI via the resolver.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.appContext.contentResolver.openFileDescriptor(Uri.parse(uri), "r")
                // ```
                appContext.contentResolver.openFileDescriptor(Uri.parse(uri), "r")
            }
        } catch (expectedFailure: Exception) {
            // What:     `Log.w(LOG_TAG, "openDescriptor failed for $uri", expectedFailure)` logs the open failure
            //           with the throwable.
            // Why:      Make the failure visible before falling back to `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.warn(LOG_TAG, `openDescriptor failed for ${uri}`, e);
            // ```
            Log.w(LOG_TAG, "openDescriptor failed for $uri", expectedFailure)
            // What:     `null` is the catch block's value (and thus the function's return on failure): the
            //           NULL variant of the `ParcelFileDescriptor?` return type.
            // Why:      Signal "could not open" to the caller without throwing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return null;
            // ```
            null
        }

    // What:     `override fun play() { ... }` implements `AudioEngine.play`.
    // Why:      Resume playback, but only if audio focus is granted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // play(): void { if (this.requestFocus()) NativeBridge.nativeEnginePlay(this.handle); }
    // ```
    /**
     * Defines play behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    override fun play() {
        /** The load still opening its file, whose start state this call changes. */
        val load: PendingLoad? = pendingLoad
        if (load != null) {
            load.play = true
            return
        }
        // What:     `if (requestFocus()) { NativeBridge.nativeEnginePlay(handle) }` requests focus (a side
        //           effect) and plays only if it was granted.
        // Why:      Do not play over another app that holds focus.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.requestFocus()) NativeBridge.nativeEnginePlay(this.handle);
        // ```
        if (requestFocus()) {
            // What:     `NativeBridge.nativeEnginePlay(handle)` calls the native play via JNI.
            // Why:      Begin/resume native playback.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // NativeBridge.nativeEnginePlay(this.handle);
            // ```
            NativeBridge.nativeEnginePlay(handle)
        }
    }

    // What:     `override fun pause() { ... }` implements `AudioEngine.pause`.
    // Why:      Pause playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // pause(): void { NativeBridge.nativeEnginePause(this.handle); }
    // ```
    /**
     * Defines pause behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    override fun pause() {
        pendingLoad?.play = false
        // What:     `NativeBridge.nativeEnginePause(handle)` calls the native pause via JNI.
        // Why:      Pause native playback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // NativeBridge.nativeEnginePause(this.handle);
        // ```
        NativeBridge.nativeEnginePause(handle)
    }

    // What:     `override fun seekTo(positionSec: Double) { ... }` implements `AudioEngine.seekTo`, taking
    //           a `Double` (64-bit) seconds position (sibling `Float` declined: seconds want the wider type).
    // Why:      Seek the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seekTo(positionSec: number): void { NativeBridge.nativeEngineSeek(this.handle, positionSec); }
    // ```
    /**
     * Defines seek to behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    override fun seekTo(positionSec: Double) {
        /** The load still opening its file, which takes the seek once the track is loaded. */
        val load: PendingLoad? = pendingLoad
        if (load != null) {
            load.seekSec = positionSec
            return
        }
        // What:     `NativeBridge.nativeEngineSeek(handle, positionSec)` calls the native seek via JNI,
        //           passing seconds directly (the native side handles the unit).
        // Why:      Move playback to `positionSec`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // NativeBridge.nativeEngineSeek(this.handle, positionSec);
        // ```
        NativeBridge.nativeEngineSeek(handle, positionSec)
    }

    // What:     `override fun setVolume(volume: Float) { ... }` implements `AudioEngine.setVolume`, taking
    //           a `Float` (32-bit) `0.0..1.0` volume.
    // Why:      Set the user volume on the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setVolume(volume: number): void { NativeBridge.nativeEngineSetVolume(this.handle, volume); }
    // ```
    /**
     * Defines set volume behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun setVolume(volume: Float) {
        // What:     `userVolume = volume` remembers the volume the user chose.
        // Why:      A duck in progress must not lose it, and the regained focus restores it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.userVolume = volume;
        // ```
        userVolume = volume
        applyVolume()
    }

    // What:     `private fun applyVolume()` sends the user volume, lowered while ducked, to the native engine.
    // Why:      The one place the engine's volume is set, so a volume change and a duck change cannot disagree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private applyVolume(): void {
    //   NativeBridge.nativeEngineSetVolume(this.handle, volumeWhileDucked(this.userVolume, this.ducked));
    // }
    // ```
    /** Applies [userVolume], lowered while [ducked], to the native engine. */
    private fun applyVolume() {
        NativeBridge.nativeEngineSetVolume(handle, volumeWhileDucked(userVolume = userVolume, ducked = ducked))
    }

    // What:     `override fun positionSec(): Double = NativeBridge.nativeEnginePositionSec(handle)` implements
    //           `AudioEngine.positionSec` as an EXPRESSION BODY delegating straight to the native getter.
    // Why:      Report the current position (seconds) from the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // positionSec(): number { return NativeBridge.nativeEnginePositionSec(this.handle); }
    // ```
    /**
     * Defines position sec behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun positionSec(): Double = if (unloaded) {
        0.0
    } else {
        pendingLoad?.let { load -> load.seekSec ?: 0.0 } ?: NativeBridge.nativeEnginePositionSec(handle)
    }

    // What:     `override fun durationSec(): Double = NativeBridge.nativeEngineDurationSec(handle)`
    //           implements `AudioEngine.durationSec` as an expression body delegating to the native getter.
    // Why:      Report the track duration (seconds) from the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // durationSec(): number { return NativeBridge.nativeEngineDurationSec(this.handle); }
    // ```
    /**
     * Defines duration sec behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun durationSec(): Double =
        if (unloaded || pendingLoad != null) 0.0 else NativeBridge.nativeEngineDurationSec(handle)

    // What:     `override fun playWhenReady(): Boolean = NativeBridge.nativeEnginePlayWhenReady(handle)`
    //           implements `AudioEngine.playWhenReady` as an expression body delegating to the native getter.
    // Why:      Report the intended play state ("should play once ready") from the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playWhenReady(): boolean { return NativeBridge.nativeEnginePlayWhenReady(this.handle); }
    // ```
    /**
     * Defines play when ready behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    override fun playWhenReady(): Boolean = pendingLoad?.play ?: NativeBridge.nativeEnginePlayWhenReady(handle)

    // What:     `override fun setOnPlayingChanged(callback: (Boolean) -> Unit) { ... }` implements the setter
    //           for the play-state callback (`callback: (Boolean) -> Unit` is a non-nullable function-type param).
    // Why:      Let the controller register its play-state handler.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setOnPlayingChanged(callback: (playing: boolean) => void): void { this.onPlayingChanged = callback; }
    // ```
    /**
     * Defines set on playing changed behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    override fun setOnPlayingChanged(callback: (Boolean) -> Unit) {
        // What:     `onPlayingChanged = callback` stores the callback in the nullable field.
        // Why:      Remember it so the poller can invoke it on state changes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.onPlayingChanged = callback;
        // ```
        onPlayingChanged = callback
    }

    // What:     `override fun setOnTrackEnded(callback: () -> Unit) { ... }` implements the setter for the
    //           natural-end callback (`callback: () -> Unit` is a no-arg void function param).
    // Why:      Let the controller register its track-ended handler.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setOnTrackEnded(callback: () => void): void { this.onTrackEnded = callback; }
    // ```
    /**
     * Defines set on track ended behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    override fun setOnTrackEnded(callback: () -> Unit) {
        // What:     `onTrackEnded = callback` stores the callback in the nullable field.
        // Why:      Remember it so the poller can invoke it on a natural end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.onTrackEnded = callback;
        // ```
        onTrackEnded = callback
    }

    /** Pauses the native engine, zeroes position and duration until the next load, and cancels a pending load. */
    override fun unload() {
        Log.i(LOG_TAG, "RustEngine.unload")
        loadGeneration += 1
        pendingLoad = null
        unloaded = true
        NativeBridge.nativeEnginePause(handle)
    }

    /** Stores the single load-failure handler; a second call replaces it. */
    override fun setOnLoadFailed(callback: (uri: String, reason: LoadFailure) -> Unit) {
        onLoadFailed = callback
    }

    // What:     `override fun release() { ... }` implements `AudioEngine.release`: tear the engine down.
    // Why:      Stop the poller, cancel the gain scope, abandon focus, unregister the receiver, and release
    //           the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // release(): void { ... }
    // ```
    /**
     * Defines release behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    override fun release() {
        // What:     `poller.removeCallbacks(pollTask)` cancels any scheduled runs of the self-rescheduling
        //           poll task.
        // Why:      Stop the poll loop so it does not run after release.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.poller.removeCallbacks(pollTask);
        // ```
        poller.removeCallbacks(pollTask)
        // What:     `resolveScope.cancel()` cancels the coroutine scope and any pending gain resolution.
        // Why:      So a pending measure cannot outlive the engine.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.resolveScope.cancel();
        // ```
        resolveScope.cancel()
        // What:     `resumeOnFocusGain = false` clears the resume flag.
        // Why:      No resume should happen during/after teardown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.resumeOnFocusGain = false;
        // ```
        resumeOnFocusGain = false
        // What:     `audioManager.abandonAudioFocusRequest(focusRequest)` gives up the audio focus the
        //           engine held.
        // Why:      Release focus back to the system on teardown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.audioManager.abandonAudioFocusRequest(focusRequest);
        // ```
        audioManager.abandonAudioFocusRequest(focusRequest)
        // What:     `try { appContext.unregisterReceiver(noisyReceiver) } catch (alreadyUnregistered:
        //           IllegalArgumentException) { ... }`
        //           unregisters the becoming-noisy receiver, catching the `IllegalArgumentException` Android
        //           throws when it was never registered (or already unregistered).
        // Why:      Stop listening for headphone-unplug; tolerate a double-release.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try {
        //   appContext.unregisterReceiver(noisyReceiver);
        // } catch (e) {
        //   console.warn(LOG_TAG, "noisy receiver already unregistered", e);
        // }
        // ```
        try {
            appContext.unregisterReceiver(noisyReceiver)
        } catch (alreadyUnregistered: IllegalArgumentException) {
            // What:     `Log.w(LOG_TAG, "noisy receiver already unregistered", alreadyUnregistered)` logs the
            //           benign double-unregister at WARN with the exception.
            // Why:      Benign: `release()` ran twice, or the receiver was never registered; nothing to undo.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.warn(LOG_TAG, "noisy receiver already unregistered", e);
            // ```
            Log.w(LOG_TAG, "noisy receiver already unregistered", alreadyUnregistered)
        }
        // What:     `NativeBridge.nativeEngineRelease(handle)` releases the native engine (stops its worker,
        //           frees resources) via JNI.
        // Why:      Free native resources.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // NativeBridge.nativeEngineRelease(this.handle);
        // ```
        NativeBridge.nativeEngineRelease(handle)
        // What:     `handle = 0L` resets the stored handle to `0` (a `Long` zero), marking it invalid.
        // Why:      So later calls (e.g. a stray poll or gain-apply) see `handle == 0L` and skip touching the
        //           freed native engine.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.handle = 0n; // mark released
        // ```
        handle = 0L
    }

    // What:     `private fun requestFocus(): Boolean = audioManager.requestAudioFocus(focusRequest) ==
    //           AudioManager.AUDIOFOCUS_REQUEST_GRANTED`
    //           declares a private function returning `Boolean` as an EXPRESSION BODY: it requests focus and
    //           compares the result against the "granted" constant with `==`.
    // Why:      Request media audio focus; returns `true` when focus was granted (so playback may start).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private requestFocus(): boolean {
    //   return this.audioManager.requestAudioFocus(focusRequest) === AudioManager.AUDIOFOCUS_REQUEST_GRANTED;
    // }
    // ```
    /**
     * Defines request focus behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun requestFocus(): Boolean =
        audioManager.requestAudioFocus(focusRequest) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED

    // What:     `private fun onFocusChange(change: Int) { ... }` declares a private function taking the
    //           focus-change code (`Int`) and returning `Unit` (void).
    // Why:      React to a system audio-focus change so a phone call or another media app pauses (and a transient
    //           interruption resumes) this engine, and a notification sound or navigation prompt ducks it (D134).
    //           A permanent loss pauses and abandons focus; a transient loss pauses and arms resume-on-gain; a
    //           can-duck loss lowers the volume and keeps playing; a gain restores the volume and resumes only if a
    //           transient loss had paused us.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private onFocusChange(change: number): void { ... }
    // ```
    /**
     * Defines on focus change behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun onFocusChange(change: Int) {
        // What:     `when (change) { ... }` is a WHEN STATEMENT: Kotlin's switch-like multi-way branch on the
        //           value of `change`. Each `LABEL -> { ... }` arm runs when `change` equals that label; a
        //           comma-separated label list (`A, B -> ...`) matches ANY of them. There is no `else` arm, so
        //           unlisted focus codes are ignored.
        // Why:      Dispatch on the specific focus-change kind (permanent loss, transient loss, gain).
        // Gotcha:   `when` arms do NOT fall through (no `break` needed), unlike a C/TS `switch`; each arm is
        //           self-contained.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (change) {
        //   case AudioManager.AUDIOFOCUS_LOSS: { ...; break; }
        //   case AudioManager.AUDIOFOCUS_LOSS_TRANSIENT:
        //   case AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK: { ...; break; }
        //   case AudioManager.AUDIOFOCUS_GAIN: { ...; break; }
        // }
        // ```
        when (change) {
            // What:     `AudioManager.AUDIOFOCUS_LOSS -> { ... }` is the arm for a PERMANENT focus loss.
            // Why:      Another app took focus for good (e.g. a different media app); stop and let go of focus.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case AudioManager.AUDIOFOCUS_LOSS: { this.resumeOnFocusGain = false;
            // NativeBridge.nativeEnginePause(this.handle); this.audioManager.abandonAudioFocusRequest(focusRequest);
            // break; }
            // ```
            AudioManager.AUDIOFOCUS_LOSS -> {
                // What:     `resumeOnFocusGain = false` clears the resume flag.
                // Why:      A permanent loss should not auto-resume later.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.resumeOnFocusGain = false;
                // ```
                resumeOnFocusGain = false
                // What:     `NativeBridge.nativeEnginePause(handle)` pauses native playback via JNI.
                // Why:      Stop playing on the permanent loss.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // NativeBridge.nativeEnginePause(this.handle);
                // ```
                NativeBridge.nativeEnginePause(handle)
                // What:     `ducked = false` and `applyVolume()` put the user volume back.
                // Why:      Focus is given up for good, so no gain event will end a duck that was in progress.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.ducked = false; this.applyVolume();
                // ```
                ducked = false
                applyVolume()
                // What:     `audioManager.abandonAudioFocusRequest(focusRequest)` gives up focus.
                // Why:      We are not resuming, so release focus to the system.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.audioManager.abandonAudioFocusRequest(focusRequest);
                // ```
                audioManager.abandonAudioFocusRequest(focusRequest)
            }
            // What:     `AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> { ... }` is the arm for a brief
            //           interruption that allows the music to go on quietly: a notification ping or a navigation
            //           prompt.
            // Why:      Lower the volume for the interruption instead of stopping the music (D134).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK: { this.ducked = true; this.applyVolume(); break; }
            // ```
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> {
                // What:     `ducked = true` and `applyVolume()` lower the volume and keep playing.
                // Why:      A notification sound or a navigation prompt asks to be heard over the music, not instead
                //           of it. Android ducks other apps' output itself, but it never ducks this engine's AAudio
                //           output, and `setWillPauseWhenDucked(true)` makes it tell this listener, so the engine
                //           lowers its own volume (D134). A paused player stays paused and is lowered too, so a
                //           Play pressed during the interruption does not blare over it.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.ducked = true; this.applyVolume();
                // ```
                ducked = true
                applyVolume()
            }
            // What:     `AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> { ... }` is the arm for a transient loss that does
            //           not allow ducking.
            // Why:      A phone call or a voice recording needs the speaker: pause now and remember to resume if we
            //           were playing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case AudioManager.AUDIOFOCUS_LOSS_TRANSIENT: { this.resumeOnFocusGain =
            // NativeBridge.nativeEngineIsPlaying(this.handle); NativeBridge.nativeEnginePause(this.handle); break; }
            // ```
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> {
                // What:     `resumeOnFocusGain = NativeBridge.nativeEngineIsPlaying(handle)` records whether we
                //           are currently playing (so we know whether to resume on a later gain).
                // Why:      Only resume after a transient loss if we were actually playing when it hit.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.resumeOnFocusGain = NativeBridge.nativeEngineIsPlaying(this.handle);
                // ```
                resumeOnFocusGain = NativeBridge.nativeEngineIsPlaying(handle)
                // What:     `NativeBridge.nativeEnginePause(handle)` pauses native playback via JNI.
                // Why:      Pause for the duration of the transient interruption.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // NativeBridge.nativeEnginePause(this.handle);
                // ```
                NativeBridge.nativeEnginePause(handle)
            }
            // What:     `AudioManager.AUDIOFOCUS_GAIN -> { ... }` is the arm for (re)gaining focus.
            // Why:      The interruption ended; resume only if a transient loss had paused us.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case AudioManager.AUDIOFOCUS_GAIN: { if (this.resumeOnFocusGain) { this.resumeOnFocusGain = false;
            // NativeBridge.nativeEnginePlay(this.handle); } break; }
            // ```
            AudioManager.AUDIOFOCUS_GAIN -> {
                // What:     `ducked = false` and `applyVolume()` put the user volume back.
                // Why:      The interruption that lowered the volume has ended.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.ducked = false; this.applyVolume();
                // ```
                ducked = false
                applyVolume()
                // What:     `if (resumeOnFocusGain) { ... }` resumes only when the resume flag is set.
                // Why:      Do not auto-start playback the user never had going.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (this.resumeOnFocusGain) { ... }
                // ```
                if (resumeOnFocusGain) {
                    // What:     `resumeOnFocusGain = false` clears the flag now that we are resuming.
                    // Why:      One-shot: do not resume again on a later gain.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.resumeOnFocusGain = false;
                    // ```
                    resumeOnFocusGain = false
                    // What:     `NativeBridge.nativeEnginePlay(handle)` resumes native playback via JNI.
                    // Why:      Continue the track that the transient loss paused.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // NativeBridge.nativeEnginePlay(this.handle);
                    // ```
                    NativeBridge.nativeEnginePlay(handle)
                }
            }
        }
    }

    // What:     `private fun poll() { ... }` declares a private function (no params) returning `Unit`.
    // Why:      Sample the native state and fire the push callbacks on a transition. `onPlayingChanged` is
    //           EDGE-TRIGGERED on the play state; `onTrackEnded` fires once on the RISING EDGE of `ended` and
    //           rearms when the native side clears `ended` (the worker resets it when the next track loads), so
    //           a track-change handoff cannot double-fire.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private poll(): void { ... }
    // ```
    /**
     * Defines poll behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    private fun poll() {
        // What:     `if (handle == 0L) { return }` early-returns when the engine is released (`handle == 0L`).
        // Why:      Do not call into a freed native engine.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.handle === 0n) return;
        // ```
        if (handle == 0L) {
            return
        }
        // What:     `val playing: Boolean = NativeBridge.nativeEngineIsPlaying(handle)` reads the native
        //           play state via JNI into a read-only `Boolean`.
        // Why:      Compare against the last reported state to detect a transition.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const playing = NativeBridge.nativeEngineIsPlaying(this.handle);
        // ```
        /**
         * Defines playing value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val playing: Boolean = NativeBridge.nativeEngineIsPlaying(handle)
        // What:     `if (playing != lastPlaying) { ... }` fires only when the play state CHANGED since the
        //           last poll (`!=`).
        // Why:      Edge-trigger: notify only on a real transition, not every poll.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (playing !== this.lastPlaying) { this.lastPlaying = playing; this.onPlayingChanged?.(playing); }
        // ```
        if (playing != lastPlaying) {
            // What:     `lastPlaying = playing` records the new state.
            // Why:      So the next poll compares against it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.lastPlaying = playing;
            // ```
            lastPlaying = playing
            // What:     `onPlayingChanged?.invoke(playing)` safe-calls the nullable callback: invoke only if set.
            // Why:      Notify the controller of the play-state change.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.onPlayingChanged?.(playing);
            // ```
            onPlayingChanged?.invoke(playing)
        }
        // What:     `val ended: Boolean = NativeBridge.nativeEngineIsEnded(handle)` reads the native
        //           track-ended flag via JNI.
        // Why:      Detect the rising edge of "track ended".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const ended = NativeBridge.nativeEngineIsEnded(this.handle);
        // ```
        /**
         * Defines ended value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val ended: Boolean = NativeBridge.nativeEngineIsEnded(handle)
        // What:     `if (ended && !endedHandled) { ... } else if (!ended) { ... }` is the rising-edge /
        //           rearm logic. The first arm fires once when `ended` is newly true (`ended && !endedHandled`);
        //           the `else if (!ended)` arm rearms (`endedHandled = false`) once native `ended` clears.
        // Why:      Fire `onTrackEnded` exactly once per ended track, rearming only when the native flag
        //           actually clears (the worker clears it when the next track loads), so a track-change handoff
        //           cannot double-fire.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (ended && !this.endedHandled) { this.endedHandled = true; this.onTrackEnded?.(); }
        // else if (!ended) { this.endedHandled = false; }
        // ```
        if (ended && !endedHandled) {
            // What:     `endedHandled = true` marks this ended state as handled.
            // Why:      Prevent re-firing on subsequent polls while `ended` stays true.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.endedHandled = true;
            // ```
            endedHandled = true
            // What:     `onTrackEnded?.invoke()` safe-calls the nullable no-arg callback: invoke only if set.
            // Why:      Tell the controller the track ended so it can advance.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.onTrackEnded?.();
            // ```
            onTrackEnded?.invoke()
        } else if (!ended) {
            // What:     `endedHandled = false` rearms the one-shot for the next track.
            // Why:      Once native `ended` clears (next track loaded), allow the next end to fire again.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.endedHandled = false;
            // ```
            endedHandled = false
        }
        // What:     `if (!unloaded && pendingLoad == null) { ... }` reads the worker's load failure number.
        // Why:      The native `load` call returned once the load was queued, so a file that opens but does not
        //           decode (a truncated or renamed file) fails later on the worker; its number is the only sign.
        //           A newer load still opening its descriptor, or an unloaded engine, makes an older failure stale.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.unloaded && this.pendingLoad === null) {
        //   const failedUri = this.queuedLoads.failureToReport(nativeEngineFailedLoad(this.handle));
        //   if (failedUri !== null) this.onLoadFailed?.(failedUri, "UNREADABLE");
        // }
        // ```
        if (!unloaded && pendingLoad == null) {
            /** File of the latest queued load when the worker has newly failed to decode it, else null. */
            val failedUri: String? = queuedLoads.failureToReport(NativeBridge.nativeEngineFailedLoad(handle))
            if (failedUri != null) {
                Log.w(LOG_TAG, "the engine worker could not decode ${failedUri.substringAfterLast('/')}")
                onLoadFailed?.invoke(failedUri, LoadFailure.UNREADABLE)
            }
        }
    }

    /**
     * Defines companion object for this music-player component; the TypeScript-oriented notes above explain its
     * shared role.
     */
    // What:     `companion object { ... }` declares the class's static-like member bag.
    // Why:      Hosts the log tag, poll cadence, and unity-gain constants shared by all instances.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // static readonly LOG_TAG/POLL_MS/UNITY_GAIN on the class
    // ```
    companion object {
        // What:     `private const val LOG_TAG: String = "RustEngine"` declares a private compile-time
        //           `String` constant.
        // Why:      Logcat tag for this engine's lines.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private static readonly LOG_TAG = "RustEngine";
        // ```
        /**
         * Defines log tag value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        private const val LOG_TAG: String = "RustEngine"

        // What:     `private const val POLL_MS: Long = 200L` declares a private compile-time `Long` (64-bit)
        //           constant `200` (the `L` suffix makes it `Long`, the unit `Handler.postDelayed` expects).
        //           Sibling `Int` is declined because the delay API takes a `Long`.
        // Why:      Poll cadence (milliseconds) for the play/ended state, matching the UI's position poll.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private static readonly POLL_MS = 200; // milliseconds
        // ```
        /**
         * Defines poll ms value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        private const val POLL_MS: Long = 200L

        /** How long a track may wait for its descriptor before the load is reported as stalled. */
        private const val OPEN_TIMEOUT_MS: Long = 8_000L

        // What:     `private const val UNITY_GAIN: Float = 1.0f` declares a private compile-time `Float`
        //           (32-bit; the `f` suffix) constant `1.0`.
        // Why:      Unity (passthrough) normalization gain, applied until a track's gain is resolved.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private static readonly UNITY_GAIN = 1.0;
        // ```
        /**
         * Defines unity gain value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        private const val UNITY_GAIN: Float = 1.0f
    }
}
