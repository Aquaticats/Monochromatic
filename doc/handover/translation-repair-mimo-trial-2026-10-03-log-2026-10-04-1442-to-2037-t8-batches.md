# Translation repair MiMo trial: log of 2026-10-04 from 14:42 to 20:37 UTC

The trial log from 14:42 to 20:37 UTC on 2026-10-04:
T8 batches on clusters of four and five uncovered stretches,
each batch a closing line and the baseline census that follows it.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-04, 14:42 to 20:37 UTC: T8 batches

- 2026-10-04,
  20:28 UTC:
  the next batch's baseline census taken at commit `81727ed38`:
  `census written to ~/.cache/translation-repair/coverage/census-vlck42/census.json`
  (`~/temp/agent/mimo-trial/census-56.log`).
  `library source: 111 files, 182 stretches over 392 lines, 4 functions never called`.
  The ranking's first three clusters
  (`corpus-run/published`,
  `active`,
  `declined`)
  all carry documented-left arms.
  The next actionable cluster is `displacement`
  with 4 stretches over 5 lines in 1 file,
  then `corpus-run/name`,
  `corpus-run/canadian`,
  `mask`,
  `source`,
  each at 4 stretches.
  This line lands in the trial-log commit that follows `81727ed38`.
  Next:
  the `displacement` batch.

