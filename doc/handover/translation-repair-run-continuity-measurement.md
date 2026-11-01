# Translation repair run continuity: restart measurement

Part of the [run continuity index](translation-repair-run-continuity.md).

## Current stop condition

Historical evidence only.
No continuity supervisor or corpus pass is running,
and this file does not authorize recreating one.

## Setting up run 2

STARTED 2026-08-22 06:51Z,
after run 1 settled all six entries in 18932.72s.

### Run 1 finished clean

`DONE processed=6 of pending=6; artifacts=6/92`.
Final per-entry consolidation counts,
which supersede the four-entry list recorded earlier:

- `Acheron`:
   27 settle,
   2 produce,
   30 tagged.
- `Weideriche_`:
   19,
   1,
   21.
- `Zha_Ke`:
   12,
   1,
   14.
- `gaoyanger`:
   9,
   2,
   12.
- `keyword233`:
   19,
   0,
   20.
- `lintong`:
   21,
   3,
   25.

Those sum to 122,
which is the log's total,
so the split is a closed accounting rather than a filter that happened to match.

### Two things had to change before run 2 could run at all

THE PASS SKIPS ENTRIES THAT ALREADY HAVE AN ARTIFACT,
which `corpus-pass.ts:48` states and `settledEntryIds` at `corpus-pass.ts:265`
 implements.
Pointing a second pass at the same directory would have reported `pending=0` and
 processed nothing.
Run 1's artifacts moved to `vub-run1-20260821/artifacts-run1`,
which also preserves them for the byte comparison the criterion needs.

THE LIVE CACHE WAS EMPTY.
`discardSliceCache` deletes each entry's cache as it settles,
and all six settled,
so `vub-run1-20260821/slice-cache` held zero files.
The 107 files under `~/temp/agent/vub-cache-capture` were the only surviving
 copy,
and they were copied back in.
Per entry the restored counts are
 `Acheron` 22,
`Zha_Ke` 23,
`lintong` 18,
`Weideriche_` 16,
`gaoyanger` 14 and
`keyword233` 14.

### The digest held

Run 2's `START` line carries the same
 `pipeline=sha256-tree-v1:2384524b15c2482c37db147b9654b0036eeebfba7e24b6297854d7bcddef4cc0`
 run 1 carried.
Had any file under `src` changed,
every cache namespace would have moved and the restore would have bought
 nothing.
That is what the source freeze was for.
THE FREEZE STILL HOLDS UNTIL RUN 2 FINISHES.
Its cache-key job is done,
because the digest was read at startup and matched,
but run 2 is executing `dist/final/node/corpus-pass.mjs`,
and editing `src` triggers a rebuild that would overwrite `dist` under a live
 pass.

`START files=96 pending=6 done=0` confirms all six were found unsettled,
and the first `SLICE-COST` lines report `exit=resumed`,
so the restored cache is being read rather than ignored.

### How to read the result

The log is `~/temp/agent/vub-run2.log`.
Run 2's artifacts land in `vub-run1-20260821/artifacts`,
beside run 1's in `artifacts-run1`.
No capture poller runs for run 2,
because the criterion reads the log and the artifacts rather than the cache.

## Run 2's first entry, and what it decided

`Acheron` settled in run 2 at 07:24Z,
33 minutes after the pass started,
against 5.3 hours for all six in run 1.

### The consolidation resume works

Slice 2 reproduced byte for byte,
350 characters in both runs.
Slice 3 reproduced as `no-standing-text` with nothing shipped.
Slice 0 was allowed to differ and did,
moving from `slate-kept-standing` to `gate-kept-standing` while shipping the
 same standing text,
which is the refused-persistence path re-executing and landing on a real vote.

Slice 1 differed,
181 characters against 195,
and the criterion called that a failure.
It is not one.
`consolidate-key.ts:108` puts `repairText` in the consolidation cache key,
so a slice whose repair candidate changed has a different key by construction
 and must re-buy.
The consolidation resumed wherever its input was stable and re-bought wherever
 its input moved,
which is what a correct cache does.

### The repair lane does not reproduce, and that is the finding

Every one of the eight lane-slices reported `exit=resumed` with `ms=0`,
so nothing was re-bought at the lane level.
The delivered text still moved.

The restored cache holds repair `repairedText` lengths of
 255,
340,
371 and 251.
Run 1's own artifact recorded `comparison[].repairText` of
 242,
