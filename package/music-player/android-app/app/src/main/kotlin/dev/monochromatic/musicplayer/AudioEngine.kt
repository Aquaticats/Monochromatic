// File summary (folded in from the interface's old KDoc):
//   This file defines the low-level audio "primitive" the app's engine implements: play one track at
//   a time, report the play/pause state and the natural end of a track, and expose the current
//   position and total duration so a seek bar can be drawn. Everything ABOVE this primitive, the play
//   queue, pagination, shuffle/scope selection, and transport orchestration (next/previous, gapless
//   handoff), lives in a separate class called PlayerController; this interface is ONLY the "play
//   this one file" seam. Keeping the surface this small lets PlayerController stay ignorant of native
//   handles, file descriptors, Rust modules, and other engine internals while still driving the single
//   production Rust engine and test fakes through one contract.
//
// What:     `package dev.monochromatic.musicplayer` declares which "package" (namespace, like a
//           folder-shaped grouping of related code) every declaration in this file belongs to. Other
//           Kotlin files that share this package can refer to these names without an import; files in
//           a different package must import them. Unlike Java, Kotlin does NOT force the on-disk
//           directory to match the package name, but this repo keeps them aligned anyway
//           (.../dev/monochromatic/musicplayer/AudioEngine.kt).
// Why:      We need this so the names defined here (the `AudioEngine` interface) live under a stable,
//           fully-qualified path `dev.monochromatic.musicplayer.AudioEngine`, and so siblings like
//           PlayerController can use `AudioEngine` directly without importing it.
//
// In TS you'd write (pseudocode):
// ```ts
// // No statement at all — in TS a file is implicitly its own module; the
// // directory path (e.g. src/dev/monochromatic/musicplayer/) is the namespace.
// ```
package dev.monochromatic.musicplayer

// What:     `interface AudioEngine { ... }` declares an "interface": a pure contract that lists method
//           signatures (names, parameters, return types) with NO bodies and NO stored fields. A class
//           that says `class RustEngine : AudioEngine` promises to provide a real implementation of
//           every method listed here. The `{ ... }` braces hold the member declarations.
//           Siblings the reader might have expected instead of `interface`:
//             - `abstract class` — could ALSO declare unimplemented methods, but a class can only
//               extend ONE class, whereas a type can implement MANY interfaces; an interface also
//               cannot hold constructor state.
//             - `class` / `data class` / `sealed interface` — those would carry implementation or a
//               closed set of subtypes; we want neither here, just the bare contract.
// Why:      We need this so the production Rust engine and the test fake conform to ONE shared shape.
//           PlayerController can then hold an `AudioEngine` reference and call `load`/`play`/`pause`
//           without knowing or caring whether it is driving native playback or a test double. This is
//           the "single seam" the file summary mentions.
// Gotcha:   This is a NOMINAL interface, not structural. A Kotlin class that happens to have all these
//           methods but does not write `: AudioEngine` is NOT an `AudioEngine`. TS would accept it on
//           shape alone; Kotlin will not.
//
// In TS you'd write (pseudocode):
// ```ts
// interface AudioEngine {
//   // ...method signatures go here, same as below...
// }
// ```
/**
 * Defines audio engine interface for this music-player component; the TypeScript-oriented notes above explain
 * its contract.
 */
