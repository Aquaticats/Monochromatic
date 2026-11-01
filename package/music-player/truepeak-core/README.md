# truepeak-core

Shared true-peak core for the desktop and Android music players.

The two flavors used to carry separate copies of true-peak measurement,
 gain math,
 and
cache shape,
 so a track normalized on one flavor was not guaranteed to be normalized
under the same policy on the other.
 This crate is the single owner of that behavior:
 the meter,
 the gain math,
 the window-placement math,
 and the versioned policy identity
live here once,
 so both apps can share one measured-and-normalized result.

Both apps depend on this crate by path with the `service` and `fingerprint` features
(`desktop-app/Cargo.toml`, `android-app/rust/Cargo.toml`), so it is live shared infrastructure:
its policy identity keys both apps' persistent decision caches.
Current consumers:

- Desktop (`desktop-app`):
  migrated.
  Its true-peak meter, normalization gain, source contract, policy resolver
  (`resolve_decision_for` with the provenance and FLAC bones inputs of `probe_inputs_from_file`,
  and `resolve_full_scan` for warming), track fingerprint and decision cache
  (`DecisionCache`) come from here.
  It keeps its own decode-loop opener, which feeds decoded chunks into the shared meter,
  and its peak-cache service (`src/peakcache_service.rs`).
- Android (`android-app/rust`):
  migrated.
  Its `TruePeakMeter`, `true_peak_interleaved`, resolver (`resolve_decision` and `resolve_full_scan`
  under `default_policy`), track fingerprint and decision cache come from here;
  the duplicated copies are gone.

The crate ships no classifier.
The decided policy is classifier-free: no probe-feature or metadata rule routes the hard tracks,
so every long track takes the same probe under its provenance bucket.
Warming is each app's call of `resolve_full_scan` on idle time;
the crate holds no warming scheduler.

The integration work,
its staging,
and its history live in
[the integration handover](../../../doc/handover/music-player-truepeak-core-integration.md)
and [the shared true-peak plan](../../../doc/planning/music-player-shared-truepeak-core.md).
The handover keeps its dated progress log, so its early entries describe earlier stages.

## What it measures

True peak (also called inter-sample peak) is the highest level the analog waveform
reaches after a converter reconstructs it between the stored samples.
 It can sit above
the largest stored sample.
 The meter estimates it by oversampling each channel about
four times with a Catmull-Rom cubic at one quarter,
 one half,
 and three quarters between
samples,
 and tracks the largest magnitude.
 The same meter drives both full scans and
window probes,
 which is what makes their peaks comparable.

The gain math turns a measured peak into one constant per-track gain that brings the
track down to a `-1 dBTP` ceiling and never amplifies,
 so playback cannot overflow the
converter.

## Public surface

- `TruePeakMeter` and `true_peak_interleaved`:
   the streaming meter and a whole-buffer
  convenience for tests and synthetic on-device checks.
- `TruePeakSource` and `AudioSpec`:
   the decoded-audio contract the platform implements,
  seeking by frame for reproducible window placement.
- `TruePeakError`:
   the typed error the fallible source methods return.
- `normalization_gain`,
   `peak_dbtp`,
   `probe_estimated_peak`,
   `CEILING`:
   the gain and dB
  helpers.
- `window_frame_starts`,
   `WindowPlacement`:
   even window placement across a long track, from frame 0 to the final legal start
  (`total_frames - window_frames`);
  a single window sits at the final legal start so it covers the ending.
  The shipped resolver does not place windows this way (it measures tenth-second bins with the frontier zoom);
  `truepeak-core.bench` uses these helpers to replay window probes over its corpus.
- `resolve_decision`,
   `resolve_decision_for`,
   `resolve_full_scan`,
   `Decision`:
   drive a `TruePeakSource` through the policy to a gain decision.
  Behind the `service` feature, `DecisionCache` and `cached_or_resolve` persist and reuse decisions in Turso.
- `Policy`,
   `CacheIdentity`,
   `default_policy`:
   the shipped policy and the identity tuple
  that keys cache rows.
- `track_fingerprint`,
   `FINGERPRINT_DOMAIN_TAG`
  (behind the `fingerprint` feature):
   the opaque `u64` cache key both apps derive from a file's path,
  size,
  and modified-time.
  It is XXH3-64 (`twox-hash`, default seed 0) over a domain tag,
  the path length,
  the path,
  the size,
  and the mtime nanoseconds,
  so desktop and Android hash identical inputs identically by construction.
  Known answers from the xxHash reference C implementation pin it in `src/fingerprint_tests.rs`.
  Decision:
  [`doc/decision/music-player-fingerprint-hash.md`](../../../doc/decision/music-player-fingerprint-hash.md).

