# Handover: answering the quarter-measure letter

## The task

Answer `doc/audit/music-player-truepeak-quarter-measure.md`:
 find a true-peak
sampling policy for the music library that beats the shipped proportional policy
on at least one of three measures (worst too-quiet,
 average too-quiet,
 cold-start
clamp count) while decoding no more than a quarter of the library's seconds
(229215 of 916861).

## Where things live

- The letter:
   `doc/audit/music-player-truepeak-quarter-measure.md`.
- Evidence + prior roads:
   `doc/planning/music-player-shared-truepeak-core.md`,
  Stage-two section (from line ~787).
- Ground truth corpus:
   `package/music-player/truepeak-core.bench/out/tracks-fine.jsonl`
  (95 MB JSONL;
   per track:
   path,
   duration_secs,
   full_peak linear,
   bin_seconds=0.1,
  bin_peaks = per-0.1s Catmull-Rom window peaks).
   Any probe policy simulates offline
  against this;
   no re-decoding needed.
- Provenance:
   `out/metadata.jsonl` (codec,
   kbps,
   lossless,
   ytdlp),
   `out/peak-tags.jsonl`.
- Baseline evaluator:
   `truepeak-core.bench --proportional` (src/proportional.rs) reads
  `truepeak_core::default_policy` (short_scan<=90s,
   coverage 1/5,
   window 0.1s,
  margin 0.8 dB) and reproduces the ledger:
   decoded ~21%,
   under-read median 0.14 /
  p90 0.63 / p99 1.33 / max 2.25 dB,
   43 tracks clamped at margin 0.8
  (clamped = under_read - margin > max_too_loud 0.5).
- The actual audio library is local at `/home/user/Seafile/Plain/Music` (65 GB;
  opus 2583,
   flac 1251,
   aac 135,
   mp3 13 in the older 3991-track metadata pass).

## Work environment

- Experiments and any code live in the worktree
  `/var/home/user/worktrees/truepeak-quarter-answer` (branch `truepeak-quarter-answer`).
  User authorized anything on worktree branches.
   Main worktree stays clean except
  this handover and the final answer doc.
- Main repo HEAD moved during the session (stage-three policy resolver landed);
  concurrent sessions are normal here,
   do not touch unrelated changes.

## Plan (task list mirrors this)

1.  Reproduce ledger baseline with bench + an offline simulator (parity check),
    and compute the baseline average-quiet measure.
2.  Tail anatomy:
     around each track's crest bin,
     does loudness decay slowly enough
    that heard neighbors betray the unheard crest?
     Especially for the 43 clamp-tail
    tracks.
     Decides whether adaptive zoom can work.
3.  Simulate adaptive two-pass (uniform then zoom),
     position priors,
     full-budget
    (25% vs shipped 21%) reallocation,
     per-provenance margins.
4.  Album prior:
     parent-dir grouping,
     within-album crest spread,
     album-max capping.
5.  Encoding bones:
     parse Ogg packet sizes / m4a stsz / flac+mp3 frame sizes (no
    decode;
     free under the letter's law) into per-time byte-rate profiles;
     test
    whether they locate crests;
     simulate profile-guided probes.
6.  Compose final policy,
     verify budget,
     wire committed evaluation into
    truepeak-core.bench on the branch.
7.  Write the answer doc under `doc/audit/`,
     same register as the letter.

## Status: reopened for the bucket round (user pushback), then re-close

The user pushed past the first close twice:
 (a) "no full hearings" variant
(measured:
 buys one clamp,
 costs 0.014 dB average,
 rule kept);
 (b) "these are
songs" (measured:
 structure ceiling bounds any echo-finder to ~14 of the
residual 26 clamps;
 9 crests are strictly one-off);
 (c) "bucket the metadata
first" (WORKS,
as the bucket round results show);
 (d) "find more levers yourself" (lever matrix run).

Bucket round results (committed 77fcc0ef0,
 branch):

- tags-sweep.mjs:
   ffprobe inventory of the whole library (no decode).
  Buckets from embedded tags only (production-legal,
   no path text):
  flac / store (store IDs or iTunNORM) / purl (youtube provenance tag) /
  bare (untagged lossy).
   Long-track seconds:
   bare 534k,
   flac 311k,
   purl 37k,
  store 22k.
- Per-bucket zoom tails diverge hard:
   flac p99 under-read 0.65 dB at 10%
  coverage;
   bare 1.83 at 10%,
   1.00 at 24%,
   0.92 at 28%.
