# @monochromatic-dev/translation-repair-coverage-census

The coverage census and its control and probe:
it finds the family code no unit test ran,
by running every package's unit suite under one V8 coverage directory against unminified builds with source maps
and mapping the cold code back to source lines named `<package>/src/<file>`.
The census is its own package because it builds coverage bundles and reads sources across the family,
and no production package depends on it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Census run and report:
  run the suite under coverage,
  tally the cold code,
  and print the census.
  Names:
  `runCoverageCensus`,
  `reportCensus`,
  `readCensusArguments`,
  `CENSUS_FORMAT`,
  `requireCoverageBuild`,
  `censusStretchesOf`,
  `sourceRowsOf`.
  Declared in `corpus-run/coverage-census-run.ts`,
  `corpus-run/coverage-census-reading.ts`,
  `corpus-run/coverage-census-input.ts`,
  `corpus-run/coverage-census-build.ts`,
  and `corpus-run/coverage-census-report.ts`.
- Family scope and commit:
  find what the census measures from the directory it runs in,
  the family of packages or the package alone,
  group named test files by the member whose suite runs them,
  refuse a member's coverage build the census cannot read,
  and name the files edited since the commit a saved census records.
  Names:
  `censusScopeOf`,
  `familySuiteRunsOf`,
  `requireFamilyCoverageBuild`,
  `isFamilyBundleName`,
  `packageCommit`,
  `sourcesEditedSince`.
  Declared in `corpus-run/coverage-census-family.ts` and `corpus-run/coverage-census-commit.ts`.
- Mapping coverage to source:
  read the coverage files and source maps of the bundles and place each cold stretch on source lines.
  Names:
  `bundleScriptsOf`,
  `bundleMapsOf`,
  `readSourceMap`,
  `mapStretch`,
  `placeTally`,
  `createCoverageTally`.
  Declared in `corpus-run/coverage-file.ts`,
  `corpus-run/coverage-bundle-maps.ts`,
  `corpus-run/coverage-lines.ts`,
  `corpus-run/coverage-pieces.ts`,
  `corpus-run/coverage-census-place.ts`,
  and `corpus-run/coverage-tally.ts`.
- Baselines and invariant throws:
  read a saved census as a baseline,
  and count the stretches that are nothing but invariant throws apart from the cold code.
  Names:
  `readBaselineCensus`,
  `baselineStatusesOf`,
  `coldSinceOf`,
  `invariantThrowRowsOf`,
  `invariantThrowCountLine`.
  Declared in `corpus-run/coverage-census-input.ts`,
  `corpus-run/coverage-census-baseline.ts`,
  and `corpus-run/coverage-census-invariant.ts`.
- Coverage control and probe:
  ask the coverage roster about passages,
  and delete the spans it anchored on to see whether an absence vote is reachable.
  Names:
  `runCoverageControl`,
  `coverageControlHolds`,
  `withoutSpans`,
  `runCoverageProbe`,
  `readCoverageProbeArguments`,
  `DEFAULT_CANDIDATE_CAP`.
  Declared in `corpus-run/coverage-control-probe-run.ts`,
  `corpus-run/coverage-control.ts`,
  `corpus-run/coverage-probe-run.ts`,
  and `corpus-run/coverage-probe-args.ts`.
- Runner entries:
  the build entry lists of every package that owns runners,
  merged into one so that the census tells a runner's command-line main from library source,
  and the sources it treats as runner entries.
  Names:
  `FAMILY_ENTRY_LISTS`,
  `familyEntriesOf`,
  `FamilyEntryConflictError`,
  `nodeEntries`,
  `runnerEntrySources`.
  Declared in `family-build-entries.ts`;
  this package's own list is in `build-entries.ts`.

## Runners

`src/build-entries.ts` lists the library index and each runner as a build entry,
and each runner is built to `dist/final/node/<runner>.mjs`.
Every runner has a task of the same name in `mise.toml`:
`coverage-probe` and `coverage-control-probe` build first,
and `coverage-census` runs `coverage-census:build` first.
Start one with `mise run //package/translation-repair/coverage-census:<task>`,
passing arguments after `--`.
A built command refuses to run in a process that holds provider keys unless `TRANSLATION_REPAIR_STARTED_BY` names
that command,
and each task sets it to its own name.

- `coverage-probe`:
  a prototype for the coverage question:
  asks a roster whether the translation already carries each passage the aligners refuse to pair,
  at both scales.
  Pass `--only ids` and `--cap n` after `--`.
  Spends quota.
- `coverage-control-probe`:
  asks the coverage roster about a passage,
  deletes the spans it anchored on,
  and asks again,
  to find out whether an absence vote is reachable at all.
  Pass `--only ids` after `--`.
  Spends quota.
- `coverage-census`:
  builds every family package unminified with source maps,
  runs each package's unit suite under one V8 coverage directory,
  and prints the code no test ran,
  mapped to source lines named `<package>/src/<file>`,
  with each stretch that is nothing but invariant throws counted and listed apart.
  Name unit test files from the family directory (`<package>/src/<file>.unit.test.ts`) after `--`
  to run only the packages they stand in.
  Add `--baseline <census.json>` and `--source <package/src/file.ts>` after `--` to read a saved census against
  this run.
  It runs `coverage-census:build` first and rebuilds every normal build afterwards.
  The whole suite writes about 8 GB of coverage under `~/.cache/translation-repair/coverage`,
  deleted once read.
  Spends no quota.

The task `coverage-census:build` is not a runner:
it runs after the `build:js:node` of every family package,
then rebuilds the build directory of each package that has a `rolldown.coverage.config.ts`,
with a source map beside every chunk and unminified,
because compression folds guards into expressions that V8 gives no range for.
It is never a child of `build`,
whose fan-out would run it beside the normal build into the same directories.

`src/build-entries.ts` lists only this package's three runners.
The census reads the union of every runner package's list through `src/family-build-entries.ts`,
so the runners of `bench`,
`corpus-pass`,
`probe`,
`run` and `slice-cache` count as runner entries too.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `archive`
- `bench`
- `corpus-source`
- `document-structure`
- `glossary`
- `insertion`
- `markup`
- `probe`
- `provider`
- `provider-stream`
- `reference`
- `run`
- `shared-contract`
- `slice-cache`
- `slice-reading`
- `stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `adjudication`
- `corpus-pass`

### Used by

No family package lists it in `dependencies` or `devDependencies`.

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/coverage-census:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/coverage-census:test:unit`,
or build and test together with `mise run //package/translation-repair/coverage-census:buildAndTest`.
Pass test file paths to run only those files.

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
