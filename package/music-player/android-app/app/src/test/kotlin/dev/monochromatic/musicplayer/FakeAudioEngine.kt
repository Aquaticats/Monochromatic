// What:     `package dev.monochromatic.musicplayer` names the namespace this test-support class
//           lives under. It is the SAME package as `AudioEngine` and `PlayerController`, so this
//           file implements `AudioEngine` and is used by `PlayerControllerTest` with no imports.
//           Test source files share the main package at compile time.
// Why:      So the fake can implement the `AudioEngine` interface and the controller test can
//           construct it by its short name.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword — the file path IS the module; here it shares the SUT's namespace.
// ```
package dev.monochromatic.musicplayer

// =============================================================================
// File summary (for a TypeScript-only reader)
// =============================================================================
//
// A hand-written test double for the `AudioEngine` interface, used by
// `PlayerControllerTest` to drive `PlayerController` on a plain JVM with no
// device, no native engine, and no audio. It records the calls the controller
// makes (which URI was loaded, the last volume, the last seek) and returns
// canned values for the reads (`positionSec`, `durationSec`, `playWhenReady`).
//
// The one behaviour that MATTERS for the resume tests: `seekTo` records the
// requested position but does NOT change what `positionSec()` returns. This
// mirrors the real engine, where a seek only posts a command to a worker thread
// and `positionSec()` keeps returning 0.0 until the worker applies it. Tests can
// therefore assert "the controller did not read a real position during the
// restore" without a real engine.

// What:     `class FakeAudioEngine : AudioEngine { ... }` declares a class that IMPLEMENTS the
//           `AudioEngine` interface (the `: AudioEngine` after the name is Kotlin's "implements").
//           Because it is a concrete class, it must provide a body for every interface method,
//           each marked `override`.
// Why:      `PlayerController`'s constructor needs an `AudioEngine`; this fake supplies one whose
//           effects the test can inspect.
// Gotcha:   Kotlin interfaces are NOMINAL: a class with matching methods but no `: AudioEngine`
//           would NOT satisfy the constructor. The `: AudioEngine` is required.
//
// In TS you'd write (pseudocode):
// ```ts
// class FakeAudioEngine implements AudioEngine { /* ...fields and methods... */ }
// ```
class FakeAudioEngine : AudioEngine {
    // What:     `var loadedUri: String? = null` declares a public, REASSIGNABLE (`var`) field of
    //           NULLABLE `String?` (a `String` OR null), initialised null.
    // Why:      Records the URI of the last `load(...)` call so a test can assert which track the
    //           controller cued.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // loadedUri: string | null = null;
    // ```
    var loadedUri: String? = null

    /** Number of engine load calls observed by controller tests. */
    var loadCount: Int = 0

    // What:     `var lastVolume: Float = 1.0f` declares a public, reassignable `Float` (32-bit
    //           float; the `f` suffix makes `1.0f` a `Float`, not the 64-bit `Double` `1.0`),
    //           initialised to full gain.
    // Why:      Records the last `setVolume(...)` so a test can assert the early settings
    //           application reached the engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lastVolume = 1.0;
    // ```
    var lastVolume: Float = 1.0f

    // What:     `var lastSeek: Double? = null` declares a public, reassignable NULLABLE `Double?`
    //           (a 64-bit float OR null), initialised null.
    // Why:      Records the last `seekTo(...)` position (or null when none happened) so a test can
    //           assert the restore path issued the seek.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lastSeek: number | null = null;
    // ```
    var lastSeek: Double? = null

    // What:     `var positionValue: Double = 0.0` declares a public, reassignable `Double`,
    //           initialised `0.0`. It is what `positionSec()` returns, and it is DELIBERATELY left
    //           unchanged by `seekTo`.
    // Why:      Models the real engine's not-yet-applied seek: `positionSec()` keeps returning 0.0
    //           until a worker applies the seek. A test may set this directly to simulate a track
    //           that has been playing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // positionValue = 0.0;
    // ```
    var positionValue: Double = 0.0

    // What:     `var durationValue: Double = 0.0` declares a public, reassignable `Double`,
    //           initialised `0.0`. It is what `durationSec()` returns.
    // Why:      The controller reads duration for snapshots; a fixed value keeps the fake simple.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // durationValue = 0.0;
    // ```
    var durationValue: Double = 0.0

    // What:     `var playWhenReadyValue: Boolean = false` declares a public, reassignable
    //           `Boolean`, initialised false. It is what `playWhenReady()` returns.
    // Why:      `load(play = true)`/`play()`/`pause()` flip it, mirroring the engine's play intent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playWhenReadyValue = false;
    // ```
    var playWhenReadyValue: Boolean = false

    /** Handler that the controller registered for a load that could not start; null until it registers one. */
    private var onLoadFailed: ((uri: String, reason: LoadFailure) -> Unit)? = null

