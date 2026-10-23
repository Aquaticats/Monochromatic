# @monochromatic-dev/translation-repair-slice-cache

The slice and consolidation caches that make a long pass resumable,
the cache-account audit that checks cache versions against the commits that changed them,
the slice census,
and the pass bookkeeping around them:
schema and generation guards,
eligibility,
attempt queues and republishing.
It is its own package because `corpus-pass` and `bench` share these stores,
and it sits above `artifact`,
`consolidate`,
`lane-contest`,
`translate-lane` and `run`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Slice cache stores and namespaces:
  one disk-backed store for each kind of question,
  each owning a file prefix in an entry's directory.
  Names:
  `openSliceCache`,
  `openTranslateSliceCache`,
  `openRefineSliceCache`,
  `openPairingCache`,
  `openConsolidateCache`,
  `openLaneContestCache`,
  `openPictureReadingCache`,
  `discardSliceCache`,
  `listResumableEntries`.
  Declared in `corpus-run/slice-cache-store.ts`,
  `corpus-run/consolidate-cache-store.ts`,
  `corpus-run/lane-contest-cache-store.ts`,
  and `corpus-run/reading-cache-store.ts`.
- Cache-account audit:
  read every cache version constant,
  the commit that set it,
  and the source commits no version's account names.
  Names:
  `auditCacheAccounts`,
  `cacheVersionsIn`,
  `unaccountedCommits`,
  `sliceCacheAccount`,
  `runsDirsUnder`.
  Declared in `corpus-run/cache-account-run.ts`,
  `corpus-run/cache-account-read.ts`,
  `corpus-run/cache-account-commits.ts`,
  and `corpus-run/cache-account-slices.ts`.
- Slice census and cost reports:
  measure the pinned corpus after slicing,
  and bucket slice costs by size.
  Names:
  `gatherSliceCensus`,
  `reportSliceCensus`,
  `censusEntry`,
  `reportSliceCost`,
  `bucketSliceCostsBySize`.
  Declared in `corpus-run/slice-census-gather.ts`,
  `corpus-run/slice-census-report.ts`,
  `corpus-run/slice-census-entry.ts`,
  `corpus-run/slice-cost-report-run.ts`,
  and `corpus-run/slice-cost-bands.ts`.
- Pass schema and generation guards:
  refuse to resume into a directory produced by a different build or schema generation.
  Names:
  `censusBySchema`,
  `assertResumableSchemaGeneration`,
  `SchemaGenerationError`,
  `assertBuildGenerationResumable`,
  `GenerationDriftError`,
  `assertArtifactsPlaceable`.
  Declared in `corpus-run/pass-schema-census.ts`,
  `corpus-run/pass-schema-guard.ts`,
  and `corpus-run/pass-generation-guard.ts`.
- Eligibility and pass bookkeeping:
  decide which entries a pass may work on,
  queue attempts,
  record declines,
  and republish the pages of a run.
  Names:
  `collectEligiblePairs`,
  `runAttemptQueue`,
  `finishedEntryIds`,
  `recordEntryDecline`,
  `removeDeclinedPages`,
  `republishRunPages`,
  `stopBeforeNextEntry`.
  Declared in `corpus-run/pass-eligibility.ts`,
  `corpus-run/entry-attempt-queue.ts`,
  `corpus-run/pass-finished.ts`,
  `corpus-run/pass-decline.ts`,
  `corpus-run/pass-republish.ts`,
  and `corpus-run/pass-stop-before-next.ts`.
- Overlap and error tallies:
  read the slice overlap setting and cap the text of tallied errors.
  Names:
  `readOverlap`,
  `CALIBRATION_OVERLAP`,
  `tallyErrorText`,
  `TALLY_ERROR_CAP`,
  `entryErrorOutcome`.
  Declared in `corpus-run/slice-overlap.ts`,
  `corpus-run/tally-error-text.ts`,
  and `corpus-run/entry-error-outcome.ts`.

## Runners

`src/build-entries.ts` lists the library index and each runner as a build entry,
and each runner is built to `dist/final/node/<runner>.mjs`.
Every runner has a task of the same name in `mise.toml` that builds first.
Start one with `mise run //package/translation-repair/slice-cache:<task>`,
passing arguments after `--`.
A built command refuses to run in a process that holds provider keys unless `TRANSLATION_REPAIR_STARTED_BY` names
that command,
and each task sets it to its own name.

- `cache-account-audit`:
  before a launch,
  reads every cache version constant the source declares,
  the commit that set each current value,
  and every non-test source commit since the earliest of them that no version's account names,
  then the slice-cache records under the runs directories,
  the newest one,
  and how many versions were set after it.
  Read-only.
  Pass `--runs-under dir` after `--`,
  repeatable,
  to also search a directory holding runs directories.
  A launch that follows the runbook writes its runs under `${HOME}/temp/agent`,
  which the default roots do not reach,
  so pass `--runs-under "${HOME}/temp/agent"` for the account a launch would resume.
  A count of zero records prints a `WARNING:` line on stdout and a warning through the package logger,
  naming every root searched and that runs root,
  because the newest-record and versions-set-after lines then say nothing about the caches.
  Spends no quota.
- `slice-cost-report`:
  reads the per-slice cost telemetry a pass wrote and says whether cost tracks slice size or a fixed overhead.
  Pass a log path after `--`.
- `slice-census`:
  measures the pinned corpus after slicing:
  slice size spread,
  sections only one side carries,
  and blocks only the translation has.
  Spends no quota.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `artifact`
- `artifact-pool`
- `assembly`
- `consolidate`
- `corpus-source`
- `document-structure`
- `glossary`
- `insertion`
- `lane-contest`
- `markup`
- `page-check`
- `page-pass`
- `provider-stream`
- `repair`
- `run`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `provider`

### Used by

Packages that list it in `dependencies`:

- `bench`
- `corpus-pass`
- `coverage-census`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/slice-cache:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/slice-cache:test:unit`,
or build and test together with `mise run //package/translation-repair/slice-cache:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/slice-overlap-dial.test-fixture.ts`.

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
