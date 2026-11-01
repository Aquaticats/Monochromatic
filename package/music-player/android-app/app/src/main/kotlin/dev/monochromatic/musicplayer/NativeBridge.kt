// What:     `package dev.monochromatic.musicplayer` declares the namespace this file's
//           declarations belong to. This file is compiled with the app's main source set and
//           declares the JNI functions implemented by the packaged native library.
// Why:      Keeps `NativeBridge` in the same package as `RustEngine` and the shared code that
//           call into it.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword — file path is the module.
// ```
package dev.monochromatic.musicplayer

// =============================================================================
// File summary (folds in the old comment's domain content)
// =============================================================================
//
// `NativeBridge` is the app's thin Kotlin FACADE over the native engine shared library
// (`.so`, built from Rust via cargo-ndk). Every function here is `external`: it has NO
// Kotlin body; its implementation lives in the native library and is linked at load through
// JNI (Java Native Interface, the bridge between JVM code and native code).
//
// Two groups of functions are declared:
//   - SMOKE-TEST probes that proved the cargo-ndk -> JNI toolchain on device: `nativePing`,
//     `nativeOpusSelfTest`, `nativeSymphoniaSelfTest`, the decode benchmarks, and
//     `nativeOutputLatencyProbe`.
//   - the REAL engine surface the `RustEngine` drives: `nativeMeasureTruePeak` plus the
//     `nativeEngine*` create/load/transport/query/release calls.
//
// `handle: Long` everywhere is an OPAQUE native engine pointer (a 64-bit address the Rust
// side owns); `fd: Int` is a borrowed file descriptor. The JVM never dereferences either; it
// just passes them back to the native side. Integer return codes follow the convention `0` =
// success, negative = error.

// What:     `object NativeBridge { ... }` declares a SINGLETON OBJECT named `NativeBridge`:
//           Kotlin creates exactly one instance, with no constructor to call. Members are
//           reached as `NativeBridge.nativePing()`, like static methods on a namespace.
// Why:      The native bridge is global, stateless glue (the engine state lives behind the
//           native `handle`, not in this object); a singleton `object` is Kotlin's "static
//           namespace" with one shared instance.
// Gotcha:   `object X { }` here is a TYPE DECLARATION creating one global instance, not the
//           `object : Interface { }` expression used elsewhere for anonymous instances.
//
// In TS you'd write (pseudocode):
// ```ts
// export const NativeBridge = {
//   // ...native function declarations as members...
// };
// ```
/**
 * Defines native bridge object for this music-player component; the TypeScript-oriented notes above explain its
 * shared role.
 */
object NativeBridge {
    // What:     `init { System.loadLibrary("musicplayer_native") }` is an INITIALIZER BLOCK that
    //           runs once when the object is first used. `System.loadLibrary("musicplayer_native")`
    //           asks the JVM to load the native shared library named `libmusicplayer_native.so`
    //           (the `lib` prefix and `.so` suffix are added by the platform), wiring up the
    //           `external` functions below to their native implementations.
    // Why:      The `external` functions cannot be called until their native library is loaded;
    //           doing it in `init` guarantees the load happens before any bridge call.
    // Gotcha:   `System.loadLibrary` takes the library's SHORT name (`musicplayer_native`), not
    //           the on-disk filename; the platform expands it to `libmusicplayer_native.so`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // const native = loadNativeAddon("musicplayer_native"); // must run before any call below
    // ```
    init {
        System.loadLibrary("musicplayer_native")
    }

