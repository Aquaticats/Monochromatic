# @monochromatic-dev/translation-repair-translate-floor

The translate lane's floors and the pieces it builds a slice answer from:
the validation findings a translated slice must clear,
the translate prompt and wire format,
candidate slates,
slice delivery records,
and the slice cost log.
It is its own package because the translate stage,
the translate lane,
the assembly checks and the artifact reader all share these floors,
and it sits above text,
provider and `stage` packages.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Floors and validation:
  findings raised for a translated slice that does not carry the references and code atoms it owes,
  repeats a Han-carrying original untranslated,
  leaves Han text in its English prose,
  forms more formulas than the original,
  carries a label of the sheet's evidence,
  or unwraps a worded link and keeps its destination.
  Names:
  `validateTranslatedSlice`,
  `atomFindings`,
  `hanResidueFindings`,
  `untranslatedFindings`,
  `addedFormulaFindings`,
  `sheetLeakFindings`,
  `unwrappedLinkFindings`.
  Declared in `translate-validate.ts`,
  `translate-atom-rendering.ts`,
  `translate-han-residue.ts`,
  `translate-untranslated.ts`,
  `translate-formula.ts`,
  `translate-sheet-leak.ts`,
  and `translate-unwrapped-link.ts`.
- Translate prompts and wire:
  the translate prompt,
  its response format,
  the repair prompt for invalid candidates and the selection criteria.
  Names:
  `buildTranslateMessages`,
  `TRANSLATE_RESPONSE_FORMAT`,
  `isTranslateReportWire`,
  `buildTranslateRepairMessages`,
  `translateSelectionCriteria`,
  `translateSelectionTask`.
  Declared in `translate-wire.ts`,
  `translate-repair-wire.ts`,
  and `translate-selection-sheet.ts`.
- Candidates and slates:
  build the candidate set for a slice,
  rotate the slate,
  fold the voices of several translators,
  and measure whether a producer favours its own work.
  Names:
  `buildTranslateCandidates`,
  `laneCandidate`,
  `describeSlate`,
  `rotateCandidates`,
  `foldTranslatorVoices`,
  `repairInvalidCandidates`,
  `selfPreference`.
  Declared in `translate-candidates.ts`,
  `translate-slate.ts`,
  `translator-answer-fold.ts`,
  `translate-repair.ts`,
  and `self-preference.ts`.
- Slice delivery:
  record what became of every slice and refuse delivery that contradicts the document or an earlier record.
  Names:
  `buildSliceDelivery`,
  `SliceDeliveryError`,
  `buildLaneSliceTexts`,
  `validateNamedSets`,
  `assertSettledRecordAgrees`,
  `LaneSliceCoverageError`.
  Declared in `slice-delivery.ts`,
  `slice-delivery-fault.ts`,
  `lane-slice-text.ts`,
  `lane-slice-sets.ts`,
  `slice-record-agreement.ts`,
  and `lane-slice-coverage-error.ts`.
- Slice cost:
  write the cost telemetry of each slice and read it back.
  Names:
  `armSliceCost`,
  `readSliceCosts`,
  `SLICE_COST_MARKER`,
  `SLICE_COST_LANES`.
  Declared in `slice-cost-log.ts` and `slice-cost-read.ts`.
- Slice pairs and bounds:
  subdivide a chunk pair,
  partition blocks into runs a slice can carry,
  bound the volume a lane may produce,
  and flag implausible slices.
  Names:
  `subdivideChunkPair`,
  `SLICE_CHAR_BUDGET`,
  `groupNodes`,
  `producedVolumeBound`,
  `sliceImplausibility`.
  Declared in `slice-pair.ts`,
  `group-nodes.ts`,
  `produced-volume-bound.ts`,
  and `slice-implausible.ts`.

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
- `provider-stream`
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
- `assembly`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `lane-contest`
- `page-check`
- `repair`
- `run`
- `slice-cache`
- `translate-lane`
- `translate-stage`

Packages that list it only in `devDependencies`:

- `probe`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/translate-floor:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/translate-floor:test:unit`,
or build and test together with `mise run //package/translation-repair/translate-floor:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/lane-result-evidence.test-fixture.ts`,
`corpus-run/slice-cost-rows.test-fixture.ts`,
and `group-aligned.test-fixture.ts`.

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
