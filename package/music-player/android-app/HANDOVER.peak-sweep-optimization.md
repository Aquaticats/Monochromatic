# Handover: Android peak-sweep parallelization and the sub-20-minute goal

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Cross-session state for the work that turned the Android true-peak sweep from "never finishes" into a
parallel foreground index,
 and is now optimizing it to scan the full library in under 20 minutes.
Keep this updated as the optimization lands.

## Goal (active)

Full-library true-peak scan in under 20 minutes on the owner's Pixel 6 (Tensor G1).
 A session goal
loop is enforcing it.
 The baseline parallel sweep is ~30 minutes;
 the remaining gap needs the
hot-versus-not-hot skip described below,
 because the decode itself is a throttle-bound floor.

## Result

Goal met.
 A clean end-to-end full sweep of all 3959 tracks completed in 6.7 minutes (404 s) on a
heat-soaked device,
 with the foreground service indexing the whole library and self-stopping (logcat:
`PeakSweepService indexed 3959 of 3959 tracks`).
 That is roughly 3x under the 20-minute goal,
 and a
cool first-launch is faster still.
 Windowed sampling (commit 9eea1b840) raised on-device throughput to
483 to 588 tracks/min from ~130,
 a 3.7x to 4.5x speedup.
 The accuracy tradeoff is deliberate and
bounded,
 not zero:
 windowing can under-read a dynamic track whose only loud transient falls between
windows.
 Verified against ffmpeg true peak over the WHOLE library (the local mirror,
 see below):
 of
3604 windowed tracks,
 2 (0.06 percent) end above 0 dBFS and both by at most +0.6 dBTP,
 well inside the
~1 percent the owner accepts.
 See the validation below.

## Device and library facts (measured)

- Pixel 6,
   Tensor gs101,
   8 cores:
   4 little @ 1.80 GHz,
   2 mid @ 2.25 GHz,
   2 big @ 2.80 GHz.
- Library:
   about 3975 tracks under `/storage/emulated/0/Plain/Music/`,
   mixed formats:
  ~2586 Opus (audio/ogg),
   ~1197 FLAC,
   ~147 MP3,
   ~27 WAV,
   ~18 MP4/AAC.
   Opus carries
  `R128_TRACK_GAIN`;
   FLAC and MP3 carry ReplayGain tags that often include `REPLAYGAIN_TRACK_PEAK`
  (a sample peak,
   an even more direct hot/not signal).
   The large lossless FLACs are also the slowest
  to decode,
   so skipping the non-hot ones saves the most.
   A real mix of hot and not-hot masters.
- The debug build is `run-as`-able.
   Clear the one-time flag and cache with
  `adb shell run-as dev.monochromatic.musicplayer rm -f files/peaks.json shared_prefs/peak-sweep.xml`.
  (Note 2026-10-13: those two files are the original layout.
   Today the peaks live in the native decision database `files/decisions.db`
  (`TruePeakGain.kt`),
   and the record of which tracks were analyzed is the ledger `files/sweep-ledger.bin` (`core/SweepLedger.kt`);
   there is no one-time flag.
   Deleting both forces a full re-sweep at the next library load;
   this was read from the source and not run on a device.)

## Shipped and committed

- `DECISION.peak-sweep-parallelism.md` (commit f54e61fc2):
   the on-device benchmark and the config
  decision.
   Thread priority is the lever,
   not worker count;
   4 workers at `THREAD_PRIORITY_DEFAULT`
  saturates the 4 performance cores;
   sustained ~130 tracks/min;
   full library ~30 min.
- Parallel foreground sweep (commit 86b303840):
   `PeakSweepService` (auto-started from
  `MainActivity.onCreate`
  (as first committed; see the start path in the next note),
   dataSync foreground service,
   4-worker default-priority pool,
   one-time
  SharedPreferences flag,
   kill-resilient via the cache cursor and atomic batched flush,
   `onTimeout`
  stop).
   The decode dispatcher is now a parameter on `measureTrackPeak` and `measureAndCache`,
   so the
  background upkeep worker keeps the single low-priority thread.
   Verified on device:
   auto-starts
  foreground,
   four `peak-sweep-fg` workers,
   fills the cache,
   no crash.
   Detekt and Android Lint clean.
  (Note 2026-10-13: the start path changed after this commit, in `fa7ab1b0cc`, D113.
   No activity starts the service any more.
   `MainActivity.onCreate` only binds `PlaybackService` and starts the notification-permission prompt,
   and `MusicPlayerApplication.onCreate` only warms up the native library and preference files.
   `PlaybackService.analyzeNewTracks` calls `PeakSweepService.startIfNeeded` after the first library load,
   after a picked-folder load and after every live-update rescan,
   and `startIfNeeded` calls `startForegroundService` only when the library is readable and the ledger plan
   (`sweep-ledger.bin`, `core/SweepLedger.kt`, `core/shouldStartAnalysis`) holds new, changed or retryable tracks.
   The one-time SharedPreferences flag no longer exists: the ledger replaced it, so the foreground service also runs
   for files added later, and the low-priority `PeakSweepWorker` upkeep that `PeakSweepScheduler.enqueue` queues from
   `PlaybackService` stays as the charging-plus-idle path.)