- Composite winner (verified by independent re-eval):
   flac=(c 0.10,
   m 0.45),
  purl=(0.18,
   0.40),
   store=(0.32,
   0.30),
   bare=(0.32,
   0.50):
   19 clamps
  (4 flac,
   14 bare,
   1 purl) / avg 0.349 / worst 0.50,
   decoded 227901 s.
  Beats uniform zoom (26 / 0.371 / 0.50).
   All-margins-0.4 variant:
   29 clamps /
  0.283 avg / 0.40 worst (vs uniform m=0.4:
   43 / 0.285 / 0.40).
- Levers measured dead this round:
   jittered/golden-ratio pass-1 (grid does not
  alias),
   wider zoom expansion radii,
   heard-clipping density (Spearman 0.037),
  declared-peak floors (rgPeak all-opus with 4.6 dB understatement band;
  iTunNORM n=55 with a 5 dB outlier).
- Duplicate-master pooling:
   measured and declined (analysis/duplicates.mjs,
  commit bd92acb15).
   Duplicate mass is 1.20% of library seconds (53 groups,
  107 tracks).
   Correlation-matched pooling is HARMFUL:
   correlation is
  scale-invariant so it pools different masters of the same song (+3 clamps,
  worst quiet 0.50 -> 1.04).
   Gain-verified pooling (close-bin fraction > 0.95,
  crest spread <= 0.16 dB) is safe but frees only ~2000 s:
   zero metric
  movement.
   Lesson recorded:
   match on gain-sensitivity,
   never correlation.
- FLAC bones CONFIRMED (analysis/flac-bones.mjs,
   commits 4a3db5dd2+3b4c782d4):
  lossless frame sizes track signal level,
   unlike perceptual codecs.
   1272 of
  1287 loud long FLACs parsed by CRC-verified frame walk (~60 s for ~50 GB;
  seektables too coarse at ~10 s).
   Key parser finding:
   FLAC frames last
  ~0.093-0.096 s,
   so bytes must be spread overlap-proportionally into 0.1 s
  slots (start-binning aliases into a 2x sawtooth;
   Pearson 0.47 -> 0.81).
  Crest byte-rank percentile:
   pad ±1 median 8.3% (lossy was 36.6%).
   Bones
  top-40 ±1 + even 5% at ~8.8% coverage:
   under-read p90 0.23 / p99 0.56 /
  max 2.67 (vs even-24%:
   0.20 / 0.46 / 1.01),
   1 clamp in 1272.
   FLAC keeps its
  accuracy at about a third of the decode cost.
   Profiles regenerate to
  out/flac-profiles.jsonl (not committed).
- FINAL COMPOSITE WIRED (compose-final.mjs + src/buckets.rs,
   commit c60ee60f8;
  clippy + rust-linter clean).
   `truepeak-core-bench <corpus> <tags-full.jsonl>
  <flac-profiles.jsonl> --buckets` reproduces:
   decided table flac=(7%
  bones-guided,
   m 0.45),
   store=(32%,
   0.3),
   purl=(14%,
   0.5),
   bare=(34%,
   0.5):
  17 clamps (flac 3,
   purl 1,
   bare 13) / avg 0.367 / worst 0.50 / worst_over
  1.14,
   decoded 227748 s (24.84%).
   Frontier alternatives recorded in
  compose-final.mjs output:
   16 clamps at avg 0.383;
   all-margins-0.4 dial gives
  28 clamps / 0.293 / 0.40.
   Progression:
   shipped 43/0.528/0.80 -> uniform zoom
  26/0.371/0.50 -> buckets 19/0.349/0.50 -> buckets+bones 17/0.367/0.50 (or
  16 / 19 / 28 depending on dial).
- Answer doc finalized (main,
   commit 81ab5f2eb):
   two-layer method (frontier
  zoom probe + bucket-first allocation with FLAC bones),
   full dial table,
  split bones verdict,
   every declined road with numbers,
   reproduction
  commands.

## Phase two: redo the policy engine with the final policy, merge to main

User asked (17:15) to rebuild truepeak-core around the decided policy and
merge the branch to main.
 Main had diverged (25 commits):
 stage three landed
cached_or_resolve + Turso DecisionCache behind the `service` feature
(src/service.rs,
 src/cache.rs),
 resolve_full_scan,
 and desktop + Android are
now on the shared decision service,
 calling
resolve_decision(&default_policy(),
 &mut source) in
desktop-app/src/truepeak.rs and android-app/rust/src/truepeak.rs,
 with
cache identity from default_policy().cache_identity(decoder_stack_id()).

- Merged main into truepeak-quarter-answer cleanly;
   bench --buckets still
  reproduces 17 clamps / 0.367 / 0.50 on the merged branch.
- Second main merge landed cleanly (Android native decision service finished
  on main;
   Kotlin peak cache and gain math moved into it).