    // What:     `external fun nativePing(): Int` declares a function with NO Kotlin body (the
    //           `external` keyword): its implementation is in the native library, linked via JNI.
    //           It takes no arguments and returns an `Int` (32-bit signed integer).
    // Why:      Smoke test: the simplest possible round-trip proving the JVM can call into the
    //           native library at all.
    // Gotcha:   `external` means "implemented elsewhere (native code)"; calling it before the
    //           library is loaded would crash. A TS reader should picture an FFI/native import.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativePing(): number; // implemented in the native addon
    // ```
    /**
     * Defines native ping behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    external fun nativePing(): Int

    // What:     `external fun nativeOpusSelfTest(): Int` is a JNI native function (no Kotlin body)
    //           taking no args and returning an `Int` result/status code.
    // Why:      Smoke test: exercise the bundled Opus decoder inside the native library and report
    //           a pass/fail code.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeOpusSelfTest(): number;
    // ```
    /**
     * Defines native opus self test behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeOpusSelfTest(): Int

    // What:     `external fun nativeSymphoniaSelfTest(): Int` is a JNI native function returning an
    //           `Int` status code.
    // Why:      Smoke test: exercise the symphonia decoder in the native library and report a
    //           pass/fail code.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeSymphoniaSelfTest(): number;
    // ```
    /**
     * Defines native symphonia self test behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeSymphoniaSelfTest(): Int

    // What:     `external fun nativeDecodeBenchmark(path: String): Double` is a JNI native function
    //           taking a `String` file `path` and returning a `Double` (64-bit float). Sibling
    //           `Float` (32-bit) is declined because a benchmark duration/throughput wants the
    //           wider precision.
    // Why:      Smoke test: decode the file at `path` natively and return a timing/throughput
    //           figure for diagnosing the native decoder path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeDecodeBenchmark(path: string): number;
    // ```
    /**
     * Defines native decode benchmark behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeDecodeBenchmark(path: String): Double

    // What:     `external fun nativeDecodeFdBenchmark(fd: Int): Double` is a JNI native function
    //           taking a file descriptor `fd` (an `Int`, 32-bit) and returning a `Double` timing.
    // Why:      Smoke test: like `nativeDecodeBenchmark` but decoding from an already-open file
    //           descriptor (the `content://` path), which is how the app actually opens media.
    // Gotcha:   `fd` is a raw OS file descriptor (an `Int`), not a path or handle; the native side
    //           reads from it directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeDecodeFdBenchmark(fd: number): number;
    // ```
    /**
     * Defines native decode fd benchmark behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeDecodeFdBenchmark(fd: Int): Double

    // What:     `external fun nativeOutputLatencyProbe(): Double` is a JNI native function returning
    //           a `Double` (64-bit) latency figure.
    // Why:      Smoke test: measure the native AAudio output path's latency for diagnostics.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeOutputLatencyProbe(): number;
    // ```
    /**
     * Defines native output latency probe behavior for this music-player component; the TypeScript-oriented
     * notes above explain its call shape and effects.
     */
    external fun nativeOutputLatencyProbe(): Double

    // What:     `external fun nativeTruePeakServiceCreate(dbPath: String): Long` opens the native
    //           true-peak decision service backed by the shared `DecisionCache` at `dbPath`, returning
    //           an opaque handle (`Long`), or `0` on failure.
    // Why:      The service owns the Turso-backed cache and the shared policy; Kotlin creates one at
    //           startup with its app-private `decisions.db` path and passes the handle to the
    //           resolve/warm calls below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeTruePeakServiceCreate(dbPath: string): bigint; // 0 = failure
    // ```
    /**
     * Opens the native true-peak decision service at the given database path and returns its handle.
     */
    external fun nativeTruePeakServiceCreate(dbPath: String): Long

    // What:     `external fun nativeTruePeakServiceRelease(handle: Long)` closes the service, stopping
    //           its cache thread. No-op for the `0` handle.
    // Why:      Release the one service on shutdown so the actor thread and connection are reclaimed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeTruePeakServiceRelease(handle: bigint): void;
    // ```
    /**
     * Releases the native true-peak decision service handle, stopping its cache thread.
     */
    external fun nativeTruePeakServiceRelease(handle: Long)

    // What:     `external fun nativeResolveGain(handle: Long, fd: Int, fingerprint: Long): Float`
    //           resolves the FOREGROUND normalization gain for the current track: a cache hit returns
    //           its stored gain, a miss decodes the `fd` (probe-or-full per the shared policy), caches
    //           the decision, and returns its gain. NEVER negative: any error falls back to the safe
    //           -1 dBTP ceiling gain.
    // Why:      The gain math and the cache now live natively, so Kotlin no longer computes gain or
    //           stores peaks; it just applies whatever gain this returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeResolveGain(handle: bigint, fd: number, fingerprint: bigint): number;
    // ```
    /**
     * Resolves the foreground normalization gain for a track, caching the decision natively.
     */
    external fun nativeResolveGain(handle: Long, fd: Int, fingerprint: Long): Float

