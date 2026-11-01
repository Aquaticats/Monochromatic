// FILE SUMMARY (folds in the old class doc):
// This is an on-device JUnit instrumentation test. It is the proof that the
// native Rust/C engine (`libmusicplayer_native.so`, built by cargo-ndk and
// cross-compiled for arm64) actually loaded and works across the JNI boundary
// on a real GrapheneOS phone. "JNI" = Java Native Interface, the bridge that
// lets Kotlin/Java call into compiled native code; think of it as TS calling a
// native Node addon (.node file) via N-API. Each `@Test` method below pokes one
// `external` native function on `NativeBridge` and asserts it behaved.
// Run it via `am instrument` (NOT `connectedAndroidTest`): the gradle task
// uninstalls + reinstalls the app, which would wipe the SAF (Storage Access
// Framework) permission grant the device tests rely on to read real music.
//
// In TS terms there is no exact analogue: imagine a Jest/Vitest suite that can
// only run on the physical device, where each test calls a function from a
// native addon and checks the result.

// What:     `package dev.monochromatic.musicplayer` declares the namespace this
//           file's class lives in. Every Kotlin file starts with the package it
//           belongs to; the dotted path mirrors the directory layout.
// Why:      So other Kotlin files (and the JNI loader) can refer to this test's
//           class and to `NativeBridge` by a stable fully-qualified name.
//
// In TS you'd write (pseudocode):
// ```ts
// // (implicit) this file lives under src/dev/monochromatic/musicplayer/
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.ContentUris` pulls in the Android SDK helper
//           class `ContentUris`. Kotlin `import` names one symbol (a class here)
//           from another package so we can write `ContentUris` instead of the
//           full dotted path.
// Why:      We need it later to build a `content://` URI for one specific audio
//           row by appending its numeric id (see `withAppendedId`).
//
// In TS you'd write (pseudocode):
// ```ts
// import { ContentUris } from "android/content";
// ```
import android.content.ContentUris

// What:     `import android.net.Uri` pulls in Android's `Uri` type, the parsed
//           representation of a URI such as `content://media/external/...`.
// Why:      We collect and pass `Uri` values that point at indexed music tracks.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Uri } from "android/net";
// ```
import android.net.Uri

// What:     `import android.provider.MediaStore` brings in `MediaStore`, the
//           Android content provider that indexes all audio/video/images on the
//           device. We read its `Audio.Media` table to find real music files.
// Why:      The device tests decode actual library tracks, so we must query
//           MediaStore for their ids and column metadata.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MediaStore } from "android/provider";
// ```
import android.provider.MediaStore

// What:     `import android.util.Log` brings in Android's logcat logger. `Log.i`
//           etc. write tagged lines visible via `adb logcat`.
// Why:      The benchmarks emit their timing/peak numbers to logcat under the
//           tag `NativeBench` so a human can read them off the device.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import androidx.test.platform.app.InstrumentationRegistry` brings in
//           the entry point that hands a running instrumentation test a handle to
//           the Android runtime (the app `Context`, the target package, etc.).
// Why:      We need the app `Context` to reach the content resolver and the
//           app's external files directory.
//
// In TS you'd write (pseudocode):
// ```ts
// import { InstrumentationRegistry } from "androidx/test/platform/app";
// ```
import androidx.test.platform.app.InstrumentationRegistry

// What:     `import org.junit.Assert.assertEquals` imports ONE static method
//           (`assertEquals`) from JUnit's `Assert` class, not the whole class.
//           Kotlin lets you import an individual member function this way.
// Why:      So we can call `assertEquals(expected, actual)` bare to assert two
//           values match, failing the test if they don't.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertEquals } from "junit/Assert";
// ```
import org.junit.Assert.assertEquals

// What:     `import org.junit.Assert.assertTrue` imports JUnit's `assertTrue`
//           static method. It fails the test unless the boolean argument is true;
//           an optional first `String` becomes the failure message.
// Why:      The benchmarks assert their numeric results are positive/sane.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertTrue } from "junit/Assert";
// ```
import org.junit.Assert.assertTrue

// What:     `import org.junit.Assume.assumeTrue` imports JUnit's `assumeTrue`.
//           Unlike `assertTrue`, a false assumption does NOT fail the test; it
//           SKIPS it (marks it as ignored). The optional first `String` is the
//           skip reason.
// Why:      Device tests that need a fixture file or an indexed library should
//           skip (not fail) when that precondition is absent on this phone.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assumeTrue } from "junit/Assume"; // assumeTrue(cond) => skip if false
// ```
import org.junit.Assume.assumeTrue

// What:     `import org.junit.Test` imports the `@Test` annotation marker. A
//           method tagged `@Test` is discovered and run by the JUnit runner.
// Why:      Every test method below carries `@Test`; without this import the
//           annotation would not resolve.
//
// In TS you'd write (pseudocode):
// ```ts
// import { test } from "junit"; // used as the @Test decorator below
// ```
import org.junit.Test

// What:     `import java.io.File` brings in the JDK's `File` class, an abstract
//           pathname (it represents a path; it does not hold open file content).
// Why:      One benchmark builds a `File` for a pushed fixture and checks whether
//           it exists on disk before decoding it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { File } from "java/io";
// ```
import java.io.File

