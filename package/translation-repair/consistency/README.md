# @monochromatic-dev/translation-repair-consistency

The page-level consistency pass:
when the same wording appears at several places on a page,
a bench is asked which rendering the page should follow,
the votes are counted deterministically,
ties are run off,
and the chosen words are carried to the other places by code or by a choice among texts.
It is its own package because the pass is a unit of its own,
with its sheets,
counts and mark rules,
and only `artifact` and `corpus-pass` use it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Ballot sheets and wire:
  the sheet a judge reads,
  the response format it must answer in,
  and the checks that make a ballot usable.
  Names:
  `buildConsistencyMessages`,
  `consistencyResponseFormat`,
  `isConsistencyBallotWire`,
  `usableConsistencyBallotFor`,
  `canonicalBallotOf`,
  `consistencyLabels`.
  Declared in `consistency-sheet.ts` and `consistency-wire.ts`.
- Counting ballots:
  count the ballots of a repeated wording deterministically,
  from usable ballots only,
  name the winner,
  and find where a judge's quoted words stand in a place's English.
  Names:
  `countConsistency`,
  `winnerOf`,
  `foldedWordsOf`,
  `locateWords`,
  `majorityChoiceOf`,
  `occurrencesOf`.
  Declared in `consistency-count.ts`,
  `consistency-fold.ts`,
  `consistency-majority.ts`,
  and `consistency-occurrences.ts`.
- Choice among renderings:
  the reply of a judge who only chooses,
  as the run-off among finalists and the choice among a receiving place's candidate texts both ask it:
  its response format,
  the label it names,
  the choices that can be counted,
  the finalists of a split
  and the tally.
  Names:
  `canonicalChoiceOf`,
  `choiceTallyOf`,
  `consistencyChoiceResponseFormat`,
  `usableChoicesOf`,
  `runOffChoiceFinalistsOf`.
  Declared in `consistency-choice.ts`.
- Run-off:
  ask the bench once more when the choices give no strict majority.
  Names:
  `buildConsistencyRunOffMessages`,
  `runOffWinnerOf`.
  Declared in `consistency-run-off.ts`.
- Carrying to a receiving place:
  offer the current text,
  the spliced text and each writer's rewrite of a paragraph,
  and read the bench's choice.
  Names:
  `buildCarryChoiceMessages`,
  `buildCarryWriterMessages`,
  `carryChoiceOf`,
  `carryOffersOf`,
  `carryRunOffFinalistsOf`,
  `blockStandingOf`.
  Declared in `consistency-carry-choice-sheet.ts`,
  `consistency-carry-writer-sheet.ts`,
  `consistency-carry.ts`,
  and `consistency-carry-standing.ts`.
- Splicing by code:
  move the chosen words over another place's words by code,
  without a model writing new text.
  Names:
  `planConsistencyEdit`,
  `applyConsistencyEdits`,
  `ConsistencyPlan`,
  `ConsistencyEdit`,
  `cutsWord`,
  `wholeSpanOf`.
  Declared in `consistency-splice.ts`,
  `consistency-word-joints.ts`,
  and `consistency-whole-end.ts`.
- Page layout and marks:
  the page joined from its slices,
  the prose view of an English,
  and the proof of which single marks are apostrophes,
  so that a splice may move them with their word;
  also the generation of the pass's cached answers and the error for a state its construction rules out.
  Names:
  `pageOfSlices`,
  `pageReadingOf`,
  `proseViewOf`,
  `markReadingsOf`,
  `provenApostrophesOf`,
  `CONSISTENCY_CACHE_VERSION`,
  `ConsistencyInvariantError`.
  Declared in `consistency-page-english.ts`,
  `consistency-prose-text.ts`,
  `consistency-proven-apostrophes.ts`,
  `consistency-cache-version.ts`,
  and `consistency-invariant-error.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `assembly`
- `document-structure`
- `house-rule`
- `markup`
- `page-pass`
- `provider`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `provider-stream`

### Used by

Packages that list it in `dependencies`:

- `artifact`
- `corpus-pass`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/consistency:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/consistency:test:unit`,
or build and test together with `mise run //package/translation-repair/consistency:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `consistency-renderings.test-fixture.ts`.

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