    // What:     `external fun nativeWarmTrack(handle: Long, fd: Int, fingerprint: Long): Float` is the
    //           BACKGROUND warming call: it full-scans a track to an EXACT decision and caches it,
    //           skipping tracks already cached exactly. Returns the gain (unused for playback).
    // Why:      Warming upgrades probe estimates to exact cached gains over idle time; the native
    //           cache's exact-over-probe precedence keeps the exact decision.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeWarmTrack(handle: bigint, fd: number, fingerprint: bigint): number;
    // ```
    /**
     * Warms a track: full-scans to an exact decision and caches it, skipping already-exact tracks.
     */
    external fun nativeWarmTrack(handle: Long, fd: Int, fingerprint: Long): Float

    // What:     `external fun nativeTruePeakSynthetic(samples: FloatArray, channels: Int): Float` is a
    //           TEST-ONLY JNI native function (no Kotlin body). It takes an in-memory `FloatArray` of
    //           interleaved `f32` PCM (`samples`) and the `channels` interleave width (`Int`), and
    //           returns the measured true peak (`Float`, 32-bit, matching the sample domain). Sibling
    //           `nativeMeasureTruePeak` takes a file DESCRIPTOR and decodes; this one bypasses the
    //           decoder so a caller can hand it a KNOWN signal.
    // Why:      The instrumented test (`NativeBridgeTest`) cannot assert a known golden peak through
    //           the decode path (that needs a real encoded file). This entry feeds a synthetic signal
    //           with a known inter-sample peak straight into the production `TruePeakMeter` so the test
    //           verifies the SAME native true-peak path on the real device. It is NOT called by
    //           production code.
    // Gotcha:   A negative return is a JNI read-error sentinel (`-1.0`); a real peak is `>= 0.0`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeTruePeakSynthetic(samples: Float32Array, channels: number): number; // <0 = error
    // ```
    /**
     * Defines native true peak synthetic behavior for this music-player component; the TypeScript-oriented
     * notes above explain its call shape and effects.
     */
    external fun nativeTruePeakSynthetic(samples: FloatArray, channels: Int): Float

    // What:     `external fun nativeFingerprint(path: String, size: Long, mtimeNanos: Long): Long`
    //           is a JNI native function (no Kotlin body). It takes the track `path` text, its `size`
    //           in bytes, and its modified-time `mtimeNanos` in nanoseconds (both `Long`, 64-bit
    //           signed; sibling `Int` is 32-bit, too narrow for nanosecond timestamps), and returns
    //           the raw `u64` cache key as a `Long` (bit-cast), which `nativeResolveGain`/
    //           `nativeWarmTrack` take.
    // Why:      The cache fingerprint (XXH3-64, shared with the desktop through `truepeak-core`) is
    //           computed natively (src/fingerprint.rs) and reached here. Both flavors call the SAME
    //           `track_fingerprint` function, and the shared `DecisionCache` keys on the `u64`
    //           directly, so no hex-string round-trip is needed.
    // Gotcha:   `size`/`mtimeNanos` are passed as signed `Long` but treated as unsigned native-side
    //           (sizes/timestamps are never negative); the returned `Long` is an opaque key, not a
    //           number to interpret.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeFingerprint(path: string, size: bigint, mtimeNanos: bigint): bigint;
    // ```
    /**
     * Computes the opaque native cache-key fingerprint for a track, returned as a Long.
     */
    external fun nativeFingerprint(path: String, size: Long, mtimeNanos: Long): Long

    // What:     `external fun nativePeekPeak(handle: Long, fingerprint: Long): FloatArray?` reads the cached true
    //           peak of one track without starting a measurement. The reply holds two floats: the peak in dBTP
    //           (negative infinity for silence), then the kind code (1 for an exact full scan, 0 for a sampled
    //           estimate). It is null when no decision is cached for the fingerprint, or the handle is null.
    // Why:      The track rows and the details menu show a peak only when it is already known, so the read must
    //           never decode a file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativePeekPeak(handle: bigint, fingerprint: bigint): Float32Array | null;
    // ```
    /**
     * Reads the cached true peak and its kind for a track fingerprint, or null when none is cached.
     */
    external fun nativePeekPeak(handle: Long, fingerprint: Long): FloatArray?