// What:     `class NativeBridgeTest { ... }` declares a class named
//           `NativeBridgeTest`. JUnit instantiates this class and runs each of
//           its `@Test` methods. No constructor parameters, no inheritance.
// Why:      JUnit groups tests by class; this class is the suite that proves the
//           native engine works on device.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("NativeBridgeTest", () => { /* @Test methods become it(...) */ });
// ```
class NativeBridgeTest {
    // What:     `@Test` is an annotation (metadata attached to the next
    //           declaration) telling the JUnit runner "this method is a test".
    //           It is NOT a function call; it decorates the method below it.
    // Why:      Marks `nativePingCrossesJniBoundary` as a runnable test case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("nativePingCrossesJniBoundary", () => { /* body */ });
    // ```
    @Test
    // What:     `fun nativePingCrossesJniBoundary() { ... }` declares a method
    //           named `nativePingCrossesJniBoundary` that takes no parameters and
    //           returns nothing (`Unit`, Kotlin's implicit void). `fun` is
    //           Kotlin's keyword for "function".
    // Why:      Cheapest possible smoke test: call the native `nativePing` and
    //           confirm the JNI call round-trips a value back into Kotlin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function nativePingCrossesJniBoundary(): void { /* body */ }
    // ```
    fun nativePingCrossesJniBoundary() {
        // What:     `assertEquals(42, NativeBridge.nativePing())` calls the native
        //           function `nativePing` (declared `external` on the
        //           `NativeBridge` object, implemented in Rust/C in the .so) and
        //           asserts its returned `Int` equals the literal `42`. The first
        //           argument to `assertEquals` is the EXPECTED value, the second
        //           is the ACTUAL.
        // Why:      `nativePing` is hard-coded native-side to return 42; getting
        //           42 back proves the .so loaded and the JNI call path works.
        // Gotcha:   `Int` is a fixed-width 32-bit signed integer here, not TS's
        //           floating `number`; the comparison is integer-exact.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(nativeAddon.nativePing()).toBe(42);
        // ```
        assertEquals(42, NativeBridge.nativePing())
    }

