# @monochromatic-dev/translation-repair-test-fixture

Test fixtures that import no production code,
shared by the tests of the whole translation-repair family:
scratch directories,
capturing loggers,
scripted wire pieces,
roster seats and the like.
It has no build,
and every other family package takes it as a development dependency.
It is its own package so that one copy of each fixture serves every package,
instead of each package keeping a copy.

## What it provides

Every file is a `*.test-fixture.ts`,
reached by file through the `./ts/*` export.
The groups that follow name the main ones.

- Test infrastructure:
  `scratch-dir.test-fixture.ts` gives `scratchDir` and `scratchDirWith`,
  throwaway directories removed when their scope ends.
  `capturing-logger.test-fixture.ts` gives `capturingLogger`,
  a logger that keeps every line.
  `entry-log-lines.test-fixture.ts` gives `entryLinesWritten`,
  which collects the lines the default logger writes for one entry while work runs,
  and `linesWithoutAttempt`,
  which picks out those that do not name the attempt.
  `hang-stop.test-fixture.ts` gives `HANG_STOP_MS`,
  the one real-clock bound a test may set over scripted work.
  `wall-clock-stub.test-fixture.ts` gives `stubWallClock`.
  `rejecting-call.test-fixture.ts` gives `rejectionOf`,
  which returns what a call threw.
  `scan-findings.test-fixture.ts` gives `expectNoFindings` and `expectFindingsAsListed` for the scans.
- Scripted models and sheets:
  `roster-seats.test-fixture.ts` names the seats unit tests sit models in by the role the test leans on.
  `translate-ballot.test-fixture.ts` gives `candidateCarrying`,
  and `structural-ballot.test-fixture.ts` gives `ballot`.
  `tense-sheet.test-fixture.ts` and `sheet-fence.test-fixture.ts` read rendered sheets.
  `reason-order.test-fixture.ts` gives `expectReasonFirstInSchema` and `expectReasonBeforeVoteInText`,
  which require a ballot's schema and reply instruction to put the reason before the vote,
  as the owner ruled on 2026-10-21.
- Wire and failure pieces:
  `sse-frame.test-fixture.ts`,
  `anthropic-frames.test-fixture.ts` and `piece-response.test-fixture.ts` build streamed replies.
  `body-cut-response.test-fixture.ts` builds a response that fails after its status line.
  `quoting-failure.test-fixture.ts` builds a failure whose own message quotes a credential.
  `stack-edge.test-fixture.ts` runs a function with its caller almost out of stack,
  and `parse-exhaustion.test-fixture.ts` makes a parse raise the engine's stack exhaustion.
  `refusal-order.test-fixture.ts` makes two concurrent members refuse in an order the case chooses.
  `child-sight.test-fixture.ts` reads what a child process can see of its parent's environment.
- Texts and generated data:
  `seeded-draws.test-fixture.ts` draws repeatable pseudo-random values,
  `runs-of-pieces.test-fixture.ts` lists every short run of pieces,
  `sorted-number-sets.test-fixture.ts` renders index sets as sorted arrays,
  and `element-reads.test-fixture.ts` counts how often a list's elements are read.
  `han-findings.test-fixture.ts`,
  `destination-address-texts.test-fixture.ts` and `consistency-marks-reading.test-fixture.ts` hold texts and
  readings.
- Corpus-run fixtures:
  the `corpus-run/` directory holds fixtures for the runners and reports,
  such as `console-log-capture.test-fixture.ts`,
  `report-run-capture.test-fixture.ts`,
  `keyless-built-command.test-fixture.ts`,
  `gated-git.test-fixture.ts`,
  `runs-dir-pointed.test-fixture.ts`,
  `score-artifacts.test-fixture.ts`,
  `settled-v1-pool.test-fixture.ts` and `unopenable-file.test-fixture.ts`,
  which writes a mode-000 file and proves it cannot be opened.

Two files test the fixtures themselves:
`wall-clock-stub.unit.test.ts` and `scan-findings.unit.test.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

No family package.
Its development dependencies include `module-logger`,
`module-test` and `unified`,
none of which is in the family.

### Used by

Every other family package lists it in `devDependencies`:

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `document-structure`
- `glossary`
- `house-rule`
- `insertion`
- `lane-contest`
- `markup`
- `page-check`
- `page-pass`
- `probe`
- `provider`
- `provider-stream`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `shared-contract`
- `slice-cache`
- `slice-reading`
- `source-scan`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

## Tests

The two unit tests sit beside the fixtures they check.
Run them with `mise run //package/translation-repair/test-fixture:test:unit`.
There is no build,
so nothing builds first.

Other packages import a fixture by file:
`@monochromatic-dev/translation-repair-test-fixture/ts/<file>`,
for example `scratch-dir.test-fixture.ts`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
