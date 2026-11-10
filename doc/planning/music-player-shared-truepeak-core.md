# Shared true-peak core plan for the music player

Status:
 implementation in progress.
Stage zero (Turso on Android) is de-risked;
 see the `Turso on Android` risk note under `Risks and review questions`.
Stage one (the shared `truepeak-core` crate:
 meter,
 gain,
 source trait,
 window math,
 policy
identity) is implemented,
 tested,
 and committed.
Stage two (the `truepeak-core.bench` evidence engine on the shared meter) is implemented and
committed,
 and its corpus run is recorded in the Stage-two evidence subsection.
Stages three through seven (the full shared service,
 desktop migration,
 Android migration,
 and
cleanup) are not started.

Audience:
 a reviewer who has never seen this product.
This document explains the product context,
 the problem,
 the agreed design,
 and the evidence that must exist before
implementation is accepted.

## What this document is

The music player has two native flavors:

- A desktop app,
   written in Rust.
- An Android app,
   with Kotlin UI and service code calling a Rust native engine through JNI.

Both flavors normalize tracks so playback does not exceed a true-peak ceiling.
The current code got there through separate experiments,
so the desktop and Android paths now disagree about measurement policy,
 cache shape,
 and cache storage.

This plan replaces those separate paths with one shared Rust true-peak package:

- `package/music-player/truepeak-core`

The package should own the true-peak meter,
 the probe policy,
 gain decisions,
 Turso cache I/O,
 and background warming
orchestration.
Desktop and Android should call the same shared service.
Platform code may still provide decoded audio access at first,
but the longer-term direction is to share more of the decode stack too.

## Product background

The app plays local music files.
A track is decoded into interleaved `f32` PCM samples,
then the output path multiplies every sample by one constant per-track gain.
That gain exists to prevent clipping at the digital-to-analog converter.

The app uses a `-1 dBTP` ceiling.
`dBTP` means decibels true peak:
it accounts for peaks that can appear between stored samples after reconstruction,
not only the largest sample value in the file.

True-peak measurement is expensive because the safe answer normally requires decoding the whole track.
The current library contains thousands of files,
so full-scanning everything is expensive enough that the app needs a selective policy:
scan exactly when needed,
probe when probing is good enough,
and cache the result.

## Current state

### Desktop

The desktop app has a Rust true-peak implementation in:

- `package/music-player/desktop-app/src/truepeak.rs`

It uses a Catmull-Rom inter-sample estimator.
It scans decoded interleaved `f32` samples,
keeps a four-sample window per channel,
checks the quarter,
 half,
 and three-quarter positions between samples,
and records the largest absolute value.

Desktop also already uses Turso for a peak cache:

- `package/music-player/desktop-app/src/peakcache.rs`
- `package/music-player/desktop-app/src/peakcache_service.rs`
- `package/music-player/desktop-app/src/peakcache_handle.rs`

That cache currently stores raw peak rows keyed by an opaque fingerprint.
It does not store a shared true-peak policy version,
and it cannot distinguish an exact full scan from a probe-derived estimate.

Desktop current-track behavior lives mainly in:

- `package/music-player/desktop-app/src/peak_swap.rs`

On a cache miss,
it starts playback with a conservative temporary gain,
measures in a worker thread,
then swaps to the measured gain when it arrives.

Background warming lives in:

- `package/music-player/desktop-app/src/measure.rs`

It scans uncached queue tracks in the background.

### Android

Android has a Rust true-peak implementation in:

- `package/music-player/android-app/rust/src/truepeak.rs`

It shares the Catmull-Rom meter shape,
but it also contains an Android-specific long-track window policy:

- four windows,
- fifteen seconds per window,
- a `1.26` linear safety factor,
- full scan below ninety seconds.

That policy came from an older hot-or-no-gain optimization.
It is not the policy this plan should ship.

Android stores peaks through Kotlin JSON code:

- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PeakCacheStore.kt`
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/PeakCache.kt`

Android gain math also still has Kotlin helpers in:

- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/Normalization.kt`

Current Android playback asks Kotlin to resolve a normalization gain in:

- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/RustEngine.kt`

Android measurement entry points call native Rust through:

- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PeakMeasurer.kt`
- `package/music-player/android-app/rust/src/lib.rs`

The new design moves cache lookup,
 true-peak policy,
 and gain-decision ownership into shared Rust.
Kotlin should no longer decide how a peak maps to a cacheable policy result.

## The problem

The current arrangement has several failure modes.

First,
there are separate true-peak algorithms.
Desktop full-scans.
Android may window-scan with an old safety factor.
A track measured on one flavor is not necessarily measured under the same policy as the other flavor.

Second,
the cache shape is wrong for the new policy.
A raw peak cache cannot represent:

- an exact full-scan result,
- a probe-derived approximate result,
- which true-peak policy produced the result,
- whether a later policy version can safely reuse it.

Third,
Android is not on Turso.
Desktop has a Turso actor,
but Android still has JSON cache storage.
The goal is not two cache implementations that happen to use the same database brand.
The goal is one shared cache implementation owned by the new true-peak package.

Fourth,
the handover target had a stale number.
The old follow-up target used `439889.5 / 2 = 219944.75` seconds.
The corrected target is the current decodable library total divided by four.

The measured current decodable total is:

```text
887897.8663151221 seconds
```

So the corrected benchmark target is:

```text
887897.8663151221 / 4 = 221974.46657878053 seconds
```

The old `219944.75` target is now only historical evidence.
It is not the target for the final plan.

## Agreed design decisions

These decisions came from the grilling session before this document was written.

### Optimize for comprehensiveness

The plan should be understandable to a reviewer who does not know the product.
It should explain product behavior,
 current code shape,
 target behavior,
 migration shape,
 evidence,
 risks,
 and open
review questions.

### The corrected target is a benchmark target

`total decodable library seconds / 4` is a benchmark acceptance target for the current library.
It is not a runtime invariant.

Runtime cannot know the denominator without measuring too much of the library first.
The shipped policy should be fixed and versioned.
The benchmark harness verifies that this fixed policy lands at the desired quarter-library budget on the current
measured corpus.

### Use decodable total only

The denominator is the total duration of tracks the current decoder can measure.
The known unsupported HE-AAC/SBR decode failure stays outside the target until decoder support changes.

The current measurement summary was:

- `3992` audio-extension files found.
- `3991` files measured.
- `1` decode failure.
- `887897.8663151221` decodable seconds.

If decoder support changes,
the benchmark denominator changes and the policy needs a fresh verification pass.

### Ship one versioned active policy

The new package should expose one active policy,
with a stable policy ID or policy version.

The app should not expose multiple normalization profiles to users.
Policy changes are code changes,
and they must bump the policy identity so stale cache rows are ignored.

### Fit the current corpus exactly

The classifier acceptance goal is exact fit on the current corpus:

- It uses only duration and probe-derived measurements at runtime.
- It catches every current-library track whose probe-derived gain would violate the gain-error bounds.
- It keeps total decoded seconds at or below the corrected quarter target.
- It does not use full-track truth,
   paths,
   or hard-coded exception lists at runtime.

The corpus is considered large and varied enough for this product's current acceptance target.

Amended 2026-07-01,
 after the Stage-two evidence (see the fine-bin collect pass finding under
 `Stage two evidence and the feasible no-classifier finding`):
strict exact fit is disproportionately expensive at the quarter budget,
 because a handful of
tracks hide a sub-tenth-second transient that no sparse probe catches,
 and covering them forces
about a decibel of extra too-quiet margin on every track.
The accepted goal is therefore exact fit for about ninety-nine percent of tracks under a fixed
`0.8 dB` margin,
 with the realtime clamp catching the rare too-loud transient and background
warming full-scanning those tracks to an exact cached gain over time.
There is no probe-feature or metadata classifier that routes the hard tracks,
 so the policy is
classifier-free:
 a proportional probe plus the fixed margin.

### Do not use an opaque model

The classifier may be whatever works best short of an opaque model.
A small auditable decision tree is preferred,
but a generated non-opaque rule set is acceptable if it is still reviewable and tests prove it depends only on probe
features.

Allowed runtime features include:

- track duration,
- sampled maximum peak,
- sampled minimum peak,
- spread between sampled windows,
- per-window peak summaries if the production probe records them,
- other values derived only from the probe windows and audio metadata.

Disallowed runtime features include:

- file paths,
- artist names,
- benchmark exception lists,
- full-track true peak before classification,
- any field that is just a disguised path list.

### Keep the quarter-target error bounds

For the corrected quarter-library target,
keep the existing quarter-flavor gain-error envelope:

```text
+0.5 dB too loud
-2.0 dB too quiet
```

Positive error means playback is too loud compared with exact full-scan normalization.
Negative error means playback is too quiet.

If future tuning ever needs looser bounds,
loosen the too-quiet side before loosening the too-loud side.

### Reset legacy caches

Do not import old desktop or Android peak caches.

This is intentionally conservative.
Desktop raw peaks are closer to reusable because they are exact full-scan peaks,
but Android JSON entries may contain old windowed values with the `1.26` safety factor.
The new cache schema must start clean rather than risk treating old values as compatible shared-policy decisions.

### The shared package owns Turso I/O

The shared true-peak crate should own the Turso service,
schema,
reads,
writes,
and policy-version matching.

Desktop should delete its current `peakcache_*` actor in favor of the shared service.
Android should stop using Kotlin JSON peak storage.

### Android uses an explicit native service handle

Android Kotlin should create a native true-peak service handle,
passing an app-private database path.
Kotlin then passes that handle to native decision calls and releases it when done.

This avoids reopening Turso for every track,
and it avoids a hidden process-global singleton.

### Keep conservative playback fallback

For an uncached current track,
playback should keep the current conservative temporary gain while the shared policy measures:

```text
normalizationGain(1.0)
```

That is the `-1 dBTP` ceiling as a linear gain.
It is slightly quiet for some tracks,
but it preserves non-blocking playback and is safer than unity gain during cold measurement.

### Shared warming engine with platform knobs

Background warming should use the same shared policy as current-track playback.
It should not be a second normalization algorithm.

The shared crate should expose a warming engine,
but platform code should still control platform-specific knobs:

- concurrency limits,
- thread priority or scheduler class,
- cancellation or lifecycle integration,
- Android WorkManager or service boundaries,
- desktop worker-thread details.

### Stage decoder sharing

The endgame is to share almost everything,
including more of the decode stack.

Do not make the first true-peak extraction depend on moving all decoding at once.
Stage the work:

- First extract true-peak policy,
   metering,
   cache I/O,
   decisions,
   and warming orchestration.
- Keep small platform adapters that open existing decoded audio sources.
- Later move shared decode access into a shared crate once the true-peak service is stable.

## Target math and stale evidence

The corrected target is:

```text
full_decodable_seconds = 887897.8663151221
target_seconds = full_decodable_seconds / 4
target_seconds = 221974.46657878053
```

A useful starting threshold can be computed by solving:

```text
sum(min(track_duration, threshold_seconds)) = target_seconds
```

Using the existing `tracks.jsonl` measurement,
that gives:

```text
threshold_seconds = 56.10006235347777
window_count = 14
window_seconds = 4.007147310962698
short_full_scan_count = 89
long_probe_candidate_count = 3902
```

This is only a starting point.
It assumes no long-track exceptions.
Once the classifier sends some long tracks to full scan,
window seconds,
 classifier thresholds,
 or both may need retuning so the final decoded seconds stay at or below the
corrected target.

The old lower-target candidate remains useful as historical evidence,
but it is not the final target:

```text
old_target_seconds = 219944.75
window_count = 14
window_seconds = 3.754228571428571
threshold_seconds = 52.5592
probe_margin_db = 0.683
decoded_seconds = 219943.7791356195
short_full_scan_count = 82
probably_no_full_scan_count = 3860
needs_full_scan_exception_count = 49
worst_too_loud_db = 0.4991655817434766
worst_too_quiet_db = -0.6830000000000007
```

Against the corrected target,
that old candidate is under budget by:

```text
2030.687443161034 seconds
```

That means it is a conservative lower starting point,
not a finished quarter-target policy.

## True-peak policy behavior

The shared policy has three outcomes for a track.

### Short enough for exact normalization

If:

```text
duration_seconds <= window_seconds * window_count
```

then the policy scans the full track and computes exact gain from the full true peak.

### Probably does not need full scan

For a longer track,
the policy samples probe windows.
If probe-only evidence says the probe-derived gain is safe for the active policy,
the policy stores and returns the approximate gain.

### Needs full scan

If probe-only evidence is not safe enough,
the policy scans the full track and stores exact gain.

The policy must never require full-track truth to decide which branch to take.
Full-track truth is allowed only in the benchmark harness,
where it labels training and verification outcomes.

## True-peak meter

The meter should be shared exactly.
Both full scans and window probes must use the same Catmull-Rom implementation.

The meter logic is:

- Read decoded interleaved `f32` samples.
- Maintain a four-sample sliding window per channel.
- Once a channel has four real samples,
  evaluate Catmull-Rom interpolation between the two middle samples.
- Probe positions at one quarter,
  one half,
  and three quarters.
- Track the maximum absolute raw or interpolated sample.

This is the measurement unit that makes full-scan peaks and window-probe peaks comparable.

## Window placement

The policy defines:

```text
threshold_seconds = window_seconds * window_count
window_frames = floor(window_seconds * sample_rate)
```

For long tracks,
place `window_count` windows evenly from the beginning to the final possible window start.
That final-window coverage is important:
a windowing implementation that misses the last possible start position can under-read endings.

Each window should use its own meter instance.
That avoids fabricating an inter-sample spike across a seek seam between unrelated audio regions.

## Gain math

The shared package should own gain math.
Kotlin should no longer carry a separate `normalizationGain` policy for true-peak decisions.

The dB formulas are:

```text
full_peak_dbtp = 20 * log10(full_peak)
sampled_max_dbtp = 20 * log10(sampled_max_peak)
exact_gain_db = min(0, -1 - full_peak_dbtp)
probe_estimated_peak_dbtp = sampled_max_dbtp + probe_margin_db
probe_gain_db = min(0, -1 - probe_estimated_peak_dbtp)
error_db = probe_gain_db - exact_gain_db
```

The runtime returns a linear gain scalar to playback.
That scalar must never exceed `1.0`.
The policy never amplifies.

## Proposed package layout

The first implementation stage should add these packages:

- `package/music-player/truepeak-core`
- `package/music-player/truepeak-core.bench`
- `package/music-player/truepeak-core.fuzz`

The core crate owns production behavior.
The bench sidecar owns corpus measurement and parameter search.
The fuzz sidecar owns adversarial and randomized checks of window placement,
 meter behavior,
 cache serialization,
 and
classifier invariants.

Each package should have its own `README.md`,
 `Cargo.toml`,
 and `mise.toml`.
The repo rule for package completeness applies:
README,
lint,
and tests are part of done.

## Shared core surface

The core crate should define a small decoded-audio abstraction.
The current desktop and Android Rust `Source` traits are already close enough for adapters:

```rust
//! The source contract of the shared true-peak core:
//! a decoded stream that reports its format,
//! yields chunks of samples and can seek.

