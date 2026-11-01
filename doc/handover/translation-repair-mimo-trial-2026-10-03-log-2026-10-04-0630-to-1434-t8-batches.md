# Translation repair MiMo trial: log of 2026-10-04 from 06:30 to 14:34 UTC

The trial log from 06:30 to 14:34 UTC on 2026-10-04:
T8 batches on clusters of five,
six and seven uncovered stretches,
each batch a closing line and the baseline census that follows it.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-04, 06:30 to 14:34 UTC: T8 batches

- 2026-10-04,
  14:34 UTC:
  the T8 `grade` batch closed with commits `894ef4d0c`,
  `dd52f9ced`,
  `df2c9a7f4`:
  5 line stretches over `grade-agreement.ts`
  and `grade-sheet-read.ts`,
  one closed as unreachable
  (the no-marker arm:
  the item filter settles it),
  four cased
  (the unbounded answer,
  the duplicate word,
  the non-string verdict refusal,
  and the duplicate indices).
  The reach census reads `left 0 cold stretches`
  (`~/temp/agent/mimo-trial/reach-grade3.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-grade-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-grade-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `df2c9a7f4`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `restoration` cluster
  (the ranking's next at `census-G4ZkGi`).

- 2026-10-04,
  14:19 UTC:
  the T8 `corpus-run/bench` batch closed with commit `2810a9104`:
  5 line stretches over `bench-report.ts`,
  two closed as unreachable
  (the middle-width fallback narrowed,
  the empty-rows note removed),
  two cased
  (the middle width repeat,
  the token sums).
  The reach census reads `left 1 cold stretch` in the focused run
  and none in the whole-suite census
  (`~/temp/agent/mimo-trial/reach-bench.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-model-suite.log` was the last full count;
  this batch's tests ran green at `~/temp/agent/mimo-trial/t8-bench-named7.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `2810a9104`.
  Next:
  the whole-suite census at this commit as the next batch's baseline:
  `census written to ~/.cache/translation-repair/coverage/census-G4ZkGi/census.json`
  (`~/temp/agent/mimo-trial/census-37.log`),
  `library source: 129 files, 228 stretches over 475 lines, 5 functions never called`
  (the never-called count moved again,
  8 to 5).
  Largest clusters by stretches then lines:
  `grade` leads with 5 stretches over 11 lines in 2 files,
  `restoration` with 5 over 8 in 2,
  `han` with 5 over 7 in 2.
  Next:
  the `grade` batch.

- 2026-10-04,
  14:00 UTC:
  the next batch's baseline census taken at commit `3970d3c0d`:
  `census written to ~/.cache/translation-repair/coverage/census-WGrBjo/census.json`
  (`~/temp/agent/mimo-trial/census-36.log`).
  `library source: 130 files, 233 stretches over 486 lines, 8 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/bench` leads with 5 stretches over 11 lines in 1 file,
  `grade` with 5 over 11 in 2,
  `restoration` with 5 over 8 in 2.
  This line lands in the trial-log commit that follows `3970d3c0d`.
  Next:
  the `corpus-run/bench` batch.

- 2026-10-04,
  13:52 UTC:
  the T8 `corpus-run/model` batch closed with commits `013e5d3ff`,
  `581363446`:
  5 line stretches over `model-catalog-compare.ts`,
  all cased
  (the two parse refusals,
  the claimed-underlying read under either spelling,
  and the report's blocklist,
  missing,
  alias
  and plain lines).
  The reach census reads `ran 5, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-model2.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-model-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-model-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `581363446`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/bench` cluster
  (the ranking's next at `census-WleNhT`).

- 2026-10-04,
  13:48 UTC:
  the next batch's baseline census taken at commit `64c05616e`:
  `census written to ~/.cache/translation-repair/coverage/census-WleNhT/census.json`
  (`~/temp/agent/mimo-trial/census-35.log`).
  `library source: 131 files, 238 stretches over 499 lines, 8 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/model` leads with 5 stretches over 13 lines in 1 file,
  `corpus-run/bench` with 5 over 11 in 1,
  `grade` with 5 over 11 in 2.
  This line lands in the trial-log commit that follows `64c05616e`.
  Next:
  the `corpus-run/model` batch.

- 2026-10-04,
  13:40 UTC:
  the T8 `stage` batch closed with commits `6d45ab0a1`,
  `4095f47e3`,
  `e4f718feb`:
  5 line stretches over the stage window,
  roster,
  fanout
  and decision files.
  Four cased
  (the voice-pair read,
  the decision guard and abort,
  and both optional-knob spreads),
  one removed as type-dead
  (the fanout content ternary's structured arm:
  `ChatMessage.content` is string-only).
  The reach census reads `ran 1, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-stage3.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-stage-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-stage-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `e4f718feb`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/model` cluster
  (the ranking's next at `census-1VGtcy`).

- 2026-10-04,
  13:26 UTC:
  the next batch's baseline census taken at commit `682015402`:
  `census written to ~/.cache/translation-repair/coverage/census-1VGtcy/census.json`
  (`~/temp/agent/mimo-trial/census-34.log`).
  `library source: 134 files, 242 stretches over 514 lines, 8 functions never called`.
  Largest clusters by stretches then lines:
  `stage` leads with 5 stretches over 16 lines in 4 files,
  `corpus-run/model` with 5 over 13 in 1,
  `corpus-run/bench` with 5 over 11 in 1.
  This line lands in the trial-log commit that follows `682015402`.
  Next:
  the `stage` batch.

- 2026-10-04,
  13:18 UTC:
  the T8 `corpus-run/probe` batch closed with commits `b37a5851f`,
  `0176fa22d`,
  `db84dbfa3`,
  `5d817c448`:
  5 line stretches over the probe telemetry and relabel files,
  all cased
  (the two telemetry notes,
  the manifest item no candidate carries,
  and the control's taken and holder refusals).
  The fixtures are a throwaway corpus clone,
  scratch-run entry artifacts,
  and a manifest through `buildSampleManifest`.
  Four source scans named the patterns the new cases must keep:
  `concurrency: 1` for process-global writes,
  the global-writer body groups (B111,
  no shared fixture),
  `scratchDir` for temp directories (B108),
  and the own-tests allowlist (B102).
  The reach census reads `ran 2, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-probe2.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-probe-suite5.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-probe-scans5.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `5d817c448`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `stage` cluster
  (the ranking's next at `census-7LEx8b`).

- 2026-10-04,
  12:31 UTC:
  the next batch's baseline census taken at commit `e9a2f06b1`:
  `census written to ~/.cache/translation-repair/coverage/census-7LEx8b/census.json`
  (`~/temp/agent/mimo-trial/census-33.log`).
  `library source: 137 files, 247 stretches over 531 lines, 8 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/probe` leads with 5 stretches over 17 lines in 3 files,
  `stage` with 5 over 16 in 4,
  `corpus-run/model` with 5 over 13 in 1.
  This line lands in the trial-log commit that follows `e9a2f06b1`.
  Next:
  the `corpus-run/probe` batch.

- 2026-10-04,
  12:23 UTC:
  the T8 `corpus-run/prose` batch closed with commits `55621aa6e`,
  `1c59e999d`,
  `943323375`,
  `794cd7764`:
  5 line stretches over `prose-ranges.ts`,
  all cased
  (the comment end reads,
  the quoting marks,
  the code fence,
  the lone backtick pair,
  the unterminated marker,
  and the bare autolink).
  The reach census over the corpus-run tests reads
  `ran 5, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-prose8.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-prose-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-prose-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `794cd7764`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/probe` cluster
  (the ranking's next at `census-OjpKcL`).

- 2026-10-04,
  12:12 UTC:
  the next batch's baseline census taken at commit `43c129ff2`:
  `census written to ~/.cache/translation-repair/coverage/census-OjpKcL/census.json`
  (`~/temp/agent/mimo-trial/census-32.log`).
  `library source: 138 files, 252 stretches over 548 lines, 8 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/prose` leads with 5 stretches over 17 lines in 1 file,
  `corpus-run/probe` with 5 over 17 in 3,
  `stage` with 5 over 16 in 4.
  This line lands in the trial-log commit that follows `43c129ff2`.
  Next:
  the `corpus-run/prose` batch.

- 2026-10-04,
  12:04 UTC:
  the T8 `preservation` batch closed with commit `05a02beab`:
  6 line stretches over `preservation-tokens.ts`
  and `preservation-check.ts`,
  the name scanner's five dead empty-string fallbacks narrowed away,
  one cased
  (the empty licensed quote that blanks nothing).
  The reach census reads `left 0 cold stretches`
  (`~/temp/agent/mimo-trial/reach-preservation.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-preservation-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-preservation-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `05a02beab`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/prose` cluster
  (the ranking's next at `census-g87Ji1`).

- 2026-10-04,
  12:02 UTC:
  the next batch's baseline census taken at commit `71e8d814d`
  (which followed a fix-forward `316f1afc7` corrected:
  that docs paragraph carried a position reference,
  caught by the census's own suite run):
  `census written to ~/.cache/translation-repair/coverage/census-g87Ji1/census.json`
  (`~/temp/agent/mimo-trial/census-31.log`).
  `library source: 140 files, 258 stretches over 554 lines, 8 functions never called`.
  Largest clusters by stretches then lines:
  `preservation` leads with 6 stretches over 6 lines in 2 files,
  `corpus-run/prose` with 5 over 17 in 1,
  `corpus-run/probe` with 5 over 17 in 3.
  This line lands in the trial-log commit that follows `71e8d814d`.
  Next:
  the `preservation` batch.

- 2026-10-04,
  11:52 UTC:
  the T8 `corpus-run/list` batch closed with commits `45e3356bc`,
  `99058e60a`:
  6 line stretches over `list-spread-restore.ts`,
  two closed as the impossible-index fallbacks they are
  (both `nonNullishOrThrow`),
  two cased
  (the single-item list and the count mismatch
  on the sides the walk actually compares),
  two left documented
  (the missing-position refusals,
  reached only where a parsed item carries no offsets).
  The reach census reads `left 2 cold stretches`,
  those two
  (`~/temp/agent/mimo-trial/reach-list2.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-list-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-list-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `99058e60a`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `preservation` cluster
  (the ranking's next at `census-RRFnX2`).

- 2026-10-04,
  11:38 UTC:
  the next batch's baseline census taken at commit `d5d3b32a7`:
  `census written to ~/.cache/translation-repair/coverage/census-RRFnX2/census.json`
  (`~/temp/agent/mimo-trial/census-28.log`).
  `library source: 140 files, 262 stretches over 558 lines, 8 functions never called`
  (the never-called count moved for the first time this trial,
  9 to 8,
  the consolidate batch's cases reaching one).
  Largest clusters by stretches then lines:
  `corpus-run/list` leads with 6 stretches over 6 lines in 1 file,
  `preservation` with 6 over 6 in 2,
  `corpus-run/prose` with 5 over 17 in 1.
  This line lands in the trial-log commit that follows `d5d3b32a7`.
  Next:
  the `corpus-run/list` batch.

- 2026-10-04,
  11:29 UTC:
  the T8 `corpus-run/consolidate` batch closed with commit `445e5b718`:
  6 line stretches over `consolidate-cache-store.ts`,
  all cased through the store's own round trip
  (the record and choice refusals,
  the absent gate beside the unreadable one,
  both shippings,
  and the non-record settlement).
  The reach census reads `ran 6, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-consolidate.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-consolidate-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-consolidate-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `445e5b718`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/list` cluster
  (the ranking's next at `census-xreqy1`).

- 2026-10-04,
  11:27 UTC:
  the next batch's baseline census taken at commit `c56975df4`:
  `census written to ~/.cache/translation-repair/coverage/census-xreqy1/census.json`
  (`~/temp/agent/mimo-trial/census-27.log`).
  `library source: 141 files, 268 stretches over 566 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/consolidate` leads with 6 stretches over 8 lines in 1 file,
  `corpus-run/list` with 6 over 6 in 1,
  `preservation` with 6 over 6 in 2.
  This line lands in the trial-log commit that follows `c56975df4`.
  Next:
  the `corpus-run/consolidate` batch.

- 2026-10-04,
  11:18 UTC:
  the T8 `corpus-run/directory` batch closed with commits `6ce30575c`
  and `f2a9a055b`:
  6 line stretches over `directory-id-name.ts`,
  all cased
  (the record and string refusals of the metadata reads,
  the alias shapes,
  the empty-letters id,
  and the id-namer).
  The reach census reads `ran 6, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-directory2.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-directory-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-directory-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `f2a9a055b`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/consolidate` cluster
  (the ranking's next at `census-eIzZ5P`).

- 2026-10-04,
  11:11 UTC:
  the next batch's baseline census taken at commit `b7e216487`:
  `census written to ~/.cache/translation-repair/coverage/census-eIzZ5P/census.json`
  (`~/temp/agent/mimo-trial/census-26.log`).
  `library source: 142 files, 274 stretches over 577 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/directory` leads with 6 stretches over 11 lines in 1 file,
  `corpus-run/consolidate` with 6 over 8 in 1,
  `corpus-run/list` with 6 over 6 in 1.
  This line lands in the trial-log commit that follows `b7e216487`.
  Next:
  the `corpus-run/directory` batch.

- 2026-10-04,
  11:02 UTC:
  the T8 `line` batch closed with commits `fe1f48671`,
  `76f87ac7b` and `5202dedc5`:
  6 line stretches over two files.
  Three carry cases
  (the pair in both language orders,
  the no-letter neighbour,
  and the page bound),
  and three read strictly behind their guarantees
  (the five-block floor,
  the loop bound),
  `nonNullishOrThrow` throughout.
  The reach census reads `ran 0, still cold 0, cold since then 0, not loaded 0`
  and the two edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-line2.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-line-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-line-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `5202dedc5`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/directory` cluster
  (the ranking's next at `census-8QcZ0L`).

- 2026-10-04,
  10:52 UTC:
  the next batch's baseline census taken at commit `41dd5bd89`:
  `census written to ~/.cache/translation-repair/coverage/census-8QcZ0L/census.json`
  (`~/temp/agent/mimo-trial/census-25.log`).
  `library source: 144 files, 280 stretches over 591 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `line` leads with 6 stretches over 14 lines in 2 files,
  `corpus-run/directory` with 6 over 11 in 1,
  `corpus-run/consolidate` with 6 over 8 in 1.
  This line lands in the trial-log commit that follows `41dd5bd89`.
  Next:
  the `line` batch.

- 2026-10-04,
  10:43 UTC:
  the T8 `sample` batch closed with commits `a23270119`,
  `c3c0ac7c2`,
  `1ecdd31b7` and `5dbdff239`:
  6 line stretches over three files.
  Four carry cases
  (the round-robin quota fill,
  the entry-shuffle tiebreak pinned in both arrival orders,
  the byte-count refusal and the no-claim placeholders,
  the repair context,
  and the unrecorded generation),
  and two settled behind their own contracts
  (the quota loop's stopgap break became a pass ceiling;
  the issue-key tiebreak was dead twice over,
  the entry-shuffle compare alone remaining).
  A new `sample-draw.unit.test.ts` leaves the own-unit-tests allowlist.
  The reach census reads `ran 4, still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-sample6.log`).
  Counts at the close:
  the full suite 1,558 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-sample-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-sample-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `5dbdff239`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `line` cluster
  (the ranking's next at `census-u4yVLi`).

- 2026-10-04,
  09:52 UTC:
  the next batch's baseline census taken at commit `d85e24703`:
  `census written to ~/.cache/translation-repair/coverage/census-u4yVLi/census.json`
  (`~/temp/agent/mimo-trial/census-24.log`).
  `library source: 147 files, 286 stretches over 614 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `sample` leads with 6 stretches over 23 lines in 3 files,
  `line` with 6 over 14 in 2,
  `corpus-run/directory` with 6 over 11 in 1.
  This line lands in the trial-log commit that follows `d85e24703`.
  Next:
  the `sample` batch.

- 2026-10-04,
  09:43 UTC:
  the T8 `corpus-run/rendering` batch closed with commits `bce653c57`,
  `b01ca171a` and `be9beb63c`:
  6 line stretches over four files.
  Four carry cases
  (the empty identity declaration,
  the missing-comparison-row invariant,
  the verify refusal over a tampered `sourceChars`,
  and the slot phrase in both its numbers),
  two settled behind their own contracts
  (the decided filter carries the narrowing as a type predicate;
  the cited-pages map is total over the subjects,
  its miss guard now `nonNullishOrThrow`).
  The reach census reads `ran 2, still cold 0, cold since then 0, not loaded 0`
  and the two edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-rendering2.log`).
  Counts at the close:
  the full suite 1,554 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-rendering-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-rendering-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `be9beb63c`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `sample` cluster
  (the ranking's next at `census-Vv2d8g`).

- 2026-10-04,
  09:12 UTC:
  the next batch's baseline census taken at commit `6c6d7203d`:
  `census written to ~/.cache/translation-repair/coverage/census-Vv2d8g/census.json`
  (`~/temp/agent/mimo-trial/census-23.log`).
  `library source: 151 files, 292 stretches over 639 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/rendering` leads with 6 stretches over 25 lines in 4 files,
  `sample` with 6 over 23 in 3,
  `line` with 6 over 14 in 2.
  This line lands in the trial-log commit that follows `6c6d7203d`.
  Next:
  the `corpus-run/rendering` batch.

- 2026-10-04,
  09:02 UTC:
  the T8 `corpus-run/coverage` batch closed with commits `c2c9c9689`
  and `db979e5b3`:
  6 stretches over `coverage-control.ts`,
  all cased in its vote-change driver file
  (the not-carried and evidence-not-locatable refusals,
  the no-room decoy riding its row,
  and the held pair through a quote-following client
  whose `none` answers carry the empty quote the wire requires).
  The reach census reads `ran 6, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-coveragecontrol.log`).
  Counts at the close:
  the full suite 1,554 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-coveragecontrol-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-coveragecontrol-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `db979e5b3`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/rendering` cluster
  (the ranking's next at `census-14CEcx`).

- 2026-10-04,
  08:45 UTC:
  the next batch's baseline census taken at commit `40afaf9b5`:
  `census written to ~/.cache/translation-repair/coverage/census-14CEcx/census.json`
  (`~/temp/agent/mimo-trial/census-22b.log`,
  its first attempt at `census-22.log` failing after the run finished
  with no census written and no suite fault,
  retry clean).
  `library source: 152 files, 298 stretches over 682 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/coverage` leads with 6 stretches over 43 lines in 1 file,
  `corpus-run/rendering` with 6 over 25 in 4,
  `sample` with 6 over 23 in 3.
  This line lands in the trial-log commit that follows `40afaf9b5`.
  Next:
  the `corpus-run/coverage` batch.

- 2026-10-04,
  08:32 UTC:
  the T8 `image` batch closed with commits `167332775`,
  `feada2f6e` and `480561e23`:
  6 stretches over three files.
  Five carry cases
  (the dotless name's empty extension,
  both decoder successes and the whole OCR path on tool-made fixtures,
  the deterministic count read both ways,
  and the reader-failure catch with its missing-binary half
  through the reader name now being a defaulted parameter).
  One roster guard came out behind the capability gate,
  the roster read now `nonNullishOrThrow`'s.
  The reach census reads `ran 1, still cold 0, cold since then 0, not loaded 0`
  and the two edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-image3.log`).
  Counts at the close:
  the full suite 1,554 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-image-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-image-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `480561e23`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/coverage` cluster
  (the ranking's next at `census-mgeQC0`).

- 2026-10-04,
  07:56 UTC:
  the next batch's baseline census taken at commit `67694f69d`:
  `census written to ~/.cache/translation-repair/coverage/census-mgeQC0/census.json`
  (`~/temp/agent/mimo-trial/census-21.log`).
  `library source: 155 files, 304 stretches over 772 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `image` leads with 6 stretches over 90 lines in 3 files,
  `corpus-run/coverage` with 6 over 43 in 1,
  `corpus-run/rendering` with 6 over 25 in 4.
  This line lands in the trial-log commit that follows `67694f69d`.
  Next:
  the `image` batch.

- 2026-10-04,
  07:46 UTC:
  the T8 `introduced` batch closed with commits `3af7885a8`,
  `74490e7dc` and `3094cc053`:
  7 stretches over two files,
  all cased and none unreachable.
  The wire guards' non-record and non-number refusals,
  the two neighbour fallbacks,
  the empty-quote dismissal
  (the case drives it through the damage gate,
  the only way an empty quote reaches the restates check),
  and the identity-context thread
  (its rules ride the system half,
  gated on a `DECLARED NAMES` entry).
  The reach census reads `ran 7, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-introduced3.log`).
  Counts at the close:
  the full suite 1,553 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-introduced-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-introduced-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `3094cc053`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `image` cluster
  (the ranking's next at `census-HKw9t2`).

- 2026-10-04,
  07:27 UTC:
  the next batch's baseline census taken at commit `b1f893810`:
  `census written to ~/.cache/translation-repair/coverage/census-HKw9t2/census.json`
  (`~/temp/agent/mimo-trial/census-20.log`).
  `library source: 157 files, 311 stretches over 779 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `introduced` leads with 7 stretches over 7 lines in 2 files,
  `image` with 6 over 90 in 3,
  `corpus-run/coverage` with 6 over 43 in 1.
  This line lands in the trial-log commit that follows `b1f893810`.
  Next:
  the `introduced` batch.

- 2026-10-04,
  07:16 UTC:
  the T8 `corpus-run/contributor` batch closed with commits `b2a0f5395`,
  `539a98d16` and `bcdd232b2`:
  7 stretches over two files.
  Five carry cases
  (the page-rendering fallback in a new authorities test file that leaves
  the own-unit-tests allowlist,
  the outright name match,
  and the heading whose original names no contributor),
  two identical-line unchanged arms came out unreachable behind
  `carriesRendering`,
  and the skip for an unwritten slice carries its case.
  The reach runs corrected three fixtures along the way
  (the authority derives from the signature;
  the archive rendering returns before the page read).
  A correction on `bcdd232b2`'s body:
  shell substitution ate the backticked phrase naming the two
  identical-line checks,
  leaving "The  arms sat unreachable";
  the phrase is `after === line` and the commit stands as written.
  The reach census reads `ran 1, still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-contributor5.log`).
  Counts at the close:
  the full suite 1,553 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-contributor-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-contributor-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `bcdd232b2`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `introduced` cluster
  (the ranking's next at `census-D1SlGz`).

- 2026-10-04,
  06:40 UTC:
  the next batch's baseline census taken at commit `d5e942e39`:
  `census written to ~/.cache/translation-repair/coverage/census-D1SlGz/census.json`
  (`~/temp/agent/mimo-trial/census-19.log`).
  `library source: 159 files, 318 stretches over 786 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/contributor` leads with 7 stretches over 7 lines in 2 files,
  `introduced` with 7 over 7 in 2,
  `image` with 6 over 90 in 3.
  This line lands in the trial-log commit that follows `d5e942e39`.
  Next:
  the `corpus-run/contributor` batch.

- 2026-10-04,
  06:30 UTC:
  the T8 `corpus-run/window` batch closed with commits `349a41a08`
  and `5997cef44`:
  7 line stretches over three files,
  the probe's three named ones the never-called functions the census
  books apart.
  Three carry cases
  (the draw-and-preparation disagreement,
  both incumbent-kind arms through a rendered slice beside an insertion
  one,
  a ledger line that parses as JSON but is no row),
  and the shipped sum's three index fallbacks came out unreachable behind
  its fixed arm list,
  now read through a named function.
  The reach census reads `ran 4, still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-window.log`).
  Counts at the close:
  the full suite 1,552 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-window-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-window-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `5997cef44`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/contributor` cluster
  (the ranking's next at `census-koUGtg`).