- Bench updated to the new engine:
   --proportional replays the stage-two ledger
  from local legacy constants;
   --buckets/--zoom read every dial from the
  SHIPPED default_policy (single source of truth) and reproduce
  17 clamps / 0.367 / 0.50 in budget (decoded 228014 s;
   profile-less FLACs now
  correctly take the lossless 10% dial instead of the bones 7%).
- API-boundary refactor (library concerns into the library):
   new
  truepeak-core inputs.rs (probe_inputs_from_file/_from_bytes:
   FLAC sniff ->
  lossless provenance + bones hot bins;
   documented degradation to the bare
  bucket),
   policy.rs stack_id(description) for decoder-stack ids,
  cached_or_resolve now provenance-aware (provenance + bones params).
  62/62 core tests pass.
- Missing-logger gap filed as issue #266 (everything must be logged;
   user
  handles implementations later);
   inputs.rs degradation and bones failures
  named as priority sites.
- Consumers updated + boundary refactor committed (71dda2060):
   desktop
  resolve_current feeds probe_inputs_from_file into resolve_decision_for;
  desktop + android decoder_stack_id use shared stack_id (android keeps gxhash
  only for fingerprints).
   Verified:
   core 62/62;
   desktop 78/78 via `mise run
  test` (raw `cargo nextest` fails the two ElementHandle UI tests because
  SLINT_EMIT_DEBUG_INFO=1 is set by the task at build time,
   not a regression);
  android native verified via `mise run build:native` cargo-ndk cross-build
  (the crate cannot host-compile by design:
   ndk-sys + gxhash target features);
  clippy + rust-linter clean on all four crates.
- MERGED TO MAIN (merge commit abc7b7cc8) and re-verified on main:
   core 62/62
  (mise run test),
   bench --buckets reproduces 17 clamps / 0.367 / 0.50 in
  budget (side files copied into main's gitignored out/;
   regenerable via
  analysis/tags-sweep.mjs and analysis/flac-bones.mjs extract),
   desktop 78/78
  (mise run test).
   Android native verified via cargo-ndk cross-build on the
  branch immediately before the merge with identical content.

## Phase two status: complete

The engine ships the bucket-zoom policy (probe.rs,
 bucketpolicy.rs,
 bones.rs,
inputs.rs),
 consumers are on the simplified boundary,
 and everything is on