    // What:     `external fun nativeInvalidateTrack(handle: Long, fingerprint: Long): Boolean` removes the cached
    //           decision of one track, so its next resolve or warm measures it again. It returns true when a row
    //           was removed, and false when none was stored or the handle is null.
    // Why:      A re-analysis clears one track and no other, and the removal completes before the caller measures.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeInvalidateTrack(handle: bigint, fingerprint: bigint): boolean;
    // ```
    /**
     * Removes the cached decision of one track fingerprint, returning true when a row was removed.
     */
    external fun nativeInvalidateTrack(handle: Long, fingerprint: Long): Boolean

    // What:     `external fun nativeEngineCreate(): Long` is a JNI native function returning a `Long`
    //           (64-bit signed integer). The `Long` is an OPAQUE HANDLE: a native pointer/address
    //           the Rust engine owns, passed back on every later call. Sibling `Int` (32-bit) is
    //           declined because a native pointer needs 64 bits on a 64-bit device.
    // Why:      Create the native engine (spawning its worker thread) and return its handle; `0`
    //           means the worker could not be spawned.
    // Gotcha:   The returned `Long` is NOT a number to do math on; it is an opaque pointer. `0` is
    //           the only meaningful value to test (failure).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineCreate(): bigint; // opaque engine handle; 0n = failed
    // ```
    /**
     * Defines native engine create behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    external fun nativeEngineCreate(): Long

    // What:     `external fun nativeEngineLoad(handle: Long, fd: Int, play: Boolean): Int` is a JNI
    //           native function. `handle: Long` is the opaque engine handle; `fd: Int` is the
    //           track's file descriptor; `play: Boolean` says whether to start playing. Returns an
    //           `Int` status code (`0` = success).
    // Why:      Load the track at `fd` into the engine `handle`, optionally starting playback; the
    //           native side dups `fd` synchronously so the JVM can close its copy after.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineLoad(handle: bigint, fd: number, play: boolean): number;
    // ```
    /**
     * Defines native engine load behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    external fun nativeEngineLoad(handle: Long, fd: Int, play: Boolean): Int

    // What:     `external fun nativeEnginePlay(handle: Long)` is a JNI native function taking the
    //           opaque `handle` and returning nothing (no return type = `Unit`/void).
    // Why:      Resume playback on the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEnginePlay(handle: bigint): void;
    // ```
    /**
     * Defines native engine play behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    external fun nativeEnginePlay(handle: Long)

    // What:     `external fun nativeEnginePause(handle: Long)` is a JNI native function taking the
    //           opaque `handle`, returning `Unit` (void).
    // Why:      Pause playback on the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEnginePause(handle: bigint): void;
    // ```
    /**
     * Defines native engine pause behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    external fun nativeEnginePause(handle: Long)

    // What:     `external fun nativeEngineSeek(handle: Long, positionSec: Double)` is a JNI native
    //           function taking the `handle` and a `positionSec` (`Double`, 64-bit, seconds),
    //           returning `Unit` (void). Sibling `Float` is declined: seconds want the wider type.
    // Why:      Seek the native engine to `positionSec`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineSeek(handle: bigint, positionSec: number): void;
    // ```
    /**
     * Defines native engine seek behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    external fun nativeEngineSeek(handle: Long, positionSec: Double)

    // What:     `external fun nativeEngineSetVolume(handle: Long, volume: Float)` is a JNI native
    //           function taking the `handle` and a `volume` (`Float`, 32-bit, `0.0..1.0`), returning
    //           `Unit` (void).
    // Why:      Set the user volume on the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineSetVolume(handle: bigint, volume: number): void;
    // ```
    /**
     * Defines native engine set volume behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeEngineSetVolume(handle: Long, volume: Float)

    // What:     `external fun nativeEngineSetNormalizationGain(handle: Long, gain: Float)` is a JNI
    //           native function taking the `handle` and a `gain` (`Float`, `0.0..1.0`), returning
    //           `Unit` (void).
    // Why:      Set the per-track true-peak normalization gain on the native engine (applied inside
    //           the native output callback, where the clamp also guards clipping).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineSetNormalizationGain(handle: bigint, gain: number): void;
    // ```
    /**
     * Defines native engine set normalization gain behavior for this music-player component; the TypeScript-
     * oriented notes above explain its call shape and effects.
     */
    external fun nativeEngineSetNormalizationGain(handle: Long, gain: Float)

