# Translation repair MiMo trial: log from 2026-10-03 18:20 UTC to 2026-10-04 01:13 UTC

The trial log from 18:20 UTC on 2026-10-03 to 01:13 UTC on 2026-10-04:
the first whole-suite census and the first T8 batches,
on clusters of nine and ten uncovered stretches,
from the group cluster to the openrouter cluster.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-03, 18:20 UTC to 2026-10-04, 01:13 UTC: the first T8 batches

- 2026-10-04,
  01:13 UTC:
  the next batch's baseline census taken at commit `148979b36`:
  `census written to ~/.cache/translation-repair/coverage/census-xB1R6c/census.json`
  (`~/temp/agent/mimo-trial/census-11.log`).
  `library source: 178 files, 378 stretches over 891 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `synthetic` leads with 9 stretches over 9 lines in 2 files,
  `inspect` with 8 over 33 in 1,
  `corpus-run/tag` with 8 over 16 in 1.
  This line lands in the trial-log commit that follows `148979b36`.
  Next:
  `B123`,
  "Fourth" in this file,
  due once the first T8 batch landed and now overdue,
  then the `synthetic` batch.

- 2026-10-04,
  01:10 UTC:
  the T8 `openrouter` batch closed with commits `adcfeb5d4`,
  `b3cfd5637` and `780e4d267`:
  9 stretches over five source files.
  One is cased through the exported `rawCharsPerCompletionTokenOf` seam
  (its `unmeasured` arm reachable by shape,
  no roster card carrying it),
  one case pinning the fallback to the median of the measured card ratios.
  Three are the wire readers' refusals,
  cased in their own files.
  Five are the client's:
  the armed deadline,
  its joined signal and the caller `maxAnswerChars` spread cased together,
  and the cost,
  cached-token and endpoint spreads folded behind the exported
  `reportedSpendFieldsOf`,
  one case pinning the field combinations
  beside a meter pair
  (a bare stream leaves the meter flat,
  a costed one moves it 0.5).
  The first reach census settled three more
  (the chunk record guard behind a `startsWith('{')` gate that came out,
  the endpoint spread's unreported arm,
  the `/credits` non-success refusal),
  and two by-position comments and one statement TSDoc came out
  after the source scans flagged them.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and the three edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-openrouter2.log`).
  Counts at the close:
  the full suite 1,548 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-openrouter-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-openrouter-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `780e4d267`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `synthetic` cluster
  (the ranking's next after `openrouter` at `census-XzBVZf`).

- 2026-10-03,
  23:59 UTC:
  the `openrouter` batch is in progress,
  this mid-batch line written after the correction that its state
  lived only in working memory.
  Written but uncommitted in the worktree:
  two cases in `openrouter-client.unit.test.ts`
  (a caller `exchangeTimeoutMs` and `maxAnswerChars` armed on the client
  and read back under both,
  and `costUsd` and `cachedTokens` carried only where the wire sent them,
  both built on `chunkOf` streams),
  one in `openrouter-cached-tokens.unit.test.ts`
  (a non-record `data:` chunk and a fractional `cached_tokens`
  both read as `unreported`),
  one in `openrouter-cost.unit.test.ts`
  (a non-finite `cost` read as `unreported`).
  Decided,
  not yet written:
  `openrouter-abandoned-spend.ts:48`,
  the `rawCharsPerToken === 'unmeasured'` ternary arm,
  is reachable by shape
  (the card side types `rawCharsPerToken` as `number | 'unmeasured'`
  and `corpus-run/roster-card-render.ts` generates that spelling)
  and cold only because no roster card carries it,
  so it gets a narrow exported seam
  (the shape the `fidelity` batch gave `needle` and `anchor`)
  plus a case in `openrouter-abandoned-spend.unit.test.ts`,
  the seam exported through `provider-barrel.ts`.
  Remaining:
  the seam and its case,
  build,
  the named test files,
  lint,
  the code commit and its line here,
  the reach census against `census-XzBVZf`,
  the full suite,
  the source scans,
  the ledger T8 paragraph,
  and a fresh whole-suite census named in the line after.
  This line lands in the trial-log commit that adds it.

- 2026-10-03,
  23:07 UTC:
  the next batch's baseline census taken at commit `51f394046`:
  `census written to ~/.cache/translation-repair/coverage/census-XzBVZf/census.json`
  (`~/temp/agent/mimo-trial/census-9.log`).
  `library source: 183 files, 387 stretches over 909 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `openrouter` leads with 9 stretches over 18 lines in 5 files,
  `synthetic` with 9 over 9 in 2,
  `inspect` with 8 over 33 in 1.
  This line lands in the trial-log commit that follows `51f394046`.
  Next:
  the `openrouter` batch.

- 2026-10-03,
  23:03 UTC:
  the T8 `decision` batch closed with commits `045c0e2f5` and `750ec0708`:
  9 stretches over the reply contract and the over-context gate,
  8 cased (the answer guard,
  the reply reader's refusals and shaped returns,
  the status refusal),
  1 replaced by the house cause shape
  (the parse catch's dead non-Error arm).
  `decision-contract.unit.test.ts` is new
  and the module leaves the own-unit-tests allowlist.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-decision3.log`).
  Counts at the close:
  the full suite 1,546 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-decision-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `750ec0708`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `openrouter` cluster.