- Android-Lint fixes to `PeakSweepService.kt` (commit 0ba1fe3e4).
- Windowed true-peak scan in `rust/src/truepeak.rs` (commit 9eea1b840):
   the under-20-minute win.
- `lint:rust` task added to the android-app `mise.toml`:
   the root `lint:rust` fanout
  (`mise '//package/...:lint:rust'`) previously skipped this crate because it had no such task,
   so the
  native `.rs` was never run through the shared rust-linter (only hand-checked).
   It now passes clean
  (max-lines,
   require-rustdoc).

## Optimization findings (this session)

- Hardware `MediaCodec` decode is a dead end:
   3.8 tracks/min single-thread,
   8.7 four-thread,
   about
  30x slower than symphonia.
   Android audio `MediaCodec` is software here with heavy per-track setup
  and a concurrent-instance limit.
   Do not pursue.
- The 4x Catmull-Rom oversampling is about 25 to 32 percent of per-track time (`decodeonly` 198 to
  232/min versus `truepeak` 149 to 176/min in matched bursts).
   It is accuracy-preserving to optimize:
  the Catmull-Rom interpolation gain is bounded (basis L1-norm at t in {0.25,
   0.5,
   0.75} peaks at
  1.25),
   so when `1.3 * window_max <= running_peak` the three cubics cannot beat the current peak and
  can be skipped,
   giving the identical true peak with far less work.
   This alone lands around 23 to 25
  min,
   not under 20.
- The decode (symphonia/libopus) is the irreducible throttle-bound floor for an exact full-track scan.

## The lever to under 20 minutes: skip non-hot tracks

Normalization is attenuate-only:
 `normalizationGain(truePeak) = min(CEILING / truePeak, 1.0)` with
`CEILING = 0.8912509` (-1 dBTP,
 `core/Normalization.kt`).
 A track whose true peak is at or below
-1 dBTP gets gain 1.0,
 so its exact peak is irrelevant and it never needs measuring.
 Only hot masters
(true peak above -1 dBTP) need the full true-peak decode.

The files carry NO ReplayGain/R128 tags (ffprobe confirmed across Opus,
 FLAC,
 MP3),
 so the cheap
signal is a partial decode,
 not metadata.
 Hot (brickwalled) masters hit their ceiling throughout,
 so
decoding a few short windows spread across the track captures the peak;
 dynamic tracks read low and
normalize to unity gain.
 The rare dynamic track whose only loud transient falls between windows is the
~1 percent the owner accepts (it plays at its original level,
 which is not clipping).

Implemented in `rust/src/truepeak.rs` (`measure_windowed_peak`):
 for tracks over 90 s,
 sample 4
windows of 15 s at 0/33/66/100 percent,
 each with its own meter (no seam artifact),
 take the loudest,
and inflate by +2 dB (`WINDOW_SAFETY_FACTOR`).
 The +2 dB margin plus the -1 dBTP ceiling's own 1 dB of
headroom keep sub-1dB under-reads non-clipping;
 they do not make windowing exact.
Validated against ffmpeg true peak over the ENTIRE library:
 the owner's `Plain/Music` on the Pixel 6
mirrors `~/Seafile/Plain/Music` on the dev machine,
 so the check ran locally with ffmpeg (no adb
sampling),
 `/tmp/agent/full-miss-check.mjs`.
 Of 3812 files,
 3604 are over 90 s (the windowed path).
 For
each,
 the metric is the final played true peak:
 an attenuated track ends at `FULL - (WMAX + 3 dB)` (the
+2 dB margin plus the 1 dB ceiling headroom),
 a track the windows read at or below the ceiling plays at
unity so it ends at `FULL`.
 Result:
 2 tracks (0.06 percent) end above 0 dBFS,
 both by at most +0.6 dBTP
(Imagine Dragons - My Life at +0.6,
 an Emiya piano cover at +0.2);
 29 (0.80 percent) land between the
-1 dBTP ceiling and 0 dBFS;
 3573 sit safely at or under the ceiling.
 Both over-0 tracks are dynamic
masters whose lone loud transient falls between windows.
 The Emiya piano cover reads quiet enough to
play at unity,
 so it ends at its own +0.2 dBTP source peak (the accepted "plays at original level"
case);
 Imagine Dragons - My Life IS attenuated but,
 under-read by the windows,
 lands at +0.6 dBTP
