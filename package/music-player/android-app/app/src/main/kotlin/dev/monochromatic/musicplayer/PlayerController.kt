// ============================================================================
// File summary (folds in the old KDoc that sat on `class PlayerController`)
// ============================================================================
//
// This file is the BRAIN. `PlayerController` orchestrates the queue,
// pagination, shuffle/scope, and transport on top of an `AudioEngine`,
// mirroring the desktop's controller. It owns the ported `Queue` and the
// paginated view, drives the engine to play the current track, follows the
// playing track's page, and advances on a natural end. The state the UI renders
// is exposed as the Compose-observable `uiState`; position/duration are read
// live via `positionSec`/`durationSec`. It is created and called on the main
// thread.
//
// One thing to keep in mind as a TS reader: `uiState` is a PROPERTY-DELEGATED
// Compose state (the `by mutableStateOf(...)` line). Reading it gives the
// current snapshot; assigning a new snapshot makes the UI recompose. Treat it
// as a reactive signal/ref whose get/set are transparent.
// ============================================================================

// What:     `package dev.monochromatic.musicplayer` names the namespace this brain
//           lives in, reachable elsewhere as
//           `dev.monochromatic.musicplayer.PlayerController`.
// Why:      So the service, the UI, and `BrainPlayer` can refer to it.
//
// In TS you'd write (pseudocode):
// ```ts
// // No `package` line in TS; the file path is the module identity.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.util.Log` pulls in `Log`, Android's logger.
// Why:      We log when a track ends and we advance.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import androidx.compose.runtime.getValue` imports the `getValue` OPERATOR
//           function. It is what makes the `by mutableStateOf(...)` property DELEGATION
//           READABLE: when a property is delegated with `by`, reading it calls the
//           delegate's `getValue`, and that operator must be in scope.
// Why:      The `uiState` property below uses `by`, so reading `uiState` needs this
//           import to resolve.
//
// In TS you'd write (pseudocode):
// ```ts
// // no import — TS getters don't need an operator function in scope
// ```
import androidx.compose.runtime.getValue

// What:     `import androidx.compose.runtime.mutableStateOf` imports `mutableStateOf(x)`,
//           the Compose factory that creates an OBSERVABLE state holder seeded with `x`.
//           When its value changes, any Compose UI that read it recomposes.
// Why:      `uiState` is delegated to a `mutableStateOf(...)` so the screen re-renders
//           when the brain swaps in a new snapshot.
//
// In TS you'd write (pseudocode):
// ```ts
// import { signal } from "@reactive/core"; // mutableStateOf(x) ~ signal(x)
// ```
import androidx.compose.runtime.mutableStateOf

// What:     `import androidx.compose.runtime.setValue` imports the `setValue` OPERATOR
//           function, the WRITE counterpart of `getValue`: assigning to a `by`-delegated
//           property calls the delegate's `setValue`, and that operator must be in scope.
// Why:      We ASSIGN new snapshots to `uiState`, which (because of `by`) needs this
//           import to resolve.
//
// In TS you'd write (pseudocode):
// ```ts
// // no import — TS setters don't need an operator function in scope
// ```
import androidx.compose.runtime.setValue

// What:     `import dev.monochromatic.musicplayer.core.Page` imports the `Page` type from
//           the `.core` package: one browsable tab (a label plus its entries).
// Why:      `pages` is a `List<Page>`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Page } from "./core/Page";
// ```
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import dev.monochromatic.musicplayer.core.BuiltLibrary
import dev.monochromatic.musicplayer.core.Page

// What:     `import dev.monochromatic.musicplayer.core.Queue` imports the ported `Queue`
//           type (the play queue: ordered tracks plus a cursor, with shuffle/repeat).
// Why:      The controller owns a `Queue`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Queue } from "./core/Queue";
// ```
import dev.monochromatic.musicplayer.core.Queue

// What:     `import dev.monochromatic.musicplayer.core.Session` imports the pure
//           "where the user left off" model (selected track + settings + position).
// Why:      `currentSession` returns one and `applySettings`/`finishLoad` take one, so the
//           service can persist and restore via `SessionStore`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Session } from "./core/Session";
// ```
import dev.monochromatic.musicplayer.core.Session

// What:     `import dev.monochromatic.musicplayer.core.PlaybackMode` imports the
//           three-value enum `PlaybackMode` (`OFF`/`WITHIN_PAGE`/`ALL`).
// Why:      `setPlaybackMode` takes a `PlaybackMode`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PlaybackMode } from "./core/PlaybackMode";
// ```
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.RowActivation
import dev.monochromatic.musicplayer.core.rowActivationFor

import dev.monochromatic.musicplayer.core.buildLibrary
/** Stable page identity distinguishes duplicate displayed labels. */
import dev.monochromatic.musicplayer.core.pageIdentity

// What:     `import dev.monochromatic.musicplayer.core.TrackMeasure` imports the per-track duration and peak value.
// Why:      The controller publishes the measures the library rows read.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TrackMeasure } from "./core/TrackMeasure";
// ```
import dev.monochromatic.musicplayer.core.TrackMeasure
import dev.monochromatic.musicplayer.core.LoadOrigin
import dev.monochromatic.musicplayer.core.UnplayableReport
import dev.monochromatic.musicplayer.core.UnplayableWalk
import dev.monochromatic.musicplayer.core.trackFileNameOf

// What:     `class PlayerController(private val engine: AudioEngine) { ... }` declares a
//           class with a PRIMARY CONSTRUCTOR. `private val engine: AudioEngine` is a
//           constructor parameter that is ALSO a private read-only PROPERTY (the `val`
//           on a constructor param is Kotlin shorthand for "store this argument as a
//           field"); `private` hides it outside the class.
// Why:      The controller drives the concrete `AudioEngine` it is handed; storing it as a
//           private field lets every method reach it.
//
// In TS you'd write (pseudocode):
// ```ts
// class PlayerController {
//   constructor(private readonly engine: AudioEngine) {}
//   // ...state and methods below...
// }
// ```
/**
 * Defines player controller type for this music-player component; the TypeScript-oriented notes above explain
 * its role.
 */
class PlayerController(private val engine: AudioEngine) {
    // What:     `private val queue: Queue = Queue.new()` declares a private read-only
    //           field `queue`, built by the FACTORY `Queue.new()` (a companion-object
    //           function on `Queue`, not a constructor; no `new` keyword in Kotlin).
    // Why:      The controller owns one play queue seeded from the wall clock.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly queue: Queue = Queue.new();
    // ```
    /**
     * Defines queue value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val queue: Queue = Queue.new()
    // What:     `private var pages: List<Page> = emptyList()` declares a private,
    //           REASSIGNABLE (`var`) field of read-only list type `List<Page>` (sibling
    //           `MutableList<Page>`), initialised to the shared empty list `emptyList()`.
    // Why:      The current paginated view; replaced wholesale on each load.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private pages: readonly Page[] = [];
    // ```
    /**
     * Defines pages value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var pages: List<Page> = emptyList()

    /** Tab-order page of each load-order index, which turns "which page holds this track" into one read. */
    private var pageLookup: IntArray = IntArray(0)

    /** Stable identity of the page currently displayed by the UI. */
    private var selectedPageIdentity: String? = null

    // What:     `private var loadedUri: String? = null` declares a private, reassignable
    //           field of NULLABLE `String?` (the trailing `?` = "a `String` OR null"),
    //           initialised `null`.
    // Why:      The content URI of the track currently loaded in the engine, or null when
    //           nothing is loaded; lets `togglePlay` tell "resume" from "load and play".
    //           Deliberately the URI (a STABLE identity), NOT a load-order index: a live
    //           rescan re-derives the queue from disk, so the same track's index shifts.
    //           Keying on the URI survives that shift, where a stored index would silently
    //           point at the wrong track after `reconcileLibrary`. (Desktop's
    //           "reselect by identity, not index" decision, applied to Android URIs.)
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private loadedUri: string | null = null;
    // ```
    /**
     * Defines loaded uri value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var loadedUri: String? = null
    // What:     `private var isPlaying: Boolean = false` declares a private, reassignable
    //           boolean field, initialised `false`.
    // Why:      Mirrors the engine's playing state for the UI snapshot.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private isPlaying: boolean = false;
    // ```
    /**
     * Defines is playing value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var isPlaying: Boolean = false

    // What:     `private var isLoading: Boolean = true` declares a private, reassignable
    //           boolean field initialised `true`.
    // Why:      Whether a library load or folder scan is in flight. It STARTS true because
    //           the owning service begins loading as soon as it creates this controller,
    //           so the screen shows a loading notice from the first frame instead of
    //           flashing the empty-library message; `openLibrary` clears it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private isLoading: boolean = true;
    // ```
    /**
     * Defines is loading value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var isLoading: Boolean = true

    // What:     `private var uris: List<String> = emptyList()` declares a private,
    //           reassignable read-only `List<String>` field, initialised empty.
    // Why:      Playback URIs aligned by load-order index with the display paths fed to
    //           `queue`; the queue never reorders its track list (shuffle permutes a
    //           separate index list), so `uris[index]` is always the URI for the track the
    //           queue reports at that load-order index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private uris: readonly string[] = [];
    // ```
    /**
     * Defines uris value for this music-player component; the TypeScript-oriented notes above explain its source
     * and use.
     */
    private var uris: List<String> = emptyList()

