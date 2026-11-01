# Translation repair MiMo trial: log from 2026-10-04 20:45 UTC to 2026-10-05 01:20 UTC

The trial log from 20:45 UTC on 2026-10-04 to 01:20 UTC on 2026-10-05:
T8 batches on the clusters of three and four uncovered stretches,
each batch a closing line and the baseline census that follows it.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-04, 20:45 UTC to 2026-10-05, 01:20 UTC: T8 batches

- 2026-10-05,
  01:18 UTC:
  the next batch's baseline census taken at commit `abb99ceb7`:
  `census written to ~/.cache/translation-repair/coverage/census-cLOz3U/census.json`
  (`~/temp/agent/mimo-trial/census-81.log`),
  `library source: 100 files, 150 stretches over 350 lines, 4 functions never called`.
  The ranking's first nine clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `corpus-run/publish`
  with 3 stretches over 2 lines in 1 file
  (`publish-fixed.ts:320`,
  `321`),
  then the ranking's next.
  This line lands in the trial-log commit that follows `abb99ceb7`.
  Next:
  the `corpus-run/publish` batch.

- 2026-10-05,
  01:20 UTC:
  the `corpus-run/publish` batch left its 3 arms documented
  (`publish-fixed.ts:320`,
  `321`):
  the dropped-destination warn's trace fallbacks,
  whose lists go absent where the address is dropped
  from the page but absent from every slice's text,
  a fixture whose artifact destination fields
  this pass did not build.
  Counts at the close:
  the full suite 1,564 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-publish-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-publish-scans.log`).
  This line lands in the trial-log commit that follows.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the ranking's next actionable cluster.

- 2026-10-05,
  01:00 UTC:
  the next batch's baseline census taken at commit `455622520`:
  `census written to ~/.cache/translation-repair/coverage/census-2aJKLC/census.json`
  (`~/temp/agent/mimo-trial/census-80.log`),
  `library source: 100 files, 150 stretches over 350 lines, 4 functions never called`.
  The ranking's first eight clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `corpus-run/canadian`
  with 3 stretches over 3 lines in 2 files,
  then `corpus-run/publish`.
  This line lands in the trial-log commit that follows `455622520`.
  Next:
  the `corpus-run/canadian` batch's remaining arms.

- 2026-10-05,
  01:10 UTC:
  the `corpus-run/canadian` second pass closed with commit `52a5e2c9c`:
  the date cases given leading dates,
  three arms left documented
  (the year-refused return,
  the runs-on comma,
  and the underscore count:
  the cases assert their rewrites,
  the lines' returns the census's attribution leaves cold).
  The reach census reads `ran 0, still cold 2`
  (`~/temp/agent/mimo-trial/reach-canadian2.log`).
  Counts at the close:
  the full suite 1,564 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-canadian2-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-canadian2-scans.log`).
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/publish` cluster
  (the ranking's next at `census-2aJKLC`).

- 2026-10-05,
  00:54 UTC:
  the T8 `claim` batch left its 3 arms documented
  (`claim-panel-voters.ts:54`,
  `claim-filers.ts:253`,
  `262`):
  the cases need the `AdjudicatedIssue`'s full fixture shape,
  whose fields the attach map reads directly,
  and the two attempts crashed on the runtime reads.
  Both attempts reverted;
  the tree stays green.
  Counts at the close:
  the full suite 1,564 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-claim-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-claim-scans.log`).
  This line lands in the trial-log commit that follows the revert.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/canadian` cluster's remaining arms
  or the next in the ranking.

- 2026-10-05,
  00:52 UTC:
  the next batch's baseline census taken at commit `e484cbf04`:
  `census written to ~/.cache/translation-repair/coverage/census-5tlfGG/census.json`
  (`~/temp/agent/mimo-trial/census-79.log`),
  `library source: 100 files, 150 stretches over 350 lines, 4 functions never called`.
  The ranking's first seven clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `claim`
  with 3 stretches over 3 lines in 2 files
  (`claim-panel-voters.ts:54`,
  `claim-filers.ts:253`,
  `262`),
  then `corpus-run/canadian`,
  `corpus-run/publish`.
  This line lands in the trial-log commit that follows `e484cbf04`.
  Next:
  the `claim` batch.

- 2026-10-05,
  00:44 UTC:
  the T8 `resolution` batch closed with commit `8bb2ce647`:
  3 line stretches over `resolution-wire.ts`,
  two cased,
  one left documented
  (the issue's non-number refusal:
  the attribution quirk's record).
  The reach census reads `ran 2, still cold 1`
  (`~/temp/agent/mimo-trial/reach-resolution.log`).
  Counts at the close:
  the full suite 1,564 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-resolution-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-resolution-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `8bb2ce647`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `claim` cluster
  (the ranking's next at `census-TXnKot`).

- 2026-10-05,
  00:38 UTC:
  the next batch's baseline census taken at commit `40af0f949`:
  `census written to ~/.cache/translation-repair/coverage/census-TXnKot/census.json`
  (`~/temp/agent/mimo-trial/census-78.log`),
  `library source: 101 files, 153 stretches over 353 lines, 4 functions never called`.
  The ranking's first seven clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `resolution`
  with 3 stretches over 3 lines in 1 file
  (`resolution-wire.ts:321`,
  `328`,
  `348`),
  then `claim`,
  `corpus-run/canadian`.
  This line lands in the trial-log commit that follows `40af0f949`.
  Next:
  the `resolution` batch.

- 2026-10-05,
  00:31 UTC:
  the T8 `photo` batch closed with commits `8284b4b4f`,
  `b5cae5d8a`:
  3 line stretches over `photo-reference.ts`,
  one cased
  (the unclosed-quote break),
  two left documented
  (the earlier-quote preference
  and the not-an-asset arm:
  the cases' shapes did not reach them).
  A TSDoc block before an expect statement tripped
  the attachment scan (ledger B19),
  fixed forward.
  The reach census reads `ran 1, still cold 2`
  (`~/temp/agent/mimo-trial/reach-photo.log`).
  Counts at the close:
  the full suite 1,564 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-photo-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-photo-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `b5cae5d8a`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `resolution` cluster
  (the ranking's next at `census-pUl33e`).

- 2026-10-05,
  00:18 UTC:
  the next batch's baseline census taken at commit `31f7b5559`:
  `census written to ~/.cache/translation-repair/coverage/census-pUl33e/census.json`
  (`~/temp/agent/mimo-trial/census-77.log`),
  `library source: 101 files, 154 stretches over 354 lines, 4 functions never called`.
  The nudged batch's allowlist entry went stale with its new test
  (`31f7b5559`),
  and one commit message's `--message` split into a stray `-- message`
  before the pathspec
  (recorded here rather than amended,
  ledger GCA).
  The ranking's first seven clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `photo`
  with 3 stretches over 3 lines in 1 file
  (`photo-reference.ts:133`,
  `240`,
  `303`),
  then `resolution`,
  `claim`.
  This line lands in the trial-log commit that follows `31f7b5559`.
  Next:
  the `photo` batch.

- 2026-10-05,
  00:06 UTC:
  the T8 `nudged` batch closed with commit `b487f4d02`:
  3 line stretches over `nudged-reask.ts`,
  the re-ask outcomes cased,
  the two log fallbacks and the abort rethrow left documented
  (the attribution quirk's fifth instance
  and the fixture's re-ask shape resolving).
  Counts at the close:
  the full suite 1,564 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-nudged-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-nudged-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `b487f4d02`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `photo` cluster
  (the ranking's next at `census-GHsN0h`).

- 2026-10-04,
  23:56 UTC:
  the next batch's baseline census taken at commit `d4894fb60`:
  `census written to ~/.cache/translation-repair/coverage/census-GHsN0h/census.json`
  (`~/temp/agent/mimo-trial/census-74.log`),
  `library source: 101 files, 154 stretches over 354 lines, 4 functions never called`.
  The ranking's first six clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `nudged`
  with 3 stretches over 3 lines in 1 file
  (`nudged-reask.ts:147`,
  `153`,
  `159`),
  then `photo`,
  `resolution`,
  `claim`.
  This line lands in the trial-log commit that follows `d4894fb60`.
  Next:
  the `nudged` batch.

- 2026-10-04,
  23:49 UTC:
  the T8 `corpus-run/reading` batch closed with commit `1fb9cb177`:
  3 line stretches over `reading-cache-store.ts`,
  all cased
  (the three malformed shapes
  refusing to resume).
  The reach census reads `ran 3, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-reading.log`).
  Counts at the close:
  the full suite 1,563 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-reading-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-reading-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `1fb9cb177`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `nudged` cluster
  (the ranking's next at `census-G1oD6W`).

- 2026-10-04,
  23:45 UTC:
  the next batch's baseline census taken at commit `c82ae61f7`:
  `census written to ~/.cache/translation-repair/coverage/census-G1oD6W/census.json`
  (`~/temp/agent/mimo-trial/census-73.log`),
  `library source: 102 files, 157 stretches over 357 lines, 4 functions never called`.
  The ranking's first six clusters carry documented-left arms
  or the attribution quirk's records.
  The next actionable cluster is `corpus-run/reading`
  with 3 stretches over 3 lines in 1 file,
  then `nudged`,
  `photo`,
  `resolution`.
  This line lands in the trial-log commit that follows `c82ae61f7`.
  Next:
  the `corpus-run/reading` batch.

- 2026-10-04,
  23:38 UTC:
  the T8 `corpus-run/lane` batch closed with commit `4fa547794`:
  3 line stretches over `lane-contest-cache-store.ts`,
  all cased
  (the archive names
  and the non-record refusal).
  The reach census reads `ran 1, still cold 2`,
  the OR chain's branches the census's attribution leaves cold
  (the fourth instance of the pattern,
  with the cases asserting their round trips directly).
  Counts at the close:
  the full suite 1,563 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-lane-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-lane-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `4fa547794`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the next actionable cluster in the ranking
  (all the leaders now carry documented-left arms
  or the attribution quirk's records).

- 2026-10-04,
  22:56 UTC:
  the T8 `retry` batch closed with commit `e8636cf64`:
  3 line stretches over `retry-stated-wait.ts`,
  one closed as unreachable
  (the digit walk's character read),
  two left documented
  (the walk's return pair
  and the gap's and-word arm).
  The reach census reads `left 2 cold stretches`:
  those
  (`~/temp/agent/mimo-trial/reach-retry.log`).
  Counts at the close:
  the full suite 1,563 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-retry-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-retry-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `e8636cf64`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/lane` cluster
  (the ranking's next at `census-bppPVq`).

- 2026-10-04,
  22:52 UTC:
  the next batch's baseline census taken at commit `ccf8a7eb7`:
  `census written to ~/.cache/translation-repair/coverage/census-bppPVq/census.json`
  (`~/temp/agent/mimo-trial/census-71.log`),
  `library source: 102 files, 159 stretches over 359 lines, 4 functions never called`.
  The lookup batch's new file tripped three scans on its way in
  (the own-tests allowlist entry,
  the direct mkdtemp,
  and the unused path join),
  each fixed forward
  (`153da6c69`,
  `ccf8a7eb7`).
  The ranking's first six clusters carry documented-left arms.
  The next actionable cluster is `retry`
  with 3 stretches over 6 lines in 1 file
  (`retry-stated-wait.ts:142`,
  `143-145`,
  `219-220`),
  then `corpus-run/lane`.
  This line lands in the trial-log commit that follows `ccf8a7eb7`.
  Next:
  the `retry` batch.

- 2026-10-04,
  22:36 UTC:
  the T8 `lookup` batch closed with commit `a9b5e71dc`:
  3 line stretches over `lookup-cache.ts`,
  all cased
  (the non-object value
  and the cache file that is no record).
  The reach census reads `ran 0, still cold 3`,
  the single-line returns the census's attribution leaves cold
  (the cases assert their values directly).
  Counts at the close:
  the full suite 1,563 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-lookup-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-lookup-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `a9b5e71dc`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `retry` cluster
  (the ranking's next at `census-5AHbot`).

- 2026-10-04,
  22:31 UTC:
  the next batch's baseline census taken at commit `2b64f2e6e`:
  `census written to ~/.cache/translation-repair/coverage/census-5AHbot/census.json`
  (`~/temp/agent/mimo-trial/census-67.log`).
  `library source: 103 files, 162 stretches over 365 lines, 4 functions never called`.
  The ranking's first six clusters carry documented-left arms.
  The next actionable cluster is `lookup`
  with 3 stretches over 6 lines in 1 file
  (`lookup-cache.ts:193`,
  `221`,
  `321-324`),
  then `retry`.
  This line lands in the trial-log commit that follows `2b64f2e6e`.
  Next:
  the `lookup` batch.

- 2026-10-04,
  22:24 UTC:
  the T8 `corpus-run/dropped` batch closed with commit `b55509754`:
  3 line stretches over `dropped-destinations.ts`,
  the run case landed
  (the stopper shed
  and several runs read in order),
  the three arms left documented
  (the earliest-reduce arm,
  the run-advance arm,
  and the stopper walk's return pair:
  the case's shapes did not reach them).
  The reach census reads `ran 0, still cold 3`:
  those three
  (`~/temp/agent/mimo-trial/reach-dropped.log`).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-dropped-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-dropped-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `b55509754`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `lookup` cluster
  (the ranking's next at `census-MkpU3W`).

- 2026-10-04,
  22:20 UTC:
  the next batch's baseline census taken at commit `c06539dd8`:
  `census written to ~/.cache/translation-repair/coverage/census-MkpU3W/census.json`
  (`~/temp/agent/mimo-trial/census-66.log`).
  `library source: 103 files, 162 stretches over 365 lines, 4 functions never called`.
  The ranking's first five clusters carry documented-left arms.
  The next actionable cluster is `corpus-run/dropped`
  with 3 stretches over 6 lines in 1 file,
  then `lookup`,
  `retry`.
  This line lands in the trial-log commit that follows `c06539dd8`.
  Next:
  the `corpus-run/dropped` batch.

- 2026-10-04,
  22:12 UTC:
  the T8 `container` batch closed with commit `4621ad099`:
  3 line stretches over `container-integrity.ts`
  and `container-half-pairs.ts`,
  one closed as unreachable
  (the splice's own guard),
  two left documented
  (the nameless-container arm
  and the half-tag integrity error:
  their fixtures need a container with no name
  and a block cutting one tag).
  The reach census reads `ran 0, still cold 2`:
  those two
  (`~/temp/agent/mimo-trial/reach-container.log`).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-container-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-container-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `4621ad099`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the next actionable cluster in the ranking
  (`edit`,
  `preservation`,
  or the documented-left revisits).

- 2026-10-04,
  22:10 UTC:
  the next batch's baseline census taken at commit `e91b5dd12`:
  `census written to ~/.cache/translation-repair/coverage/census-VzG5Wr/census.json`
  (`~/temp/agent/mimo-trial/census-65.log`).
  `library source: 104 files, 163 stretches over 366 lines, 4 functions never called`.
  The ranking's first five clusters
  (`corpus-run/published`,
  `active`,
  `declined`,
  `corpus-run/runs`,
  `transient`)
  all carry documented-left arms.
  The next actionable cluster is `container`
  with 3 stretches over 7 lines in 2 files,
  then `edit`,
  `preservation`.
  This line lands in the trial-log commit that follows `e91b5dd12`.
  Next:
  the `container` batch.

- 2026-10-04,
  22:02 UTC:
  the T8 `model` batch closed with commit `cb4b07506`:
  3 line stretches over `model-card-derive.ts`
  and `model-content.ts`,
  one cased
  (the fence line with no line end),
  two left documented
  (the two RangeError guards:
  their builders take keys from the roster lists
  and need a fixture disagreeing with them).
  The reach census reads `still cold 1` on `model-content.ts:36`,
  the single-line return the census's attribution leaves cold
  (the case asserts its value directly).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-model2-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-model2-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `cb4b07506`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/runs` cluster's remaining arms
  or the next in the ranking.

- 2026-10-04,
  22:00 UTC:
  the next batch's baseline census taken at commit `187348d0e`:
  `census written to ~/.cache/translation-repair/coverage/census-pN6fUe/census.json`
  (`~/temp/agent/mimo-trial/census-64.log`).
  `library source: 105 files, 164 stretches over 367 lines, 4 functions never called`.
  The ranking's first three clusters carry documented-left arms.
  The next actionable cluster is `model`
  with 3 stretches over 9 lines in 2 files
  (`model-card-derive.ts:366`,
  `421-427`,
  `model-content.ts:36`),
  then `corpus-run/runs`,
  `transient`.
  This line lands in the trial-log commit that follows `187348d0e`.
  Next:
  the `model` batch.

- 2026-10-04,
  21:53 UTC:
  the T8 `hyper` batch closed with commit `309cafca9`:
  3 line stretches over `hyper-client.ts`,
  all cased
  (the deadline and answer-length knobs
  carried only when set).
  The reach census reads `ran 3, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-hyper.log`).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-hyper-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-hyper-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `309cafca9`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `model` cluster
  (the ranking's next at `census-bOt8GY`).

- 2026-10-04,
  21:50 UTC:
  the next batch's baseline census taken at commit `4e08e3027`:
  `census written to ~/.cache/translation-repair/coverage/census-bOt8GY/census.json`
  (`~/temp/agent/mimo-trial/census-63.log`).
  `library source: 106 files, 167 stretches over 377 lines, 4 functions never called`.
  The ranking's first three clusters carry documented-left arms.
  The next actionable cluster is `hyper`
  with 3 stretches over 10 lines in 1 file
  (`hyper-client.ts:364-369`,
  `375-376`,
  `421-422`),
  then `model`,
  `corpus-run/runs`.
  This line lands in the trial-log commit that follows `4e08e3027`.
  Next:
  the `hyper` batch.

- 2026-10-04,
  21:28 UTC:
  the next batch's baseline census taken at commit `8c9136d6b`:
  `census written to ~/.cache/translation-repair/coverage/census-oVnawX/census.json`
  (`~/temp/agent/mimo-trial/census-61.log`).
  `library source: 107 files, 171 stretches over 380 lines, 4 functions never called`.
  The ranking's first three clusters carry documented-left arms.
  The next actionable cluster is `source`
  with 4 stretches over 3 lines in 1 file
  (`source-only-breaks.ts:170`,
  `186`,
  `199`),
  then `corpus-run/cache`,
  `hyper`.
  This line lands in the trial-log commit that follows `8c9136d6b`.
  Next:
  the `source` batch.

- 2026-10-04,
  21:36 UTC:
  the T8 `source` batch closed with commit `250da1333`:
  4 line stretches over `source-only-breaks.ts`,
  all four closed as unreachable
  (the break-count fallbacks
  the walks' own indices settle),
  one arm left documented
  (the sumOwed reduce's other-kind arm:
  its fixture needs blocks of more than one kind).
  The whole-suite census lists one stretch for the file
  (`census-ceHkbQ`,
  `library source: 107 files, 168 stretches over 378 lines, 4 functions never called`).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-source-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-source-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `250da1333`.
  Next:
  the whole-suite census at this commit as the next batch's baseline
  (`census-ceHkbQ` stands),
  then the `corpus-run/cache` cluster
  (the ranking's next at `census-oVnawX`).

- 2026-10-04,
  21:42 UTC:
  the T8 `corpus-run/cache` batch closed with commits `4d5c7d96e`,
  `14a44721c`:
  3 line stretches over `cache-account-read.ts`
  and `cache-account-slices.ts`,
  one cased
  (the name whose tail runs to its line end),
  the other two matched in the reach run.
  The reach census reads `ran 1, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-cache2.log`).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-cache-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-cache-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `14a44721c`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `hyper` cluster
  (the ranking's next at `census-oVnawX`).

- 2026-10-04,
  21:09 UTC:
  the next batch's baseline census taken at commit `a7d014baf`:
  `census written to ~/.cache/translation-repair/coverage/census-4BqrOY/census.json`
  (`~/temp/agent/mimo-trial/census-59.log`).
  `library source: 109 files, 175 stretches over 384 lines, 4 functions never called`.
  The ranking's first three clusters carry documented-left arms.
  The next actionable cluster is `mask`
  with 4 stretches over 4 lines in 2 files
  (`mask-invisible-lines.ts:168`,
  `333`,
  `mask-container-tags.ts:156`,
  `171`),
  then `source`,
  `corpus-run/cache`.
  This line lands in the trial-log commit that follows `a7d014baf`.
  Next:
  the `mask` batch.

- 2026-10-04,
  21:21 UTC:
  the T8 `mask` batch closed with commits `29326895e`,
  `d4a3c3c65`:
  4 line stretches over `mask-invisible-lines.ts`
  and `mask-container-tags.ts`,
  two closed as unreachable
  (the code point
  and the line start:
  both reads their walks guarantee),
  two cased
  (the non-letter tag name
  and the closer with anything past its name).
  The reach census reads `left 1 cold stretch` for
  `mask-invisible-lines.ts` in the focused run,
  none in the whole-suite census
  (`~/temp/agent/mimo-trial/reach-mask2.log`).
  Counts at the close:
  the full suite 1,562 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-mask-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-mask-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `d4a3c3c65`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `source` cluster
  (the ranking's next at `census-4BqrOY`).

- 2026-10-04,
  20:56 UTC:
  the next batch's baseline census taken at commit `c087e5884`:
  `census written to ~/.cache/translation-repair/coverage/census-E4DYQO/census.json`
  (`~/temp/agent/mimo-trial/census-58.log`).
  `library source: 110 files, 176 stretches over 385 lines, 4 functions never called`.
  The ranking's first three clusters carry documented-left arms.
  The next actionable cluster is `corpus-run/canadian`
  with 4 stretches over 4 lines in 3 files
  (`canadian-date-parts.ts:354`,
  `canadian-date-read-leading.ts:259`,
  `335`,
  `canadian-spelling-capital.ts:463`),
  then `mask`,
  `source`,
  each at 4.
  This line lands in the trial-log commit that follows `c087e5884`.
  Next:
  the `corpus-run/canadian` batch.

- 2026-10-04,
  21:01 UTC:
  the T8 `corpus-run/canadian` batch closed with commit `10210a333`:
  4 line stretches over the canadian date and spelling files,
  all cased
  (the short year,
  the missing year,
  the runs-on comma,
  and the stray underscore).
  The reach census reads `ran 1, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-canadian.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-canadian-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-canadian-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `10210a333`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `mask` cluster
  (the ranking's next at `census-E4DYQO`).

- 2026-10-04,
  20:45 UTC:
  the next batch's baseline census taken at commit `2b86649b6`:
  `census written to ~/.cache/translation-repair/coverage/census-E2Gm5a/census.json`
  (`~/temp/agent/mimo-trial/census-57.log`).
  `library source: 110 files, 178 stretches over 387 lines, 4 functions never called`.
  The ranking's first three clusters carry documented-left arms.
  The next actionable cluster is `corpus-run/name`
  with 4 stretches over 4 lines in 1 file
  (`name-gloss-restore.ts:87`,
  `100`,
  `247`,
  `312`),
  then `corpus-run/canadian`,
  `mask`,
  each at 4.
  This line lands in the trial-log commit that follows `2b86649b6`.
  Next:
  the `corpus-run/name` batch.

- 2026-10-04,
  20:48 UTC:
  the T8 `corpus-run/name` batch closed with commits `1daaca6e1`,
  `f29a7247c`:
  4 line stretches over `name-gloss-restore.ts`,
  two cased
  (the unclosed quote
  and the name with no gloss verb),
  the comma case landed but missed the arm's shape,
  two left documented
  (the comma arm
  and the archive-slice map fallback).
  The reach census reads `ran 2, still cold 2`:
  those two
  (`~/temp/agent/mimo-trial/reach-name2.log`).
  Counts at the close:
  the full suite 1,561 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-name-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-name-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `f29a7247c`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/canadian` cluster
  (the ranking's next at `census-E2Gm5a`).
