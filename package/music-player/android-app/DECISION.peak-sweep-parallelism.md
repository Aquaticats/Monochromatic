# Decision: parallelize the peak sweep behind an auto-started foreground service

Records why the Android peak sweep moves from one low-priority background thread to a
parallel decode pool inside an auto-started foreground service,
 and the on-device benchmark
that set the worker count and thread priority.
The question was "the sweep never finishes on a Pixel 6 while the desktop finishes the same
library in three minutes;
 do we need Turso,
 non-blocking,
 and multi-core like the desktop got?
"
Answer:
 only multi-core,
 the cache and async are already fine,
 and the real fix is parallel
decode plus escaping WorkManager's windowing.
Written so a future session does not re-run the ninety-minute benchmark.

## Decision

- Decode the sweep with a parallel pool of `max(2, availableParallelism / 2)` workers
  (four on a Pixel 6:
   the two big plus two mid performance cores) at
  `Process.THREAD_PRIORITY_DEFAULT` for the initial full index.
- Run that initial index as a foreground service that auto-starts when the app is opened and
  a backlog exists,
   not the periodic charging-gated `WorkManager` job.
   The owner accepted the
  foreground-service notification for now and plans a better launch UX later.
- Keep a low-priority background `WorkManager` job for incremental upkeep of newly added tracks,
  where a trickle of files does not need speed and should yield to playback.
- Do not adopt Turso and do not move the cache to Room.
   The cache was never the bottleneck.

Update of 2026-10-13 (the decision above stands; only the trigger and the upkeep split changed):
no activity or application class starts the foreground service.
`PlaybackService.analyzeNewTracks` calls `PeakSweepService.startIfNeeded` after the first library load,
after a picked-folder load and after every live-update rescan (D113),
and the service starts only when the library is readable and the ledger (`sweep-ledger.bin`, `core/SweepLedger.kt`)
holds new, changed or retryable tracks.
"When the app is opened" therefore means "when `PlaybackService` has loaded or rescanned the library",
and the foreground service also analyzes files added later;
the low-priority `WorkManager` job that `PeakSweepScheduler.enqueue` queues stays as the charging-plus-idle path.

## Context

The sweep measures each track's true peak once and memoizes it
(`PeakSweepWorker.kt`,
 `PeakMeasurer.kt`,
 `PeakCacheStore.kt`).
The per-track true-peak calculation is already native Rust
(`rust/src/truepeak.rs` via `NativeBridge.nativeMeasureTruePeak`);
 what is in Kotlin is the
sweep orchestration and the cache,
 so "the calc should have been Rust" does not describe the
slow part.

Three independent things made the Pixel 6 sweep never land,
 none of them the cache:

- One slow core.
   `PeakMeasurer.kt:140` runs every decode on a single
  `Executors.newSingleThreadExecutor` at `THREAD_PRIORITY_LOWEST` (`:183`),
   and the background
  cpuset pins nice-19 work to the little cluster,
   so it used one of the four slowest cores.
- WorkManager windowing.
   The sweep is a periodic,
   `setRequiresCharging(true)` job
  (`PeakSweepScheduler.kt`) capped near ten minutes per run with a fifteen-minute minimum
  period (`PeakSweepWorker.kt`),
   so a multi-thousand-track first pass is sliced across many
  short plugged-in windows and depends on Android firing them.
- Library size.
   The production enumeration returns about 3959 tracks on the owner's device.

What was ruled out by inspection,
 before benchmarking:

- The warm-restart skip is deterministic,
   so it is not a re-measure-forever bug.
   The
  fingerprint was `gxhash64(uri + size + mtime, seed = 0)` (`rust/src/fingerprint.rs:81`; since 2026-10-14
  it is the shared XXH3-64 `track_fingerprint`, see `doc/decision/music-player-fingerprint-hash.md`),
   and
  for MediaStore URIs the mtime query asks for the DocumentsContract `COLUMN_LAST_MODIFIED`
  column that MediaStore lacks,
   so it falls back to a stable `UNKNOWN_MODIFIED_MS = 0L`
  (`TrackFingerprint.kt:185`,
   `:439`).
   Same inputs every run,
   same key,
   real cache hit.
- The cache is tiny.
   Fully populated it is roughly 120 KB of JSON,
   so the rewrite-the-whole-file
  every sixteen measurements is negligible at this scale.
- It is already non-blocking (Kotlin coroutines,
   `Dispatchers.IO`,
   `WorkManager`);
   there is no
  async runtime to add.

## The benchmark (and why the first one was wrong)

A first benchmark decoded forty tracks per variant and reported about 135 tracks/min for the
default-priority configs.
 That was a cold burst:
 a real phone heats up and the governor
