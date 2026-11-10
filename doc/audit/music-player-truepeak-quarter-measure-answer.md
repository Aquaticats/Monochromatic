# On the quarter measure: the answer

This answers `doc/audit/music-player-truepeak-quarter-measure.md`:
 a true-peak probe policy
that beats the shipped proportional policy on all three of the letter's measures at once,
under the same law (decode at most a quarter of the library's seconds),
 reproduced from the
same corpus by committed bench modes.
 Every road the letter asked about was tested,
 plus the
rounds of levers raised afterwards;
 refusals are reported with numbers.

## The method, in two layers

Layer one,
 the probe:
 **frontier zoom**.
 Keep the ninety-second rule (short tracks decoded
whole,
 known exactly).
 For each longer track,
 spend its budget in two motions:
 an even pass
of single 0.1 s windows,
 then the climb:
 repeatedly decode the 0.1 s windows on either side
of the loudest window heard so far,
 until the track's share is spent.
 Half of all loud long
tracks end within 0.01 dB of their true crest.

Layer two,
 the allocation:
 **bucket-first coverage and margin**.
 Buckets come from embedded
tags only (codec,
 store identifiers,
 iTunNORM,
 youtube provenance;
 never path text,
 per the
plan's classifier rules).
 Their measured tails diverge so hard that uniform coverage is
wasteful:

- flac (lossless,
   311k long-track seconds):
   coverage 7%,
   margin 0.45 dB,
   probe seeded by
  frame-size bones (below).
- store-tagged lossy (ISRC/UPC/iTunNORM,
   22k seconds):
   coverage 32%,
   margin 0.30 dB.
- youtube-provenance lossy (37k seconds):
   coverage 14%,
   margin 0.50 dB.
- bare untagged lossy (534k seconds,
   the risk bucket):
   coverage 34%,
   margin 0.50 dB.

The freed lossless seconds are what buy the risk bucket its extra coverage.
 Everything is
fitted to this corpus exactly,
 per the plan's agreed fit-the-current-corpus decision;
 the
mechanism is content-generic,
 the numbers are not.

## Measured result

Same corpus,
 same meter,
 same law.
 The three measures,
 as policy generations:

- shipped even probe,
   margin 0.8:
   43 clamps,
   average needless-quiet 0.528 dB,
   worst 0.80 dB
  (decodes 193094 s of the 229215 s allowed).
- uniform frontier zoom,
   margin 0.5:
   26 clamps,
   0.371 dB,
   0.50 dB (228970 s).
- bucket table,
   no bones:
   19 clamps,
   0.349 dB,
   0.50 dB (227901 s).
- bucket table with FLAC bones (the decided dial):
   **17 clamps,
   0.367 dB,
   0.50 dB**
  (227748 s,
   24.84% of the corpus).

Alternative dials,
 all in budget,
 all beating the shipped policy on every measure:

- clamp-min:
   16 clamps at 0.383 dB average (flac margin up to 0.50).
- average-min at worst 0.50:
   22 clamps at 0.303 dB (flac at 8%,
   margin 0.30).
- all margins at or under 0.40:
   28 clamps,
   0.293 dB average,
   worst quiet 0.40 dB
  (versus the uniform zoom's 43 / 0.285 / 0.40 at that worst-quiet dial).

Ranking among the dials:
 the decided 17-clamp table first (fewest clamps at an average
still below the uniform zoom's,
 and every measure beats the ledger),
 then the all-0.40
table (if the worst quiet is what the ear notices,
 0.40 everywhere at 28 clamps is the
better trade),
 then the 22-clamp average-min (buys 0.06 dB average for 5 clamps),
 last the
16-clamp point (one clamp for 0.016 dB average is a poor exchange).

## Why this works

The even probe's under-read is set by how many 0.1 s bins anywhere in a track come near its
crest.
 The median track has about twenty such bins scattered through its loud passages;
 a
sparse probe strikes one almost surely,
 and the climb walks up the shoulder it lands on.
The 43 clamp-tail tracks of the shipped policy have one or two such bins:
 needles.
 A
needle's neighbors sit 1.6 to 2 dB below it,
 no louder than the track's ordinary chorus,
which is why no probe-feature classifier can exist (`analysis/anatomy.mjs`).

Buckets work for the same reason in reverse:
 lossless and store-mastered tracks have thin
tails (flac p99 under-read 0.65 dB at 10% coverage),
 so their coverage is nearly free to
cut,
 and the bare-lossy bucket's tail thins measurably with every extra point of coverage
(clamps at margin 0.5:
 200 at 10%,
 23 at 24%,
 14 at 32%).

## The encoding bones: lies for lossy, truth for lossless

The letter's most tempting road splits cleanly in two.

For perceptual codecs it is refuted.
 Packet-size profiles of all 2441 lossy loud long
tracks (Opus TOC walk,
 m4a sample tables,
 mp3 frame walk;
 no decoding) place the crest's
slot at the median at the 60th percentile of its own track's byte-rate,
 worse than chance.
Codecs spend bits on spectral busyness,
 not peak height;
 every composed policy that
diverted budget toward byte-heavy slots matched or lost to zoom alone
(`analysis/correlate.mjs`).

For FLAC it is confirmed.
 Lossless bits track residual entropy,
 which tracks signal level:
a CRC-verified frame walk over 1272 loud long FLACs (about 60 s of wall clock for 50 GB,
reading only headers) puts the crest's slot at the median at the 8.3rd byte-rank percentile
(pad ±1).
 One parser subtlety matters:
 FLAC frames last about 0.095 s,
 so frame bytes must
be spread overlap-proportionally into 0.1 s slots;
 binning by start time aliases into a
sawtooth and halves the correlation (`analysis/flac-bones.mjs`;
that script was retired on 2026-11-10,
and the measurement now comes from
`truepeak-core-bench <corpus> --bones <out.jsonl>`,
which walks through `truepeak_core::flac_bones_profile`).
 A bones-seeded probe at
roughly 9% coverage nearly matches an even probe at 24% (p90 under-read 0.23 vs 0.20 dB),
which is what lets the flac bucket run at 7% coverage in the decided table.

## The roads measured and declined

### Album kinship

Real (95.7% of loud long tracks live in groups of three or more;
 the median member sits
0.92 dB below its group max;
 probing picks the true loudest member 66.8% of the time) but a
poor buyer of seconds:
 capping fellows by a fully-scanned album master spends all spare
budget for average 0.362 dB at best,
 never touches the worst case,
 and at allowance zero
creates five net new clamps.
 The same seconds spent on coverage do strictly better
(`analysis/album-prior.mjs`).

### Duplicate-master pooling

Only 1.20% of the library's seconds are duplicated (53 groups,
 107 tracks).
 The naive
version is actively harmful:
 profile correlation is scale-invariant,
 so it pools different
masters of the same song (+3 clamps,
 worst quiet doubles).
 The gain-verified version
(close-bin fraction over 0.95;
 crest spread at most 0.16 dB) is safe and useless:
 it frees
about 2000 s and removes zero clamps.
 If ever revisited,
 match on gain sensitivity,
 never
correlation (`analysis/duplicates.mjs`).

### Declared peaks

All 723 ReplayGain carriers are opus,
 and the true crest exceeds the declaration by a
4.6 dB-wide band (median 1.92),
 useless as a bound and always below what the zoom probe
already found.
 iTunNORM peaks are tighter (median gap 0.42 dB) but number 55 with a 5.08 dB
outlier.
 The marks name the safe;
 they cannot bound the loud.

### Per-track margins, reallocation, position priors, placement variants

Measured dead (`analysis/levers.mjs`,
 `analysis/variants.mjs`):
 heard-statistic margin
formulas (Spearman of residual under-read vs heard spread,
 level,
 duration:
 -0.06 to
-0.15,
 and the spread points the wrong way,
 since flat heard profiles are hot masters
hiding needles);
 provable-safety early stops (22 of 3903 long tracks qualify,
 613 s);
position-weighted pass one (crests skew late,
 but the climb already recovers late
structure);
 golden-ratio jittered pass one (the grid does not alias);
 wider zoom expansion
radii (worse than ±1);
 heard-clipping density (Spearman 0.037).

### No full hearings at all

Dropping the ninety-second rule frees three quarters of 12153 short-track seconds and buys
exactly one clamp at margin 0.5,
 while costing 0.014 dB of average quiet,
 because two
hundred short tracks trade exactness for margin-bearing estimates.
 Both variants beat the
shipped policy;
 the rule stays because quiet error is the letter's priced cost and exact
short-track gains never need the background warming pass probed tracks still owe.

## What no method here can do

The needles stay needles.
 Of the tracks the decided table still clamps,
 the crests are
events that break their own song's structure:
 measured on the uniform zoom's 26 residual
clamps,
 9 crests are strictly one-off (the probe already reads within 0.1 dB of the loudest
instant two or more seconds away),
 and the other 17 keep an echo that is itself a needle
(the tail's median is one crest plus one echo within 1 dB,
 whole-track).
 A perfect
echo-finder would dissolve about 14 clamps and nothing else.
 No free channel locates
needles:
 not heard neighbors,
 not heard distribution shape,
 not lossy byte profiles,
 not
provenance,
 not albums,
 not declared tags.
 Exactness on every track under the quarter law
is not a missing cleverness;
 it is absent information.
 The honest frontier is the
miss-distribution,
 the margin dial,
 the realtime clamp on a cold first play,
 and background
warming converging every track to exact.

Practical notes:

- The climb is adaptive (pass two depends on pass one),
   so the decoder seeks more than an
  even probe;
   expansions cluster into contiguous runs an implementation can decode as runs.
  The law counts decoded seconds;
   the machine additionally pays seek time.
- The bucket table and the bones threshold are fitted to this corpus,
   per the plan's
  fit-the-corpus decision.
   The margin remains the guard against drift,
   and the realtime
  clamp remains the safety net (worst first-play overshoot 1.14 dB above the ceiling under
  the decided table,
   brief and cold-start-only).
- Bucket assignment reads embedded tags only;
   the tag sweep and FLAC profiles are
  regenerable side files,
   not path heuristics.

## Reproduction

Everything reproduces from the recorded corpus without re-decoding the library.
 On the
branch `truepeak-quarter-answer`,
 in `package/music-player/truepeak-core.bench`:

- Decided composite:
   regenerate side files with `node analysis/tags-sweep.mjs` and
  `node analysis/flac-bones.mjs extract`,
   then run the bench with the corpus,
  `out/tags-full.jsonl`,
   `out/flac-profiles.jsonl`,
   and `--buckets`
  (17 clamps / 0.367 / 0.50;
   commit c60ee60f8).
  The profile half of that recipe changed on 2026-11-10:
  `analysis/flac-bones.mjs` was retired,
  and `out/flac-profiles.jsonl` now comes from
  `mise run //package/music-player/truepeak-core.bench:run -- <corpus> --bones out/flac-profiles.jsonl`,
  which walks through the shared `truepeak_core::flac_bones_profile`.
  On a sample of the corpus the retired script and the bench wrote identical rows.
- Uniform zoom layer:
   `--zoom` with the corpus and metadata (commit 0bda0177f).
- The searches and dead ends:
   `analysis/compose-final.mjs` (the table search),
  `analysis/buckets.mjs`,
   `analysis/final.mjs`,
   `analysis/adaptive.mjs`,
  `analysis/anatomy.mjs`,
   `analysis/correlate.mjs` with `analysis/bones-*.mjs`,
  `analysis/flac-bones.mjs` (retired 2026-11-10;
  the bench's `--bones` mode runs the shared walk),
   `analysis/album-prior.mjs`,
   `analysis/duplicates.mjs`,
  `analysis/levers.mjs`,
   `analysis/variants.mjs`,
   `analysis/tags-sweep.mjs`.