344,
371 and 251.
Run 2 recorded
 252,
326,
371 and 251.

Chunks carrying 371 and 251 agree across all three readings.
The other two disagree in all three,
and 255 and 340 appear in neither artifact.
Those are multisets,
so the `chunkIndex` ambiguity `#99` records cannot explain the gap:
no remapping makes 255 and 340 into 242 and 344.

So run 1's artifact already disagreed with run 1's own cache,
and run 2 disagreed with both.

### What this rules in and out

The lane contest is not the cause.
Every verdict is identical across the runs,
at the same `usable` of 6,
and the translate lane reproduced its text exactly on all four slices.

The mechanism between the cached `repairedText` and the delivered `repairText`
 is not yet identified,
and naming it is the next step rather than a guess to record here.
What is measured is that `exit=resumed` is not evidence that a slice reproduced
 what it cached.

### What this does to the criterion

The per-slice criterion recorded earlier reads a difference at a MUST slice as a
 broken resume.
That inference does not hold,
because a legitimate upstream change moves the key.
The criterion needs to compare against the CACHE rather than against run 1's
 artifact,
since the artifact is the thing now known not to match it.

### The delivered text is not the cached text, measured by overlap

Comparing each cached `repairedText` against the `shippedText` the same run
 delivered,
by common prefix and common suffix,
which needs no passage quoted.

- chunk 2 is identical in both runs at 371 characters.
- chunk 3 is the same length in both runs and NOT identical,
   sharing a 14 character prefix and a 209 character suffix.
   Both runs give the same two numbers,
   so whatever rewrites the middle there is deterministic.
- chunk 0 shares a 62 character prefix in both runs,
   and a suffix of 32 in run 1 against 38 in run 2.
- chunk 1 shares a 39 character suffix in both,
   and a prefix of 222 in run 1 against 82 in run 2.

MOST OF THAT IS THE SEMANTIC WRAP,
which `repair-assemble.ts` applies at assembly and which is length preserving,
so it is benign and expected.
Normalising whitespace on both sides separates it from a real change.
Of 14 readings across the two entries,
11 are wrap-only.
Three carry a genuine text difference:
`Acheron` chunk 0 and chunk 1 in both runs,
and `Weideriche_` chunk 1 in run 1 but NOT in run 2,
where it came back wrap-only.

So which slices diverge from their cache is itself unstable between runs.

### The fact that carries the finding

Chunk 0's cached record says `changed` is false,
carries zero `repairRegions`,
and records no rounds.
The repair lane did nothing there.
Its `repairedText` is 255 characters,
the same length as the incumbent.

Both runs nevertheless recorded `delivery=replacement-shipped` for that chunk,
shipping 242 characters in run 1 and 252 in run 2.
Neither is the incumbent and neither is the cached text.

A slice whose own record says it changed nothing cannot have produced that
 delivery,
so the delivered text does not come from the cached record.
That holds without naming the stage that does produce it,
which is the next thing to find and is deliberately not guessed here.

### The positive control that makes chunk 0 an anomaly

`Weideriche_` chunk 2 has the same cache state as `Acheron` chunk 0:
`changed` false,
zero repair regions.
It delivered `incumbent-retained`,
with the shipped text identical to the cached text,
in both runs.

That is the correct handling of a slice the lane did not change,
and it proves the pipeline can produce it.

`Acheron` chunk 0 has that same cache state and delivered
 `replacement-shipped` instead,
with text that is neither the incumbent nor the cached text,
and that differs between the two runs.
Two slices with the same recorded lane outcome took different delivery paths.

### What run 2 has decided so far

The consolidation resume works.
The semantic wrap accounts for 11 of the 14 cache-to-delivery comparisons.
Three readings carry a real divergence,
and one of those appeared in run 1 and not in run 2.
A slice recorded as changing nothing shipped a replacement.

`#171` carries this.
It outranks the frozen queue,
because a rerun over an unchanged corpus publishing different text is a
 correctness problem on a memorial corpus rather than a performance one.

## Run 2 finished, and the verification's answer

`DONE processed=6 of pending=6` in 4825 seconds,
against 18933 for run 1.
All 36 lane-slices reported `exit=resumed`,
none `computed`.

### The resume works mechanically

