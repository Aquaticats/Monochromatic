# @monochromatic-dev/translation-repair-adjudication

Critics,
the adjudication panel,
candidate selection and introduced-defect screening:
the prompts and wire formats those stages send and read,
the ballots they tally,
and the screen that tests a repair for defects it brought in.
It is its own package because repair,
consolidation,
assembly checks and the page checks all read the same ballot rules,
and it sits above the text,
provider and `stage` packages.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Panel adjudication:
  run a panel over the claimed issues of a slice and tally its votes.
  A claim is decided when `MIN_PANEL_BALLOTS` panelists answer it,
  whatever their weight,
  by the owner's rulings of 2026-10-25;
  one panelist alone never decides.
  Names:
  `runPanelStage`,
  `buildAdjudicationMessages`,
  `ADJUDICATION_RESPONSE_FORMAT`,
  `resolvePanelBallot`,
  `usablePanelBallotFor`,
  `MIN_PANEL_BALLOTS`,
  `tallyVotes`.
  Declared in `panel-stage.ts`,
  `adjudicate-prompt.ts`,
  `adjudicate-wire.ts`,
  and `tally-votes.ts`.
- Critics:
  the critic prompt,
  its wire format and the record of which critic raised which claim.
  Names:
  `buildCriticMessages`,
  `criticOriginalsOf`,
  `CRITIC_RESPONSE_FORMAT`,
  `resolveCriticIssue`,
  `buildSliceCriticRecords`,
  `collectClaimAttributions`.
  Declared in `critic-prompt.ts`,
  `critic-wire.ts`,
  and `critic-attribution.ts`.
- Candidate selection:
  weigh candidate ballots and pick the winning repair candidate.
  Names:
  `selectRepairCandidate`,
  `compareCandidates`,
  `buildCandidateSelectMessages`,
  `selectDecision`,
  `countBallots`,
  `CandidateSlateError`.
  Declared in `select-candidate.ts`,
  `candidate-select-wire.ts`,
  `candidate-select-decision.ts`,
  and `candidate-select-count.ts`.
- Introduced-defect screening:
  probe a repaired text for defects the edit introduced,
  and screen the claims that come back.
  Names:
  `runIntroducedDefectProbe`,
  `screenIntroducedDefects`,
  `buildIntroducedDefectMessages`,
  `INTRODUCED_DEFECT_VERDICTS`,
  `screenEvidence`,
  `proberOriginalOf`,
  `resolveProberChecks`.
  Declared in `introduced-defect-probe.ts`,
  `introduced-defect-screen.ts`,
  `introduced-defect-shown.ts`,
  and `introduced-defect-wire.ts`.
- Resolution checks:
  ask checkers whether each issue is resolved and tally their sheets.
  Names:
  `buildResolutionMessages`,
  `RESOLUTION_VERDICTS`,
  `usableResolutionReportFor`,
  `resolveResolutionChecks`,
  `tallyResolutionChecks`.
  Declared in `resolution-wire.ts` and `tally-resolution.ts`.
- Producer standing and the start guard:
  rank producers by the share of ballots their work drew,
  and keep a built command from running by hand with provider keys held.
  Names:
  `producerStandings`,
  `rankStandings`,
  `readStandingCoverage`,
  `parseGradedRepairSheet`,
  `collectRepairRegions`,
  `taskRunnerVerdict`,
  `STARTED_BY_VARIABLE`.
  Declared in `producer-standing.ts`,
  `producer-standing-report.ts`,
  `producer-silence.ts`,
  `repair-grade-read.ts`,
  `repair-region.ts`,
  and `task-runner-guard.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `page-pass`
- `provider`
- `reference`
- `rendering-audit`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `archive`
- `artifact`
- `assembly`
- `bench`
- `consistency`
- `consolidate`
- `lane-contest`
- `page-check`
- `probe`
- `repair`
- `run`
- `slice-cache`
- `translate-stage`

Packages that list it only in `devDependencies`:

- `artifact-pool`
- `corpus-pass`
- `coverage-census`
- `insertion`
- `translate-lane`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/adjudication:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/adjudication:test:unit`,
or build and test together with `mise run //package/translation-repair/adjudication:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `child-environment.test-fixture.ts`,
`corpus-lacked-commit.test-fixture.ts`,
and `claim-attribution-proposer-ids.test-fixture.ts`.

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
