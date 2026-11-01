// ============================================================================
// File summary (folds in the old KDoc's domain content for a TS-only reader)
// ============================================================================
//
// What this file is, in one breath: it is an ADAPTER. The app has a "brain"
// (a plain Kotlin object called PlayerController) that owns the playback
// queue, the current scope of tracks, and the transport (play/pause/seek).
// Android's media framework (androidx.media3) wants to talk to a thing that
// implements its `Player` interface so the system notification, the
// lockscreen controls, the headset/Bluetooth buttons, and external
// controllers (Android Auto, Wear OS) can SHOW playback and DRIVE it. This
// file is the shim that makes the brain LOOK like a media3 `Player`.
//
// The base class we extend, `SimpleBasePlayer`, works in a pull/push way:
//   - PULL: it calls our `getState()` whenever it needs a fresh picture of
//     what's playing. We answer by taking an immutable snapshot of the brain
//     and translating it into media3's `State` shape.
//   - PUSH: when the user taps a notification button, the framework calls one
//     of our `handle*()` methods (handleSeek, handleSetPlayWhenReady, ...).
//     We forward the intent into the brain and return a "done" future. The
//     brain stays the SINGLE SOURCE OF TRUTH; the in-app Compose UI reads the
//     same brain directly, so the notification and the on-screen UI never
//     disagree.
//
// Two domain quirks a TS reader should keep in mind while reading below:
//   1. Repeat-track is NOT modeled as a media3 "repeat mode". The framework
//      never auto-advances a `SimpleBasePlayer` (our brain advances itself
//      when the audio engine reports track-end), so the repeat mode we report
//      exists ONLY to make the framework's built-in Next/Previous wrap around
//      the scope. We therefore always report REPEAT_MODE_ALL. Replaying a
//      single track on natural end is handled entirely inside the brain.
//   2. Audio focus (ducking/pausing when another app grabbed the speaker) is
//      NOT handled here. It lives in `RustEngine` and surfaces as a normal pause
//      through the brain, which then re-pulls this state, so a focus-induced
//      pause flips the notification icon correctly.
//

// What:     `package dev.monochromatic.musicplayer` declares the namespace
//           every type in this file lives under. In Kotlin the package name
//           is a logical grouping, NOT tied to the on-disk folder the way
//           Java strictly requires (though this repo mirrors them anyway).
// Why:      Other files in the same package (PlayerController, PlaybackSnapshot)
//           can refer to `BrainPlayer` without an import; cross-package code
//           imports `dev.monochromatic.musicplayer.BrainPlayer`.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — TS uses file-path-based modules + explicit imports.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.os.Handler` pulls in the `Handler` class. A
//           `Handler` is Android's way to POST a function onto a specific
//           thread's message queue so it runs later on THAT thread.
// Why:      We use it (see `handler` field) to defer `invalidateState()` onto
//           the main looper instead of calling it inline.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Handler } from "android/os";
// ```
import android.os.Handler

// What:     `import android.os.Looper`. A `Looper` is the object that OWNS a

import android.util.Log
//           thread's message loop (the forever-running "take next message,
//           run it" loop). Each thread that processes posted work has exactly
//           one Looper; the app's main/UI thread's Looper is the "main looper".
// Why:      The constructor receives the main `Looper` so this player runs all
//           its work on the same thread as the wrapping MediaSession.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Looper } from "android/os";
// ```
import android.os.Looper

// What:     `import android.os.SystemClock` pulls in the monotonic clock. `SystemClock.elapsedRealtime()` counts
//           milliseconds since boot and never jumps with the wall clock.
// Why:      media3 extrapolates a pulled position with this same clock, so `SessionPositionAnchor` must use it to
//           predict what the session reports.
//
// In TS you'd write (pseudocode):
// ```ts
// // performance.now() is the nearest monotonic clock.
// ```
import android.os.SystemClock

// What:     `import androidx.annotation.OptIn`. This is the ANNOTATION class
//           used to say "yes, I knowingly use an API that the library marked
//           as unstable/experimental." (See the `@OptIn(...)` on the class.)
// Why:      `SimpleBasePlayer` and friends are marked `@UnstableApi`; without
//           opting in, the compiler refuses to let us use them.
//
// In TS you'd write (pseudocode):
// ```ts
// import { OptIn } from "androidx/annotation";
// ```
import androidx.annotation.OptIn

// What:     `import ...media3.common.C`. `C` is a media3 utility class full of
//           integer CONSTANTS (sentinel values). We use `C.TIME_UNSET` (the
//           "duration/position is unknown" sentinel) and `C.INDEX_UNSET`
//           ("no specific item index") below.
// Why:      Needed so we can compare against and emit those sentinel values.
//
// In TS you'd write (pseudocode):
// ```ts
// import { C } from "media3/common";
// ```
import androidx.media3.common.C

// What:     `import ...media3.common.MediaItem`. `MediaItem` is media3's
//           description of "one playable thing": its URI, an id, and metadata.
// Why:      We build one `MediaItem` per track when reporting the timeline.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MediaItem } from "media3/common";
// ```
import androidx.media3.common.MediaItem

// What:     `import ...media3.common.MediaMetadata`. Holds the human-facing
//           details of a track (title, artist, artwork). We only set the title.
// Why:      The notification/lockscreen read this for the displayed track name.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MediaMetadata } from "media3/common";
// ```
import androidx.media3.common.MediaMetadata

// What:     `import ...media3.common.Player`. `Player` is the big media3
//           INTERFACE (the contract for "a thing that plays media"). It also
//           carries many `const` int constants (`Player.STATE_READY`,
//           `Player.COMMAND_*`, `Player.REPEAT_MODE_ALL`, ...).
// Why:      We reference its constants throughout and report a `Player`-shaped
//           state. (We don't implement `Player` directly; `SimpleBasePlayer`
//           does, and we subclass that.)
//
// In TS you'd write (pseudocode):
// ```ts
// import { Player } from "media3/common";
// ```
import androidx.media3.common.Player

