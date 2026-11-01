# @monochromatic-dev/translation-repair-translate-lane

The translate lane over a document:
every prepared slice is translated afresh from the Chinese,
judged against what the archive already had,
and the accepted text is assembled back into a whole translation.
It is its own package because `artifact`,
`corpus-pass`,
`page-check`,
`run` and `slice-cache` drive it as one unit,
and it sits above `consolidate`,
`repair` and `translate-stage`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Document drivers:
  translate a document,
  or run both lanes over one preparation without arbitrating between them.
  Names:
  `translateDocument`,
  `runDocumentLanes`,
  `translateLaneWordings`,
  `assembleTranslation`,
  `wrapTranslateRecords`.
  Declared in `translate-document.ts`,
  `document-lanes.ts`,
  `translate-lane-wordings.ts`,
  `translate-assemble.ts`,
  and `translate-wrap.ts`.
- Slice settlement:
  attempt and settle one slice,
  and key its cache.
  Names:
  `settleTranslateSlice`,
  `attemptTranslateSlice`,
  `translateSliceInput`,
  `translateSliceKey`,
  `TRANSLATE_SLICE_CACHE_VERSION`,
  `buildSliceSelections`.
  Declared in `translate-slice.ts`,
  `translate-slice-attempt.ts`,
  `translate-slice-input.ts`,
  `translate-slice-key.ts`,
  `translate-document-contract.ts`,
  and `slice-selection.ts`.
- Slice alignment:
  judge whether a slice's archive text aligns with its source and name the refusals.
  Names:
  `assessSliceAlignment`,
  `alignmentRefusals`,
  `alignmentRefusalFinding`,
  `MAX_INCUMBENT_TO_SOURCE_RATIO`.
  Declared in `translate-alignment.ts` and `translate-alignment-refusals.ts`.
- Target-only runs:
  hold out English the archive carries that the Chinese never said,
  and splice it back unchanged.
  Names:
  `splitTargetOnlyRun`,
  `restoreTargetOnlyRun`,
  `TargetOnlySplit`.
  Declared in `target-only-run.ts`.
- Unheard slices and readings:
  refuse a record that heard no translator yet changed the archive text,
  and read the pictures of a document.
  Names:
  `assertUnheardKeptIncumbent`,
  `TranslateUnheardError`,
  `heardNobody`,
  `readDocumentPictures`.
  Declared in `translate-unheard.ts` and `document-readings.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `archive`
- `assembly`
- `consolidate`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `lane-contest`
- `markup`
- `page-pass`
- `provider`
- `provider-stream`
- `reference`
- `repair`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `adjudication`

### Used by

Packages that list it in `dependencies`:

- `artifact`
- `corpus-pass`
- `page-check`
- `run`
- `slice-cache`

Packages that list it only in `devDependencies`:

- `probe`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/translate-lane:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/translate-lane:test:unit`,
or build and test together with `mise run //package/translation-repair/translate-lane:buildAndTest`.
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
