# @monochromatic-dev/translation-repair-assembly

Assembly integrity and repetition checks,
and the repair records assembly reads:
it guards the footnotes and structure of an assembled repair,
finds repetition and lost content a repair introduced,
and holds the repair contract,
record and round shapes.
It is its own package because the repair,
consistency,
lane-contest and page layers all assemble through the same guards,
and it sits above the text and `adjudication` packages.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Assembly integrity:
  refuse a replacement that restates what its slice already says,
  check both change sets and return them in document order,
  splice replacements into a document and withdraw those that break its footnote graph or page structure,
  withhold a container half whose partner ships nothing,
  and cut the blocks of orphan footnote definitions.
  Names:
  `guardFootnoteAssembly`,
  `assertReplacementsChange`,
  `checkedChangeSets`,
  `orderedChangeSets`,
  `AssemblyContractError`,
  `withholdLoneContainerHalves`,
  `trimOrphanDefinitions`.
  Declared in `assembly-integrity.ts`,
  `assembly-invariant.ts`,
  `assembly-container-halves.ts`,
  and `assembly-orphan-trim.ts`.
- Repetition checks:
  find passages the shipped document repeats more often than the archive did,
  and wording that neighbouring slices both ship but the archive did not repeat.
  Names:
  `findIntroducedRepetitions`,
  `repetitionFindings`,
  `adjacentRepetitionFindings`,
  `countPhrases`,
  `grownSpans`.
  Declared in `assembly-repetition.ts`,
  `assembly-adjacent-repetition.ts`,
  and `assembly-repetition-span.ts`.
- Content survival and structure:
  measure how much of the archive's specific vocabulary the assembled document still has,
  count the footnote and structural defects it carries beyond its incumbent's,
  and look for a single withdrawal that leaves no such defect.
  Names:
  `measureContentSurvival`,
  `contentSurvivalFindings`,
  `introducedStructuralRegressions`,
  `introducedFootnoteFindings`,
  `singleStructuralWithdrawal`.
  Declared in `assembly-content-survival.ts`,
  `assembly-regressions.ts`,
  and `assembly-structural-withdrawal.ts`.
- Repair records:
  assemble a repair,
  record each issue's disposition,
  and describe each round.
  Names:
  `assembleRepair`,
  `buildIssueRecords`,
  `REPAIR_DISPOSITIONS`,
  `repairReplacements`,
  `describeAdoptedRound`,
  `RepairTranslationResult`.
  Declared in `repair-assemble.ts`,
  `repair-record.ts`,
  `repair-replacements.ts`,
  `repair-round-record.ts`,
  and `repair-result.ts`.
- Repair contract:
  refuse an editor roster that could not decide a round,
  a checker roster that repeats a model or,
  unless self-certification is permitted,
  includes an editor,
  and a checker roster too small for a disagreement to resolve,
  and name the roster shape of a repair run by role.
  Names:
  `assertCheckerIndependence`,
  `assertCheckerQuorumReachable`,
  `assertJudgeableEditorRoster`,
  `CheckerIndependenceError`,
  `ProducerRosterError`,
  `RepairModels`.
  Declared in `repair-contract.ts`.
- Unheard voices:
  say whether the repair lane heard no voice at all about a slice,
  and refuse a slice it heard nobody about that carries anything but the archive's own wording.
  Names:
  `assertUnheardKeptArchive`,
  `heardNobodyAbout`,
  `RepairUnheardError`.
  Declared in `repair-unheard.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `document-structure`
- `house-rule`
- `markup`
- `page-pass`
- `shared-contract`
- `slice-reading`
- `translate-floor`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `archive`
- `artifact`
- `artifact-pool`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `lane-contest`
- `page-check`
- `probe`
- `repair`
- `run`
- `slice-cache`
- `translate-lane`
- `translate-stage`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/assembly:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/assembly:test:unit`,
or build and test together with `mise run //package/translation-repair/assembly:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `sunbathing-recheck.test-fixture.ts` and `issue-author-fixtures.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