- 2026-10-03,
  22:39 UTC:
  the next batch's baseline census taken at commit `d017074e2`:
  `census written to ~/.cache/translation-repair/coverage/census-bR9IAH/census.json`
  (`~/temp/agent/mimo-trial/census-8.log`).
  `library source: 185 files, 396 stretches over 928 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `decision` leads with 9 stretches over 19 lines in 2 files,
  `openrouter` with 9 over 18 in 5,
  `synthetic` with 9 over 9 in 2.
  This line lands in the trial-log commit that follows `d017074e2`.
  Next:
  the `decision` batch.

- 2026-10-03,
  22:35 UTC:
  the T8 `fidelity` batch closed with commit `e9d07984e`:
  9 stretches over the two damage builders,
  2 dropped as unreachable,
  7 cased in `fidelity-alteration-variant.unit.test.ts` and `fidelity-damage.unit.test.ts`
  through an optional `needle` and `anchor` a case can supply.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and both edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-fidelity.log`).
  Counts at the close:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-fidelity-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `e9d07984e`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `decision` cluster.

- 2026-10-03,
  22:12 UTC:
  the next batch's baseline census taken at commit `911154fe8`:
  `census written to ~/.cache/translation-repair/coverage/census-Ai2qpF/census.json`
  (`~/temp/agent/mimo-trial/census-7.log`).
  `library source: 187 files, 405 stretches over 952 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `fidelity` leads with 9 stretches over 24 lines in 2 files,
  `decision` with 9 over 19 in 2,
  `openrouter` with 9 over 18 in 5.
  This line lands in the trial-log commit that follows `911154fe8`.
  Next:
  the `fidelity` batch.

- 2026-10-03,
  22:07 UTC:
  the T8 `corpus-run/meter` batch closed with commit `729931a40`:
  10 stretches over the two meter readers,
  3 `??` fallbacks replaced with `nonNullishOrThrow`,
  7 cased through `seriesFor`,
  `longestDrySpan` and `readMeterLine`.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and both edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-meter.log`).
  Counts at the close:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-meter-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `729931a40`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `fidelity` cluster.

- 2026-10-03,
  21:50 UTC:
  the next batch's baseline census taken at commit `dae4b8eea`:
  `census written to ~/.cache/translation-repair/coverage/census-DbGZ25/census.json`
  (`~/temp/agent/mimo-trial/census-6.log`).
  `library source: 189 files, 415 stretches over 967 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/meter` leads with 10 stretches over 15 lines in 2 files,
  `fidelity` with 9 over 24 in 2,
  `decision` with 9 over 19 in 2.
  This line lands in the trial-log commit that follows `dae4b8eea`.
  Next:
  the `corpus-run/meter` batch.