throttles the big cores within a minute,
 so a twenty-second sample overstates sustained
throughput.
 The corrected benchmark runs each variant for a full fifteen minutes against the
real library (the production `LibrarySource.load` order,
 decode bypassing the cache so every
variant actually decodes),
 and reports tracks completed per minute so the throttle curve is
visible.
 It samples `cpu0/cpu4/cpu6` `scaling_cur_freq` every two minutes as the corroborating
throttle signal because the thermal zones were not readable from the shell.

Device:
 Pixel 6 (Tensor gs101),
 eight cores,
 four little at 1.80 GHz (cpu0 to cpu3),
two mid at 2.25 GHz (cpu4 to cpu5),
 two big at 2.80 GHz (cpu6 to cpu7).

## Measured results

Sustained fifteen-minute throughput against 3959 real tracks,
 with extrapolated full-library
time:

-   `default` priority,
     4 workers:
     130.9 tracks/min,
     full library about 30 min.
-   `default` priority,
     6 workers:
     128.8 tracks/min,
     about 31 min.
-   `default` priority,
     8 workers:
     128.1 tracks/min,
     about 31 min.
-   `lowest` priority,
     8 workers:
     96.7 tracks/min,
     about 41 min.
-   `lowest` priority,
     4 workers:
     55.5 tracks/min,
     about 71 min.

The cold thirty-second burst was 238 tracks/min,
 so sustained throughput is roughly a 45 percent
drop from cold;
 the per-minute curves dip at minute two then hold flat for the rest of the run,
and the cpu6 frequency oscillates between 2802 MHz and roughly 1100 to 1700 MHz under load
(the governor cycling),
 confirming throttling that stabilizes rather than collapses.

Two findings set the config:

- Thread priority is the dominant lever,
   not worker count.
   Every `default` config lands near
  130 tracks/min;
   the best `lowest` config is 97.
   Letting the scheduler use the big cores is
  worth about 1.35x over the best low-priority config.
- Worker count saturates at four.
   `default` 4,
   6,
   and 8 are within two percent (four marginally
  best),
   because the Pixel 6 has exactly four performance cores and the work is throttle-bound,
  so extra threads add heat and UI contention for nothing.
   More threads only help at `lowest`
  priority (55 to 97 from four to eight) because that path is stuck on the four little cores.

Single-threaded was deliberately not run (it is the never-finishes case).
 Derived from
`lowest` 4 workers,
 one little core is about 14 tracks/min,
 so today's single nice-19 thread is
roughly `3959 / 14`,
 about 4.7 hours of pure decode before any windowing,
 which is why it never
lands.
 The chosen config does the same library in about 30 min of continuous compute.

## Why a foreground service, not just a parallel WorkManager job

The benchmark measured sustained decode under thermal load with the device awake.
 It did not
model Android's job suppression:
 the ten-minute WorkManager cap,
 the fifteen-minute period,
 the
charging gate,
 or Doze.
 Those are what chop a thirty-minute job into rare windows today.
Parallelism alone would already turn "never" into "a few charging sessions" because each window
becomes about eight times more productive,
 but an auto-started foreground service escapes the
windowing entirely and finishes the first index in one continuous session.
 The owner accepted
the notification cost.
 Incremental upkeep stays a low-priority background job because a few new
tracks do not need the big cores and should yield to playback.

## When to revisit

- A device with many more than four performance cores,
   or a much larger library,
   would justify
  re-running the variant matrix;
   the saturation point is hardware-specific.
- If the cache ever grows large enough that the full-file JSON rewrite shows up in a profile,
  the Android-correct fix is Room or `SQLiteDatabase` (native,
   JVM-side,
   incremental upserts,
  zero binary growth),
   not Turso.
   Turso would drag `turso_core` (about 6.7 MiB) into the `.so`
  to solve a non-problem.

## How this was measured (reproducible)

A throwaway instrumented benchmark (`PeakSweepBenchmark.benchmarkRealLibrary`) ran in a forked
worktree,
 not committed to main.
 It loads `LibrarySource.load`,
 decodes the library in order
with a fixed thread pool whose worker count and `Process` priority come from instrumentation
args,
 for a `durationSec` budget,
 and logs a `RESULT` line under the `PeakBench` tag with the
per-minute series.
 Drive it with `am instrument` against the installed debug app and its test
APK,
 granting `READ_MEDIA_AUDIO`,
 looping the variants with cooldowns and a `scaling_cur_freq`
sampler.
 Keep the device awake with `svc power stayon true` so a ninety-minute matrix is not
interrupted.