    // What:     `external fun nativeEnginePositionSec(handle: Long): Double` is a JNI native function
    //           taking the `handle` and returning a `Double` (seconds).
    // Why:      Query the current playback position in seconds from the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEnginePositionSec(handle: bigint): number;
    // ```
    /**
     * Defines native engine position sec behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeEnginePositionSec(handle: Long): Double

    // What:     `external fun nativeEngineDurationSec(handle: Long): Double` is a JNI native function
    //           taking the `handle` and returning a `Double` (seconds).
    // Why:      Query the current track's duration in seconds from the native engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineDurationSec(handle: bigint): number;
    // ```
    /**
     * Defines native engine duration sec behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeEngineDurationSec(handle: Long): Double

    // What:     `external fun nativeEngineIsPlaying(handle: Long): Boolean` is a JNI native function
    //           taking the `handle` and returning a `Boolean`.
    // Why:      Query whether the native engine is currently playing (the poller edge-triggers the
    //           play/pause callback off this).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineIsPlaying(handle: bigint): boolean;
    // ```
    /**
     * Defines native engine is playing behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeEngineIsPlaying(handle: Long): Boolean

    // What:     `external fun nativeEngineIsEnded(handle: Long): Boolean` is a JNI native function
    //           taking the `handle` and returning a `Boolean`.
    // Why:      Query whether the current track has ended (the poller fires the track-ended callback
    //           on the rising edge of this).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineIsEnded(handle: bigint): boolean;
    // ```
    /**
     * Defines native engine is ended behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeEngineIsEnded(handle: Long): Boolean

    // What:     `external fun nativeEngineFailedLoad(handle: Long): Long` is a JNI native function taking the
    //           `handle` and returning a `Long`.
    // Why:      `nativeEngineLoad` returns once the load is queued, so a file that opens but does not decode fails
    //           later on the engine worker; this is the sequence number of the last queued load that failed, or zero
    //           when none did, and the poller turns a new number into a load failure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineFailedLoad(handle: bigint): bigint;
    // ```
    /**
     * Sequence number of the last queued load that the engine worker could not open for playback, or zero when
     * none failed.
     *
     * @param handle engine handle from `nativeEngineCreate`
     */
    external fun nativeEngineFailedLoad(handle: Long): Long

    // What:     `external fun nativeEnginePlayWhenReady(handle: Long): Boolean` is a JNI native
    //           function taking the `handle` and returning a `Boolean`.
    // Why:      Query the engine's intended play state ("should play once ready"), the value the
    //           shared `AudioEngine` contract reports to the media-session projection.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEnginePlayWhenReady(handle: bigint): boolean;
    // ```
    /**
     * Defines native engine play when ready behavior for this music-player component; the TypeScript-oriented
     * notes above explain its call shape and effects.
     */
    external fun nativeEnginePlayWhenReady(handle: Long): Boolean

    // What:     `external fun nativeEngineRelease(handle: Long)` is a JNI native function taking the
    //           `handle` and returning `Unit` (void).
    // Why:      Release the native engine (stop its worker thread, free its resources); after this
    //           the handle is invalid and must not be reused.
    // Gotcha:   After `nativeEngineRelease`, the `handle` is dangling; the Kotlin side sets its
    //           stored handle to `0` so later calls are guarded.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // declare function nativeEngineRelease(handle: bigint): void; // handle invalid afterward
    // ```
    /**
     * Defines native engine release behavior for this music-player component; the TypeScript-oriented notes
     * above explain its call shape and effects.
     */
    external fun nativeEngineRelease(handle: Long)
}