    // What:     `var uiState: PlayerUiState by mutableStateOf(PlayerUiState(loading = true))`
    //           declares a PROPERTY whose storage is DELEGATED, via the `by` keyword, to a
    //           Compose `mutableStateOf(...)` holder. With `by`, reading `uiState` runs the
    //           delegate's `getValue` (the imported operator) to fetch the current value,
    //           and ASSIGNING `uiState = x` runs the delegate's `setValue` to store `x` AND
    //           trigger Compose recomposition. The initial value is
    //           `PlayerUiState(loading = true)` (a constructor call with the `loading`
    //           named argument).
    // Why:      It is the Compose-observable snapshot the screen renders; reassigning it by
    //           `refresh` is what repaints the UI. Seeding `loading = true` makes the first
    //           frame show the loading notice.
    // Gotcha:   `by` is PROPERTY DELEGATION, not assignment: `uiState` has no plain backing
    //           field; every get/set goes through the `mutableStateOf` holder. This is why
    //           `getValue`/`setValue` are imported at the top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly _uiState = signal<PlayerUiState>(makePlayerUiState({ loading: true }));
    // get uiState(): PlayerUiState { return this._uiState.value; }
    // // (setter is restricted to this class — see `private set` below)
    // ```
    /**
     * Defines ui state value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    var uiState: PlayerUiState by mutableStateOf(PlayerUiState(loading = true))
        // What:     `private set` restricts the SETTER of `uiState` to this class while the
        //           GETTER stays public. It is written on its own indented line directly
        //           under the property. There is no setter BODY; `private` just narrows
        //           visibility of the (delegated) setter.
        // Why:      Outside code may READ `uiState` (the UI does) but only the controller
        //           may REPLACE it, so the brain stays the single writer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private set uiState(v: PlayerUiState) { this._uiState.value = v; }
        // ```
        private set

    // What:     `var onStateChanged: (() -> Unit)? = null` declares a public, reassignable
    //           field whose type is a NULLABLE FUNCTION TYPE: `(() -> Unit)` is "a function
    //           taking no args and returning `Unit` (void)", and the outer `?` makes the
    //           whole thing nullable (a function OR null). Initialised `null`.
    // Why:      Invoked at the end of every `refresh` so a `MediaSession` projection
    //           (`BrainPlayer`) can re-pull its state on a discontinuity. Left null when no
    //           session is attached; the callback must POST `invalidateState` to the looper
    //           rather than run it synchronously, since some refreshes happen inside a
    //           player command.
    // Gotcha:   The parentheses matter: `(() -> Unit)?` is a nullable function; `() -> Unit?`
    //           would be a function returning a nullable `Unit` (a different type).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onStateChanged: (() => void) | null = null;
    // ```
    /**
     * Defines on state changed value for this music-player component; the TypeScript-oriented notes above
     * explain its source and use.
     */
    var onStateChanged: (() -> Unit)? = null

    /**
     * The process's Search service, which a wholesale library change feeds, or null when nothing searches (the unit
     * tests). A scan that streams feeds it batch by batch instead, through a `LibraryFeed`.
     */
    var searchHost: SearchHost? = null

    // What:     `var onPersist: (() -> Unit)? = null` declares a SECOND public, reassignable
    //           nullable no-arg callback, initialised `null`.
    // Why:      Invoked alongside `onStateChanged` at the end of every `refresh` so the
    //           service can persist the session (selected track + settings + position) on
    //           any meaningful change. It is SEPARATE from `onStateChanged` because that one
    //           is already owned by `BrainPlayer` (the MediaSession projection); a single
    //           callback cannot serve both, and overwriting `onStateChanged` would freeze the
    //           MediaSession. Left null when no persister is attached.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onPersist: (() => void) | null = null;
    // ```
    /**
     * Defines on persist value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    var onPersist: (() -> Unit)? = null

    // What:     `var loadFailure: LoadFailureNotice? by mutableStateOf(null)` is observable state naming the last track
    //           the engine could not open, with a serial so the same failure twice still changes the value.
    // Why:      The file open happens after `load` returned, so the host learns of a failure from this state and shows
    //           a
    //           message (M6).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // loadFailure: LoadFailureNotice | null = null; // observable
    // ```
    /** Last track the engine could not open, observed by the host to show a message; null before any failure. */
    var loadFailure: LoadFailureNotice? by mutableStateOf(null)
        private set

    /** Artist tags read so far by track URI, kept for the few tracks that played, for the media notification. */
    private val trackArtists: MutableMap<String, String> = LinkedHashMap()

    /** Timeline rows of the playback snapshot, kept between snapshots so a track change does not rebuild the queue. */
    private val snapshotItems: SnapshotItemCache = SnapshotItemCache()

    /** Called with the URI of each track that is loaded, so the service can read its tags off the main thread. */
    var onTrackLoaded: ((String) -> Unit)? = null

    /**
     * Called each time the player passes a track it was moving to by itself and could not open, with every file the
     * current walk has passed so far; the service turns it into one system notification (D122, D123).
     */
    var onUnplayableFiles: ((UnplayableReport) -> Unit)? = null

    /** How the load now in flight was started; a failure of it walks on unless the person chose the track. */
    private var latestLoadOrigin: LoadOrigin = LoadOrigin.TAP

    /** Whether the load now in flight asked to play, so a walk past an unreadable track keeps that intent. */
    private var latestLoadPlays: Boolean = false

    /** The failed loads of the walk in progress, which the next load begun afresh forgets. */
    private val unplayableWalk: UnplayableWalk = UnplayableWalk()

    /**
     * Records the artist tag of [uri] and republishes the media session so its notification shows the artist line.
     *
     * @param uri track whose tags were read
     * @param artist artist tag, or null when the file has none
     */
    fun setTrackArtist(uri: String, artist: String?) {
        if (artist == null) {
            return
        }
        trackArtists[uri] = artist
        while (trackArtists.size > MAX_TRACK_ARTISTS) {
            trackArtists.remove(trackArtists.keys.first())
        }
        onStateChanged?.invoke()
    }

    // What:     `init { ... }` is Kotlin's INITIALIZER BLOCK: code that runs once as part
    //           of constructing every instance, after the field initializers above. It has
    //           no method name.
    // Why:      Wire the engine's callbacks (playing-changed and track-ended) at
    //           construction so the controller reacts to engine events.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // (inside the constructor body)
    // ```
    init {
        // What:     `engine.setOnPlayingChanged { playing -> ... }` registers a callback by
        //           passing a TRAILING LAMBDA. `{ playing -> ... }` is the lambda; its
        //           single parameter `playing` (a `Boolean`) is named before `->`. Because
        //           the lambda is the last (only) argument, Kotlin lets it sit outside the
        //           parentheses.
        // Why:      Keep `isPlaying` in sync with the engine and repaint when it flips.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.engine.setOnPlayingChanged((playing) => {
        //   this.isPlaying = playing;
        //   this.refresh();
        // });
        // ```
        engine.setOnPlayingChanged { playing ->
            // What:     `isPlaying = playing` stores the engine's new playing flag into the
            //           field.
            // Why:      Record the play/pause change for the next snapshot.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.isPlaying = playing;
            // ```
            isPlaying = playing
            // What:     `refresh()` rebuilds `uiState` from the queue and pages.
            // Why:      Repaint the UI with the new playing state.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.refresh();
            // ```
            refresh()
        }
        // What:     `engine.setOnTrackEnded { ... }` registers a no-argument trailing-lambda
        //           callback for when a track finishes on its own.
        // Why:      A natural end should advance the queue and play the next track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.engine.setOnTrackEnded(() => {
        //   Log.i(LOG_TAG, "track ended; advancing");
        //   this.queue.advance(true);
        //   this.playCurrent();
        // });
        // ```
        engine.setOnTrackEnded {
            // What:     `Log.i(LOG_TAG, "track ended; advancing")` logs the natural end.
            // Why:      Trace auto-advance for verification.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.info(`[${LOG_TAG}] track ended; advancing`);
            // ```
            Log.i(LOG_TAG, "track ended; advancing")
            // What:     `handleNaturalEnd()` moves on after the track ended by itself.
            // Why:      Kept in a named method so unit tests can drive a natural end without
            //           the Android logger.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.handleNaturalEnd();
            // ```
            handleNaturalEnd()
        }
        engine.setOnLoadFailed { uri, reason ->
            Log.w(LOG_TAG, "track could not be opened reason=${reason.name} origin=${latestLoadOrigin.name}")
            if (latestLoadOrigin != LoadOrigin.TAP) {
                // A track the player moved to by itself is passed and reported, never an in-app message (D122, D123).
                if (uri == loadedUri) {
                    walkPastUnplayable(uri = uri, reason = reason)
                }
                return@setOnLoadFailed
            }
            // Forget the failed track as loaded, so Play loads it again instead of resuming the previous track.
            loadedUri = null
            loadFailure = LoadFailureNotice(uri = uri, reason = reason, serial = (loadFailure?.serial ?: 0) + 1)
            refresh()
        }
    }

    // What:     `fun beginLoad() { ... }` declares a public method, block body, returning
    //           `Unit`.
    // Why:      Mark a library load as in progress so the screen shows a loading notice
    //           instead of the empty-library message while a source scan runs. The owning
    //           service calls this before launching a (possibly slow) query or folder scan;
    //           `openLibrary` clears it on delivery.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // beginLoad(): void { this.isLoading = true; this.refresh(); }
    // ```
    /**
     * Defines begin load behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun beginLoad() {
        // What:     `isLoading = true` sets the loading flag.
        // Why:      Tell the next snapshot a scan is running.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.isLoading = true;
        // ```
        isLoading = true
        // What:     `refresh()` rebuilds the snapshot so the loading notice shows now.
        // Why:      Repaint immediately with the loading state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.refresh();
        // ```
        refresh()
    }

    // What:     `fun openLibrary(tracks: List<Track>)` replaces the library with `tracks`, in load order.
    // Why:      Keep their playback URIs in `uris`, feed their display paths to the queue (whose pagination trims the
    //           shared root and groups by folder, exactly as on the desktop), repaginate, and show the first page
    //           without starting playback (Android is tap-to-play). Clears the loading state, since this is a load's
    //           delivery. The derived lists are built here for a plain call and off the main thread for a streaming
    //           scan, which calls `openBuilt`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openLibrary(tracks: readonly Track[]): void { this.openBuilt(buildLibrary(tracks), true); }
    // ```
    /** Replaces the library with [tracks] and indexes them for Search. */
    fun openLibrary(tracks: List<Track>) {
        openBuilt(buildLibrary(tracks), feedSearch = true)
    }

    // What:     `fun openBuilt(built: BuiltLibrary, feedSearch: Boolean)` opens a library whose lists are built.
    // Why:      Installing is a few assignments, so a library built on a worker thread does not freeze the screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openBuilt(built: BuiltLibrary, feedSearch: boolean): void { /* install, clear selection, refresh */ }
    // ```
    /**
     * Opens a library that was built off the main thread: swaps it in, clears the selection and the loading flag.
     * Search is indexed from it only when [feedSearch] is true, because a streaming scan feeds it batch by batch.
     */
    fun openBuilt(built: BuiltLibrary, feedSearch: Boolean) {
        installParts(built)
        queue.clearSelection()
        if (feedSearch) {
            searchHost?.libraryReplaced(built.displayPaths, built.uris)
        }
        loadedUri = null
        isLoading = false
        refresh(followCurrent = true)
    }