    // What:     `@Test` annotation marking the next method as a JUnit test case.
    // Why:      Registers `fingerprintIsDeterministicOpaqueAndChangeSensitiveOnDevice`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("fingerprintIsDeterministicOpaqueAndChangeSensitiveOnDevice", () => { /* body */ });
    // ```
    @Test
    // What:     `fun fingerprintIsDeterministicOpaqueAndChangeSensitiveOnDevice() { ... }` declares a
    //           no-arg, `Unit`-returning test method. Domain note: this is the on-device successor to
    //           the old pure-Kotlin host test `PeakCacheTest.fingerprintIsStableOpaqueAndChangeSensitive`.
    //           The fingerprint hash is XXH3-64 (the shared `truepeak-core` `track_fingerprint`), reached
    //           via `NativeBridge.nativeFingerprint`. It pins the same contract the host test did:
    //           DETERMINISM (same inputs -> same key), a non-zero key, and CHANGE-SENSITIVITY to size,
    //           mtime, and path. The exact digests are pinned separately by
    //           `fingerprintMatchesTheReferenceKnownAnswersOnDevice`.
    // Why:      Prove the native fingerprint behaves like a cache key on the actual CPU.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function fingerprintIsDeterministicOpaqueAndChangeSensitiveOnDevice(): void { /* body */ }
    // ```
    fun fingerprintIsDeterministicOpaqueAndChangeSensitiveOnDevice() {
        // What:     `val path = "/music/Artist/Album/a.flac"`. A read-only `String` local holding a
        //           fixed baseline path (the same literal the old host test used). `val` is TS `const`.
        // Why:      One path reused across the vectors; its basename `a.flac` is what the opacity check
        //           confirms the hex key does NOT contain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const path = "/music/Artist/Album/a.flac";
        // ```
        val path = "/music/Artist/Album/a.flac"
        // What:     `val size = 5L`. The `L` suffix makes this a `Long` (64-bit signed integer; sibling
        //           `Int` is 32-bit). `nativeFingerprint` takes the size as a `Long` (reinterpreted
        //           unsigned native-side), so a plain `Long` is what we pass.
        // Why:      A fixed file size (5 bytes) feeding the fingerprint's size input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const size = 5n; // bigint stands in for Kotlin's Long
        // ```
        val size = 5L
        // What:     `val mtimeNanos = 1_000_000_000L`. A `Long` (the `L` suffix); the `_` are cosmetic
        //           DIGIT SEPARATORS (one billion ns = 1 second).
        // Why:      A fixed modified-time (1 second past the epoch, in nanoseconds) feeding the mtime
        //           input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const mtimeNanos = 1_000_000_000n;
        // ```
        val mtimeNanos = 1_000_000_000L

        // What:     `val first = NativeBridge.nativeFingerprint(path, size, mtimeNanos)`. Calls the
        //           `external` native function across JNI; returns the raw `u64` cache key as a `Long`.
        // Why:      The reference key the later assertions compare against.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const first = NativeBridge.nativeFingerprint(path, size, mtimeNanos);
        // ```
        val first = NativeBridge.nativeFingerprint(path, size, mtimeNanos)
        // What:     `assertEquals(first, NativeBridge.nativeFingerprint(path, size, mtimeNanos))`.
        //           `assertEquals(expected, actual)`: a SECOND call with the SAME inputs must equal the
        //           reference key.
        // Why:      Determinism: identical inputs yield an identical key, which is what makes a cache
        //           lookup hit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(NativeBridge.nativeFingerprint(path, size, mtimeNanos)).toEqual(first);
        // ```
        assertEquals(first, NativeBridge.nativeFingerprint(path, size, mtimeNanos))
        // What:     `assertTrue("fingerprint was zero", first != 0L)`. The key is now a raw `u64`
        //           returned as a `Long` (not a hex string), so there is no length or substring to
        //           check; a real hash is non-zero, and `0L` is only the native read-failure sentinel.
        // Why:      Opacity is inherent in a 64-bit hash (it carries no path text); a non-zero value
        //           confirms the native fingerprint computed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(first).not.toEqual(0n);
        // ```
        assertTrue("fingerprint was zero", first != 0L)
        // What:     `assertTrue("...", first != NativeBridge.nativeFingerprint(path, 6L, mtimeNanos))`.
        //           Fails unless the keys DIFFER. The second call keeps path and mtime but uses a
        //           different size (`6L` vs `5L`). `!=` is structural inequality on `Long`.
        // Why:      Change-sensitivity to size: a re-encoded file (new size) must invalidate the entry.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(NativeBridge.nativeFingerprint(path, 6n, mtimeNanos)).not.toEqual(first);
        // ```
        assertTrue("size change did not change key", first != NativeBridge.nativeFingerprint(path, 6L, mtimeNanos))
        // What:     Same "must differ" check, varying only the mtime (`2_000_000_000L`, 2 seconds, vs
        //           the baseline 1 second).
        // Why:      Change-sensitivity to mtime: an in-place edit (same size, new mtime) must invalidate
        //           the entry.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(NativeBridge.nativeFingerprint(path, size, 2_000_000_000n)).not.toEqual(first);
        // ```
        assertTrue("mtime change did not change key", first != NativeBridge.nativeFingerprint(path, size, 2_000_000_000L))
        // What:     Same "must differ" check, varying only the path (`b.flac` instead of `a.flac`).
        // Why:      Change-sensitivity to path: two different tracks with identical size and mtime must
        //           get distinct keys.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(NativeBridge.nativeFingerprint("/music/Artist/Album/b.flac", size, mtimeNanos)).not.toEqual(first);
        // ```
        assertTrue("path change did not change key", first != NativeBridge.nativeFingerprint("/music/Artist/Album/b.flac", size, mtimeNanos))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test case.
    // Why:      Registers `fingerprintMatchesTheReferenceKnownAnswersOnDevice` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("fingerprintMatchesTheReferenceKnownAnswersOnDevice", () => { /* body */ });
    // ```
    @Test
    // What:     `fun fingerprintMatchesTheReferenceKnownAnswersOnDevice() { ... }` declares a no-arg test
    //           method that calls the REAL JNI entry with invented inputs and compares the returned `Long`
    //           with constants printed by the xxHash reference C implementation (`XXH3_64bits`) over the
    //           same material (domain tag, path length, path, size, mtime). The same constants are asserted
    //           host-side in `truepeak-core/src/fingerprint_tests.rs`, so this test proves the Android JNI
    //           path (string decoding, the signed `Long` to `u64` reinterpretation, the return cast) hashes
    //           exactly like the desktop. The cases cover a non-ASCII path, a size above `Long.MAX_VALUE`
    //           (passed as `-1L`), an empty path, and paths of 300 and 1100 bytes so the vector kernels run.
    // Why:      A change of hash crate, seed, tag, or field order on the Android side fails here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function fingerprintMatchesTheReferenceKnownAnswersOnDevice(): void { /* body */ }
    // ```
    fun fingerprintMatchesTheReferenceKnownAnswersOnDevice() {
        /** An ASCII path of exactly [length] bytes: a slash, then the lowercase alphabet repeating. */
        fun longPath(length: Int): String =
            "/" + (1 until length).map { index -> 'a' + ((index - 1) % 26) }.joinToString("")
        assertEquals(
            "plain path",
            0x6a80e982cfb4dab6uL.toLong(),
            NativeBridge.nativeFingerprint("/music/invented/a.flac", 12_345_678L, 1_700_000_000_123_456_789L),
        )
        assertEquals("empty path", 0x13d18443ce11969auL.toLong(), NativeBridge.nativeFingerprint("", 0L, 0L))
        assertEquals(
            "non-ASCII path, size above Long.MAX_VALUE, mtime Long.MAX_VALUE",
            0x5c24c1d06ceeb36fuL.toLong(),
            NativeBridge.nativeFingerprint("/musique/été/chanson.ogg", -1L, Long.MAX_VALUE),
        )
        assertEquals(
            "300-byte path",
            0x8eddf5f59a842a1cuL.toLong(),
            NativeBridge.nativeFingerprint(longPath(300), 987_654_321L, 1_600_000_000_000_000_001L),
        )
        assertEquals(
            "1100-byte path",
            0x0362151749ae2a5euL.toLong(),
            NativeBridge.nativeFingerprint(longPath(1100), 4_294_967_296L, 1_700_000_000_999_999_999L),
        )
    }

    // What:     `@Test` annotation marking the next method as a JUnit test case.
    // Why:      Registers `opusDecoderConstructsOnDevice` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("opusDecoderConstructsOnDevice", () => { /* body */ });
    // ```
    @Test
    // What:     `fun opusDecoderConstructsOnDevice() { ... }` declares a no-arg,
    //           void-returning test method. Domain note: it proves the bundled
    //           libopus C library cross-compiled for arm64 and actually runs on
    //           device. The native side (`nativeOpusSelfTest`) constructs an opus
    //           decoder via `opus_decoder_create` and returns 1 on success.
    // Why:      Linking a C library is one thing; constructing a decoder at
    //           runtime on this CPU is the real proof.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function opusDecoderConstructsOnDevice(): void { /* body */ }
    // ```
    fun opusDecoderConstructsOnDevice() {
        // What:     `assertEquals(1, NativeBridge.nativeOpusSelfTest())` calls the
        //           `external` native `nativeOpusSelfTest` and asserts its `Int`
        //           return equals `1` (the native success sentinel).
        // Why:      `1` means the opus decoder was created successfully on device.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(nativeAddon.nativeOpusSelfTest()).toBe(1);
        // ```
        assertEquals(1, NativeBridge.nativeOpusSelfTest())
    }

    // What:     `@Test` annotation marking the next method as a test case.
    // Why:      Registers `symphoniaRegistryInitializesOnDevice`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("symphoniaRegistryInitializesOnDevice", () => { /* body */ });
    // ```
    @Test
    // What:     `fun symphoniaRegistryInitializesOnDevice() { ... }`. Domain note:
    //           proves symphonia (a pure-Rust decoder covering all the codecs)
    //           cross-compiled and links into the .so. Native side
    //           (`nativeSymphoniaSelfTest`) initializes symphonia's format prober
    //           plus codec registry and returns 1.
    // Why:      Confirms the Rust audio stack is present and self-initializes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function symphoniaRegistryInitializesOnDevice(): void { /* body */ }
    // ```
    fun symphoniaRegistryInitializesOnDevice() {
        // What:     `assertEquals(1, NativeBridge.nativeSymphoniaSelfTest())`
        //           calls the native self-test and asserts `Int` `1`.
        // Why:      `1` means symphonia's prober + codec registry initialized.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(nativeAddon.nativeSymphoniaSelfTest()).toBe(1);
        // ```
        assertEquals(1, NativeBridge.nativeSymphoniaSelfTest())
    }

    // What:     `@Test` annotation marking the next method as a test case.
    // Why:      Registers `benchmarkNativeDecode`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("benchmarkNativeDecode", () => { /* body */ });
    // ```
    @Test
    // What:     `fun benchmarkNativeDecode() { ... }`. Domain note: benchmarks
    //           native decode-to-PCM throughput on device for a pushed opus and
    //           flac fixture, logging results under tag `NativeBench` for a
    //           head-to-head against the Media3/MediaCodec baseline. It SKIPS
    //           (does not fail) when a fixture is absent; you push fixtures to the
    //           app's external files dir as `bench.opus` / `bench.flac` first. A
    //           negative native result is an error code.
    // Why:      Measures whether the Rust decode path is fast enough on hardware.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function benchmarkNativeDecode(): void { /* body */ }
    // ```
    fun benchmarkNativeDecode() {
        // What:     `val context = InstrumentationRegistry.getInstrumentation()`
        //           `.targetContext`. `val` declares an immutable local (like TS
        //           `const`). `getInstrumentation()` returns the running test's
        //           instrumentation handle; `.targetContext` is the application
        //           `Context` under test (gives access to app dirs, resolvers).
        // Why:      We need the app `Context` to find the external files directory
        //           where fixtures were pushed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const context = getInstrumentation().targetContext;
        // ```
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        // What:     `val dir = context.getExternalFilesDir(null)` asks the app for
        //           its app-private external files directory. `null` (the `type`
        //           argument) means "the root of that dir", not a typed subfolder.
        //           This returns a nullable `File?` (could be null if external
        //           storage is unavailable).
        // Why:      Fixtures (`bench.opus`, `bench.flac`) are pushed into this dir;
        //           we build paths under it.
        // Gotcha:   `null` is a real argument value here, not absence; the API
        //           overload uses it to mean "no type subdirectory".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const dir = context.getExternalFilesDir(null); // File | null
        // ```
        val dir = context.getExternalFilesDir(null)
        // What:     `for (name in listOf("bench.opus", "bench.flac")) { ... }`.
        //           `listOf(...)` builds a read-only `List<String>` of two literal
        //           filenames. `for (name in <list>)` iterates each element,
        //           binding it to the immutable loop variable `name`.
        // Why:      We run the same benchmark routine once per fixture filename.
        // Gotcha:   Kotlin's `for (x in xs)` is value iteration (TS `for...of`),
        //           NOT index iteration like a JS `for (x in obj)` (which walks
        //           keys). Easy to misread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const name of ["bench.opus", "bench.flac"]) { /* body */ }
        // ```
        for (name in listOf("bench.opus", "bench.flac")) {
            // What:     `val fixture = File(dir, name)` constructs a `File` from a
            //           parent directory (`dir`) and a child name (`name`). This
            //           is a constructor call (no `new` keyword in Kotlin); it
            //           just joins the path, it does not open or read anything.
            // Why:      Gives a concrete path we can existence-check and pass to
            //           the native decoder by absolute path.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const fixture = new File(dir, name);
            // ```
            val fixture = File(dir, name)
            // What:     `assumeTrue("missing fixture $name", fixture.exists())`.
            //           `$name` is a string-template interpolation: Kotlin splices
            //           the value of `name` into the string. `fixture.exists()`
            //           returns a `Boolean`. `assumeTrue(msg, cond)` SKIPS (does
            //           not fail) the test when `cond` is false, recording `msg`.
            // Why:      If the fixture was never pushed, skip this iteration's
            //           benchmark instead of failing the suite.
            // Gotcha:   `assumeTrue` is SKIP-on-false, unlike `assertTrue` which is
            //           FAIL-on-false. The distinction is the whole point here.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (!fixture.exists()) return; // assumeTrue => skip the test
            // ```
            assumeTrue("missing fixture $name", fixture.exists())
            // What:     `val usPerSample = NativeBridge.nativeDecodeBenchmark(`
            //           `fixture.absolutePath)`. `fixture.absolutePath` is a
            //           property getter returning the file's absolute path as a
            //           `String`. We pass it to the `external` native benchmark,
            //           which returns a `Double` (microseconds per decoded sample;
            //           negative encodes a native error code).
            // Why:      Captures the throughput number for logging and asserting.
            // Gotcha:   `Double` (64-bit float) is chosen over `Float` (32-bit)
            //           because microsecond timings want the extra precision; both
            //           collapse to TS `number`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const usPerSample = nativeAddon.nativeDecodeBenchmark(fixture.absolutePath);
            // ```
            val usPerSample = NativeBridge.nativeDecodeBenchmark(fixture.absolutePath)
            // What:     `Log.i("NativeBench", "$name -> $usPerSample us/sample`
            //           `(native symphonia/opus, decode-only)")`. `Log.i(tag, msg)`
            //           writes an info-level logcat line. `$name` and
            //           `$usPerSample` are string-template interpolations.
            // Why:      Emits the timing to logcat so a human can read off the
            //           benchmark result under the `NativeBench` tag.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.info("NativeBench", `${name} -> ${usPerSample} us/sample (native symphonia/opus, decode-only)`);
            // ```
            Log.i("NativeBench", "$name -> $usPerSample us/sample (native symphonia/opus, decode-only)")
            // What:     `assertTrue("decode failed for $name (native code`
            //           `$usPerSample)", usPerSample > 0.0)`. Fails the test
            //           UNLESS `usPerSample > 0.0`. The first arg is the failure
            //           message (with `$name`/`$usPerSample` interpolated); `0.0`
            //           is a `Double` literal.
            // Why:      A non-positive result means the native decoder failed (it
            //           returns negative error codes); assert it actually decoded.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // expect(usPerSample > 0).toBe(true); // else: "decode failed for ${name}"
            // ```
            assertTrue("decode failed for $name (native code $usPerSample)", usPerSample > 0.0)
        }
    }

    // What:     `@Test` annotation marking the next method as a test case.
    // Why:      Registers `aaudioOutputLatencyOnDevice`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("aaudioOutputLatencyOnDevice", () => { /* body */ });
    // ```
    @Test
    // What:     `fun aaudioOutputLatencyOnDevice() { ... }`. Domain note: opens a
    //           silent low-latency AAudio output stream (via raw `ndk::audio` in
    //           Rust) and reads its presentation latency, proving the pure-Rust
    //           AAudio output path opens and runs on this GrapheneOS device. It is
    //           inaudible (writes zeros), so the resident-noise rule isn't engaged.
    // Why:      Confirms the native audio OUTPUT path works, not just decode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function aaudioOutputLatencyOnDevice(): void { /* body */ }
    // ```
    fun aaudioOutputLatencyOnDevice() {
        // What:     `val latencyMs = NativeBridge.nativeOutputLatencyProbe()`
        //           calls the `external` native probe, which opens the silent
        //           stream and returns its latency in milliseconds as a `Double`
        //           (negative encodes a native error code).
        // Why:      Captures the measured latency for logging and assertion.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const latencyMs = nativeAddon.nativeOutputLatencyProbe();
        // ```
        val latencyMs = NativeBridge.nativeOutputLatencyProbe()
        // What:     `Log.i("NativeBench", "AAudio output latency = $latencyMs ms`
        //           `(ndk::audio, silent)")`. Info-level logcat line; `$latencyMs`
        //           is interpolated.
        // Why:      Surfaces the latency number to a human reading logcat.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // console.info("NativeBench", `AAudio output latency = ${latencyMs} ms (ndk::audio, silent)`);
        // ```
        Log.i("NativeBench", "AAudio output latency = $latencyMs ms (ndk::audio, silent)")
        // What:     `assertTrue("AAudio output probe failed (native code`
        //           `$latencyMs)", latencyMs > 0.0)`. Fails unless the latency is
        //           positive; `0.0` is a `Double` literal.
        // Why:      A non-positive value is the native error sentinel; assert the
        //           probe actually opened the stream.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(latencyMs > 0).toBe(true); // else: "AAudio output probe failed"
        // ```
        assertTrue("AAudio output probe failed (native code $latencyMs)", latencyMs > 0.0)
    }

    // What:     `@Test` annotation marking the next method as a test case.
    // Why:      Registers `nativeTruePeakInterpolatesInterSamplePeaks`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("nativeTruePeakInterpolatesInterSamplePeaks", () => { /* body */ });
    // ```
    @Test
    // What:     `fun nativeTruePeakInterpolatesInterSamplePeaks() { ... }`. Feeds the
    //           native true-peak meter known synthetic signals through the test-only
    //           `nativeTruePeakSynthetic` JNI entry and asserts the result, proving the
    //           Rust `TruePeakMeter` + `catmull_rom` path runs correctly on this real
    //           device. These golden cases were ported from the deleted Kotlin
    //           `core/TruePeakTest` scanner tests, which exercised a Kotlin meter that
    //           production never ran; this drives the actual production native path.
    // Why:      After deleting the unused Kotlin scanner, the production true-peak path
    //           is Rust-only; this is its on-device coverage.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function nativeTruePeakInterpolatesInterSamplePeaks(): void { /* body */ }
    // ```
    fun nativeTruePeakInterpolatesInterSamplePeaks() {
        // What:     `val signal = floatArrayOf(0.0f, 0.0f, 0.9f, -0.9f, 0.9f, -0.9f, 0.0f, 0.0f)`.
        //           A mono signal with a sharp alternating transient (the `0.9, -0.9`
        //           runs) whose reconstructed inter-sample peak should EXCEED the raw
        //           stored peak of 0.9. `floatArrayOf` builds a primitive `FloatArray`
        //           (no boxing), the shape the JNI entry expects.
        // Why:      The same vector the old Kotlin `meterReportsInterSamplePeak` used;
        //           it forces the Catmull-Rom interpolation to overshoot.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const signal = new Float32Array([0, 0, 0.9, -0.9, 0.9, -0.9, 0, 0]);
        // ```
        val signal = floatArrayOf(0.0f, 0.0f, 0.9f, -0.9f, 0.9f, -0.9f, 0.0f, 0.0f)
        // What:     `val rawPeak = 0.9f`. The largest stored-sample magnitude in
        //           `signal`, written as a literal (`Float`) because the vector is fixed.
        // Why:      The inter-sample (true) peak must be at least this raw peak.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rawPeak = 0.9;
        // ```
        val rawPeak = 0.9f
        // What:     `val measured = NativeBridge.nativeTruePeakSynthetic(signal, 1)`.
        //           Call the test-only JNI entry with the mono (`channels = 1`) signal;
        //           it returns the native meter's measured true peak as a `Float`.
        // Why:      Run the real Rust meter over the synthetic signal on this device.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const measured = NativeBridge.nativeTruePeakSynthetic(signal, 1);
        // ```
        val measured = NativeBridge.nativeTruePeakSynthetic(signal, 1)
        // What:     `assertTrue("native read error $measured", measured >= 0.0f)`.
        //           A negative return is the JNI read-error sentinel; assert it did not
        //           occur. `$measured` interpolates the value into the failure message.
        // Why:      Distinguish a JNI failure from a genuine measurement.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(measured >= 0).toBe(true);
        // ```
        assertTrue("native true-peak read error (sentinel $measured)", measured >= 0.0f)
        // What:     `assertTrue("measured $measured should be >= raw $rawPeak", measured >= rawPeak - 1e-4f)`.
        //           The measured inter-sample peak must be at least the raw peak, within
        //           a `1e-4f` tolerance.
        // Why:      Interpolation can only overshoot the stored samples; a result below
        //           the raw peak would mean the oversampling is not running.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(measured >= rawPeak - 1e-4).toBe(true);
        // ```
        assertTrue(
            "measured peak $measured should be at least the raw peak $rawPeak",
            measured >= rawPeak - 1e-4f,
        )
        // What:     `assertTrue("measured $measured should be a sane, finite level", measured < 4.0f)`.
        //           Guard against a runaway/NaN/infinite result; a real inter-sample
        //           peak for this signal stays well under 4.0.
        // Why:      Catches a broken native path that returns garbage rather than a peak.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(measured < 4.0).toBe(true);
        // ```
        assertTrue("measured peak $measured should be a sane, finite level", measured < 4.0f)
        // What:     `val silence = NativeBridge.nativeTruePeakSynthetic(floatArrayOf(0.0f, 0.0f, 0.0f, 0.0f), 1)`.
        //           Measure an all-zero mono signal.
        // Why:      Silence must measure as ~0, the value that maps to unity
        //           normalization gain on the Kotlin side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const silence = NativeBridge.nativeTruePeakSynthetic(new Float32Array([0, 0, 0, 0]), 1);
        // ```
        val silence = NativeBridge.nativeTruePeakSynthetic(floatArrayOf(0.0f, 0.0f, 0.0f, 0.0f), 1)
        // What:     `assertTrue("silence peak $silence should be ~0", silence in -1e-4f..1e-4f)`.
        //           Assert the silence peak is within a tiny band around zero using
        //           Kotlin's `in range` containment check on a `Float` range.
        // Why:      A non-zero peak for pure silence would indicate a meter bug.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(Math.abs(silence) <= 1e-4).toBe(true);
        // ```
        assertTrue("silence peak $silence should be ~0", silence in -1e-4f..1e-4f)
        // What:     `val flat = NativeBridge.nativeTruePeakSynthetic(floatArrayOf(0.5f, 0.5f, 0.5f, 0.5f, 0.5f, 0.5f), 1)`.
        //           Measure a constant (DC) mono signal at 0.5.
        // Why:      A constant signal has NO inter-sample overshoot (the cubic through
        //           equal points is flat), so the true peak must equal the sample level.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const flat = NativeBridge.nativeTruePeakSynthetic(new Float32Array([0.5, 0.5, 0.5, 0.5, 0.5, 0.5]), 1);
        // ```
        val flat = NativeBridge.nativeTruePeakSynthetic(floatArrayOf(0.5f, 0.5f, 0.5f, 0.5f, 0.5f, 0.5f), 1)
        // What:     `assertTrue("flat DC peak $flat should be ~0.5", flat in 0.5f - 1e-3f..0.5f + 1e-3f)`.
        //           Assert the constant-signal peak is ~0.5 within a small tolerance.
        // Why:      Proves the interpolation does not invent overshoot where there is
        //           none, complementing the overshoot case above.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(Math.abs(flat - 0.5) <= 1e-3).toBe(true);
        // ```
        assertTrue("flat DC peak $flat should be ~0.5", flat in 0.5f - 1e-3f..0.5f + 1e-3f)
    }

    // What:     `@Test` marks the true-peak decision service on-device check.
    // Why:      Registers `truePeakServiceOpensAndResolvesOnDevice` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("truePeakServiceOpensAndResolvesOnDevice", () => { /* body */ });
    // ```
    @Test
    // What:     `fun truePeakServiceOpensAndResolvesOnDevice() { ... }`. Creates the native
    //           true-peak decision service (opening a Turso `decisions.db` in the app cache dir),
    //           resolves the gain for one real library track through it (decode + cache), resolves
    //           the SAME fingerprint again to prove the cache round-trips, then releases the service.
    // Why:      Proves the shared Turso-backed decision service builds AND runs on the real arm64
    //           device: the db opens in app-private storage, a decode-and-cache yields a sane
    //           attenuate-only gain, and a second resolve returns the cached gain. Skips when no
    //           library is indexed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function truePeakServiceOpensAndResolvesOnDevice(): void { /* body */ }
    // ```
    fun truePeakServiceOpensAndResolvesOnDevice() {
        /** The app context under test, for the content resolver and cache dir. */
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        /** The content resolver used to query MediaStore and open track descriptors. */
        val resolver = context.contentResolver
        /** A throwaway decision database in the app cache dir (never the real decisions.db). */
        val dbFile = File(context.cacheDir, "test-decisions-${System.nanoTime()}.db")
        dbFile.delete()
        /** The opened native service handle; 0 means Turso failed to open on device. */
        val service = NativeBridge.nativeTruePeakServiceCreate(dbFile.absolutePath)
        assertTrue("true-peak service failed to open Turso on device", service != 0L)
        try {
            /** The audio table on the external volume. */
            val collection = MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL)
            /** The first indexed library track URI, or null when the library is empty. */
            val trackUri: Uri? = resolver.query(
                collection,
                arrayOf(MediaStore.Audio.Media._ID),
                null,
                null,
                "${MediaStore.Audio.Media._ID} ASC",
            )?.use { cursor ->
                if (cursor.moveToFirst()) ContentUris.withAppendedId(collection, cursor.getLong(0)) else null
            }
            if (trackUri == null) {
                Log.i("NativeBench", "no indexed library tracks on device; skipping service resolve check")
                return
            }
            /** A stable per-track cache key for this test (any unique Long works). */
            val fingerprint = trackUri.toString().hashCode().toLong()
            /** First resolve: a cache miss decodes, caches, and returns the gain. */
            val firstGain = resolver.openFileDescriptor(trackUri, "r")?.use { pfd ->
                NativeBridge.nativeResolveGain(service, pfd.fd, fingerprint)
            } ?: -100.0f
            Log.i("NativeBench", "service resolve (${trackUri.lastPathSegment}) -> gain=$firstGain")
            assertTrue("resolved gain out of (0, 1]: $firstGain", firstGain > 0.0f && firstGain <= 1.0001f)
            /** Second resolve, same fingerprint: a cache hit must return the stored gain. */
            val cachedGain = resolver.openFileDescriptor(trackUri, "r")?.use { pfd ->
                NativeBridge.nativeResolveGain(service, pfd.fd, fingerprint)
            } ?: -100.0f
            assertEquals("cache round-trip changed the gain", firstGain, cachedGain, 0.0f)
        } finally {
            NativeBridge.nativeTruePeakServiceRelease(service)
            dbFile.delete()
        }
    }

