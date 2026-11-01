# @monochromatic-dev/translation-repair-artifact

The two-lane settled artifact:
building it from a run's results,
reading it back with strict key and relation checks,
rebuilding the preparation a run settled with,
and checking its measurements.
It is its own package because the passes write artifacts and the probes,
reports and page checks read them,
and it sits above nearly every lane package.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Building:
  assemble a two-lane artifact in one place and project its rows.
  Names:
  `buildSettledTwoLaneArtifact`,
  `toArtifactRow`,
  `toArtifactDecisions`,
  `toArtifactDelivery`,
  `toArtifactOutcome`,
  `projectLanes`.
  Declared in `corpus-run/artifact-two-lane-build.ts`,
  `corpus-run/artifact-two-lane-project.ts`,
  and `corpus-run/artifact-two-lane-derive.ts`.
- Reading:
  read a settled artifact of any known generation,
  with every key and relation checked.
  Names:
  `readSettledArtifact`,
  `parseSettledArtifact`,
  `parseSettledTwoLaneArtifact`,
  `readArtifactSchemaVersion`,
  `KNOWN_ARTIFACT_SCHEMA_VERSIONS`.
  Declared in `artifact-read.ts`,
  `artifact-v1-read.ts`,
  `corpus-run/artifact-two-lane-read.ts`,
  and `artifact-schema-version.ts`.
- Schema versions and generations:
  the version constants,
  generations and key vocabularies the reader tells artifacts apart by.
  Names:
  `ARTIFACT_SCHEMA_VERSION_V1`,
  `ARTIFACT_SCHEMA_VERSION_V2`,
  `TWO_LANE_GENERATIONS`,
  `isTwoLaneArtifactGeneration`,
  `keyVocabularyOf`,
  `UnknownArtifactGenerationError`.
  Declared in `artifact-schema-version.ts`,
  `corpus-run/artifact-two-lane-contract.ts`,
  and `artifact-key-vocabulary.ts`.
- Consistency and consolidation sections:
  the shapes and parsers of the sections that record the consistency pass and the consolidation.
  Names:
  `parseConsistency`,
  `parseConsolidation`,
  `parseConsolidationPolish`,
  `ArtifactConsistency`,
  `ArtifactConsolidation`,
  `NO_CONSISTENCY`.
  Declared in `corpus-run/artifact-two-lane-read-consistency.ts`,
  `corpus-run/artifact-two-lane-read-consolidate.ts`,
  `corpus-run/artifact-two-lane-read-polish.ts`,
  `corpus-run/artifact-two-lane-consistency.ts`,
  and `corpus-run/artifact-two-lane-consolidate.ts`.
- Rebuilding and verification:
  carve a document pair the way the run that settled the artifact carved it,
  and check the artifact describes it.
  Names:
  `rebuildPreparation`,
  `recipeOf`,
  `readSettledEntryInfo`,
  `ArtifactPreparationMismatchError`,
  `assertLedgerDescribesPreparation`,
  `verifyArtifactMeasurements`.
  Declared in `corpus-run/artifact-two-lane-rebuild.ts`,
  `corpus-run/artifact-two-lane-verify.ts`,
  and `corpus-run/artifact-two-lane-corpus-verify.ts`.
- Guards and probe readings:
  exact-key guards,
  row coherence checks and the reading of probe results kept in an artifact.
  Names:
  `requireExactKeys`,
  `requireOneOf`,
  `assertRowsCoherent`,
  `assertEvidenceMatchesLedger`,
  `readArtifactProbe`,
  `parseRegionTally`.
  Declared in `artifact-exact-guard.ts`,
  `corpus-run/artifact-two-lane-read-row-relations.ts`,
  `artifact-probe-read.ts`,
  and `artifact-probe-tally.ts`.
- Change sets and repair lane records:
  read which slices a settled artifact says its document changed,
  and the repair lane's two record lists out of one artifact.
  Names:
  `readArtifactChangeSets`,
  `repairLaneRecordsOf`.
  Declared in `artifact-change-sets.ts` and `artifact-repair-lane-records.ts`.
- Consolidation helpers:
  the archive wording a consolidation settlement takes,
  whether a majority of contest ballots call the archive flawed,
  the lane texts a consolidation slate offers beside the writers' proposals,
  and the tense candidates added to a record that ships wording.
  Names:
  `archiveStandInFor`,
  `archiveFlawedByMajority`,
  `laneTextsForSlate`,
  `withShippedTenseCandidates`.
  Declared in `consolidate-archive-stand-in.ts`,
  `consolidate-archive-flawed.ts`,
  `consolidate-lane-offer.ts`,
  and `consolidate-shipped-tense.ts`.
- Page titles,
  displacement and consolidation keys:
  settle page titles,
  classify what a size anomaly means,
  and key the consolidation cache.
  Names:
  `settlePageTitles`,
  `pageTitleLines`,
  `classifyDisplacement`,
  `consolidateSliceKey`,
  `CONSOLIDATE_CACHE_VERSION`.
  Declared in `page-title-lexicon-stage.ts`,
  `displacement-class.ts`,
  and `consolidate-key.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `archive`
- `artifact-pool`
- `assembly`
- `consistency`
- `consolidate`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `insertion`
- `lane-contest`
- `markup`
- `page-pass`
- `provider`
- `rendering-audit`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `repair`

### Used by

Packages that list it in `dependencies`:

- `bench`
- `corpus-pass`
- `page-check`
- `probe`
- `run`
- `slice-cache`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/artifact:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/artifact:test:unit`,
or build and test together with `mise run //package/translation-repair/artifact:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/settled-artifact.test-fixture.ts`,
`corpus-run/consistency-group.test-fixture.ts`,
and `corpus-run/score-probe-artifacts.test-fixture.ts`.

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
