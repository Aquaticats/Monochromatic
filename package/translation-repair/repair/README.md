# @monochromatic-dev/translation-repair-repair

The repair lane and the refine lane,
with the editor ensemble they share.
The repair lane keeps the archive's English and mends it where critics find defects;
the refine phase then improves accurate but awkward text;
several editors rewrite each chunk and judges choose what ships,
an editor's ballot for its own candidate counting at the self-vote weight.
It is its own package because the lane is one unit that `consolidate`,
`corpus-pass`,
`probe`,
`translate-lane` and others list in `dependencies`,
and it sits above `adjudication`,
`assembly`,
`archive` and `translate-stage`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Repair entry and chunks:
  the text-in entry point,
  the document driver,
  the per-chunk repair and its cache key.
  Names:
  `repairTranslation`,
  `repairPreparedDocument`,
  `repairChunk`,
  `settleChunkVerdict`,
  `buyRepairSlice`,
  `repairSliceKey`,
  `SLICE_CACHE_VERSION`.
  Declared in `repair-entry.ts`,
  `repair-translation.ts`,
  `repair-chunk.ts`,
  `repair-chunk-verdict.ts`,
  `repair-slice-buy.ts`,
  and `repair-slice-key.ts`.
- Critic and checker stages:
  run the critics,
  file their claims,
  and run the checkers that decide whether an issue is resolved.
  Names:
  `runChunkCriticPhase`,
  `runCriticStage`,
  `runCheckerStage`,
  `attachClaimFilers`,
  `foldStageFindings`,
  `assertCheckerBench`.
  Declared in `chunk-critic-phase.ts`,
  `repair-stages.ts`,
  `repair-edit-stages.ts`,
  `claim-filers.ts`,
  `repair-stage-findings.ts`,
  and `repair-checker-reseat.ts`.
- Editor ensemble:
  several editors rewrite the same chunk,
  every proposal passes the same deterministic apply gate,
  and judges choose what ships,
  per chunk and per envelope.
  Names:
  `runEditorStage`,
  `gatherEditorProposals`,
  `buildEditorMessages`,
  `EDITOR_RESPONSE_FORMAT`,
  `resolveEditorEdits`,
  `buildEditorCandidates`,
  `selectChunkPatch`,
  `selectPerEnvelope`.
  Declared in `repair-editor-stage.ts`,
  `editor-gather.ts`,
  `edit-prompt.ts`,
  `edit-wire.ts`,
  `editor-candidates.ts`,
  and `editor-ensemble.ts`.
- Refine lane:
  a second per-slice phase for naturalness,
  with its own prompt,
  cache key and settlement.
  Names:
  `runRefinePhase`,
  `runRefineStage`,
  `buildRefineMessages`,
  `REFINE_RESPONSE_FORMAT`,
  `resolveRefineRewrites`,
  `refineSliceKey`,
  `settleRefinedSlice`.
  Declared in `refine-phase.ts`,
  `refine-stage.ts`,
  `refine-prompt.ts`,
  `refine-wire.ts`,
  `refine-slice-key.ts`,
  and `refine-slice-settle.ts`.
- Derivability and restoration judging:
  derive omission seeds from a clean translation,
  audit whether the deleted sentences can be derived from the source,
  and judge restoration.
  `sentenceRegionAround` widens a span to the whole sentences it touches.
  Names:
  `deriveOmissionSeeds`,
  `sentenceRegionAround`,
  `runDerivabilityProbe`,
  `buildDerivabilityMessages`,
  `runRestorationJudge`,
  `buildRestorationJudgeMessages`.
  Declared in `derive-seeds.ts`,
  `sentence-region.ts`,
  `derivability-probe.ts`,
  `derivability-wire.ts`,
  `restoration-judge.ts`,
  and `restoration-judge-wire.ts`.
- Fidelity damage and benchmark:
  damage a text in known ways
  (delete a sentence,
  alter a number the original states,
  insert a borrowed sentence),
  choose how much of the original a judge is shown,
  and run the milestone-two repair benchmark and score it.
  Names:
  `deleteOneSentence`,
  `alterSharedNumber`,
  `insertBorrowedSentence`,
  `FIDELITY_DAMAGE_KINDS`,
  `sliceNeighbourContexts`,
  `runRepairBenchmark`,
  `computeRepairScorecard`.
  Declared in `fidelity-damage.ts`,
  `fidelity-window.ts`,
  `repair-benchmark.ts`,
  and `repair-scorecard.ts`.
- Paragraph inspection:
  read a paragraph into the ordered atoms the naturalness gate compares,
  and reject a rewrite that does not keep them.
  Names:
  `inspectParagraph`,
  `gateParagraphRewrite`,
  `ParagraphInspection`.
  Declared in `inspect-paragraph.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `archive`
- `assembly`
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
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `bench`
- `consolidate`
- `corpus-pass`
- `probe`
- `run`
- `slice-cache`
- `translate-lane`

Packages that list it only in `devDependencies`:

- `artifact`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/repair:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/repair:test:unit`,
or build and test together with `mise run //package/translation-repair/repair:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `rendered-sheets-preparation.test-fixture.ts`,
`rendered-sheets-slates.test-fixture.ts`,
and `rendered-sheets-texts.test-fixture.ts`.

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