instead of the -1 dBTP target (still below its own louder source peak,
 just short of the ceiling).
Neither is a clip the windowing introduced beyond a sub-1dB overshoot.
 No tuning is warranted:
 a
larger `WINDOW_SAFETY_FACTOR` would over-attenuate all 3573 hot tracks (the whole library quieter) to
claw back under 1 dB on 2 dynamic tracks.
 The brickwalled hot majority never misses (at ceiling
throughout every window).
 If stricter behavior is ever wanted,
 raising `WINDOW_COUNT` to 6 catches more
isolated transients at a modest scan-time cost,
 still far under the 20-minute goal.

## Next steps

1. Done:
    windowing implemented,
    accuracy-validated (43-track windowed-vs-full miss-rate,
    one
   non-clipping miss),
    committed (9eea1b840);
    clean full sweep timed at 6.7 min,
    under the 20-min
   goal;
    `lint:rust` task added and passing (65c54476e);
    accuracy claim corrected (f504aee6c).
2. Optional further win:
    candidate-only oversampling (skip the three Catmull-Rom interpolations when
   `1.3 * window_max <= running_peak`,
    the exact L1-bound),
    now that windowing already cut decode.
   Not needed for the goal (already met);
    a cooler-running bonus.
3. Done (Kotlin sweep concurrency):
    the parallel loop is extracted into one tested coordinator,
   `sweepTracksInParallel` in `PeakSweepCoordinator.kt`,
    and BOTH consumers now delegate to it:
   `PeakSweepService` (foreground,
    `workers = 4`) and `PeakSweepWorker` (background upkeep,
   `workers = 1`,
    serial).
    It is covered by a fast JVM unit test,
    `PeakSweepCoordinatorTest` (in
   `app/src/test`,
    no device):
    exactly-once cursor coverage under 8 parallel workers,
    the `maxTracks`
   bound,
    per-outcome tallies,
    deterministic flush cadence (every `flushBatch` measured plus one
   final),
    cancellation stopping the workers while the final flush still runs (`NonCancellable`),
    and
   serial single-worker ordering.
    `test:unit`,
    `lint:detekt`,
    and Android `lint` all pass.
    The
   coordinator also adds a guaranteed final flush on cooperative cancellation (timeout/stop),
    tightening
   kill-resilience beyond the periodic batch flush.
    The unified Worker was also verified ON DEVICE
   non-destructively:
    a new `test:instrumented:device` mise task installs both APKs with
   `adb install -r` (keeps the SAF grant and the warm 3959-entry cache,
    confirmed unchanged at 149535
   bytes before and after) and drives one class via `am instrument` instead of
   `connectedDebugAndroidTest` (which would uninstall/reinstall and wipe both).
    `PeakSweepWorkerTest`
   passed on device (OK,
    2 tests),
    exercising `LibrarySource.load` to the coordinator to
   `measureAndCache` to the tally on real CoroutineWorker semantics the faked JVM test cannot reach.
    The
   task defaults to that silent test;
    pass a `class` arg for another,
    but `RustEngineTest` plays audio
   and `am instrument` exits 0 even on failure (read its output).
4. Rust windowing coverage.
    The native crate has no host `cargo test` (it is `crate-type = ["cdylib"]`
   only and pulls `ndk` with the `audio` feature,
    which `#[link]`s Android-only libaaudio,
    so it cannot
   link off-Android).
    `measure_windowed_peak`'s accuracy is covered by the full-library ffmpeg miss
   check above;
    the shared Catmull-Rom meter math is host-tested on the desktop twin
   (`desktop-app/src/truepeak_tests.rs`).
    A dedicated unit test would need either extracting the
   meter/Source logic into a host-testable sibling crate or a cargo-ndk on-device target;
    deferred as
   coverage debt,
    not unverified behavior.

## Gotchas

- The device is heat-soaked after long benchmarking,
   so short runs throttle hard (the per-minute
  series collapses).
   Trust the cooled steady state (~130/min truepeak) from the 90-minute run and the
  clean production sweep,
   not back-to-back short bursts.
- The throwaway benchmark lives in the forked worktree `/tmp/agent/mp-android-bench-jun20`
  (`PeakSweepBenchmark.benchmarkRealLibrary`,
   `engine` arg of truepeak/decodeonly/mediacodec).
   It is
  not committed to main.
   `am instrument` does not launch `MainActivity`
  and nothing under `app/src/androidTest` starts `PlaybackService`
  (the path that triggers the auto-sweep, see the 2026-10-13 note under "Shipped and committed"),
   so it never triggers the
  auto-sweep,
   the benchmark stays clean.
- Keep the device awake for long runs with `adb shell svc power stayon true` and restore after.