/// A decoded audio stream that the true-peak analysis reads,
/// owned by the caller and movable between threads.
pub trait TruePeakSource: Send {
    /// The sample format of the stream.
    fn spec(&self) -> AudioSpec;
    /// Decodes and returns the next chunk of interleaved samples,
    /// or the analysis error that stopped the decoder.
    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError>;
    /// Moves the read position to the given time in seconds.
    fn seek(&mut self, seconds: f64) -> Result<(), TruePeakError>;
}
```

The production API should not expose only raw peak measurement.
It should expose gain decisions.

Conceptually,
the main service API is:

```rust
//! The service calls of the shared true-peak core,
//! as signatures only:
//! open the database,
//! read a cached decision,
//! resolve a decision from a source,
//! and warm the library in bulk.
TruePeakService::open(database_path, active_policy)
TruePeakService::cached_decision(fingerprint)
TruePeakService::resolve_decision(fingerprint, source)
TruePeakService::warm_library(track_descriptors, opener, warming_options)
```

Exact function names can change during implementation,
but the ownership should not:

- The shared service owns Turso.
- The shared service owns policy version matching.
- The shared service owns meter and classifier behavior.
- Platform adapters provide decoded audio sources until decoder sharing is staged in.

## Turso schema

The new cache stores decision rows,
not raw peak rows.

Primary key:

- `fingerprint`
- `policy_id`

Required row fields:

- source fingerprint,
- active policy ID,
- decision kind,
- final linear gain,
- duration seconds,
- whether the decision is exact or probe-derived,
- exact peak when a full scan happened,
- sampled maximum peak when probing happened,
- sampled minimum peak when probing happened,
- window count,
- window seconds,
- probe margin dB,
- measured timestamp or monotonic schema metadata useful for debugging.

Decision kinds should be explicit values,
for example:

- `exact_full_scan_short`
- `exact_full_scan_classifier`
- `probe_estimate`

The row shape must let a future reader answer:

- Was this gain exact or approximate?
- Which policy produced it?
- Which probe constants produced it?
- Can this row be reused by the current policy?

## Cache semantics

A cache hit is valid only when both the source fingerprint and policy ID match.

A source fingerprint changes when the file changes.
A policy ID changes when constants,
 classifier logic,
 gain math,
 or cache interpretation changes.

Old rows may remain in Turso.
They should not be read by a newer policy.
A cleanup task can delete old-policy rows later,
but cleanup is not part of the correctness path.

Legacy desktop and Android caches should not be imported.
The first shared-policy run starts from a cold shared cache.

## Desktop migration plan

Desktop should stop using its local peak-cache actor.
Remove or retire:

- `package/music-player/desktop-app/src/peakcache_service.rs`
- `package/music-player/desktop-app/src/peakcache_handle.rs`
- raw-peak-only logic in `package/music-player/desktop-app/src/peakcache.rs`

Replace them with the shared true-peak service handle.

The current-track flow in `peak_swap.rs` should become:

- Ask the shared service for a cached decision by fingerprint and policy ID.
- If present,
  apply the cached final gain immediately.
- If absent,
  start a shared measurement decision in a worker.
- Use conservative fallback gain while waiting.
- When the shared decision arrives,
  apply its final gain and cache row.

The background sweep in `measure.rs` should become a call into the shared warming engine.
Desktop can still supply worker priority hooks so background decode runs politely.

Desktop playback should continue applying one constant gain scalar per track.
No UI change is required.

## Android migration plan

Android should replace Kotlin peak-cache and normalization orchestration with native shared-service calls.

Kotlin should create an explicit native true-peak service handle:

```text
nativeTruePeakServiceCreate(databasePath) -> handle
nativeTruePeakServiceRelease(handle)
```

Kotlin decision calls should pass that handle.
The exact JNI surface can be refined,
but the important change is that native Rust returns a gain decision,
not only a raw peak.

Current Kotlin pieces to retire or narrow:

- `PeakCacheStore.kt`,
   because Turso cache I/O moves to shared Rust.
- `core/PeakCache.kt`,
   because the in-memory JSON-backed cache goes away.
- true-peak decision use of `core/Normalization.kt`,
   because gain decisions move to shared Rust.

Kotlin may keep sample clamp helpers if they are still useful as output-stage test or UI references,
but they must not define the true-peak policy.

The native Rust crate should depend on `package/music-player/truepeak-core` by path.
It should expose JNI functions that call the shared service.

The Android native build must prove Turso works for both native ABIs:

- `arm64-v8a`,
- `x86_64`.

An isolated spike has already cleared most of this;
 see the Turso on Android risk note.
`turso` `0.6` cross-compiles for both ABIs through `cargo-ndk`,
and the `arm64-v8a` build round-trips an on-disk decision row on a physical device.
The in-process path is also proven:
a spike JNI entry linked `turso` into `libmusicplayer_native.so` and ran an instrumented round-trip
to the app-private `filesDir` on both the Pixel 6 (`arm64-v8a`) and the emulator (`x86_64`).
The Android package already builds native Rust through `cargo-ndk`,
so the verification belongs in the Android native build task and in the new core package tasks.

## Background warming plan

The shared warming engine should accept a list of track descriptors and an opener callback.
A descriptor needs at least:

- fingerprint material or a ready fingerprint,
- enough platform data to open a fresh decoded source,
- duration metadata if cheaply available,
- display path only for logging or benchmark reporting,
  not for classification.

The shared engine should:

- skip cache hits for the active policy,
- probe or full-scan cache misses using the same policy as current playback,
- store decision rows through the shared Turso service,
- expose progress and error hooks for platform logs,
- allow platform-specific concurrency and priority settings.

Desktop can keep its idle thread priority behavior.
Android can keep WorkManager or service scheduling behavior.
The policy and cache writes must still go through the shared core.

## Classifier search plan

The corrected target requires a new parameter search.
The old `14 x 3.754228571428571` result is below the corrected target and should not be copied into production as-is.

The search should work like this:

1.  Measure full-library truth with the shared Catmull-Rom meter.
2.  Generate exact window measurements for candidate window counts and seconds.
3.  Use full truth only in the harness to label tracks whose probe gain violates `+0.5 / -2.0 dB`.
4.  Search for a non-opaque classifier that exactly catches those labels from probe-only features.
5.  Retune window seconds and margin until decoded seconds are at or below `221974.46657878053`.
6.  Verify every decodable track against full truth.
7.  Commit the harness and summarized evidence.

The classifier should prefer simple threshold rules when possible.
If a generated rule list is necessary,
it must remain auditable:

- It must name every feature it reads.
- It must be generated from probe metrics only.
- Tests must fail if path text or full peak enters the production classifier input.
- The generated artifact must be small enough for review.

Search objective (decided 2026-06-25):
among policies that satisfy the hard constraints,
catching every violator,
staying under the decoded-seconds budget,
and keeping every track within `+0.5 / -2.0 dB`,
the search minimizes worst-case too-quiet error first,
then prefers the simplest classifier as the tie-break.
Spare budget is spent pulling tracks up toward the ceiling,
because the too-quiet side is the only audible cost inside the bounds.

### Stage two evidence and the feasible no-classifier finding

The Stage-two bench sidecar (`package/music-player/truepeak-core.bench`) was built on the
shared `truepeak-core` meter,
 gain,
 window-placement,
 and policy math,
 and run against the
existing per-track corpus (full peak plus per-second bin peaks,
 whose meter is identical to
`truepeak-core`'s,
 so the bins are valid for the shared meter).
That run changes the classifier approach above,
 so the steps stay but their conclusion moves.

The bench's own copies of the probe selection,
 the neighbour climb and the even pass,
 differed from the production probe in two places:
 a budget the copies left uncapped at the bin count,
 which no run could notice because the climb stops when the heap empties with every bin
 measured,
 and a bones slot at or past the last bin,
 where the bucket copy measured only that bin and playback measures its neighbour too.
The figures below come from the bones-free zoom path,
 whose coverage share never exceeds one,
 so they measured the bins playback measures;
 a bones-seeded bucket run could have differed on that last seed alone.
Since issue 1461 both paths call `select_probe_bins` from `truepeak-core`,
 so no run can differ.

The corpus and target:

```text
tracks measured        = 3991
full decodable seconds = 887897.8663151221
corrected target (/4)  = 221974.46657878053
```

First finding:
 a full-scan violator classifier is not feasible at this budget.
The oracle sweep (a perfect classifier that full-scans exactly the violators) lands inside
the target only because it routes nothing extra;
 its headroom is about `500` seconds,
 roughly
two extra full scans.
Any real probe-feature classifier over-routes far past that.
The violators are hot masters whose loudest sampled window overlaps the non-violators in
amplitude,
 so no loudest-window threshold separates them:
 routing every loud-enough track
costs over `780000` decoded seconds,
 more than three times the budget.

Second finding (the metadata signal):
 provenance metadata is a real but partial separator.
A yt-dlp / youtube provenance tag appears in `0` of the `123` violators and about `21%` of the
non-violators,
 and every lossless (FLAC) track is a non-violator,
 so a "lossless or yt-dlp"
marker never mislabels a violator as safe.
But the safe classes cover only part of the library;
 the remaining untagged lossy tracks hold
all the violators,
 and metadata improves the average error,
 not the worst case (the safe group
still has a worst under-read near `1.97 dB`).
Provenance metadata is a legal classifier feature (it is encoding and origin,
 not a path or
artist),
 and it is worth using to lower the average error,
 but it does not rescue the
full-scan approach.

Third finding (the feasible model):
 drop the violator classifier entirely.
Probe every long track,
 and apply one fixed margin large enough to cover the worst under-read,
which guarantees zero violators with no full scans.
The objective then becomes spending the budget on probe density:
 more,
 shorter windows cover
more distinct regions and shrink the gaps that cause under-read,
 so the required margin,
 and
thus the worst-case too-quiet error,
 falls.
Measured on the corpus (windows at or above one second,
 the reliable resolution of the current
one-second bins):

```text
count=28, window~1.63s : margin 1.68 dB, worst-quiet -1.68 dB, decoded ~181000s (well under target)
count=56, window~0.93s : margin 1.61 dB, worst-quiet -1.61 dB, decoded ~206000s (under target)
```

Sub-second windows (for example `count=80`,
 `window~0.69s`) report a margin near `1.23 dB`,
 but a
one-second bin overestimates a sub-second window,
 so that figure is optimistic until the
collector emits finer bins.
This model satisfies every agreed design decision:
 zero violations inside `+0.5 / -2.0 dB`,
decoded seconds under the corrected target,
 no opaque model (the simplest possible classifier
is a single fixed margin),
 and no path or full-truth input at runtime.
The decided objective stands;
 the `-1.6 dB` worst-too-quiet is simply the honest achievable
floor once the infeasible perfect-classifier assumption is removed,
 and probe density (plus a
provenance-dependent margin for the average) is the lever that lowers it.

Fourth finding (why not a cheap probe plus a full-scan router):
spending less on the probe and full-scanning the residual has a tempting oracle floor.
A cheap twenty-second spread probe plus full-scanning exactly the tracks whose probe gain
would violate reaches a worst-too-quiet near `-0.15 dB` and stays under budget,
 because the
cheap probe frees roughly `140000` seconds for full scans.
That floor is not reachable,
 because the router cannot be built from observable features.
Some violators probe quiet (a loudest sampled window near `-4.5 dBTP`) yet hide a sharp
transient in an undecoded gap whose true level is near `-0.5 dBTP`,
 a four-decibel under-read.
No loudest-window threshold isolates them (routing to catch the quietest one routes nearly
everything),
 and they are not a provenance class.
A peak that lives in a gap the probe never decodes is invisible by definition,
 so guaranteed
exact fit needs either a margin that covers the worst gap-miss or a probe dense enough that no
gap can hide a damaging transient.
Dense probing reduces that worst gap-miss (it fell from over four decibels at twenty seconds to
about `2.18 dB` at the forty-five-second density),
 which is why density,
 not a router,
 is the
lever,
 and why finer bins are the next step.

The provenance-dependent margin is wired into the bench.
At the forty-five-second density it gives the roughly `1250` safe-provenance tracks a `1.47 dB`
margin and the rest `1.64 dB`,
 so it lowers the average too-quiet error while the worst case
stays at the untagged group's `1.64 dB`.

Fifth finding (the fine-bin collect pass,
 and why the worst case is irreducible by density):
the shared-meter `collect` pass (`truepeak-core-collect`,
 decoding through the production meter
via an ffmpeg pipe) regenerated the corpus at one-tenth-second bins.
It matches the prior corpus to `0.0000 dB`,
 and ffmpeg decoded the formerly-unsupported HE-AAC
track plus more files,
 so the current corpus is:

```text
tracks                 = 4114
full decodable seconds = 916861
corrected target (/4)  = 229215
```

Finer window granularity does NOT lower the margin.
At `450` windows of one-tenth second (a forty-five-second probe) the margin is `1.78 dB`,
 no
better than the coarse result,
 because the worst track is a LONG one:
 a forty-five-second probe
is about one percent of an hour-long mix,
 so a transient in the unsampled remainder is missed
regardless of how the probe seconds are sliced.
The limit is coverage FRACTION,
 not window granularity.
Switching to proportional coverage (probe a fixed fraction of every long track) at the budget
fraction of about twenty percent still floors near `1.75 dB`,
 because a handful of tracks carry a
single sharp sub-tenth-second transient that no sparse probe catches at any granularity or any
coverage fraction up to thirty percent.

But the under-read distribution is extremely skewed,
 which is the practical opening.
At proportional twenty-percent coverage the loud long tracks distribute as:

```text
under-read dB: median 0.14  p90 0.63  p95 0.84  p99 1.33  p99.5 1.56  max 2.25
```

So almost every track is easy;
 a long tail of a few sharp-transient tracks sets the worst case.
Guaranteeing the last one percent costs about a decibel of margin on every track.
The margin-versus-clamp tradeoff makes that cost optional:

```text
margin 0.5 dB -> worst-quiet -0.5, 119 tracks (3.17%) exceed +0.5 and are clamped
margin 0.8 dB -> worst-quiet -0.8,  43 tracks (1.14%) clamped
margin 1.0 dB -> worst-quiet -1.0,  26 tracks (0.69%) clamped
margin 1.2 dB -> worst-quiet -1.2,   7 tracks (0.19%) clamped
```

The realtime per-sample clamp already catches any too-loud transient (no converter overflow,
only brief distortion on that sub-second peak),
 and background warming eventually full-scans
every track to an exact cached gain,
 so a clamped transient is a cold-start-only effect on a
handful of tracks,
 not a permanent error.
This reframes the policy:
 "exact fit at the quarter budget" is disproportionately expensive in
the last percent,
 so the recommended shape is a proportional probe with a moderate fixed margin
(the margin sets the clamp count,
 a design choice),
 the album and provenance priors layered on
to make the average gain louder than that margin,
 the realtime clamp as the safety net,
 and
background warming converging the cache to exact over time.

Stage-two decisions (2026-07-01):

- The proportional-coverage probe and the margin-versus-clamp tradeoff are wired into the bench
  as the committed `--proportional` evaluation,
   which reads the shipped
  `truepeak_core::default_policy` and reproduces the numbers in the
  `Stage two evidence and the feasible no-classifier finding` section on the fine corpus.
- The shipped margin is `0.8 dB`,
   decided with the requester:
   about ninety-nine percent of
  tracks stay within `-0.8 dB` too-quiet,
   and about one percent (forty-three tracks,
   none of
  them safe-provenance) clamp on cold start until background warming full-scans them to exact.
- The shipped policy now lives in `truepeak-core`'s `default_policy`:
   a ninety-second full-scan
  cutoff for short tracks,
   one-fifth proportional coverage in tenth-of-a-second windows for long
  tracks,
   and the `0.8 dB` margin.
   The old classifier-era window-count parameters are gone.

Remaining Stage-two work:

- Verify the chosen coverage and margin against exact decoded windows,
   not only the bins.
- Layer the album prior (scan a few members per album,
   infer the rest) to lower the average
  error further;
   the worst case and the clamp count are already handled by the margin above.

## Bench sidecar

`package/music-player/truepeak-core.bench` should own corpus-scale measurement and parameter search.

It should be able to:

- scan a music library root,
- decode tracks through the same meter,
- emit `tracks.jsonl`,
- emit exact window probe files,
- evaluate candidate policies,
- report decoded seconds,
- report gain-error bounds,
- report exception paths,
- report whether classifier inputs are production-legal.

The sidecar should not require committing the user's music.
It may write artifacts under a local ignored output directory.
The planning and verification docs may include full paths because that was explicitly accepted for review.

## Fuzz sidecar

`package/music-player/truepeak-core.fuzz` should test invariants that do not need the real library.

Useful fuzz targets include:

- Catmull-Rom meter never panics for arbitrary channel counts and sample chunks.
- Zero-channel input is handled as silence or rejected predictably.
- Window placement never seeks before start or beyond the final legal window.
- Window placement includes the beginning and the final possible start.
- Gain is never greater than unity.
- Cache row serialization and deserialization round-trip policy IDs and decision kinds.
- Classifier input type cannot carry path strings or full-track truth.

Fuzzing is not a replacement for the corpus verifier.
It complements it.

## Required verification before implementation is accepted

The implementation is not done until all these layers pass.

### Core unit tests

Cover:

- Catmull-Rom control-point behavior.
- Inter-sample peak detection on synthetic samples.
- Silence and zero-channel behavior.
- Exact gain math.
- Probe-derived gain math.
- No amplification above unity.
- Short-track full-scan classification.
- Long-track window placement.
- Final-window coverage.

### Core integration tests

Use fake decoded sources and a throwaway Turso database.

Cover:

- Cache miss to exact decision row.
- Cache miss to probe decision row.
- Cache hit with matching policy.
- Cache miss with mismatched policy.
- Source fingerprint mismatch.
- Decision kind persisted and read back.
- Background warming skips active-policy hits.
- Background warming writes the same decision current playback would write.

### Desktop integration tests

Cover:

- Current-track cache hit applies gain immediately.
- Current-track cache miss starts with conservative fallback.
- Late measurement swaps to final shared decision gain.
- Background warming uses shared service.
- Legacy desktop raw-peak cache is not read as a shared-policy hit.

### Android host and device tests

Cover:

- Native service handle create and release.
- Native decision API returns a valid gain decision for a fixture.
- Turso database path lives under app-private storage.
- Kotlin no longer reads or writes `peaks.json` for true-peak policy.
- Device build includes the shared true-peak crate for both native ABIs.
- Instrumented synthetic true-peak tests still exercise the real shared meter.

### Corpus verifier

Run against the current decodable library.

Acceptance:

- Target denominator is `887897.8663151221` decodable seconds unless the library or decoder support changed.
- Target budget is `221974.46657878053` decoded seconds.
- Gain-error bounds are `+0.5 dB` too loud and `-2.0 dB` too quiet.
- Production classifier uses no paths and no full-track truth.
- Every decodable track stays inside the gain-error bounds.
- Decoded seconds are at or below the target budget.
- The report lists full exception paths for review.

## Historical lower-target exception paths

The following paths are from the old lower-target run.
They are included as evidence of the prior search,
not as a production exception list and not as the final corrected-target result.
Some very long paths are wrapped across physical lines to keep this document readable.

```text
/home/user/Seafile/Plain/Music/3LAU/3LAU - Star Crossed.opus
/home/user/Seafile/Plain/Music/Afrojack/Afrojack Martin Garrix - Turn Up The Speakers (Original Mix).opus
/home/user/Seafile/Plain/Music/Alesso/Alesso Ryan Tedder - Scars.opus
/home/user/Seafile/Plain/Music/Alesso/Falling.opus
/home/user/Seafile/Plain/Music/Avicii/Avicii - Levels (Radio Edit).opus
/home/user/Seafile/Plain/Music/Avicii/Avicii Audra Mae - Long Road to Hell.opus
/home/user/Seafile/Plain/Music/Bassjackers/Bassjackers & KSHMR feat. Sidnie Tipton - Extreme.opus
/home/user/Seafile/Plain/Music/Boyhood's end OST/The Old Days.opus
/home/user/Seafile/Plain/Music/Calvin Harris/Calvin Harris - How Deep Is Your Love (Radio Edit).opus
/home/user/Seafile/Plain/Music/Coldplay/Coldplay - Up&Up.opus
/home/user/Seafile/Plain/Music/Coldplay/Parachutes/09 We Never Change.opus
/home/user/Seafile/Plain/Music/David Guetta/David Guetta Sia - Titanium (Alesso Remix) - remix.opus
/home/user/Seafile/Plain/Music/Don Diablo/Don Diablo BullySongs - Found You.opus
/home/user/Seafile/Plain/Music/Don Diablo/Don Diablo Holly Winter - Don't Let Go.opus
/home/user/Seafile/Plain/Music/Ed Sheeran/Ed Sheeran - Shape of You (Extended Mix).opus
/home/user/Seafile/Plain/Music/Fun. Janelle Monáe - We Are Young.opus
/home/user/Seafile/Plain/Music/Gloriana - (Kissed You) Good Night.opus
/home/user/Seafile/Plain/Music/Halsey/Halsey - 01. So Good.opus
/home/user/Seafile/Plain/Music/Halsey/Manic (Explicit)/Halsey - 06. Dominic's Interlude.opus
/home/user/Seafile/Plain/Music/Halsey/Manic (Explicit)/Halsey - 08. 3am (Explicit).opus
/home/user/Seafile/Plain/Music/Hardwell/Being Alive feat_JGUAR Extended Mix.opus
/home/user/Seafile/Plain/Music/Hardwell/Hardwell - 2017 End of the Year Mix.opus
/home/user/Seafile/Plain/Music/Hardwell/Hardwell feat. Jake Reese - Run Wild (extended mix).opus
/home/user/Seafile/Plain/Music/Imagine Dragons/Imagine Dragons - 02. Boomerang.opus
/home/user/Seafile/Plain/Music/Imagine Dragons/Night Visions (Deluxe)/Imagine Dragons - 09. Bleeding Out.opus
/home/user/Seafile/Plain/Music/KLYMVX/KLYMVX - Lean On (KLYMVX Remix).opus
/home/user/Seafile/Plain/Music/Kygo/Kygo & St. Lundi - To Die For.opus
/home/user/Seafile/Plain/Music/Kygo/Kygo & Zac Brown - Someday.opus
/home/user/Seafile/Plain/Music/Kygo/Kygo, Patrick Droney & Petey Martin - Say You Will.opus
/home/user/Seafile/Plain/Music/Kygo/Kygo, Zara Larsson & Tyga - Like It Is.opus
/home/user/Seafile/Plain/Music/Lost Frequencies/Are You With Me/Are You With Me (Dimaro radio edit).opus
/home/user/Seafile/Plain/Music/Magnet マグネット (Magnet) - Hatsune Miku Megurine Luka 初音ミク
(Hatsune Miku) 巡音ルカ (Megurine Luka) DIVA English lyrics romaji subtitles.opus
/home/user/Seafile/Plain/Music/Maksim Mrvica - Somewhere in Time (Barry).opus
/home/user/Seafile/Plain/Music/Martin Garrix/Midnight Sun Extended Version.opus
/home/user/Seafile/Plain/Music/Medicine イガク (Igaku - Medicine) - 重音テト (Kasane Teto).opus
/home/user/Seafile/Plain/Music/ONE OK ROCK/Ambitions/ONE OK ROCK - Bombs away.opus
/home/user/Seafile/Plain/Music/ONE OK ROCK/Kanjo Effect/Break My Strings.opus
/home/user/Seafile/Plain/Music/ONE OK ROCK/ONE OK ROCK (ワンオクロック) Avril Lavigne - Listen (Japanee Verion).opus
/home/user/Seafile/Plain/Music/Retrograde Extended Mix.opus
/home/user/Seafile/Plain/Music/Rihanna Eminem - Love The Way You Lie (Part II).opus
/home/user/Seafile/Plain/Music/Riot Extended Mix.opus
/home/user/Seafile/Plain/Music/Selena Gomez/Selena Gomez - Back to You.opus
/home/user/Seafile/Plain/Music/Tetoris テトリス (Tetoris) - 重音テトSV (Kasane Teto SV).opus
/home/user/Seafile/Plain/Music/Tsukihime/Tsukihime - Natsu no Ao (Summer Blue).opus
/home/user/Seafile/Plain/Music/Zedd/Zedd Logic X Ambassadors - Transmission.opus
/home/user/Seafile/Plain/Music/secret - Megurine Luka - sm11401862.opus
/home/user/Seafile/Plain/Music/【なつうた (Natsu Uta - Summer Song) 2015】Meltdown
ENGLISH_Piano ver. を 歌ってみた (Cover)【Lizz Robinett】.opus
/home/user/Seafile/Plain/Music/李克勤 (Hacken Lee) - 红日 (Red Sun) (粤语).opus
/home/user/Seafile/Plain/Music/水木年华 (Shui Mu Nian Hua) 老狼 (Lao Lang) 李健 (Li Jian)
叶世荣 (Ye Shirong) - 青春再见 (Goodbye Youth).opus
```

## What not to implement

Do not reintroduce the old Android hot-or-no-gain contract.
Probe-derived approximate gain is allowed for tracks classified as safe.

Do not use the Android `1.26` window safety factor as the shared policy.

Do not keep separate desktop and Android true-peak constants.

Do not hard-code path exceptions into production.
The exception paths may appear in benchmark reports,
but the classifier must use probe-derived features only.

Do not reuse legacy caches as compatible shared-policy decisions.

Do not treat a successful compile as enough verification.
The user boundary is playback gain decisions and warm-cache behavior in both flavors.

## Risks and review questions

### Turso on Android

Status:
 fully de-risked by an isolated spike on 2026-06-25.

The crate desktop depends on is `turso` `0.6.1`,
the pure-Rust SQLite rewrite (`turso_core`,
 `turso_parser`,
 `turso_macros`),
not the libsql C fork.
There is no bundled C SQLite to cross-compile,
which is the usual source of Android native-database pain.

A throwaway spike confirmed the build and runtime path.
It changed no production code and ran outside the repo.
The spike used the desktop pin,
`turso` `0.6` with `default-features = false`.
It cross-compiled for both named ABIs through `cargo-ndk`:
`aarch64-linux-android` and `x86_64-linux-android`.
The `aarch64` binary ran on a physical Pixel 6,
arm64-v8a,
 API 36.
On that device it opened an on-disk database,
created a composite-key `(fingerprint, policy_id)` decision table,
wrote a `probe_estimate` row with a `REAL` gain,
read the row back unchanged,
and returned a clean miss for an absent policy key.
The database file persisted:
a second process reopened it and read the same row.

The in-process path is also proven.
A spike JNI entry linked `turso` into the real `libmusicplayer_native.so` for both ABIs,
and an instrumented `NativeBridgeTest` ran a round-trip to the app-private `filesDir`:

- `arm64-v8a` on the physical Pixel 6:
   `OK (10 tests)`.
- `x86_64` on the emulator:
   `OK (10 tests)`.

So Turso is confirmed end to end on Android:
it cross-compiles for both ABIs,
links into the app library,
and round-trips a decision row to an app-private path under the JVM-loaded library,
on both a real device and an emulator.
No residual Turso-on-Android risk remains.
The spike code was a throwaway in a forked worktree and is not part of the plan.

### Exact classifier fit

The plan deliberately asks for exact fit on the current corpus without an opaque model.
That may require several search iterations.
The benchmark harness is part of the plan because this is not a hand-tuned constant change.

### Source reopening and seeks

The shared service needs to probe windows and sometimes full-scan the same track.
The first implementation can rely on the existing seekable decoded sources,
but tests should cover sources that seek repeatedly and then scan from the beginning.

If any real decoder cannot seek accurately enough for window probing,
the plan needs a fallback:
full scan that track or reopen the source.

### Cache semantics after policy changes

Composite cache keys protect correctness,
but old rows can accumulate.
A cleanup plan can come later.
Correctness must not depend on cleanup.

### Future decoder sharing

This plan stages decoder sharing after true-peak core extraction.
A reviewer should check that the first-stage adapter boundary does not make later decoder sharing harder.

### Conservative fallback and hot masters

The fallback gain is `normalization_gain(1.0)`,
which is `-1 dB` of attenuation (linear `0.8912509`),
confirmed in `package/music-player/desktop-app/src/peak_swap.rs`.
That gain is correct only for a track whose true peak is `0 dBTP`.
A hot master with inter-sample peaks above `+1 dBTP` (linear above `1.122`)
still sits above `0 dBFS` after the fallback gain,
so the realtime per-sample clamp distorts it during the cold-measurement window.
The fallback never overflows the converter,
 because the clamp catches it,
but it is not distortion-free for the loudest masters.
The doc's "slightly quiet for some tracks" describes only one direction.
This is a brief transient until the measured gain swaps in,
so it is a limitation to document,
 not necessarily a blocker.

### Source fingerprint definition

The plan uses `fingerprint` throughout but never defines its inputs.
Desktop currently derives it as a one-way hash of `(path, size, mtime)`
in `package/music-player/desktop-app/src/peakcache.rs`.
The shared package should state the exact fingerprint inputs,
because the choice has correctness consequences:

- A content edit that preserves `mtime` would not invalidate a `(path, size, mtime)` row.
- Including `path` means a renamed or moved file re-scans even when its bytes are unchanged.
- Desktop and Android use different path roots for the same logical track,
  so a path-based fingerprint never matches across flavors.

### Concurrent writes and decision precedence

Current-track measurement and background warming can both resolve the same track.
The plan should define the write model,
a single-writer actor or a serialized handle,
plus upsert precedence when two decisions race.
An `exact_full_scan` row must not be overwritten by a later `probe_estimate` for the same key,
because the exact decision is strictly better evidence.

### Meter truth is Catmull-Rom, not ITU true peak

The plan calls a full scan "exact gain from the full true peak"
and uses "full-track truth" as the benchmark label.
The meter samples Catmull-Rom interpolation at one quarter,
 one half,
 and three quarters between samples
in `package/music-player/desktop-app/src/truepeak.rs`,
not the oversampling filter of ITU-R BS.
1770 true-peak metering.
Both probe and full scan share that meter,
so the system is internally consistent and the gain-error bounds hold against this meter.
But "truth" here is the shared meter's estimate,
not an absolute true-peak figure,
and the meter can under-read a real inter-sample peak that falls between its three probe positions.

### Generalization beyond the current corpus

The shipped policy is fixed and versioned,
but it is tuned for exact fit on the current corpus.
New tracks the user adds later are not covered by that fit.
The only protection for an unseen between-window peak is the fixed probe margin.
The plan should state the expected behavior and safety margin for tracks outside the measured corpus,
not only for the measured library.

### Policy identity derivation

The plan says a policy change must bump the policy ID,
but it does not say how the ID is produced.
A hand-maintained constant can be forgotten when a constant or the classifier artifact changes.
Deriving `policy_id` from a hash of the policy parameters and the classifier artifact
would make a stale-cache bug impossible rather than merely discouraged.

## Review amendments to fold in

These came from a later review pass on 2026-06-25,
an external review plus direct codebase verification.
They extend the sections from `Agreed design decisions` to `Risks and review questions`
and should be merged in place during implementation.
They do not reopen any agreed design decision.

### Decoded-seconds accounting

The benchmark budget must count actual decode work,
including the probe windows that a probe-then-full-scan track decodes before its full scan.
The starting-threshold model `sum(min(duration, threshold))` undercounts those tracks.
Use:

```text
decoded_cost(track) =
  duration                                   if short exact scan
  window_count * window_seconds              if probe estimate
  window_count * window_seconds + duration   if probe then full scan
