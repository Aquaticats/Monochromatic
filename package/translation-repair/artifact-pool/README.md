# @monochromatic-dev/translation-repair-artifact-pool

The pool of settled artifacts and the comparison of the two lanes' rows:
the guards that parse an artifact,
the eligible pool and its policy,
artifact file names,
generations and provenance,
and row equality.
It is its own package because `artifact`,
`lane-contest`,
the passes and the probes all select from one pool,
and the `artifact` package,
which builds and reads whole artifacts,
sits above it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Parse guards:
  require the shape of each part of an artifact record,
  refuse a malformed one,
  and parse one issue record's repair provenance.
  Names:
  `ArtifactParseError`,
  `requireString`,
  `requireRecord`,
  `requireArray`,
  `requireCount`,
  `parseRecordRepair`.
  Declared in `artifact-guard.ts` and `artifact-repair-read.ts`.
- Eligible pool:
  select the entries a pool may use under its policy,
  and refuse an empty or mixed-generation pool.
  Names:
  `selectEligible`,
  `resolvePool`,
  `readPoolPolicy`,
  `keepEligible`,
  `EmptyPoolError`,
  `MixedGenerationError`.
  Declared in `corpus-run/artifact-eligible.ts`,
  `corpus-run/artifact-pool.ts`,
  and `corpus-run/artifact-pool-refusal.ts`.
- Artifact files:
  where artifacts live and how their names map to entries.
  Names:
  `ARTIFACTS_DIR`,
  `artifactsDirOf`,
  `artifactFileNameOf`,
  `entryIdOfArtifact`,
  `listArtifactFiles`,
  `isArtifactFileName`.
  Declared in `corpus-run/artifact-file-name.ts`.
- Generations and provenance:
  partition the settled artifacts by the pipeline that produced them,
  resolve a named commit to a full commit id,
  say whether one pipeline commit contains another,
  read which pipeline one artifact records,
  and refuse a loaded artifact that is not the one the pool admitted.
  Names:
  `censusByGeneration`,
  `resolveCommit`,
  `tipContains`,
  `assertArtifactProvenance`,
  `ArtifactProvenanceError`,
  `readPlacement`.
  Declared in `corpus-run/artifact-generation.ts`,
  `corpus-run/artifact-provenance.ts`,
  and `corpus-run/artifact-placement.ts`.
- Lane comparison and row equality:
  derive the comparison of the two lanes from their delivery ledgers,
  refuse a run whose frozen and live derivations disagree,
  name the fields where two comparison rows differ,
  and say how a rebuilt carve departs from the one an artifact records.
  Names:
  `compareLanes`,
  `assertDerivationsAgree`,
  `ArtifactComparisonError`,
  `comparisonRowDifferences`,
  `decisionsEqual`,
  `outcomesEqual`,
  `carveDivergence`.
  Declared in `corpus-run/artifact-two-lane-comparison.ts`,
  `corpus-run/artifact-two-lane-comparison-fault.ts`,
  `corpus-run/artifact-two-lane-row-equality.ts`,
  and `corpus-run/artifact-two-lane-rebuild-rows.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `assembly`
- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `shared-contract`
- `slice-reading`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `adjudication`

### Used by

Packages that list it in `dependencies`:

- `artifact`
- `bench`
- `corpus-pass`
- `insertion`
- `lane-contest`
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
`mise run //package/translation-repair/artifact-pool:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/artifact-pool:test:unit`,
or build and test together with `mise run //package/translation-repair/artifact-pool:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/pool-policy.test-fixture.ts`,
`sole-lane-comparison-row.test-fixture.ts`,
and `two-lane-comparison-row.test-fixture.ts`.

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