    /** Remembers the controller's load-failure handler, so [failLoad] can call it the way the real engine does. */
    override fun setOnLoadFailed(callback: (uri: String, reason: LoadFailure) -> Unit) {
        onLoadFailed = callback
    }

    /** Reports that [uri] could not be opened, as the real engine does after a tapped track fails to open. */
    fun failLoad(uri: String, reason: LoadFailure) {
        onLoadFailed?.invoke(uri, reason)
    }

    // What:     `override fun load(uri: String, play: Boolean) { ... }` provides the interface's
    //           `load`. `override` is REQUIRED in Kotlin when implementing/overriding a member.
    // Why:      Record the cued URI and the play intent for assertions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // load(uri: string, play: boolean): void { this.loadedUri = uri; this.playWhenReadyValue = play; }
    // ```
    override fun load(uri: String, play: Boolean) {
        // What:     `loadedUri = uri` records the loaded URI; `playWhenReadyValue = play` records
        //           whether it was cued playing or paused.
        // Why:      So tests can assert what the controller cued.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.loadedUri = uri; this.playWhenReadyValue = play;
        // ```
        loadedUri = uri
        loadCount += 1
        playWhenReadyValue = play
    }

    // What:     `override fun play() { playWhenReadyValue = true }` provides `play`.
    // Why:      Resume sets the play intent true.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // play(): void { this.playWhenReadyValue = true; }
    // ```
    override fun play() {
        playWhenReadyValue = true
    }

    // What:     `override fun pause() { playWhenReadyValue = false }` provides `pause`.
    // Why:      Pause clears the play intent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // pause(): void { this.playWhenReadyValue = false; }
    // ```
    override fun pause() {
        playWhenReadyValue = false
    }

    // What:     `override fun seekTo(positionSec: Double) { lastSeek = positionSec }` provides
    //           `seekTo`. It records the target but deliberately does NOT update `positionValue`.
    // Why:      Model the async seek: `positionSec()` must keep returning the old value (0.0) until
    //           a worker applies the seek, which is the exact hazard the restore path avoids.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seekTo(positionSec: number): void { this.lastSeek = positionSec; /* positionValue unchanged */ }
    // ```
    override fun seekTo(positionSec: Double) {
        lastSeek = positionSec
    }

    // What:     `override fun setVolume(volume: Float) { lastVolume = volume }` provides
    //           `setVolume`.
    // Why:      Record the gain the controller applied.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setVolume(volume: number): void { this.lastVolume = volume; }
    // ```
    override fun setVolume(volume: Float) {
        lastVolume = volume
    }

    // What:     `override fun positionSec(): Double = positionValue` provides `positionSec` as an
    //           EXPRESSION body returning the canned `positionValue`.
    // Why:      The controller reads this for `currentSession()`; returning the unchanged value is
    //           what exposes the not-yet-seeked hazard in tests.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // positionSec(): number { return this.positionValue; }
    // ```
    override fun positionSec(): Double = positionValue

    // What:     `override fun durationSec(): Double = durationValue` provides `durationSec`.
    // Why:      The controller reads duration for snapshots.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // durationSec(): number { return this.durationValue; }
    // ```
    override fun durationSec(): Double = durationValue

    // What:     `override fun playWhenReady(): Boolean = playWhenReadyValue` provides
    //           `playWhenReady`.
    // Why:      Reports the recorded play intent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playWhenReady(): boolean { return this.playWhenReadyValue; }
    // ```
    override fun playWhenReady(): Boolean = playWhenReadyValue

    // What:     `override fun setOnPlayingChanged(callback: (Boolean) -> Unit) { }` provides the
    //           callback setter with an EMPTY body. `(Boolean) -> Unit` is the function type "takes
    //           a Boolean, returns nothing".
    // Why:      The controller wires this in its `init`; the fake ignores it because no test drives
    //           real playing-state changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setOnPlayingChanged(callback: (b: boolean) => void): void { /* ignored */ }
    // ```
    override fun setOnPlayingChanged(callback: (Boolean) -> Unit) {
    }

    // What:     `override fun setOnTrackEnded(callback: () -> Unit) { }` provides the
    //           track-ended callback setter with an EMPTY body. `() -> Unit` is "takes nothing,
    //           returns nothing".
    // Why:      The controller wires this in its `init`; no test drives natural track ends, so the
    //           fake ignores it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setOnTrackEnded(callback: () => void): void { /* ignored */ }
    // ```
    override fun setOnTrackEnded(callback: () -> Unit) {
    }

    // What:     `override fun release() { }` provides `release` with an EMPTY body.
    // Why:      The fake holds no native resources, so teardown is a no-op.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // release(): void { /* nothing to free */ }
    // ```
    override fun release() {
    }
}