    // What:     `private fun installParts(built: BuiltLibrary)` swaps in the derived lists of a library.
    // Why:      The three ways to install a library (open, restore, reconcile) share these assignments, and the
    //           expensive derivation already happened in the build.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private installParts(built: BuiltLibrary) {
    //   this.uris = built.uris; this.queue.setTracks(built.rawPaths, built); /* ... */
    // }
    // ```
    /** Adopts the URIs, queue, pages and page lookup of [built]. */
    private fun installParts(built: BuiltLibrary) {
        uris = built.uris
        queue.setTracks(built.rawPaths, built)
        pages = built.pages
        pageLookup = built.pageOfIndex
    }

    // What:     `private fun currentUri(): String? = queue.currentIndex()?.let { uris[it] }`
    //           declares a private helper returning the current track's content URI, or null,
    //           expression body.
    //           - `queue.currentIndex()` is the current load-order index (or null).
    //           - `?.let { uris[it] }` runs only when non-null, mapping the index through the
    //             parallel `uris` list; null short-circuits to null.
    // Why:      The engine is keyed by URI (a stable identity), so "is the current track the
    //           one loaded?" and "what should the session save as selected?" both need the
    //           current URI, not its shifting index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private currentUri(): string | null {
    //   const i = this.queue.currentIndex();
    //   return i === null ? null : this.uris[i];
    // }
    // ```
    /**
     * Defines current uri behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    private fun currentUri(): String? = queue.currentIndex()?.let { uris[it] }

    // What:     `private fun pageIndices(page: Int): List<Int>` maps one displayed
    //           page to its queue load-order indices.
    // Why:      The label shown in the playback-mode segment and the queue scope must
    //           always describe the same selected page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function pageIndices(page: number): readonly number[] { /* map entries */ }
    // ```
    /** Returns load-order indices for one displayed page, or an empty list. */
    private fun pageIndices(page: Int): List<Int> =
        pages.getOrNull(page)?.entries?.map { entry -> entry.index } ?: emptyList()

    // What:     `fun currentSession(): Session { ... }` declares a public method returning a
    //           snapshot of the persistable state as a `core.Session`, expression body.
    // Why:      The service calls this to persist via `SessionStore`: the selected track URI,
    //           the live resume position, and the settings. The Source Root is NOT included
    //           (Android re-resolves it each launch via `LibrarySource`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // currentSession(): Session {
    //   return makeSession({
    //     selected: this.currentUri(),
    //     positionSecs: this.engine.positionSec(),
    //     volume: this.uiState.volume,
    //     playbackMode: this.queue.playbackMode(),
    //   });
    // }
    // ```
    /**
     * Defines current session behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun currentSession(): Session = Session(
        selected = currentUri(),
        positionSecs = engine.positionSec(),
        volume = uiState.volume,
        playbackMode = queue.playbackMode(),
    )

    // What:     `fun applySettings(session: Session) { ... }` declares a public method taking a
    //           saved session, block body, `Unit` return.
    // Why:      The settings-only half of the old `restoreLibrary`, applied EARLY (before the
    //           streaming load starts) so shuffle, repeat, and volume are correct from the first
    //           frame and serve as the baseline the user can override mid-load. This is safe to
    //           apply before any tracks exist and before the terminal track load because: volume
    //           is a persistent gain on the engine that a later `engine.load` does not reset, and
    //           shuffle is a stored mode field the queue re-reads on every `setTracks`/`playIndex`,
    //           so setting the mode on an empty queue still yields the correct scope once tracks
    //           arrive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // applySettings(session) {
    //   this.queue.setPlaybackMode(session.playbackMode);
    //   this.engine.setVolume(session.volume);
    //   this.uiState = { ...this.uiState, volume: session.volume };
    // }
    // ```
    /**
     * Defines apply settings behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun applySettings(session: Session) {
        // Restore one mode before tracks arrive; later scope rebuilds read this value.
        queue.setPlaybackMode(session.playbackMode)
        // What:     `engine.setVolume(session.volume)` applies the saved gain to the engine.
        // Why:      Restore the user's volume from the first frame; this persistent gain survives
        //           the terminal `engine.load`, so applying it early is safe.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.engine.setVolume(session.volume);
        // ```
        engine.setVolume(session.volume)
        // What:     `uiState = uiState.copy(volume = session.volume)` reassigns `uiState` to a copy
        //           with the saved volume (via the data-class `copy` + the Compose delegate).
        // Why:      Make the slider match the engine immediately; `beginLoad`'s following `refresh`
        //           carries this volume into the loading snapshot.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.uiState = { ...this.uiState, volume: session.volume };
        // ```
        uiState = uiState.copy(volume = session.volume)
    }

    // What:     `private fun restoreSelectedTrack(built: BuiltLibrary, session: Session, feedSearch: Boolean)`
    //           adopts the full scanned library and reselects the saved track.
    // Why:      Rebuild the library from the authoritative full list once the scan finishes, put the saved track back
    //           in the engine without playing it, and seek to its saved position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private restoreSelectedTrack(built, session, feedSearch) {
    //   this.installParts(built); this.loadedUri = null; this.isLoading = false;
    //   const i = session.selected ? this.uris.indexOf(session.selected) : -1;
    //   if (i >= 0) { this.queue.playIndex(i); this.loadedUri = this.uris[i]; this.engine.load(this.uris[i], false); if
    //   (session.positionSecs > 0) this.engine.seekTo(session.positionSecs); }
    //   else this.queue.clearSelection();
    //   this.refresh(true);
    // }
    // ```
    /** Adopts [built] as the library and puts the session's saved track back without playing it. */
    private fun restoreSelectedTrack(built: BuiltLibrary, session: Session, feedSearch: Boolean) {
        installParts(built)
        if (feedSearch) {
            searchHost?.libraryReplaced(built.displayPaths, built.uris)
        }
        loadedUri = null
        isLoading = false
        /** Load-order index of the saved track in the full library, or -1. */
        val savedIndex: Int = session.selected?.let { uris.indexOf(it) } ?: -1
        if (savedIndex >= 0) {
            queue.playIndex(savedIndex)
            loadTrack(index = savedIndex, play = false, origin = LoadOrigin.FORWARD)
            if (session.positionSecs > 0.0) {
                engine.seekTo(session.positionSecs)
            }
        } else {
            queue.clearSelection()
        }
        refresh(followCurrent = true)
    }

    // What:     `fun finishLoad(tracks: List<Track>, session: Session): FinishLoadResult { ... }`
    //           declares a public method taking the full scanned library and the saved session,
    //           returning a `FinishLoadResult` tag, block body.
    // Why:      The terminal step of a streaming cold-start load. Because the list was interactive
    //           while it filled in, there are two outcomes: if the user tapped a track during the
    //           load, KEEP that choice and only adopt the full list; otherwise reselect the saved
    //           track and seek to the saved position. The returned tag tells the service which
    //           path ran so it can persist correctly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // finishLoad(tracks, session): FinishLoadResult {
    //   if (this.loadedUri !== null) {
    //     this.isLoading = false;
    //     this.reconcileLibrary(tracks);
    //     return "KeptUserSelectionDuringLoad";
    //   }
    //   this.restoreSelectedTrack(tracks, session);
    //   return "RestoredSavedSession";
    // }
    // ```
    /**
     * Defines finish load behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun finishLoad(tracks: List<Track>, session: Session, feedSearch: Boolean = true): FinishLoadResult =
        finishBuilt(buildLibrary(tracks), session, feedSearch)

    /**
     * Finishes a load with the library already built off the main thread: it keeps a track the user tapped while the
     * scan ran, and otherwise restores the saved track. See [finishLoad].
     */
    fun finishBuilt(built: BuiltLibrary, session: Session, feedSearch: Boolean): FinishLoadResult {
        // What:     `if (loadedUri != null) { ... }` tests whether a track is loaded in the engine,
        //           which on a cold start means "the user tapped a row while it was loading."
        // Why:      That tap must win over the saved track; we keep it and just grow the list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.loadedUri !== null) { ... }
        // ```
        if (loadedUri != null) {
            // What:     `isLoading = false` clears the loading flag BEFORE the reconcile, because
            //           `reconcileLibrary` does not touch it; without this an empty terminal would
            //           stay stuck on the spinner.
            // Why:      Release the spinner gate as part of this terminal delivery.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.isLoading = false;
            // ```
            isLoading = false
            // What:     `reconcileLibrary(tracks)` adopts the full list while preserving the tapped
            //           track by URI (it is present, since a tappable row was already discovered) and
            //           leaving the engine playing.
            // Why:      Grow to the complete library without disturbing the user's mid-load choice.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.reconcileLibrary(tracks);
            // ```
            reconcileBuilt(built, feedSearch)
            // What:     `return FinishLoadResult.KeptUserSelectionDuringLoad` returns the kept-tap
            //           tag. `FinishLoadResult.KeptUserSelectionDuringLoad` reads a named constant
            //           off the enum type (the `.` selects an enum member, not a method call).
            // Why:      Tell the service this path ran so it saves immediately (the tapped track's
            //           position is already real).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return "KeptUserSelectionDuringLoad";
            // ```
            return FinishLoadResult.KeptUserSelectionDuringLoad
        }
        // What:     `restoreSelectedTrack(tracks, session)` runs the no-tap path: reselect the saved
        //           track, seek to the saved position, and clear `isLoading`.
        // Why:      With nothing tapped, restore where the user left off.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.restoreSelectedTrack(tracks, session);
        // ```
        restoreSelectedTrack(built, session, feedSearch)
        // What:     `return FinishLoadResult.RestoredSavedSession` returns the no-tap tag (a named
        //           enum constant).
        // Why:      Tell the service this path ran so it does NOT save immediately (the engine has
        //           not applied the async seek yet, so saving now would write position 0).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return "RestoredSavedSession";
        // ```
        return FinishLoadResult.RestoredSavedSession
    }

    // What:     `fun reconcileLibrary(tracks: List<Track>, feedSearch: Boolean = true)` adopts a fresh scan of the
    //           library while keeping the playing track.
    // Why:      Re-derive the queue from what is on disk now, point it at the track that is loaded by URI, and do not
    //           flash a loading state on a routine foreground rescan. The derived lists are built here for a plain
    //           call and off the main thread for a streaming scan, which calls `reconcileBuilt`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // reconcileLibrary(tracks: readonly Track[], feedSearch = true): void {
    //   this.reconcileBuilt(buildLibrary(tracks), feedSearch);
    // }
    // ```
    /** Adopts [tracks] as the library, keeping the loaded track when it is still there. */
    fun reconcileLibrary(tracks: List<Track>, feedSearch: Boolean = true) {
        reconcileBuilt(buildLibrary(tracks), feedSearch)
    }

    // What:     `suspend fun reconcileFromScan(tracks: List<Track>)` adopts a scan that returned a whole list.
    // Why:      Compare, sort into pages and build run on a worker thread, so a foreground rescan of a big library does
    //           not freeze the screen, and a rescan that found the same tracks changes nothing but the page the view
    //           follows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async reconcileFromScan(tracks: readonly Track[]): Promise<void> { /* compare, build off thread, install */ }
    // ```
    /** Reconciles with [tracks], building off the main thread and skipping the swap when nothing changed. */
    suspend fun reconcileFromScan(tracks: List<Track>) {
        /** URIs of the library now, read before the comparison runs on a worker thread. */
        val currentUris: List<String> = uris
        /** Raw display paths of the library now, read before the comparison runs on a worker thread. */
        val currentPaths: List<String> = queue.tracks()
        /** URI of the loaded track, which the build locates in the scan. */
        val playingUri: String? = loadedUri
        /** The scan built as a library, or null when it equals the library now. */
        val built: BuiltLibrary? = withContext(Dispatchers.Default) {
            if (sameLibrary(tracks, currentUris, currentPaths)) null else buildLibrary(tracks, playingUri)
        }
        if (built == null) {
            refresh(followCurrent = true)
        } else {
            reconcileBuilt(built, feedSearch = true)
        }
    }

    // What:     `fun reconcileBuilt(built: BuiltLibrary, feedSearch: Boolean)` reconciles with a built library.
    // Why:      Installing is a few assignments plus one lookup of the loaded URI, so a library built on a worker
    //           thread does not freeze the screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // reconcileBuilt(built: BuiltLibrary, feedSearch: boolean): void {
    //   /* install, relocate the playing track, refresh */
    // }
    // ```
    /** Reconciles with a library that was built off the main thread; see [reconcileLibrary]. */
    fun reconcileBuilt(built: BuiltLibrary, feedSearch: Boolean) {
        /** URI of the loaded track before the swap. */
        val playingUri: String? = loadedUri
        installParts(built)
        if (feedSearch) {
            searchHost?.libraryReplaced(built.displayPaths, built.uris)
        }
        /** Index of the loaded track in the new library, or -1 when it is gone. */
        val newIndex: Int = when {
            playingUri == null -> -1
            playingUri == built.hintUri -> built.hintIndex
            else -> uris.indexOf(playingUri)
        }
        if (newIndex >= 0) {
            queue.playIndex(newIndex)
        } else {
            if (playingUri != null) {
                engine.unload()
            }
            loadedUri = null
            queue.clearSelection()
        }
        refresh(followCurrent = true)
    }

    // What:     `fun playIndex(index: Int) { ... }` declares a public method taking a
    //           load-order `Int` index, block body, `Unit` return.
    // Why:      Load and play the track at load-order index `index` (the track menu's Play, and a tap on a
    //           non-current row when Auto-play on select is on). An index past the library does nothing, so the
    //           track that was current is not started by mistake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playIndex(index: number): void { if (this.queue.playIndex(index) === null) return; this.playCurrent(); }
    // ```
    /**
     * Defines play index behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun playIndex(index: Int) {
        // What:     `queue.playIndex(index)` selects that track in the queue (switching
        //           scope if it is on another page).
        // Why:      Make the tapped track current.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.queue.playIndex(index);
        // ```
        if (queue.playIndex(index) == null) {
            return
        }
        // What:     `playCurrent()` loads and plays the now-current track.
        // Why:      Start playback of the tapped track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playCurrent();
        // ```
        playCurrent(LoadOrigin.TAP)
    }

    // What:     `fun selectIndex(index: Int) { ... }` declares a public method taking a load-order `Int` index,
    //           block body, `Unit` return.
    // Why:      Make the track at `index` the current one WITHOUT starting it (D124): pause whatever plays, point the
    //           queue at the track, and load it paused so the deck shows it. An index past the library does nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // selectIndex(index: number): void {
    //   if (this.queue.playIndex(index) === null) return;
    //   this.engine.pause(); this.loadedUri = this.uris[index]; this.engine.load(this.uris[index], false);
    //   this.onTrackLoaded?.(this.uris[index]); this.refresh(true);
    // }
    // ```
    /** Selects the track at [index] and loads it paused, stopping a playing track; an unknown index does nothing. */
    fun selectIndex(index: Int) {
        if (queue.playIndex(index) == null) {
            return
        }
        engine.pause()
        loadTrack(index = index, play = false, origin = LoadOrigin.TAP)
        refresh(followCurrent = true)
    }

    // What:     `fun activateRow(index: Int, autoPlayOnSelect: Boolean)` declares a public method taking a load-order
    //           `Int` index and the Auto-play on select setting.
    // Why:      The one rule for a tap, Enter or Search result on a track (D124, D127, D129): the current row toggles
    //           play and pause through the same `togglePlay` as the deck's Play button, so a restored row that was
    //           never loaded starts and a row whose load failed tries again; any other row is only selected and stays
    //           paused, or is selected and started at once when Auto-play on select is on. `rowActivationFor` decides.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // activateRow(index: number, autoPlayOnSelect: boolean): void {
    //   switch (rowActivationFor(autoPlayOnSelect, this.queue.currentIndex() === index)) {
    //     case "togglePlay": this.togglePlay(); break;
    //     case "selectAndPlay": this.playIndex(index); break;
    //     case "selectOnly": this.selectIndex(index); break;
    //   }
    // }
    // ```
    /**
     * Handles a tap, Enter or Search result on the row of [index]: toggles play on the current row, otherwise selects
     * the row and also starts it when [autoPlayOnSelect] is on.
     */
    fun activateRow(index: Int, autoPlayOnSelect: Boolean) {
        when (rowActivationFor(autoPlayOnSelect = autoPlayOnSelect, isCurrent = queue.currentIndex() == index)) {
            RowActivation.TOGGLE_PLAY -> togglePlay()
            RowActivation.SELECT_AND_PLAY -> playIndex(index)
            RowActivation.SELECT_ONLY -> selectIndex(index)
        }
    }

    // What:     `fun seekToScopeIndex(scopeIndex: Int, positionSec: Double = 0.0) { ... }`
    //           declares a public method with two params, the second having a DEFAULT VALUE
    //           (`= 0.0`): callers may omit `positionSec` and it defaults to `0.0` (a 64-bit
    //           `Double`; sibling `Float` is 32-bit). Block body, `Unit` return.
    // Why:      Move to scope position `scopeIndex` (a `MediaSession` timeline window index
    //           the framework computed for Next/Previous or a queue-item jump) and play it;
    //           an out-of-range index does nothing, matching the framework's no-op. When
    //           `positionSec` is positive the new track starts there instead of at 0.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seekToScopeIndex(scopeIndex: number, positionSec: number = 0): void { ... }
    // ```
    /**
     * Defines seek to scope index behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun seekToScopeIndex(scopeIndex: Int, positionSec: Double = 0.0) {
        // What:     `if (queue.moveCursorTo(scopeIndex) == null) { return }`. `moveCursorTo`
        //           returns `Int?` (the now-current index, or null for out-of-range);
        //           `== null` tests that; `return` bails out (returning `Unit`).
        // Why:      An out-of-range target moves nothing, so we exit early.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.queue.moveCursorTo(scopeIndex) === null) return;
        // ```
        if (queue.moveCursorTo(scopeIndex) == null) {
            // What:     `return` exits the method early (bare `return`, `Unit`).
            // Why:      Nothing to play for an out-of-range index.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            return
        }
        // What:     `playCurrent()` loads and plays the newly-current track.
        // Why:      Start the track the cursor moved to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playCurrent();
        // ```
        playCurrent(LoadOrigin.TAP)
        // What:     `if (positionSec > 0.0) { engine.seekTo(positionSec) }` seeks within the
        //           new track only when a positive start position was requested.
        // Why:      An external controller may seek to a specific item AND position; honour
        //           the position when given.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (positionSec > 0) this.engine.seekTo(positionSec);
        // ```
        if (positionSec > 0.0) {
            // What:     `engine.seekTo(positionSec)` moves the playhead to `positionSec`
            //           seconds.
            // Why:      Start the new track at the requested offset.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.engine.seekTo(positionSec);
            // ```
            engine.seekTo(positionSec)
        }
    }

    // What:     `fun togglePlay() { ... }` declares a public method, block body, `Unit`.
    // Why:      Toggle play/pause: pause if playing, resume the loaded track, else load and
    //           play the current track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // togglePlay(): void { ... }
    // ```
    /**
     * Defines toggle play behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun togglePlay() {
        // What:     `if (isPlaying) { ... } else if (loadedUri != null && loadedUri == currentUri()) { ... } else { ...
        //           }`
        //           is an if / else-if / else CHAIN. `loadedUri != null` is a null check;
        //           `&&` is logical AND; `loadedUri == currentUri()` compares the URI loaded
        //           in the engine to the current track's URI (both `String?`; `==` is
        //           null-safe value equality). Comparing URIs, not indices, keeps "is the
        //           current track already loaded?" correct after a rescan shifts indices.
        // Why:      Three cases: currently playing -> pause; the current track is already
        //           loaded -> resume; otherwise -> load and play.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.isPlaying) this.engine.pause();
        // else if (this.loadedUri !== null && this.loadedUri === this.currentUri()) this.engine.play();
        // else this.playCurrent();
        // ```
        if (isPlaying) {
            // What:     `engine.pause()` pauses playback.
            // Why:      We were playing; the toggle pauses.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.engine.pause();
            // ```
            engine.pause()
        } else if (loadedUri != null && loadedUri == currentUri()) {
            // What:     `engine.play()` resumes the already-loaded track.
            // Why:      The current track is loaded, so just resume it (no reload).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.engine.play();
            // ```
            engine.play()
        } else {
            // What:     `playCurrent()` loads and plays the current track.
            // Why:      Nothing relevant is loaded, so load and play from scratch.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.playCurrent();
            // ```
            playCurrent(LoadOrigin.TAP)
        }
        // What:     `refresh()` rebuilds the snapshot after the play/pause change.
        // Why:      Repaint the play/pause icon.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.refresh();
        // ```
        refresh()
    }

    // What:     `fun setPlayWhenReady(play: Boolean) { ... }` declares a public method
    //           taking a `Boolean`, block body, `Unit`.
    // Why:      Set the play intent EXPLICITLY (the `MediaSession`'s play/pause command and
    //           the system media buttons): resume/start the current track, or pause it.
    //           Unlike `togglePlay` the caller names the target state, so a duplicate
    //           command (pause while already paused) is a safe no-op.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setPlayWhenReady(play: boolean): void { ... }
    // ```
    /**
     * Defines set play when ready behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun setPlayWhenReady(play: Boolean) {
        // What:     `if (play) { ... } else { engine.pause() }` branches on the requested
        //           intent: play vs pause.
        // Why:      Apply exactly the named state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (play) { ... } else { this.engine.pause(); }
        // ```
        if (play) {
            // What:     `if (loadedUri != null && loadedUri == currentUri()) { engine.play() } else { playCurrent() }`
            //           is the same "already-loaded?" check as in `togglePlay`: resume if
            //           the current track's URI is the one loaded, else load and play.
            // Why:      Resume cheaply when possible; otherwise load and play. URI identity
            //           (not index) survives a rescan reshuffling the load order.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (this.loadedUri !== null && this.loadedUri === this.currentUri()) this.engine.play();
            // else this.playCurrent();
            // ```
            if (loadedUri != null && loadedUri == currentUri()) {
                // What:     `engine.play()` resumes the loaded track.
                // Why:      The current track is already loaded.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.engine.play();
                // ```
                engine.play()
            } else {
                // What:     `playCurrent()` loads and plays the current track.
                // Why:      The current track is not loaded, so load and play.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.playCurrent();
                // ```
                playCurrent(LoadOrigin.TAP)
            }
        } else {
            // What:     `engine.pause()` pauses playback.
            // Why:      The named intent is "pause".
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.engine.pause();
            // ```
            engine.pause()
        }
        // What:     `refresh()` rebuilds the snapshot after the intent change.
        // Why:      Repaint to reflect the new play/pause state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.refresh();
        // ```
        refresh()
    }

    // What:     `fun next() { ... }` declares a public method, block body, `Unit`.
    // Why:      Skip to the next track in scope and play it (user pressed Next).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // next(): void { this.queue.advance(false); this.playCurrent(); }
    // ```
    /**
     * Defines next behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun next() {
        // What:     `queue.advance(natural = false)` advances the queue with the `natural`
        //           argument by NAME as `false` (a manual Next does NOT honour repeat-track).
        // Why:      Move forward one track on an explicit Next.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.queue.advance(false);
        // ```
        queue.advance(natural = false)
        // What:     `playCurrent()` loads and plays the next track.
        // Why:      Start whatever `advance` selected.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playCurrent();
        // ```
        playCurrent(LoadOrigin.FORWARD)
    }

    // What:     `fun prev() { ... }` declares a public method, block body, `Unit`.
    // Why:      Skip to the previous track in scope and play it (user pressed Prev).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // prev(): void { this.queue.prev(); this.playCurrent(); }
    // ```
    /**
     * Defines prev behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun prev() {
        // What:     `queue.prev()` moves the cursor to the previous track in scope.
        // Why:      Step backward on an explicit Prev.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.queue.prev();
        // ```
        queue.prev()
        // What:     `playCurrent()` loads and plays the previous track.
        // Why:      Start the track `prev` selected.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playCurrent();
        // ```
        playCurrent(LoadOrigin.BACKWARD)
    }

    // What:     `fun setPlaybackMode(mode: PlaybackMode) { ... }` declares a public method taking
    //           a `PlaybackMode` enum, block body, `Unit`.
    // Why:      Change shuffle/scope, keeping the current track current.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setPlaybackMode(mode: PlaybackMode): void { this.queue.setPlaybackMode(mode); this.refresh(); }
    // ```
    /**
     * Defines set shuffle behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun setPlaybackMode(mode: PlaybackMode) {
        // Scope the choice to the page whose displayed name labels the segment.
        queue.setPageScope(pageIndices(uiState.selectedPage))
        queue.setPlaybackMode(mode)
        refresh()
    }

    // What:     `fun selectPage(page: Int) { ... }` declares a public method taking an `Int`
    //           page index, block body, `Unit`.
    // Why:      Show a different page tab without moving playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // selectPage(page: number): void { ... }
    // ```
    /**
     * Defines select page behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun selectPage(page: Int) {
        // What:     `if (page in pages.indices) { ... }` uses Kotlin's `in` operator for
        //           RANGE MEMBERSHIP: `pages.indices` is the `0 until pages.size` range, and
        //           `page in <range>` tests whether `page` is a valid index.
        // Why:      Only switch to a real page; ignore out-of-range requests.
        // Gotcha:   `page in pages.indices` is RANGE membership, NOT JS's `key in object`
        //           property check.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (page >= 0 && page < this.pages.length) {
        //   this.uiState = { ...this.uiState, selectedPage: page, pageItems: this.pages[page].entries };
        // }
        // ```
        if (page in pages.indices) {
            selectedPageIdentity = pageIdentity(pages[page])
            queue.setPageScope(pageIndices(page))
            // What:     `uiState = uiState.copy(selectedPage = page, pageItems = pages[page].entries)`
            //           reassigns `uiState` to a near-duplicate built by `copy(...)`. `.copy`
            //           is the data-class method that returns a NEW value with the named
            //           fields replaced (here `selectedPage` and `pageItems`) and all others
            //           kept. Assigning goes through the Compose delegate (recompose).
            // Why:      Change only the visible page and its rows, leaving playback alone.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.uiState = { ...this.uiState, selectedPage: page, pageItems: this.pages[page].entries };
            // ```
            uiState = uiState.copy(selectedPage = page, pageItems = pages[page].entries)
        }
    }

    // What:     `fun seek(positionSec: Double) { ... }` declares a public method taking a
    //           `Double` seconds value, block body, `Unit`.
    // Why:      Seek within the current track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seek(positionSec: number): void { this.engine.seekTo(positionSec); }
    // ```
    /**
     * Defines seek behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun seek(positionSec: Double) {
        // What:     `engine.seekTo(positionSec)` moves the playhead within the current track.
        // Why:      Scrub to the requested position.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.engine.seekTo(positionSec);
        // ```
        engine.seekTo(positionSec)
        // What:     `onStateChanged?.invoke()` tells the `MediaSession` projection that the playhead moved.
        // Why:      The scrubber, the seek buttons and restart call this method directly, so nothing else announced a
        //           seek: while paused, the session kept the position from before the seek (26 s behind the app in an
        //           emulator run) because no pull happened to correct it. This is the media session call only; the
        //           full `refresh` would also rebuild the UI state and save the session on every scrub.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.onStateChanged?.();
        // ```
        onStateChanged?.invoke()
    }

    // What:     `fun setVolume(volume: Float) { ... }` declares a public method taking a
    //           `Float` gain (32-bit; sibling `Double` is 64-bit, but the audio API works in
    //           32-bit floats), block body, `Unit`.
    // Why:      Set the output gain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setVolume(volume: number): void { ... }
    // ```
    /**
     * Defines set volume behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun setVolume(volume: Float) {
        // What:     `engine.setVolume(volume)` applies the gain to the engine.
        // Why:      Change the actual output level.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.engine.setVolume(volume);
        // ```
        engine.setVolume(volume)
        // What:     `uiState = uiState.copy(volume = volume)` reassigns `uiState` to a copy
        //           with the new `volume` (via the data-class `copy` + the Compose delegate).
        // Why:      Reflect the new volume in the UI snapshot without rebuilding everything.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.uiState = { ...this.uiState, volume };
        // ```
        uiState = uiState.copy(volume = volume)
    }

    // What:     `fun positionSec(): Double = engine.positionSec()` declares a public method
    //           returning a `Double`, as an EXPRESSION body that returns the engine's
    //           current position.
    // Why:      Live playback position for the seek bar.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // positionSec(): number { return this.engine.positionSec(); }
    // ```
    /**
     * Defines position sec behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun positionSec(): Double = engine.positionSec()

    // What:     `fun durationSec(): Double = engine.durationSec()` declares a public method
    //           returning a `Double`, expression body, returning the engine's track
    //           duration (0.0 when unknown).
    // Why:      Live track duration for the seek bar.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // durationSec(): number { return this.engine.durationSec(); }
    // ```
    /**
     * Defines duration sec behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun durationSec(): Double = engine.durationSec()

    // What:     `fun libraryTracks(): List<Track>` returns every library track in load order.
    // Why:      Search and the template preview read the library as tracks, and each track's position in the list
    //           is its load-order index, which playback uses.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // libraryTracks(): Track[] { return libraryTracksFrom(this.uris, this.pages); }
    // ```
    /** Returns every library track in load order, rebuilt from the stored URIs and the page entries. */
    fun libraryTracks(): List<Track> {
        /** Display path of every track as the pages show it. */
        val paths: List<String> = queue.displayPaths()
        return uris.mapIndexed { index, uri -> Track(uri = uri, displayPath = paths[index]) }
    }

    // What:     `fun uriAt(index: Int): String?` returns the URI of the track at one load-order index, or null
    //           when the index is outside the library.
    // Why:      The measures are keyed by URI, while the rows and the page name their tracks by index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // uriAt(index: number): string | null { return this.uris[index] ?? null; }
    // ```
    /** Returns the URI of the track at a load-order index, or null when the index is outside the library. */
    fun uriAt(index: Int): String? = uris.getOrNull(index)

    /**
     * Returns the URI of the loaded track, or null. A worker thread may read it to ask a library build to locate that
     * track; a stale answer only costs the main thread one lookup, because the install checks the URI again.
     */
    fun loadedUriSnapshot(): String? = loadedUri

    /** Returns the URI of every track in load order, which the library holds until it is replaced. */
    fun urisSnapshot(): List<String> = uris

    /** Returns the raw display path of every track in load order, which the queue holds until it is replaced. */
    fun pathsSnapshot(): List<String> = queue.tracks()

    /** Returns the tab-order page that holds a load-order index, or null when none does. */
    private fun pageOfLibraryIndex(index: Int): Int? = pageLookup.getOrNull(index)?.takeIf { page -> page >= 0 }

    /** Returns the load-order index of the track with [uri], or -1 when the library does not hold it; one scan. */
    fun indexOfUri(uri: String): Int = uris.indexOf(uri)

    /** Returns the track at a load-order index with its display path as the pages show it, or null past the end. */
    fun trackAt(index: Int): Track? {
        /** URI of the track at the index. */
        val uri: String = uris.getOrNull(index) ?: return null
        /** Display path of the track at the index. */
        val path: String = queue.displayPaths().getOrNull(index) ?: return null
        return Track(uri = uri, displayPath = path)
    }

    // What:     `fun dropMeasures(uris: Set<String>)` forgets the shown measures of some tracks.
    // Why:      A replaced or edited file keeps its URI; dropping its old measure makes the page refresh read the new
    //           one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dropMeasures(uris): void { this.uiState = { ...this.uiState, measures: omit(this.uiState.measures, uris) }; }
    // ```
    /** Removes the measures of [uris] so the next page refresh reads them again. */
    fun dropMeasures(uris: Set<String>) {
        if (uris.isEmpty()) {
            return
        }
        uiState = uiState.copy(measures = uiState.measures - uris)
    }

    // What:     `fun mergeMeasures(updates: Map<String, TrackMeasure>)` publishes newly read measures.
    // Why:      The refresh reads tracks in batches, and each batch replaces only its own keys.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mergeMeasures(updates): void { this.uiState = { ...this.uiState, measures: { ...old, ...updates } }; }
    // ```
    /** Publishes measures read for some tracks, keeping the measures of every other track unchanged. */
    fun mergeMeasures(updates: Map<String, TrackMeasure>) {
        if (updates.isEmpty()) {
            return
        }
        uiState = uiState.copy(measures = uiState.measures + updates)
    }

    // What:     `fun snapshot(): PlaybackSnapshot { ... }` declares a public method, block
    //           body, returning a `PlaybackSnapshot`.
    // Why:      Point-in-time view of the current scope and transport for the `MediaSession`
    //           projection (`BrainPlayer`). The scope's tracks are reported in playback
    //           order so the session's framework-computed Next/Previous matches this queue;
    //           position and duration are sampled here and extrapolated by the session
    //           between pulls.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // snapshot(): PlaybackSnapshot { ... }
    // ```
    /**
     * Defines snapshot behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun snapshot(): PlaybackSnapshot {
        // What:     `val order: List<Int> = queue.playbackOrder()` declares a read-only
        //           `List<Int>` local `order`: the current scope's load-order indices in
        //           playback order.
        // Why:      The timeline reports tracks in this order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const order: readonly number[] = this.queue.playbackOrder();
        // ```
        /**
         * Defines order value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val order: List<Int> = queue.playbackOrder()
        // What:     `val display: List<String> = queue.displayPaths()` declares a read-only
        //           `List<String>` local `display`: the per-track display strings in load
        //           order.
        // Why:      We title each timeline row with the track's display path.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const display: readonly string[] = this.queue.displayPaths();
        // ```
        /**
         * Defines display value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val display: List<String> = queue.displayPaths()
        // What:     `snapshotItems.itemsFor(...)` returns the timeline rows from a cache that rebuilds only what
        //           changed.
        // Why:      Building a row per queue position on every media session refresh made a track change cost time
        //           proportional to the queue size; the order, display and URI lists are replaced when they change,
        //           so identical instances mean identical rows, and an artist tag patches only its own rows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const items = this.snapshotItems.itemsFor({ order, display, uris: this.uris, artists: this.trackArtists });
        // ```
        /** Timeline rows in playback order, shared with the previous snapshot where nothing changed. */
        val items: List<SnapshotItem> = snapshotItems.itemsFor(
            order = order,
            display = display,
            uris = uris,
            artists = trackArtists,
        )
        // What:     `return PlaybackSnapshot( ... )` constructs and returns the snapshot with
        //           NAMED arguments. Two of them convert seconds to milliseconds:
        //           `(durationSec() * MILLIS_PER_SEC).toLong()` multiplies a `Double` by the
        //           `Double` constant `1000.0` (real division/multiplication), then
        //           `.toLong()` truncates to a 64-bit `Long` (milliseconds). Same for
        //           `positionMs`.
        // Why:      Hand the session a complete, immutable picture of scope + transport, with
        //           position/duration in the milliseconds the framework expects.
        // Gotcha:   `.toLong()` TRUNCATES toward zero (drops the fraction), it does not
        //           round; `Double * Double` is float math, then the cast narrows to `Long`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return {
        //   items,
        //   currentIndex: this.queue.cursorPosition(),
        //   playWhenReady: this.engine.playWhenReady(),
        //   volume: this.uiState.volume,
        //   durationMs: Math.trunc(this.durationSec() * MILLIS_PER_SEC),
        //   positionMs: Math.trunc(this.positionSec() * MILLIS_PER_SEC),
        // };
        // ```
        return PlaybackSnapshot(
            // What:     `items = items` passes the built timeline rows by name.
            // Why:      The snapshot's track list.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // items,
            // ```
            items = items,
            // What:     `currentIndex = queue.cursorPosition()` passes the current scope
            //           position (an `Int?`) by name.
            // Why:      Which timeline row is current.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // currentIndex: this.queue.cursorPosition(),
            // ```
            currentIndex = queue.cursorPosition(),
            // What:     `playWhenReady = engine.playWhenReady()` passes the engine's play
            //           intent (a `Boolean`) by name.
            // Why:      The notification's play/pause state follows this.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // playWhenReady: this.engine.playWhenReady(),
            // ```
            playWhenReady = engine.playWhenReady(),
            // What:     `volume = uiState.volume` passes the current gain by name.
            // Why:      Informational volume for external surfaces.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // volume: this.uiState.volume,
            // ```
            volume = uiState.volume,
            // What:     `durationMs = (durationSec() * MILLIS_PER_SEC).toLong()` converts the
            //           duration seconds to a `Long` milliseconds (multiply by 1000.0, then
            //           `.toLong()` truncates).
            // Why:      media3 reports durations in milliseconds.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // durationMs: Math.trunc(this.durationSec() * MILLIS_PER_SEC),
            // ```
            durationMs = (durationSec() * MILLIS_PER_SEC).toLong(),
            // What:     `positionMs = (positionSec() * MILLIS_PER_SEC).toLong()` converts the
            //           position seconds to a `Long` milliseconds (same multiply + truncate).
            // Why:      media3 reports positions in milliseconds.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // positionMs: Math.trunc(this.positionSec() * MILLIS_PER_SEC),
            // ```
            positionMs = (positionSec() * MILLIS_PER_SEC).toLong(),
        )
    }

    // What:     `fun currentScopeIndex(): Int? = queue.cursorPosition()` declares a public
    //           method returning a NULLABLE `Int?`, expression body, returning the queue's
    //           cursor position directly.
    // Why:      Current scope position (timeline window index), cheaply, without building a
    //           full `snapshot`; the `MediaSession` projection uses it to tell an in-place
    //           seek from a jump to another track. Null when the queue is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // currentScopeIndex(): number | null { return this.queue.cursorPosition(); }
    // ```
    /**
     * Defines current scope index behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun currentScopeIndex(): Int? = queue.cursorPosition()

    // What:     `fun release() { ... }` declares a public method, block body, `Unit`.
    // Why:      Release the underlying engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // release(): void { this.engine.release(); }
    // ```
    /**
     * Defines release behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun release() {
        // What:     `engine.release()` releases the engine's resources.
        // Why:      Free audio focus, buffers, and file handles.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.engine.release();
        // ```
        engine.release()
    }

    // What:     `internal fun handleNaturalEnd()` asks the queue for the track after a natural end
    //           (`advance(natural = true)`, which honours Repeat) and plays it. `advance` returns
    //           `null` when nothing follows: In order has played the last track of its folder
    //           (D2), or the queue is empty.
    // Why:      Move to the next track (or replay under Repeat) on a natural end, and stop at
    //           the end of the folder instead of wrapping.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // handleNaturalEnd(): void {
    //   if (this.queue.advance(true) === null) this.stopAtScopeEnd();
    //   else this.playCurrent();
    // }
    // ```
    /** Reacts to a track that ended on its own: plays the next one, or stops at the end of an In order folder. */
    internal fun handleNaturalEnd() {
        if (queue.advance(natural = true) == null) {
            stopAtScopeEnd()
        } else {
            playCurrent(LoadOrigin.NATURAL_END)
        }
    }

    // What:     `private fun stopAtScopeEnd()` pauses, puts the queue cursor on the first track of
    //           the scope and loads that track paused.
    // Why:      D2: In order plays to the end of the folder and stops. Loading the first track
    //           means the next Play starts the folder over, as a disc player does; an engine
    //           left at the end of the last file would end again at once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private stopAtScopeEnd(): void {
    //   this.engine.pause();
    //   const first = this.queue.rewindScope();
    //   if (first !== null) {
    //     this.loadedUri = this.uris[first];
    //     this.engine.load(this.uris[first], false);
    //     this.onTrackLoaded?.(this.uris[first]);
    //   }
    //   this.refresh(true);
    // }
    // ```
    /** Stops playback at the end of an In order folder and readies its first track. */
    private fun stopAtScopeEnd() {
        engine.pause()
        /** Load-order index of the scope's first track, or null when nothing is queued. */
        val first: Int? = queue.rewindScope()
        if (first == null) {
            refresh()
            return
        }
        // The readied first track is the loaded track, so its artist tag is read like that of any other load.
        loadTrack(index = first, play = false, origin = LoadOrigin.FORWARD)
        refresh(followCurrent = true)
    }

    // What:     `private fun loadTrack(index: Int, play: Boolean, origin: LoadOrigin, continuesWalk: Boolean = false)`
    //           records the track being loaded and hands it to the engine.
    // Why:      Every load goes through here so the origin of the load in flight is always known when its failure
    //           arrives, however late. A load the walk makes after a failure continues that walk; every other load
    //           begins a new one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private loadTrack(index, play, origin, continuesWalk = false): void {
    //   if (!continuesWalk) this.walk.reset();
    //   this.loadedUri = this.uris[index]; this.latestOrigin = origin; this.latestPlays = play;
    //   this.engine.load(this.uris[index], play); this.onTrackLoaded?.(this.uris[index]);
    // }
    // ```
    /** Loads the track at [index], remembering how the load started; a walk's own next load keeps its walk. */
    private fun loadTrack(index: Int, play: Boolean, origin: LoadOrigin, continuesWalk: Boolean = false) {
        if (!continuesWalk) {
            unplayableWalk.reset()
        }
        loadedUri = uris[index]
        latestLoadOrigin = origin
        latestLoadPlays = play
        engine.load(uris[index], play = play)
        onTrackLoaded?.invoke(uris[index])
    }

    // What:     `private fun walkPastUnplayable(uri: String, reason: LoadFailure)` answers the failed load of a track
    //           the player moved to by itself.
    // Why:      D122 and D128: a playback-driven move keeps walking past unreadable files and reports them, and D123
    //           keeps that report out of the app. The report grows with every file passed and the host replaces its
    //           notification. The walk steps the way the move went, stops when every track has had one try (so a
    //           queue of bad files cannot spin) and stops at once on a timeout, because a library that does not
    //           answer would stall the next file too and each try waits for the limit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private walkPastUnplayable(uri: string, reason: LoadFailure): void {
    //   this.loadedUri = null;
    //   const more = this.walk.record(uri, this.queue.len());
    //   this.onUnplayableFiles?.({ fileNames: names, timedOut: reason === "TIMED_OUT" });
    //   if (!more || reason === "TIMED_OUT") { this.engine.pause(); this.refresh(); return; }
    //   const next = this.stepAfterFailure(); /* load it, or ready the first track of the folder */
    // }
    // ```
    /** Passes the unreadable track [uri] by loading the next one the move would have reached, and reports it. */
    private fun walkPastUnplayable(uri: String, reason: LoadFailure) {
        loadedUri = null
        /** Whether the walk may still try another track. */
        val mayTryAnother: Boolean = unplayableWalk.record(uri = uri, limit = queue.len())
        /** Whether this failure says the music library was slow rather than the file broken. */
        val timedOut: Boolean = reason == LoadFailure.TIMED_OUT
        /** Bare names of every file this walk has passed so far, in the order met. */
        val fileNames: List<String> = unplayableWalk.failedUris().map { failed -> bareNameOf(failed) }
        onUnplayableFiles?.invoke(UnplayableReport(fileNames = fileNames, timedOut = timedOut))
        if (!mayTryAnother || timedOut) {
            engine.pause()
            refresh()
            return
        }
        /** How the move that began this walk was going, which decides the direction of the next try. */
        val origin: LoadOrigin = latestLoadOrigin
        /** Whether the move started playback, which the next try keeps. */
        val play: Boolean = latestLoadPlays
        /** Load-order index of the next track to try, or null at the end of an In order folder. */
        val next: Int? = when {
            origin == LoadOrigin.BACKWARD -> queue.prev()
            origin == LoadOrigin.NATURAL_END && queue.playbackMode() != PlaybackMode.REPEAT ->
                queue.advance(natural = true)
            else -> queue.advance(natural = false)
        }
        if (next == null) {
            engine.pause()
            /** First track of the folder, readied paused as when a folder plays to its end. */
            val first: Int? = queue.rewindScope()
            if (first == null) {
                refresh()
                return
            }
            loadTrack(index = first, play = false, origin = LoadOrigin.FORWARD, continuesWalk = true)
        } else {
            loadTrack(index = next, play = play, origin = origin, continuesWalk = true)
        }
        refresh(followCurrent = true)
    }

    // What:     `private fun bareNameOf(uri: String): String` is the file name the report shows for a track.
    // Why:      The report names files, never folders; the library knows the track's display path, and a track the
    //           library no longer lists falls back to the last segment of its URI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private bareNameOf(uri: string): string {
    //   return fileNameOf(this.trackAt(this.indexOfUri(uri))?.displayPath ?? uri);
    // }
    // ```
    /** Bare file name of the track at [uri], as the library names it. */
    private fun bareNameOf(uri: String): String =
        trackFileNameOf(trackAt(indexOfUri(uri))?.displayPath ?: uri)

    // What:     `private fun startOfScope(): Int?` selects the first track of the folder on screen in the queue
    //           and returns it, or returns null for an empty library.
    // Why:      Play with nothing selected (a fresh install, or after the playing track was trashed) used to do
    //           nothing while the button flipped to Pause; it now starts the folder on screen, as the desktop does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private startOfScope(): number | null {
    //   const first = this.pageIndices(this.uiState.selectedPage)[0] ?? 0;
    //   return this.uris.length === 0 ? null : this.queue.playIndex(first);
    // }
    // ```
    /** Selects the first track of the folder on screen, or of the library, and returns it; null when empty. */
    private fun startOfScope(): Int? {
        /** First track of the folder on screen, else of the whole library, else none. */
        val first: Int = pageIndices(uiState.selectedPage).firstOrNull() ?: uris.indices.firstOrNull() ?: return null
        return queue.playIndex(first)
    }

    // What:     `private fun playCurrent() { ... }` declares a PRIVATE method (this-class
    //           only), block body, `Unit`.
    // Why:      Load the queue's current track and play it, or refresh idle state when the
    //           queue is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private playCurrent(): void { ... }
    // ```
    /**
     * Defines play current behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    private fun playCurrent(origin: LoadOrigin) {
        // What:     `val index = queue.currentIndex()` declares a read-only local `index`
        //           (type INFERRED as `Int?`) from the queue's current load-order index.
        // Why:      Decide whether there is a track to play.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const index = this.queue.currentIndex();
        // ```
        /**
         * Defines index value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val selected: Int? = queue.currentIndex()
        /** Load-order index to load: the selected track, else the first track of the folder on screen. */
        val index = selected ?: startOfScope()
        // What:     `if (index == null) { refresh(); return }` handles the empty-queue case:
        //           when there is no current track, repaint idle state and bail.
        // Why:      Nothing to load when the queue is empty.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (index === null) { this.refresh(); return; }
        // ```
        if (index == null) {
            // What:     `refresh()` rebuilds the (idle) snapshot.
            // Why:      Show idle state when nothing is current.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.refresh();
            // ```
            refresh()
            // What:     `return` exits early (bare `return`, `Unit`).
            // Why:      No track to load.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            return
        }
        // What:     `loadedUri = uris[index]` records the URI being loaded (the stable
        //           identity), not the index. Inside this block `index` is smart-cast to a
        //           non-null `Int` (the null case returned above), so `uris[index]` is safe.
        // Why:      Remember the loaded track by URI so `togglePlay`/`setPlayWhenReady` can
        //           resume it without reloading, and so the check survives a rescan that
        //           shifts indices (the same URI keeps matching).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.loadedUri = this.uris[index];
        // ```
        // What:     `loadTrack(...)` records the URI being loaded and loads it, starting it.
        // Why:      A Play with nothing selected starts at the first track by itself, so the person named no row and
        //           a failure walks on (D128); every other call passes the origin it was started with.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.loadTrack(index, true, selected === null ? "forward" : origin);
        // ```
        loadTrack(index = index, play = true, origin = if (selected == null) LoadOrigin.FORWARD else origin)
        // What:     `refresh(followCurrent = true)` rebuilds the snapshot, switching to the
        //           current track's page (named arg `followCurrent = true`).
        // Why:      Keep the highlighted row on screen after loading a track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.refresh(true);
        // ```
        refresh(followCurrent = true)
    }

    // What:     `private fun resolveViewedPage(): Int { ... }` declares a PRIVATE helper, block
    //           body, returning an `Int` page index. It is declared BEFORE `refresh` (which calls
    //           it) so the file reads top-down.
    // Why:      When `refresh` is not following a current track, it must decide which page tab to
    //           show. Plain index-clamping breaks during a streaming load, because newly
    //           discovered earlier-sorting folders shift every page's index, so the tab the user
    //           was looking at would jump. Resolving by LABEL keeps the same tab; only when that
    //           label no longer exists do we fall back to the clamp.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private resolveViewedPage(): number {
    //   const previousLabel = this.uiState.pageLabels[this.uiState.selectedPage] ?? null;
    //   const byLabel = previousLabel === null
    //     ? -1
    //     : this.pages.findIndex((p) => p.label === previousLabel);
    //   return byLabel >= 0 ? byLabel : clamp(this.uiState.selectedPage, 0, Math.max(0, this.pages.length - 1));
    // }
    // ```
    /**
     * Defines resolve viewed page behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun resolveViewedPage(): Int {
        // What:     `val previousLabel: String? = uiState.pageLabels.getOrNull(uiState.selectedPage)`
        //           reads the label of the tab the user was on, or null. `getOrNull(i)` returns the
        //           element or null for an out-of-range index (no throw); the result type `String?`
        //           is "a String OR null".
        // Why:      The label is the stable identity of the viewed tab across a re-pagination; we
        //           need it to find where that tab moved to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const previousLabel: string | null = this.uiState.pageLabels[this.uiState.selectedPage] ?? null;
        // ```
        /**
         * Defines previous label value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val previousIdentity: String? = selectedPageIdentity
        // What:     `val byLabel: Int = previousLabel?.let { label -> pages.indexOfFirst { it.label == label } } ?: -1`
        //           finds the new index of that label, or -1.
        //           - `previousLabel?.let { label -> ... }` runs the block only when the label is
        //             non-null, binding it as `label`.
        //           - `pages.indexOfFirst { it.label == label }` returns the index of the first page
        //             whose `label` matches (`it` is the implicit single-lambda parameter, here a
        //             `Page`), or -1 if none.
        //           - `?: -1` covers the "no previous label at all" case.
        // Why:      Locate where the viewed tab landed after the pages shifted.
        // Gotcha:   `indexOfFirst` already returns -1 when nothing matches, so both "no label" and
        //           "label vanished" collapse to the same -1 sentinel handled below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const byLabel = previousLabel === null ? -1 : this.pages.findIndex((p) => p.label === previousLabel);
        // ```
        /**
         * Defines by label value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val byIdentity: Int = previousIdentity
            ?.let { identity -> pages.indexOfFirst { page -> pageIdentity(page) == identity } }
            ?: -1
        // What:     `if (byLabel >= 0) { return byLabel }` returns the resolved index early when the
        //           label was found.
        // Why:      The viewed tab still exists; keep showing it at its new position.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (byLabel >= 0) return byLabel;
        // ```
        if (byIdentity >= 0) {
            return byIdentity
        }
        // What:     `return uiState.selectedPage.coerceIn(0, maxOf(0, pages.size - 1))` is the
        //           fallback. `coerceIn(min, max)` CLAMPS the old numeric index into range;
        //           `maxOf(0, pages.size - 1)` is the last valid index but never below 0 (so an
        //           empty `pages` clamps to 0).
        // Why:      The viewed label is gone (or there was none), so fall back to the old
        //           index-keeping behaviour, never out of bounds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return clamp(this.uiState.selectedPage, 0, Math.max(0, this.pages.length - 1));
        // ```
        return uiState.selectedPage.coerceIn(0, maxOf(0, pages.size - 1))
    }

    // What:     `private fun refresh(followCurrent: Boolean = false) { ... }` declares a
    //           PRIVATE method with one DEFAULT-VALUED parameter (`followCurrent` defaults
    //           to `false`), block body, `Unit`.
    // Why:      Rebuild `uiState` from the queue and pages. When `followCurrent` is true,
    //           switch the visible page to the current track's page so the highlighted row
    //           stays on screen; otherwise keep the user's selected page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private refresh(followCurrent: boolean = false): void { ... }
    // ```
    /**
     * Defines refresh behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    private fun refresh(followCurrent: Boolean = false) {
        // What:     `val current = queue.currentIndex()` declares a read-only local `current`
        //           (inferred `Int?`) from the queue's current index.
        // Why:      Used both to pick the page and to fill the snapshot's `currentIndex`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const current = this.queue.currentIndex();
        // ```
        /**
         * Defines current value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val current = queue.currentIndex()
        // What:     `val selected = if (followCurrent && current != null) { ... } else { ... }`
        //           declares `selected` from an `if/else` EXPRESSION. The condition combines
        //           `followCurrent` with a null check `current != null` via `&&` (the null
        //           check also SMART-CASTS `current` to non-null `Int` inside the `then`
        //           branch).
        // Why:      Choose the page to show: follow the current track's page, or keep the
        //           user's selection clamped into range.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const selected = (followCurrent && current !== null)
        //   ? (pageOfIndex(this.pages, current) ?? this.uiState.selectedPage)
        //   : clamp(this.uiState.selectedPage, 0, Math.max(0, this.pages.length - 1));
        // ```
        /**
         * Defines selected value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val selected = if (followCurrent && current != null) {
            // What:     `pageOfIndex(pages, current) ?: uiState.selectedPage` is the `then`
            //           branch value. `pageOfIndex(...)` returns the page holding `current`
            //           (an `Int?`); the ELVIS `?:` falls back to the existing
            //           `uiState.selectedPage` when that is null.
            // Why:      Follow the current track's page, but keep the old selection if the
            //           track is on no page.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // pageOfIndex(this.pages, current) ?? this.uiState.selectedPage
            // ```
            pageOfLibraryIndex(current) ?: uiState.selectedPage
        } else {
            // What:     `resolveViewedPage()` is the `else` branch value: the page index to keep
            //           when we are NOT following a current track. It preserves the page the user
            //           was viewing by its LABEL (re-resolving its index against the new pages),
            //           falling back to a clamp when the label is gone.
            // Why:      During a streaming load the pages shift as earlier-sorting folders are
            //           discovered late, so a numeric index would point at a different tab; keying
            //           on the label keeps the viewed tab stable while the list fills in.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.resolveViewedPage()
            // ```
            resolveViewedPage()
        }
        selectedPageIdentity = pages.getOrNull(selected)?.let { page -> pageIdentity(page) }
        queue.setPageScope(pageIndices(selected))
        // What:     `uiState = PlayerUiState( ... )` reassigns `uiState` to a brand-new
        //           snapshot built with NAMED constructor arguments (the assignment goes
        //           through the Compose delegate, triggering recompose). Two arguments use
        //           helpers: `pages.map { it.label }` builds the tab labels (implicit-`it`
        //           lambda), and `pages.getOrNull(selected)?.entries ?: emptyList()` reads the
        //           selected page's entries safely (`getOrNull` returns null for an
        //           out-of-range index, `?.entries` reads its entries if present, and `?:`
        //           falls back to the empty list).
        // Why:      Produce the fresh immutable snapshot the UI renders.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.uiState = {
        //   pageLabels: this.pages.map((p) => p.label),
        //   selectedPage: selected,
        //   pageItems: this.pages[selected]?.entries ?? [],
        //   currentIndex: current,
        //   playing: this.isPlaying,
        //   playbackMode: this.queue.playbackMode(),
        //   volume: this.uiState.volume,
        //   queueSize: this.queue.len(),
        //   loading: this.isLoading,
        // };
        // ```
        /** Tab-order index of the page that holds the current track, or null when no track is current. */
        val playingPage: Int? = current?.let { index -> pageOfLibraryIndex(index) }
        uiState = PlayerUiState(
            // What:     `pageLabels = pages.map { it.label }` passes the page-tab labels by
            //           name; `pages.map { it.label }` maps each `Page` to its `label`
            //           (implicit `it`).
            // Why:      The tab captions.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // pageLabels: this.pages.map((p) => p.label),
            // ```
            pageLabels = pages.map { it.label },
            // What:     `selectedPage = selected` passes the chosen page index by name.
            // Why:      Which tab is active.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // selectedPage: selected,
            // ```
            selectedPage = selected,
            // What:     `pageItems = pages.getOrNull(selected)?.entries ?: emptyList()` passes
            //           the visible page's rows. `getOrNull(selected)` returns the `Page` or
            //           null (no throw on a bad index); `?.entries` reads its entries if
            //           present; `?: emptyList()` falls back to the empty list.
            // Why:      The rows for the active page, safely defaulting to none.
            // Gotcha:   `getOrNull` is the no-throw indexed read (vs `pages[selected]` which
            //           would throw out of range); chained with `?.`/`?:` for a total result.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // pageItems: this.pages[selected]?.entries ?? [],
            // ```
            pageItems = pages.getOrNull(selected)?.entries ?: emptyList(),
            // What:     `currentIndex = current` passes the current track's load-order index
            //           (`Int?`) by name.
            // Why:      Drives the current-row highlight.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // currentIndex: current,
            // ```
            currentIndex = current,
            // What:     `playingPageIndex` and `playingItems` pass the playing track's own folder by name.
            // Why:      The deck names the playing track even when its folder is not the one on screen.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // playingItems: pages[pageOfIndex(pages, current)]?.entries ?? [],
            // ```
            playingPageIndex = playingPage,
            playingItems = playingPage?.let { page -> pages[page].entries } ?: emptyList(),
            // What:     `playing = isPlaying` passes the play/pause flag by name.
            // Why:      The play vs pause icon.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // playing: this.isPlaying,
            // ```
            playing = isPlaying,
            // Mirror the one queue mode into the single-select control.
            playbackMode = queue.playbackMode(),
            // What:     `volume = uiState.volume` carries the current volume forward by name
            //           (reading the OLD snapshot's volume).
            // Why:      Preserve the gain across the rebuild.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // volume: this.uiState.volume,
            // ```
            volume = uiState.volume,
            // What:     `measures = uiState.measures` carries the read track measures forward by name.
            // Why:      A rebuild of the snapshot must not drop the peaks and durations already read.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // measures: this.uiState.measures,
            // ```
            measures = uiState.measures,
            // What:     `queueSize = queue.len()` passes the total track count by name.
            // Why:      Lets the UI tell "empty" from "has tracks".
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // queueSize: this.queue.len(),
            // ```
            queueSize = queue.len(),
            // What:     `loading = isLoading` passes the loading flag by name.
            // Why:      Show the loading notice while a scan runs.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // loading: this.isLoading,
            // ```
            loading = isLoading,
        )
        // What:     `onStateChanged?.invoke()` SAFE-CALLs `.invoke()` on the nullable
        //           function field `onStateChanged`: if a callback is attached, call it (a
        //           function value is invoked with `.invoke()` or `()`); if null, do nothing.
        // Why:      Notify a `MediaSession` projection that the state changed so it can
        //           re-pull.
        // Gotcha:   `?.invoke()` is the null-safe way to call a nullable function; bare
        //           `onStateChanged()` would not compile on a nullable function type.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.onStateChanged?.();
        // ```
        onStateChanged?.invoke()
        // What:     `onPersist?.invoke()` SAFE-CALLs the separate persist callback: if a
        //           persister is attached (the service), call it; if null, do nothing.
        // Why:      Save the session on any state change (selection, settings, play/pause), so
        //           a later kill does not lose it. Distinct from `onStateChanged` because that
        //           one is reserved for the MediaSession projection.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.onPersist?.();
        // ```
        onPersist?.invoke()
    }

    /**
     * Defines companion object for this music-player component; the TypeScript-oriented notes above explain its
     * shared role.
     */
    // What:     `companion object { ... }` declares the static-like object on
    //           `PlayerController`; its member is read as `PlayerController.MILLIS_PER_SEC`
    //           (within the class, just `MILLIS_PER_SEC`).
    // Why:      Hold the milliseconds-per-second constant used by `snapshot`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // static member of PlayerController:
    // //   private static readonly MILLIS_PER_SEC = 1000;
    // ```
    companion object {
        // What:     `private const val MILLIS_PER_SEC: Double = 1000.0` declares a private
        //           compile-time `Double` constant (64-bit float; sibling `Float` is 32-bit).
        //           The literal `1000.0` is a `Double` (a bare `1000` would be an `Int`).
        // Why:      Milliseconds per second, for the snapshot's millisecond position/
        //           duration. It is a `Double` so `seconds * MILLIS_PER_SEC` stays
        //           floating-point before the `.toLong()` truncation.
        // Gotcha:   `1000.0` is a `Double` literal on purpose; `Double * Double` keeps the
        //           fraction, which the later `.toLong()` then truncates.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private static readonly MILLIS_PER_SEC = 1000;
        // ```
        /**
         * Defines millis per sec value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        private const val MILLIS_PER_SEC: Double = 1000.0

        /** How many artist tags the controller keeps; the notification only reads the playing track's. */
        private const val MAX_TRACK_ARTISTS: Int = 32
    }
}

/** Reports whether [tracks] are exactly the library the player holds, in order, by URI and by raw display path. */
private fun sameLibrary(tracks: List<Track>, uris: List<String>, rawPaths: List<String>): Boolean {
    if (tracks.size != uris.size || tracks.size != rawPaths.size) {
        return false
    }
    return tracks.indices.all { index ->
        tracks[index].uri == uris[index] && tracks[index].displayPath == rawPaths[index]
    }
}

// What:     `data class LoadFailureNotice(...)` names one track the engine could not open.
// Why:      The host turns it into a message; the serial makes a repeat of the same failure a new value.
//
// In TS you'd write (pseudocode):
// ```ts
// type LoadFailureNotice = { uri: string; reason: LoadFailure; serial: number };
// ```
/** One failed attempt to open a track for playback. */
data class LoadFailureNotice(
    /** Location of the track that could not be opened. */
    val uri: String,
    /** Why it could not be opened. */
    val reason: LoadFailure,
    /** Count of failures so far, so two failures of one track differ. */
    val serial: Int,
)
