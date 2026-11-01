# @monochromatic-dev/translation-repair-slice-reading

Reads a document into slices and pictures:
chunking by headings,
aligning sections,
the image reading and OCR path,
slice coverage and indexing guards,
the block-pairing question and its outcomes,
and the grade sheets used to score precision.
It is its own package because the lanes and passes above it all work on slices,
and it needs only text,
provider and `stage` packages below it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Image reading and OCR:
  read a picture with models and with a local OCR program,
  and judge whether a reading makes sense.
  Names:
  `readImageAsset`,
  `readImagePair`,
  `readImageWithOcr`,
  `ocrReaderOver`,
  `IMAGE_READING_PERSPECTIVES`,
  `readingMakesSense`,
  `readPastRefusal`.
  Declared in `image-reading-stage.ts`,
  `image-reading-pair.ts`,
  `image-ocr.ts`,
  `image-reading-sense.ts`,
  and `image-reading-past-refusal.ts`.
- Chunked documents:
  cut a document into heading chunks,
  align their sections,
  and slice front matter.
  Names:
  `chunkByHeadings`,
  `alignDocumentSections`,
  `chunkLabel`,
  `frontMatterSlice`,
  `reindexSlicePair`,
  `describeAlignmentAttachment`.
  Declared in `chunk-document.ts`,
  `front-matter-slice.ts`,
  and `slice-indexing.ts`.
- Slice coverage and integrity:
  refuse slices that drop blocks,
  break indexing,
  leave a gap in a span or split a container.
  Names:
  `assertSliceCoverage`,
  `SliceCoverageError`,
  `assertSliceIndexing`,
  `SliceIndexingError`,
  `assertSpanContiguity`,
  `assertContainerIntegrity`,
  `containerHalfPairs`.
  Declared in `slice-coverage.ts`,
  `slice-indexing.ts`,
  `span-contiguity.ts`,
  `container-integrity.ts`,
  and `container-half-pairs.ts`.
- Slice pictures:
  render what is known of the pictures a slice shows and of its neighbours' pictures,
  name what is not known,
  and give a pairing sheet the transcripts of an aligned section's pictures.
  Names:
  `slicePictures`,
  `slicePictureContexts`,
  `slicePictureNames`,
  `pairingPictureContext`,
  `SIDE_LEGEND`.
  Declared in `slice-pictures.ts` and `pairing-pictures.ts`.
- Block pairing outcomes:
  the question put to the roster about unpaired blocks and the outcomes read back.
  Names:
  `pairBlocksWithRoster`,
  `blockPairingQuestion`,
  `blockPairingProtocol`,
  `readBlockPairingOutcomes`,
  `sectionPairingsOf`,
  `queriedBlockPairingDetails`.
  Declared in `pair-blocks-stage.ts`,
  `block-pairing-question.ts`,
  `block-pairing-protocol.ts`,
  `pair-blocks-read-outcomes.ts`,
  `section-pairing.ts`,
  and `queried-block-pairing-details.ts`.
- Cited references and grade sheets:
  fetch a cited reference for the critics,
  and read and score human grade sheets.
  Names:
  `fetchCitedReference`,
  `CITED_REFERENCE_RULE`,
  `citedReferenceEvidence`,
  `parseGradedSheet`,
  `scoreGradedPrecision`,
  `scoreGradeAgreement`,
  `parsePreGrades`.
  Declared in `cited-reference-fetch.ts`,
  `cited-reference-rule.ts`,
  `grade-sheet-read.ts`,
  and `grade-agreement.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `provider`
- `provider-stream`
- `shared-contract`
- `stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `insertion`
- `lane-contest`
- `page-check`
- `page-pass`
- `probe`
- `reference`
- `rendering-audit`
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
`mise run //package/translation-repair/slice-reading:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/slice-reading:test:unit`,
or build and test together with `mise run //package/translation-repair/slice-reading:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `content-slice-of.test-fixture.ts`,
`insertion-anchor-pair.test-fixture.ts`,
and `corpus-run/insertion-only-slice.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Reading the pictures a document shows](doc/pictures.md):
  the deterministic reader,
  the model readers and what is sent.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
