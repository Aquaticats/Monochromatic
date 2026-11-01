# @monochromatic-dev/translation-repair-lane-contest

The contest between the lanes over a contested slice,
the comparison and delivery checks that follow it,
and the seeded-error and sampling tools that measure the pipeline.
A contest asks the roster which candidate the original supports and counts the answers,
so no single model decides.
It is its own package because the passes,
the artifact builder,
the page checks and the probes all read the contest records,
and it sits above `stage`,
`translate-floor` and `translate-stage`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Lane contest:
  settle one contested slice by asking the roster which candidate the original supports,
  and persist the outcome.
  Names:
  `contestLaneSlice`,
  `LANE_CONTEST_QUORUM`,
  `buildLaneContestMessages`,
  `settleLaneContestBallots`,
  `laneContestSliceKey`,
  `persistLaneContestOutcome`.
  Declared in `lane-contest-stage.ts`,
  `lane-contest-wire.ts`,
  `lane-contest-key.ts`,
  and `lane-contest-persist.ts`.
- Lane comparison and delivery checks:
  compare what the two lanes did to the same slice,
  and refuse a delivery that is incoherent or disagrees with the document.
  Names:
  `compareDocumentLanes`,
  `LaneComparisonError`,
  `assertDeliveryCoherent`,
  `DeliveryCoherenceError`,
  `assertDeliveryAgreesWithDocument`,
  `DeliveryInvariantError`.
  Declared in `lane-comparison.ts`,
  `delivery-coherence.ts`,
  and `delivery-invariants.ts`.
- Contest eligibility:
  compute the deterministic publication eligibility of front matter candidates,
  name the candidates the prompt marks unavailable before the panel votes,
  and settle the ballots with the votes for an ineligible candidate excluded,
  the raw ballots left unchanged for audit.
  Names:
  `applyLaneContestEligibility`,
  `frontMatterContestEligibility`,
  `laneContestChoiceVerdict`,
  `settleEligibleLaneContestBallots`,
  `ineligibleCandidatesOf`.
  Declared in `lane-contest-eligibility.ts`.
- Seeded errors:
  plant known errors in a translation and grade whether the pipeline detected them.
  Names:
  `applySeededErrors`,
  `SeedApplicationError`,
  `gradeSeedDetection`,
  `seedHitByRegion`,
  `SEED_MATCH_TOLERANCE`.
  Declared in `seeded-error.ts` and `seed-detection.ts`.
- Probe scoring and sample manifests:
  score the introduced-defect probe against human repair grades,
  apply its majority rule to a region and summarize its telemetry over the regions that shipped,
  and bind a graded sheet to the manifest it was drawn under.
  Names:
  `scoreProbeAgainstGrades`,
  `summarizeProbeTelemetry`,
  `judgeRegionProbe`,
  `buildSampleManifest`,
  `parseSampleManifest`,
  `assertSheetMatchesManifest`.
  Declared in `probe-agreement.ts`,
  `probe-telemetry.ts`,
  `sample-manifest.ts`,
  and `sheet-binding.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `artifact-pool`
- `assembly`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `page-pass`
- `provider`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `archive`
- `artifact`
- `consolidate`
- `corpus-pass`
- `insertion`
- `page-check`
- `probe`
- `repair`
- `run`
- `slice-cache`
- `translate-lane`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/lane-contest:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/lane-contest:test:unit`,
or build and test together with `mise run //package/translation-repair/lane-contest:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/score-sheets.test-fixture.ts` and `slice-covers-application.test-fixture.ts`.

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