    // What:     `@Test` annotation marking the next method as a test case.
    // Why:      Registers `decodeFromContentFd`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // it("decodeFromContentFd", () => { /* body */ });
    // ```
    @Test
    // What:     `fun decodeFromContentFd() { ... }`. Domain note: decodes a real
    //           library track straight from a `content://` file descriptor, the
    //           exact path the full-Rust engine will use. Proves on this
    //           GrapheneOS device that (1) MediaProvider hands back a seekable
    //           regular-file fd, (2) symphonia probes and decodes over a BORROWED
    //           `ParcelFileDescriptor`, and (3) the dup-based fd-ownership protocol
    //           does not double-close (which would be a deterministic fdsan
    //           SIGABRT). The fd is the borrowed `pfd.fd` (`getFd`) and decode
    //           happens synchronously inside `use {}`, so Rust dups before Kotlin
    //           closes the original. Needs `READ_MEDIA_AUDIO` (granted via
    //           `adb shell pm grant` before the run); skips (not fails) when the
    //           permission or indexed library is absent. Silent (decode-only).
    // Why:      Final proof the engine's real input path (content fd) works.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function decodeFromContentFd(): void { /* body */ }
    // ```
    fun decodeFromContentFd() {
        // What:     `val context = ...targetContext` — the app `Context` (same as
        //           earlier methods).
        // Why:      Needed to reach the content resolver.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const context = getInstrumentation().targetContext;
        // ```
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        // What:     `val resolver = context.contentResolver` — the content
        //           resolver (same as earlier).
        // Why:      We query MediaStore and open content fds through it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const resolver = context.contentResolver;
        // ```
        val resolver = context.contentResolver
        // What:     `val collection = MediaStore.Audio.Media.getContentUri(`
        //           `MediaStore.VOLUME_EXTERNAL)` — the external audio table URI
        //           (same as earlier).
        // Why:      The table we query for music rows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const collection = MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL);
        // ```
        val collection = MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL)
        // What:     `val projection = arrayOf(MediaStore.Audio.Media._ID,`
        //           `MediaStore.Audio.Media.DISPLAY_NAME)`. `arrayOf(...)` builds a
        //           typed array of the two column names we want to read: the row id
        //           and the display filename.
        // Why:      We need both the id (to build a URI) and the name (to pick the
        //           file extension) for each row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const projection = [MediaStore.Audio.Media._ID, MediaStore.Audio.Media.DISPLAY_NAME];
        // ```
        val projection = arrayOf(MediaStore.Audio.Media._ID, MediaStore.Audio.Media.DISPLAY_NAME)
        // What:     `val wantedExtensions = listOf(".flac", ".opus", ".mp3")`.
        //           `listOf(...)` builds a READ-ONLY `List<String>` of the three
        //           file extensions we want one sample of each. Sibling
        //           `mutableListOf` would allow `.add`; we don't need that here.
        // Why:      We pick the first track matching each of these extensions.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const wantedExtensions = [".flac", ".opus", ".mp3"] as const;
        // ```
        val wantedExtensions = listOf(".flac", ".opus", ".mp3")
        // What:     `val firstByExtension = mutableMapOf<String, Uri>()`.
        //           `mutableMapOf<String, Uri>()` constructs an empty, GROWABLE map
        //           from `String` keys (the extension) to `Uri` values. The
        //           `<String, Uri>` are explicit generic type arguments. Sibling
        //           `mapOf` would be read-only; we need mutable to insert.
        // Why:      Records the first track URI found per extension, deduplicating.
        // Gotcha:   Like lists, Kotlin separates read-only `Map` from
        //           `MutableMap`; `mutableMapOf` is what makes index-set legal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const firstByExtension = new Map<string, Uri>();
        // ```
        val firstByExtension = mutableMapOf<String, Uri>()
        // What:     `resolver.query(collection, projection,`
        //           `"${MediaStore.Audio.Media.IS_MUSIC} != 0", null, null)?.use {`
        //           `cursor -> ... }`. Same shape as the earlier query: run the
        //           query (nullable `Cursor?`), and `?.use { cursor -> ... }`
        //           safe-calls `.use` to run the lambda and auto-close the cursor.
        //           The WHERE clause `${...IS_MUSIC} != 0` filters to music rows.
        // Why:      Open a cursor over music rows and guarantee it closes after.
        // Gotcha:   `?.` safe-call + `.use {}` auto-close, same as before; no TS
        //           one-liner equivalent.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const cursor = resolver.query(collection, projection, `${IS_MUSIC} != 0`, null, null);
        // if (cursor) { try { /* lambda body */ } finally { cursor.close(); } }
        // ```
        resolver.query(collection, projection, "${MediaStore.Audio.Media.IS_MUSIC} != 0", null, null)?.use { cursor ->
            // What:     `val idColumn = cursor.getColumnIndexOrThrow(`
            //           `MediaStore.Audio.Media._ID)` — index of the `_ID` column,
            //           throwing if absent.
            // Why:      Needed to read each row's id.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const idColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID);
            // ```
            val idColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID)
            // What:     `val nameColumn = cursor.getColumnIndexOrThrow(`
            //           `MediaStore.Audio.Media.DISPLAY_NAME)` — index of the
            //           display-name column, throwing if absent.
            // Why:      Needed to read each row's filename to pick its extension.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const nameColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DISPLAY_NAME);
            // ```
            val nameColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DISPLAY_NAME)
            // What:     `while (cursor.moveToNext() && firstByExtension.size <`
            //           `wantedExtensions.size) { ... }`. Advance row by row
            //           (`moveToNext()` returns `Boolean`) while we still have fewer
            //           collected extensions than we want.
            // Why:      Stop scanning once we've found one track per wanted ext.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // while (cursor.moveToNext() && firstByExtension.size < wantedExtensions.length) { /* body */ }
            // ```
            while (cursor.moveToNext() && firstByExtension.size < wantedExtensions.size) {
                // What:     `val name = cursor.getString(nameColumn)?.lowercase()`
                //           `?: continue`. Concepts:
                //           - `cursor.getString(nameColumn)` reads the name column
                //             as a nullable `String?`.
                //           - `?.lowercase()` safe-calls `.lowercase()` only if the
                //             string is non-null, producing a lowercased copy (or
                //             null).
                //           - `?: continue` is the ELVIS operator with a control-flow
                //             right side: if the left is null, run `continue` to skip
                //             to the next loop iteration. `continue` here is the
                //             whole expression's value-position fallback.
                // Why:      Get a normalized lowercase filename, or skip this row if
                //           it has no name.
                // Gotcha:   `?:` Elvis with `continue` on the right is idiomatic
                //           Kotlin for "default-or-bail"; there's no single TS
                //           operator that branches control flow this way.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const raw = cursor.getString(nameColumn);
                // if (raw == null) continue;
                // const name = raw.toLowerCase();
                // ```
                val name = cursor.getString(nameColumn)?.lowercase() ?: continue
                // What:     `val extension = wantedExtensions.firstOrNull {`
                //           `name.endsWith(it) } ?: continue`. Concepts:
                //           - `wantedExtensions.firstOrNull { ... }` returns the
                //             first element matching the predicate, or null if none.
                //             `{ name.endsWith(it) }` is a trailing-lambda predicate;
                //             `it` is the implicit single parameter (the current
                //             extension).
                //           - `?: continue` Elvis: if no extension matched (null),
                //             skip to the next row.
                // Why:      Find which wanted extension this filename ends with, or
                //           skip the row if it's none of them.
                // Gotcha:   `it` is Kotlin's auto-named single lambda parameter
                //           (no `(it) =>` needed); `firstOrNull` is `Array.find`.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const extension = wantedExtensions.find((ext) => name.endsWith(ext));
                // if (extension == null) continue;
                // ```
                val extension = wantedExtensions.firstOrNull { name.endsWith(it) } ?: continue
                // What:     `if (extension !in firstByExtension) { ... }`. `!in` is
                //           the negated membership operator: true when `extension` is
                //           NOT a key of the map `firstByExtension`. (For a map,
                //           `in`/`!in` test the KEYS.)
                // Why:      Only record the FIRST track per extension; ignore later
                //           duplicates of an extension we already have.
                // Gotcha:   `in`/`!in` on a map check keys (like `Map.has`), not
                //           values; on a list/range they check membership.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (!firstByExtension.has(extension)) { /* body */ }
                // ```
                if (extension !in firstByExtension) {
                    // What:     `firstByExtension[extension] =`
                    //           `ContentUris.withAppendedId(collection,`
                    //           `cursor.getLong(idColumn))`. Concepts:
                    //           - `firstByExtension[extension] = ...` is index-set
                    //             syntax that inserts/updates the map entry.
                    //           - `cursor.getLong(idColumn)` reads the row id as a
                    //             `Long` (64-bit; ids can exceed 32-bit `Int`).
                    //           - `ContentUris.withAppendedId(collection, id)` builds
                    //             the per-row `Uri`.
                    // Why:      Remember this track's URI under its extension key.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // firstByExtension.set(extension, ContentUris.withAppendedId(collection, cursor.getLong(idColumn)));
                    // ```
                    firstByExtension[extension] = ContentUris.withAppendedId(collection, cursor.getLong(idColumn))
                }
            }
        }
        // What:     `assumeTrue("no indexed MediaStore audio (grant`
        //           `READ_MEDIA_AUDIO)", firstByExtension.isNotEmpty())`. SKIPS the
        //           test (does not fail) when the map is empty.
        // Why:      Nothing to decode without an indexed library, so skip.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (firstByExtension.size === 0) return; // assumeTrue => skip
        // ```
        assumeTrue("no indexed MediaStore audio (grant READ_MEDIA_AUDIO)", firstByExtension.isNotEmpty())
        // What:     `for ((extension, uri) in firstByExtension) { ... }`. Iterates
        //           the map's entries, DESTRUCTURING each entry into two locals:
        //           `extension` (the key) and `uri` (the value). `(a, b)` on the
        //           left of `in` is Kotlin component-destructuring.
        // Why:      Decode each collected track, labeled by its extension.
        // Gotcha:   `(extension, uri)` is destructuring of a Map.Entry, equivalent
        //           to TS's `[key, value]` array destructuring in `for...of`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const [extension, uri] of firstByExtension) { /* body */ }
        // ```
        for ((extension, uri) in firstByExtension) {
            // What:     `val usPerSample = resolver.openFileDescriptor(uri, "r")`
            //           `?.use { pfd -> NativeBridge.nativeDecodeFdBenchmark(pfd.fd)`
            //           `} ?: -100.0`. Concepts:
            //           - `openFileDescriptor(uri, "r")` opens the content URI for
            //             reading, nullable `ParcelFileDescriptor?`.
            //           - `?.use { pfd -> ... }` safe-call: only if non-null, run the
            //             lambda (param `pfd`) and AUTO-CLOSE the pfd after. `pfd.fd`
            //             is the borrowed raw fd passed to the native fd-benchmark,
            //             which returns a `Double` (us/sample; negative is an error).
            //           - `?: -100.0` Elvis: if the open failed (null), use `-100.0`
            //             (a `Double` sentinel; no `f` suffix).
            // Why:      Decode straight from the borrowed content fd and capture the
            //           throughput, falling back to a clearly-bad sentinel on open
            //           failure.
            // Gotcha:   Borrowed fd: native side dups it before this `.use` closes
            //           the original (the dup-based fd-ownership protocol that
            //           avoids a double-close fdsan SIGABRT).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const pfd = resolver.openFileDescriptor(uri, "r");
            // let usPerSample: number;
            // if (pfd) { try { usPerSample = nativeDecodeFdBenchmark(pfd.fd); } finally { pfd.close(); } }
            // else { usPerSample = -100; }
            // ```
            val usPerSample = resolver.openFileDescriptor(uri, "r")?.use { pfd ->
                NativeBridge.nativeDecodeFdBenchmark(pfd.fd)
            } ?: -100.0
            // What:     `Log.i("NativeBench", "content-fd $extension ($uri) ->`
            //           `$usPerSample us/sample (native symphonia/opus,`
            //           `decode-only)")`. Info logcat line; `$extension`, `$uri`,
            //           `$usPerSample` interpolated.
            // Why:      Logs which extension/track decoded and how fast.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.info("NativeBench", `content-fd ${extension} (${uri}) -> ${usPerSample} us/sample (native symphonia/opus, decode-only)`);
            // ```
            Log.i("NativeBench", "content-fd $extension ($uri) -> $usPerSample us/sample (native symphonia/opus, decode-only)")
            // What:     `assertTrue("content-fd decode failed for $extension`
            //           `(native code $usPerSample)", usPerSample > 0.0)`. Fails
            //           unless the throughput is positive; `0.0` is a `Double`
            //           literal.
            // Why:      A non-positive value is the native error sentinel; assert
            //           the content-fd decode actually succeeded.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // expect(usPerSample > 0).toBe(true); // else: "content-fd decode failed for ${extension}"
            // ```
            assertTrue("content-fd decode failed for $extension (native code $usPerSample)", usPerSample > 0.0)
        }
    }
}