- 2026-10-04,
  20:37 UTC:
  the T8 `displacement` batch closed with commits `f7fa3843e`,
  `70ab0cae0`,
  `1160a11a8`:
  4 line stretches over `displacement-class.ts`,
  three closed as unreachable
  (the surplus,
  the target-only slice,
  and the class:
  each read through a fallback its own comment
  marks as known present),
  one cased
  (the conserved-fraction refusal).
  The reach census reads `left 0 cold stretches`
  (`~/temp/agent/mimo-trial/reach-displacement2.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-displacement-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-displacement-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `1160a11a8`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/name` cluster
  (the ranking's next at `census-vlck42`).

- 2026-10-04,
  20:20 UTC:
  the next batch's baseline census taken at commit `edd68a330`:
  `census written to ~/.cache/translation-repair/coverage/census-PGu68D/census.json`
  (`~/temp/agent/mimo-trial/census-55.log`).
  `library source: 111 files, 182 stretches over 392 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `active` with 4 over 8 in 1
  (its documented-left arms),
  `declined` with 4 over 6 in 1
  (`declined-target-runs.ts:109`,
  `158`,
  `193-194`,
  `195-196`).
  This line lands in the trial-log commit that follows `edd68a330`.
  Next:
  the `declined` batch.

- 2026-10-04,
  20:20 UTC:
  the T8 `declined` batch closed with commit `3326ef801`:
  4 line stretches over `declined-target-runs.ts`,
  the empty-pairing case landed,
  the four arms left documented
  (the rendering-continue mark,
  the empty-pairing refusal's own line,
  and the node-read ternary arms:
  their step shapes need alignments producing
  continuations the pairing fixtures do not stage).
  The reach census reads `ran 0, still cold 4`:
  those four
  (`~/temp/agent/mimo-trial/reach-declined.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-declined-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-declined-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `3326ef801`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the next cluster in the ranking
  (the remaining clusters all sit near 4 stretches each;
  the published and active ones carry documented-left arms).

- 2026-10-04,
  20:08 UTC:
  the next batch's baseline census taken at commit `0d76e5014`:
  `census written to ~/.cache/translation-repair/coverage/census-Kyrgyt/census.json`
  (`~/temp/agent/mimo-trial/census-54.log`).
  `library source: 111 files, 182 stretches over 392 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `active` with 4 over 8 in 1
  (`active-footnote-markers.ts:309-310`,
  `315-316`,
  `328-329`,
  `334-335`),
  `declined` with 4 over 6 in 1.
  This line lands in the trial-log commit that follows `0d76e5014`.
  Next:
  the `active` batch.

- 2026-10-04,
  20:12 UTC:
  the T8 `active` batch closed with commits `4a87b3658`,
  `6fc0fd2a4`:
  4 line stretches over `active-footnote-markers.ts`,
  the run-bounding case landed
  (the paragraph one untokenized literal),
  the four bounds arms left documented
  (the transform never produced a run whose parent
  holds only unpositioned children in these fixtures,
  and the walk is footnote-scoped).
  The reach census reads `ran 0, still cold 4`:
  those four
  (`~/temp/agent/mimo-trial/reach-active2.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-active-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-active-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `6fc0fd2a4`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `declined` cluster
  (the ranking's next at `census-Kyrgyt`).

- 2026-10-04,
  19:48 UTC:
  the next batch's baseline census taken at commit `870253616`:
  `census written to ~/.cache/translation-repair/coverage/census-KYvmuo/census.json`
  (`~/temp/agent/mimo-trial/census-53.log`).
  `library source: 111 files, 183 stretches over 394 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `transient` with 4 over 9 in 1
  (`transient-retry.ts:225`,
  `402-403`,
  `580-583`,
  `584-585`),
  `active` with 4 over 8 in 1.
  This line lands in the trial-log commit that follows `870253616`.
  Next:
  the `transient` batch.

- 2026-10-04,
  20:00 UTC:
  the T8 `transient` batch closed with commit `c1b403b95`:
  4 line stretches over `transient-retry.ts`,
  one cased
  (the non-Error throw wrapped into one),
  three left documented
  (the backoff's non-abort rethrow,
  the aborted-exchange pair,
  and the loop's exhausted throw).
  The commit message for `c1b403b95` names only two of
  the three left
  (the aborted-exchange pair went unnamed);
  recorded in the ledger rather than amended
  (ledger GCA).
  The reach census reads `ran 1, still cold 3`:
  those three
  (`~/temp/agent/mimo-trial/reach-transient.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-transient-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-transient-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `c1b403b95`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `active` cluster
  (the ranking's next at `census-KYvmuo`).

- 2026-10-04,
  19:18 UTC:
  the next batch's baseline census taken at commit `5840665be`:
  `census written to ~/.cache/translation-repair/coverage/census-pVW4Um/census.json`
  (`~/temp/agent/mimo-trial/census-52.log`).
  `library source: 111 files, 184 stretches over 395 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `corpus-run/runs` with 4 over 10 in 2
  (`runs-lock-holder.ts:198`,
  `463-468`,
  `runs-lock.ts:250-251`,
  `311`),
  `transient` with 4 over 9 in 1.
  This line lands in the trial-log commit that follows `5840665be`.
  Next:
  the `corpus-run/runs` batch.

- 2026-10-04,
  19:40 UTC:
  the T8 `corpus-run/runs` batch closed with commit `4e95c885d`:
  4 line stretches over `runs-lock-holder.ts`
  and `runs-lock.ts`,
  two cased
  (the unrecorded-identity lock line
  and the race loss's named holder),
  two left documented
  (the unrecorded identity construction,
  whose reads fail only where the host exposes none,
  and the race branch's holder spread,
  whose two-process race a single-process fixture
cannot stage without flakiness).
  The reach census reads `still cold 2`:
  those two
  (`~/temp/agent/mimo-trial/reach-runs.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-runs-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-runs-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `4e95c885d`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `transient` cluster
  (the ranking's next at `census-pVW4Um`).

- 2026-10-04,
  19:03 UTC:
  the next batch's baseline census taken at commit `6209604f3`:
  `census written to ~/.cache/translation-repair/coverage/census-cQ5IKo/census.json`
  (`~/temp/agent/mimo-trial/census-51.log`).
  `library source: 112 files, 186 stretches over 397 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `provider` with 4 over 10 in 2,
  `corpus-run/runs` with 4 over 10 in 2.
  This line lands in the trial-log commit that follows `6209604f3`.
  Next:
  the `provider` batch.

- 2026-10-04,
  19:11 UTC:
  the T8 `provider` batch closed with commit `f8598d546`:
  4 line stretches over `provider-budget-refusal.ts`
  and `provider-router.ts`,
  three cased
  (the two refusal reads
  and the last-attempt rethrow),
  the loop-exhausted throw its comment marks unreachable
  left as written.
  The reach census reads `ran 2, still cold 0, cold since then 0`
  (`~/temp/agent/mimo-trial/reach-provider.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-provider-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-provider-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `f8598d546`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/runs` cluster
  (the ranking's next at `census-cQ5IKo`).

- 2026-10-04,
  18:24 UTC:
  the next batch's baseline census taken at commit `4cbad5f31`
  (its first run failed on a flaky timeout in the lane contest's
  half-quorum case,
  green on re-run):
  `census written to ~/.cache/translation-repair/coverage/census-1UFhGm/census.json`
  (`~/temp/agent/mimo-trial/census-50.log`).
  `library source: 114 files, 190 stretches over 407 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `corpus-run/slice` with 4 over 10 in 2,
  `provider` with 4 over 10 in 2.
  This line lands in the trial-log commit that follows `4cbad5f31`.
  Next:
  the `corpus-run/slice` batch.

- 2026-10-04,
  18:56 UTC:
  the T8 `corpus-run/slice` batch closed with commits `5bf207307`,
  `3fdbf9b19`:
  4 line stretches over `slice-cache-namespace.ts`
  and `slice-census-entry.ts`,
  one closed as unreachable
  (the missing-node fallback narrowed),
  three cased
  (the recomputed cache file,
  the surfaced fault,
  and the target-only block count).
  The reach census reads `still cold 2`
  on `266`
  and `378`,
  both the single-line return and throw
  the census's attribution leaves cold
  (the reading the spend batch recorded),
  and both cases assert their behavior directly.
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-slice-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-slice-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `3fdbf9b19`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `provider` cluster
  (the ranking's next at `census-1UFhGm`).

- 2026-10-04,
  18:08 UTC:
  the next batch's baseline census taken at commit `aba6f1b61`:
  `census written to ~/.cache/translation-repair/coverage/census-c2UUBT/census.json`
  (`~/temp/agent/mimo-trial/census-48.log`).
  `library source: 116 files, 194 stretches over 419 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `corpus-run/handle` with 4 over 12 in 2,
  `corpus-run/slice` with 4 over 10 in 2.
  This line lands in the trial-log commit that follows `aba6f1b61`.
  Next:
  the `corpus-run/handle` batch.

- 2026-10-04,
  18:14 UTC:
  the T8 `corpus-run/handle` batch closed with commits `216e4986d`,
  `2db4930ee`:
  4 line stretches over `handle-gloss-place.ts`
  and `handle-reading.ts`,
  one closed as unreachable
  (the slice-text fallback narrowed),
  three cased
  (the unclosed and multiline parenthesis,
  the embedded handle,
  and the name-shaped rendering).
  The reach census reads `left 0 cold stretches`
  (`~/temp/agent/mimo-trial/reach-handle2.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-handle-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-handle-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `2db4930ee`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/slice` cluster
  (the ranking's next at `census-c2UUBT`).

- 2026-10-04,
  17:48 UTC:
  the next batch's baseline census taken at commit `3a433ae2e`:
  `census written to ~/.cache/translation-repair/coverage/census-IjEvWX/census.json`
  (`~/temp/agent/mimo-trial/census-47.log`).
  `library source: 117 files, 198 stretches over 432 lines, 4 functions never called`
  (the count crossed below 200).
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `corpus-run/roster` with 4 over 13 in 1,
  `corpus-run/handle` with 4 over 12 in 2.
  This line lands in the trial-log commit that follows `3a433ae2e`.
  Next:
  the `corpus-run/roster` batch.

- 2026-10-04,
  18:00 UTC:
  the T8 `corpus-run/roster` batch closed with commit `e1ee49e95`:
  4 line stretches over `roster-card-render.ts`,
  all cased
  (the body as the row array,
  the non-array modalities,
  the vision flag both ways,
  and the bedrock listing).
  The reach census reads `ran 4, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-roster.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-roster-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-roster-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `e1ee49e95`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/handle` cluster
  (the ranking's next at `census-IjEvWX`).

- 2026-10-04,
  16:31 UTC:
  the T8 `corpus-run/editor` batch is in progress:
  part 1 landed with commit `26b7b7909`
  (the standing reader names a missing run directory
  rather than reading an empty standing).
  Uncommitted work:
  none.
  The reach census reads `ran 1, still cold 12`
  on `editor-standing-read.ts`
  (`~/temp/agent/mimo-trial/reach-editor2.log`),
  the twelve being the artifact error paths
  and the reporter bodies still to case.
  Reasoning so far:
  the reader runs as a CLI over fixture run directories
  (`standingOver` spawns the built entry),
  so the error paths need artifact files carrying
  an off-roster model or no recorded rounds.
  Next:
  part 2,
  those two error paths.

- 2026-10-04,
  16:45 UTC:
  the `corpus-run/editor` batch part 2 preparation:
  the artifact schema walked field by field
  (`~/temp/agent/mimo-trial/artifact-schema-walk.mjs`
  iterates the CLI's refusals until parse),
  leaving the fixture spec:
  top keys `artifactSchemaVersion`,
  `id`,
  `tip`,
  `pipelineDigest`,
  `corpusSha`,
  `callConfig`,
  `durationMs`,
  `timestamp`,
  `preparation`,
  `lanes`,
  `comparison`,
  `laneSelection`,
  `consolidation`,
  `pageAssembly`;
  `preparation` carries `alignmentPairCount`,
  `identity`
  (`sha256-preparation-v2:`
  plus 64 hex),
  `sliceCount`,
  `sourceChars`,
  `targetChars`,
  `sourceBytes`,
  `alignmentFindings`;
  each lane is `{ result, delivery }`
  with `result.status` one of `repaired`,
  `unchanged`,
  `blocked-non-translation`
  and `delivery` one row per prepared slice
  (`chunkIndex`,
  `sourceText`,
  `incumbentKind`,
  `incumbentText`,
  `outcome`,
  `shippedText`,
  `delivery`).
  Uncommitted work:
  none.
  Next:
  write the off-roster and earlier-schema artifacts
  from that spec
  and case the two error paths.

- 2026-10-04,
  17:41 UTC:
  the T8 `corpus-run/editor` batch closed with commits `26b7b7909`,
  `df9436878`,
  `41726bb37`,
  `637c54309`,
  `5b0bf4d57`:
  the standing reader's 13 line stretches all cased
  (the missing run directory,
  the earlier-schema artifact,
  the off-roster artifact,
  the seated one read and reported,
  and the runs variable),
  plus the contest's none seat
  and the width report's neither-shipped row.
  The reach census reads `ran 13, still cold 0, cold since then 0`
  for the standing reader
  (`~/temp/agent/mimo-trial/reach-editor4.log`).
  Left documented:
  the width input's envelope and ready returns
  (`editor-width-input.ts:221-254`),
  whose claims pass the screen's own claim schema first.
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-editor-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-editor-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `5b0bf4d57`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/roster` cluster
  (the ranking's next at `census-fUfTA1`).

- 2026-10-04,
  16:26 UTC:
  the next batch's baseline census taken at commit `19f402d54`:
  `census written to ~/.cache/translation-repair/coverage/census-fUfTA1/census.json`
  (`~/temp/agent/mimo-trial/census-46.log`).
  `library source: 119 files, 201 stretches over 436 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `corpus-run/editor` with 4 over 38 in 3,
  `corpus-run/roster` with 4 over 13 in 1.
  This line lands in the trial-log commit that follows `19f402d54`.
  Next:
  the `corpus-run/editor` batch.

- 2026-10-04,
  16:20 UTC:
  the T8 `corpus-run/spend` batch closed with commit `07d5f9647`:
  5 line stretches over `spend-read.ts`,
  all cased
  (the not-a-record and unreadable refusals
  read off logged SPEND lines).
  The reach census reads `ran 3, still cold 2`
  on `238`
  and `345`,
  but a direct probe shows each arm's own value returning
  (`~/temp/agent/mimo-trial/spend-arm-probe.log`),
  so those two are the census's line attribution
  rather than unrun code.
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-spend-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-spend-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `07d5f9647`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/editor` cluster
  (the ranking's next at `census-1iMkkv`).

- 2026-10-04,
  16:10 UTC:
  the T8 `edit` batch closed with commit `527789637`:
  5 line stretches over `edit-prompt.ts`
  and `edit-wire.ts`,
  one closed as unreachable
  (the last-name fallback narrowed),
  three cased
  (the neighbouring block with either half alone,
  and the two wire refusals),
  one left documented
  (the single-name join:
  its names shrink below two only for an issue
  touching one identifier kind,
  whose fixtures are the deeper suite's).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-edit-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-edit-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `527789637`.
  Next:
  the whole-suite census at this commit as the next batch's baseline:
  `census written to ~/.cache/translation-repair/coverage/census-1iMkkv/census.json`
  (`~/temp/agent/mimo-trial/census-45.log`),
  `library source: 119 files, 204 stretches over 439 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its documented-left arms),
  `corpus-run/spend` with 5 over 5 in 1,
  `corpus-run/editor` with 4 over 38 in 3.
  Next:
  the `corpus-run/spend` batch.

- 2026-10-04,
  16:02 UTC:
  the next batch's baseline census taken at commit `222e844fc`:
  `census written to ~/.cache/translation-repair/coverage/census-i9Duoq/census.json`
  (`~/temp/agent/mimo-trial/census-44.log`).
  `library source: 120 files, 208 stretches over 443 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files
  (its five are the batch's documented-left arms,
  so the pass moves past it),
  `edit` with 5 over 6 in 2,
  `corpus-run/spend` with 5 over 5 in 1.
  This line lands in the trial-log commit that follows `222e844fc`.
  Next:
  the `edit` batch.

- 2026-10-04,
  15:55 UTC:
  the T8 `corpus-run/published` batch closed with commits `13a1780c9`,
  `2b3d73ec6`,
  `39b9c1be1`:
  5 line stretches over `published-page-disagreement.ts`
  and `published-page-check.ts`,
  the disagreement path and the weigh and wording checks driven
  over a slice-count mismatch and an anchor slice,
  four left documented
  (the three incumbent-map fallbacks
  and the unweighable return:
  the map is built from the artifact's own comparison rows,
  so a key miss needs a divergence the fixture builders
  do not expose).
  The reach census reads `still cold 4`:
  those four
  (`~/temp/agent/mimo-trial/reach-published3.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-published-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-published-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `39b9c1be1`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `edit` cluster
  (the ranking's next at `census-HQOwnq`).

- 2026-10-04,
  15:41 UTC:
  the next batch's baseline census taken at commit `5bd1c9712`:
  `census written to ~/.cache/translation-repair/coverage/census-HQOwnq/census.json`
  (`~/temp/agent/mimo-trial/census-43.log`).
  `library source: 120 files, 208 stretches over 443 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines in 2 files,
  `edit` with 5 over 6 in 2,
  `corpus-run/spend` with 5 over 5 in 1.
  This line lands in the trial-log commit that follows `5bd1c9712`.
  Next:
  the `corpus-run/published` batch.

- 2026-10-04,
  15:34 UTC:
  the T8 `corpus-run/cap` batch closed with commits `a5a9c4ab3`,
  `9b1240f9a`,
  `3af9376a5`:
  5 line stretches over the cap census and override files,
  three closed as unreachable
  (the stamp split,
  the paired content,
  and the P99 zero:
  all `nonNullishOrThrow`),
  two cased
  (the malformed stamp and unit word,
  and the environment read).
  The reach census reads `left 0 cold stretches`
  (`~/temp/agent/mimo-trial/reach-cap3.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-cap-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-cap-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `3af9376a5`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/published` cluster
  (the ranking's next at `census-4VTupe`).

- 2026-10-04,
  15:02 UTC:
  the T8 `han` batch closed with commit `3948763a2`:
  5 line stretches over `han-only-text.ts`
  and `han-title-read.ts`,
  all cased
  (the empty character,
  the letter walk both ways,
  the linked and linkless title reads,
  and the skipped titles).
  The reach census reads `still cold 0` on the five
  (`~/temp/agent/mimo-trial/reach-han3.log`),
  with one pre-existing line cold since then in the focused run
  and none in the whole-suite census.
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-han-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-han-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `3948763a2`.
  Next:
  the whole-suite census at this commit as the next batch's baseline:
  `census written to ~/.cache/translation-repair/coverage/census-4VTupe/census.json`
  (`~/temp/agent/mimo-trial/census-40.log`),
  `library source: 123 files, 213 stretches over 449 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/cap` leads with 5 stretches over 6 lines in 3 files,
  `corpus-run/published` with 5 over 6 in 2,
  `edit` with 5 over 6 in 2.
  Next:
  the `corpus-run/cap` batch.

- 2026-10-04,
  14:52 UTC:
  the next batch's baseline census taken at commit `91df90bfe`:
  `census written to ~/.cache/translation-repair/coverage/census-AwkhvA/census.json`
  (`~/temp/agent/mimo-trial/census-39.log`).
  `library source: 125 files, 218 stretches over 456 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `han` leads with 5 stretches over 7 lines in 2 files,
  `corpus-run/cap` with 5 over 6 in 3,
  `corpus-run/published` with 5 over 6 in 2.
  This line lands in the trial-log commit that follows `91df90bfe`.
  Next:
  the `han` batch.

- 2026-10-04,
  14:44 UTC:
  the T8 `restoration` batch closed with commit `e16e16184`:
  5 line stretches over `restoration-judge-wire.ts`
  and `restoration-judge.ts`,
  all cased
  (the wire guards,
  the duplicate judgment,
  and the ballot naming one seed only).
  The reach census reads `ran 4, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-restoration.log`).
  Counts at the close:
  the full suite 1,560 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-restoration-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-restoration-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `e16e16184`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `han` cluster
  (the ranking's next at `census-RD9far`).

- 2026-10-04,
  14:42 UTC:
  the next batch's baseline census taken at commit `8c66817c7`:
  `census written to ~/.cache/translation-repair/coverage/census-RD9far/census.json`
  (`~/temp/agent/mimo-trial/census-38.log`).
  `library source: 127 files, 223 stretches over 464 lines, 4 functions never called`
  (the never-called count moved again,
  5 to 4).
  Largest clusters by stretches then lines:
  `restoration` leads with 5 stretches over 8 lines in 2 files,
  `han` with 5 over 7 in 2,
  `corpus-run/cap` with 5 over 6 in 3.
  This line lands in the trial-log commit that follows `8c66817c7`.
  Next:
  the `restoration` batch.