// What:     `import ...media3.common.SimpleBasePlayer`. The ABSTRACT base
//           class we extend. It implements all the fiddly `Player` interface
//           bookkeeping and leaves us two jobs: produce a `State` in
//           `getState()`, and react to commands in `handle*()`.
// Why:      Extending it is how we become a `Player` without writing hundreds
//           of interface methods by hand.
//
// In TS you'd write (pseudocode):
// ```ts
// import { SimpleBasePlayer } from "media3/common";
// ```
import androidx.media3.common.SimpleBasePlayer

// What:     `import ...SimpleBasePlayer.MediaItemData`. A NESTED class inside
//           `SimpleBasePlayer`: the per-track row of the player's timeline
//           (its URI/metadata plus seekability and duration). The `.` here is
//           a member access into the outer class's namespace, not a package.
// Why:      We build a `List<MediaItemData>` to describe the current scope's
//           tracks as the player's timeline.
//
// In TS you'd write (pseudocode):
// ```ts
// import { SimpleBasePlayer } from "media3/common";
// type MediaItemData = SimpleBasePlayer.MediaItemData;
// ```
import androidx.media3.common.SimpleBasePlayer.MediaItemData

// What:     `import ...SimpleBasePlayer.State`. Another NESTED class: the whole
//           immutable snapshot the base class wants from `getState()` (the
//           playlist, play/pause flag, position, volume, repeat mode, ...).
// Why:      `getState()` must return one of these; we build it with its
//           `State.Builder`.
//
// In TS you'd write (pseudocode):
// ```ts
// type State = SimpleBasePlayer.State;
// ```
import androidx.media3.common.SimpleBasePlayer.State

// What:     `import ...media3.common.util.UnstableApi`. The ANNOTATION media3
//           stamps on APIs it reserves the right to change. Using such an API
//           is a compile error unless you opt in (see `@OptIn` below).
// Why:      Needed as the argument to `@OptIn(UnstableApi::class)`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { UnstableApi } from "media3/common/util";
// ```
import androidx.media3.common.util.UnstableApi

// What:     `import ...common.util.concurrent.Futures`. Guava's helper class of
//           static factory methods for `ListenableFuture` (an
//           already-completed future, etc.). We use
//           `Futures.immediateVoidFuture()` to return "done, nothing to wait
//           for".
// Why:      Every `handle*` method must return a future; this gives us a
//           cheap already-resolved one.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Futures } from "guava/concurrent";
// ```
import com.google.common.util.concurrent.Futures

// What:     `import ...concurrent.ListenableFuture`. Guava's FUTURE type: like
//           a JS Promise (a value that will be ready later) but you can attach
//           listeners to it. It is generic; below it appears as
//           `ListenableFuture<*>` (any element type — see the star note).
// Why:      The `handle*` methods are declared to return one of these so the
//           framework can wait for the command to finish.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ListenableFuture } from "guava/concurrent";
// ```
import com.google.common.util.concurrent.ListenableFuture

// What:     `@OptIn(UnstableApi::class)` is an ANNOTATION applied to the class
//           on the next line. `UnstableApi::class` is a CLASS REFERENCE (a
//           `KClass` literal — Kotlin's way to name a class as a value, like a
//           runtime handle to the type). The `::class` is non-TS punctuation
//           meaning "the class object for this name".
// Why:      It tells the compiler "I know `SimpleBasePlayer` and its friends
//           are marked unstable, and I accept that"; otherwise this file would
//           not compile.
//
// In TS you'd write (pseudocode):
// ```ts
// // @OptIn(UnstableApi)  // class-level acknowledgement of experimental API
// ```
@OptIn(UnstableApi::class)
// What:     `class BrainPlayer( ...constructor params... ) : SimpleBasePlayer(looper)`
//           declares a class named `BrainPlayer`. The parentheses right after
//           the name are Kotlin's PRIMARY CONSTRUCTOR (its parameters are
//           listed there, not in a separate method). The `: SimpleBasePlayer(looper)`
//           after the param list means "this class EXTENDS `SimpleBasePlayer`,
//           and immediately calls that base class's constructor passing
//           `looper`."
// Why:      Subclassing `SimpleBasePlayer` is what makes `BrainPlayer` usable
//           as a media3 `Player`; passing `looper` pins the base class to the
//           thread we were handed.
// Gotcha:   The base-constructor call `SimpleBasePlayer(looper)` happens HERE
//           in the header, before the class body's `init {}` block runs.
//
// In TS you'd write (pseudocode):
// ```ts
// class BrainPlayer extends SimpleBasePlayer {
//   constructor(
//     private readonly controller: PlayerController,
//     looper: Looper,
//   ) {
//     super(looper);
//   }
//   // ...body...
// }
// ```
/**
 * Defines brain player type for this music-player component; the TypeScript-oriented notes above explain its
 * role.
 */