- 2026-10-03,
  21:45 UTC:
  the T8 `corpus-run/final` batch closed with commits `e0bf15a10` and `3da0a9a3b`:
  10 stretches over the two completeness checkers,
  11 cases in `consolidation-polish.unit.test.ts` and its own test file,
  and the polish branch's consolidation narrowing hoisted before the loop
  so both its arms run.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-final2.log`).
  Counts at the close:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-final-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `3da0a9a3b`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/meter` cluster.

- 2026-10-03,
  21:10 UTC:
  the next batch's baseline census taken at commit `8fa092e8b`:
  `census written to ~/.cache/translation-repair/coverage/census-7Aczvk/census.json`
  (`~/temp/agent/mimo-trial/census-5.log`).
  `library source: 191 files, 425 stretches over 984 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/final` leads with 10 stretches over 17 lines in 2 files,
  `corpus-run/meter` with 10 over 15 in 2,
  `fidelity` with 9 over 24 in 2.
  This line lands in the trial-log commit that follows `8fa092e8b`.
  Next:
  the `corpus-run/final` batch.

- 2026-10-03,
  21:05 UTC:
  the T8 `rendering` batch closed with commits `18f0b3a85` and `0bea2bc58`:
  10 baseline stretches and 2 more the per-source reading found,
  3 replaced with `nonNullishOrThrow`,
  9 cased through `corroborate`,
  `corroborateByOverlap`,
  `nearMisses`,
  the exported `quotesRequired`
  and `anchorLocatedSpan`'s new `fold` seam.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and every edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-rendering2.log`).
  Counts at the close:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-rendering-final-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `0bea2bc58`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/final` cluster.

- 2026-10-03,
  20:18 UTC:
  the next batch's baseline census taken at commit `90b76fe15`:
  `census written to ~/.cache/translation-repair/coverage/census-MgQUMe/census.json`
  (`~/temp/agent/mimo-trial/census-4.log`).
  `library source: 194 files, 435 stretches over 1007 lines, 10 functions never called`.
  Largest clusters by stretches then lines:
  `rendering` leads with 10 stretches over 23 lines in 3 files,
  `corpus-run/final` with 10 over 17 in 2,
  `corpus-run/meter` with 10 over 15 in 2,
  `fidelity` with 9 over 24 in 2.
  This line lands in the trial-log commit that follows `90b76fe15`.
  Next:
  the `rendering` batch.

- 2026-10-03,
  20:13 UTC:
  the T8 `inline` batch closed with commit `eef1e1610`
  (`test(module-translation-repair): T8 inline, case the tag reader's refusals and its quoted spans`):
  all 10 stretches of `src/inline-container-tags.ts` cased through `containerHalfPairs` and `readSliceSkeleton`,
  and the reach census reads `still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-inline.log`).
  Counts at the close:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-inline-final-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `eef1e1610`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `rendering` cluster.

- 2026-10-03,
  19:35 UTC:
  the next batch's baseline census taken at commit `ae88792c7`:
  `census written to ~/.cache/translation-repair/coverage/census-DXY8l2/census.json`
  (`~/temp/agent/mimo-trial/census-3.log`).
  `library source: 195 files, 447 stretches over 1037 lines, 11 functions never called`,
  one stretch below `census-QFKBeX` with the `group` cluster now empty of them.
  The `inline` cluster holds 10,
  all in `src/inline-container-tags.ts`.
  This line lands in the trial-log commit that follows `ae88792c7`.
  Next:
  the `inline` batch.

- 2026-10-03,
  19:30 UTC:
  the fresh census caught the `unreachable:` throw's own line as a stretch
  and the guard came out with commit `c592bcc0f`
  (`fix(module-translation-repair): T8 group, remove the no-boundary guard instead of throwing from it`):
  `census-QFKBeX` listed `src/group-run-anchor.ts` line 172,
  an unreachable throw is unreachable code too,
  so the dead arm is removed
  and its invariant stands in the comment above `settled`.
  M112 records it
  and the T8 group paragraph's correction names itself.
  Counts:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-group-throw-fix-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL],
  Markdown lint clean in the ledger.
  This line lands in the trial-log commit that follows `c592bcc0f`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `inline` cluster.