interface AudioEngine {
    // What:     `fun load(uri: String, play: Boolean)` declares a method named `load`. `fun` is
    //           Kotlin's keyword for "function/method". It takes two parameters: `uri` of type
    //           `String` and `play` of type `Boolean`. The parameter syntax is `name: Type` (the type
    //           comes AFTER a colon, the reverse of nothing-special but worth flagging). There is no
    //           `: ReturnType` after the parentheses, which in Kotlin means the return type is `Unit`
    //           (Kotlin's "returns nothing meaningful", the equivalent of `void`). No body here because
    //           this is an interface; each engine fills it in.
    //           Type siblings worth naming:
    //             - `String` is Kotlin's immutable text type (UTF-16 under the hood). There is no
    //               separate "borrowed string" type like Rust's `&str`; a Kotlin `String` is always a
    //               full, garbage-collected object, so the choice is just `String` (not `CharSequence`,
    //               its read-only super-type, because callers pass concrete file paths/URIs).
    //             - `Boolean` is the true/false type. Siblings you might wonder about, `Int` used as a
    //               0/1 flag, are deliberately avoided; a real `Boolean` documents the intent ("play or
    //               not") and cannot hold any other value.
    // Why:      We need this so a caller can hand the engine a track location (`uri`) and say whether to
    //           start playing immediately (`play = true`) or just load it paused (`play = false`). It
    //           is the entry point that puts a track into the engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // uri: absolute filesystem path or content:// URI the engine can resolve.
    // // play: true = start immediately, false = load paused.
    // load(uri: string, play: boolean): void;
    // ```
    /**
     * Defines load behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun load(uri: String, play: Boolean)

    // What:     `fun play()` declares a method named `play` that takes no parameters and (no `:` after
    //           the parens) returns `Unit` (Kotlin's "nothing meaningful", i.e. `void`). No body,
    //           because this is an interface.
    // Why:      We need this so a caller can resume playback of an already-loaded track after a pause,
    //           without re-`load`-ing it (the position and decoded track are kept).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // play(): void;
    // ```
    /**
     * Defines play behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun play()

    // What:     `fun pause()` declares a method named `pause`, no parameters, returns `Unit` (void). No
    //           body (interface). It is the counterpart to `play()` above.
    // Why:      We need this so a caller can stop the sound while KEEPING the loaded track and its
    //           current position, so a later `play()` resumes from the same spot.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // pause(): void;
    // ```
    /**
     * Defines pause behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun pause()

    // What:     `fun seekTo(positionSec: Double)` declares a method named `seekTo` taking one parameter
    //           `positionSec` of type `Double`, returning `Unit` (void). `Double` is a 64-bit IEEE-754
    //           floating-point number.
    //           Type siblings worth naming:
    //             - `Float` is the 32-bit floating-point sibling (about 7 decimal digits of precision).
    //             - `Int` / `Long` are the 32-bit / 64-bit INTEGER siblings (no fractional part).
    //           `Double` is chosen (not `Float`) because a seek target in SECONDS wants sub-second
    //           precision over long tracks, and Kotlin's `Double` is the same 64-bit type as TS's
    //           `number`, so values cross the language boundary without rounding surprises; it is not
    //           `Int`/`Long` because positions are fractional.
    // Why:      We need this so the seek bar (or a "skip 10s" gesture) can jump playback to an exact
    //           time offset within the current track.
    // Gotcha:   In Kotlin, `Double` and `Float` are DIFFERENT types and do not auto-convert; you cannot
    //           pass a `Float` where a `Double` is wanted without `.toDouble()`. TS collapses both into
    //           one `number`, so this distinction is invisible there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // positionSec: target position in seconds.
    // seekTo(positionSec: number): void;
    // ```
    /**
     * Defines seek to behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun seekTo(positionSec: Double)

    // What:     `fun setVolume(volume: Float)` declares a method named `setVolume` taking one parameter
    //           `volume` of type `Float` (a 32-bit IEEE-754 floating-point number), returning `Unit`
    //           (void).
    //           Type siblings worth naming:
    //             - `Double` is the 64-bit floating-point sibling (more precision; used by `seekTo`
    //               above).
    //             - `Int`/`Long` are the integer siblings.
    //           `Float` is chosen here (not `Double`) because Android audio APIs and the native Rust
    //           engine both use 32-bit floating-point gain values at the output boundary; matching that
    //           type avoids a narrowing conversion at the boundary. It is not `Int` because the gain is a
    //           fraction in `0.0..1.0`.
    // Why:      We need this so the caller can set the output loudness as a linear gain between `0.0`
    //           (silent) and `1.0` (full), e.g. to honor a volume slider or duck during a notification.
    // Gotcha:   The value is a LINEAR gain in `0.0..1.0`, not decibels and not a 0..100 percentage.
    //           Passing `100` here would be wildly out of range, not "100%".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // volume: linear gain in 0.0..1.0.
    // setVolume(volume: number): void;
    // ```
    /**
     * Defines set volume behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun setVolume(volume: Float)

    // What:     `fun positionSec(): Double` declares a method named `positionSec` that takes no
    //           parameters and DOES return a value: the `: Double` after the parentheses is the return
    //           type, a 64-bit floating-point number. This is a "getter-style" method (it reads state),
    //           but written as a plain method, not a Kotlin property.
    //           Type sibling reminder: `Double` (64-bit) is chosen over `Float` (32-bit) so the seconds
    //           value crosses cleanly into TS's `number` and keeps sub-second precision over long
    //           tracks; not `Int`/`Long` because the position is fractional.
    // Why:      We need this so the UI can poll "where are we now?" each frame/tick and draw the seek
    //           bar's thumb at the right place. The contract says it returns `0.0` when nothing is
    //           loaded (a safe default, not an error).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // returns current position in seconds, 0.0 when nothing is loaded.
    // positionSec(): number;
    // ```
    /**
     * Defines position sec behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun positionSec(): Double

    // What:     `fun durationSec(): Double` declares a method named `durationSec`, no parameters,
    //           returning a `Double` (64-bit float) total length in seconds. Same getter-style shape as
    //           `positionSec` above.
    //           Type sibling reminder: `Double` (64-bit) over `Float` (32-bit) so the value matches
    //           TS's `number` and keeps precision; not `Int`/`Long` because durations are fractional.
    // Why:      We need this so the UI can draw the seek bar's TOTAL width / end label. The contract
    //           says it returns `0.0` when the duration is still unknown (e.g. while a stream is
    //           probing its length), which the UI treats as "not ready yet" rather than an error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // returns duration in seconds, 0.0 when still unknown.
    // durationSec(): number;
    // ```
    /**
     * Defines duration sec behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun durationSec(): Double

    // What:     `fun playWhenReady(): Boolean` declares a method named `playWhenReady`, no parameters,
    //           returning a `Boolean` (true/false). Sibling worth naming: it is NOT an `Int` flag or a
    //           three-state enum; it is a plain two-valued `Boolean`.
    // Why:      We need this so callers can read the engine's play INTENT, which is deliberately
    //           different from "is sound actually coming out". It returns `true` from the moment
    //           playback is requested (INCLUDING while the track is still buffering) until a pause, and
    //           `false` after a pause (including a pause the engine performs itself when it loses audio
    //           focus). A MediaSession reports THIS value so the notification's play/pause icon does not
    //           flicker during the buffering window. The truly-producing-sound state is a separate
    //           signal delivered through `setOnPlayingChanged` (declared just below).
    // Gotcha:   Do NOT read this as "is audio playing right now". It is INTENT (play-requested), so it
    //           can be `true` while the engine is still buffering and emitting no sound.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // true when the engine intends to play (playing OR buffering),
    // // false when paused. Distinct from "sound is actually audible".
    // playWhenReady(): boolean;
    // ```
    /**
     * Defines play when ready behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun playWhenReady(): Boolean

    // What:     `fun setOnPlayingChanged(callback: (Boolean) -> Unit)` declares a method named
    //           `setOnPlayingChanged`. Its single parameter `callback` has the FUNCTION type
    //           `(Boolean) -> Unit`: "a function that takes one `Boolean` argument and returns `Unit`
    //           (nothing)". In Kotlin a function TYPE is written `(ParamTypes) -> ReturnType`, with the
    //           parameter types in parentheses and `->` before the return type. The method itself
    //           returns `Unit` (no `:` after its own parens).
    //           Sibling note: `(Boolean) -> Unit` is the callback shape; you might have expected the
    //           bare-event sibling `() -> Unit` (no argument), which is what the NEXT method uses; here
    //           we need the `Boolean` argument because the callback must report WHICH state (running vs
    //           stopped) it changed to.
    // Why:      We need this so the rest of the app can register ONE listener that the engine invokes
    //           every time the actual play/pause sound state flips, passing `true` when playback is
    //           genuinely running and `false` when paused or stopped. This is the "real sound" signal
    //           that complements the intent-only `playWhenReady()` above.
    // Gotcha:   Naming is "set...", singular: this stores ONE callback. Calling it again replaces the
    //           previous callback rather than adding a second subscriber; it is not an event-emitter
    //           with multiple listeners.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // callback: invoked with true when playback is running,
    // //           false when paused or stopped.
    // setOnPlayingChanged(callback: (isPlaying: boolean) => void): void;
    // ```
    /**
     * Defines set on playing changed behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    fun setOnPlayingChanged(callback: (Boolean) -> Unit)

    // What:     `fun setOnTrackEnded(callback: () -> Unit)` declares a method named `setOnTrackEnded`.
    //           Its single parameter `callback` has the function type `() -> Unit`: "a function that
    //           takes NO arguments and returns `Unit` (nothing)". The empty parentheses `()` mean zero
    //           parameters; `-> Unit` means it returns nothing. The method itself also returns `Unit`.
    //           Sibling note: contrast with `(Boolean) -> Unit` on the previous method; here the
    //           callback carries no payload because "the track ended" is a bare event with nothing to
    //           report.
    // Why:      We need this so the queue/transport layer (PlayerController) can be told the instant a
    //           track plays through to its natural end, which is its cue to advance to the next track
    //           (or stop, or repeat, depending on mode).
    // Gotcha:   Same single-callback semantics as `setOnPlayingChanged`: this stores ONE handler;
    //           calling it again replaces, it does not subscribe an additional listener.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // callback: invoked when the loaded track plays through to its end.
    // setOnTrackEnded(callback: () => void): void;
    // ```
    /**
     * Defines set on track ended behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun setOnTrackEnded(callback: () -> Unit)

    // What:     `fun release()` declares a method named `release`, no parameters, returning `Unit`
    //           (void). No body (interface).
    // Why:      We need this so a caller can free the NATIVE resources the engine holds: the audio-focus
    //           registration, native Rust worker thread, native buffers, output stream, and open file
    //           handles. After `release()` the engine object is spent and must not be used again; the
    //           caller drops its reference. Android/native resources are not reclaimed by garbage
    //           collection alone, so an explicit teardown hook is required.
    // Gotcha:   This is a ONE-WAY, terminal operation. Unlike `pause()`, you cannot `play()` again after
    //           `release()`; the object is dead. Forgetting to call it leaks native resources because GC
    //           will not free them for you.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Frees native resources; the engine is unusable afterwards.
    // // Closest TS idea: an explicit dispose()/close() you must call.
    // release(): void;
    // ```
    /**
     * Defines release behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun release()

    // What:     `fun setOnLoadFailed(callback: (uri: String, reason: LoadFailure) -> Unit)` registers the handler
    //           for a load that could not start. The default body ignores it, so engines that cannot fail ignore it.
    // Why:      A stalled or refused file open happens after `load` returned (the open runs off the main thread),
    //           so the failure cannot be a return value; the controller shows a message when it arrives.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setOnLoadFailed(callback: (uri: string, reason: LoadFailure) => void): void {}
    // ```
    /** Registers the single handler told when a requested track could not be opened; the default ignores it. */
    fun setOnLoadFailed(callback: (uri: String, reason: LoadFailure) -> Unit) {
    }

