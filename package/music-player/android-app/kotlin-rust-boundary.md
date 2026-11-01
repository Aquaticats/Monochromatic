# Kotlin/Rust boundary for the Android music player

Status:
 SUPERSEDED on 2026-06-21 by `../PROPOSAL.shared-core.md`.

This document records one team member's opinion from a source audit on 2026-06-19:
 that the queue,
pagination,
 relpath,
 session,
 and normalization logic should stay re-implemented in Kotlin,
 keeping
the JNI surface narrow and the Compose state native.
 It favored simplicity,
 fewer moving parts and
no UI state crossing the boundary,
 over a single source of truth.

That recommendation is now superseded.
 The team's direction is that maintainability beats
simplicity.
 The same domain logic re-implemented in both Rust and Kotlin drifts (the two true-peak
implementations already differ by hundreds of lines),
 so one shared Rust core consumed by thin
platform adapters is the maintainable shape,
 even though it must marshal UI snapshots across the
boundary once per user action.
 The reasoning below is kept as historical context for why the
simpler split was once chosen;
 it no longer reflects the intended architecture.

Original scope:
 `package/music-player/android-app`.
Original goal:
 performance first,
 without turning the app into cross-language soup.

## One-sentence rule

Kotlin owns Android and user state.
Rust owns audio bytes,
 decoded samples,
 DSP,
 and the realtime output thread.

If code is mostly about screens,
 permissions,
 content URIs,
 services,
 saved settings,
 or lists of tracks,
keep it in Kotlin.
If code is mostly about PCM samples,
 decoders,
 true-peak scans,
 ring buffers,
 atomics,
 or AAudio callbacks,
keep it in Rust.

## Why this is the boundary

Kotlin is the native language of the Android framework.
The app talks to Compose,
 `ContentResolver`,
 SAF,
 MediaStore,
 WorkManager,
 `SharedPreferences`,
`MediaSessionService`,
 permissions,
 and lifecycle callbacks.
Those APIs already live on the JVM side.
Moving that work into Rust would force Kotlin to call Rust,
 then Rust to call back into Kotlin or Java,
which adds JNI crossings while making the code harder to debug.

Rust is the right side for the audio engine because the hot path is not Android UI work.
The Rust engine decodes audio to interleaved `f32` PCM,
feeds an SPSC ring buffer,
and fills AAudio's realtime callback without allocation,
 locking,
 or blocking.
That shape is documented in `rust/src/engine.rs` and `rust/src/engine_worker.rs`.
A garbage-collected Kotlin loop in the realtime sample path is the wrong tool.

JNI is the border crossing.
Cross it for coarse operations:
 load a track,
 play,
 pause,
 seek,
 set volume,
 measure a whole track,
query a few scalar values.
Do not cross it for every sample,
 every row render,
 every queue step,
 or every Compose state update.

## Current build reality

The current build is already a single full Rust engine app.
`app/build.gradle.kts` says the Media3 and hybrid flavors are gone,
and `mise.toml` drives one debug/release variant after `build:native` builds the `.so` files.
`EngineFactory.kt` returns `RustEngine(context)`.

Source comments now describe that current shape:
Kotlin owns the Android/media-session shell,
and `RustEngine` owns native decode/output behind the small `AudioEngine` contract.

Measured inventory from the source tree,
 excluding Gradle and Rust build outputs:

- Kotlin main source:
   35 files under `app/src/main/kotlin`.
- Kotlin unit tests:
   11 files under `app/src/test/kotlin`.
- Kotlin instrumented tests:
   3 files under `app/src/androidTest/kotlin`.
- Rust source:
   9 files under `rust/src`.
- Native libraries built locally for packaging:
   2 `.so` files under `app/src/main/jniLibs`
  (gitignored,
   produced by `build:native`,
   not committed).
- Current built APK artifacts found locally:
  debug APK about 23.5 MB,
  release APK about 19.6 MB,
  arm64 native library about 4.4 MB,
  x86_64 native library about 4.7 MB.