```

Stage one does not reuse probe-decoded samples for the later full scan,
so a needs-full-scan track pays both.
A later optimization may cache probe windows to drop the double cost,
but the benchmark must assume the double cost until that exists.

### Cache validity beyond policy_id

A row is reusable only when its production environment matches.
Keep these as separate columns,
 not collapsed into `policy_id`:

- `policy_id`:
   constants,
   classifier logic,
   gain math,
   cache interpretation.
- `meter_id`:
   Catmull-Rom behavior,
   including boundary and end-of-track handling.
- `decoder_stack_id`:
   Symphonia and libopus versions and their channel and sample-conversion behavior.
- `schema_version`:
   row layout.

Collapsing decoder identity into `policy_id` is rejected:
it would churn `policy_id` on every decoder bump and needlessly invalidate unrelated rows.
A read is a hit only when the full identity tuple matches.

### Source seeking precision

`seek(seconds: f64)` is too loose for reproducible window placement.
Prefer frame-based seeking,
or require the adapter to seek at or before the target frame then discard decoded frames to the exact frame.
Otherwise the bench sidecar and runtime can measure different windows,
especially near the final window start.

### Unknown or contradicted duration

Duration drives the policy branch,
so define the degenerate cases:

- unknown duration:
   full scan exact.
- duration at or below zero with non-empty audio:
   full scan exact.
- reported duration shorter than the decoded stream:
   use the decoded duration for stored metadata and verification.

Cover these with fake sources.

### Silence and degenerate frames

- A fully silent or zero-peak track yields unity gain and a valid decision kind,
   never a `log10(0)` path.
- `window_frames = max(1, floor(window_seconds * sample_rate))`.
- `TruePeakSource` chunks may end mid-frame;
   the meter must preserve channel routing across chunk boundaries.
- The meter interpolates only once four real samples exist and adds no synthetic end padding;
  that boundary rule is part of `meter_id`.

### Decode-failure handling

Stage one does not cache decode failures;
the known HE-AAC/SBR case may be retried on later runs.
A later stage may add a failure row keyed by `fingerprint + policy_id + decoder_stack_id` with an error kind,
never treated as a gain decision.

### Foreground and background arbitration

The shared service serves foreground current-track decisions and background warming from one cache.
Foreground must win:

- foreground cached reads and resolve requests take priority over warming writes,
- duplicate in-flight `fingerprint + policy_id` work is coalesced where practical.

This keeps the old "background sweep interferes with the current track" failure from returning.

### Android service-handle concurrency

The explicit native handle needs a stated concurrency contract:

- whether `TruePeakService` is internally thread-safe and callable from multiple Kotlin dispatchers,
- whether `release(handle)` drains in-flight work,
   cancels it,
   or only marks the handle closed,
- what a late callback against a released handle does,
- whether errors cross JNI as structured result codes rather than thrown exceptions or sentinels,
- whether the service or Kotlin and WorkManager own the worker threads.

### Android opener boundary

Recommended shape:
Kotlin opens each track source and calls a per-track native resolve or warm-one,
with WorkManager driving iteration as the platform knob.
Avoid Rust worker threads calling a Kotlin opener back over JNI,
which needs JVM attach,
 careful local-reference handling,
 and strict file-descriptor lifetimes.
The shared policy,
 meter,
 cache,
 and decision logic stay in the core either way.

### Gain-change smoothing

A late swap from the conservative fallback to the measured gain should ramp over a short interval
to avoid an audible step.
This lives in the platform output stage,
 not in truepeak-core,
but the plan states it so the Android native move does not introduce a sudden mid-track jump.

### Bench evidence hardening

The bench sidecar should:

- emit the exact `policy_id` and the git commit of the meter and classifier that produced the report,
- report classifier complexity and the full feature list,
- run perturbation tests on probe features,
- run leave-one-artist or leave-one-directory sensitivity checks where metadata allows,
- exercise synthetic adversarial cases near thresholds.

This makes the "non-opaque,
 not a disguised path list" requirement reviewable.

### CI wiring

The new core,
 bench,
 and fuzz packages need CI wiring,
not only local tasks,
so they do not become local-only sidecars.

### Revised staging

Insert a platform-viability stage before the rest,
because the service API depends on Turso being viable on Android:

- Stage zero (done):
   platform viability.
  Build,
   `arm64`/`x86_64` standalone runtime,
   and the in-process `cdylib` round-trip to an
  app-private path are all proven by the spike on the Pixel 6 and the emulator.
- Stage one (done):
   shared meter crate.
  `package/music-player/truepeak-core` ships the meter,
   gain math,
   `TruePeakSource`,
   window
  placement,
   and policy identity,
   with unit tests and a clean `lint:rust` and clippy run.
- Stage two (engine done,
   search ongoing):
   durable evidence.
  `package/music-player/truepeak-core.bench` evaluates and searches on the shared meter;
   its
  run produced the feasible no-classifier finding in the
  `Stage two evidence and the feasible no-classifier finding` section.
  The finer-bin shared-meter `collect` regeneration and exact-window verification remain.
- Stage three (shared surface done):
   full shared service.
   The policy resolver (`resolve_decision`),
   the Turso decision cache keyed by the identity tuple with exact-over-probe precedence
  (`DecisionCache`,
   behind an optional `service` feature so the meter-only apps stay dependency-free),
   the cache-aware `cached_or_resolve`,
   and their fake-source and throwaway-database integration tests are all in the crate.
   The warming loop's concurrency,
   priority,
   and lifecycle stay platform glue over these primitives,
   as this plan assigns them.
- Stage four (done):
   desktop migration.
   The desktop peak-cache actor owns a shared `DecisionCache` (not a hand-rolled `peaks` table),
   the sync handle carries `u64` fingerprints and `Decision`s,
   a `DesktopSource` adapts the
   decoder to `TruePeakSource`,
   and `resolve_current`/`resolve_full` drive the foreground and
   warming paths onto a fresh `decisions.db`.
   78 tests,
   clippy and lint green.
- Stage five (done):
   Android migration.
   The windowed policy and the `1.26` factor are gone;
   a native `TruePeakService` handle
   (dedicated Tokio thread plus `DecisionCache`) is reached over JNI
   (`nativeTruePeakServiceCreate`/`Release`,
  `nativeResolveGain`,
  `nativeWarmTrack`),
   the Kotlin
   JSON peak cache and gain math are deleted,
   and a `TruePeakGain` singleton owns the one shared
   handle.
   The `.so` cross-compiles for both ABIs with Turso linked;
   on-device on the Pixel 6 the
   `NativeBridgeTest` (9 tests,
   including a full service-open + resolve + cache round-trip) and
   `PeakSweepWorkerTest` (2 tests,
   the warming sweep) pass.
- Stage six (done):
   cleanup.
   Both apps' old measurement policies and the Kotlin peak cache are deleted;
   the shared crate
   gained `resolve_full_scan` (warming-upgrade primitive) and `DecisionCache::exact_fingerprints`
   (warming skip-snapshot).
- Stage seven:
   decoder-sharing follow-up.

This supersedes the stage list under `Suggested implementation stages`.
Stage zero plus the meter-first order keep benchmark evidence
from being generated with code that later diverges from production.

## Suggested implementation stages

### Stage one: durable evidence

Move the benchmark harness into repo sidecars.
Regenerate the corrected quarter-target search from the current measured library.
Produce a report proving exact corpus fit.

### Stage two: shared core crate

Create `package/music-player/truepeak-core`.
Implement the shared meter,
 gain math,
 window policy,
 classifier input types,
 policy identity,
 and Turso service.
Use fake-source tests and throwaway Turso databases.

### Stage three: desktop migration

Replace desktop raw-peak cache I/O with the shared service.
Update current-track swap and background warming to use shared decisions.
Do not import legacy desktop cache rows.

### Stage four: Android migration

Add explicit native true-peak service handles.
Replace Kotlin JSON cache use with shared Rust decisions.
Verify Android native Turso build and device behavior.
Do not import legacy Android JSON rows.

### Stage five: cleanup

Remove unused desktop and Android true-peak/cache code.
Update docs that still describe old Android windowing or Kotlin peak cache ownership.

### Stage six: decoder sharing follow-up

After the shared true-peak service is stable,
plan a second extraction that shares more decode code between desktop and Android.

## Reviewer checklist

A reviewer should challenge these points:

- Is the product behavior clear without knowing the repo?
- Does the corrected target use decodable total seconds divided by four?
- Does the plan avoid using full-track truth in runtime classification?
- Does the cache row shape clearly distinguish exact and probe-derived decisions?
- Does policy versioning make stale cache reuse impossible?
- Does Android really move cache I/O and gain decisions into native shared Rust?
- Does desktop really delete its local Turso actor rather than wrapping it?
- Is resetting legacy caches acceptable for both flavors?
- Is the classifier evidence reproducible from repo sidecars?
- Is the staged decoder-sharing path compatible with the first-stage adapter design?