    // What:     `fun unload()` stops playback and forgets the loaded track, so position and duration read zero until
    //           the next `load`. The default body only pauses.
    // Why:      A track that left the library (trashed) must not leave its position and duration on the deck.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // unload(): void { this.pause(); }
    // ```
    /** Stops playback and drops the loaded track; the default pauses. */
    fun unload() {
        pause()
    }
}

// What:     `enum class LoadFailure` lists why a requested track did not start.
// Why:      The two causes need different advice: an unreadable file is gone or damaged, a stalled open is worth a
//           retry.
//
// In TS you'd write (pseudocode):
// ```ts
// type LoadFailure = "unreadable" | "timedOut";
// ```
/** Why a requested track could not be loaded. */
enum class LoadFailure {
    /** The file could not be opened or decoded. */
    UNREADABLE,

    /** The media provider did not answer within the time limit. */
    TIMED_OUT,
}

// What:     `internal class PendingLoad(...)` is the state of one load whose descriptor is still opening.
// Why:      `play`, `pause` and `seekTo` calls that arrive before the open completes must change how the load
//           starts, not the previous track.
//
// In TS you'd write (pseudocode):
// ```ts
// type PendingLoad = { generation: number; uri: string; play: boolean; seekSec: number | null };
// ```
/** One load waiting for its descriptor, with the start state the caller last asked for. */
internal class PendingLoad(
    /** Number of the load, matched against the engine's current generation. */
    val generation: Int,
    /** Track being opened. */
    val uri: String,
    /** Whether playback should start when the load completes. */
    var play: Boolean,
    /** Position to seek to once loaded, or null to start at the beginning. */
    var seekSec: Double? = null,
)
