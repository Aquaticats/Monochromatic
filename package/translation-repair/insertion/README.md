# @monochromatic-dev/translation-repair-insertion

Insertion checks and the corpus-run bookkeeping that sits beside them:
admitting insertions for container halves,
container deficits and referenced definitions,
folding a carried insertion into the slice that carries it,
reading entry records,
closing over the code a runner executed,
and ordering entries by size band.
It is its own package because `artifact`,
`corpus-pass`,
`probe`,
`slice-cache` and others share these checks,
and it sits below `corpus-pass`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Insertion admission:
  admit the other half of every container one of whose slices is admitted,
  admit passages no majority found carried while the archive's container holds fewer blocks than the original's,
  admit definitions that the shipping page references and does not define,
  fold a carried passage into the slice that carries it,
  and classify the latest rows as inserted,
  carried or unresolved.
  Names:
  `admitContainerDeficit`,
  `admitContainerHalves`,
  `admitReferencedDefinitions`,
  `foldCarriedInsertions`,
  `classifyInsertionCoverage`,
  `CARRIED_FOLDED_FINDING`.
  Declared in `corpus-run/insertion-container-deficit.ts`,
  `corpus-run/insertion-container-halves.ts`,
  `corpus-run/insertion-referenced-definitions.ts`,
  `corpus-run/insertion-carried-fold.ts`,
  and `corpus-run/insertion-coverage-model.ts`.
- Entry records and pool generation:
  read the `info.yml` of an entry at the pin,
  where its date of death is recorded,
  and name the generation a draw's pool was settled under.
  Names:
  `readEntryInfoRecord`,
  `entryInfoFindingsOf`,
  `EntryInfoParseError`,
  `poolGeneration`.
  Declared in `corpus-run/entry-info-record.ts` and `corpus-run/pool-generation.ts`.
- Runner closure:
  name the code a run actually executed,
  which the pipeline digest of the whole built tree cannot say.
  Names:
  `readRunnerClosure`,
  `RunnerClosure`.
  Declared in `corpus-run/runner-closure.ts`.
- Band order:
  order the entries a corpus pass starts so coverage fills every size band at the same pace.
  Names:
  `bandOf`,
  `rankWithinBands`,
  `countSettledPerBand`,
  `SMALL_PAGE_BYTES`,
  `MEDIUM_PAGE_BYTES`.
  Declared in `corpus-run/band-order.ts`.
- Markup-only slices and provider listing:
  tell slices that are only markup,
  and read a provider's model listing within a timeout,
  refusing a listing that did not come back whole.
  Names:
  `isMarkupOnly`,
  `markupFraction`,
  `readProviderListing`,
  `LISTING_TIMEOUT_MS`.
  Declared in `corpus-run/markup-slice.ts` and `corpus-run/provider-listing.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `archive`
- `artifact-pool`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `lane-contest`
- `markup`
- `provider-stream`
- `shared-contract`
- `slice-reading`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `adjudication`
- `page-pass`

### Used by

Packages that list it in `dependencies`:

- `artifact`
- `bench`
- `corpus-pass`
- `coverage-census`
- `probe`
- `slice-cache`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/insertion:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/insertion:test:unit`,
or build and test together with `mise run //package/translation-repair/insertion:buildAndTest`.
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
