# @monochromatic-dev/translation-repair-translate-stage

The translate stage:
every slice is translated from the original by several models independently,
the translation already in the archive stands as one more candidate,
and the judge roster chooses.
The package holds the slate production and judging,
run-offs of tied finalists,
retries,
and the present-tense candidates shown to judges.
It is its own package because the translate lane,
the consolidation lane and the lane contest all run this one stage,
and it sits above `stage` and `translate-floor`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- The translate stage:
  produce the slate,
  floor its voices and judge it.
  Names:
  `runTranslateStage`,
  `produceTranslateSlate`,
  `judgeTranslateSlate`,
  `floorTranslateVoices`,
  `TranslateStageResult`.
  Declared in `translate-stage.ts`,
  `translate-produce.ts`,
  `translate-judge.ts`,
  `translate-floor.ts`,
  and `translate-stage-result.ts`.
- Run-offs and retries:
  run off tied finalists,
  ask a panel that declined a slate once more over the same slate,
  and settle a slice no voice cleared the floor for.
  A declined round ships a candidate only when two or more judges named it,
  and otherwise the wording in place stays (the owner's answer of 2026-10-25,
  "Two judges must name it").
  A round tied across every offered candidate still ships one by a fixed preference
  (repair lane,
  then translate lane,
  then slate order).
  Names:
  `runoffFinalists`,
  `runoffTiedFinalists`,
  `judgeSlateWithRetry`,
  `settleUnflooredSlice`,
  `unflooredFinding`.
  Declared in `translate-runoff.ts`,
  `translate-retry.ts`,
  and `translate-unfloored.ts`.
- Candidate selection:
  run one selection round that names a winner or declines,
  record the contest that decided it,
  and size the judges asked to the seats that could answer.
  A winner needs `MIN_SELECTION_BALLOTS` ballots whatever their weight,
  by the owner's rulings of 2026-10-25.
  Names:
  `decideBestCandidate`,
  `selectBestCandidate`,
  `selectionFanOut`,
  `selectionBench`,
  `MIN_SELECTION_BALLOTS`,
  `shortBenchFinding`.
  Declared in `candidate-select.ts`,
  `candidate-select-record.ts`,
  `candidate-select-fanout.ts`,
  and `candidate-select-bench.ts`.
- Tense candidates:
  flag sentences that read as present narration of a life that has ended,
  as candidates for judges to check,
  and label where each came from.
  Names:
  `presentTenseCandidates`,
  `tenseCandidateGroups`,
  `tenseCandidateLines`,
  `shippedTenseLabelOf`,
  `maskQuotedSpeech`.
  Declared in `present-tense-candidates.ts`,
  `tense-candidate-block.ts`,
  `tense-candidate-labels.ts`,
  and `present-tense-text.ts`.
- Evidence and criteria:
  the evidence and criteria a judge sees,
  and the operational stop raised when an absent passage cannot form another unique repair task.
  Names:
  `translateSlateEvidence`,
  `translatedSlateCriteria`,
  `TranslationRepairInterruptedError`.
  Declared in `translate-slate-evidence.ts`,
  `translated-slate-criteria.ts`,
  and `translation-repair-interrupted-error.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `assembly`
- `corpus-source`
- `document-structure`
- `house-rule`
- `markup`
- `provider`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `glossary`

### Used by

Packages that list it in `dependencies`:

- `archive`
- `artifact`
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

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/translate-stage:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/translate-stage:test:unit`,
or build and test together with `mise run //package/translation-repair/translate-stage:buildAndTest`.
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
