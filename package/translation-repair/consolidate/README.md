# @monochromatic-dev/translation-repair-consolidate

Consolidation:
the step that decides,
for a contested slice,
whether a rendering written in this run replaces the standing text.
It floors the slate,
judges it against the standing text,
gates what wins,
wraps what ships,
and runs one fixed naturalness polish round.
It is its own package because the corpus pass runs it
and the translate lane,
the artifact reader and the slice cache read its records and rules,
and it sits above `repair`,
`archive` and `lane-contest`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Settling a consolidation:
  produce the proposals,
  floor the slate,
  judge it and settle what ships.
  Names:
  `settleConsolidation`,
  `produceConsolidations`,
  `buildConsolidateMessages`,
  `renderConsolidationBrief`,
  `floorConsolidateSlate`,
  `wrapConsolidation`.
  Declared in `consolidate-settle.ts`,
  `consolidate-produce.ts`,
  `consolidate-wire.ts`,
  `consolidate-brief.ts`,
  `consolidate-validity-floor.ts`,
  and `consolidate-wrap.ts`.
- Gate:
  decide whether the rendering this run wrote replaces the one that would ship.
  Names:
  `gateConsolidatedSlice`,
  `settleGateBallots`,
  `buildConsolidateGateMessages`,
  `CONSOLIDATE_GATE_QUORUM`,
  `readConsolidateGateBallot`.
  Declared in `consolidate-gate-stage.ts` and `consolidate-gate-wire.ts`.
- Standing text:
  the baseline a consolidation must beat,
  and what happens when the standing text is ineligible.
  Names:
  `standingTextFor`,
  `contestStandingMayShip`,
  `readStandingVerdict`,
  `keepTheArchive`,
  `slateIncumbentFor`,
  `describeStandingVerdict`.
  Declared in `consolidate-standing.ts`,
  `consolidate-standing-verdict.ts`,
  and `consolidate-ineligible-standing.ts`.
- Naturalness polish:
  one fixed polish round:
  refiners propose,
  judges choose between proposal and standing text,
  and the gate applies.
  The judges' sheet shows the claims the repair panel accepted for the slice,
  and a tie between the polished and the standing text ships the polished text
  (owner,
  2026-10-21,
  "Polished text wins ties"),
  provided the minimum ballot count is met.
  Names:
  `polishConsolidation`,
  `configuredConsolidationPolish`,
  `gateConsolidationPolish`,
  `applyFinalPolish`,
  `finalPolishParagraphs`,
  `CONSOLIDATION_POLISH_GATE_QUORUM`.
  Declared in `consolidation-polish.ts`,
  `consolidation-polish-config.ts`,
  `consolidation-polish-gate-stage.ts`,
  `consolidation-polish-apply.ts`,
  and `consolidation-polish-round.ts`.
- Persistence:
  save a settlement,
  tell whether a saved one is worth resuming,
  buy a slice's consolidation in a single attempt,
  and refuse a contested slice the repair ledger never wrote a row for
  or a withheld slate that the judging left undecided.
  Names:
  `persistConsolidationSettlement`,
  `consolidationWorthResuming`,
  `buyConsolidationSlice`,
  `ConsolidationLedgerGapError`,
  `WithheldSlateAbsenceError`.
  Declared in `consolidate-persistence.ts`,
  `consolidate-slice-buy.ts`,
  `consolidation-ledger-gap.ts`,
  and `consolidate-settle-judge.ts`.

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
- `provider`
- `repair`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `markup`

### Used by

Packages that list it in `dependencies`:

- `artifact`
- `corpus-pass`
- `slice-cache`
- `translate-lane`

Packages that list it only in `devDependencies`:

- `page-check`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/consolidate:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/consolidate:test:unit`,
or build and test together with `mise run //package/translation-repair/consolidate:buildAndTest`.
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