The cache is doing its job.
Run 2 took 80 minutes where run 1 took 5.3 hours,
every lane-slice resumed,
and consolidation work fell on five of six entries:
`Acheron` 30 to 24,
`Weideriche_` 21 to 11,
`gaoyanger` 12 to 9,
`keyword233` 20 to 11,
`lintong` 25 to 12.
`Zha_Ke` rose,
14 to 30,
for a reason the next section gives.

### The pipeline does not publish the same text twice

This is the finding,
and it is what verifying at the user boundary was for.

Comparing run 1's artifacts against run 2's,
on the same corpus,
under the same pipeline digest,
from the same restored cache:

- Repair lane,
   18 slices:
   11 published identical text,
   0 differed only by wrapping,
   7 published DIFFERENT TEXT.
- Consolidation,
   17 slices:
   11 identical,
   0 wrap-only,
   6 published DIFFERENT TEXT,
   and 3 changed terminal.

The repair-lane divergences are
 `Acheron` chunks 0 and 1,
`Weideriche_` chunk 1,
`Zha_Ke` chunk 0,
chunk 1 and chunk 3,
and `gaoyanger` chunk 1.
Two entries,
`keyword233` and `lintong`,
reproduced their repair lane exactly.

### The largest divergence

`Zha_Ke` chunks 0 and 3 settled `no-standing-text` in run 1,
shipping nothing at all,
and settled `consolidated` in run 2,
shipping 162 and 278 characters.

Text appeared where a previous run published none.
That is also why `Zha_Ke` bought MORE consolidation work in run 2:
two slices that had no standing text to consolidate acquired some.

### The consolidation contributes its own share

`keyword233` chunk 1 published 419 characters in run 1 and 411 in run 2,
while its repair lane reproduced exactly between the runs.
So the divergence there did not come from upstream.
`consolidate-key.ts` puts `ballots` in the consolidation key,
and ballots come from the contest,
so a contest that answered differently moves the consolidation key even when
 the lane text is stable.

### What this settles

The question run 2 was built to answer was whether the resume path works.
It does.
The question it actually answered is larger:
the pipeline is not reproducible,
and the resume signal does not indicate reproduction.

`#171` carries this.
It outranks the frozen queue.

## The cause: the slice cache stops before the naturalness lane

FOUND 2026-08-22,
from source and confirmed by measurement.

### The structure

`repair-translation.ts` threads `sliceCache` through the ACCURACY pass only.
It resumes at `repair-translation.ts:330` and persists at
 `repair-translation.ts:454`.

`refineSettledSlices` is then called at `repair-translation.ts:535`,
over every slice the accuracy pass settled,
under a comment saying it runs after every accuracy outcome settled and before
 anything reads `changed`.
Neither `repair-refine-step.ts` nor `refine-phase.ts` reads or writes the slice
 cache.

So a resumed run replays the accuracy pass from disk and then BUYS THE
 NATURALNESS LANE AGAIN,
with fresh model calls,
every time.
That is also why run 2 ran a checker stage four times while every slice reported
 `exit=resumed`:
`refine-phase.ts:441` has its own checker.

### The confirmation

Every cached envelope carries `refined` false,
because the cache is written before the naturalness lane runs.

Reading `lanes.repair.result.chunks[].refined` in the artifacts,
against whether a slice published different text in the two runs:

- diverged and refined:
   7.
- diverged and NOT refined:
   0.
- stable and refined:
   1.
- stable and NOT refined:
   10.

Every divergence is a refined slice,
and almost every stable slice was never refined.

The flags also show the mechanism directly.
`Weideriche_` chunk 1,
`Zha_Ke` chunk 0 and `Zha_Ke` chunk 3 were refined in run 1 and NOT refined in
 run 2.
The lane fired on those slices once and declined to the next,
which is why their text moved.

### What this explains that was previously unexplained

`Acheron` chunk 0's cached record says `changed` false with zero repair regions,
and it still shipped a replacement.
It was refined.
A refinement-only change is a real change to the page made by a slice the
 accuracy pass left alone,
and `repair-translation.ts:531` says so outright:
a refinement-only change reaches `changedOutcomes` and `anyChanged`.

`Zha_Ke` publishing 162 and 278 characters where run 1 published nothing follows
 the same way,
one stage further on.

### What a fix has to decide

This is not a bug in the naturalness lane.
The lane is doing what it was built to do.
What is missing is that its output is not part of the unit the cache stores,
so the cache cannot make a run reproducible.