class BrainPlayer(
    // What:     `private val controller: PlayerController` is a constructor
    //           parameter that is ALSO a property. `private val` in the
    //           constructor list means "store this argument as a private,
    //           read-only field on the instance." `controller` is the brain
    //           this player projects and drives.
    // Why:      We keep a handle to the brain so every `handle*` method can
    //           forward commands and `getState()` can pull its snapshot.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly controller: PlayerController
    // ```
    private val controller: PlayerController,
    // What:     `looper: Looper` is a plain constructor parameter (NO `val`/`var`),
    //           so it is NOT stored as a field; it is only visible during
    //           construction. It is the application main `Looper` (the thread
    //           all calls and listener callbacks happen on).
    // Why:      It is forwarded straight to `SimpleBasePlayer(looper)` and used
    //           to build the `Handler`; we never need it again afterward, so we
    //           don't keep it as a field.
    // Gotcha:   The wrapping `MediaSession` MUST be built on this same thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // looper: Looper   // not stored; just passed to super(looper)
    // ```
    looper: Looper,
) : SimpleBasePlayer(looper) {
    // What:     `private val handler: Handler = Handler(looper)` declares a
    //           private read-only field `handler` of type `Handler`, built by
    //           CALLING `Handler(looper)`. `Handler(looper)` is a constructor
    //           call (Kotlin has no `new` keyword), constructing a `Handler`
    //           bound to `looper`'s thread.
    // Why:      We post `invalidateState()` through this handler so a brain
    //           change re-pulls state on the right thread without re-entering
    //           an in-flight command (see the `init` block).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly handler: Handler = new Handler(looper);
    // ```
    /**
     * Defines handler value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val handler: Handler = Handler(looper)

    /** Posts one state refresh per burst of brain changes onto the main looper. */
    private val invalidateTrigger: CoalescedTrigger = CoalescedTrigger(
        schedule = { task -> handler.post { task() } },
        action = { invalidateState() },
    )

    // What:     `anchor` remembers the position the last `getState()` handed to media3; `positionCheck` compares it
    //           with the engine a moment after each pull and while playing, and pulls again when they disagree.
    // Why:      The engine applies a seek on its worker thread after `getState()` has read the old position, so the
    //           session extrapolated from a stale position (the notification scrubber showed a different time from
    //           the app) until an unrelated change pulled again. See `SessionPositionAnchor`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly anchor = new SessionPositionAnchor(() => performance.now());
    // private readonly positionCheck = () => { ... };
    // ```
    /** Position the last pulled state reported, checked against the engine by [positionCheck]. */
    private val anchor: SessionPositionAnchor = SessionPositionAnchor(now = { SystemClock.elapsedRealtime() })

    /** Pulls the state again when the engine has moved away from the position the session extrapolates. */
    private val positionCheck: Runnable = Runnable {
        /** The engine's position now, in milliseconds. */
        val actualMs: Long = (controller.positionSec() * MILLIS_PER_SEC).toLong()
        /** Whether the engine still agrees with the anchor, and when to look again. */
        val result: SessionPositionAnchor.Check = anchor.check(actualMs)
        when (result) {
            SessionPositionAnchor.Check.Resync -> {
                Log.d(LOG_TAG, "position anchor ${anchor.expectedMs()} ms differs from engine $actualMs ms; resync")
                invalidateState()
            }
            is SessionPositionAnchor.Check.CheckAgainIn -> handler.postDelayed(positionCheck, result.delayMs)
            SessionPositionAnchor.Check.Settled -> Unit
        }
    }

    // What:     `init { ... }` is Kotlin's INITIALIZER BLOCK: code that runs as
    //           part of constructing every instance, after the primary
    //           constructor's field assignments. There is no method name; the
    //           block just runs once per object.
    // Why:      We use it to wire the brain's `onStateChanged` callback so that
    //           any brain mutation triggers a state re-pull.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // (inside constructor, after super(looper))
    // ```
    init {
        // What:     `controller.onStateChanged = { handler.post { invalidateState() } }`
        //           ASSIGNS a function to the brain's `onStateChanged` callback
        //           slot. The right-hand `{ ... }` is a LAMBDA (an anonymous
        //           function with no parameters). Inside it, `handler.post { ... }`
        //           takes ANOTHER no-arg lambda (a TRAILING LAMBDA: when a
        //           lambda is the last argument, Kotlin lets you write it
        //           outside the parentheses as `post { ... }`). `post` schedules
        //           that inner lambda to run later on the handler's thread.
        //           The innermost call `invalidateState()` is the base class's
        //           method that asks the framework to re-pull `getState()`.
        // Why:      When the brain changes, we must tell media3 to refresh.
        //           Posting (rather than calling `invalidateState()` directly)
        //           defers it so it never runs in the middle of a `handle*`
        //           command that itself caused the brain change, which would
        //           re-enter the player.
        // Gotcha:   Both `{ }` here are LAMBDAS, not object literals. In Kotlin
        //           `foo { ... }` after a function call passes a function, not
        //           a JSON object.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // controller.onStateChanged = () => {
        //   handler.post(() => this.invalidateState());
        // };
        // ```
        controller.onStateChanged = { invalidateTrigger.request() }
    }

    // What:     `override fun getState(): State` declares a method named
    //           `getState` that takes no arguments and returns a `State`. The
    //           `override` keyword is MANDATORY in Kotlin when replacing a base
    //           class method (here `SimpleBasePlayer.getState()`); forgetting
    //           it is a compile error.
    // Why:      `SimpleBasePlayer` calls this whenever it needs the current
    //           playback picture; we answer by translating the brain's snapshot
    //           into media3's `State`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override getState(): State { ... }
    // ```
    /**
     * Defines get state behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun getState(): State {
        // What:     `val snapshot: PlaybackSnapshot = controller.snapshot()`
        //           declares a read-only local `snapshot` (the `val` keyword =
        //           cannot be reassigned) with an explicit type annotation
        //           `: PlaybackSnapshot`, initialised by calling the brain's
        //           `snapshot()`. `PlaybackSnapshot` is the app's own immutable
        //           record of the current scope + transport state.
        // Why:      We read the brain ONCE into an immutable value, then build
        //           the media3 `State` purely from it, so the picture cannot
        //           shift mid-method.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const snapshot: PlaybackSnapshot = controller.snapshot();
        // ```
        /**
         * Defines snapshot value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val snapshot: PlaybackSnapshot = controller.snapshot()
        // What:     `val playlist: List<MediaItemData> = playlistFor(snapshot)` returns the timeline rows, built
        //           again only for the positions that changed since the previous call.
        // Why:      Building a row per queue position on every pull made a track change cost time proportional to the
        //           queue size; only the row that stopped being current and the row that became current differ.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const playlist: readonly MediaItemData[] = this.playlistFor(snapshot);
        // ```
        /** Timeline rows, the same list instance as the previous pull when no row changed. */
        val playlist: List<MediaItemData> = playlistFor(snapshot)
        // What:     `stateWith(playlist).buildUpon()` starts the state from a base that already holds this playlist.
        // Why:      `setPlaylist` checks every id and builds a timeline over the whole queue; the base is built once
        //           per distinct playlist and `buildUpon()` keeps its timeline, so a pull that changes only the
        //           position or the play intent costs nothing per queue row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const builder = this.stateWith(playlist).buildUpon();
        // ```
        /** State builder that carries [playlist] and its timeline; every other field is set below. */
        val builder: State.Builder = stateWith(playlist).buildUpon()
            // What:     `.setPlayWhenReady(snapshot.playWhenReady, Player.PLAY_WHEN_READY_CHANGE_REASON_USER_REQUEST)`
            //           sets the play INTENT (true = should be playing) plus the
            //           REASON constant explaining why it changed (here "the user
            //           asked for it").
            // Why:      The notification's play/pause icon follows the intent, and
            //           the reason keeps it from flickering during buffering.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setPlayWhenReady(
            //   snapshot.playWhenReady,
            //   Player.PLAY_WHEN_READY_CHANGE_REASON_USER_REQUEST,
            // )
            // ```
            .setPlayWhenReady(snapshot.playWhenReady, Player.PLAY_WHEN_READY_CHANGE_REASON_USER_REQUEST)
            // What:     `.setPlaybackState(if (playlist.isEmpty()) Player.STATE_IDLE else Player.STATE_READY)`
            //           sets the coarse playback state. The argument is an `if/else`
            //           EXPRESSION (Kotlin `if` yields a value): empty playlist =>
            //           `STATE_IDLE`, otherwise `STATE_READY`. `playlist.isEmpty()`
            //           returns a `Boolean`.
            // Why:      The framework shows "idle" (nothing loaded) vs "ready"
            //           (a track is loaded and prepared) based on whether the
            //           scope has any tracks.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setPlaybackState(
            //   playlist.length === 0 ? Player.STATE_IDLE : Player.STATE_READY,
            // )
            // ```
            .setPlaybackState(if (playlist.isEmpty()) Player.STATE_IDLE else Player.STATE_READY)
            // What:     `.setContentPositionMs(snapshot.positionMs)` reports the
            //           current playback position (in milliseconds, a `Long`) so
            //           the scrubber knows where the playhead is.
            // Why:      Drives the seek-bar position and the elapsed-time readout.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setContentPositionMs(snapshot.positionMs)
            // ```
            .setContentPositionMs(snapshot.positionMs)
            // What:     `.setVolume(snapshot.volume)` reports output gain as a
            //           `Float` in 0.0..1.0 (a 32-bit float; the sibling `Double`
            //           is 64-bit — media3's volume API wants the 32-bit `Float`).
            // Why:      This is INFORMATIONAL only (volume is an in-app control);
            //           we still report it so external surfaces can display it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setVolume(snapshot.volume)
            // ```
            .setVolume(snapshot.volume)
            // What:     `.setRepeatMode(Player.REPEAT_MODE_ALL)` reports the
            //           repeat mode as "repeat the whole list". `REPEAT_MODE_ALL`
            //           is a media3 `const` int (siblings: `REPEAT_MODE_OFF`,
            //           `REPEAT_MODE_ONE`).
            // Why:      We report ALL (not OFF/ONE) ONLY so the framework's
            //           built-in Next/Previous WRAPS around the scope's ends.
            //           Per-track repeat is NOT a player mode here; the brain
            //           replays a single track itself on natural end. (See the
            //           file summary at top.)
            // Gotcha:   This does NOT mean "auto-repeat the playlist" in the usual
            //           sense; `SimpleBasePlayer` never auto-advances, so the only
            //           effect is making manual Next/Previous loop.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .setRepeatMode(Player.REPEAT_MODE_ALL)
            // ```
            .setRepeatMode(Player.REPEAT_MODE_ALL)
        // What:     `val current: Int? = snapshot.currentIndex` declares a
        //           read-only local `current` whose type is `Int?` — the `?`
        //           makes it a NULLABLE `Int` (it may hold an `Int` OR `null`).
        //           In Kotlin, plain `Int` can NEVER be null; only the `?`-typed
        //           form can. It is the brain's current track position, or null
        //           when the queue is empty.
        // Why:      We may not have a current track (empty queue), so we model
        //           "no current index" explicitly as null rather than a magic
        //           number.
        // Gotcha:   `Int?` (nullable) vs `Int` (never null) is a COMPILE-TIME
        //           distinction Kotlin enforces; you can't use `current` as a
        //           plain `Int` until you've checked it for null (see next line).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const current: number | null = snapshot.currentIndex;
        // ```
        /**
         * Defines current value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val current: Int? = snapshot.currentIndex
        // What:     `if (current != null && current in playlist.indices) { ... }`
        //           is a plain `if` STATEMENT (used for side effects here, not as
        //           a value). The condition has two parts joined by `&&`:
        //           `current != null` is a null check that also SMART-CASTS
        //           `current` from `Int?` to plain `Int` inside the block; and
        //           `current in playlist.indices` uses Kotlin's `in` operator to
        //           test range membership (`playlist.indices` is the `0 until
        //           playlist.size` range of valid positions).
        // Why:      Only set a "current item" when we actually have one AND it is
        //           a valid position in the reported timeline; otherwise we leave
        //           it unset.
        // Gotcha:   `x in range` here is RANGE-MEMBERSHIP, not JS's
        //           `key in object` property check; different meaning entirely.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (current !== null && current >= 0 && current < playlist.length) {
        //   builder.setCurrentMediaItemIndex(current);
        // }
        // ```
        if (current != null && current in playlist.indices) {
            // What:     `builder.setCurrentMediaItemIndex(current)` tells the
            //           builder which timeline row is the currently-playing one.
            //           `current` is, inside this block, a plain non-null `Int`
            //           thanks to the smart-cast from the null check above.
            // Why:      So the notification highlights and reports the right
            //           track as "now playing".
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // builder.setCurrentMediaItemIndex(current);
            // ```
            builder.setCurrentMediaItemIndex(current)
        }
        // What:     `anchor.record(...)` notes the position this pull reports and whether media3 will run it forward on
        //           its own (it does while a loaded track plays); the two lines after it replace any waiting check
        //           with a first one `FIRST_CHECK_DELAY_MS` from now.
        // Why:      The engine may not have applied a seek or a track change yet, so this pull can carry a stale
        //           position; the check pulls again if the engine disagrees (see `SessionPositionAnchor`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.anchor.record(snapshot.positionMs, snapshot.playWhenReady && playlist.length > 0);
        // clearTimeout(this.positionCheckTimer);
        // this.positionCheckTimer = setTimeout(this.positionCheck, FIRST_CHECK_DELAY_MS);
        // ```
        anchor.record(positionMs = snapshot.positionMs, advancing = snapshot.playWhenReady && playlist.isNotEmpty())
        handler.removeCallbacks(positionCheck)
        handler.postDelayed(positionCheck, SessionPositionAnchor.FIRST_CHECK_DELAY_MS)
        // What:     `return builder.build()` finalises the accumulated builder
        //           into the immutable `State` and RETURNS it from `getState()`.
        //           `.build()` is the builder-to-value conversion.
        // Why:      Hand the framework the finished snapshot of playback state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return builder.build();
        // ```
        return builder.build()
    }

    /** Rows handed to the base player by the previous pull. */
    private var rows: List<MediaItemData> = emptyList()

    /** Snapshot rows the [rows] were built from, compared by identity. */
    private var rowItems: List<SnapshotItem> = emptyList()

    /** Position of the current track when [rows] were built, or null. */
    private var rowsCurrent: Int? = null

    /** Duration of the current track in milliseconds when [rows] were built. */
    private var rowsDurationMs: Long = 0L

    /** State that holds [rows] as its playlist and nothing else, or null before the first pull. */
    private var baseState: State? = null

    /** Playlist that [baseState] was built from. */
    private var keptPlaylist: List<MediaItemData>? = null

    // What:     `private fun playlistFor(snapshot: PlaybackSnapshot): List<MediaItemData>` returns the timeline rows of
    //           [snapshot], reusing the row of the previous pull at every position that did not change.
    // Why:      Only the row that stopped being current and the row that became current carry different data (the
    //           duration is reported for the current track only), and a changed artist tag arrives as a new
    //           `SnapshotItem` at its own positions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private playlistFor(snapshot: PlaybackSnapshot): readonly MediaItemData[] { /* reuseRows(...) */ }
    // ```
    /** Returns the timeline rows of [snapshot] and remembers them for the next pull. */
    private fun playlistFor(snapshot: PlaybackSnapshot): List<MediaItemData> {
        /** Positions whose row can differ although their item is unchanged: the previous and the new current track. */
        val rebuilt: Set<Int> = currentRowPositions(
            previousCurrent = rowsCurrent,
            previousDurationMs = rowsDurationMs,
            current = snapshot.currentIndex,
            durationMs = snapshot.durationMs,
        )
        /** Rows for the new snapshot, sharing every row that did not change. */
        val next: List<MediaItemData> = reuseRows(
            previousItems = rowItems,
            previousRows = rows,
            items = snapshot.items,
            rebuilt = rebuilt,
            build = { position, item -> rowOf(position = position, item = item, snapshot = snapshot) },
        )
        Log.d(LOG_TAG, "getState rows=${next.size} sameList=${next === rows} rebuiltCurrent=${rebuilt.size}")
        rows = next
        rowItems = snapshot.items
        rowsCurrent = snapshot.currentIndex
        rowsDurationMs = snapshot.durationMs
        return next
    }

    /** Returns a state that holds [playlist], building it only when the playlist is a different list than last time. */
    private fun stateWith(playlist: List<MediaItemData>): State {
        /** State kept from the previous pull. */
        val kept: State? = baseState
        if (kept != null && keptPlaylist === playlist) {
            return kept
        }
        /** Fresh state over [playlist]; the base player validates the ids and builds the timeline here. */
        val built: State = State.Builder().setAvailableCommands(AVAILABLE_COMMANDS).setPlaylist(playlist).build()
        baseState = built
        keptPlaylist = playlist
        return built
    }

    /** Builds the timeline row of the track at [position], reporting a duration only for the current track. */
    private fun rowOf(position: Int, item: SnapshotItem, snapshot: PlaybackSnapshot): MediaItemData {
        /** Duration in microseconds for the current track once known, otherwise the unset marker. */
        val durationUs: Long = if (position == snapshot.currentIndex && snapshot.durationMs > 0L) {
            snapshot.durationMs * MICROS_PER_MILLI
        } else {
            C.TIME_UNSET
        }
        return MediaItemData.Builder(item.entryId)
            .setMediaItem(
                MediaItem.Builder()
                    .setUri(item.uri)
                    .setMediaId(item.loadIndex.toString())
                    .setMediaMetadata(MediaMetadata.Builder().setTitle(item.title).setArtist(item.artist).build())
                    .build(),
            )
            .setIsSeekable(true)
            .setDurationUs(durationUs)
            .build()
    }

    // What:     `override fun handleSetPlayWhenReady(playWhenReady: Boolean): ListenableFuture<*>`
    //           declares an override of the base method called when the user
    //           toggles play/pause. It takes a `Boolean` (true = play) and
    //           returns `ListenableFuture<*>`. The `<*>` is a STAR-PROJECTION:
    //           "a `ListenableFuture` of some unknown element type we don't care
    //           about" (we only care that it completes). It is Kotlin's analogue
    //           of Java's `<?>` wildcard.
    // Why:      The framework routes the play/pause command here; we forward it
    //           to the brain and report "done".
    // Gotcha:   `<*>` is NOT "any"; it is "some specific but unspecified type",
    //           a read-only/producer wildcard.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override handleSetPlayWhenReady(playWhenReady: boolean): Promise<unknown> {
    //   this.controller.setPlayWhenReady(playWhenReady);
    //   return Promise.resolve();
    // }
    // ```
    /**
     * Defines handle set play when ready behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    override fun handleSetPlayWhenReady(playWhenReady: Boolean): ListenableFuture<*> {
        // What:     `controller.setPlayWhenReady(playWhenReady)` forwards the
        //           play/pause intent into the brain (the single source of
        //           truth), which will start/stop the engine and re-pull state.
        // Why:      Apply the user's play/pause request through the brain rather
        //           than touching the engine directly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.controller.setPlayWhenReady(playWhenReady);
        // ```
        controller.setPlayWhenReady(playWhenReady)
        // What:     `return Futures.immediateVoidFuture()` returns an
        //           already-completed `ListenableFuture` carrying no value
        //           (Guava's "void future"). It is the method's return value.
        // Why:      The command is synchronous (we already applied it), so we
        //           hand back a future that is instantly resolved.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Promise.resolve();
        // ```
        return Futures.immediateVoidFuture()
    }

    // What:     `override fun handlePrepare(): ListenableFuture<*>` overrides the
    //           base method the framework calls to ask the player to "prepare"
    //           (buffer/ready the current item). Returns `ListenableFuture<*>`
    //           (a future of unspecified element type; see the star note above).
    // Why:      We must override it to satisfy the contract, but the brain
    //           already prepares the engine when it loads a track, so there's
    //           nothing extra to do.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override handlePrepare(): Promise<unknown> {
    //   return Promise.resolve();
    // }
    // ```
    /**
     * Defines handle prepare behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    override fun handlePrepare(): ListenableFuture<*> {
        // What:     `return Futures.immediateVoidFuture()` returns an instantly
        //           resolved, value-less future from this no-op handler.
        // Why:      Signal "preparation finished" immediately, because the brain
        //           already handled it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Promise.resolve();
        // ```
        return Futures.immediateVoidFuture()
    }

    // What:     `override fun handleSeek(mediaItemIndex: Int, positionMs: Long, seekCommand: Int): ListenableFuture<*>`
    //           overrides the framework's seek hook. Params: `mediaItemIndex`
    //           (`Int`, which timeline row to seek to, or `C.INDEX_UNSET` for
    //           "the current one"), `positionMs` (`Long`, target position in
    //           ms, or `C.TIME_UNSET` for "no specific position"), and
    //           `seekCommand` (`Int`, which kind of seek triggered this; unused
    //           here). Returns `ListenableFuture<*>`.
    // Why:      The framework routes Next/Previous, the scrubber, and item-jumps
    //           here; we translate them into brain calls.
    // Gotcha:   `positionMs` is `Long` (64-bit) while `mediaItemIndex` and
    //           `seekCommand` are `Int` (32-bit); the widths differ on purpose
    //           (a position in ms can be large; an index/command code is small).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override handleSeek(
    //   mediaItemIndex: number,
    //   positionMs: number,
    //   seekCommand: number,
    // ): Promise<unknown> { ... }
    // ```
    /**
     * Defines handle seek behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun handleSeek(mediaItemIndex: Int, positionMs: Long, seekCommand: Int): ListenableFuture<*> {
        // What:     `val hasPosition: Boolean = positionMs != C.TIME_UNSET`
        //           declares a read-only `Boolean` `hasPosition` that is true
        //           when the caller passed a real position (i.e. it is NOT the
        //           "unset" sentinel). `!=` is plain not-equal.
        // Why:      Some seeks specify a target time; others (a bare item jump)
        //           do not. We branch on that below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const hasPosition: boolean = positionMs !== C.TIME_UNSET;
        // ```
        /**
         * Defines has position value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val hasPosition: Boolean = positionMs != C.TIME_UNSET
        // What:     `val targetSec: Double = if (hasPosition) positionMs / MILLIS_PER_SEC else 0.0`
        //           declares a read-only `Double` (64-bit float; sibling `Float`
        //           is 32-bit). The initialiser is an `if/else` EXPRESSION:
        //           when we have a position, divide ms by 1000.0 to get seconds;
        //           otherwise default to `0.0`. `positionMs` is `Long` and
        //           `MILLIS_PER_SEC` is `Double`, so the division promotes to
        //           `Double` (real division, not integer truncation). `0.0` is a
        //           `Double` literal.
        // Why:      The brain's seek API works in fractional SECONDS, so we
        //           convert from media3's milliseconds. We pick `Double` (not
        //           `Float`) for the extra precision the brain's API expects.
        // Gotcha:   Because `MILLIS_PER_SEC` is a `Double`, `Long / Double` is
        //           floating-point division. If it were `Long / Long` (1000L),
        //           Kotlin would do INTEGER division and truncate the fraction.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const targetSec: number = hasPosition
        //   ? positionMs / MILLIS_PER_SEC
        //   : 0;
        // ```
        /**
         * Defines target sec value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val targetSec: Double = if (hasPosition) positionMs / MILLIS_PER_SEC else 0.0
        // What:     `val staysOnCurrent: Boolean = mediaItemIndex == C.INDEX_UNSET || mediaItemIndex ==
        //           controller.currentScopeIndex()`
        //           declares a read-only `Boolean`. The `||` is logical OR:
        //           true if the requested index is the "unset" sentinel (meaning
        //           "seek within whatever is current"), OR if it equals the
        //           brain's current scope position (`controller.currentScopeIndex()`
        //           returns the current position, possibly null; the `==`
        //           comparison handles null safely in Kotlin).
        // Why:      Distinguish an in-place seek (stay on this track, move the
        //           playhead) from a jump to a different track.
        // Gotcha:   `currentScopeIndex()` can return `null`; Kotlin's `==`
        //           compares it to the `Int` safely (yields false if null),
        //           unlike a raw pointer compare.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const staysOnCurrent: boolean =
        //   mediaItemIndex === C.INDEX_UNSET ||
        //   mediaItemIndex === controller.currentScopeIndex();
        // ```
        /**
         * Defines stays on current value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val staysOnCurrent: Boolean = mediaItemIndex == C.INDEX_UNSET ||
            mediaItemIndex == controller.currentScopeIndex()
        // What:     `if (staysOnCurrent) { ... } else { ... }` is a plain `if/else`
        //           STATEMENT (used for control flow, not as a value) that
        //           branches between the in-place-seek path and the jump path.
        // Why:      Each path calls a different brain method, so we route here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (staysOnCurrent) { ... } else { ... }
        // ```
        if (staysOnCurrent) {
            // What:     `if (hasPosition) { controller.seek(targetSec) }` is a
            //           plain `if` STATEMENT with no `else`: only when a target
            //           position was given do we move the playhead.
            // Why:      A current-item seek without a position is a no-op (nothing
            //           to move to); with one, we scrub to it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (hasPosition) { controller.seek(targetSec); }
            // ```
            if (hasPosition) {
                // What:     `controller.seek(targetSec)` calls the brain's seek,
                //           moving the current track's playhead to `targetSec`
                //           seconds WITHOUT reloading the track.
                // Why:      Keep playback going from the requested position
                //           (scrubber drag, or a seek-to-current with a position).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // controller.seek(targetSec);
                // ```
                controller.seek(targetSec)
            }
        } else {
            // What:     `controller.seekToScopeIndex(mediaItemIndex, targetSec)`
            //           tells the brain to switch to a DIFFERENT track at scope
            //           position `mediaItemIndex` and start it at `targetSec`
            //           seconds (0.0 meaning from the beginning).
            // Why:      Handle Next/Previous and explicit item jumps, honoring any
            //           requested start position instead of always restarting at 0.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // controller.seekToScopeIndex(mediaItemIndex, targetSec);
            // ```
            controller.seekToScopeIndex(mediaItemIndex, targetSec)
        }
        // What:     `return Futures.immediateVoidFuture()` returns an instantly
        //           resolved, value-less future after the seek has been applied.
        // Why:      The seek is synchronous from our side, so signal "done".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Promise.resolve();
        // ```
        return Futures.immediateVoidFuture()
    }

    // What:     `override fun handleRelease(): ListenableFuture<*>` overrides the
    //           framework's teardown hook, called when the player is being
    //           destroyed. Returns `ListenableFuture<*>` (future of unspecified
    //           element type).
    // Why:      We must release the inner engine and unwire the brain callback
    //           so nothing leaks when the service is destroyed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override handleRelease(): Promise<unknown> { ... }
    // ```
    /**
     * Defines handle release behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    override fun handleRelease(): ListenableFuture<*> {
        // What:     `controller.onStateChanged = null` clears the brain's
        //           state-changed callback by ASSIGNING null. This is legal
        //           because the property's type is nullable (`(() -> Unit)?` in
        //           the brain). It detaches this player from the brain.
        // Why:      Once we're releasing, the brain must stop posting state
        //           re-pulls to a player that is going away.
        // Gotcha:   Assigning `null` is only allowed because the slot is declared
        //           nullable; a non-nullable Kotlin property would reject it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.controller.onStateChanged = null;
        // ```
        controller.onStateChanged = null
        // What:     `handler.removeCallbacks(positionCheck)` drops a queued position check.
        // Why:      A check that ran after the release would read a released engine and pull state from a player
        //           that is going away.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // clearTimeout(this.positionCheckTimer);
        // ```
        handler.removeCallbacks(positionCheck)
        // What:     `controller.release()` tells the brain to release its
        //           resources (the native engine), freeing audio focus, buffers,
        //           worker threads, and file handles.
        // Why:      Prevent leaking the inner engine every time the service is
        //           destroyed (this is why COMMAND_RELEASE must be advertised;
        //           see the companion object).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.controller.release();
        // ```
        controller.release()
        // What:     `return Futures.immediateVoidFuture()` returns an
        //           already-resolved, value-less future to report that release
        //           finished.
        // Why:      Teardown is synchronous here, so signal completion at once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Promise.resolve();
        // ```
        return Futures.immediateVoidFuture()
    }

    /**
     * Defines companion object for this music-player component; the TypeScript-oriented notes above explain its
     * shared role.
     */
    // What:     `companion object { ... }` declares a COMPANION OBJECT: a single
    //           shared instance attached to the `BrainPlayer` CLASS itself (not
    //           to each `BrainPlayer` instance). Members inside it are accessed
    //           as `BrainPlayer.MEMBER`, i.e. they behave like static class
    //           members. Kotlin has no `static` keyword; the companion object is
    //           how you get class-level (one-per-class) constants and helpers.
    // Why:      We keep the shared constants (`MICROS_PER_MILLI`,
    //           `MILLIS_PER_SEC`) and the precomputed `AVAILABLE_COMMANDS` set
    //           here so they exist once, not once per player instance.
    // Gotcha:   A companion object is itself a singleton OBJECT; it is not the
    //           same as marking individual members `static`, though for a
    //           TS reader "these are the static members" is the right mental
    //           model.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // (the static members of class BrainPlayer)
    // ```
    companion object {
        /** Tag of this class in the device log. */
        private const val LOG_TAG: String = "BrainPlayer"

        // What:     `private const val MICROS_PER_MILLI: Long = 1000L` declares a
        //           private COMPILE-TIME constant (`const val` = inlined at
        //           compile time, must be a primitive/String literal) of type
        //           `Long` (64-bit int; sibling `Int` is 32-bit). `1000L`'s `L`
        //           suffix makes the literal a `Long`.
        // Why:      Used to convert the brain's milliseconds to the microseconds
        //           media3 wants for each timeline row's duration. `Long` (not
        //           `Int`) because the multiply it feeds can exceed 32 bits.
        // Gotcha:   `const val` is stronger than `val`: it is a true compile-time
        //           constant inlined into call sites, not just a read-only field.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private static readonly MICROS_PER_MILLI = 1000;
        // ```
        /**
         * Defines micros per milli value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        private const val MICROS_PER_MILLI: Long = 1000L

        // What:     `private const val MILLIS_PER_SEC: Double = 1000.0` declares a
        //           private compile-time constant of type `Double` (64-bit float;
        //           sibling `Float` is 32-bit). `1000.0` is a `Double` literal
        //           (the decimal point makes it floating-point, not int).
        // Why:      Used to convert a seek position (ms) back into the brain's
        //           fractional SECONDS. It is a `Double` ON PURPOSE so the
        //           division `positionMs / MILLIS_PER_SEC` is floating-point and
        //           keeps the fraction (a `Long`/`Long` divide would truncate).
        // Gotcha:   The float type here is what prevents integer truncation in
        //           the `handleSeek` division; this is not just cosmetic.
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

        // What:     `private val AVAILABLE_COMMANDS: Player.Commands = Player.Commands.Builder().addAll(...).build()`
        //           declares a private read-only field (`val`, NOT `const val`,
        //           because its value is a constructed OBJECT, not a primitive
        //           literal, so it can't be a compile-time constant) of type
        //           `Player.Commands`. It is built by a builder chain:
        //           `Player.Commands.Builder()` makes the builder,
        //           `.addAll(...)` adds the supported command constants, and
        //           `.build()` finalises the immutable command set.
        // Why:      We precompute, ONCE for the whole class, the exact set of
        //           transport commands this player advertises so `getState()`
        //           can hand it to the framework cheaply on every pull.
        // Gotcha:   It is `val`, not `const val`: companion-object `const` is
        //           limited to primitive/String literals, and this is a built
        //           object, so `val` (runtime-initialised once) is required.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // private static readonly AVAILABLE_COMMANDS: Player.Commands =
        //   new Player.Commands.Builder().addAll(/* ...constants... */).build();
        // ```
        /**
         * Defines available commands value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        private val AVAILABLE_COMMANDS: Player.Commands = Player.Commands.Builder()
            // What:     `.addAll( ... )` adds many command CONSTANTS to the
            //           builder in one call. Each `Player.COMMAND_*` below is a
            //           media3 `const` int naming a capability. The whole list
            //           is the player's advertised command surface.
            // Why:      These determine which buttons/actions the notification,
            //           lockscreen, and external controllers expose.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .addAll(
            //   Player.COMMAND_PLAY_PAUSE,
            //   Player.COMMAND_PREPARE,
            //   /* ...the rest... */
            // )
            // ```
            .addAll(
                // What:     `Player.COMMAND_PLAY_PAUSE` is the constant enabling
                //           the play/pause toggle.
                // Why:      So the notification shows a working play/pause button.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_PLAY_PAUSE,
                // ```
                Player.COMMAND_PLAY_PAUSE,
                // What:     `Player.COMMAND_PREPARE` is the constant enabling the
                //           "prepare" command (routed to `handlePrepare`).
                // Why:      Required for the framework's normal lifecycle even
                //           though our prepare is a no-op.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_PREPARE,
                // ```
                Player.COMMAND_PREPARE,
                // What:     `Player.COMMAND_RELEASE` is the constant enabling the
                //           release command (routed to `handleRelease`).
                // Why:      MUST be advertised, otherwise `SimpleBasePlayer.release()`
                //           early-returns before `handleRelease()` runs, leaking
                //           the native engine on every service destroy.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_RELEASE,
                // ```
                Player.COMMAND_RELEASE,
                // What:     `Player.COMMAND_SEEK_TO_NEXT` enables the "skip to
                //           next" action (framework computes the next index).
                // Why:      So the notification's Next button works.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_SEEK_TO_NEXT,
                // ```
                Player.COMMAND_SEEK_TO_NEXT,
                // What:     `Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM` enables
                //           "skip to the next media item" specifically.
                // Why:      Pairs with the above so external controllers' Next
                //           item-stepping works.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM,
                // ```
                Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM,
                // What:     `Player.COMMAND_SEEK_TO_PREVIOUS` enables the "skip to
                //           previous" action.
                // Why:      So the notification's Previous button works.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_SEEK_TO_PREVIOUS,
                // ```
                Player.COMMAND_SEEK_TO_PREVIOUS,
                // What:     `Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM` enables
                //           "skip to the previous media item" specifically.
                // Why:      Pairs with the above for Previous item-stepping.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM,
                // ```
                Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM,
                // What:     `Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM` enables
                //           seeking WITHIN the current track (the scrubber).
                // Why:      So dragging the seek-bar works.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM,
                // ```
                Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM,
                // What:     `Player.COMMAND_SEEK_TO_MEDIA_ITEM` enables seeking to
                //           a specific timeline item (jump to a chosen track).
                // Why:      So selecting a particular item in an external UI works.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_SEEK_TO_MEDIA_ITEM,
                // ```
                Player.COMMAND_SEEK_TO_MEDIA_ITEM,
                // What:     `Player.COMMAND_GET_CURRENT_MEDIA_ITEM` enables the
                //           framework/clients to READ the current media item.
                // Why:      So the notification can read which track is playing.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_GET_CURRENT_MEDIA_ITEM,
                // ```
                Player.COMMAND_GET_CURRENT_MEDIA_ITEM,
                // What:     `Player.COMMAND_GET_TIMELINE` enables reading the full
                //           timeline (the list of items).
                // Why:      So clients can see the scope/queue.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_GET_TIMELINE,
                // ```
                Player.COMMAND_GET_TIMELINE,
                // What:     `Player.COMMAND_GET_METADATA` enables reading track
                //           metadata (title, etc.).
                // Why:      So the notification can show the track title we set.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // Player.COMMAND_GET_METADATA,
                // ```
                Player.COMMAND_GET_METADATA,
            )
            // What:     `.build()` finalises the command-set builder into the
            //           immutable `Player.Commands` value stored in
            //           `AVAILABLE_COMMANDS`. This builder-to-value `.build()`
            //           is the chain's tail and the field's initialiser result.
            // Why:      Produce the finished, immutable advertised-commands set.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .build()
            // ```
            .build()
    }
}