The build configuration records this as a settled decision.
`app/build.gradle.kts` states the head-to-head comparison `is decided (the full-Rust variant won decisively)`,
which is why the Media3 and hybrid flavors were removed and only one variant remains.
Aquaticat has accepted the full-Rust engine as the final architecture and does not plan to re-measure it:
the raw head-to-head numbers were not retained,
 and the comparison cannot be reproduced anyway because the
old engines are gone.
Anyone who later wants fresh numbers can rebuild and benchmark the current engine,
 but this decision does
not wait on them.

## Keep these parts in Kotlin

### UI and user interaction

Keep `MainActivity.kt`,
 `PlayerUiState.kt`,
 and the composables in Kotlin.
They are Compose code,
 they bind to the service,
 they observe Compose state,
and they handle taps,
 sliders,
 folder picking,
 and permission prompts.
Rust has no advantage here.
Every move to Rust would just marshal UI state back into Kotlin for rendering.

### Android services and media-system integration

Keep `PlaybackService.kt` and `BrainPlayer.kt` in Kotlin.
They speak `MediaSessionService`,
 `MediaSession`,
 and media3 `SimpleBasePlayer`.
The operating system expects this JVM-side shape for notification controls,
lockscreen controls,
 headset buttons,
 and external controllers.

The Rust engine can produce sound.
Kotlin must still present that sound to Android as a normal media app.

### Permissions, lifecycle, and storage framework access

Keep `Permissions.kt`,
 `LibraryRoot.kt`,
 `LibrarySource.kt`,
 `SafTreeSource.kt`,
and `MediaStoreSource.kt` in Kotlin.
They use Android permission APIs,
 `ContentResolver`,
 cursors,
 SAF document trees,
MediaStore columns,
 and content URIs.
Rust cannot read a SAF tree by itself.
Kotlin has to ask Android for the URI and descriptor anyway,
so Rust should not own the scan.

The right split is what the code already does:
Kotlin resolves a playable thing to a `content://` URI,
then opens a `ParcelFileDescriptor` only when the native engine or native peak scanner needs bytes.

### Session and app-private persistence

Keep `SessionStore.kt`,
 `LibraryRoot.kt`,
 and `PeakCacheStore.kt` in Kotlin.
They are app storage code:
 `SharedPreferences`,
 app-private files,
 JSON text,
mutexes around the in-process cache,
 and Android context access.
This is not the realtime path.

The pure in-memory `PeakCache` map can stay Kotlin too.
It is just `fingerprint -> peak` lookup and update.
The expensive part is measuring the peak,
 not storing the result.

### Queue, pagination, and controller state

Keep `PlayerController.kt`,
 `core/Queue.kt`,
 `core/Pagination.kt`,
 `core/Page.kt`,
`core/RelPath.kt`,
 `core/DisplayPath.kt`,
 `core/AudioExtensions.kt`,
`core/ShuffleMode.kt`,
 and `core/Session.kt` in Kotlin.

These files control what the user sees and what track should play next.
They work with lists of display strings,
 selected pages,
 Compose snapshots,
 and content URI identities.
`doc/decision/music-player-android-port.md` records a real library size of 3,857 files,
which is a normal Kotlin collection workload.
Moving this to Rust would mean copying track lists,
 page entries,
 selected indices,
and state snapshots across JNI just to send them back to Kotlin for display.
That is likely slower and definitely messier.

Optimize these algorithms in Kotlin if profiling finds a problem.
Do not move them to Rust only because the files are large.
Large UI-state code is not the same thing as hot audio code.

### Work scheduling

Keep `PeakSweepScheduler.kt` and `PeakSweepWorker.kt` in Kotlin.
WorkManager,
 charging/idle constraints,
 and worker lifecycle are Android framework concerns.
The worker should schedule and iterate in Kotlin,
then call the Rust true-peak scanner for each track.

## Keep these parts in Rust

### Decoding and demuxing