main.
 Remaining follow-ups:
 logging (issue #266),
 platform tag readers for
store/youtube provenance (desktop and android currently get lossless + bones
via probe_inputs_from_file and land everything else in bare),
 android
file-bytes plumbing for bones over JNI fds,
 and the plan's standing
exact-decoded-window verification of the bin-level corpus numbers.
- ENGINE REDESIGN LANDED on the branch (commit 685f5ce96):
   truepeak-core now
  ships the bucket-zoom policy.
   New modules:
   probe.rs (frontier zoom driving
  the seekable source),
   bucketpolicy.rs (BucketProbe/BucketTable/
  TrackProvenance;
   unknown provenance lands in bare,
   the deepest coverage,
   so
  uninformed callers never under-probe),
   bones.rs (FLAC CRC-8-verified frame
  walk,
   overlap-proportional slot spreading,
   flac_bones_profile +
  bones_hot_bins).
   resolve_decision keeps its exact signature (delegates);
  resolve_decision_for(policy,
   source,
   provenance,
   bones_hot_bins) is the
  informed entry.
   policy_id hashes all 18 dials (caches re-key).
   57/57 tests,
  clippy + rust-linter clean.
   Android untouched (their work is ongoing;
  source-compatible by construction).
   Desktop untouched so far (compiles
  against unchanged signatures;
   provenance wiring is a follow-up).
- Engine redesign plan:
   zoom lives INSIDE resolve_decision (TruePeakSource is
  already seekable + pull-based,
   so adaptivity is engine-internal);
   Policy
  grows per-bucket coverage/margin (lossless/store/youtube/bare + a
  lossless-with-bones coverage),
   pass-1 coverage,
   bones top-slot count;
  policy_id() must hash all new fields (auto re-keys caches;
   SCHEMA_VERSION
  in policy.rs);
   resolve_decision gains provenance input (+ optional FLAC
  bones hot-window starts);
   FLAC frame-walk ported to a core module so
  platforms can compute bones from raw file bytes (reference implementation:
  bench analysis/flac-bones.mjs,
   overlap-proportional slot spreading,
  CRC-8-verified frame walk).
   Consumers pass provenance from their decoders
  (lossless flag at minimum;
   store/purl default false degrades safely into
  the bare bucket).
   Bench legacy modes (--proportional etc.) get local
  constants for the old shipped policy so the ledger stays reproducible.

## Prior status: complete (first close)

- The answer is committed:
   `doc/audit/music-player-truepeak-quarter-measure-answer.md`
  (main,
   commit d33f2bc58).
- The committed evaluation lives on branch `truepeak-quarter-answer` (pushed):
  `truepeak-core.bench --zoom` (src/zoom.rs,
   commit 0bda0177f) reproduces every
  quoted number;
   analysis/*.mjs hold each road's evidence (baseline parity,
  anatomy,
   adaptive zoom,
   bones extraction + correlation,
   album prior,
   final
  comparison).
   Byte profiles regenerate via `node analysis/bones-extract.mjs`.
- Result:
   frontier zoom (even 0.1-coverage pass,
   then loudest-neighbor
  expansion to the full quarter budget,
   228970 of 229215 s) at margin 0.5 beats
  the ledger on all three measures at once:
   26 clamps vs 43,
   avg quiet
  0.371 dB vs 0.528,
   worst quiet 0.50 dB vs 0.80.
   Margin dial and a
  provenance-split option (avg 0.315,
   27 clamps) are recorded in the answer.
- Not done (possible follow-ups):
   shipping the zoom policy in truepeak-core's
  resolver (the bench mode is evaluation-only;
   production needs two-phase
  adaptive window placement),
   and verifying zoom against exact decoded windows
  rather than the 0.1 s bins (same caveat the plan records for the shipped
  policy).

## Findings so far

- Baseline measures to beat (margin 0.8):
   worst quiet 0.80 dB,
   average
  needless-quiet across all 4114 tracks 0.528 dB,
   43 cold-start clamps.
  Shipped policy decodes 193094 s;
   the quarter budget is 229215 s,
   so
  36121 s go unspent.
- Sanity:
   max(bin_peaks) equals full_peak exactly for every track,
   so bin-level
  simulation is faithful.
- Tail anatomy:
   the 43 clamped tracks are isolated needles.
   Median tail track
  has exactly 1 bin (0.1 s) within 0.5 dB of its crest across the whole track
  (ordinary tracks:
   median 20);
   even bins adjacent to the crest sit 1.6-2 dB
  below it.
   Under-read is governed by how many near-crest bins exist,
   so no
  local shoulder betrays the needles;
   zoom alone cannot catch them.
  All 43 are lossy (41 opus,
   2 m4a),
   none FLAC.
- Frontier zoom (pass-1 even single 0.1 s bins at 10% coverage,
   then repeatedly
  decode undecoded neighbors of the loudest decoded bin,
   up to 24% total
  coverage = full budget):
   under-read p50 0.01 / p90 0.35 / p99 0.91 / max 1.84.
  With margin 0.5 it beats the ledger on all three measures at once:
  26 clamps vs 43,
   avg quiet 0.372 vs 0.528,
   worst quiet 0.50 vs 0.80.
  With margin 0.4:
   43 clamps (equal),
   avg 0.286,
   worst 0.40.
- Gotcha for the answer:
   better sampling with an unchanged margin makes average
  quiet WORSE (probe closer to truth means the fixed margin over-attenuates
  more);
   sampling gains must be taken by lowering the margin.
- Album prior measured (analysis/album-prior.mjs,
   commit 9ee84f77b):
   244 groups
  with >= 3 loud long members cover 95.7% of loud long tracks;
   within-group
  spread median 0.92 dB below group max;
   probe picks the true loudest member
  66.8% of the time (mispicks mild,
   median 0.12 dB).
   But the cap policy costs
  ~36k s (all remaining headroom) for avg quiet 0.528 -> 0.362 at best,
   never
  improves worst quiet (0.8),
   and at allowance 0 nets +5 clamps.
   Zoom spends the
  same headroom strictly better (avg 0.285,
   worst 0.40,
   clamps equal).
   Under
  zoom's tight probes + small margin the cap almost never binds.
   Verdict:
  measured and declined;
   document in the answer.
- Encoding bones refuted (analysis/bones-*.mjs,
   correlate.mjs):
   all 2441 lossy
  loud long files parsed without decoding (opus TOC,
   m4a stsz/stts,
   mp3 frames;
  timelines within 0.15 s).
   The crest slot ranks at the 60th byte-rank
  percentile of its own track (median),
   worse than chance;
   bits follow spectral
  busyness,
   not peak height.
   Composed bones+zoom never beats zoom alone.
- Declared peaks verified useless:
   rgPeak understates the true crest by median
  1.92 dB (max 4.62) across the 723 tagged tracks.
- No-full-hearings variant measured (user suggestion):
   cutoff 0 s buys one
  clamp (25 vs 26 at margin 0.5) but worsens avg quiet 0.371 -> 0.385 and
  forfeits exact cached short-track crests;
   recorded in the answer,
   rule kept.