## Policy identity

A cached decision is reusable only when the full identity matches:
 `policy_id` (the
constants,
 gain math,
 and cache interpretation),
 `meter_id` (the meter
behavior,
 including the chunk-seam and end-of-track rules),
 `decoder_stack_id` (the
platform's decoder behavior,
 supplied by the platform),
 and `schema_version` (the row
layout and the fingerprint hash that fills the key column).
 These stay separate so a decoder bump does not churn unrelated rows.
 The
`policy_id` is derived from the policy parameters,
 so changing a constant cannot silently
reuse a stale cache row.
 `DecisionCache::open` deletes every row whose `schema_version` is below the shipped one,
so rows left by a retired fingerprint hash do not accumulate.
 Version 2 is the switch from the gxhash fingerprint to XXH3-64.

`default_policy` holds the shipped policy (`src/policy.rs`; each constant below is named there).
A track at or below `SHORT_SCAN_MAX_SECS` (90 s), or of unknown length, is scanned in full for an exact peak.
A longer track is probed by the frontier zoom (`src/probe.rs`, driven by `resolve_decision_for` in `src/resolve.rs`)
over bins of `PROBE_WINDOW_SECS` (0.1 s):
an even pass over `PASS1_COVERAGE_FRACTION` (a tenth) of the bins
(`BONES_EVEN_COVERAGE_FRACTION`, 0.05, when bones seeds are present),
then a climb that repeatedly measures the bins beside the loudest one heard so far
until the bucket's coverage is spent.
Each bin has its own meter, so a seek seam cannot fabricate a spike.
For a lossless FLAC file, bones seeds (`BONES_TOP_SLOTS`, 40 hot slots read from the frame sizes without decoding)
start the climb on the loud passages.
The loudest sampled bin is then inflated by the margin of the track's provenance bucket.

The bucket table sets the share of bins the probe may measure (`coverage_fraction`)
and the margin in dB (`probe_margin_db`):

- `LOSSLESS_PROBE`: lossless without bones, 0.10 coverage, 0.45 dB.
- `LOSSLESS_BONES_PROBE`: lossless with bones seeds, 0.07 coverage, 0.45 dB.
- `STORE_PROBE`: store-tagged lossy, 0.32 coverage, 0.30 dB.
- `YOUTUBE_PROBE`: YouTube-tagged lossy, 0.14 coverage, 0.50 dB.
- `BARE_PROBE`: untagged lossy, 0.34 coverage, 0.50 dB.

The gain error bounds are `CEILING_DBTP` (-1 dBTP), `MAX_TOO_LOUD_DB` (+0.5 dB) and `MAX_TOO_QUIET_DB` (-2 dB).
The margins trade worst-case too-quiet against the count of cold-start tracks the realtime clamp catches;
see the plan's fine-bin findings.
Changing any constant re-keys the cache automatically.

The Stage-two policy, a proportional probe (a fifth of each longer track in short evenly placed windows)
with a fixed 0.8 dB margin, is retired.
Its figures live on only as the `LEGACY_*` constants of `truepeak-core.bench/src/main.rs`,
which keep the bench's `--proportional` ledger reproducible.

## Versioning

The crate is unpublished (`publish = false` in `Cargo.toml`, absent from crates.io) and both apps and the bench take
it by path, so the version number is bookkeeping and gates no build.
It moved from 0.1.0 to 0.2.0 on 2026-10-18 to record the changes since the first version:
the XXH3-64 track fingerprint, the cache schema 2, `DecisionCache::put_batch` and the per-track decision removal.
While the crate stays below 1.0, a change that alters the public surface or the cache schema bumps the minor version,
and the four `Cargo.lock` files (this crate, `truepeak-core.bench`, `desktop-app` and `android-app/rust`) are
regenerated with cargo in the same commit.
The apps' own versions (the release tag) are independent of this one.

## Tasks

Run these with `mise run //package/music-player/truepeak-core:<task>`.

- `build`,
   `build:debug`:
   compile the library.
- `lint`:
   `cargo check`.
- `lint:clippy`:
   clippy with warnings denied.
- `test`,
   `test:debug`:
   the unit tests.