Keep `rust/src/decode.rs` and `rust/src/opus.rs` in Rust.
They turn a file descriptor into decoded interleaved `f32` PCM.
They use Symphonia for most formats and libopus for Opus,
which is exactly the kind of CPU-heavy byte processing Rust is good at.

Kotlin should not receive decoded PCM arrays.
The moment decoded samples cross JNI in chunks,
the boundary is in the wrong place.

### Playback engine and worker thread

Keep `rust/src/engine.rs` and `rust/src/engine_worker.rs` in Rust.
They own the playback worker,
 commands,
 atomics,
 ring buffer,
 AAudio stream,
 and realtime callback.
The callback only pops samples,
 applies gain,
 fills silence on underrun,
 and advances counters.
That is the performance-critical loop.

Kotlin should drive it through coarse commands:
`load`,
 `play`,
 `pause`,
 `seek`,
 `setVolume`,
 `setNormalizationGain`,
 and scalar state polls.
It should not own the sample loop.

### AAudio output and latency probe

Keep `rust/src/output.rs` in Rust.
It opens AAudio through the NDK bindings and reads native presentation timestamps.
This is native audio API work,
 not app UI work.

### True-peak measurement

Keep true-peak measurement in `rust/src/truepeak.rs`,
 reached from `PeakMeasurer.kt`.
A true-peak scan decodes the whole track and walks the samples.
That is CPU-heavy and naturally belongs next to the decoder.

Kotlin keeps the tiny `normalizationGain(peak)` formula in `core/Normalization.kt`.
That function maps one float to one float,
and it is called once per cache hit or measurement result.
There is no performance reason to force that tiny formula into Rust.

The unused Kotlin true-peak scanner that used to share `core/TruePeak.kt` has been removed:
production measures peaks only in Rust,
 so the Kotlin scanner was a never-run second implementation.
Its algorithm is now covered on-device against the real Rust path by
`NativeBridgeTest.nativeTruePeakInterpolatesInterSamplePeaks`,
which feeds synthetic PCM through the test-only `nativeTruePeakSynthetic` JNI entry.
The gain helpers (`normalizationGain`,
 `processSample`) and their tests moved to `core/Normalization.kt`.

### Fingerprinting, with one caveat

`TrackFingerprint.kt` queries Android for size and modified time in Kotlin,
then calls `NativeBridge.nativeFingerprint`.
The Rust side calls the shared `track_fingerprint` (XXH3-64 from `twox-hash`) through `rust/src/fingerprint.rs`.
That is justified by desktop parity, because both apps call the one function in `truepeak-core`,
not by a huge Android performance win.
Until 2026-10-14 the hash was `gxhash`, which has no JVM port.
A Kotlin copy of XXH3-64 would be a second implementation to keep bit-identical with the Rust one.

This call is acceptable because it is per track and not realtime.
Do not put it inside row rendering or the AAudio callback.
If fingerprinting ever becomes a scan bottleneck,
either batch fingerprints across one JNI call or switch to a Kotlin-owned hash for Android-only caches.

### JNI exports

Keep `rust/src/lib.rs` as the exported native surface,
and keep `NativeBridge.kt` as the thin Kotlin declaration surface.
That split is normal:
Kotlin declares `external` methods,
Rust exports `Java_dev_monochromatic_musicplayer_NativeBridge_*` symbols.

The good sign in the current bridge is that it passes primitives and file descriptors,
not giant object graphs.
Preserve that.

## The boundary contract

Future code should follow these rules.

- If it touches Android SDK classes,
   keep it Kotlin.
  Examples:
   `Context`,
   `Uri`,
   `ContentResolver`,
   `Cursor`,
   `SharedPreferences`,
   `WorkManager`,
  `MediaSession`,
   `AudioManager`,
   `BroadcastReceiver`,
   Compose state.

- If it touches decoded samples,
   keep it Rust.
  Examples:
   `f32` PCM buffers,
   oversampling,
   decoder packets,
   audio output buffers,
  ring-buffer producer/consumer halves,
   AAudio callbacks.

