# @monochromatic-dev/translation-repair-corpus-source

Reads the pinned corpus and prepares an entry for the pipeline:
the corpus reader and its git access,
repeat groups of source wording,
the identity and seal of a prepared document pair,
insertion admission,
and the displacement ratios of slices.
It is its own package because corpus preparation is the entry point the passes,
the lanes and the reports all share,
and none of the family packages it lists in `dependencies` lists it back.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Corpus reader and git access:
  read files of the pinned corpus commit and list its people,
  through a git context that names the objects of its own clone and inherits no repository state,
  and index the Han names and aliases the entries declare.
  Names:
  `readCorpusFile`,
  `readCorpusBytes`,
  `listCorpusPeople`,
  `CORPUS_COMMIT_SHA`,
  `CorpusReadError`,
  `corpusGitEnvironment`,
  `readCorpusNames`.
  Declared in `corpus-source.ts`,
  `corpus-git-context.ts`,
  and `corpus-name-index.ts`.
- Repeat groups:
  find the maximal runs and refrain frames of repeated source wording,
  deterministically and without judging them.
  Names:
  `repeatGroupsOf`,
  `maximalRunsOf`,
  `refrainFramesOf`,
  `disjointRepeatsOf`,
  `compareFoundRepeats`.
  Declared in `repeat-groups.ts`,
  `repeat-groups-runs.ts`,
  `repeat-groups-refrain.ts`,
  `repeat-groups-overlap.ts`,
  and `repeat-groups-order.ts`.
- Preparation identity and seal:
  stamp a prepared document pair with an identity and report targets that are sealed or unclaimed.
  Names:
  `preparationIdentity`,
  `assertPreparationIdentity`,
  `PreparationIdentityError`,
  `chunkSealedTargets`,
  `sealedFinding`,
  `unclaimedFinding`.
  Declared in `preparation-identity.ts`,
  `preparation-seal.ts`,
  and `preparation-unclaimed.ts`.
- Insertion admission:
  choose which insertion slices the page has room to be missing,
  spending one shortfall budget for the whole page in document order,
  and say how an inserted fragment is joined to its neighbours.
  Names:
  `admitInsertions`,
  `composedInsertion`,
  `fragmentBody`,
  `documentLineEnding`.
  Declared in `insertion-admission.ts` and `insertion-separator.ts`.
- Displacement ratios:
  the size primitives the displacement classifier reads:
  each slice's size ratio,
  and the median of a document's ratios as its own baseline.
  Names:
  `sliceRatios`,
  `sliceSizesOf`,
  `documentBaseline`,
  `SliceBlockCountRefusalError`.
  Declared in `displacement-ratio.ts`.
- Archive notes and prepared sheets:
  read archive notes and footnote relabels,
  check that target spans can be written back to their document,
  give sheets the facts of a prepared pair,
  and render line breaks for prompts.
  Names:
  `footnoteRelabelOf`,
  `archiveOriginalReadingOf`,
  `assertPlacementLayout`,
  `PlacementLayoutError`,
  `preparedSheetFactsOf`,
  `renderedBreakContract`,
  `renderedBreakFacts`.
  Declared in `archive-footnote-relabel.ts`,
  `archive-original-note.ts`,
  `placement-layout.ts`,
  `prepared-sheet-facts.ts`,
  and `rendered-break-prompt.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `reference`
- `rendering-audit`
- `shared-contract`
- `slice-reading`
- `stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `bench`
- `consolidate`
- `corpus-pass`
- `coverage-census`
- `insertion`
- `lane-contest`
- `page-check`
- `page-pass`
- `probe`
- `repair`
- `run`
- `slice-cache`
- `translate-floor`
- `translate-lane`
- `translate-stage`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/corpus-source:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/corpus-source:test:unit`,
or build and test together with `mise run //package/translation-repair/corpus-source:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/ordered-page-reads.test-fixture.ts`.

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