- 2026-10-03,
  19:17 UTC:
  the T8 `group` batch closed:
  commit `691e70e3b` landed its last case
  (`test(module-translation-repair): T8 group, case the continuation riding with its seal`),
  written for `src/group-aligned.ts` lines 377 to 383,
  and the reach census reads `still cold 0, cold since then 0, not loaded 0`
  over the three claimed sources and three test files
  (`~/temp/agent/mimo-trial/reach-group.log`).
  Counts at the close:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-group-final-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `691e70e3b`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `inline` cluster
  (10 stretches in one file).

- 2026-10-03,
  19:00 UTC:
  the T8 `group` batch's first case landed with commit `aa5f63aec`
  (`test(module-translation-repair): T8 group, case the run split at the step after a decline`),
  written for `src/group-aligned.ts` lines 195 to 198:
  a pairing placing every original with the middle translation declined,
  asserting the two runs it splits into.
  Counts:
  lint "Found 0 warnings and 0 errors.",
  the named file no [FAIL] and 2 [PASS]
  (`~/temp/agent/mimo-trial/t8-group-case-a2.log`),
  35 source scans and no [FAIL].
  The cluster's state:
  6 of 10 stretches closed as unreachable in `92c912b08`,
  1 cased here,
  and `src/group-aligned.ts` lines 377 to 383 (the sealed cohesive continuation arm)
  resist placement behind `mergeOneSidedRuns`' kind mapping:
  probe that mapping before deciding case or unreachable.
  This line lands in the trial-log commit that follows `aa5f63aec`.
  Next:
  place the two sealed-arm stretches,
  then the batch's reach census against `census-WkdwRV`
  (`--baseline` with one `--source` per file the batch claims),
  the full suite,
  lint,
  the scans,
  the T8 paragraph and this line,
  and a whole-suite census as the next batch's baseline.

- 2026-10-03,
  18:48 UTC:
  the T8 `group` cluster's six unreachable stretches closed with commit `92c912b08`
  (`fix(module-translation-repair): T8 group, drop the unreachable anchor fallbacks`):
  three `?? NO_BOUNDARY` fallbacks,
  one undefined-step arm
  and one dead `? []` arm removed,
  and one no-boundary arm replaced with an `unreachable:` throw
  after four probes on the built package never built the run list it guards.
  Counts:
  the full suite 1,545 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-group-unreachable-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL].
  This line lands in the trial-log commit that follows `92c912b08`.
  Next:
  the `group` cluster's two reachable stretches in `group-aligned.ts`
  (the position after a decline and the sealed cohesive continuation),
  then the batch's reach census.

- 2026-10-03,
  18:20 UTC:
  "Second:
  the whole-suite census" taken at commit `6d5f13a8b`,
  `census written to ~/.cache/translation-repair/coverage/census-WkdwRV/census.json`
  (`~/temp/agent/mimo-trial/census-base.log`).
  `library source: 198 files, 457 stretches over 1061 lines, 11 functions never called`.
  Largest clusters by stretches then lines:
  `group` leads with 10 stretches over 24 lines in 3 files,
  `inline` with 10 over 23 in 1,
  `rendering` with 10 over 23 in 3,
  `corpus-run/final` with 10 over 17 in 2,
  `corpus-run/meter` with 10 over 15 in 2,
  `fidelity` with 9 over 24 in 2,
  `decision` with 9 over 19 in 2,
  `openrouter` with 9 over 18 in 5.
  No cluster is near the split-by-file size.
  This line lands in the trial-log commit that follows `6d5f13a8b`.
  Next:
  "Third:
  T8 batches,
  one cluster at a time",
  the `group` cluster first.