The decision is whether the cached unit becomes the slice AFTER refinement,
or whether the refinement gets a cache of its own keyed on the accuracy result.
Both make a resumed run reproduce.
They differ in what a cache invalidation costs and in whether a refinement can
 be re-asked without re-buying the accuracy pass.

## The full accounting from cache to shipped text

MEASURED 2026-08-22 over all 18 repair-lane slices of the band pair,
comparing each captured slice-cache record against the text that lane
 delivered.
Two transforms separate them,
and together they explain every row with no residue.

### Refinement explains every divergent row and nothing else does

`refined=false` and "the cached text is what shipped" are the same fact on
 16 of 18 rows,
and the two remaining rows are explained in the next section.

The correlation carries its own positive control,
which is why it is worth more than a count.
Three slices changed their refinement answer between the runs:
`Zha_Ke` chunk 0,
`Zha_Ke` chunk 3,
and `Weideriche_` chunk 1 were refined in run 1 and not in run 2.
At all three the cache match flips in lockstep,
from mismatching in run 1 to matching in run 2.
A predictor that moves when the thing it predicts moves is not a coincidence
 of one sample.

`refine-phase.ts` sets `refined: true` on exactly one path,
`refine-phase.ts:352`,
the path where a rewrite both changed the text and kept every confirmed issue.
Every other path pushes the accuracy outcome unmodified.
So `refined=false` is a positive claim that the accuracy text is what shipped,
which is what makes the correlation testable at all.

### The wrap is not length-preserving inside a blockquote

`lintong` chunks 1 and 2 were the only rows refinement did not explain.
They ship 4 and 12 characters more than the cache holds,
they are identical across both runs,
and neither was refined.

The cause is the semantic wrap.
Chunk 1 gains 2 blockquote markers and exactly 4 characters,
chunk 2 gains 6 markers and exactly 12.
When the wrap breaks a line that sits inside a blockquote,
the new line needs its own `> ` prefix,
so two characters appear per inserted break.

Stripping blockquote markers and collapsing whitespace makes the cached text
 and the shipped text equal on 10 of 10 unrefined rows,
with no exceptions.

This corrects a claim recorded earlier in this file and in
 `doc/planning/the-third-rendering.md`,
that the wrap is length-preserving because it only exchanges a space for a
 newline.
That holds in running prose and fails inside a blockquote.
`#167` is where the wrap's treatment of line-structured slices is decided,
and this belongs to it.

### Why this matters for the fix

Once refinement is cached,
cache to shipped becomes a pure deterministic function.
The reproduction criterion can therefore be stated exactly:
a resumed slice must ship text equal to its cached text after blockquote
 markers are normalised,
and any other difference is a defect.

Before this measurement the criterion could not be stated,
because two unexplained rows would have failed it.

## Correction: the consolidation divergence was an incomplete capture

The section titled "The consolidation contributes its own share" names the
 wrong mechanism.
It attributes `keyword233` chunk 1 publishing 419 characters in run 1 and 411
 in run 2 to ballots moving the consolidation key.
That mechanism was never measured.

What the capture actually holds settles it.
`keyword233` has one captured consolidation envelope,
a `gate-kept-standing` terminal carrying 212 characters,
which is chunk 0's length.
Chunk 1's settlement was never captured,
so run 2 had nothing to resume and bought a fresh one.

This is not confined to one entry.
Every entry in the band pair is missing at least one consolidation envelope:
`keyword233` 1 of 2,
`gaoyanger` 1 of 2,
`lintong` 2 of 3,
`Weideriche_` 1 of 3,
`Zha_Ke` 3 of 4,
`Acheron` 2 of 4.
The consolidation settles last per slice,
and the poller loses the last file it has not yet copied,
so the stage that settles last is the stage the capture is worst at holding.

### What still stands and what does not

A consolidation bought fresh produces different text between runs.
That is expected of fresh model calls and is not evidence of a defect.

A consolidation RESUMED from cache reproduces byte for byte.
That was verified separately on `Acheron` slice 2 through the validating
 store,
and `Acheron` slice 1 correctly re-bought because `consolidate-key.ts:108`
 puts `repairText` in the key and the repair text had moved.

So the consolidation resume path is clean,
and run 2 exercised it far less than the run's own numbers suggest.
The repair lane's refinement remains the only measured source of
 non-determinism on a resumed run.