- If it runs on the realtime audio thread,
   keep it Rust and keep it boring.
  No allocation,
   no locks,
   no disk I/O,
   no JNI,
   no logging in the hot callback.

- If it runs once per user action,
   once per track,
   or once per app lifecycle event,
  Kotlin is usually fine.

- If it runs once per sample,
   once per frame,
   or once per output buffer,
  Rust owns it.

- Pass file descriptors to Rust,
   not Android content APIs.
  Kotlin opens the `content://` URI with `ContentResolver.openFileDescriptor`.
  Rust duplicates the borrowed fd synchronously and owns its copy.
  Kotlin closes its descriptor normally.

- Pass scalars back to Kotlin.
  Good return values:
   status codes,
   `Float` peak values,
   `Double` seconds,
   booleans,
  opaque `Long` handles.
  Bad return values:
   decoded sample arrays,
   whole queue snapshots,
   UI page trees.

- Do not add native-to-JVM callbacks from the AAudio callback.
  Kotlin polling every 200 ms for playing/ended state is not the audio hot path.
  If lower end-of-track latency is ever needed,
  signal from the worker thread,
   not from the realtime callback.

## What not to move just because Rust feels faster

Do not move pagination to Rust.
It builds UI tabs and row labels.
Rust would still hand the result back to Kotlin for Compose.

Do not move queue state to Rust unless profiling proves the Kotlin queue is a CPU bottleneck.
The queue is tied to selected pages,
 snapshots,
 shuffle controls,
 and user events.
A JNI queue would make every tap and every rescan more complicated.

Do not move SAF or MediaStore scanning to Rust.
Rust cannot magically avoid Android's content provider boundary.
Kotlin already has the cursor and URI APIs.

Do not move `SessionStore` to Rust.
Android sessions are per-device app state,
and `SharedPreferences` is the simplest correct storage.

Do not move Compose UI state to Rust.
Compose observes Kotlin state.
Rust cannot recompose the screen.

## What to clean up next

The Kotlin/Rust split has been cleaned up;
 the last item below is future guidance.

1.  Done:
     the unused Kotlin true-peak scanner was split out and deleted.
    Production has one true-peak path (Rust);
    `normalizationGain`/`processSample` and their tests live in `core/Normalization.kt`;
    the Rust path is now covered on-device (see "True-peak measurement").

2.  `AudioEngine` is the only high-level PLAYBACK-engine seam.
    The rest of Kotlin does not know about native engine handles,
     fds,
     or Rust modules.
    The only other sanctioned native crossings are `NativeBridge.nativeFingerprint` and
    `NativeBridge.nativeMeasureTruePeak` (plus the test-only `nativeTruePeakSynthetic`):
    coarse per-track calls,
     not engine internals.

3.  No benchmark evidence will be produced.
    Aquaticat has settled the full-Rust decision (see "Current build reality");
    the head-to-head cannot be reproduced because the Media3 and hybrid engines are gone,
    and re-measuring is out of scope.

4.  If a Kotlin list operation is suspected slow,
     profile before moving it.
    First try data-structure and allocation fixes in Kotlin.
    Move to Rust only if the hot loop is independent of Android and UI state.

## Decision guide for new files

Ask these questions in order.

1.  Does the file import Android framework or AndroidX UI/service/storage APIs?
    Put it in Kotlin.

2.  Does the file need to run without the garbage collector interrupting a sound buffer?
    Put it in Rust.

3.  Does the file transform compressed audio bytes into PCM or scan PCM samples?
    Put it in Rust.

4.  Does the file decide what the user sees,
     which track is selected,
     or what gets saved?
    Put it in Kotlin.

5.  Would moving it to Rust require passing lists of tracks,
     strings,
     pages,
     or UI state over JNI?
    Keep it Kotlin.

6.  Would keeping it in Kotlin require passing PCM samples over JNI?
    Put it in Rust.

That is the whole rule.
Keep the crossing narrow,
with Kotlin as the Android shell and Rust as the audio machine.
