# truepeak-core-bench

Corpus evaluation and corrected-target parameter search for the shared true-peak policy.

This is Stage two of the shared true-peak plan:
 durable,
 reproducible evidence built on
the production [`truepeak-core`](../truepeak-core) meter,
 gain,
 window-placement,
 policy math,
 and probe selection,
 so the benchmark cannot diverge from what playback runs.

## What it does

It reads a per-track measurement corpus (`tracks.jsonl`) where each line is one track
measured by the shared meter:
 the full true peak and per-second bin peaks.
 Because the
bins are exact per-second meter peaks,
 any window policy can be simulated by re-slicing
them,
 so the search never re-decodes audio.

The probe's bin selection is the shipped one:
the frontier zoom and the bucket composite ask
`truepeak_core::select_probe_bins` which bins to measure,
which is the same function the production probe calls.
What the bench simulates is the measurement it feeds that selection:
the corpus carries bin peaks a collector already measured,
so the selection reads those instead of decoded windows.
A core test drives both measurement shapes over a synthetic bin corpus and fails if the
bins they measure ever differ.

For a candidate policy `(window_count, window_seconds, probe_margin_db)` it:

- splits tracks into short (exact full scan) and long (probe),
- places the probe windows with `truepeak_core::WindowPlacement` and reads the bins they
  cover,
- computes the probe and exact gains with the shared gain math,
- scores the gain error against the `+0.5 / -2.0 dB` bounds,
- sums the decoded-seconds cost under the plan's amended accounting (a probe-then-full
  track pays both the probe and the full scan),
- ranks candidates by the decided objective:
   loudest-safe (least worst-case too-quiet)
  first,
   then simplest classifier.

It then prints the feature distributions of violators versus non-violators,
 fits a
probe-only full-scan classifier,
 and writes a per-track feature dump
(`out/long_features.jsonl`) so a metadata-aware classifier can be evaluated offline by
joining on the track path.

## Run

```sh
# The corpus is produced by the collector (truepeak-core-collect); it is not committed.
# The classifier-era sweep (historical exploration):
mise run //package/music-player/truepeak-core.bench:run -- /path/to/tracks.jsonl
# The decided policy on the fine corpus, with optional provenance metadata:
mise run //package/music-player/truepeak-core.bench:run -- /path/to/tracks-fine.jsonl /path/to/metadata.jsonl --proportional
```

The target is computed from the corpus as `total decodable seconds / 4`,
 the corrected
quarter-library benchmark target.

## The bucket composite's side files

`--buckets` joins two files that are not committed,
 because both read the library itself:

```sh
# The tag sweep: one codec and embedded-tag record per track.
node analysis/tags-sweep.mjs
# The FLAC byte profiles: the shared framing walk over every loud long FLAC.
mise run //package/music-player/truepeak-core.bench:run -- /path/to/tracks-fine.jsonl --bones out/flac-profiles.jsonl
# The composite, over the corpus and both side files.
mise run //package/music-player/truepeak-core.bench:run -- /path/to/tracks-fine.jsonl out/tags-full.jsonl out/flac-profiles.jsonl --buckets
```

`--bones` writes one `{"path", "slotSecs", "bytes"}` row per loud long FLAC,
 walking container framing through `truepeak_core::flac_bones_profile`,
 the same function the shipped probe seeds its bins with.
The side file therefore comes from one implementation of the walk,
 not a second one kept beside it in another language.
`slotSecs` is `truepeak_core::BONES_SLOT_SECS`,
 and each slot's bytes are whole numbers.
The report reads a slot index as a corpus bin index,
 so the file means what it says only for a corpus whose bins are that slot wide
 (the collector's default,
 and the width the recorded corpus carries);
 issue 5393 holds making the join width explicit.
The targets are the shipped policy's long lossless tracks above its dBTP ceiling,
 the same policy `--buckets` evaluates,
 so a policy change moves the file and the report together;
 every row is written in corpus order,
 so two regenerations of one corpus diff cleanly.
A regeneration of the recorded corpus took about five minutes
 (1173 tracks,
 48.6 GB,
 four read at a time),
and every row it wrote was byte-identical to the file the retired JavaScript
profiler had written for the same track.
A track that left the disk,
 or whose bytes are not a walkable FLAC stream,
 is counted and skipped,
 so a library that changed under an older corpus still yields the rows beside it.

## Findings

The full exploration and its evidence live in the plan's Stage-two section.
 The decided policy
is proportional coverage plus a fixed margin,
 with no classifier:

- A fixed-length probe under-covers long tracks,
 so coverage FRACTION,
 not window granularity,
  is the lever.
 A handful of tracks hide a sub-tenth-second transient no sparse probe catches,
  so guaranteeing exact fit costs about a decibel of margin on every track.
- The under-read distribution is extremely skewed (median near `0.14 dB`),
 so the shipped
  policy probes a fifth of each long track and applies a fixed `0.8 dB` margin:
 about
  ninety-nine percent of tracks stay within `-0.8 dB`,
 and the rest clamp on cold start until
  warming corrects them.
 `--proportional` reproduces these numbers on the fine corpus.
- Provenance metadata (lossless or yt-dlp) never mislabels a hard track as safe,
 so it lowers
  the average error;
 amplitude and probe-window statistics do not separate the hard tracks,
  because their peaks live in unsampled gaps.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/music-player/truepeak-core.bench:<task>`.
The repository's Rust rules run from the root `mise run lint`.
