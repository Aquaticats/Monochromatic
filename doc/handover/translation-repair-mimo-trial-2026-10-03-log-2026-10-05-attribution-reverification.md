# Translation repair MiMo trial: log of 2026-10-05, the coverage attributions re-verified

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/test` is now `package/test-harness/core`.

The trial log from 01:49 to 05:22 UTC on 2026-10-05,
when the eight ledger paragraphs blaming the census's attribution were found unverified and re-opened;
it holds the correction of the record,
the byte-level method that replaced it,
and the left-open clusters re-verified one by one afterwards.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-05, 01:49 to 05:22 UTC: the coverage attributions re-verified

### 2026-10-05, 04:22 to 05:22 UTC: left-open clusters taken one by one

- 2026-10-05,
  05:22 UTC:
  the next batch's baseline census taken at commit `9fe80a109`:
  `census written to ~/.cache/translation-repair/coverage/census-aU1H0y/census.json`
  (`~/temp/agent/mimo-trial/census-final2.log` of this round),
  `library source: 90 files, 119 stretches over 309 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/runs` leads with 3 over 9 in 2,
  `transient` with 3 over 7 in 1,
  `artifact` with 2 over 17 in 2.
  This line lands in the trial-log commit that follows `9fe80a109`.
  Next:
  the `corpus-run/runs` re-verification
  with the byte-level method.

- 2026-10-05,
  05:17 UTC:
  `declined` is FULLY COVERED
  (the ledger's
  `DECLINED RE-VERIFICATION` paragraph,
  corrected in place):
  the four spans first left open
  with the cause "the aligner's emission rules unmapped"
  were covered with hand-built steps,
  and that cause was itself unverified
  (`declinedTargetBlocks` takes its steps
  as an argument).
  The whole file reads COVERED
  in the run's own census byte spans.
  Gates at the close:
  1,564 [PASS] and no [FAIL],
  35 source scans and no [FAIL].

  Pattern this round confirmed again:
  every cause I wrote for a cold line
  before running the byte-level check
  turned out wrong or incomplete,
  and the byte spans plus the code reads
  settled each one.

  Next:
  the left-open list in the ranking's order:
  `corpus-run/runs` (3),
  `transient` (3),
  `artifact` (2),
  `corpus-run/cache` (2),
  `corpus-run/name`,
  `edit`,
  and the `model-card-derive` data-invariant guard
  (its cause named and confirmed).
  Each takes the byte-level method:
  read the run's own census spans,
  slice the coverage chunk at those offsets,
  then read the code path
  before writing any fixture.

- 2026-10-05,
  05:10 UTC:
  `declined` re-verified
  (the ledger's
  `DECLINED RE-VERIFICATION` paragraph):
  the flatMap's node read cannot miss
  and became one `nonNullishOrThrow`
  under a type-guard filter,
  and the empty-pairing refusal has its case.
  LEFT with the cause named:
  the rendering-continues machinery
  (`declined-target-runs.ts` 105,
  106 to 109,
  111)
  and the claiming walk's source-only arm
  (172 to 173)
  need the aligner's `continuesPairing` emission rules
  mapped first.
  Gates at the close:
  1,564 [PASS] and no [FAIL],
  35 source scans and no [FAIL].
  Two scan slips in the edit
  (a TSDoc before a `return`,
  a position word in its comment)
  caught and fixed forward.

  Next:
  read `align-blocks-walk.ts`
  to map the `continuesPairing` emission rules,
  then design the fixtures for the four open spans;
  then the rest of the left-open list
  (`corpus-run/runs`,
  `transient`,
  `corpus-run/name`,
  `edit`)
  in the ranking's order.

- 2026-10-05,
  04:52 UTC:
  the next batch's baseline census taken at commit `6a015b331`:
  `census written to ~/.cache/translation-repair/coverage/census-i01hsd/census.json`
  (`~/temp/agent/mimo-trial/census-pub.log` of this round),
  `library source: 91 files, 123 stretches over 315 lines, 4 functions never called`.
  Largest clusters by stretches then lines:
  `declined` leads with 4 stretches over 6 lines in 1 file
  (`declined-target-runs.ts:109`,
  `158`,
  `193-194`,
  `195-196`),
  `corpus-run/runs` with 3 over 9 in 2,
  `transient` with 3 over 7 in 1.
  This line lands in the trial-log commit that follows `6a015b331`.
  Next:
  the `declined` re-verification
  with the byte-level method.

- 2026-10-05,
  04:45 UTC:
  `corpus-run/published` re-verified and CLOSED
  (the ledger's
  `PUBLISHED-PAGE RE-VERIFICATION` paragraph):
  the three incumbent-map fallbacks dead
  (`wouldShipTextPerSlice` walks the same
  `artifact.comparison` the map is built from)
  and narrowed,
  the filled-anchor caveat covered
  with its distinguishing message pair,
  and the unweighable guard
  an `unreachable:` throw
  whose line stays cold by design.
  One WR5 slip in a comment
  caught by the scan and fixed forward.
  Gates at the close:
  1,564 [PASS] and no [FAIL],
  35 source scans and no [FAIL].

  The left-open list stands at:
  `declined`'s shape reasons,
  `corpus-run/runs`' unconstructible claims,
  `transient`'s abort siblings,
  `corpus-run/name`'s comma arm
  and map fallback,
  `edit`'s single-name join,
  and the `model-card-derive` data-invariant guard
  (its cause named,
  unconstructible).
  Each takes the byte-level method
  next round.

  Next:
  re-verify `declined`
  (`declined-target-runs.ts`)
  with the byte-level method,
  then the rest in the ranking's order.

- 2026-10-05,
  04:22 UTC:
  the next batch's baseline census taken at commit `744aa0d40`:
  `census written to ~/.cache/translation-repair/coverage/census-DJtsdw/census.json`
  (`~/temp/agent/mimo-trial/census-final.log` of this round),
  `library source: 92 files, 127 stretches over 320 lines, 4 functions never called`.
  The re-verification closed 15 stretches
  (the counting is in the ledger's
  `RE-VERIFICATION COMPLETE` paragraph).
  Largest clusters by stretches then lines:
  `corpus-run/published` leads with 5 stretches over 6 lines
  in 2 files,
  `declined` with 4 over 6 in 1,
  `corpus-run/runs` with 3 over 9 in 2.
  The ranking's leaders still carry left-open claims
  whose reasons were never byte-verified
  (`corpus-run/published`'s incumbent-map fallbacks,
  `declined`'s shape reasons,
  `corpus-run/runs`' unconstructible claims,
  `transient`'s abort siblings,
  `corpus-run/name`'s comma arm,
  `edit`'s single-name join),
  and the method standard applies to them all.
  This line lands in the trial-log commit that follows `744aa0d40`.
  Next:
  re-verify `corpus-run/published`
  (`published-page-check.ts` 179,
  196,
  236
  and `published-page-disagreement.ts` 99 to 100,
  552)
  with the byte-level method,
  then the rest of the left-open list
  in the ranking's order.

### 2026-10-05, 01:49 to 04:10 UTC: the eight attribution claims and the sites left for fixtures

- 2026-10-05,
  04:10 UTC:
  the re-verification is COMPLETE
  and recorded in the ledger
  (the `CORRECTION (2026-10-05)` paragraph
  and the `RE-VERIFICATION COMPLETE (2026-10-05)`
  paragraph that follows it).
  Every site in both groups
  (the eight attribution claims
  and the fixture-reasoned left sites)
  now carries either a case whose assertion
distinguishes its branch
  with the run's own census bytes excluding the statement,
  or a named unconstructible cause:

- covered since the correction:
  `photo-reference.ts` 133 and 303,
  `dropped-destinations.ts` 187 and 292 to 295,
  `active-footnote-markers.ts` 309 to 335,
  `claim-panel-voters.ts` 54,
  `claim-filers.ts` 253 and 262,
  `model-card-derive.ts` 366;
  and dead fallbacks narrowed in
  `dropped-destinations.ts` 241,
  `claim-filers.ts` 253,
  `publish-fixed.ts` 320 and 321.

- UNRESOLVED with the cause known:
  `model-card-derive.ts` 421 to 427,
  whose two inputs are the module's own card
  and roster tables,
  so the disagreement it guards is data drift
  no fixture can stage.

  The lesson the ledger now states as method:
  a case covers a branch only where its assertion
distinguishes that branch from the other paths
  returning the same value,
  and the run's own census byte spans
  exclude the target statement from the cold set.
  Passing the suite is not branch evidence.
  A stretch's line label can start one line early,
  so the verdict line's words are derived text
  and the JSON spans are the ground truth.

  Working tree:
  clean at `f4780cd0b`,
  suite 1,564 [PASS] and no [FAIL],
  35 source scans and no [FAIL],
  two TSDoc-before-statement slips
  caught by the attachment scan
  and fixed forward.

  Verification worktree `/home/user/worktrees/verify-coverage`
  (detached,
  probe files present)
  is disposable;
  `git worktree remove` once the counts are read.

  Next:
  the whole-suite census at this commit,
  then the ranking's next cluster
  with the method standard
  (distinguishing assertions,
  byte-span proof,
  and named causes for whatever stays open).

- 2026-10-05,
  03:10 UTC:
  the eight attribution claims are re-verified and CLOSED,
  with a self-named correction in the ledger
  (`CORRECTION (2026-10-05)`).
  Every one traced to my own fixtures or assertions,
  not the tool:

- resolution's 328 was covered all along
  (V8's range for the return's bytes reports count 1;
  the run's cold span begins after them,
  and the stretch's line label starts at the boundary line).

- lane read the archive field of each BALLOT,
  where my case set it on the outcome.

- nudged's first reply carried no `servedBy`,
  so the function answered before the re-ask,
  and both assertions read `kind === 'ok'`,
  which every path returns.

- canadian's dates sat in other readers:
  the refused year wants a month-first date
  followed by four or more digits the year read refuses,
  the runs-on comma belongs to the year-first reader
  and needs an ordinal day,
  and the underscore counter runs only
  for a capitalized candidate word.

- spend's negative-count return shares its literal
  with the returns beside it,
  so the assertion could not tell them apart;
  the file's own known-good tail now proves the readable path.
  Its `fields[0]` fallback is dead
  (`split` always yields one)
  and became `nonNullishOrThrow`.

- lookup 193 and 221,
  slice-cache-namespace 266 and 378,
  and model-content 36 were covered as written;
  the residual spans beside them are neighbouring statements.

  Verified tool facts,
  recorded in the ledger for later readers:
  the suite prints one info line per suite
  (per-case lines at debug),
  so pass counts are suite counts;
  a stretch's line label can start one line early,
  and each run's own census JSON byte spans
  are the ground truth;
  the coverage build is unminified because
  compression folds guards into expressions V8 gives no range for.

  Method from here:
  a case covers a branch only where its assertion
distinguishes that branch from other paths returning the same value,
  and the run's own census bytes exclude the target statement
  from the cold set.

  Open work re-opened by the same standard:
  the sites earlier recorded as left for fixture reasons
  were not byte-verified either,
  and the same method applies to them:
  `photo-reference.ts` 133 and 303,
  `dropped-destinations.ts` 187,
  241,
  292 to 295,
  `active-footnote-markers.ts` 309 to 335,
  `claim-panel-voters.ts` 54
  and `claim-filers.ts` 253 and 262,
  `publish-fixed.ts` 320 and 321,
  `model-card-derive.ts` 366 and 421 to 427,
  and the nudged abort arm's siblings
  (all covered now).
  The claim group's fixture blocker
  (the `AdjudicatedIssue` shape)
  and the publish group's
  (the destination fields)
  are the two named ones still needing fixture work.

  Verification worktree:
  `/home/user/worktrees/verify-coverage`
  (detached at `1099a5495`),
  holds the coverage build and the probe files
  `zz-probe-guard.ts` and `zz-attribution-probe.unit.test.ts`
  used for the V8 experiment;
  it is disposable and can be removed
  (`git worktree remove`) once the remaining sites are done.

  Next:
  re-verify `photo-reference.ts` 133 and 303
  with the byte-level method,
  then the rest of the re-opened list in the order named.

- 2026-10-05,
  01:49 UTC:
  CORRECTION OF THE RECORD,
  and the investigation it re-opens.
  Eight ledger paragraphs explain uncovered lines
  as "the census's attribution" leaving them cold
  (ledger lines 6515,
  6602,
  6889,
  6949,
  6987,
  7020,
  7065,
  7106).
  That explanation was never verified,
  and each of the eight is now re-opened.
  The claims stand as UNRESOLVED until re-verified.
  What the re-verification established so far:

- The test harness logs one visible line per suite,
  not per case.
  Per-case `[PASS]` lines are debug level,
  and empty-name suites log at debug too
  (`package/module/test/src/describe.ts`,
  "Empty-name suites are invisible wrappers").
  Consequence:
  counting `[PASS]` lines counts named suites,
  so an earlier reading
  (`3 passes, my case never ran`)
  was wrong,
  and every earlier case count read off those lines
  was wrong with it.

- `isJsonRecord` is only
  object and non-null and not-array
  (`package/module/translation-repair/src/json-guard.ts` (moved on 2026-10-18 to
  `package/translation-repair/document-structure/src/json-guard.ts`)),
  so a check element like `{ issue: 'one', verdict: 'fixed' }`
  passes it and reaches the type guard behind it.

- The coverage-census verdict line derives its words
  from the baseline's stretch boundaries
  matched against the run's own merged cold byte spans.
  Adjacent statements merge into one span
  (the reach run's own JSON shows
  `resolution-wire.ts` 328 to 331 as one span `[11812..11892]`).
  The run's own census JSON
  (the `census written to ...` path)
  is the ground truth;
  the verdict line is derived and can misattribute.

- Verified about the `resolution` case
  (`resolution-wire.unit.test.ts`):
  the case runs.
  Its first two calls turned the baseline-cold lines 321 and 348 hot
  in the reach run's own census
  (`census-qIxdKv`),
  and no other case in that file touches those guards.
  The whole-suite censuses taken after the case landed
  (`census-5tlfGG`,
  `census-hCZX4J`)
  list `resolution-wire.ts` clean.

- UNRESOLVED about that case:
  whether its third call executes the `return false` at
  `resolution-wire.ts:328`.
  Two hypotheses remain open:
  the coverage build's source-map attribution drops that
  statement's bytes,
  or a code-path reason not yet found.
  The decisive experiment,
  not yet run:
  a single-case scratch test carrying only that call,
  run under the reach census,
  read against its own census JSON,
  with a minimal synthetic module of the same guard shape
  as the tool's positive control,
  in a throwaway worktree.

- The sites still to re-verify,
  in the order they will be taken:
  ledger 7106 (`canadian-date-read-leading.ts` 259 and 335,
  `canadian-spelling-capital.ts` 463),
  7020 (`nudged-reask.ts` 147 and 153),
  6987 (`lane-contest-cache-store.ts` 59 to 61),
  6949 (`lookup-cache.ts` 193,
  221,
  321 to 324),
  6889 (`model-content.ts` 36),
  6602 (`slice-cache-namespace.ts` 266 and 378),
  6515 (`spend-read.ts` 238 and 345).
  Each takes a focused case,
  the reach census's own JSON,
  and where a tool gap is claimed,
  a minimal reproduction before the claim is written.

- Reporting standard from here:
  COMPLETED means a case landed,
  the run including the case shows the line hot,
  and the assertion distinguishes the branch where one exists.
  ATTEMPTED means a case landed and the cause of a cold line
  is identified.
  UNRESOLVED means the cause is not found,
  and it is recorded as unresolved rather than explained.
  A passing suite is not branch evidence.
  Contrary evidence is kept in place beside the correction.

- Also corrected:
  reach runs select narrow test file sets,
  so lines other drivers exercise are cold there on purpose
  (the `cold since then (ran there)` rows);
  earlier paragraphs mixed those rows into the same claim
  as the stuck lines,
  and the re-verification separates them.

- Environment:
  throwaway worktrees are available for verification runs
  (the owner's word),
  and the tool-behavior reproductions will use one
  so the trial tree stays clean.
  No execution limit of any kind was reported by the runtime;
  earlier notes claiming one are retracted.

  Next:
  the decisive experiment for `resolution-wire.ts:328`
  in a throwaway worktree,
  then the seven sites in the order named.
